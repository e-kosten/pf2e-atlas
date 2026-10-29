use super::event_execution::EncounterEvent;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    DamageMultiplierView, PersistentDamageGroupView, PersistentDamageSourceView,
};
use atlas_app_model::{PersistentRecoveryResultView, RecoverPersistentDamageRequest};
use atlas_local_state::{
    ConditionEventChange, ConditionEventOutcome, EncounterEventReceipt, PreparedConditionEvent,
};
use atlas_local_state::{ConditionParameters, DamageMultiplier};

impl AtlasAppService {
    pub fn persistent_damage_review(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        boundary_key: Option<&str>,
    ) -> AppServiceResult<atlas_app_model::PersistentReviewView> {
        let (snapshot, boundary, round, claims, damage_result, recovery_results) = self
            .with_state(|session| {
                let store = session.local_state().encounters();
                let snapshot = store
                    .get_with_participants(encounter_ref)?
                    .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
                let participant = snapshot
                    .participants
                    .iter()
                    .find(|p| p.participant_key == participant_key)
                    .ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "Participant is no longer in this encounter",
                        )
                    })?;
                let boundary = super::outgoing_turns::boundary(
                    &store,
                    &snapshot.encounter.encounter_key,
                    boundary_key,
                    participant_key,
                )?;
                let round = boundary
                    .as_ref()
                    .map_or(snapshot.encounter.round_number, |saved| {
                        saved.snapshot.encounter.round_number
                    });
                let claims = store.boundary_claims(
                    &snapshot.encounter.encounter_key,
                    participant_key,
                    round,
                )?;
                let damage_result = store
                    .boundary_receipt(
                        &snapshot.encounter.encounter_key,
                        &boundary_claim(
                            participant,
                            round,
                            atlas_local_state::EncounterBoundaryKind::PersistentDamage,
                            "",
                        ),
                    )?
                    .map(super::persistent_damage_resolution::damage_view)
                    .transpose()?;
                let mut recovery_results = vec![];
                for claim in &claims {
                    if claim.kind == atlas_local_state::EncounterBoundaryKind::PersistentRecovery
                        && let Some(receipt) =
                            store.boundary_receipt(&snapshot.encounter.encounter_key, claim)?
                    {
                        let mut matching = receipt;
                        matching.outcomes.retain(|outcome|matches!(outcome,ConditionEventOutcome::PersistentRecovery{episode_id,..} if episode_id==&claim.scope_key));
                        recovery_results.push(recovery_view(matching)?);
                    }
                }
                Ok((
                    snapshot,
                    boundary,
                    round,
                    claims,
                    damage_result,
                    recovery_results,
                ))
            })?;
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?;
        let frozen = boundary
            .as_ref()
            .map(|saved| super::outgoing_turns::frozen_participant(saved, participant))
            .transpose()?;
        let active = active_damage_subject(frozen.as_ref().unwrap_or(participant), participant);
        let record_key = participant.record_key.clone();
        let (record, source_conditions, generation) = self
            .with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |session| {
                    let record = record_key
                        .as_deref()
                        .map(|key| {
                            super::hydration::resolve_record_ref(session.retrieval_mut(), key)
                        })
                        .transpose()?;
                    let sources = match &record {
                        Some(record) => super::hydration::source_baseline(
                            &record.source,
                            session.retrieval_mut(),
                            None,
                        )?
                        .map(|baseline| baseline.source_conditions)
                        .unwrap_or_else(
                            super::rules::source_conditions::SourceConditions::unavailable,
                        ),
                        None => Default::default(),
                    };
                    let generation = session.retrieval_mut().artifact_generation()?;
                    Ok((
                        record.map(|record| record.source),
                        sources,
                        Some(generation),
                    ))
                },
            )
            .unwrap_or_else(|_| {
                (
                    None,
                    super::rules::source_conditions::SourceConditions::unavailable(),
                    None,
                )
            });
        let preparation_context = atlas_local_state::PersistentPreparationContext {
            participant_key: participant.participant_key.clone(),
            record_key: participant.record_key.clone(),
            boundary_key: boundary_key.map(str::to_owned),
            artifact_generation: generation,
        };
        let mut prepared_damage = Vec::new();
        let mut prepared_recovery = Vec::new();
        for claim in &claims {
            if !matches!(
                claim.kind,
                atlas_local_state::EncounterBoundaryKind::PreparedPersistentDamage
                    | atlas_local_state::EncounterBoundaryKind::PreparedPersistentRecovery
            ) {
                continue;
            }
            let prefix = format!("{}:", boundary_key.unwrap_or("current"));
            let Some(prepared_episode) = claim.scope_key.strip_prefix(&prefix) else {
                continue;
            };
            let receipt = self.with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .boundary_receipt(&snapshot.encounter.encounter_key, claim)?)
            })?;
            if let Some(receipt) = receipt {
                let id = receipt.request_id.clone();
                if let Ok((groups, _, recovery)) =
                    super::persistent_preparation::prepared(receipt, &preparation_context)
                {
                    if claim.kind
                        == atlas_local_state::EncounterBoundaryKind::PreparedPersistentDamage
                    {
                        prepared_damage.extend(
                            groups
                                .into_iter()
                                .filter(|g| g.episode_id == prepared_episode)
                                .map(|group| atlas_app_model::PreparedPersistentDamageGroupView {
                                    request_id: id.clone(),
                                    group: super::persistent_damage_resolution::outcome_view(group),
                                }),
                        );
                    } else if let Some(recovery) = recovery {
                        prepared_recovery.push(atlas_app_model::PersistentRecoveryResultView {
                            encounter_key: snapshot.encounter.encounter_key.clone(),
                            request_id: id,
                            episode_id: recovery.episode_id,
                            damage_type: recovery.damage_type,
                            dc: Some(recovery.dc),
                            roll: recovery.roll,
                            success: recovery.success,
                        });
                    }
                }
            }
        }
        let recorded = frozen.as_ref().unwrap_or(participant);
        let episodes = super::rules::persistent_damage::participant_groups(recorded)
            .into_iter()
            .map(|group| {
                let recovered = claims.contains(&boundary_claim(
                    recorded,
                    round,
                    atlas_local_state::EncounterBoundaryKind::PersistentRecovery,
                    group.episode_id,
                ));
                let damage_ready = damage_result.as_ref().is_some_and(|result| {
                    result
                        .groups
                        .iter()
                        .any(|result| result.episode_id == group.episode_id)
                });
                let damage_resolution = match super::rules::damage::persistent_protection(
                    record.as_ref(),
                    participant,
                    &source_conditions,
                    group.damage_type,
                ) {
                    Ok(p) => atlas_app_model::PersistentDamageResolutionView::Known {
                        protection: atlas_app_model::PersistentDamageProtectionView {
                            immune: p.immune,
                            weakness: p.weakness,
                            resistance: p.resistance,
                        },
                    },
                    Err(reason) => {
                        atlas_app_model::PersistentDamageResolutionView::Table { reason }
                    }
                };
                atlas_app_model::PersistentEpisodeReviewView {
                    episode_id: group.episode_id.into(),
                    damage_resolution,
                    end_turn_recovery_available: !recovered,
                    end_turn_recovery_unavailable_reason: recovered
                        .then(|| "Recovery already recorded for this turn.".into()),
                    end_turn_roll_available: !recovered && damage_ready,
                    end_turn_roll_unavailable_reason: if recovered {
                        Some("Recovery already recorded for this turn.".into())
                    } else if !damage_ready {
                        Some("Record this turn’s damage before rolling recovery.".into())
                    } else {
                        None
                    },
                }
            })
            .collect();
        Ok(atlas_app_model::PersistentReviewView {
            encounter_revision: snapshot.encounter.revision,
            damage_groups: group_views(&active, participant, record.as_ref(), &source_conditions),
            round_number: round,
            damage_applied: claims.contains(&boundary_claim(
                recorded,
                round,
                atlas_local_state::EncounterBoundaryKind::PersistentDamage,
                "",
            )),
            episodes,
            damage_result,
            recovery_results,
            prepared_damage,
            prepared_recovery,
        })
    }

    pub fn recover_persistent_damage(
        &self,
        encounter_ref: &str,
        request: RecoverPersistentDamageRequest,
    ) -> AppServiceResult<PersistentRecoveryResultView> {
        let intent = serde_json::json!({"action": "persistent_recovery", "request": request});
        self.with_write(|session| {
            let store = session.local_state().encounters();
            let encounter = store
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            if let Some(receipt) = store.condition_event_receipt(
                &encounter.encounter_key,
                &request.request_id,
                &intent,
            )? {
                return recovery_view(receipt);
            }
            let snapshot = store
                .get_with_participants(&encounter.encounter_key)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let participant = snapshot
                .participants
                .iter()
                .find(|p| p.participant_key == request.participant_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Participant is no longer in this encounter")
                })?;
            let current_participant=participant;
            let boundary=super::outgoing_turns::boundary(&store,&encounter.encounter_key,request.boundary_key.as_deref(),&request.participant_key)?;
            let frozen=boundary.as_ref().map(|saved|super::outgoing_turns::frozen_participant(saved,current_participant)).transpose()?;
            let participant=frozen.as_ref().unwrap_or(current_participant);
            let round=boundary.as_ref().map_or(encounter.round_number,|saved|saved.snapshot.encounter.round_number);
            let groups = super::rules::persistent_damage::participant_groups(participant);
            let group = groups
                .iter()
                .find(|group| group.episode_id == request.episode_id)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("This persistent damage episode has ended")
                })?;
            let clearing = request.resolution == atlas_app_model::PersistentRecoveryResolution::Clear;
            let dc = if clearing {
                None
            } else {
            let source = group.selected_source;
            let dc = request.dc_override.or_else(|| super::rules::persistent_damage::recovery_dc(source));
            if dc.is_none() {return Err(AppServiceError::invalid_request("Enter the recovery DC"));}
            if dc.is_some_and(|dc| !(0..=100).contains(&dc)) {return Err(AppServiceError::invalid_request("Recovery DC must be between 0 and 100"));}
            dc
            };
            if !clearing && request.mode == atlas_app_model::PersistentRecoveryMode::EndTurn {
                let damage = store
                    .boundary_receipt(
                        &encounter.encounter_key,
                        &boundary_claim(
                            participant,
                            round,
                            atlas_local_state::EncounterBoundaryKind::PersistentDamage,
                            "",
                        ),
                    )?
                    .map(super::persistent_damage_resolution::damage_view)
                    .transpose()?;
                if !damage.is_some_and(|result| {
                    result
                        .groups
                        .iter()
                        .any(|outcome| outcome.episode_id == request.episode_id)
                }) {
                    return Err(AppServiceError::invalid_request(
                        "Resolve this episode's damage before recovery",
                    ));
                }
            }
            // Explicit recovery is GM-triggered and independent of initiative. Only
            // an end-turn resolution claims that turn's recovery; request receipts
            // still deduplicate retries for either mode.
            let claims = if !clearing && request.mode == atlas_app_model::PersistentRecoveryMode::EndTurn {
                vec![boundary_claim(participant, round, atlas_local_state::EncounterBoundaryKind::PersistentRecovery, group.episode_id)]
            } else {
                vec![]
            };
            let receipt = EncounterEvent {
            condition_sources: &[],
                encounter_key: &encounter.encounter_key, request_id: &request.request_id, intent: &intent,
                expected_revision: request.expected_encounter_revision, reference_revision: None, participant_revision: None,
            }.evaluate_in(session, atlas_local_state::ConditionEventBoundary { claims: &claims, outgoing_boundary_key: request.boundary_key.as_deref() },
                || {
                    let roll = if !clearing && dc.is_some_and(|dc|(2..=20).contains(&dc)) {Some(atlas_dice::roll("1d20")?)} else {None};
                    let result = roll.as_ref().map(|roll| i64::from(roll.total));
                    let success = clearing || dc.is_some_and(|dc|dc<=1 || (dc<=20 && result.is_some_and(|result|result>=dc)));
                    Ok(PreparedConditionEvent {
                        condition_changes: if success {
                            group
                                .sources
                                .iter()
                                .filter(|source|current_participant.conditions.iter().any(|current|current.condition_id==source.condition_id && matches!(&current.parameters,ConditionParameters::PersistentDamage{episode_id,..} if episode_id==group.episode_id)))
                                .map(|source| ConditionEventChange::Remove {
                                    participant_key: participant.participant_key.clone(),
                                    condition_id: source.condition_id,
                                })
                                .collect()
                        } else {
                            vec![]
                        },
                        outcomes: vec![ConditionEventOutcome::PersistentRecovery {
                            episode_id: group.episode_id.into(),
                            damage_type: group.damage_type.into(),
                            dc,
                            roll,
                            success,
                        }],
                        ..Default::default()
                    })
                },
            )?;
            recovery_view(receipt)
        })
    }
}

pub(super) fn boundary_claim(
    participant: &atlas_local_state::EncounterParticipant,
    round_number: i64,
    kind: atlas_local_state::EncounterBoundaryKind,
    scope_key: &str,
) -> atlas_local_state::EncounterBoundaryClaim {
    atlas_local_state::EncounterBoundaryClaim {
        participant_key: participant.participant_key.clone(),
        round_number,
        kind,
        scope_key: scope_key.into(),
    }
}

fn recovery_view(receipt: EncounterEventReceipt) -> AppServiceResult<PersistentRecoveryResultView> {
    let outcome = receipt
        .outcomes
        .into_iter()
        .find_map(|outcome| match outcome {
            ConditionEventOutcome::PersistentRecovery {
                episode_id,
                damage_type,
                dc,
                roll,
                success,
            } => Some(PersistentRecoveryResultView {
                encounter_key: receipt.encounter_key.clone(),
                request_id: receipt.request_id.clone(),
                episode_id,
                damage_type,
                dc,
                roll,
                success,
            }),
            _ => None,
        });
    outcome.ok_or_else(|| AppServiceError::invalid_request("Recovery result is unavailable"))
}

pub(super) fn active_damage_subject(
    recorded: &atlas_local_state::EncounterParticipant,
    current: &atlas_local_state::EncounterParticipant,
) -> atlas_local_state::EncounterParticipant {
    let mut subject = recorded.clone();
    subject.conditions.retain(|source| {
        !matches!(source.parameters, ConditionParameters::PersistentDamage { .. })
            || current.conditions.iter().any(|condition| condition.condition_id == source.condition_id
                && matches!((&source.parameters, &condition.parameters),
                    (ConditionParameters::PersistentDamage {episode_id: old, ..}, ConditionParameters::PersistentDamage {episode_id: live, ..}) if old == live))
    });
    subject
}

pub(super) fn group_views(
    participant: &atlas_local_state::EncounterParticipant,
    protection_subject: &atlas_local_state::EncounterParticipant,
    record: Option<&atlas_record::source_record::SourceBackedRecord>,
    sources: &super::rules::source_conditions::SourceConditions,
) -> Vec<PersistentDamageGroupView> {
    super::rules::persistent_damage::participant_groups(participant)
        .into_iter()
        .map(|group| PersistentDamageGroupView {
            immunity_note: super::rules::damage::persistent_protection(
                record,
                protection_subject,
                sources,
                group.damage_type,
            )
            .is_ok_and(|protection| protection.immune)
            .then(|| format!("Immune to {}", group.damage_type)),
            episode_id: group.episode_id.into(),
            damage_type: group.damage_type.into(),
            selected_source_id: group.selected_source.condition_id,
            sources: group
                .sources
                .into_iter()
                .filter_map(|condition| {
                    let ConditionParameters::PersistentDamage {
                        formula,
                        multiplier,
                        ..
                    } = &condition.parameters
                    else {
                        return None;
                    };
                    Some(PersistentDamageSourceView {
                        condition_id: condition.condition_id,
                        formula: formula.clone(),
                        roll_preview: atlas_dice::prepare(formula)
                            .ok()
                            .and_then(|dice| dice.preview_expression().ok()),
                        multiplier: match multiplier {
                            DamageMultiplier::Normal => DamageMultiplierView::Normal,
                            DamageMultiplier::Double => DamageMultiplierView::Double,
                            DamageMultiplier::Half => DamageMultiplierView::Half,
                        },
                        recovery_dc: super::rules::persistent_damage::recovery_dc(condition),
                    })
                })
                .collect(),
        })
        .collect()
}
