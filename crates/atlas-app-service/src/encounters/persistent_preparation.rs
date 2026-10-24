//! Source-bound preparation receipts; no HP or condition changes before Apply.
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    PersistentDamageInput, PersistentDamageResultView, PersistentRecoveryResultView,
    PreparePersistentDamageRequest, PreparedPersistentDamageView,
};
use atlas_local_state::{
    ConditionEventOutcome, EncounterBoundaryKind, EncounterEventReceipt, PersistentDamageOutcome,
    PersistentPreparationContext, PreparedPersistentRecovery,
};

pub(super) fn prepared(
    receipt: EncounterEventReceipt,
    context: &PersistentPreparationContext,
) -> AppServiceResult<(
    Vec<PersistentDamageOutcome>,
    i64,
    Option<PreparedPersistentRecovery>,
)> {
    receipt
        .outcomes
        .into_iter()
        .find_map(|outcome| match outcome {
            ConditionEventOutcome::PersistentPrepared {
                context: saved,
                groups,
                hardness,
                recovery,
                ..
            } if saved.participant_key == context.participant_key
                && saved.record_key == context.record_key
                && saved.boundary_key == context.boundary_key
                && (saved.artifact_generation == context.artifact_generation
                    || recovery.is_some()
                    || (saved.artifact_generation.is_none()
                        && groups.iter().all(|g| g.table_resolved))) =>
            {
                Some((groups, hardness, recovery))
            }
            _ => None,
        })
        .ok_or_else(|| {
            AppServiceError::invalid_request("Encounter changed; review the prepared results")
        })
}

pub(super) fn scope(boundary_key: Option<&str>, episode: &str) -> String {
    format!("{}:{episode}", boundary_key.unwrap_or("current"))
}

pub(super) fn receipt_for(
    service: &AtlasAppService,
    key: &str,
    context: &PersistentPreparationContext,
    round: i64,
    episode: &str,
    request_id: &str,
    kind: EncounterBoundaryKind,
) -> AppServiceResult<EncounterEventReceipt> {
    service.with_state(|session| {
        session
            .local_state()
            .encounters()
            .boundary_receipt(
                key,
                &atlas_local_state::EncounterBoundaryClaim {
                    participant_key: context.participant_key.clone(),
                    round_number: round,
                    kind,
                    scope_key: scope(context.boundary_key.as_deref(), episode),
                },
            )?
            .filter(|receipt| receipt.request_id == request_id)
            .ok_or_else(|| {
                AppServiceError::invalid_request(
                    "Prepared result is unavailable; review this damage type",
                )
            })
    })
}

pub(super) fn total(
    groups: &[PersistentDamageOutcome],
    hardness: i64,
) -> AppServiceResult<(i64, i64)> {
    if hardness > 0
        && groups.iter().any(|g| g.table_resolved)
        && groups.iter().any(|g| !g.table_resolved)
    {
        return Err(AppServiceError::invalid_request(
            "For Hardness, roll all types or enter all table totals",
        ));
    }
    let subtotal = groups
        .iter()
        .try_fold(0i64, |sum, g| sum.checked_add(g.amount))
        .ok_or_else(|| AppServiceError::invalid_request("Damage total is too large"))?;
    let absorbed = subtotal.min(hardness);
    Ok((subtotal - absorbed, absorbed))
}

pub(super) fn revalue(
    mut outcome: PersistentDamageOutcome,
    group: &super::rules::persistent_damage::PersistentGroup<'_>,
    record: Option<&atlas_record::source_record::SourceBackedRecord>,
    participant: &atlas_local_state::EncounterParticipant,
) -> AppServiceResult<(PersistentDamageOutcome, i64)> {
    let atlas_local_state::ConditionParameters::PersistentDamage {
        formula,
        multiplier,
        ..
    } = &group.selected_source.parameters
    else {
        return Err(AppServiceError::invalid_request(
            "Damage formula is unavailable",
        ));
    };
    let roll = outcome
        .roll
        .as_ref()
        .filter(|roll| roll.formula == *formula)
        .ok_or_else(|| {
            AppServiceError::invalid_request(
                "Damage formula changed; use a table result for this type",
            )
        })?;
    if outcome.source_condition_id != group.selected_source.condition_id {
        return Err(AppServiceError::invalid_request(
            "Active damage source changed; use a table result for this type",
        ));
    }
    let protection =
        super::rules::damage::persistent_protection(record, participant, group.damage_type)
            .map_err(AppServiceError::invalid_request)?;
    outcome.amount = protection.apply(super::rules::persistent_damage::scaled(
        i64::from(roll.total),
        *multiplier,
    ));
    outcome.protection = Some(atlas_local_state::PersistentDamageProtection {
        immune: protection.immune,
        weakness: protection.weakness,
        resistance: protection.resistance,
    });
    Ok((outcome, protection.hardness))
}

impl AtlasAppService {
    pub fn prepare_persistent_damage(
        &self,
        encounter_ref: &str,
        request: PreparePersistentDamageRequest,
    ) -> AppServiceResult<PreparedPersistentDamageView> {
        let intent = serde_json::json!({"action":"prepare_persistent_damage", "request":request});
        if let Some(receipt) =
            super::event_execution::recover(self, encounter_ref, &request.request_id, &intent)?
        {
            return preparation_view(receipt);
        }
        let snapshot = super::event_execution::snapshot(
            self,
            encounter_ref,
            request.expected_encounter_revision,
        )?;
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == request.participant_key)
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
            .map(|b| super::outgoing_turns::frozen_participant(b, participant))
            .transpose()?;
        let active = super::persistent_damage::active_damage_subject(
            frozen.as_ref().unwrap_or(participant),
            participant,
        );
        let groups = super::rules::persistent_damage::participant_groups(&active);
        let round = boundary
            .as_ref()
            .map_or(snapshot.encounter.round_number, |b| {
                b.snapshot.encounter.round_number
            });
        let context = PersistentPreparationContext {
            participant_key: participant.participant_key.clone(),
            record_key: participant.record_key.clone(),
            boundary_key: request.boundary_key.clone(),
            artifact_generation: request.artifact_generation.clone(),
        };
        let requested: std::collections::BTreeSet<_> = request
            .groups
            .iter()
            .map(|g| g.episode_id.as_str())
            .collect();
        if requested.len() != request.groups.len()
            || request
                .groups
                .iter()
                .any(|input| !groups.iter().any(|g| g.episode_id == input.episode_id))
        {
            return Err(AppServiceError::invalid_request(
                "Persistent damage sources changed; review the groups",
            ));
        }
        let new_rolls = request
            .groups
            .iter()
            .filter(|g| matches!(g.damage, PersistentDamageInput::Roll))
            .count();
        if new_rolls > 1 || (new_rolls > 0 && request.recovery.is_some()) {
            return Err(AppServiceError::invalid_request(
                "Prepare one roll at a time",
            ));
        }
        let needs_source = request
            .groups
            .iter()
            .any(|g| !matches!(g.damage, PersistentDamageInput::TableResult { .. }));
        let (record, reference_revision) = if needs_source {
            let expected_generation = request.artifact_generation.clone();
            let record_key = participant.record_key.clone();
            self.with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |session| {
                    if expected_generation.as_deref()
                        != Some(session.retrieval_mut().artifact_generation()?.as_str())
                    {
                        return Err(AppServiceError::invalid_request(
                            "Reference data changed; review the damage",
                        ));
                    }
                    let key = record_key.as_deref().ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "Damage protections are unavailable; enter table results",
                        )
                    })?;
                    let record =
                        super::hydration::resolve_record_ref(session.retrieval_mut(), key)?;
                    Ok((Some(record.source), Some(session.reference_revision()?)))
                },
            )?
        } else {
            (None, None)
        };
        let mut resolved = Vec::new();
        let mut hardness = 0;
        let mut rolling = None;
        let mut recovered_id = None;
        let mut claims = Vec::new();
        for input in &request.groups {
            let group = groups
                .iter()
                .find(|g| g.episode_id == input.episode_id)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Persistent damage source is unavailable")
                })?;
            match &input.damage {
                PersistentDamageInput::TableResult { amount } => {
                    if !(0..=1_000_000_000).contains(amount) {
                        return Err(AppServiceError::invalid_request(
                            "Damage must be between 0 and 1000000000",
                        ));
                    }
                    resolved.push(PersistentDamageOutcome {
                        episode_id: group.episode_id.into(),
                        damage_type: group.damage_type.into(),
                        source_condition_id: group.selected_source.condition_id,
                        roll: None,
                        amount: *amount,
                        table_resolved: true,
                        protection: None,
                    });
                }
                PersistentDamageInput::Prepared { request_id } => {
                    let receipt = receipt_for(
                        self,
                        &snapshot.encounter.encounter_key,
                        &context,
                        round,
                        group.episode_id,
                        request_id,
                        EncounterBoundaryKind::PreparedPersistentDamage,
                    )?;
                    let (saved, _, _) = prepared(receipt, &context)?;
                    let outcome = saved
                        .into_iter()
                        .find(|g| {
                            g.episode_id == group.episode_id
                                && g.source_condition_id == group.selected_source.condition_id
                                && !g.table_resolved
                        })
                        .ok_or_else(|| {
                            AppServiceError::invalid_request("Prepared damage source changed")
                        })?;
                    let (outcome, shared) = revalue(outcome, group, record.as_ref(), participant)?;
                    hardness = hardness.max(shared);
                    resolved.push(outcome);
                }
                PersistentDamageInput::Roll => {
                    let existing = self.with_state(|session| {
                        Ok(session.local_state().encounters().boundary_receipt(
                            &snapshot.encounter.encounter_key,
                            &super::persistent_damage::boundary_claim(
                                participant,
                                round,
                                EncounterBoundaryKind::PreparedPersistentDamage,
                                &scope(request.boundary_key.as_deref(), group.episode_id),
                            ),
                        )?)
                    })?;
                    if let Some(receipt) = existing {
                        let id = receipt.request_id.clone();
                        let (saved, _, _) = prepared(receipt, &context)?;
                        let outcome = saved
                            .into_iter()
                            .find(|g| g.episode_id == group.episode_id)
                            .ok_or_else(|| {
                                AppServiceError::invalid_request(
                                    "Prepared damage unavailable; use Table results",
                                )
                            })?;
                        let (outcome, shared) =
                            revalue(outcome, group, record.as_ref(), participant)?;
                        resolved.push(outcome);
                        hardness = hardness.max(shared);
                        recovered_id = Some(id);
                        continue;
                    }
                    let atlas_local_state::ConditionParameters::PersistentDamage {
                        formula,
                        multiplier,
                        ..
                    } = &group.selected_source.parameters
                    else {
                        return Err(AppServiceError::invalid_request(
                            "Damage formula unavailable",
                        ));
                    };
                    let protection = super::rules::damage::persistent_protection(
                        record.as_ref(),
                        participant,
                        group.damage_type,
                    )
                    .map_err(AppServiceError::invalid_request)?;
                    hardness = hardness.max(protection.hardness);
                    let dice = atlas_dice::prepare(formula)
                        .map_err(atlas_local_state::LocalStateError::from)?;
                    rolling = Some((group, dice, *multiplier, protection));
                    claims.push(super::persistent_damage::boundary_claim(
                        participant,
                        round,
                        EncounterBoundaryKind::PreparedPersistentDamage,
                        &scope(request.boundary_key.as_deref(), group.episode_id),
                    ));
                }
            }
        }
        let recovery = request
            .recovery
            .as_ref()
            .map(|input| {
                let group = groups
                    .iter()
                    .find(|g| g.episode_id == input.episode_id)
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("Persistent damage episode ended")
                    })?;
                if !resolved.iter().any(|g| g.episode_id == input.episode_id) {
                    return Err(AppServiceError::invalid_request(
                        "Enter or roll this type's damage before recovery",
                    ));
                }
                let dc = input
                    .dc_override
                    .or_else(|| super::rules::persistent_damage::recovery_dc(group.selected_source))
                    .filter(|dc| (0..=100).contains(dc))
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("Enter a recovery DC between 0 and 100")
                    })?;
                claims.push(super::persistent_damage::boundary_claim(
                    participant,
                    round,
                    EncounterBoundaryKind::PreparedPersistentRecovery,
                    &scope(request.boundary_key.as_deref(), group.episode_id),
                ));
                Ok((group, dc))
            })
            .transpose()?;
        if hardness > 0
            && request
                .groups
                .iter()
                .any(|g| matches!(g.damage, PersistentDamageInput::TableResult { .. }))
            && needs_source
        {
            return Err(AppServiceError::invalid_request(
                "For Hardness, roll all types or enter all table totals",
            ));
        }
        let complete = request.groups.len() == groups.len();
        // Preview requests do not roll, write receipts, or advance encounter revisions.
        if rolling.is_none() && recovery.is_none() {
            return preview_view(
                &snapshot.encounter.encounter_key,
                recovered_id.as_deref().unwrap_or(&request.request_id),
                resolved,
                hardness,
                complete,
                None,
            );
        }
        self.with_write(|session| {
            if let Some(revision) = reference_revision {
                session.require_reference_revision(revision)?;
            }
            let store = session.local_state().encounters();
            let damage_claim = super::persistent_damage::boundary_claim(
                participant,
                round,
                EncounterBoundaryKind::PersistentDamage,
                "",
            );
            if store.boundary_claimed(&snapshot.encounter.encounter_key, &damage_claim)? {
                return Err(AppServiceError::invalid_request(
                    "This turn's damage is already applied",
                ));
            }
            let receipt = store.prepare_condition_event_with_claims(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                atlas_local_state::ConditionEventBoundary {
                    claims: &claims,
                    outgoing_boundary_key: request.boundary_key.as_deref(),
                },
                || {
                    if let Some((group, dice, multiplier, protection)) = rolling {
                        let roll = dice.roll()?;
                        resolved.push(PersistentDamageOutcome {
                            episode_id: group.episode_id.into(),
                            damage_type: group.damage_type.into(),
                            source_condition_id: group.selected_source.condition_id,
                            amount: protection.apply(super::rules::persistent_damage::scaled(
                                i64::from(roll.total),
                                multiplier,
                            )),
                            roll: Some(roll),
                            table_resolved: false,
                            protection: Some(atlas_local_state::PersistentDamageProtection {
                                immune: protection.immune,
                                weakness: protection.weakness,
                                resistance: protection.resistance,
                            }),
                        });
                    }
                    let recovery = recovery
                        .map(|(group, dc)| {
                            let roll = if (2..=20).contains(&dc) {
                                Some(atlas_dice::roll("1d20")?)
                            } else {
                                None
                            };
                            Ok::<_, atlas_local_state::LocalStateError>(
                                PreparedPersistentRecovery {
                                    episode_id: group.episode_id.into(),
                                    damage_type: group.damage_type.into(),
                                    dc,
                                    success: dc <= 1
                                        || (dc <= 20
                                            && roll
                                                .as_ref()
                                                .is_some_and(|r| i64::from(r.total) >= dc)),
                                    roll,
                                },
                            )
                        })
                        .transpose()?;
                    Ok(vec![ConditionEventOutcome::PersistentPrepared {
                        context: Box::new(context),
                        groups: resolved,
                        hardness,
                        complete,
                        recovery,
                    }])
                },
            )?;
            preparation_view(receipt)
        })
    }
}

fn preparation_view(
    receipt: EncounterEventReceipt,
) -> AppServiceResult<PreparedPersistentDamageView> {
    let key = receipt.encounter_key;
    let id = receipt.request_id;
    let (groups, hardness, complete, recovery) = receipt
        .outcomes
        .into_iter()
        .find_map(|o| match o {
            ConditionEventOutcome::PersistentPrepared {
                groups,
                hardness,
                complete,
                recovery,
                ..
            } => Some((groups, hardness, complete, recovery)),
            _ => None,
        })
        .ok_or_else(|| AppServiceError::invalid_request("Prepared result is unavailable"))?;
    preview_view(&key, &id, groups, hardness, complete, recovery)
}

fn preview_view(
    key: &str,
    id: &str,
    groups: Vec<PersistentDamageOutcome>,
    hardness: i64,
    complete: bool,
    recovery: Option<PreparedPersistentRecovery>,
) -> AppServiceResult<PreparedPersistentDamageView> {
    let (total, hardness_absorbed) = total(&groups, hardness)?;
    let recovery = recovery.map(|r| PersistentRecoveryResultView {
        encounter_key: key.into(),
        request_id: id.into(),
        episode_id: r.episode_id,
        damage_type: r.damage_type,
        dc: Some(r.dc),
        roll: r.roll,
        success: r.success,
    });
    let damage = PersistentDamageResultView {
        encounter_key: key.into(),
        request_id: id.into(),
        groups: groups
            .into_iter()
            .map(super::persistent_damage_resolution::outcome_view)
            .collect(),
        total,
        hardness_absorbed,
    };
    Ok(PreparedPersistentDamageView {
        damage,
        complete,
        recovery,
    })
}
