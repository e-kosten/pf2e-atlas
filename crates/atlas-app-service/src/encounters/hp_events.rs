//! HP event preparation, receipt recovery and explicit pending lifecycle decisions.
use super::event_execution::{EncounterEvent, mutation, recover};
use super::rules::hp_events::HpEventInput;
use super::{rules, workflow};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{DamageLethality, PendingConsequence, SavedHpEventContext};
impl AtlasAppService {
    pub(super) fn record_hp_amount(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterHpActionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let request_id = request.request_id.clone();
        let intent = serde_json::json!({"action":"hp_table_entry","request":request});
        if let Some(receipt) = recover(self, encounter_ref, &request_id, &intent)? {
            return Ok(mutation(receipt));
        }
        let snapshot = self.with_state(|session| {
            session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))
        })?;
        let participant = snapshot
            .participants
            .iter()
            .find(|participant| participant.participant_key == request.participant_key)
            .ok_or_else(participant_not_found)?;
        if participant.revision != request.expected_revision {
            return Err(
                atlas_local_state::LocalStateError::StaleParticipantRevision {
                    participant_key: participant.participant_key.clone(),
                    expected_revision: request.expected_revision,
                    actual_revision: participant.revision,
                }
                .into(),
            );
        }
        let damage = matches!(request.action, EncounterHpAction::Damage { .. });
        let amount = match &request.action {
            EncounterHpAction::Damage { amount } | EncounterHpAction::Heal { amount } => {
                super::hp::formula_value(amount, false)?
            }
            _ => return Err(AppServiceError::invalid_request("select damage or healing")),
        };
        let damage = if amount < 0 { !damage } else { damage };
        let input = if damage {
            HpEventInput::ResolvedDamage {
                amount: amount.abs(),
            }
        } else {
            HpEventInput::ResolvedHealing {
                amount: amount.abs(),
            }
        };
        let subject = hp_subject_context(self, participant, &input)?;
        let context = request.context.unwrap_or_default();
        validate_context(&snapshot, &context)?;
        let changes = rules::hp_events::plan_hp_event(
            participant,
            &input,
            &hp_context_for_subject(context, &subject),
            rules::effective_hp_maximum(participant, subject.level),
            &request_id,
        )?;
        EncounterEvent {
            encounter_key: &snapshot.encounter.encounter_key,
            request_id: &request_id,
            intent: &intent,
            expected_revision: snapshot.encounter.revision,
            reference_revision: subject.reference_revision,
            participant_revision: Some((&request.participant_key, request.expected_revision)),
        }
        .commit(self, changes)
        .map(mutation)
    }
    pub fn pending_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
    ) -> AppServiceResult<Vec<PendingHpLifecycleView>> {
        self.with_state(|session| {
            let encounter = session
                .local_state()
                .encounters()
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            Ok(session
                .local_state()
                .encounters()
                .pending_condition_consequences(&encounter.encounter_key)?
                .into_iter()
                .filter_map(|saved| match saved.consequence {
                    PendingConsequence::HpLifecycle {
                        participant_key,
                        before_hp,
                        after_hp,
                        context,
                        was_dying,
                        ..
                    } => Some(PendingHpLifecycleView {
                        required_facts: rules::hp_events::required_lifecycle_facts(
                            was_dying, &context,
                        ),
                        consequence_key: saved.consequence_key,
                        initiative_only: false,
                        participant_key,
                        before_hp,
                        after_hp,
                        context: hp_context_view(context),
                    }),
                    PendingConsequence::InitiativePlacement {
                        participant_key,
                        before_hp,
                        after_hp,
                        context,
                    } => Some(PendingHpLifecycleView {
                        initiative_only: true,
                        required_facts: vec![HpLifecycleRequiredFactView::ResponsibleParticipant],
                        consequence_key: saved.consequence_key,
                        participant_key,
                        before_hp,
                        after_hp,
                        context: hp_context_view(context),
                    }),
                    _ => None,
                })
                .collect())
        })
    }
    pub fn preview_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
        request: ResolveEncounterHpLifecycleRequest,
    ) -> AppServiceResult<PendingHpLifecycleView> {
        let pending = self.with_state(|session| {
            let encounter = session
                .local_state()
                .encounters()
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            session
                .local_state()
                .encounters()
                .pending_condition_consequences(&encounter.encounter_key)?
                .into_iter()
                .find(|p| p.consequence_key == request.consequence_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("pending lifecycle decision no longer exists")
                })
        })?;
        let context = hp_context_input(request.context.clone());
        let correction = request.resolution == HpLifecycleResolutionView::KeepCurrentState;
        match pending.consequence {
            PendingConsequence::HpLifecycle {
                participant_key,
                before_hp,
                after_hp,
                was_dying,
                context: saved_context,
                ..
            } => {
                let context = if correction {
                    context
                } else {
                    rules::hp_events::merge_pending_context(&saved_context, &context)?
                };
                Ok(PendingHpLifecycleView {
                    consequence_key: pending.consequence_key,
                    participant_key,
                    before_hp,
                    after_hp,
                    initiative_only: false,
                    required_facts: if correction {
                        vec![]
                    } else {
                        rules::hp_events::required_lifecycle_facts(was_dying, &context)
                    },
                    context: hp_context_view(context),
                })
            }
            PendingConsequence::InitiativePlacement {
                participant_key,
                before_hp,
                after_hp,
                context: saved_context,
            } => {
                let context = if correction {
                    context
                } else {
                    rules::hp_events::merge_pending_context(&saved_context, &context)?
                };
                Ok(PendingHpLifecycleView {
                    consequence_key: pending.consequence_key,
                    participant_key,
                    before_hp,
                    after_hp,
                    initiative_only: true,
                    required_facts: if !correction
                        && context.responsible_participant_key.is_none()
                        && context.keep_current_initiative != Some(true)
                    {
                        vec![HpLifecycleRequiredFactView::ResponsibleParticipant]
                    } else {
                        vec![]
                    },
                    context: hp_context_view(context),
                })
            }
            _ => Err(AppServiceError::invalid_request(
                "selected consequence is not an HP lifecycle decision",
            )),
        }
    }
    pub fn resolve_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
        request: ResolveEncounterHpLifecycleRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"resolve_hp_lifecycle","request":request});
        if let Some(receipt) = recover(self, encounter_ref, &request.request_id, &intent)? {
            return Ok(mutation(receipt));
        }
        let (snapshot, pending) = self.with_state(|session| {
            let snapshot = session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let pending = session
                .local_state()
                .encounters()
                .pending_condition_consequences(&snapshot.encounter.encounter_key)?
                .into_iter()
                .find(|p| p.consequence_key == request.consequence_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("pending lifecycle decision no longer exists")
                })?;
            Ok((snapshot, pending))
        })?;
        let participant_key = match &pending.consequence {
            PendingConsequence::HpLifecycle {
                participant_key, ..
            }
            | PendingConsequence::InitiativePlacement {
                participant_key, ..
            } => participant_key,
            _ => {
                return Err(AppServiceError::invalid_request(
                    "selected consequence is not an HP lifecycle decision",
                ));
            }
        };
        let participant = snapshot
            .participants
            .iter()
            .find(|p| &p.participant_key == participant_key)
            .ok_or_else(participant_not_found)?;
        let subject = if request.resolution == HpLifecycleResolutionView::KeepCurrentState {
            workflow::ConditionSubjectContext {
                participant: participant.clone(),
                facts: Default::default(),
                reference_revision: None,
                level: participant.level_override,
                fortitude: None,
                perception: None,
            }
        } else {
            workflow::condition_subject_context(self, participant)?
        };
        validate_context(&snapshot, &request.context)?;
        let mut changes = if request.resolution == HpLifecycleResolutionView::KeepCurrentState {
            atlas_local_state::PreparedConditionEvent::default()
        } else if let PendingConsequence::InitiativePlacement {
            context: saved_context,
            ..
        } = &pending.consequence
        {
            let context = rules::hp_events::merge_pending_context(
                saved_context,
                &hp_context_input(request.context.clone()),
            )?;
            if context.keep_current_initiative == Some(true) {
                atlas_local_state::PreparedConditionEvent::default()
            } else {
                let source = context.responsible_participant_key.clone().ok_or_else(|| {
                    AppServiceError::invalid_request(
                        "select whose initiative the participant moves before",
                    )
                })?;
                atlas_local_state::PreparedConditionEvent {
                    initiative_moves: vec![atlas_local_state::ReorderEncounterParticipant {
                        participant_key: participant.participant_key.clone(),
                        target_participant_key: source,
                        placement: atlas_local_state::ReorderPlacement::Before,
                    }],
                    ..Default::default()
                }
            }
        } else {
            rules::hp_events::resolve_pending_lifecycle(
                participant,
                &pending.consequence,
                &hp_context_for_subject(request.context, &subject),
                &request.request_id,
            )?
        };
        changes.resolved_pending_keys.push(request.consequence_key);
        EncounterEvent {
            encounter_key: &snapshot.encounter.encounter_key,
            request_id: &request.request_id,
            intent: &intent,
            expected_revision: request.expected_encounter_revision,
            reference_revision: subject.reference_revision,
            participant_revision: None,
        }
        .commit(self, changes)
        .map(mutation)
    }
}
fn participant_not_found() -> AppServiceError {
    AppServiceError::new(
        atlas_app_model::AppErrorCode::EncounterParticipantNotFound,
        "Encounter participant was not found",
    )
}
fn validate_context(
    snapshot: &atlas_local_state::EncounterWithParticipants,
    context: &HpEventContextView,
) -> AppServiceResult<()> {
    if context
        .responsible_participant_key
        .as_ref()
        .is_some_and(|key| {
            !snapshot
                .participants
                .iter()
                .any(|p| &p.participant_key == key)
        })
    {
        return Err(AppServiceError::invalid_request(
            "responsible participant must belong to this encounter",
        ));
    }
    if context
        .base_death_threshold
        .is_some_and(|threshold| threshold < 1)
    {
        return Err(AppServiceError::invalid_request(
            "base death threshold must be positive",
        ));
    }
    Ok(())
}
pub(super) fn hp_subject_context(
    service: &AtlasAppService,
    participant: &atlas_local_state::EncounterParticipant,
    input: &HpEventInput,
) -> AppServiceResult<workflow::ConditionSubjectContext> {
    let needs_level = matches!(*input, HpEventInput::ResolvedHealing { .. })
        && participant.level_override.is_none()
        && participant.conditions.iter().any(|condition| {
            super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                == Some(super::conditions::ConditionRule::Drained)
        });
    let may_fall = matches!(*input,HpEventInput::ResolvedDamage{amount} if participant.track_dying && participant.current_hp.is_some_and(|hp|amount.saturating_sub(participant.temporary_hp)>=hp));
    let source_hazard = participant.participant_kind == atlas_local_state::ParticipantKind::Hazard
        && matches!(*input, HpEventInput::ResolvedDamage { .. });
    if needs_level || may_fall || source_hazard {
        workflow::condition_subject_context(service, participant)
    } else {
        Ok(workflow::ConditionSubjectContext {
            participant: participant.clone(),
            facts: Default::default(),
            reference_revision: None,
            level: participant.level_override,
            fortitude: None,
            perception: None,
        })
    }
}
pub(super) fn hp_context_for_subject(
    mut value: HpEventContextView,
    subject: &workflow::ConditionSubjectContext,
) -> SavedHpEventContext {
    if super::condition_events::prone_exception(subject, value.keep_upright == Some(true)).is_some()
    {
        value.keep_upright = Some(true);
    }
    let mut context = hp_context_input(value);
    context.broken_threshold = subject
        .facts
        .broken_threshold
        .as_ref()
        .and_then(serde_json::Number::as_i64);
    context
}
pub(super) fn hp_context_input(value: HpEventContextView) -> SavedHpEventContext {
    SavedHpEventContext {
        keep_current_initiative: value.keep_current_initiative,
        keep_upright: value.keep_upright,
        natural_rest_healing: value.natural_rest_healing,
        attack_or_spell_damage: value.attack_or_spell_damage,
        broken_threshold: None,
        lethality: value.lethality.map(|v| match v {
            DamageLethalityView::Lethal => DamageLethality::Lethal,
            DamageLethalityView::Nonlethal => DamageLethality::Nonlethal,
        }),
        critical: value.critical,
        enemy_source: value.enemy_source,
        responsible_participant_key: value.responsible_participant_key,
        base_death_threshold: value.base_death_threshold,
    }
}
fn hp_context_view(value: SavedHpEventContext) -> HpEventContextView {
    HpEventContextView {
        keep_current_initiative: value.keep_current_initiative,
        keep_upright: value.keep_upright,
        natural_rest_healing: value.natural_rest_healing,
        attack_or_spell_damage: value.attack_or_spell_damage,
        lethality: value.lethality.map(|v| match v {
            DamageLethality::Lethal => DamageLethalityView::Lethal,
            DamageLethality::Nonlethal => DamageLethalityView::Nonlethal,
        }),
        critical: value.critical,
        enemy_source: value.enemy_source,
        responsible_participant_key: value.responsible_participant_key,
        base_death_threshold: value.base_death_threshold,
    }
}

#[cfg(test)]
mod hp_lifecycle_tests {
    use super::super::conditions::{ConditionRule, modeled_condition_by_rule};
    use super::*;
    fn fixture() -> (AtlasAppService, String, String) {
        let worker = crate::test_support::encounter_fixture_worker().worker;
        let encounter = worker
            .create_encounter(CreateEncounterRequest {
                name: "HP lifecycle".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: encounter.encounter_key.clone(),
                display_name: "PC".into(),
                max_hp: Some(10),
                current_hp: Some(10),
                initiative: None,
            })
            .unwrap();
        let participant = worker
            .encounter(&encounter.encounter_key)
            .unwrap()
            .participants[0]
            .participant_key
            .clone();
        (worker, encounter.encounter_key, participant)
    }
    fn add(
        worker: &AtlasAppService,
        key: &str,
        participant: &str,
        rule: ConditionRule,
        id: &str,
        value: Option<i64>,
        parameters: ConditionParametersInputView,
    ) {
        let definition = modeled_condition_by_rule(rule).unwrap();
        worker
            .add_encounter_participant_condition(
                key,
                AddEncounterParticipantConditionRequest {
                    request_id: id.into(),
                    expected_encounter_revision: worker.encounter(key).unwrap().revision,
                    event_intent: ConditionEventIntentView::Correction,
                    decisions: Default::default(),
                    participant_key: participant.into(),
                    condition_ref: Some(definition.condition_ref.into()),
                    name: None,
                    value,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters,
                    origin: None,
                    duration: None,
                    note: None,
                },
            )
            .unwrap();
    }
    fn event(
        worker: &AtlasAppService,
        key: &str,
        participant: &str,
        id: &str,
        input: HpEventInput,
        context: HpEventContextView,
    ) {
        worker
            .apply_encounter_hp_action(
                key,
                ApplyEncounterHpActionRequest {
                    request_id: id.into(),
                    expected_revision: worker
                        .encounter(key)
                        .unwrap()
                        .participants
                        .iter()
                        .find(|p| p.participant_key == participant)
                        .unwrap()
                        .revision,
                    participant_key: participant.into(),
                    action: match input {
                        HpEventInput::ResolvedDamage { amount } => EncounterHpAction::Damage {
                            amount: amount.to_string(),
                        },
                        HpEventInput::ResolvedHealing { amount } => EncounterHpAction::Heal {
                            amount: amount.to_string(),
                        },
                        HpEventInput::DirectLoss { .. } => panic!(
                            "Direct loss is an internal consequence, not a product HP action"
                        ),
                    },
                    context: Some(context),
                },
            )
            .unwrap();
    }
    #[test]
    fn unknown_knockout_facts_apply_hp_once_then_resume_lifecycle_without_damage() {
        let (worker, key, participant) = fixture();
        let request = ApplyEncounterHpActionRequest {
            request_id: "damage".into(),
            expected_revision: worker.encounter(&key).unwrap().participants[0].revision,
            participant_key: participant.clone(),
            action: EncounterHpAction::Damage {
                amount: "12".into(),
            },
            context: None,
        };
        worker
            .apply_encounter_hp_action(&key, request.clone())
            .unwrap();
        worker.apply_encounter_hp_action(&key, request).unwrap();
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].current_hp, Some(0));
        assert!(!detail.participants[0].defeated);
        let pending = worker.pending_encounter_hp_lifecycle(&key).unwrap();
        assert_eq!(pending.len(), 1);
        let resolve = ResolveEncounterHpLifecycleRequest {
            resolution: HpLifecycleResolutionView::Apply,
            request_id: "resolve".into(),
            expected_encounter_revision: detail.revision,
            consequence_key: pending[0].consequence_key.clone(),
            context: HpEventContextView {
                lethality: Some(DamageLethalityView::Lethal),
                critical: Some(false),
                responsible_participant_key: Some(participant.clone()),
                ..Default::default()
            },
        };
        worker
            .resolve_encounter_hp_lifecycle(&key, resolve.clone())
            .unwrap();
        worker
            .resolve_encounter_hp_lifecycle(&key, resolve)
            .unwrap();
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].current_hp, Some(0));
        let dying = modeled_condition_by_rule(ConditionRule::Dying)
            .unwrap()
            .condition_ref;
        assert_eq!(
            detail.participants[0]
                .conditions
                .iter()
                .find(|c| c.condition_key.as_deref() == Some(dying))
                .unwrap()
                .value,
            Some(1)
        );
        assert!(
            worker
                .pending_encounter_hp_lifecycle(&key)
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn positive_hp_healing_clears_dying_increases_wounded_and_preserves_source_controlled_sleep_and_prone()
     {
        let (worker, key, participant) = fixture();
        event(
            &worker,
            &key,
            &participant,
            "knockout",
            HpEventInput::ResolvedDamage { amount: 10 },
            HpEventContextView {
                lethality: Some(DamageLethalityView::Lethal),
                critical: Some(false),
                responsible_participant_key: Some(participant.clone()),
                ..Default::default()
            },
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Unconscious,
            "deep-sleep",
            None,
            ConditionParametersInputView::Unconscious {
                waking: WakingRuleView::SourceControlled,
            },
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Wounded,
            "wounded-one",
            Some(1),
            Default::default(),
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Wounded,
            "wounded-three",
            Some(3),
            Default::default(),
        );
        event(
            &worker,
            &key,
            &participant,
            "healing",
            HpEventInput::ResolvedHealing { amount: 1 },
            Default::default(),
        );
        let detail = worker.encounter(&key).unwrap();
        let rows = &detail.participants[0].conditions;
        let key_for = |rule| modeled_condition_by_rule(rule).unwrap().condition_ref;
        assert!(
            !rows
                .iter()
                .any(|c| c.condition_key.as_deref() == Some(key_for(ConditionRule::Dying)))
        );
        assert!(
            rows.iter()
                .any(|c| c.condition_key.as_deref() == Some(key_for(ConditionRule::Unconscious)))
        );
        assert!(
            rows.iter()
                .any(|c| c.condition_key.as_deref() == Some(key_for(ConditionRule::Prone)))
        );
        let wounded = rows
            .iter()
            .filter(|c| c.condition_key.as_deref() == Some(key_for(ConditionRule::Wounded)))
            .collect::<Vec<_>>();
        assert_eq!(wounded.len(), 2);
        assert!(wounded.iter().all(|c| c.value == Some(4)));
        assert_eq!(detail.participants[0].current_hp, Some(1));
    }
    fn lifecycle(
        worker: &AtlasAppService,
        key: &str,
        participant: &str,
        id: &str,
        action: ConditionLifecycleActionView,
    ) -> ConditionLifecycleResultView {
        worker
            .apply_condition_lifecycle(
                key,
                ApplyConditionLifecycleRequest {
                    boundary_key: None,
                    request_id: id.into(),
                    expected_encounter_revision: worker.encounter(key).unwrap().revision,
                    participant_key: participant.into(),
                    action,
                },
            )
            .unwrap()
    }
    #[test]
    fn dying_recovery_normalizes_all_sources_and_new_request_cannot_repeat_boundary() {
        let (worker, key, participant) = fixture();
        event(
            &worker,
            &key,
            &participant,
            "zero",
            HpEventInput::ResolvedDamage { amount: 10 },
            HpEventContextView {
                lethality: Some(DamageLethalityView::Lethal),
                critical: Some(false),
                keep_current_initiative: Some(true),
                ..Default::default()
            },
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Dying,
            "second-dying",
            Some(2),
            Default::default(),
        );
        let request = ApplyConditionLifecycleRequest {
            boundary_key: None,
            request_id: "recovery".into(),
            expected_encounter_revision: worker.encounter(&key).unwrap().revision,
            participant_key: participant.clone(),
            action: ConditionLifecycleActionView::DyingRecovery {
                check: LifecycleCheckInputView::TableDegree {
                    degree: CheckDegreeView::CriticalSuccess,
                },
            },
        };
        let first = worker
            .apply_condition_lifecycle(&key, request.clone())
            .unwrap();
        assert_eq!(
            worker.apply_condition_lifecycle(&key, request).unwrap(),
            first
        );
        let detail = worker.encounter(&key).unwrap();
        let rule = |rule| modeled_condition_by_rule(rule).unwrap().condition_ref;
        assert!(!detail.participants[0].conditions.iter().any(|condition| {
            condition.condition_key.as_deref() == Some(rule(ConditionRule::Dying))
        }));
        assert!(
            detail.participants[0]
                .conditions
                .iter()
                .any(|condition| condition.condition_key.as_deref()
                    == Some(rule(ConditionRule::Wounded))
                    && condition.value == Some(1))
        );
        assert!(
            worker
                .apply_condition_lifecycle(
                    &key,
                    ApplyConditionLifecycleRequest {
                        boundary_key: None,
                        request_id: "second-recovery".into(),
                        expected_encounter_revision: detail.revision,
                        participant_key: participant,
                        action: ConditionLifecycleActionView::DyingRecovery {
                            check: LifecycleCheckInputView::Roll
                        }
                    }
                )
                .is_err()
        );
    }
    #[test]
    fn retch_changes_selected_source_and_preserves_other_source_and_receipt() {
        let (worker, key, participant) = fixture();
        for (id, value) in [("sick-two", 2), ("sick-three", 3)] {
            add(
                &worker,
                &key,
                &participant,
                ConditionRule::Sickened,
                id,
                Some(value),
                ConditionParametersInputView::Sickened {
                    recovery_dc: Some(20),
                },
            );
        }
        let detail = worker.encounter(&key).unwrap();
        let selected = detail.participants[0].conditions[0].condition_id;
        let result = lifecycle(
            &worker,
            &key,
            &participant,
            "retch",
            ConditionLifecycleActionView::Retch {
                confusion: None,
                judgments: vec![],
                condition_id: selected,
                dc: None,
                fortitude_modifier: None,
                actions_spent: true,
                check: LifecycleCheckInputView::TableDegree {
                    degree: CheckDegreeView::Success,
                },
            },
        );
        assert_eq!(result.check.unwrap().dc, Some(20));
        let detail = worker.encounter(&key).unwrap();
        assert!(
            detail.participants[0]
                .conditions
                .iter()
                .any(|condition| condition.condition_id == selected && condition.value == Some(1))
        );
        assert!(
            detail.participants[0]
                .conditions
                .iter()
                .any(|condition| condition.condition_id != selected && condition.value == Some(3))
        );
    }
    #[test]
    fn simultaneous_stunned_reduction_cannot_reveal_old_lower_source() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Stunned,
            "stun-two",
            Some(2),
            Default::default(),
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Stunned,
            "stun-three",
            Some(3),
            Default::default(),
        );
        let selected = worker.encounter(&key).unwrap().participants[0].conditions[0].condition_id;
        lifecycle(
            &worker,
            &key,
            &participant,
            "consume",
            ConditionLifecycleActionView::StunnedActionsLost {
                condition_id: selected,
                actions_lost: 3,
            },
        );
        assert!(
            worker.encounter(&key).unwrap().participants[0]
                .conditions
                .is_empty()
        );
    }
    #[test]
    fn confused_damage_has_one_saved_recovery_opportunity_and_explicit_source_selection() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Confused,
            "confusion-one",
            None,
            Default::default(),
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Confused,
            "confusion-two",
            None,
            Default::default(),
        );
        event(
            &worker,
            &key,
            &participant,
            "attack",
            HpEventInput::ResolvedDamage { amount: 1 },
            HpEventContextView {
                attack_or_spell_damage: Some(true),
                ..Default::default()
            },
        );
        let pending = worker.pending_condition_recoveries(&key).unwrap();
        let PendingConditionRecoveryView::ConfusedDamage {
            consequence_key,
            condition_ids,
            ..
        } = &pending[0]
        else {
            panic!("expected Confused damage opportunity")
        };
        let selected = condition_ids[0];
        lifecycle(
            &worker,
            &key,
            &participant,
            "confused-recovery",
            ConditionLifecycleActionView::ConfusedDamageRecovery {
                consequence_key: consequence_key.clone(),
                condition_ids: vec![selected],
                qualifying_damage: true,
                check: Some(LifecycleCheckInputView::TableDegree {
                    degree: CheckDegreeView::Success,
                }),
            },
        );
        assert!(
            worker
                .pending_condition_recoveries(&key)
                .unwrap()
                .is_empty()
        );
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].conditions.len(), 1);
        assert_ne!(detail.participants[0].conditions[0].condition_id, selected);
    }
    #[test]
    fn frightened_end_turn_respects_held_source_and_independent_floor() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Frightened,
            "fear-normal",
            Some(3),
            ConditionParametersInputView::Frightened {
                recovery: FrightenedRecoveryView::EndTurn { minimum: 2 },
            },
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Frightened,
            "fear-held",
            Some(2),
            ConditionParametersInputView::Frightened {
                recovery: FrightenedRecoveryView::Held,
            },
        );
        lifecycle(
            &worker,
            &key,
            &participant,
            "end-turn",
            ConditionLifecycleActionView::FrightenedEndTurn { decisions: vec![] },
        );
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].conditions.len(), 2);
        assert!(
            detail.participants[0]
                .conditions
                .iter()
                .all(|condition| condition.value == Some(2))
        );
        let retry = worker.apply_condition_lifecycle(
            &key,
            ApplyConditionLifecycleRequest {
                boundary_key: None,
                request_id: "another-end-turn".into(),
                expected_encounter_revision: detail.revision,
                participant_key: participant,
                action: ConditionLifecycleActionView::FrightenedEndTurn { decisions: vec![] },
            },
        );
        assert!(retry.is_err());
    }
    #[test]
    fn overnight_drained_recovery_does_not_restore_current_hp() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Drained,
            "drained",
            Some(2),
            Default::default(),
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Fatigued,
            "fatigue",
            None,
            Default::default(),
        );
        lifecycle(
            &worker,
            &key,
            &participant,
            "overnight",
            ConditionLifecycleActionView::OvernightRest,
        );
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].current_hp, Some(10));
        assert_eq!(detail.participants[0].conditions.len(), 1);
        assert_eq!(detail.participants[0].conditions[0].value, Some(1));
    }
    #[test]
    fn broken_crossing_has_one_damage_change_and_petrified_dies_without_dying() {
        let (worker, key, participant) = fixture();
        let mut saved = worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&key)?
                    .unwrap()
                    .participants
                    .remove(0))
            })
            .unwrap();
        saved.participant_kind = atlas_local_state::ParticipantKind::Hazard;
        saved.track_dying = false;
        let changes = rules::hp_events::plan_hp_event(
            &saved,
            &HpEventInput::ResolvedDamage { amount: 6 },
            &SavedHpEventContext {
                broken_threshold: Some(5),
                ..Default::default()
            },
            Some(10),
            "hazard-damage",
        )
        .unwrap();
        assert_eq!(changes.participant_changes.len(), 1);
        assert_eq!(changes.participant_changes[0].current_hp, Some(4));
        assert!(changes.condition_changes.iter().any(|change|matches!(change,atlas_local_state::ConditionEventChange::Add(input) if matches!(input.parameters,atlas_local_state::ConditionParameters::Broken{threshold:Some(5)}))));
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Petrified,
            "stone",
            None,
            Default::default(),
        );
        event(
            &worker,
            &key,
            &participant,
            "destroyed",
            HpEventInput::ResolvedDamage { amount: 10 },
            Default::default(),
        );
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].mortality, MortalityStateView::Dead);
        let dying = modeled_condition_by_rule(ConditionRule::Dying)
            .unwrap()
            .condition_ref;
        assert!(
            !detail.participants[0]
                .conditions
                .iter()
                .any(|condition| condition.condition_key.as_deref() == Some(dying))
        );
    }
    #[test]
    fn saved_recoveries_and_mixed_menu_work_without_reference_data() {
        let (worker, key, _) = fixture();
        worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: key.clone(),
                record_ref: "Test Creature 1".into(),
                quantity: 1,
                initiative: None,
            })
            .unwrap();
        let participant = worker
            .encounter(&key)
            .unwrap()
            .participants
            .into_iter()
            .find(|p| p.record_key.is_some())
            .unwrap()
            .participant_key;
        for (rule, value) in [
            (ConditionRule::Unconscious, None),
            (ConditionRule::Prone, None),
            (ConditionRule::Drained, Some(2)),
            (ConditionRule::Fatigued, None),
            (ConditionRule::Wounded, Some(1)),
            (ConditionRule::Sickened, Some(1)),
        ] {
            add(
                &worker,
                &key,
                &participant,
                rule,
                &format!("seed-{rule:?}"),
                value,
                if rule == ConditionRule::Sickened {
                    ConditionParametersInputView::Sickened {
                        recovery_dc: Some(15),
                    }
                } else {
                    Default::default()
                },
            );
        }
        let snapshot = || {
            worker
                .with_state(|session| {
                    Ok(session
                        .local_state()
                        .encounters()
                        .get_with_participants(&key)?
                        .unwrap())
                })
                .unwrap()
        };
        let before = snapshot();
        let actor = before
            .participants
            .iter()
            .find(|p| p.participant_key == participant)
            .unwrap();
        let initial_hp = actor.current_hp;
        let sickened = actor
            .conditions
            .iter()
            .find(|c| {
                c.condition_key.as_deref()
                    == Some(
                        modeled_condition_by_rule(ConditionRule::Sickened)
                            .unwrap()
                            .condition_ref,
                    )
            })
            .unwrap()
            .condition_id;
        atlas_search::test_support::invalidate_reference(
            worker.workspace_path(),
            atlas_runtime::WORKSPACE_SCHEMA_VERSION,
        )
        .unwrap();
        assert!(worker.encounter(&key).is_err());
        let offers = worker
            .condition_lifecycle_actions(&key, &participant, None)
            .unwrap()
            .actions;
        for offer in &offers {
            match offer.action {
                ConditionRecoveryOfferView::Wake
                | ConditionRecoveryOfferView::OvernightRest
                | ConditionRecoveryOfferView::TreatWounds => assert!(offer.available),
                ConditionRecoveryOfferView::Retch { .. }
                | ConditionRecoveryOfferView::WakeFromNoise { .. } => {
                    assert!(!offer.available);
                    assert_eq!(
                        offer.unavailable_reason.as_deref(),
                        Some("Reference data unavailable")
                    );
                }
                _ => {}
            }
        }
        assert!(
            offers
                .iter()
                .any(|o| matches!(o.action, ConditionRecoveryOfferView::Wake))
        );
        assert!(
            offers
                .iter()
                .any(|o| matches!(o.action, ConditionRecoveryOfferView::Retch { .. }))
        );
        assert!(
            !offers
                .iter()
                .any(|o| matches!(o.action, ConditionRecoveryOfferView::FullHpRest))
        );
        for action in [
            ConditionLifecycleActionView::Retch {
                confusion: None,
                judgments: vec![],
                condition_id: sickened,
                dc: Some(15),
                fortitude_modifier: Some(15),
                actions_spent: true,
                check: LifecycleCheckInputView::TableDegree {
                    degree: CheckDegreeView::Success,
                },
            },
            ConditionLifecycleActionView::FullHpRest { minutes: 10 },
        ] {
            assert!(
                worker
                    .apply_condition_lifecycle(
                        &key,
                        ApplyConditionLifecycleRequest {
                            boundary_key: None,
                            request_id: format!("blocked-{action:?}"),
                            expected_encounter_revision: before.encounter.revision,
                            participant_key: participant.clone(),
                            action,
                        }
                    )
                    .is_err()
            );
            assert_eq!(snapshot(), before);
        }
        for (id, action) in [
            (
                "wake",
                ConditionLifecycleActionView::Wake {
                    cause: WakeCauseView::Interact,
                },
            ),
            ("night-one", ConditionLifecycleActionView::OvernightRest),
            ("night-two", ConditionLifecycleActionView::OvernightRest),
            (
                "full-hp-rest",
                ConditionLifecycleActionView::FullHpRest { minutes: 10 },
            ),
        ] {
            let request = ApplyConditionLifecycleRequest {
                boundary_key: None,
                request_id: id.into(),
                expected_encounter_revision: snapshot().encounter.revision,
                participant_key: participant.clone(),
                action,
            };
            let first = worker
                .apply_condition_lifecycle(&key, request.clone())
                .unwrap();
            let saved = snapshot();
            assert_eq!(
                worker.apply_condition_lifecycle(&key, request).unwrap(),
                first
            );
            assert_eq!(snapshot(), saved);
        }
        let after = snapshot();
        let actor = after
            .participants
            .iter()
            .find(|p| p.participant_key == participant)
            .unwrap();
        assert_eq!(actor.current_hp, initial_hp);
        let remaining: Vec<_> = actor
            .conditions
            .iter()
            .filter_map(|c| {
                super::super::conditions::condition_rule_for_key(c.condition_key.as_deref())
            })
            .collect();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.contains(&ConditionRule::Prone));
        assert!(remaining.contains(&ConditionRule::Sickened));
    }

    #[test]
    fn recovery_outcome_preserves_rule_error_and_rolls_back_without_receipt() {
        let (worker, key, participant) = fixture();
        event(
            &worker,
            &key,
            &participant,
            "zero",
            HpEventInput::ResolvedDamage { amount: 10 },
            HpEventContextView {
                lethality: Some(DamageLethalityView::Lethal),
                critical: Some(false),
                keep_current_initiative: Some(true),
                ..Default::default()
            },
        );
        // Saved state can contain unknown counters; recovery must explain the actual issue.
        let wounded = worker
            .with_write(|session| {
                Ok(session.local_state().encounters().add_condition(
                    atlas_local_state::AddEncounterParticipantCondition {
                        participant_key: participant.clone(),
                        condition_key: Some(
                            modeled_condition_by_rule(ConditionRule::Wounded)
                                .unwrap()
                                .condition_ref
                                .into(),
                        ),
                        name: "Wounded".into(),
                        value: None,
                        source_participant_key: None,
                        scope: Default::default(),
                        extra_action: Default::default(),
                        immunity_override: false,
                        parameters: Default::default(),
                        origin: None,
                        duration: None,
                        note: None,
                    },
                )?)
            })
            .unwrap();
        let snapshot = || {
            worker
                .with_state(|session| {
                    Ok(session
                        .local_state()
                        .encounters()
                        .get_with_participants(&key)?
                        .unwrap())
                })
                .unwrap()
        };
        let before = snapshot();
        let mut request = ApplyConditionLifecycleRequest {
            boundary_key: None,
            request_id: "recover-with-unknown-wounded".into(),
            expected_encounter_revision: before.encounter.revision,
            participant_key: participant.clone(),
            action: ConditionLifecycleActionView::DyingRecovery {
                check: LifecycleCheckInputView::TableDegree {
                    degree: CheckDegreeView::Success,
                },
            },
        };
        let intent = serde_json::json!({"action":"condition_lifecycle","request":request});
        let error = worker
            .apply_condition_lifecycle(&key, request.clone())
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, AppErrorCode::InvalidRequest);
        assert_eq!(
            error.message,
            "Set the condition's positive numeric value before resolving recovery"
        );
        assert_eq!(snapshot(), before);
        worker
            .with_state(|session| {
                let store = session.local_state().encounters();
                assert!(
                    store
                        .condition_event_receipt(&key, &request.request_id, &intent)?
                        .is_none()
                );
                assert!(
                    store
                        .boundary_claims(&key, &participant, before.encounter.round_number)?
                        .is_empty()
                );
                Ok(())
            })
            .unwrap();
        worker
            .with_write(|session| {
                assert!(
                    session
                        .local_state()
                        .encounters()
                        .remove_condition(&participant, wounded.condition_id)?
                );
                Ok(())
            })
            .unwrap();
        request.expected_encounter_revision = snapshot().encounter.revision;
        let first = worker
            .apply_condition_lifecycle(&key, request.clone())
            .unwrap();
        assert_eq!(
            worker.apply_condition_lifecycle(&key, request).unwrap(),
            first
        );
    }

    #[test]
    fn unknown_existing_mortality_counter_is_not_zero() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Dying,
            "dying",
            Some(1),
            Default::default(),
        );
        let mut saved = worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&key)?
                    .unwrap()
                    .participants
                    .remove(0))
            })
            .unwrap();
        saved.conditions[0].value = None;
        assert!(
            rules::hp_events::plan_hp_event(
                &saved,
                &HpEventInput::ResolvedDamage { amount: 10 },
                &SavedHpEventContext {
                    lethality: Some(atlas_local_state::DamageLethality::Lethal),
                    critical: Some(false),
                    keep_current_initiative: Some(true),
                    ..Default::default()
                },
                Some(10),
                "unknown-counter"
            )
            .is_err()
        );
    }
    #[test]
    fn known_prone_immunity_prevents_knockout_prone_but_not_unconscious() {
        let (worker, key, _) = fixture();
        let participant = worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&key)?
                    .unwrap()
                    .participants
                    .remove(0))
            })
            .unwrap();
        let subject = workflow::ConditionSubjectContext {
            participant: participant.clone(),
            facts: rules::ConditionApplicabilityFacts {
                immunities: vec![rules::ConditionImmunity {
                    slug: "prone".into(),
                    coverage: rules::ImmunityCoverage::Unconditional,
                }]
                .into(),
                ..Default::default()
            },
            reference_revision: None,
            level: None,
            fortitude: None,
            perception: None,
        };
        let context = hp_context_for_subject(
            HpEventContextView {
                lethality: Some(DamageLethalityView::Nonlethal),
                keep_current_initiative: Some(true),
                ..Default::default()
            },
            &subject,
        );
        let changes = rules::hp_events::plan_hp_event(
            &participant,
            &HpEventInput::ResolvedDamage { amount: 10 },
            &context,
            Some(10),
            "immune-prone",
        )
        .unwrap();
        let added = |rule| {
            changes.condition_changes.iter().any(|change|matches!(change,atlas_local_state::ConditionEventChange::Add(input) if input.condition_key.as_deref()==Some(modeled_condition_by_rule(rule).unwrap().condition_ref)))
        };
        assert!(added(ConditionRule::Unconscious));
        assert!(!added(ConditionRule::Prone));
    }
    #[test]
    fn stable_recovery_restores_one_hp_and_source_controlled_sleep_survives_natural_healing() {
        let (worker, key, participant) = fixture();
        event(
            &worker,
            &key,
            &participant,
            "nonlethal",
            HpEventInput::ResolvedDamage { amount: 10 },
            HpEventContextView {
                lethality: Some(DamageLethalityView::Nonlethal),
                keep_current_initiative: Some(true),
                ..Default::default()
            },
        );
        lifecycle(
            &worker,
            &key,
            &participant,
            "stable-rest",
            ConditionLifecycleActionView::StableRecovery { minutes: 10 },
        );
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].current_hp, Some(1));
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Unconscious,
            "sleep",
            None,
            Default::default(),
        );
        event(
            &worker,
            &key,
            &participant,
            "natural-heal",
            HpEventInput::ResolvedHealing { amount: 1 },
            HpEventContextView {
                natural_rest_healing: Some(true),
                ..Default::default()
            },
        );
        let unconscious = modeled_condition_by_rule(ConditionRule::Unconscious)
            .unwrap()
            .condition_ref;
        assert!(
            worker.encounter(&key).unwrap().participants[0]
                .conditions
                .iter()
                .any(|condition| condition.condition_key.as_deref() == Some(unconscious))
        );
    }
    #[test]
    fn explicit_pending_correction_preserves_later_hp_and_does_not_invent_dying() {
        let (worker, key, participant) = fixture();
        event(
            &worker,
            &key,
            &participant,
            "unknown-knockout",
            HpEventInput::ResolvedDamage { amount: 10 },
            Default::default(),
        );
        let pending = worker
            .pending_encounter_hp_lifecycle(&key)
            .unwrap()
            .remove(0);
        event(
            &worker,
            &key,
            &participant,
            "later-heal",
            HpEventInput::ResolvedHealing { amount: 2 },
            Default::default(),
        );
        worker
            .resolve_encounter_hp_lifecycle(
                &key,
                ResolveEncounterHpLifecycleRequest {
                    resolution: HpLifecycleResolutionView::KeepCurrentState,
                    request_id: "correct-pending".into(),
                    expected_encounter_revision: worker.encounter(&key).unwrap().revision,
                    consequence_key: pending.consequence_key,
                    context: Default::default(),
                },
            )
            .unwrap();
        assert!(
            worker
                .pending_encounter_hp_lifecycle(&key)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].current_hp,
            Some(2)
        );
        assert!(
            worker.encounter(&key).unwrap().participants[0]
                .conditions
                .is_empty()
        );
    }
    #[test]
    fn retch_missing_facts_can_record_failure_but_cannot_roll_or_skip_action_cost() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Sickened,
            "sick",
            Some(2),
            Default::default(),
        );
        let detail = worker.encounter(&key).unwrap();
        let id = detail.participants[0].conditions[0].condition_id;
        let action = |actions_spent, check| ConditionLifecycleActionView::Retch {
            confusion: None,
            judgments: vec![],
            condition_id: id,
            dc: None,
            fortitude_modifier: None,
            actions_spent,
            check,
        };
        for (name, action) in [
            ("missing-facts", action(true, LifecycleCheckInputView::Roll)),
            (
                "unspent",
                action(
                    false,
                    LifecycleCheckInputView::TableDegree {
                        degree: CheckDegreeView::Failure,
                    },
                ),
            ),
        ] {
            assert!(
                worker
                    .apply_condition_lifecycle(
                        &key,
                        ApplyConditionLifecycleRequest {
                            boundary_key: None,
                            request_id: name.into(),
                            expected_encounter_revision: detail.revision,
                            participant_key: participant.clone(),
                            action
                        }
                    )
                    .is_err()
            );
        }
        let result = lifecycle(
            &worker,
            &key,
            &participant,
            "failed-retch",
            action(
                true,
                LifecycleCheckInputView::TableDegree {
                    degree: CheckDegreeView::Failure,
                },
            ),
        );
        assert_eq!(result.check.unwrap().degree, CheckDegreeView::Failure);
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].conditions[0].value,
            Some(2)
        );
    }
    #[test]
    fn ordinary_noise_wakes_but_source_controlled_noise_does_not_and_wounded_rest_requires_full_hp()
    {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Unconscious,
            "sleep",
            None,
            Default::default(),
        );
        lifecycle(
            &worker,
            &key,
            &participant,
            "noise",
            ConditionLifecycleActionView::WakeFromNoise {
                dc: 15,
                perception_modifier: None,
                check: LifecycleCheckInputView::TableDegree {
                    degree: CheckDegreeView::Success,
                },
            },
        );
        assert!(
            worker.encounter(&key).unwrap().participants[0]
                .conditions
                .is_empty()
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Unconscious,
            "deep",
            None,
            ConditionParametersInputView::Unconscious {
                waking: WakingRuleView::SourceControlled,
            },
        );
        assert!(
            worker
                .apply_condition_lifecycle(
                    &key,
                    ApplyConditionLifecycleRequest {
                        boundary_key: None,
                        request_id: "deep-noise".into(),
                        expected_encounter_revision: worker.encounter(&key).unwrap().revision,
                        participant_key: participant.clone(),
                        action: ConditionLifecycleActionView::WakeFromNoise {
                            dc: 15,
                            perception_modifier: None,
                            check: LifecycleCheckInputView::TableDegree {
                                degree: CheckDegreeView::Success
                            }
                        }
                    }
                )
                .is_err()
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Wounded,
            "wounded",
            Some(2),
            Default::default(),
        );
        lifecycle(
            &worker,
            &key,
            &participant,
            "full-rest",
            ConditionLifecycleActionView::FullHpRest { minutes: 10 },
        );
        let wounded = modeled_condition_by_rule(ConditionRule::Wounded)
            .unwrap()
            .condition_ref;
        assert!(
            !worker.encounter(&key).unwrap().participants[0]
                .conditions
                .iter()
                .any(|condition| condition.condition_key.as_deref() == Some(wounded))
        );
    }
    #[test]
    fn direct_broken_entry_caps_once_and_never_raises_hp() {
        let (worker, key, _) = fixture();
        let mut participant = worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&key)?
                    .unwrap()
                    .participants
                    .remove(0))
            })
            .unwrap();
        participant.participant_kind = atlas_local_state::ParticipantKind::Hazard;
        let definition = modeled_condition_by_rule(ConditionRule::Broken).unwrap();
        for (before, after) in [(20, 10), (5, 5)] {
            participant.current_hp = Some(before);
            let input = atlas_local_state::AddEncounterParticipantCondition {
                participant_key: participant.participant_key.clone(),
                condition_key: Some(definition.condition_ref.into()),
                name: definition.name.into(),
                value: None,
                source_participant_key: None,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                parameters: atlas_local_state::ConditionParameters::Broken {
                    threshold: Some(10),
                },
                origin: None,
                duration: None,
                note: None,
            };
            let planned = rules::events::plan_condition_change(
                &participant,
                atlas_local_state::ConditionEventChange::Add(input),
                true,
                false,
                None,
                "direct-broken",
            )
            .unwrap();
            assert_eq!(planned.changes.participant_changes.len(), 1);
            assert_eq!(
                planned.changes.participant_changes[0].current_hp,
                Some(after)
            );
        }
    }
    #[test]
    fn pending_apply_rejects_contradiction_and_preserves_recorded_choices_when_omitted() {
        let (worker, key, participant) = fixture();
        event(
            &worker,
            &key,
            &participant,
            "recorded-lethal",
            HpEventInput::ResolvedDamage { amount: 10 },
            HpEventContextView {
                lethality: Some(DamageLethalityView::Lethal),
                enemy_source: Some(false),
                responsible_participant_key: Some(participant.clone()),
                keep_current_initiative: Some(true),
                keep_upright: Some(true),
                base_death_threshold: Some(5),
                attack_or_spell_damage: Some(false),
                natural_rest_healing: Some(false),
                ..Default::default()
            },
        );
        let pending = worker
            .pending_encounter_hp_lifecycle(&key)
            .unwrap()
            .remove(0);
        let request = |id: &str, context| ResolveEncounterHpLifecycleRequest {
            resolution: HpLifecycleResolutionView::Apply,
            request_id: id.into(),
            expected_encounter_revision: worker.encounter(&key).unwrap().revision,
            consequence_key: pending.consequence_key.clone(),
            context,
        };
        let contradiction = request(
            "contradict-lethality",
            HpEventContextView {
                lethality: Some(DamageLethalityView::Nonlethal),
                ..Default::default()
            },
        );
        assert!(
            worker
                .preview_encounter_hp_lifecycle(&key, contradiction.clone())
                .is_err()
        );
        assert!(
            worker
                .resolve_encounter_hp_lifecycle(&key, contradiction)
                .is_err()
        );
        let false_upright = request(
            "contradict-upright",
            HpEventContextView {
                critical: Some(false),
                keep_upright: Some(false),
                ..Default::default()
            },
        );
        assert!(
            worker
                .resolve_encounter_hp_lifecycle(&key, false_upright)
                .is_err()
        );
        for (id, context) in [
            (
                "contradict-enemy",
                HpEventContextView {
                    critical: Some(false),
                    enemy_source: Some(true),
                    ..Default::default()
                },
            ),
            (
                "contradict-threshold",
                HpEventContextView {
                    critical: Some(false),
                    base_death_threshold: Some(4),
                    ..Default::default()
                },
            ),
            (
                "contradict-initiative",
                HpEventContextView {
                    critical: Some(false),
                    keep_current_initiative: Some(false),
                    ..Default::default()
                },
            ),
            (
                "contradict-waking-qualification",
                HpEventContextView {
                    critical: Some(false),
                    natural_rest_healing: Some(true),
                    ..Default::default()
                },
            ),
            (
                "contradict-damage-kind",
                HpEventContextView {
                    critical: Some(false),
                    attack_or_spell_damage: Some(true),
                    ..Default::default()
                },
            ),
        ] {
            assert!(
                worker
                    .resolve_encounter_hp_lifecycle(&key, request(id, context))
                    .is_err()
            );
        }
        let omitted = request(
            "fill-critical",
            HpEventContextView {
                critical: Some(false),
                ..Default::default()
            },
        );
        let preview = worker
            .preview_encounter_hp_lifecycle(&key, omitted.clone())
            .unwrap();
        assert!(preview.required_facts.is_empty());
        assert_eq!(
            preview.context.responsible_participant_key,
            Some(participant.clone())
        );
        assert_eq!(preview.context.enemy_source, Some(false));
        assert_eq!(preview.context.natural_rest_healing, Some(false));
        assert_eq!(preview.context.lethality, Some(DamageLethalityView::Lethal));
        assert_eq!(preview.context.keep_upright, Some(true));
        assert_eq!(preview.context.keep_current_initiative, Some(true));
        assert_eq!(preview.context.base_death_threshold, Some(5));
        assert_eq!(preview.context.attack_or_spell_damage, Some(false));
        worker
            .resolve_encounter_hp_lifecycle(&key, omitted.clone())
            .unwrap();
        worker
            .resolve_encounter_hp_lifecycle(&key, omitted)
            .unwrap();
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].current_hp, Some(0));
        let rule = |rule| modeled_condition_by_rule(rule).unwrap().condition_ref;
        assert!(
            detail.participants[0]
                .conditions
                .iter()
                .any(|condition| condition.condition_key.as_deref()
                    == Some(rule(ConditionRule::Dying))
                    && condition.value == Some(1))
        );
        assert!(!detail.participants[0].conditions.iter().any(|condition| {
            condition.condition_key.as_deref() == Some(rule(ConditionRule::Prone))
        }));
        assert!(
            worker
                .pending_encounter_hp_lifecycle(&key)
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn known_initiative_choice_cannot_be_replaced_by_ordinary_pending_apply() {
        let (worker, key, participant) = fixture();
        event(
            &worker,
            &key,
            &participant,
            "known-knockout",
            HpEventInput::ResolvedDamage { amount: 10 },
            HpEventContextView {
                lethality: Some(DamageLethalityView::Nonlethal),
                keep_current_initiative: Some(false),
                keep_upright: Some(true),
                ..Default::default()
            },
        );
        let pending = worker
            .pending_encounter_hp_lifecycle(&key)
            .unwrap()
            .remove(0);
        assert!(pending.initiative_only);
        let mut request = ResolveEncounterHpLifecycleRequest {
            resolution: HpLifecycleResolutionView::Apply,
            request_id: "change-initiative".into(),
            expected_encounter_revision: worker.encounter(&key).unwrap().revision,
            consequence_key: pending.consequence_key,
            context: HpEventContextView {
                keep_current_initiative: Some(true),
                ..Default::default()
            },
        };
        assert!(
            worker
                .preview_encounter_hp_lifecycle(&key, request.clone())
                .is_err()
        );
        assert!(
            worker
                .resolve_encounter_hp_lifecycle(&key, request.clone())
                .is_err()
        );
        request.resolution = HpLifecycleResolutionView::KeepCurrentState;
        request.request_id = "correct-initiative".into();
        worker
            .resolve_encounter_hp_lifecycle(&key, request)
            .unwrap();
        assert!(
            worker
                .pending_encounter_hp_lifecycle(&key)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].current_hp,
            Some(0)
        );
    }
}
