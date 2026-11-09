//! Preview and roll one browser-held persistent damage draft without saving gameplay.
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    PersistentDamageDraft, PersistentDamageInput, PersistentDamageResultView,
    PersistentDraftContext, PersistentRecoveryDraft, PersistentRecoveryResultView,
    PreparePersistentDamageRequest, PreparedPersistentDamageDraft, PreparedPersistentDamageView,
};
use atlas_local_state::{EncounterBoundaryKind, PersistentDamageOutcome};

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

pub(super) fn draft_damage(
    draft: &PersistentDamageDraft,
    context: &PersistentDraftContext,
    group: &super::rules::persistent_damage::PersistentGroup<'_>,
    record: Option<&atlas_record::source_record::SourceBackedRecord>,
    participant: &atlas_local_state::EncounterParticipant,
    sources: &super::rules::source_conditions::SourceConditions,
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
    if &draft.context != context || draft.source_condition_id != group.selected_source.condition_id
    {
        return Err(AppServiceError::invalid_request(
            "Prepared damage source or turn changed; review this damage type",
        ));
    }
    let dice = atlas_dice::prepare(formula).map_err(atlas_local_state::LocalStateError::from)?;
    dice.validate_outcome(&draft.roll).map_err(|_| {
        AppServiceError::invalid_request("Prepared damage roll does not match the current formula")
    })?;
    let protection = super::rules::damage::persistent_protection(
        record,
        participant,
        sources,
        group.damage_type,
    )
    .map_err(AppServiceError::invalid_request)?;
    Ok((
        rolled_damage(group, draft.roll.clone(), *multiplier, protection),
        protection.hardness,
    ))
}

fn rolled_damage(
    group: &super::rules::persistent_damage::PersistentGroup<'_>,
    roll: atlas_dice::RollOutcome,
    multiplier: atlas_local_state::DamageMultiplier,
    protection: super::rules::damage::DamageProtection,
) -> PersistentDamageOutcome {
    PersistentDamageOutcome {
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
    }
}

fn recovery_dc(
    group: &super::rules::persistent_damage::PersistentGroup<'_>,
    dc_override: Option<i64>,
) -> AppServiceResult<i64> {
    dc_override
        .or_else(|| super::rules::persistent_damage::recovery_dc(group.selected_source))
        .filter(|dc| (0..=100).contains(dc))
        .ok_or_else(|| AppServiceError::invalid_request("Enter a recovery DC between 0 and 100"))
}

fn recovery_success(dc: i64, roll: Option<&atlas_dice::RollOutcome>) -> bool {
    dc <= 1 || roll.is_some_and(|roll| i64::from(roll.total) >= dc)
}

pub(super) fn validate_recovery(
    draft: &PersistentRecoveryDraft,
    context: &PersistentDraftContext,
    group: &super::rules::persistent_damage::PersistentGroup<'_>,
) -> AppServiceResult<bool> {
    // Recovery uses the saved condition's DC or the GM override. It remains
    // usable when only reference data changed.
    let anchor = &draft.context;
    if anchor.encounter_key != context.encounter_key
        || anchor.participant_key != context.participant_key
        || anchor.boundary_key != context.boundary_key
        || anchor.round_number != context.round_number
        || anchor.record_key != context.record_key
        || draft.source_condition_id != group.selected_source.condition_id
        || draft.dc != recovery_dc(group, draft.dc_override)?
    {
        return Err(AppServiceError::invalid_request(
            "Prepared recovery source, DC, or turn changed; review this damage type",
        ));
    }
    let valid = match (&draft.roll, (2..=20).contains(&draft.dc)) {
        (Some(roll), true) => atlas_dice::prepare("1d20")
            .and_then(|dice| dice.validate_outcome(roll))
            .is_ok(),
        (None, false) => true,
        _ => false,
    };
    if !valid {
        return Err(AppServiceError::invalid_request(
            "Prepared recovery roll does not match the flat check",
        ));
    }
    Ok(recovery_success(draft.dc, draft.roll.as_ref()))
}

impl AtlasAppService {
    pub fn prepare_persistent_damage(
        &self,
        encounter_ref: &str,
        request: PreparePersistentDamageRequest,
    ) -> AppServiceResult<PreparedPersistentDamageView> {
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
                &session.local_state()?.encounters(),
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
        let (record, source_conditions, reference_revision, artifact_generation) = if needs_source {
            let expected_generation = request.artifact_generation.clone();
            let record_key = participant.record_key.clone();
            self.with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |session| {
                    let generation = session.retrieval_mut().artifact_generation()?;
                    if expected_generation.as_deref() != Some(generation.as_str()) {
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
                    let sources = super::hydration::source_conditions(
                        &record.source,
                        session.retrieval_mut(),
                    )?;
                    Ok((
                        Some(record.source),
                        sources,
                        Some(session.reference_revision()?),
                        Some(generation),
                    ))
                },
            )?
        } else {
            (None, Default::default(), None, None)
        };
        let context = PersistentDraftContext {
            encounter_key: snapshot.encounter.encounter_key.clone(),
            participant_key: participant.participant_key.clone(),
            boundary_key: request.boundary_key.clone(),
            round_number: round,
            record_key: participant.record_key.clone(),
            artifact_generation,
        };
        let mut resolved = Vec::new();
        let mut hardness = 0;
        let mut rolling = None;
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
                PersistentDamageInput::Prepared { draft } => {
                    let (outcome, shared) = draft_damage(
                        draft,
                        &context,
                        group,
                        record.as_ref(),
                        participant,
                        &source_conditions,
                    )?;
                    hardness = hardness.max(shared);
                    resolved.push(outcome);
                }
                PersistentDamageInput::Roll => {
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
                        &source_conditions,
                        group.damage_type,
                    )
                    .map_err(AppServiceError::invalid_request)?;
                    hardness = hardness.max(protection.hardness);
                    let dice = atlas_dice::prepare(formula)
                        .map_err(atlas_local_state::LocalStateError::from)?;
                    rolling = Some((group, dice, *multiplier, protection));
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
                Ok((
                    group,
                    recovery_dc(group, input.dc_override)?,
                    input.dc_override,
                ))
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
        if rolling.is_some() || recovery.is_some() {
            self.with_state(|session| {
                let store = session.local_state()?.encounters();
                let current = store
                    .get(&context.encounter_key)?
                    .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
                if current.revision != request.expected_encounter_revision {
                    return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                        encounter_key: current.encounter_key,
                        expected_revision: request.expected_encounter_revision,
                        actual_revision: current.revision,
                    }
                    .into());
                }
                if let Some(revision) = reference_revision
                    && session.reference_revision()? != revision
                {
                    return Err(AppServiceError::invalid_request(
                        "Reference data changed; review the damage",
                    ));
                }
                super::outgoing_turns::boundary(
                    &store,
                    &context.encounter_key,
                    context.boundary_key.as_deref(),
                    &context.participant_key,
                )?;
                let damage_claim = super::persistent_damage::boundary_claim(
                    participant,
                    round,
                    EncounterBoundaryKind::PersistentDamage,
                    "",
                );
                if store.boundary_claimed(&context.encounter_key, &damage_claim)? {
                    return Err(AppServiceError::invalid_request(
                        "This turn's damage is already applied",
                    ));
                }
                Ok(())
            })?;
        }
        let damage_draft = if let Some((group, dice, multiplier, protection)) = rolling {
            let roll = dice
                .roll()
                .map_err(atlas_local_state::LocalStateError::from)?;
            let draft = PreparedPersistentDamageDraft {
                episode_id: group.episode_id.into(),
                draft: PersistentDamageDraft {
                    context: context.clone(),
                    source_condition_id: group.selected_source.condition_id,
                    roll: roll.clone(),
                },
            };
            resolved.push(rolled_damage(group, roll, multiplier, protection));
            Some(draft)
        } else {
            None
        };
        let mut recovery_draft = None;
        let recovery = recovery
            .map(|(group, dc, dc_override)| {
                let roll = (2..=20)
                    .contains(&dc)
                    .then(|| atlas_dice::roll("1d20"))
                    .transpose()
                    .map_err(atlas_local_state::LocalStateError::from)?;
                let success = recovery_success(dc, roll.as_ref());
                recovery_draft = Some(PersistentRecoveryDraft {
                    context: context.clone(),
                    source_condition_id: group.selected_source.condition_id,
                    dc,
                    dc_override,
                    roll: roll.clone(),
                });
                Ok::<_, AppServiceError>(PersistentRecoveryResultView {
                    encounter_key: context.encounter_key.clone(),
                    request_id: request.request_id.clone(),
                    episode_id: group.episode_id.into(),
                    damage_type: group.damage_type.into(),
                    dc: Some(dc),
                    roll,
                    success,
                })
            })
            .transpose()?;
        let (total, hardness_absorbed) = total(&resolved, hardness)?;
        Ok(PreparedPersistentDamageView {
            damage: PersistentDamageResultView {
                encounter_key: context.encounter_key.clone(),
                request_id: request.request_id,
                groups: resolved
                    .into_iter()
                    .map(super::persistent_damage_resolution::outcome_view)
                    .collect(),
                total,
                hardness_absorbed,
            },
            complete,
            recovery,
            damage_draft,
            recovery_draft,
        })
    }
}
