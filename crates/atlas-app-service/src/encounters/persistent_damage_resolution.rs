//! All persistent types at one boundary feed one HP/lifecycle event.
use super::event_execution::EncounterEvent;
use super::rules::hp_events::HpEventInput;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    ApplyPersistentDamageRequest, PersistentDamageInput, PersistentDamageOutcomeView,
    PersistentDamageResultView,
};
use atlas_local_state::{
    ConditionEventOutcome, ConditionParameters, EncounterEventReceipt, PersistentDamageOutcome,
};

impl AtlasAppService {
    pub fn apply_persistent_damage(
        &self,
        encounter_ref: &str,
        request: ApplyPersistentDamageRequest,
    ) -> AppServiceResult<PersistentDamageResultView> {
        let intent = serde_json::json!({"action":"persistent_damage", "request":request});
        let (snapshot, recovered) = self.with_state(|session| {
            let store = session.local_state().encounters();
            let snapshot = store
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let recovered = store.condition_event_receipt(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
            )?;
            Ok((snapshot, recovered))
        })?;
        if let Some(receipt) = recovered {
            return damage_view(receipt);
        }
        let participant = snapshot
            .participants
            .iter()
            .find(|participant| participant.participant_key == request.participant_key)
            .ok_or_else(|| {
                AppServiceError::invalid_request("Participant is no longer in this encounter")
            })?;
        let boundary = self.with_state(|session| {
            super::outgoing_turns::boundary(
                &session.local_state().encounters(),
                &snapshot.encounter.encounter_key,
                request.boundary_key.as_deref(),
                &request.participant_key,
            )
        })?;
        let frozen = boundary
            .as_ref()
            .map(|saved| super::outgoing_turns::frozen_participant(saved, participant))
            .transpose()?;
        let active = super::persistent_damage::active_damage_subject(
            frozen.as_ref().unwrap_or(participant),
            participant,
        );
        let damage_sources = &active;
        let round = boundary
            .as_ref()
            .map_or(snapshot.encounter.round_number, |saved| {
                saved.snapshot.encounter.round_number
            });
        let groups = super::rules::persistent_damage::participant_groups(damage_sources);
        if groups.is_empty() || groups.len() != request.groups.len() {
            return Err(AppServiceError::invalid_request(
                "Resolve every current damage type together",
            ));
        }
        let requested: std::collections::BTreeSet<_> = request
            .groups
            .iter()
            .map(|group| group.episode_id.as_str())
            .collect();
        if requested.len() != request.groups.len()
            || groups
                .iter()
                .any(|group| !requested.contains(group.episode_id))
        {
            return Err(AppServiceError::invalid_request(
                "Persistent damage sources changed; review the groups",
            ));
        }
        let needs_protections = request.groups.iter().any(|group| {
            matches!(
                group.damage,
                PersistentDamageInput::Roll | PersistentDamageInput::Prepared { .. }
            )
        });
        let table_damage = request
            .groups
            .iter()
            .fold(0_i64, |sum, group| match group.damage {
                PersistentDamageInput::TableResult { amount } => sum.saturating_add(amount),
                _ => sum,
            });
        let needs_lifecycle = participant.record_key.is_some()
            && (participant
                .current_hp
                .is_some_and(|hp| table_damage.saturating_sub(participant.temporary_hp) >= hp)
                || participant.conditions.iter().any(|condition| {
                    matches!(
                        super::conditions::condition_rule_for_key(
                            condition.condition_key.as_deref()
                        ),
                        Some(
                            super::conditions::ConditionRule::Unconscious
                                | super::conditions::ConditionRule::Confused
                        )
                    )
                }));
        let (record, source_conditions, reference_revision) = if needs_protections
            || needs_lifecycle
        {
            let record_key = participant.record_key.clone().ok_or_else(|| {
                AppServiceError::invalid_request(
                    "Damage protections are unavailable; enter table results",
                )
            })?;
            let expected_generation = request.artifact_generation.clone();
            self.with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |session| {
                    let generation = session.retrieval_mut().artifact_generation()?;
                    if (needs_protections || expected_generation.is_some())
                        && expected_generation.as_deref() != Some(generation.as_str())
                    {
                        return Err(AppServiceError::invalid_request(
                            "Reference data changed; reload before rolling",
                        ));
                    }
                    let record =
                        super::hydration::resolve_record_ref(session.retrieval_mut(), &record_key)?;
                    let sources = super::hydration::source_baseline(
                        &record.source,
                        session.retrieval_mut(),
                        None,
                    )?
                    .map(|baseline| baseline.source_conditions)
                    .unwrap_or_else(super::rules::source_conditions::SourceConditions::unavailable);
                    Ok((
                        Some(record.source),
                        sources,
                        Some(session.reference_revision()?),
                    ))
                },
            )?
        } else {
            (None, Default::default(), None)
        };
        let preparation_context = atlas_local_state::PersistentPreparationContext {
            participant_key: participant.participant_key.clone(),
            record_key: participant.record_key.clone(),
            boundary_key: request.boundary_key.clone(),
            artifact_generation: request.artifact_generation.clone(),
        };
        let mut recorded = std::collections::BTreeMap::new();
        let mut prepared = vec![];
        for group in &groups {
            let input = request
                .groups
                .iter()
                .find(|input| input.episode_id == group.episode_id)
                .ok_or_else(|| AppServiceError::invalid_request("Choose every damage group"))?;
            let source = group.selected_source;
            let ConditionParameters::PersistentDamage {
                formula,
                multiplier,
                ..
            } = &source.parameters
            else {
                return Err(AppServiceError::invalid_request(
                    "Persistent damage parameters are unavailable",
                ));
            };
            let (roll, protection) = match &input.damage {
                PersistentDamageInput::Roll => (
                    Some(
                        atlas_dice::prepare(formula)
                            .map_err(atlas_local_state::LocalStateError::from)?,
                    ),
                    Some(
                        super::rules::damage::persistent_protection(
                            record.as_ref(),
                            participant,
                            &source_conditions,
                            group.damage_type,
                        )
                        .map_err(AppServiceError::invalid_request)?,
                    ),
                ),
                PersistentDamageInput::Prepared { request_id } => {
                    let receipt = super::persistent_preparation::receipt_for(
                        self,
                        &snapshot.encounter.encounter_key,
                        &preparation_context,
                        round,
                        group.episode_id,
                        request_id,
                        atlas_local_state::EncounterBoundaryKind::PreparedPersistentDamage,
                    )?;
                    let (saved, _, _) =
                        super::persistent_preparation::prepared(receipt, &preparation_context)?;
                    let outcome = saved
                        .into_iter()
                        .find(|saved| {
                            saved.episode_id == group.episode_id
                                && saved.source_condition_id == source.condition_id
                                && !saved.table_resolved
                        })
                        .ok_or_else(|| {
                            AppServiceError::invalid_request("Prepared damage source changed")
                        })?;
                    let (outcome, hardness) = super::persistent_preparation::revalue(
                        outcome,
                        group,
                        record.as_ref(),
                        participant,
                        &source_conditions,
                    )?;
                    let protection = outcome.protection.as_ref().ok_or_else(|| {
                        AppServiceError::invalid_request("Prepared damage protections unavailable")
                    })?;
                    let protection = super::rules::damage::DamageProtection {
                        immune: protection.immune,
                        weakness: protection.weakness,
                        resistance: protection.resistance,
                        hardness,
                    };
                    recorded.insert(group.episode_id.to_owned(), outcome);
                    (None, Some(protection))
                }
                PersistentDamageInput::TableResult { amount } => {
                    if !(0..=1_000_000_000).contains(amount) {
                        return Err(AppServiceError::invalid_request(
                            "Damage must be between 0 and 1000000000",
                        ));
                    }
                    (None, None)
                }
            };
            prepared.push((
                group,
                input,
                source.condition_id,
                *multiplier,
                roll,
                protection,
            ));
        }
        let hardness = prepared
            .iter()
            .filter_map(|(_, _, _, _, _, protection)| {
                protection.as_ref().map(|protection| protection.hardness)
            })
            .max()
            .unwrap_or(0);
        if hardness > 0
            && request
                .groups
                .iter()
                .any(|group| matches!(group.damage, PersistentDamageInput::TableResult { .. }))
        {
            return Err(AppServiceError::invalid_request(
                "For Hardness, roll all types or enter all table totals",
            ));
        }
        let possible_damage = request
            .groups
            .iter()
            .try_fold(0_i64, |total, group| match group.damage {
                PersistentDamageInput::TableResult { amount } => total.checked_add(amount),
                PersistentDamageInput::Prepared { .. } => recorded
                    .get(&group.episode_id)
                    .and_then(|saved| total.checked_add(saved.amount)),
                PersistentDamageInput::Roll => None,
            })
            .unwrap_or(i64::MAX);
        let subject = super::hp_events::hp_subject_context(
            self,
            participant,
            &HpEventInput::ResolvedDamage {
                amount: possible_damage,
            },
        )?;
        let context = super::hp_events::hp_context_for_subject(request.context.clone(), &subject);
        if reference_revision.is_some()
            && subject.reference_revision.is_some()
            && reference_revision != subject.reference_revision
        {
            return Err(AppServiceError::invalid_request(
                "Reference data changed; reload before rolling",
            ));
        }
        if participant.current_hp.is_none() {
            return Err(AppServiceError::invalid_request(
                "Set current HP before recording this event",
            ));
        }
        self.with_write(|session| {
            let store = session.local_state().encounters();
            if let Some(receipt) = store.condition_event_receipt(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
            )? {
                return damage_view(receipt);
            }
            let mut recovery_ids = std::collections::BTreeSet::new();
            for input in &request.recoveries {
                let episode_id = match input {
                    atlas_app_model::PersistentRecoveryInput::Clear { episode_id }
                    | atlas_app_model::PersistentRecoveryInput::Prepared { episode_id, .. } => {
                        episode_id
                    }
                };
                if !recovery_ids.insert(episode_id.as_str()) {
                    return Err(AppServiceError::invalid_request(
                        "Choose one recovery outcome per damage type",
                    ));
                }
                if !groups.iter().any(|group| group.episode_id == episode_id) {
                    return Err(AppServiceError::invalid_request("Recovery source changed"));
                }
            }
            let mut recovery_outcomes = Vec::new();
            for group in &groups {
                let input = request.recoveries.iter().find(|input| match input {
                    atlas_app_model::PersistentRecoveryInput::Clear { episode_id }
                    | atlas_app_model::PersistentRecoveryInput::Prepared { episode_id, .. } => {
                        episode_id == group.episode_id
                    }
                });
                // Preparation does not advance the encounter revision. Its claims
                // and results must be read under the final write's lock.
                let receipt = store.boundary_receipt(
                    &snapshot.encounter.encounter_key,
                    &super::persistent_damage::boundary_claim(
                        participant,
                        round,
                        atlas_local_state::EncounterBoundaryKind::PreparedPersistentRecovery,
                        &super::persistent_preparation::scope(
                            request.boundary_key.as_deref(),
                            group.episode_id,
                        ),
                    ),
                )?;
                let Some(input) = input else {
                    if receipt.is_some() {
                        return Err(AppServiceError::invalid_request(
                            "Review the prepared recovery before applying damage",
                        ));
                    }
                    continue;
                };
                let clearing = matches!(
                    input,
                    atlas_app_model::PersistentRecoveryInput::Clear { .. }
                );
                if let atlas_app_model::PersistentRecoveryInput::Prepared { request_id, .. } = input
                    && receipt
                        .as_ref()
                        .is_none_or(|receipt| receipt.request_id != *request_id)
                {
                    return Err(AppServiceError::invalid_request(
                        "Prepared result is unavailable; review this damage type",
                    ));
                }
                let result = if let Some(receipt) = receipt {
                    let (_, _, recovery) =
                        super::persistent_preparation::prepared(receipt, &preparation_context)?;
                    let recovery = recovery
                        .filter(|recovery| recovery.episode_id == group.episode_id)
                        .ok_or_else(|| {
                            AppServiceError::invalid_request("Recovery result is unavailable")
                        })?;
                    (Some(recovery.dc), recovery.roll, recovery.success)
                } else {
                    (None, None, true)
                };
                let remove = clearing || result.2;
                recovery_outcomes.push((group, remove, result));
            }
            let claim = super::persistent_damage::boundary_claim(
                participant,
                round,
                atlas_local_state::EncounterBoundaryKind::PersistentDamage,
                "",
            );
            let mut claims = vec![claim];
            claims.extend(recovery_outcomes.iter().map(|(group, _, _)| {
                super::persistent_damage::boundary_claim(
                    participant,
                    round,
                    atlas_local_state::EncounterBoundaryKind::PersistentRecovery,
                    group.episode_id,
                )
            }));
            let receipt =
                EncounterEvent {
                    encounter_key: &snapshot.encounter.encounter_key,
                    request_id: &request.request_id,
                    intent: &intent,
                    expected_revision: request.expected_encounter_revision,
                    reference_revision: reference_revision.or(subject.reference_revision),
                    participant_revision: None,
                }
                .evaluate_in(
                    session,
                    atlas_local_state::ConditionEventBoundary {
                        claims: &claims,
                        outgoing_boundary_key: request.boundary_key.as_deref(),
                    },
                    || {
                        let mut outcomes = vec![];
                        let mut total = 0i64;
                        for (group, input, selected, multiplier, roll, protection) in prepared {
                            if let Some(saved) = recorded.remove(group.episode_id) {
                                total = total.checked_add(saved.amount).ok_or_else(|| {
                                    atlas_local_state::LocalStateError::InvalidRollRequest(
                                        "Damage total is too large",
                                    )
                                })?;
                                outcomes.push(saved);
                                continue;
                            }
                            let rolled = roll
                                .map(|roll| roll.roll())
                                .transpose()
                                .map_err(atlas_local_state::LocalStateError::from)?;
                            let amount = match &input.damage {
                                PersistentDamageInput::Prepared { .. } => {
                                    return Err(
                                        atlas_local_state::LocalStateError::InvalidRollRequest(
                                            "Prepared damage is unavailable",
                                        ),
                                    );
                                }
                                PersistentDamageInput::TableResult { amount } => *amount,
                                PersistentDamageInput::Roll => {
                                    let roll = rolled.as_ref().ok_or_else(|| {
                                        atlas_local_state::LocalStateError::InvalidRollRequest(
                                            "Damage roll unavailable",
                                        )
                                    })?;
                                    let protection = protection.ok_or_else(|| {
                                        atlas_local_state::LocalStateError::InvalidRollRequest(
                                            "Damage protections unavailable",
                                        )
                                    })?;
                                    protection.apply(super::rules::persistent_damage::scaled(
                                        i64::from(roll.total),
                                        multiplier,
                                    ))
                                }
                            };
                            total = total.checked_add(amount).ok_or_else(|| {
                                atlas_local_state::LocalStateError::InvalidRollRequest(
                                    "Damage total is too large",
                                )
                            })?;
                            outcomes.push(PersistentDamageOutcome {
                                episode_id: group.episode_id.into(),
                                damage_type: group.damage_type.into(),
                                source_condition_id: selected,
                                roll: rolled,
                                amount,
                                table_resolved: matches!(
                                    input.damage,
                                    PersistentDamageInput::TableResult { .. }
                                ),
                                protection: protection.map(|p| {
                                    atlas_local_state::PersistentDamageProtection {
                                        immune: p.immune,
                                        weakness: p.weakness,
                                        resistance: p.resistance,
                                    }
                                }),
                            });
                        }
                        let hardness_absorbed = total.min(hardness);
                        total -= hardness_absorbed;
                        let mut changes = super::rules::hp_events::plan_hp_event(
                            participant,
                            &source_conditions,
                            &HpEventInput::ResolvedDamage { amount: total },
                            &context,
                            participant.max_hp,
                            &request.request_id,
                        )
                        .map_err(|_| {
                            atlas_local_state::LocalStateError::InvalidRollRequest(
                                "Persistent damage could not be applied",
                            )
                        })?;
                        for (group, remove, (dc, roll, success)) in recovery_outcomes {
                            if remove {
                                changes.condition_changes.extend(group.sources.iter().map(
                                    |source| atlas_local_state::ConditionEventChange::Remove {
                                        participant_key: participant.participant_key.clone(),
                                        condition_id: source.condition_id,
                                    },
                                ));
                            }
                            changes
                                .outcomes
                                .push(ConditionEventOutcome::PersistentRecovery {
                                    episode_id: group.episode_id.into(),
                                    damage_type: group.damage_type.into(),
                                    dc,
                                    roll,
                                    success,
                                });
                        }
                        changes
                            .outcomes
                            .push(ConditionEventOutcome::PersistentDamage {
                                participant_key: participant.participant_key.clone(),
                                groups: outcomes,
                                total,
                                hardness_absorbed,
                            });
                        Ok(changes)
                    },
                )?;
            damage_view(receipt)
        })
    }
}

pub(super) fn damage_view(
    receipt: EncounterEventReceipt,
) -> AppServiceResult<PersistentDamageResultView> {
    let (groups, total, hardness_absorbed) = receipt
        .outcomes
        .into_iter()
        .find_map(|outcome| match outcome {
            ConditionEventOutcome::PersistentDamage {
                groups,
                total,
                hardness_absorbed,
                ..
            } => Some((
                groups.into_iter().map(outcome_view).collect(),
                total,
                hardness_absorbed,
            )),
            _ => None,
        })
        .ok_or_else(|| AppServiceError::invalid_request("Damage receipt is unavailable"))?;
    Ok(PersistentDamageResultView {
        encounter_key: receipt.encounter_key,
        request_id: receipt.request_id,
        groups,
        total,
        hardness_absorbed,
    })
}

pub(super) fn outcome_view(
    group: atlas_local_state::PersistentDamageOutcome,
) -> PersistentDamageOutcomeView {
    PersistentDamageOutcomeView {
        episode_id: group.episode_id,
        damage_type: group.damage_type,
        source_condition_id: group.source_condition_id,
        roll: group.roll,
        amount: group.amount,
        table_resolved: group.table_resolved,
        protection: group
            .protection
            .map(|p| atlas_app_model::PersistentDamageProtectionView {
                immune: p.immune,
                weakness: p.weakness,
                resistance: p.resistance,
            }),
    }
}
