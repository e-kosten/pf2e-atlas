//! Strike actions use the same frozen checks and atomic completion as casting.
use super::rules::action_context::{ActionFact, ActionJudgment, ActionKind};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::*;

impl AtlasAppService {
    pub fn preview_encounter_strike_attempt(
        &self,
        encounter_ref: &str,
        request: PreviewEncounterStrikeAttemptRequest,
    ) -> AppServiceResult<EncounterStrikeAttemptPreviewView> {
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| Ok(prepare(workspace, &encounter_ref, request)?.0),
        )
    }
    pub fn begin_encounter_strike_attempt(
        &self,
        encounter_ref: &str,
        request: BeginEncounterStrikeAttemptRequest,
    ) -> AppServiceResult<EncounterActionMutationView> {
        if request.strike.roll_kind != StrikeRollKind::Attack {
            return Err(AppServiceError::invalid_request(
                "Choose an attack roll for a Strike action.",
            ));
        }
        let intent = serde_json::json!({"action":"begin_strike_attempt","request":request});
        let (key, recovered) =
            self.casting_receipt(encounter_ref, &request.strike.request_id, &intent)?;
        if let Some(receipt) = recovered {
            return Ok(EncounterActionMutationView {
                encounter_key: receipt.encounter_key,
                attempt_key: receipt.attempt_key,
            });
        }
        let lookup_key = key.clone();
        let preview = PreviewEncounterStrikeAttemptRequest {
            strike: EncounterStrikeOptionsRequest {
                participant_key: request.strike.participant_key.clone(),
                reference: request.strike.reference.clone(),
            },
            judgments: request.judgments.clone(),
            target: request.target.clone(),
            confusion: request.confusion.clone(),
        };
        let (view, mut prepared) = self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| prepare(workspace, &lookup_key, preview),
        )?;
        if !view.action.issues.is_empty() {
            return Err(AppServiceError::invalid_request(
                view.action
                    .issues
                    .iter()
                    .map(|issue| issue.reason.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            ));
        }
        if view.action.targeting.as_ref().is_some_and(|target| {
            target.prevents_effect
                || target.requires_guessed_location
                || !target.required_facts.is_empty()
        }) {
            return Err(AppServiceError::invalid_request(
                "Confirm target detection and choose a guessed location when required.",
            ));
        }
        let automatic_self_hit = matches!(
            prepared.context.action,
            FrozenAction::Strike {
                automatic_self_hit: true,
                ..
            }
        );
        let modifier = request
            .strike
            .modifier_override
            .or_else(|| {
                view.options
                    .attacks
                    .iter()
                    .find(|attack| attack.attack_number == request.strike.attack_number)
                    .map(|attack| attack.modifier)
            })
            .or_else(|| automatic_self_hit.then_some(0))
            .ok_or_else(|| AppServiceError::invalid_request("Enter an attack modifier."))?;
        if !(1..=3).contains(&request.strike.attack_number) || !(-10000..=10000).contains(&modifier)
        {
            return Err(AppServiceError::invalid_request(
                "Choose a valid attack number and modifier.",
            ));
        }
        let retained = if let Some(original) = &request.existing_roll {
            if original.participant_key != request.strike.participant_key
                || original.reference != request.strike.reference
                || original.attack_number != request.strike.attack_number
                || original.mode != request.strike.mode
                || original.modifier_override != request.strike.modifier_override
                || original.roll_kind != StrikeRollKind::Attack
            {
                return Err(AppServiceError::invalid_request(
                    "Original attack identity does not match this context.",
                ));
            }
            let original_intent = serde_json::json!({"action":"strike", "request":original});
            let receipt = self
                .with_state(|session| {
                    Ok(session.local_state().encounters().strike_roll_receipt(
                        &key,
                        &original.request_id,
                        &original_intent,
                    )?)
                })?
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Original attack roll was not found.")
                })?;
            let StoredStrikeOutcome::Attack { outcome, .. } = receipt.roll.outcome else {
                return Err(AppServiceError::invalid_request(
                    "Original receipt is not an attack roll.",
                ));
            };
            Some(RetainedStrikeRoll {
                request_id: original.request_id.clone(),
                outcome,
            })
        } else {
            None
        };
        if let FrozenAction::Strike {
            retained_roll,
            attack_number,
            modifier: stored,
            mode,
            ..
        } = &mut prepared.context.action
        {
            *retained_roll = retained;
            *attack_number = request.strike.attack_number;
            *stored = Some(modifier);
            *mode = match request.strike.mode {
                RollMode::Normal => ActionRollMode::Normal,
                RollMode::Fortune => ActionRollMode::Fortune,
                RollMode::Misfortune => ActionRollMode::Misfortune,
            };
        }
        self.with_write(move |session| {
            if let Some(receipt) = session.local_state().encounters().action_attempt_receipt(
                &key,
                &request.strike.request_id,
                &intent,
            )? {
                return Ok(EncounterActionMutationView {
                    encounter_key: receipt.encounter_key,
                    attempt_key: receipt.attempt_key,
                });
            }
            session.require_reference_revision(prepared.context.reference_revision)?;
            let receipt = session.local_state().encounters().begin_action_attempt(
                &key,
                &request.strike.request_id,
                &intent,
                ParticipantRollGuard {
                    expected_revision: request.strike.expected_revision,
                    participant_key: &request.strike.participant_key,
                    expected_participant_revision: request.strike.expected_participant_revision,
                },
                &prepared,
                super::action_attempts::complete_action,
            )?;
            Ok(EncounterActionMutationView {
                encounter_key: receipt.encounter_key,
                attempt_key: receipt.attempt_key,
            })
        })
    }
    pub fn encounter_strike_attempts(
        &self,
        encounter_ref: &str,
    ) -> AppServiceResult<EncounterStrikeAttemptsView> {
        self.with_state(|session| {
            let encounter = session
                .local_state()
                .encounters()
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let attempts = session
                .local_state()
                .encounters()
                .action_attempts(&encounter.encounter_key)?
                .into_iter()
                .filter_map(|attempt| {
                    if !matches!(attempt.context.action, FrozenAction::Strike { .. }) {
                        return None;
                    }
                    let (secret, shareable_summary) = super::action_attempts::disclosure(&attempt);
                    Some(EncounterStrikeAttemptView {
                        attempt_key: attempt.attempt_key,
                        participant_key: attempt.context.participant_key,
                        revision: attempt.revision,
                        status: match attempt.status {
                            ActionAttemptStatus::AwaitingCheck => {
                                EncounterActionAttemptStatusView::AwaitingCheck
                            }
                            ActionAttemptStatus::Ready => EncounterActionAttemptStatusView::Ready,
                            ActionAttemptStatus::Disrupted => {
                                EncounterActionAttemptStatusView::Disrupted
                            }
                            ActionAttemptStatus::Abandoned => {
                                EncounterActionAttemptStatusView::Abandoned
                            }
                        },
                        secret,
                        shareable_summary,
                        title: attempt.context.title,
                        checks: attempt
                            .context
                            .checks
                            .iter()
                            .map(|check| EncounterActionCheckView {
                                dc: check.dc,
                                sources: check
                                    .sources
                                    .iter()
                                    .map(|source| EncounterActionCheckSourceView {
                                        condition_id: source.condition_id,
                                        name: source.name.clone(),
                                    })
                                    .collect(),
                            })
                            .collect(),
                        pending_check: if attempt.status == ActionAttemptStatus::AwaitingCheck {
                            attempt
                                .context
                                .checks
                                .get(attempt.outcomes.len())
                                .map(|check| EncounterActionCheckView {
                                    dc: check.dc,
                                    sources: check
                                        .sources
                                        .iter()
                                        .map(|source| EncounterActionCheckSourceView {
                                            condition_id: source.condition_id,
                                            name: source.name.clone(),
                                        })
                                        .collect(),
                                })
                        } else {
                            None
                        },
                        outcomes: attempt
                            .outcomes
                            .into_iter()
                            .map(|result| EncounterActionCheckResultView {
                                roll: result.roll,
                                table_result: result.table_result,
                                success: result.success,
                            })
                            .collect(),
                        automatic_self_hit: matches!(
                            attempt.context.action,
                            FrozenAction::Strike {
                                automatic_self_hit: true,
                                ..
                            }
                        ),
                        attack: match attempt.completion {
                            Some(ActionCompletion::StrikeRoll { roll }) => Some(roll),
                            _ => match attempt.context.action {
                                FrozenAction::Strike {
                                    retained_roll: Some(retained),
                                    ..
                                } => Some(retained.outcome),
                                _ => None,
                            },
                        },
                    })
                })
                .collect();
            Ok(EncounterStrikeAttemptsView {
                encounter_key: encounter.encounter_key,
                attempts,
            })
        })
    }
}

fn prepare(
    workspace: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    request: PreviewEncounterStrikeAttemptRequest,
) -> AppServiceResult<(EncounterStrikeAttemptPreviewView, PreparedActionAttempt)> {
    let (options, activity) =
        super::strike_roll::resolve_strike(workspace, encounter_ref, request.strike.clone())?;
    let snapshot = workspace
        .local_state()
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
    let participant = snapshot
        .participants
        .iter()
        .find(|participant| participant.participant_key == request.strike.participant_key)
        .ok_or_else(|| {
            AppServiceError::invalid_request("Participant is no longer in this encounter.")
        })?;
    let confusion = super::action_attempts::checked_confusion(
        &snapshot.participants,
        participant,
        request.confusion.as_ref(),
    )?;
    let target = super::action_attempts::effective_confusion_target(
        participant,
        confusion.as_ref(),
        request.target.as_ref(),
    )?;
    let automatic_self_hit = matches!(confusion, Some(ConfusionChoice::NoViableTargets));
    let relationships = workspace
        .local_state()
        .encounters()
        .relationships(&snapshot.encounter.encounter_key)?;
    let mut facts = vec![];
    for (fact, value) in [
        (ActionFact::Reaction, false),
        (ActionFact::Attack, true),
        (ActionFact::MentalOnly, false),
        (ActionFact::VoluntaryIngestion, false),
    ] {
        facts.push(ActionJudgment {
            source_id: None,
            fact,
            value,
        });
    }
    if activity.traits_known {
        for (fact, key) in [
            (ActionFact::Manipulate, "manipulate"),
            (ActionFact::Auditory, "auditory"),
            (ActionFact::Concentrate, "concentrate"),
            (ActionFact::Move, "move"),
        ] {
            facts.push(ActionJudgment {
                source_id: None,
                fact,
                value: activity.traits.iter().any(|value| value == key),
            });
        }
    }
    let target_key = target
        .as_ref()
        .map(|target| target.participant_key.as_str());
    let judged = super::action_attempts::apply_judgments(
        participant,
        super::rules::action_context::ActionContext {
            kind: ActionKind::Strike,
            reaction: Some(false),
            other_participant: target_key,
            confusion: confusion.as_ref(),
            relationships: &relationships,
            facts: &facts,
        },
        &request.judgments,
    )?;
    let facts = judged.facts;
    let evaluated = judged.evaluation;
    let mut issues = evaluated
        .issues
        .into_iter()
        .map(|issue| super::action_attempts::issue_view(issue, participant, &relationships))
        .collect::<Vec<_>>();
    if participant.defeated
        || participant.hazard_disabled
        || participant.mortality == MortalityState::Dead
    {
        issues.push(EncounterActionIssueView {
            subject: None,
            source_condition_id: 0,
            condition_name: "Participant".into(),
            controller_key: None,
            instructions: None,
            kind: EncounterActionIssueKindView::Prohibited,
            fact: None,
            reason: super::rules::capabilities::cannot_act_reason(participant)
                .unwrap_or("This participant cannot act.")
                .into(),
        });
    }
    let disruption = evaluated.disruption.map(|check| FrozenActionCheck {
        stage: ActionCheckStage::Disruption,
        dc: check.dc,
        sources: check
            .source_ids
            .into_iter()
            .map(|id| ActionCheckSource {
                condition_id: id,
                name: participant
                    .conditions
                    .iter()
                    .find(|source| source.condition_id == id)
                    .map(|source| source.name.clone())
                    .unwrap_or_default(),
            })
            .collect(),
    });
    let mut checks = disruption.into_iter().collect::<Vec<_>>();
    let observer_source = super::hydration::resolve_record_ref(
        workspace.retrieval_mut(),
        &request.strike.reference.selection.record_key,
    )?;
    let sensory_baseline = super::rules::source_baseline(
        &observer_source.source,
        workspace.retrieval_mut().source_fingerprint(),
        None,
    );
    let prepared_targeting = if automatic_self_hit {
        None
    } else {
        super::action_preview::targeting_for_action(
            &snapshot.participants,
            &relationships,
            &participant.participant_key,
            target.as_ref(),
            sensory_baseline.as_ref(),
            activity.traits_known.then_some(activity.traits.as_slice()),
        )?
    };
    let saved_target = prepared_targeting
        .as_ref()
        .map(|target| target.saved.clone());
    let targeting = prepared_targeting.map(|target| target.requirements);
    if let Some(targeting) = &targeting
        && let Some(dc) = targeting.flat_check_dc
    {
        checks.push(super::action_preview::targeting_check(
            &snapshot.participants,
            targeting,
            dc,
        ));
    }
    let artifact_generation = workspace.retrieval_mut().artifact_generation()?;
    let context = FrozenActionContext {
        participant_key: participant.participant_key.clone(),
        source_record_key: request.strike.reference.selection.record_key,
        title: activity.label.clone(),
        target: saved_target,
        confusion: confusion.clone(),
        action: FrozenAction::Strike {
            activity_id: activity.activity_id,
            attack_number: 1,
            modifier: options.attacks.first().map(|attack| attack.modifier),
            automatic_self_hit,
            mode: ActionRollMode::Normal,
            retained_roll: None,
        },
        source_fingerprint: workspace.retrieval_mut().source_fingerprint().into(),
        artifact_generation: artifact_generation.clone(),
        reference_revision: workspace.reference_revision()?,
        judgments: facts
            .iter()
            .map(|fact| FrozenActionJudgment {
                source_condition_id: fact.source_id,
                fact: super::action_attempts::fact_to_saved(fact.fact),
                value: fact.value,
            })
            .collect(),
        cost: None,
        checks,
    };
    let action = EncounterActionPreviewView {
        considers_target_ally: super::rules::action_context::affiliation(
            participant,
            target_key,
            None,
        )
        .0,
        can_flank: super::rules::action_context::affiliation(participant, target_key, None).1,
        encounter_key: snapshot.encounter.encounter_key,
        participant_key: participant.participant_key.clone(),
        expected_revision: snapshot.encounter.revision,
        expected_participant_revision: participant.revision,
        issues,
        confusion: super::action_attempts::confusion_view(
            participant,
            confusion.as_ref(),
            ActionKind::Strike,
            true,
        ),
        disruption: context
            .checks
            .iter()
            .find(|check| check.stage == ActionCheckStage::Disruption)
            .map(|check| EncounterActionCheckView {
                dc: check.dc,
                sources: check
                    .sources
                    .iter()
                    .map(|source| EncounterActionCheckSourceView {
                        condition_id: source.condition_id,
                        name: source.name.clone(),
                    })
                    .collect(),
            }),
        targeting,
        quickened_use: None,
        request: None,
        sensory: None,
        forced_movement: None,
        prone_cover: None,
    };
    Ok((
        EncounterStrikeAttemptPreviewView {
            targeting_check: context
                .checks
                .iter()
                .find(|check| check.stage == ActionCheckStage::Targeting)
                .map(|check| EncounterActionCheckView {
                    dc: check.dc,
                    sources: check
                        .sources
                        .iter()
                        .map(|source| EncounterActionCheckSourceView {
                            condition_id: source.condition_id,
                            name: source.name.clone(),
                        })
                        .collect(),
                }),
            options,
            action,
            artifact_generation,
        },
        PreparedActionAttempt {
            context,
            resources: None,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_attack_receipt_survives_optional_context_checks_and_retries() {
        let fixture = crate::test_support::encounter_fixture_worker();
        let encounter = fixture
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Strike checks".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        fixture
            .worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: encounter.encounter_key.clone(),
                record_ref: "Test Creature 1".into(),
                quantity: 2,
                initiative: None,
            })
            .unwrap();
        let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
        let unknown = fixture
            .worker
            .preview_encounter_action(
                &encounter.encounter_key,
                PreviewEncounterActionRequest {
                    normally_ally: None,
                    confusion: None,
                    participant_key: current.participants[0].participant_key.clone(),
                    action: EncounterActionKindView::Strike,
                    reaction: None,
                    target: Some(EncounterActionTargetView {
                        participant_key: current.participants[1].participant_key.clone(),
                        mode: EncounterTargetModeView::Direct,
                        vision_only_precise: None,
                        distance_feet: None,
                        guessed_location_correct: None,
                        visual_effect: None,
                        auditory_effect: None,
                    }),
                    judgments: vec![],
                    extra_action: None,
                    request_context: None,
                    sensory: None,
                    forced_movement: None,
                    prone_cover: None,
                },
            )
            .unwrap();
        assert!(!unknown.targeting.unwrap().required_facts.is_empty());
        fixture
            .worker
            .set_encounter_relationship(
                &encounter.encounter_key,
                SetEncounterRelationshipRequest {
                    layer: atlas_app_model::RelationshipLayerView::Baseline,
                    request_id: "observed".into(),
                    expected_encounter_revision: current.revision,
                    subject_key: current.participants[1].participant_key.clone(),
                    other_key: current.participants[0].participant_key.clone(),
                    property: RelationshipPropertyView::Detection,
                    knowledge: Some(RelationshipKnowledgeView::Known {
                        value: RelationshipValueView::Detection(DetectionStateView::Observed),
                        provenance: RelationshipProvenanceView::Table,
                    }),
                },
            )
            .unwrap();
        let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
        let subject = &current.participants[1];
        let definition = super::super::conditions::modeled_condition_by_rule(
            super::super::conditions::ConditionRule::Concealed,
        )
        .unwrap();
        fixture
            .worker
            .add_encounter_participant_condition(
                &encounter.encounter_key,
                AddEncounterParticipantConditionRequest {
                    request_id: "concealed".into(),
                    expected_encounter_revision: current.revision,
                    event_intent: Default::default(),
                    decisions: Default::default(),
                    participant_key: subject.participant_key.clone(),
                    condition_ref: Some(definition.condition_ref.into()),
                    name: None,
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
            )
            .unwrap();
        let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
        let actor = &current.participants[0];
        let RecordBodyView::Creature(body) = &actor.presentation.as_ref().unwrap().body else {
            panic!("actor");
        };
        let strike = body
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .find(|activity| activity.title == "Claw")
            .unwrap();
        let roll = RollEncounterStrikeRequest {
            request_id: "strike-begin".into(),
            expected_revision: current.revision,
            expected_participant_revision: actor.revision,
            participant_key: actor.participant_key.clone(),
            reference: EncounterReferenceRequest {
                artifact_generation: current.artifact_generation.clone().unwrap(),
                selection: RecordDetailRequest {
                    record_key: strike.navigation.record_key.clone(),
                    owners: strike.navigation.owners.clone(),
                    source_fingerprint: strike.navigation.source_fingerprint.clone(),
                    spell_rank: None,
                    fields: vec![],
                    passage: None,
                },
            },
            attack_number: 1,
            mode: RollMode::Normal,
            modifier_override: None,
            roll_kind: StrikeRollKind::Attack,
        };
        let baseline = fixture
            .worker
            .encounter_strike_options(
                &encounter.encounter_key,
                EncounterStrikeOptionsRequest {
                    participant_key: roll.participant_key.clone(),
                    reference: roll.reference.clone(),
                },
            )
            .unwrap();
        for rule in [
            super::super::conditions::ConditionRule::Frightened,
            super::super::conditions::ConditionRule::Blinded,
        ] {
            let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
            let definition = super::super::conditions::modeled_condition_by_rule(rule).unwrap();
            fixture
                .worker
                .add_encounter_participant_condition(
                    &encounter.encounter_key,
                    AddEncounterParticipantConditionRequest {
                        request_id: format!("attack-hint-{rule:?}"),
                        expected_encounter_revision: current.revision,
                        participant_key: roll.participant_key.clone(),
                        condition_ref: Some(definition.condition_ref.into()),
                        value: (rule == super::super::conditions::ConditionRule::Frightened)
                            .then_some(2),
                        name: None,
                        event_intent: Default::default(),
                        decisions: Default::default(),
                        source_participant_key: None,
                        scope: Default::default(),
                        extra_action: Default::default(),
                        immunity_override: false,
                        parameters: Default::default(),
                        origin: None,
                        duration: None,
                        note: None,
                    },
                )
                .unwrap();
        }
        let mut roll = roll;
        let options = fixture
            .worker
            .encounter_strike_options(
                &encounter.encounter_key,
                EncounterStrikeOptionsRequest {
                    participant_key: roll.participant_key.clone(),
                    reference: roll.reference.clone(),
                },
            )
            .unwrap();
        roll.expected_revision = options.expected_revision;
        roll.expected_participant_revision = options.expected_participant_revision;
        assert_eq!(
            options.attacks[0].modifier,
            baseline.attacks[0].modifier - 2
        );
        assert!(
            options
                .applied
                .iter()
                .any(|hint| hint.label.contains("Frightened") && hint.reason == "-2 attack")
        );
        assert!(
            options
                .conditional
                .iter()
                .any(|hint| hint.label.contains("Blinded") && hint.reason == "Check detection")
        );
        assert!(
            !options
                .conditional
                .iter()
                .any(|hint| hint.reason.contains("Concealed"))
        );
        let mut invalid_source = roll.clone();
        invalid_source.request_id = "invalid-source".into();
        invalid_source.reference.selection.source_fingerprint = Some("wrong-source".into());
        assert!(
            fixture
                .worker
                .roll_encounter_strike(&encounter.encounter_key, invalid_source)
                .is_err()
        );
        let original = fixture
            .worker
            .roll_encounter_strike(&encounter.encounter_key, roll.clone())
            .unwrap();
        assert!(original.attack.is_some());
        assert_eq!(
            fixture
                .worker
                .roll_encounter_strike(&encounter.encounter_key, roll.clone())
                .unwrap(),
            original
        );

        let request = BeginEncounterStrikeAttemptRequest {
            existing_roll: Some(roll.clone()),
            confusion: None,
            strike: roll,
            judgments: vec![],
            target: Some(EncounterActionTargetView {
                participant_key: subject.participant_key.clone(),
                mode: EncounterTargetModeView::Direct,
                vision_only_precise: None,
                distance_feet: None,
                guessed_location_correct: None,
                visual_effect: None,
                auditory_effect: None,
            }),
        };
        let mut mismatched = request.clone();
        mismatched.strike.mode = RollMode::Fortune;
        assert!(
            fixture
                .worker
                .begin_encounter_strike_attempt(&encounter.encounter_key, mismatched)
                .is_err()
        );
        let mut wrong_source = request.clone();
        wrong_source
            .existing_roll
            .as_mut()
            .unwrap()
            .reference
            .selection
            .source_fingerprint = Some("wrong-source".into());
        assert!(
            fixture
                .worker
                .begin_encounter_strike_attempt(&encounter.encounter_key, wrong_source)
                .is_err()
        );
        let context_request = request.clone();
        let receipt = fixture
            .worker
            .begin_encounter_strike_attempt(&encounter.encounter_key, request.clone())
            .unwrap();
        assert_eq!(
            fixture
                .worker
                .begin_encounter_strike_attempt(&encounter.encounter_key, request)
                .unwrap(),
            receipt
        );
        let saved = fixture
            .worker
            .encounter_strike_attempts(&encounter.encounter_key)
            .unwrap()
            .attempts
            .remove(0);
        assert_eq!(
            saved.status,
            EncounterActionAttemptStatusView::AwaitingCheck
        );
        assert_eq!(saved.checks[0].dc, 5);
        assert_eq!(saved.attack, original.attack);
        let check = ResolveEncounterActionCheckRequest {
            request_id: "target-check".into(),
            attempt_key: receipt.attempt_key,
            expected_attempt_revision: saved.revision,
            outcome: EncounterActionCheckInputView::TableResult { result: 20 },
        };
        fixture
            .worker
            .resolve_encounter_action_check(&encounter.encounter_key, check.clone())
            .unwrap();
        let first = fixture
            .worker
            .encounter_strike_attempts(&encounter.encounter_key)
            .unwrap()
            .attempts
            .remove(0);
        fixture
            .worker
            .resolve_encounter_action_check(&encounter.encounter_key, check)
            .unwrap();
        let second = fixture
            .worker
            .encounter_strike_attempts(&encounter.encounter_key)
            .unwrap()
            .attempts
            .remove(0);
        assert_eq!(first, second);
        assert_eq!(second.status, EncounterActionAttemptStatusView::Ready);
        assert_eq!(second.attack, original.attack);
        let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
        let actor = current
            .participants
            .iter()
            .find(|participant| {
                participant.participant_key == context_request.strike.participant_key
            })
            .unwrap();
        let mut duplicate = context_request;
        duplicate.strike.request_id = "second-context-for-same-roll".into();
        duplicate.strike.expected_revision = current.revision;
        duplicate.strike.expected_participant_revision = actor.revision;
        assert!(
            fixture
                .worker
                .begin_encounter_strike_attempt(&encounter.encounter_key, duplicate)
                .is_err()
        );
        assert_eq!(
            fixture
                .worker
                .encounter_strike_attempts(&encounter.encounter_key)
                .unwrap()
                .attempts
                .len(),
            1
        );
    }
}
