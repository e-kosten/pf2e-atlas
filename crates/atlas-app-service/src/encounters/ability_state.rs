//! Source-defined ability state uses the normal guarded encounter-event boundary.
use super::{event_execution, rules, workflow};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{
    AbilityStateChange, AbilityStateValue, ParticipantAbilityState, PreparedConditionEvent,
};

impl AtlasAppService {
    pub fn update_encounter_ability_state(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterAbilityStateRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"ability_state", "request":request});
        if let Some(receipt) =
            event_execution::recover(self, encounter_ref, &request.request_id, &intent)?
        {
            return Ok(event_execution::mutation(receipt));
        }
        let snapshot =
            event_execution::snapshot(self, encounter_ref, request.expected_encounter_revision)?;
        let participant = snapshot
            .participants
            .iter()
            .find(|participant| participant.participant_key == request.participant_key)
            .ok_or_else(|| {
                AppServiceError::invalid_request("Participant is no longer in this encounter.")
            })?;
        let subject = workflow::condition_subject_context(self, participant)?;
        let sources = &subject.source_conditions;
        if request.navigation.record_key.as_str()
            != participant.record_key.as_deref().unwrap_or_default()
            || request.navigation.owners.len() != 1
            || request.navigation.field.is_some()
            || request.navigation.passage.is_some()
            || request.navigation.spell_rank.is_some()
            || request.navigation.source_fingerprint.as_deref()
                != sources.source_fingerprint.as_deref()
            || sources.source_fingerprint.is_none()
        {
            return Err(AppServiceError::invalid_request(
                "Ability source changed. Reload the encounter.",
            ));
        }
        let owner = &request.navigation.owners[0];
        let (value, declaration_identity) = match &request.action {
            EncounterAbilityStateAction::SetOption { active, selection } => {
                let option = sources
                    .options
                    .iter()
                    .find(|option| {
                        &option.owner == owner && Some(option.ordinal) == request.rule_ordinal
                    })
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("Ability option is no longer available.")
                    })?;
                if (option.always_active && !*active)
                    || (!option.choices.is_empty()
                        && !option
                            .choices
                            .iter()
                            .any(|choice| Some(choice.value.as_str()) == selection.as_deref()))
                    || (option.choices.is_empty() && selection.is_some())
                {
                    return Err(AppServiceError::invalid_request(
                        "Choose an available ability option.",
                    ));
                }
                (
                    AbilityStateValue::Option {
                        active: *active,
                        selection: selection.clone(),
                    },
                    option.declaration_identity.clone(),
                )
            }
            EncounterAbilityStateAction::SpendUse | EncounterAbilityStateAction::RestoreUse => {
                let ability = sources
                    .uses
                    .iter()
                    .find(|ability| &ability.owner == owner && request.rule_ordinal.is_none())
                    .ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "Limited-use ability is no longer available.",
                        )
                    })?;
                let current = ability.remaining(participant);
                let spending = matches!(request.action, EncounterAbilityStateAction::SpendUse);
                let remaining = if spending {
                    current.checked_sub(1)
                } else {
                    current.checked_add(1)
                }
                .filter(|remaining| (0..=ability.maximum).contains(remaining))
                .ok_or_else(|| {
                    AppServiceError::invalid_request(if spending {
                        "No uses remaining."
                    } else {
                        "All uses are available."
                    })
                })?;
                (
                    AbilityStateValue::Uses { remaining },
                    ability.declaration_identity.clone(),
                )
            }
        };
        let state = ParticipantAbilityState {
            owner: owner.clone(),
            rule_ordinal: request.rule_ordinal,
            declaration_identity: Some(declaration_identity),
            source_fingerprint: sources
                .source_fingerprint
                .clone()
                .ok_or_else(|| AppServiceError::invalid_request("Ability source unavailable."))?,
            value,
        };
        let changes = PreparedConditionEvent {
            ability_states: vec![AbilityStateChange {
                participant_key: participant.participant_key.clone(),
                expected_revision: participant.revision,
                state,
            }],
            ..Default::default()
        };
        let mut planned = rules::events::plan_condition_transition(
            participant,
            sources,
            changes,
            true,
            false,
            subject.level,
            &request.request_id,
        )?;
        if planned.effects.iter().any(|effect| {
            matches!(
                effect,
                rules::events::ConditionConsequence::DrainedLevelRequired(_)
            )
        }) {
            return Err(AppServiceError::invalid_request(
                "Set this creature's level before applying Drained.",
            ));
        }
        if snapshot.encounter.current_turn_participant_key.as_deref()
            == Some(participant.participant_key.as_str())
        {
            rules::stunned_arrival::apply(participant, sources, &mut planned)?;
        }
        let changes = planned.changes;
        event_execution::EncounterEvent {
            condition_sources: &[(&participant.participant_key, &subject.source_conditions)],
            encounter_key: &snapshot.encounter.encounter_key,
            request_id: &request.request_id,
            intent: &intent,
            expected_revision: request.expected_encounter_revision,
            reference_revision: subject.reference_revision,
            participant_revision: Some((
                &request.participant_key,
                request.expected_participant_revision,
            )),
        }
        .commit(self, changes)
        .map(event_execution::mutation)
    }
}

pub(super) fn views(
    participant: &atlas_local_state::EncounterParticipant,
    sources: &rules::source_conditions::SourceConditions,
    level: Option<i64>,
) -> Vec<EncounterAbilityStateView> {
    let navigation = |owner: &atlas_domain::OwnedContentLocator| RecordNavigationView {
        record_key: participant.record_key.clone().unwrap_or_default(),
        owners: vec![owner.clone()],
        source_fingerprint: sources.source_fingerprint.clone(),
        field: None,
        passage: None,
        spell_rank: None,
    };
    sources
        .options
        .iter()
        .map(|option| {
            let (active, selection) = option.state(participant);
            EncounterAbilityStateView {
                navigation: navigation(&option.owner),
                rule_ordinal: Some(option.ordinal),
                label: option_label(option, &sources.options).into(),
                state: EncounterAbilityStateKindView::Option {
                    active,
                    always_active: option.always_active,
                    selection,
                    choices: option
                        .choices
                        .iter()
                        .map(|choice| EncounterAbilityChoiceView {
                            value: choice.value.clone(),
                            label: choice.rule.definition().name.into(),
                            consequence: choice_consequence(
                                participant,
                                sources,
                                option,
                                choice,
                                level,
                            ),
                        })
                        .collect(),
                },
            }
        })
        .chain(sources.uses.iter().map(|ability| {
            EncounterAbilityStateView {
                navigation: navigation(&ability.owner),
                rule_ordinal: None,
                label: ability.label.clone(),
                state: EncounterAbilityStateKindView::Uses {
                    remaining: ability.remaining(participant),
                    maximum: ability.maximum,
                    interval: match ability.per {
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::Day => {
                            "day"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::Turn => {
                            "turn"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::Round => {
                            "round"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::P1M => {
                            "month"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::P1W => {
                            "week"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::P1Y => {
                            "year"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::PT10M => {
                            "10 minutes"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::PT1H => {
                            "hour"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::PT1M => {
                            "minute"
                        }
                        atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer::PT24H => {
                            "24 hours"
                        }
                    }
                    .into(),
                },
            }
        }))
        .collect()
}

fn option_label<'a>(
    option: &'a rules::source_ability_state::SourceOption,
    options: &[rules::source_ability_state::SourceOption],
) -> &'a str {
    let multiple = options
        .iter()
        .filter(|candidate| {
            candidate.owner == option.owner
                && (!candidate.always_active || !candidate.choices.is_empty())
        })
        .take(2)
        .count()
        > 1;
    if !multiple {
        return &option.label;
    }
    // Exact English labels at model pin 28d9d156, static/lang/re-en.json.
    match option.authored_label.as_deref() {
        Some("PF2E.NPCAbility.SanctumKnight.AeonStar.Black.RollOptionLabel") => {
            "Black Aeon Star used"
        }
        Some("PF2E.NPCAbility.SanctumKnight.AeonStar.Gold.RollOptionLabel") => {
            "Gold Aeon Star used"
        }
        Some("PF2E.NPCAbility.SanctumKnight.AeonStar.Green.RollOptionLabel") => {
            "Green Aeon Star used"
        }
        Some("PF2E.NPCAbility.SanctumKnight.AeonStar.White.RollOptionLabel") => {
            "White Aeon Star used"
        }
        Some(label) if !label.starts_with("PF2E.") && !label.trim().is_empty() => label,
        _ => &option.label,
    }
}

fn choice_consequence(
    participant: &atlas_local_state::EncounterParticipant,
    sources: &rules::source_conditions::SourceConditions,
    option: &rules::source_ability_state::SourceOption,
    choice: &rules::source_ability_state::SourceOptionChoice,
    level: Option<i64>,
) -> Option<String> {
    if choice.rule != super::conditions::ConditionRule::Drained {
        return None;
    }
    let before = rules::events::effective_drained_value(participant, sources).ok()?;
    let mut after = participant.clone();
    after
        .ability_states
        .retain(|state| state.owner != option.owner || state.rule_ordinal != Some(option.ordinal));
    after.ability_states.push(ParticipantAbilityState {
        owner: option.owner.clone(),
        rule_ordinal: Some(option.ordinal),
        declaration_identity: Some(option.declaration_identity.clone()),
        source_fingerprint: sources.source_fingerprint.clone()?,
        value: AbilityStateValue::Option {
            active: true,
            selection: Some(choice.value.clone()),
        },
    });
    let after = rules::events::effective_drained_value(&after, sources).ok()?;
    if after <= before {
        return None;
    }
    Some(
        match rules::condition_hp_loss(choice.rule, after - before, level) {
            Some(amount) => format!("Lose {amount} HP"),
            None => "Level needed for HP loss".into(),
        },
    )
}

/// Per-turn uses refresh at every initiative advance; per-round uses refresh at
/// their owner's turn start, matching the pinned actor/encounter recharge policy.
/// Longer intervals require an explicit restore because Atlas has no game clock.
/// Incoming condition causes and ability refreshes share one reference snapshot.
#[derive(Default)]
pub(super) struct PreparedTurnSources {
    pub conditions: rules::source_conditions::SourceConditions,
    pub ability_refresh: Vec<AbilityStateChange>,
    pub reference_revision: Option<u64>,
}

pub(super) fn prepare_turn_sources(
    service: &AtlasAppService,
    snapshot: &atlas_local_state::EncounterWithParticipants,
    incoming: Option<&str>,
) -> AppServiceResult<PreparedTurnSources> {
    if snapshot
        .participants
        .iter()
        .all(|participant| participant.record_key.is_none())
    {
        return Ok(PreparedTurnSources::default());
    }
    let incoming_needs_source = snapshot.participants.iter().any(|participant| {
        incoming == Some(participant.participant_key.as_str())
            && participant.record_key.is_some()
            && participant.participant_kind != atlas_local_state::ParticipantKind::Hazard
    });
    let participants = snapshot.participants.clone();
    let incoming = incoming.map(str::to_owned);
    let result = service.with_workspace(
        &crate::ReadExecutionContext::uncancellable(),
        move |session| {
            let mut changes = vec![];
            let mut incoming_conditions = if incoming_needs_source {
                rules::source_conditions::SourceConditions::unavailable()
            } else {
                Default::default()
            };
            let mut source_conditions = std::collections::BTreeMap::new();
            for participant in &participants {
                let Some(key) = participant.record_key.as_deref() else {
                    continue;
                };
                if !source_conditions.contains_key(key) {
                    let Some(record_key) = atlas_domain::RecordKey::parse(key).ok() else {
                        continue;
                    };
                    let source =
                        session
                            .retrieval_mut()
                            .get_record(atlas_search::GetRecordRequest {
                                record_key: &record_key,
                                selected_content: &[],
                            })?;
                    let conditions = source
                        .as_ref()
                        .map(|source| {
                            super::hydration::source_conditions(
                                &source.source,
                                session.retrieval_mut(),
                            )
                        })
                        .transpose()?;
                    source_conditions.insert(key.to_owned(), conditions);
                }
                let Some(sources) = source_conditions.get(key).and_then(Option::as_ref) else {
                    continue;
                };
                if incoming_needs_source
                    && incoming.as_deref() == Some(participant.participant_key.as_str())
                {
                    incoming_conditions = sources.clone();
                }
                for ability in &sources.uses {
                    use atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer as Per;
                    let due = ability.per == Per::Turn
                        || (ability.per == Per::Round
                            && incoming.as_deref() == Some(participant.participant_key.as_str()));
                    if !due || ability.remaining(participant) == ability.maximum {
                        continue;
                    }
                    let Some(fingerprint) = sources.source_fingerprint.as_ref() else {
                        continue;
                    };
                    changes.push(AbilityStateChange {
                        participant_key: participant.participant_key.clone(),
                        expected_revision: participant.revision,
                        state: ParticipantAbilityState {
                            owner: ability.owner.clone(),
                            rule_ordinal: None,
                            declaration_identity: Some(ability.declaration_identity.clone()),
                            source_fingerprint: fingerprint.clone(),
                            value: AbilityStateValue::Uses {
                                remaining: ability.maximum,
                            },
                        },
                    });
                }
            }
            Ok(PreparedTurnSources {
                conditions: incoming_conditions,
                ability_refresh: changes,
                reference_revision: Some(session.reference_revision()?),
            })
        },
    );
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let error = error.into_app_error();
            if matches!(
                error.code,
                AppErrorCode::SetupRequired
                    | AppErrorCode::ArtifactIncompatible
                    | AppErrorCode::ArtifactGenerationChanged
                    | AppErrorCode::IndexUnavailable
                    | AppErrorCode::ArtifactNotReady
            ) {
                Ok(PreparedTurnSources {
                    conditions: if incoming_needs_source {
                        rules::source_conditions::SourceConditions::unavailable()
                    } else {
                        Default::default()
                    },
                    ..Default::default()
                })
            } else {
                Err(error.into())
            }
        }
    }
}
