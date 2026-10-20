//! All persistent types at one boundary feed one HP/lifecycle event.
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    ApplyPersistentDamageRequest, HpEventInputView, PersistentDamageInput,
    PersistentDamageOutcomeView, PersistentDamageResultView,
};
use atlas_local_state::{
    ConditionEventOutcome, ConditionParameters, EncounterEventReceipt, PersistentDamageOutcome,
    PersistentSelectionChange, PersistentSourceSelection,
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
        let damage_sources = frozen.as_ref().unwrap_or(participant);
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
        let needs_source = request
            .groups
            .iter()
            .any(|group| matches!(group.damage, PersistentDamageInput::Roll));
        let (record, reference_revision) = if needs_source {
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
                    if expected_generation.as_deref() != Some(generation.as_str()) {
                        return Err(AppServiceError::invalid_request(
                            "Reference data changed; reload before rolling",
                        ));
                    }
                    let record =
                        super::hydration::resolve_record_ref(session.retrieval_mut(), &record_key)?;
                    Ok((Some(record.source), Some(session.reference_revision()?)))
                },
            )?
        } else {
            (None, None)
        };
        let mut prepared = vec![];
        for group in &groups {
            let input = request
                .groups
                .iter()
                .find(|input| input.episode_id == group.episode_id)
                .ok_or_else(|| AppServiceError::invalid_request("Choose every damage group"))?;
            let selected = input
                .selected_source_id
                .or(group.selected_source_id)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Choose the greater damage source")
                })?;
            if group.selected_source_id.is_some_and(|id| id != selected) {
                return Err(AppServiceError::invalid_request(
                    "Use the active damage source",
                ));
            }
            let source = group
                .sources
                .iter()
                .find(|source| source.condition_id == selected)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Choose a source in this damage group")
                })?;
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
                            group.damage_type,
                        )
                        .map_err(AppServiceError::invalid_request)?,
                    ),
                ),
                PersistentDamageInput::TableResult { amount } => {
                    if !(0..=1_000_000_000).contains(amount) {
                        return Err(AppServiceError::invalid_request(
                            "Damage must be between 0 and 1000000000",
                        ));
                    }
                    (None, None)
                }
            };
            prepared.push((group, input, selected, *multiplier, roll, protection));
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
                PersistentDamageInput::Roll => None,
            })
            .unwrap_or(i64::MAX);
        let subject = super::hp_events::hp_subject_context(
            self,
            encounter_ref,
            participant,
            &HpEventInputView::ResolvedDamage {
                amount: possible_damage,
            },
        )?;
        if subject.participant.revision != participant.revision {
            return Err(AppServiceError::invalid_request(
                "Participant changed; review the damage",
            ));
        }
        let context = super::hp_events::hp_context_for_subject(request.context.clone(), &subject);
        self.with_write(|session| {
            let store = session.local_state().encounters();
            if let Some(receipt) = store.condition_event_receipt(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
            )? {
                return damage_view(receipt);
            }
            if let Some(revision) = reference_revision {
                session.require_reference_revision(revision)?;
            }
            if let Some(revision) = subject.reference_revision {
                session.require_reference_revision(revision)?;
            }
            let current = store
                .get(&snapshot.encounter.encounter_key)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            if current.revision != request.expected_encounter_revision
                || snapshot.encounter.revision != request.expected_encounter_revision
            {
                return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                    encounter_key: current.encounter_key,
                    expected_revision: request.expected_encounter_revision,
                    actual_revision: current.revision,
                }
                .into());
            }
            let claim = super::persistent_damage::boundary_claim(
                participant,
                round,
                atlas_local_state::EncounterBoundaryKind::PersistentDamage,
                "",
            );
            if store.boundary_claimed(&current.encounter_key, &claim)? {
                return Err(atlas_local_state::LocalStateError::EncounterBoundaryConflict.into());
            }
            let mut outcomes = vec![];
            let mut selections = vec![];
            let mut total = 0i64;
            for (group, input, selected, multiplier, roll, protection) in prepared {
                let rolled = roll
                    .map(|roll| roll.roll())
                    .transpose()
                    .map_err(atlas_local_state::LocalStateError::from)?;
                let amount = match &input.damage {
                    PersistentDamageInput::TableResult { amount } => *amount,
                    PersistentDamageInput::Roll => {
                        let roll = rolled.as_ref().ok_or_else(|| {
                            AppServiceError::invalid_request("Damage roll unavailable")
                        })?;
                        let protection = protection.ok_or_else(|| {
                            AppServiceError::invalid_request("Damage protections unavailable")
                        })?;
                        protection.apply(super::rules::persistent_damage::scaled(
                            i64::from(roll.total),
                            multiplier,
                        ))
                    }
                };
                total = total
                    .checked_add(amount)
                    .ok_or_else(|| AppServiceError::invalid_request("Damage total is too large"))?;
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
                });
                selections.push(PersistentSelectionChange {
                    participant_key: participant.participant_key.clone(),
                    selection: PersistentSourceSelection {
                        episode_id: group.episode_id.into(),
                        selected_source_id: selected,
                        source_condition_ids: group
                            .sources
                            .iter()
                            .map(|source| source.condition_id)
                            .collect(),
                    },
                });
            }
            let hardness_absorbed = total.min(hardness);
            total -= hardness_absorbed;
            let mut changes = super::rules::hp_events::plan_hp_event(
                participant,
                &HpEventInputView::ResolvedDamage { amount: total },
                &context,
                participant.max_hp,
                &request.request_id,
            )?;
            changes.selection_changes = selections;
            changes
                .outcomes
                .push(ConditionEventOutcome::PersistentDamage {
                    participant_key: participant.participant_key.clone(),
                    groups: outcomes,
                    total,
                    hardness_absorbed,
                });
            let receipt = store.apply_claimed_condition_event(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                atlas_local_state::ConditionEventBoundary {
                    claims: &[claim],
                    outgoing_boundary_key: request.boundary_key.as_deref(),
                },
                changes,
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
                groups
                    .into_iter()
                    .map(|group| PersistentDamageOutcomeView {
                        episode_id: group.episode_id,
                        damage_type: group.damage_type,
                        source_condition_id: group.source_condition_id,
                        roll: group.roll,
                        amount: group.amount,
                        table_resolved: group.table_resolved,
                    })
                    .collect(),
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
