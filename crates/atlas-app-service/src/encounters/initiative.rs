use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    InitiativeOptionsView, InitiativeParticipantOptionsView, InitiativeRollView,
    InitiativeStatisticView, ParticipantRollView, RollInitiativeRequest, RollMode,
};
use atlas_local_state::{EncounterWithParticipants, InitiativeRollReceipt, ResolvedInitiativeRoll};

impl AtlasAppService {
    pub fn encounter_initiative_options(
        &self,
        encounter_ref: &str,
    ) -> AppServiceResult<InitiativeOptionsView> {
        let initial = self.with_state(|session| {
            session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))
        })?;
        if !initial.participants.iter().any(|p| {
            p.record_key
                .as_deref()
                .is_some_and(|key| atlas_domain::RecordKey::parse(key).is_ok())
        }) {
            return Ok(options_with_records(&initial, None, Default::default()));
        }
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let snapshot = session
                    .local_state()
                    .encounters()
                    .get_with_participants(&encounter_ref)?
                    .ok_or_else(|| super::projection::encounter_not_found(&encounter_ref))?;
                options(session.retrieval_mut(), &snapshot)
            },
        )
    }

    pub fn roll_encounter_initiative(
        &self,
        encounter_ref: &str,
        request: RollInitiativeRequest,
    ) -> AppServiceResult<InitiativeRollView> {
        let intent = serde_json::to_value(&request)
            .map_err(|e| AppServiceError::invalid_request(e.to_string()))?;
        let (encounter, recovered) = self.with_state(|session| {
            let encounters = session.local_state().encounters();
            let encounter = encounters
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let recovered = encounters.initiative_roll_receipt(
                &encounter.encounter_key,
                &request.request_id,
                &intent,
            )?;
            Ok((encounter, recovered))
        })?;
        if let Some(receipt) = recovered {
            return Ok(receipt_view(receipt));
        }
        let key = &encounter.encounter_key;
        if request.participants.is_empty() || request.participants.len() > 200 {
            return Err(AppServiceError::invalid_request(
                "Select between 1 and 200 participants",
            ));
        }
        let needs_source = request
            .participants
            .iter()
            .any(|p| p.modifier_override.is_none());
        let (choices, reference_revision) = if needs_source {
            let key = key.clone();
            let (current, revision) = self.with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |session| {
                    let snapshot = session
                        .local_state()
                        .encounters()
                        .get_with_participants(&key)?
                        .ok_or_else(|| super::projection::encounter_not_found(&key))?;
                    let choices = options(session.retrieval_mut(), &snapshot)?;
                    Ok((choices, session.reference_revision()?))
                },
            )?;
            if current.artifact_generation != request.artifact_generation {
                return Err(AppServiceError::artifact_generation_changed(
                    request.artifact_generation.as_deref().unwrap_or(""),
                    current.artifact_generation.as_deref().unwrap_or(""),
                    atlas_app_model::AppRecoverableAction::ReloadRecord,
                ));
            }
            (Some(current), Some(revision))
        } else {
            (None, None)
        };
        let mut resolved = Vec::with_capacity(request.participants.len());
        for input in &request.participants {
            let statistic = choices
                .as_ref()
                .and_then(|c| {
                    c.participants
                        .iter()
                        .find(|p| p.participant_key == input.participant_key)
                })
                .and_then(|p| p.statistics.iter().find(|s| s.key == input.statistic));
            let modifier = input
                .modifier_override
                .or_else(|| statistic.map(|s| s.modifier))
                .ok_or_else(|| {
                    AppServiceError::invalid_request(
                        "This initiative modifier is unavailable; enter a modifier",
                    )
                })?;
            if !(-10000..=10000).contains(&modifier) || input.statistic.len() > 128 {
                return Err(AppServiceError::invalid_request(
                    "Initiative modifier or statistic exceeds supported bounds",
                ));
            }
            let dice = match input.mode {
                RollMode::Normal => "1d20",
                RollMode::Fortune => "2d20kh1",
                RollMode::Misfortune => "2d20kl1",
            };
            let basis = if input.modifier_override.is_some() {
                "Manual modifier".into()
            } else {
                statistic.map(|s| s.label.clone()).ok_or_else(|| {
                    AppServiceError::invalid_request("Initiative statistic is unavailable")
                })?
            };
            resolved.push(ResolvedInitiativeRoll {
                participant_key: input.participant_key.clone(),
                expected_revision: input.expected_revision,
                basis,
                formula: format!("{dice}{modifier:+}"),
            });
        }
        self.with_write(|session| {
            let encounters = session.local_state().encounters();
            if let Some(receipt) =
                encounters.initiative_roll_receipt(key, &request.request_id, &intent)?
            {
                return Ok(receipt_view(receipt));
            }
            if let Some(revision) = reference_revision {
                session.require_reference_revision(revision)?;
            }
            Ok(receipt_view(encounters.roll_initiative(
                key,
                &request.request_id,
                &intent,
                request.expected_revision,
                &resolved,
            )?))
        })
    }
}

fn options(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    snapshot: &EncounterWithParticipants,
) -> AppServiceResult<InitiativeOptionsView> {
    let (artifact_generation, records) =
        super::hydration::hydrate_participant_records(retrieval, &snapshot.participants)?;
    Ok(options_with_records(snapshot, artifact_generation, records))
}

fn options_with_records(
    snapshot: &EncounterWithParticipants,
    artifact_generation: Option<String>,
    records: std::collections::BTreeMap<String, super::hydration::HydratedParticipantRecord>,
) -> InitiativeOptionsView {
    let participants = snapshot
        .participants
        .iter()
        .map(|p| {
            let record = p.record_key.as_ref().and_then(|key| records.get(key));
            let statistics = record
                .and_then(|r| {
                    r.arithmetic_baseline.as_ref().map(|b| {
                        let mut baseline = b.clone();
                        let source = &r.detail.source;
                        if atlas_record::source_record::SourceQueryView::new(
                            source.source(),
                            source.key().pack().as_str(),
                            "",
                        )
                        .source
                        .source_type()
                        .value()
                            == Some("npc")
                        {
                            add_unlisted_npc_skills(&mut baseline);
                        }
                        super::rules::project_participant(
                            p,
                            &baseline,
                            r.adjustment_applicable,
                            &[],
                        )
                    })
                })
                .map(|block| {
                    let block = super::rules::project_initiative(block, p);
                    block
                        .values
                        .into_iter()
                        .filter(|v| {
                            v.target == super::stat_block::StatisticTarget::Perception
                                || v.target.skill()
                                || v.target == super::stat_block::StatisticTarget::HazardStealth
                        })
                        .filter(|v| !block.unresolved_stats.contains(&v.target))
                        .filter_map(|v| {
                            Some(InitiativeStatisticView {
                                key: v.target.key().into_owned(),
                                label: v.label,
                                modifier: i32::try_from(super::mechanics::integer(
                                    &v.adjusted_value,
                                )?)
                                .ok()?,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            InitiativeParticipantOptionsView {
                participant_key: p.participant_key.clone(),
                expected_revision: p.revision,
                statistics,
            }
        })
        .collect();
    InitiativeOptionsView {
        artifact_generation,
        encounter_key: snapshot.encounter.encounter_key.clone(),
        expected_revision: snapshot.encounter.revision,
        participants,
    }
}

// PF2e NPC.prepareDerivedData: an unlisted skill uses its ability modifier.
// Add it before variant/condition overlays so its effective value follows the
// same skill policy as listed skills. This baseline is not applied to PCs/hazards.
fn add_unlisted_npc_skills(block: &mut super::stat_block::StatBlockView) {
    for (key, label, ability) in [
        (
            "acrobatics",
            "Acrobatics",
            super::stat_block::Ability::Dexterity,
        ),
        ("arcana", "Arcana", super::stat_block::Ability::Intelligence),
        (
            "athletics",
            "Athletics",
            super::stat_block::Ability::Strength,
        ),
        (
            "crafting",
            "Crafting",
            super::stat_block::Ability::Intelligence,
        ),
        (
            "deception",
            "Deception",
            super::stat_block::Ability::Charisma,
        ),
        (
            "diplomacy",
            "Diplomacy",
            super::stat_block::Ability::Charisma,
        ),
        (
            "intimidation",
            "Intimidation",
            super::stat_block::Ability::Charisma,
        ),
        ("medicine", "Medicine", super::stat_block::Ability::Wisdom),
        ("nature", "Nature", super::stat_block::Ability::Wisdom),
        (
            "occultism",
            "Occultism",
            super::stat_block::Ability::Intelligence,
        ),
        (
            "performance",
            "Performance",
            super::stat_block::Ability::Charisma,
        ),
        ("religion", "Religion", super::stat_block::Ability::Wisdom),
        (
            "society",
            "Society",
            super::stat_block::Ability::Intelligence,
        ),
        ("stealth", "Stealth", super::stat_block::Ability::Dexterity),
        ("survival", "Survival", super::stat_block::Ability::Wisdom),
        (
            "thievery",
            "Thievery",
            super::stat_block::Ability::Dexterity,
        ),
    ] {
        let target = super::stat_block::StatisticTarget::Skill(key.into());
        if block.values.iter().any(|v| v.target == target) {
            continue;
        }
        let Some(value) = block
            .values
            .iter()
            .find(|v| v.target == super::stat_block::StatisticTarget::Ability(ability))
            .map(|v| v.base_value.clone())
        else {
            continue;
        };
        block.values.push(super::stat_block::StatValueView {
            notes: vec![],
            target,
            label: label.into(),
            ability: Some(ability),
            base_value: value.clone(),
            adjusted_value: value,
            modifiers: vec![],
            suppressed_modifiers: vec![],
        });
    }
}

fn receipt_view(receipt: InitiativeRollReceipt) -> InitiativeRollView {
    InitiativeRollView {
        request_id: receipt.request_id,
        encounter_key: receipt.encounter_key,
        rolls: receipt
            .rolls
            .into_iter()
            .map(|r| ParticipantRollView {
                participant_key: r.participant_key,
                display_name: r.display_name,
                basis: r.basis,
                outcome: r.outcome,
            })
            .collect(),
    }
}
