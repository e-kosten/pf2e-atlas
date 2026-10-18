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
        let store = self.local_state_store()?;
        let snapshot = store
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        options(self, &snapshot)
    }

    pub fn roll_encounter_initiative(
        &self,
        encounter_ref: &str,
        request: RollInitiativeRequest,
    ) -> AppServiceResult<InitiativeRollView> {
        let store = self.local_state_store()?;
        let encounter = store
            .encounters()
            .get(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        let key = &encounter.encounter_key;
        let intent = serde_json::to_value(&request)
            .map_err(|e| AppServiceError::invalid_request(e.to_string()))?;
        if let Some(receipt) =
            store
                .encounters()
                .initiative_roll_receipt(key, &request.request_id, &intent)?
        {
            return Ok(receipt_view(receipt));
        }
        if request.participants.is_empty() || request.participants.len() > 200 {
            return Err(AppServiceError::invalid_request(
                "Select between 1 and 200 participants",
            ));
        }
        let snapshot = store
            .encounters()
            .get_with_participants(key)?
            .ok_or_else(|| super::projection::encounter_not_found(key))?;
        let needs_source = request
            .participants
            .iter()
            .any(|p| p.modifier_override.is_none());
        if snapshot.encounter.revision != request.expected_revision {
            return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                encounter_key: key.clone(),
                expected_revision: request.expected_revision,
                actual_revision: snapshot.encounter.revision,
            }
            .into());
        }
        let choices = if needs_source {
            let current = options(self, &snapshot)?;
            if current.artifact_generation != request.artifact_generation {
                return Err(AppServiceError::artifact_generation_changed(
                    request.artifact_generation.as_deref().unwrap_or(""),
                    current.artifact_generation.as_deref().unwrap_or(""),
                    atlas_app_model::AppRecoverableAction::ReloadRecord,
                ));
            }
            Some(current)
        } else {
            None
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
        Ok(receipt_view(store.encounters().roll_initiative(
            key,
            &request.request_id,
            &intent,
            request.expected_revision,
            &resolved,
        )?))
    }
}

fn options(
    service: &AtlasAppService,
    snapshot: &EncounterWithParticipants,
) -> AppServiceResult<InitiativeOptionsView> {
    let (artifact_generation, records) =
        super::hydration::hydrate_participant_records(service, &snapshot.participants)?;
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
                        super::mechanics::participant_stat_block(
                            p,
                            &baseline,
                            r.adjustment_applicable,
                        )
                    })
                })
                .map(|block| {
                    block
                        .values
                        .into_iter()
                        .filter(|v| {
                            v.target == "perception"
                                || v.target.starts_with("skill.")
                                || v.target == "hazard.stealth"
                        })
                        .filter_map(|v| {
                            Some(InitiativeStatisticView {
                                key: v.target,
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
    Ok(InitiativeOptionsView {
        artifact_generation,
        encounter_key: snapshot.encounter.encounter_key.clone(),
        expected_revision: snapshot.encounter.revision,
        participants,
    })
}

// PF2e NPC.prepareDerivedData: an unlisted skill uses its ability modifier.
// Add it before variant/condition overlays so its effective value follows the
// same skill policy as listed skills. This baseline is not applied to PCs/hazards.
fn add_unlisted_npc_skills(block: &mut super::stat_block::StatBlockView) {
    for (key, label, ability) in [
        ("acrobatics", "Acrobatics", "dex"),
        ("arcana", "Arcana", "int"),
        ("athletics", "Athletics", "str"),
        ("crafting", "Crafting", "int"),
        ("deception", "Deception", "cha"),
        ("diplomacy", "Diplomacy", "cha"),
        ("intimidation", "Intimidation", "cha"),
        ("medicine", "Medicine", "wis"),
        ("nature", "Nature", "wis"),
        ("occultism", "Occultism", "int"),
        ("performance", "Performance", "cha"),
        ("religion", "Religion", "wis"),
        ("society", "Society", "int"),
        ("stealth", "Stealth", "dex"),
        ("survival", "Survival", "wis"),
        ("thievery", "Thievery", "dex"),
    ] {
        let target = format!("skill.{key}");
        if block.values.iter().any(|v| v.target == target) {
            continue;
        }
        let Some(value) = block
            .values
            .iter()
            .find(|v| v.target == format!("ability.{ability}"))
            .map(|v| v.base_value.clone())
        else {
            continue;
        };
        block.values.push(super::stat_block::StatValueView {
            target,
            label: label.into(),
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
