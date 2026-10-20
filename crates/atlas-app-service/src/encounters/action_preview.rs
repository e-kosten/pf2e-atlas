//! Contextual action policy projection; queries never mutate saved state.
use super::rules::action_context::{ActionFact, ActionJudgment, ActionKind};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;

impl AtlasAppService {
    pub fn preview_encounter_action(
        &self,
        encounter_ref: &str,
        request: PreviewEncounterActionRequest,
    ) -> AppServiceResult<EncounterActionPreviewView> {
        let requires_source =
            request.sensory.is_some() || request.prone_cover.is_some() || request.target.is_some();
        let snapshot = self.with_state(|session| {
            session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))
        })?;
        let source_expected = snapshot
            .participants
            .iter()
            .find(|participant| participant.participant_key == request.participant_key)
            .and_then(|participant| participant.record_key.as_deref())
            .is_some_and(|key| atlas_domain::RecordKey::parse(key).is_ok());
        if requires_source && source_expected {
            let encounter_ref = encounter_ref.to_owned();
            self.with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |workspace| {
                    let snapshot = workspace
                        .local_state()
                        .encounters()
                        .get_with_participants(&encounter_ref)?
                        .ok_or_else(|| super::projection::encounter_not_found(&encounter_ref))?;
                    let relationships = workspace
                        .local_state()
                        .encounters()
                        .relationships(&snapshot.encounter.encounter_key)?;
                    let source = if let Some(key) = snapshot
                        .participants
                        .iter()
                        .find(|participant| participant.participant_key == request.participant_key)
                        .and_then(|participant| participant.record_key.as_deref())
                        .and_then(|key| atlas_domain::RecordKey::parse(key).ok())
                    {
                        workspace
                            .retrieval_mut()
                            .get_record(atlas_search::GetRecordRequest {
                                record_key: &key,
                                selected_content: &[],
                            })?
                    } else {
                        None
                    };
                    let baseline = source.as_ref().and_then(|source| {
                        super::rules::source_baseline(
                            &source.source,
                            workspace.retrieval_mut().source_fingerprint(),
                            None,
                        )
                        .map(|mut baseline| {
                            baseline.contextual_modifiers =
                                super::rules::relationships::observer_defense_modifiers(
                                    &request.participant_key,
                                    &snapshot.participants,
                                    &relationships,
                                );
                            let adjustment = !matches!(
                                atlas_record::source_record::SourceQueryView::new(
                                    source.source.source(),
                                    source.source.key().pack().as_str(),
                                    ""
                                )
                                .actor()
                                .authored_adjustment(),
                                atlas_record::source_record::SourceFieldView::NotApplicable
                            );
                            (baseline, adjustment)
                        })
                    });
                    project_action(snapshot, relationships, request, baseline)
                },
            )
        } else {
            self.with_state(|session| {
                let snapshot = session
                    .local_state()
                    .encounters()
                    .get_with_participants(encounter_ref)?
                    .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
                let relationships = session
                    .local_state()
                    .encounters()
                    .relationships(&snapshot.encounter.encounter_key)?;
                project_action(snapshot, relationships, request, None)
            })
        }
    }
}
fn project_action(
    snapshot: atlas_local_state::EncounterWithParticipants,
    relationships: Vec<atlas_local_state::DirectedRelationship>,
    request: PreviewEncounterActionRequest,
    baseline: Option<(super::stat_block::StatBlockView, bool)>,
) -> AppServiceResult<EncounterActionPreviewView> {
    let participant = snapshot
        .participants
        .iter()
        .find(|participant| participant.participant_key == request.participant_key)
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
    let automatic_self_hit = matches!(
        confusion,
        Some(atlas_local_state::ConfusionChoice::NoViableTargets)
    );
    let kind = action_kind(request.action);
    let facts = builtin_facts(kind);
    let reaction = if matches!(kind, ActionKind::Cast | ActionKind::Other) {
        request.reaction
    } else {
        Some(false)
    };
    if request.reaction == Some(true) && !matches!(kind, ActionKind::Cast | ActionKind::Other) {
        return Err(AppServiceError::invalid_request(
            "This basic action is not a reaction.",
        ));
    }
    let other = target
        .as_ref()
        .map(|target| target.participant_key.as_str());
    let judged = super::action_attempts::apply_judgments(
        participant,
        super::rules::action_context::ActionContext {
            kind,
            reaction,
            other_participant: other,
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
    if let Some(reason) = super::rules::capabilities::cannot_act_reason(participant)
        && (participant.defeated
            || participant.hazard_disabled
            || participant.mortality == atlas_local_state::MortalityState::Dead)
    {
        issues.push(EncounterActionIssueView {
            subject: None,
            source_condition_id: 0,
            condition_name: "Participant".into(),
            controller_key: None,
            instructions: None,
            kind: EncounterActionIssueKindView::Prohibited,
            fact: None,
            reason: reason.into(),
        });
    }
    let request_requirements = if kind == ActionKind::Request {
        let context = request.request_context.clone().unwrap_or_default();
        Some(if let Some(target) = &target {
            let receiver = snapshot
                .participants
                .iter()
                .find(|participant| participant.participant_key == target.participant_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Recipient is no longer in this encounter.")
                })?;
            let attitude = super::rules::relationships::relationship_knowledge(
                receiver,
                &participant.participant_key,
                atlas_local_state::RelationshipProperty::Attitude,
                &relationships,
            );
            let result = super::rules::relationships::request_requirements(
                &attitude,
                context.reasonable,
                context.against_goals,
            );
            EncounterRequestRequirementsView {
                outcome: match result.outcome {
                    super::rules::relationships::RequestOutcome::CheckRequired => {
                        EncounterRequestOutcomeView::CheckRequired
                    }
                    super::rules::relationships::RequestOutcome::Prohibited => {
                        EncounterRequestOutcomeView::Prohibited
                    }
                    super::rules::relationships::RequestOutcome::NeedsContext => {
                        EncounterRequestOutcomeView::NeedsContext
                    }
                },
                required_facts: result.required_facts,
                reason: result.reason,
            }
        } else {
            EncounterRequestRequirementsView {
                outcome: EncounterRequestOutcomeView::NeedsContext,
                required_facts: vec!["Choose a recipient.".into()],
                reason: "Choose a recipient.".into(),
            }
        })
    } else {
        None
    };
    let targeting = if kind == ActionKind::Request || automatic_self_hit {
        None
    } else {
        targeting_for_action(
            &snapshot.participants,
            &relationships,
            &participant.participant_key,
            target.as_ref(),
            baseline.as_ref().map(|(baseline, _)| baseline),
            None,
        )?
        .map(|prepared| prepared.requirements)
    };
    let quickened_use = if let Some(action) = request.extra_action {
        let converted =
            super::workflow::condition_extra_action_input(ConditionExtraActionView::Restricted {
                actions: vec![action],
                guidance: None,
            })?;
        match converted {
            atlas_local_state::ConditionExtraAction::Restricted { actions, .. } => {
                actions.into_iter().next().map(|action| {
                    let result =
                        super::rules::action_context::quickened_use(participant, action, other);
                    EncounterQuickenedUseView {
                        allowed: result.allowed,
                        allowing_source_ids: result.allowing_source_ids,
                        unknown_source_ids: result.unknown_source_ids,
                    }
                })
            }
            _ => None,
        }
    } else {
        None
    };
    let disruption = evaluated.disruption.map(|check| EncounterActionCheckView {
        dc: check.dc,
        sources: check
            .source_ids
            .into_iter()
            .map(|id| EncounterActionCheckSourceView {
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
    let baseline = baseline
        .as_ref()
        .map(|(baseline, adjustment)| (baseline, *adjustment));
    let sensory = request
        .sensory
        .map(|context| {
            if context
                .distance_feet
                .is_some_and(|distance| !distance.is_finite() || distance < 0.0)
            {
                return Err(AppServiceError::invalid_request(
                    "Enter a nonnegative distance.",
                ));
            }
            use super::rules::sensory_context::{PerceptionBasis as B, SensoryContext};
            let result = super::rules::sensory_context::evaluate_senses(
                participant,
                baseline,
                SensoryContext {
                    basis: match context.basis {
                        EncounterPerceptionBasisView::General => B::General,
                        EncounterPerceptionBasisView::VisualOnly => B::VisualOnly,
                        EncounterPerceptionBasisView::AuditoryOnly => B::AuditoryOnly,
                        EncounterPerceptionBasisView::MixedSound => B::MixedSound,
                        EncounterPerceptionBasisView::Initiative => B::Initiative,
                    },
                    distance_feet: context.distance_feet,
                    requires_movement: context.requires_movement,
                },
                other,
            );
            Ok::<_, AppServiceError>(EncounterSensoryRequirementsView {
                perception: result.perception,
                critical_failure: result.critical_failure,
                visual_effects_immune: result.visual_effects_immune,
                auditory_effects_immune: result.auditory_effects_immune,
                all_terrain_difficult: result.all_terrain_difficult,
                required_facts: result.required_facts,
            })
        })
        .transpose()?;
    let forced_movement = request
        .forced_movement
        .map(|context| {
            if context
                .difficulty_class
                .is_some_and(|dc| !(1..=10000).contains(&dc))
                || context
                    .check_total
                    .is_some_and(|total| !(-100000..=100000).contains(&total))
            {
                return Err(AppServiceError::invalid_request(
                    "Enter a valid movement check DC and total.",
                ));
            }
            use super::rules::movement_context::{
                ForcedMovementContext, ForcedMovementOutcome as O,
            };
            let result = super::rules::movement_context::forced_movement(
                participant,
                other,
                ForcedMovementContext {
                    holding_in_place: context.holding_in_place,
                    difficulty_class: context.difficulty_class,
                    check_total: context.check_total,
                },
            );
            Ok::<_, AppServiceError>(EncounterForcedMovementRequirementsView {
                outcome: match result.outcome {
                    O::Unrestricted => EncounterForcedMovementOutcomeView::Unrestricted,
                    O::NeedsCheck => EncounterForcedMovementOutcomeView::NeedsCheck,
                    O::CheckFailed => EncounterForcedMovementOutcomeView::CheckFailed,
                    O::CheckSucceeded => EncounterForcedMovementOutcomeView::CheckSucceeded,
                    O::NeedsContext => EncounterForcedMovementOutcomeView::NeedsContext,
                },
                difficulty_class: result.difficulty_class,
                source_ids: result.source_ids,
                required_facts: result.required_facts,
            })
        })
        .transpose()?;
    let prone_cover = request.prone_cover.map(|context| {
        let result = super::rules::movement_context::prone_cover(
            participant,
            baseline,
            other,
            context.against_ranged_attack,
        );
        EncounterProneCoverRequirementsView {
            armor_class: result.armor_class,
            required_facts: result.required_facts,
        }
    });
    let affiliation =
        super::rules::action_context::affiliation(participant, other, request.normally_ally);
    Ok(EncounterActionPreviewView {
        considers_target_ally: affiliation.0,
        can_flank: affiliation.1,
        encounter_key: snapshot.encounter.encounter_key,
        participant_key: participant.participant_key.clone(),
        expected_revision: snapshot.encounter.revision,
        expected_participant_revision: participant.revision,
        issues,
        confusion: super::action_attempts::confusion_view(
            participant,
            confusion.as_ref(),
            kind,
            facts.iter().any(|fact| {
                fact.source_id.is_none() && fact.fact == ActionFact::Attack && fact.value
            }),
        ),
        disruption,
        targeting,
        quickened_use,
        request: request_requirements,
        sensory,
        forced_movement,
        prone_cover,
    })
}

fn action_kind(kind: EncounterActionKindView) -> ActionKind {
    match kind {
        EncounterActionKindView::Strike => ActionKind::Strike,
        EncounterActionKindView::Cast => ActionKind::Cast,
        EncounterActionKindView::Escape => ActionKind::Escape,
        EncounterActionKindView::ForceOpen => ActionKind::ForceOpen,
        EncounterActionKindView::Stand => ActionKind::Stand,
        EncounterActionKindView::Crawl => ActionKind::Crawl,
        EncounterActionKindView::Seek => ActionKind::Seek,
        EncounterActionKindView::Delay => ActionKind::Delay,
        EncounterActionKindView::Ready => ActionKind::Ready,
        EncounterActionKindView::Other => ActionKind::Other,
        EncounterActionKindView::Retch => ActionKind::Retch,
        EncounterActionKindView::Request => ActionKind::Request,
        EncounterActionKindView::TravelingExploration => ActionKind::TravelingExploration,
    }
}
pub(super) fn builtin_facts(kind: ActionKind) -> Vec<ActionJudgment> {
    let mut facts = vec![];
    if matches!(kind, ActionKind::Stand | ActionKind::Crawl) {
        facts.push(ActionJudgment {
            source_id: None,
            fact: ActionFact::Move,
            value: true,
        });
    }
    if matches!(
        kind,
        ActionKind::Strike | ActionKind::Escape | ActionKind::ForceOpen
    ) {
        facts.push(ActionJudgment {
            source_id: None,
            fact: ActionFact::Attack,
            value: true,
        });
    }
    if kind == ActionKind::Request {
        for (fact, value) in [
            (ActionFact::Auditory, true),
            (ActionFact::Concentrate, true),
            (ActionFact::Manipulate, false),
            (ActionFact::Move, false),
            (ActionFact::Attack, false),
            (ActionFact::MentalOnly, false),
            (ActionFact::VoluntaryIngestion, false),
        ] {
            facts.push(ActionJudgment {
                source_id: None,
                fact,
                value,
            });
        }
    }
    if kind == ActionKind::Retch {
        for fact in [
            ActionFact::Manipulate,
            ActionFact::Auditory,
            ActionFact::Concentrate,
            ActionFact::Move,
            ActionFact::Attack,
            ActionFact::MentalOnly,
            ActionFact::VoluntaryIngestion,
        ] {
            facts.push(ActionJudgment {
                source_id: None,
                fact,
                value: false,
            });
        }
    }
    if kind == ActionKind::Seek {
        facts.push(ActionJudgment {
            source_id: None,
            fact: ActionFact::Concentrate,
            value: true,
        });
    }
    facts
}

pub(super) struct PreparedTargeting {
    pub requirements: EncounterTargetingRequirementsView,
    pub saved: atlas_local_state::FrozenActionTarget,
}
pub(super) fn targeting_for_action(
    participants: &[atlas_local_state::EncounterParticipant],
    relationships: &[atlas_local_state::DirectedRelationship],
    observer: &str,
    target: Option<&EncounterActionTargetView>,
    observer_baseline: Option<&super::stat_block::StatBlockView>,
    source_traits: Option<&[String]>,
) -> AppServiceResult<Option<PreparedTargeting>> {
    use atlas_local_state::{
        DetectionState as D, RelationshipKnowledge as K, RelationshipProperty as P,
        RelationshipValue as V,
    };
    target
        .map(|target| {
            let subject = participants
                .iter()
                .find(|participant| participant.participant_key == target.participant_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Target is no longer in this encounter.")
                })?;
            let detection=super::rules::relationships::relationship_knowledge(subject,observer,P::Detection,relationships);
            let mode = match target.mode {
                EncounterTargetModeView::Direct => super::rules::relationships::TargetMode::Direct,
                EncounterTargetModeView::Area => super::rules::relationships::TargetMode::Area,
                EncounterTargetModeView::GuessedLocation => {
                    super::rules::relationships::TargetMode::GuessedLocation
                }
            };
            let mut requirement =
                super::rules::relationships::targeting_requirements(subject, observer, mode);
            let target_causes = super::rules::causes::condition_causes(subject, Some(observer));
            let visual_effects_immune = target_causes.iter().any(|cause| cause.rule == super::conditions::ConditionRule::Blinded);
            let auditory_effects_immune = target_causes.iter().any(|cause| cause.rule == super::conditions::ConditionRule::Deafened);
            let checked_trait = |name: &str, declared: Option<bool>| -> AppServiceResult<Option<bool>> {
                if let Some(traits) = source_traits {
                    let known = traits.iter().any(|value| value == name);
                    if declared.is_some_and(|value| value != known) {
                        return Err(AppServiceError::invalid_request(format!("The source establishes whether this effect has the {name} trait.")));
                    }
                    Ok(Some(known))
                } else { Ok(declared) }
            };
            let visual_effect = checked_trait("visual", target.visual_effect)?;
            let auditory_effect = checked_trait("auditory", target.auditory_effect)?;
            let prevents_effect = (visual_effects_immune && visual_effect == Some(true)) || (auditory_effects_immune && auditory_effect == Some(true));
            let mut required_facts = vec![];
            let mut required_inputs=vec![];
            if visual_effects_immune && visual_effect.is_none() { required_inputs.push(EncounterTargetRequiredInputView::VisualEffect); required_facts.push("Confirm whether this effect has the visual trait.".into()); }
            if auditory_effects_immune && auditory_effect.is_none() { required_inputs.push(EncounterTargetRequiredInputView::AuditoryEffect); required_facts.push("Confirm whether this effect has the auditory trait.".into()); }
            let observer_participant = participants
                .iter()
                .find(|participant| participant.participant_key == observer)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Observer is no longer in this encounter.")
                })?;
            let distance = target
                .distance_feet
                .as_ref()
                .and_then(serde_json::Number::as_f64);
            if distance.is_some_and(|distance| distance < 0.0) {
                return Err(AppServiceError::invalid_request(
                    "Enter a nonnegative target distance.",
                ));
            }
            if mode != super::rules::relationships::TargetMode::Area {
                let dazzled = super::rules::sensory_context::dazzled_concealment(
                    observer_participant,
                    &subject.participant_key,
                    observer_baseline,
                    distance,
                    target.vision_only_precise,
                );
                match dazzled.applies {
                    Some(true) => {
                        requirement.flat_check_dc =
                            Some(requirement.flat_check_dc.unwrap_or(0).max(5));
                        requirement.source_ids.extend(dazzled.source_ids);
                    }
                    Some(false) => {}
                    None => {required_facts.push("Confirm whether vision is the only usable precise sense for this target.".into());required_inputs.push(EncounterTargetRequiredInputView::VisionOnlyPrecise);if distance.is_none()&&observer_baseline.is_some_and(|baseline|baseline.sense_facts.iter().any(|sense|!sense.visual&&sense.range_feet.is_some())) {required_inputs.push(EncounterTargetRequiredInputView::DistanceFeet);}},
                }
            }

            if mode != super::rules::relationships::TargetMode::Area {
                match &detection {
                    K::Known {
                        value: V::Detection(D::Observed),
                        ..
                    } => {}
                    K::Known {
                        value: V::Detection(D::Hidden),
                        ..
                    } => {
                        requirement.flat_check_dc =
                            Some(requirement.flat_check_dc.unwrap_or(0).max(11))
                    }
                    K::Known {
                        value: V::Detection(D::Undetected),
                        ..
                    } => {
                        requirement.flat_check_dc = Some(11);
                        requirement.requires_guessed_location =
                            mode != super::rules::relationships::TargetMode::GuessedLocation;
                        requirement.secret = true;
                    }
                    K::Unknown {
                        required_facts: facts,
                    } => required_facts.extend(facts.clone()),
                    K::Conflicting { .. } => {
                        required_facts.push("Choose how this observer detects the target.".into())
                    }
                    K::Invalidated { .. } => required_facts
                        .push("Update detection after the previous effect ended.".into()),
                    _ => required_facts.push("Choose how this observer detects the target.".into()),
                }
            }
            if mode!=super::rules::relationships::TargetMode::Area {
                if !matches!(detection,K::Known {value:V::Detection(_),..}) {required_inputs.push(EncounterTargetRequiredInputView::Detection);}
                if requirement.requires_guessed_location {required_inputs.push(EncounterTargetRequiredInputView::GuessedLocation);}
            }
            if requirement.secret && mode == super::rules::relationships::TargetMode::GuessedLocation && target.guessed_location_correct.is_none() {
                required_inputs.push(EncounterTargetRequiredInputView::GuessedLocationCorrect);
                required_facts.push("Confirm whether the guessed location contains the target.".into());
            }
            Ok(PreparedTargeting {
                requirements: EncounterTargetingRequirementsView {
                    visual_effects_immune,
                    auditory_effects_immune,
                    prevents_effect,
                    flat_check_dc: requirement.flat_check_dc,
                    source_ids: requirement.source_ids,
                    requires_guessed_location: requirement.requires_guessed_location,
                    secret: requirement.secret,
                    required_facts,
                    required_inputs,
                },
                saved: atlas_local_state::FrozenActionTarget {
                    participant_key: target.participant_key.clone(),
                    mode: match target.mode {
                        EncounterTargetModeView::Direct => {
                            atlas_local_state::ActionTargetMode::Direct
                        }
                        EncounterTargetModeView::Area => atlas_local_state::ActionTargetMode::Area,
                        EncounterTargetModeView::GuessedLocation => {
                            atlas_local_state::ActionTargetMode::GuessedLocation
                        }
                    },
                    detection,
                    vision_only_precise: target.vision_only_precise,
                    distance_feet: target.distance_feet.clone(),
                    guessed_location_correct: if requirement.secret { target.guessed_location_correct } else { None },
                    visual_effect,
                    auditory_effect,
                },
            })
        })
        .transpose()
}
pub(super) fn targeting_check(
    participants: &[atlas_local_state::EncounterParticipant],
    requirement: &EncounterTargetingRequirementsView,
    dc: i64,
) -> atlas_local_state::FrozenActionCheck {
    atlas_local_state::FrozenActionCheck {
        stage: atlas_local_state::ActionCheckStage::Targeting,
        dc,
        sources: requirement
            .source_ids
            .iter()
            .map(|id| atlas_local_state::ActionCheckSource {
                condition_id: *id,
                name: participants
                    .iter()
                    .flat_map(|participant| &participant.conditions)
                    .find(|condition| condition.condition_id == *id)
                    .map(|condition| condition.name.clone())
                    .unwrap_or_default(),
            })
            .collect(),
    }
}
