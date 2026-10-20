//! Casting commits costs once, then resumes frozen disruption checks independently.
use super::rules::action_context::{
    ActionContext, ActionFact, ActionIssue, ActionJudgment, ActionKind, evaluate_action,
};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{
    ActionAttemptReceipt, ActionAttemptStatus, ActionCheckResult, ActionCheckStage, FrozenAction,
    FrozenActionContext, PreparedActionAttempt, SavedActionAttempt, SpellResource,
};
use atlas_record::source_record::{ItemSourceView, SourceNodeView, SourceQueryView};

impl AtlasAppService {
    pub fn preview_encounter_casting_attempt(
        &self,
        encounter_ref: &str,
        request: PreviewEncounterCastingAttemptRequest,
    ) -> AppServiceResult<EncounterCastingPreviewView> {
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| Ok(prepare(workspace, &encounter_ref, &request)?.view),
        )
    }
    pub fn begin_encounter_casting_attempt(
        &self,
        encounter_ref: &str,
        request: BeginEncounterCastingAttemptRequest,
    ) -> AppServiceResult<EncounterActionMutationView> {
        let intent = serde_json::to_value(&request)
            .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
        let (encounter_key, recovered) =
            self.casting_receipt(encounter_ref, &request.request_id, &intent)?;
        if let Some(receipt) = recovered {
            return Ok(receipt_view(receipt));
        }
        let key = encounter_key.clone();
        let preview = request.preview.clone();
        let prepared = self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| prepare(workspace, &key, &preview),
        )?;
        if prepared.view.artifact_generation != request.artifact_generation {
            return Err(AppServiceError::artifact_generation_changed(
                &request.artifact_generation,
                &prepared.view.artifact_generation,
                AppRecoverableAction::ReloadRecord,
            ));
        }
        if prepared.view.targeting.as_ref().is_some_and(|target| {
            target.prevents_effect
                || target.requires_guessed_location
                || !target.required_facts.is_empty()
        }) {
            return Err(AppServiceError::invalid_request(
                "Confirm target detection and choose a guessed location when required.",
            ));
        }
        if !prepared.view.issues.is_empty() {
            return Err(AppServiceError::invalid_request(
                "Resolve the casting requirements before beginning.",
            ));
        }
        if prepared.view.pending_attempt_key.is_some() {
            return Err(AppServiceError::invalid_request(
                "Finish or abandon the pending casting check.",
            ));
        }
        self.with_write(move |workspace| {
            let encounters = workspace.local_state().encounters();
            // A committed retry remains recoverable even if source reference changed.
            if let Some(receipt) =
                encounters.action_attempt_receipt(&encounter_key, &request.request_id, &intent)?
            {
                return Ok(receipt_view(receipt));
            }
            workspace.require_reference_revision(prepared.prepared.context.reference_revision)?;
            let receipt = encounters.begin_action_attempt(
                &encounter_key,
                &request.request_id,
                &intent,
                atlas_local_state::ParticipantRollGuard {
                    expected_revision: request.expected_encounter_revision,
                    participant_key: &request.preview.participant_key,
                    expected_participant_revision: request.expected_participant_revision,
                },
                &prepared.prepared,
                complete_action,
            )?;
            Ok(receipt_view(receipt))
        })
    }
    pub fn encounter_casting_attempts(
        &self,
        encounter_ref: &str,
    ) -> AppServiceResult<EncounterCastingAttemptsView> {
        self.with_state(|workspace| {
            let encounters = workspace.local_state().encounters();
            let encounter = encounters
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let attempts = encounters
                .action_attempts(&encounter.encounter_key)?
                .into_iter()
                .filter_map(attempt_view)
                .collect();
            Ok(EncounterCastingAttemptsView {
                encounter_key: encounter.encounter_key,
                attempts,
            })
        })
    }
    pub fn resolve_encounter_action_check(
        &self,
        encounter_ref: &str,
        request: ResolveEncounterActionCheckRequest,
    ) -> AppServiceResult<EncounterActionMutationView> {
        let intent = serde_json::to_value(&request)
            .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
        let (key, recovered) = self.casting_receipt(encounter_ref, &request.request_id, &intent)?;
        if let Some(receipt) = recovered {
            return Ok(receipt_view(receipt));
        }
        if let EncounterActionCheckInputView::TableResult { result } = &request.outcome
            && !(1..=20).contains(result)
        {
            return Err(AppServiceError::invalid_request(
                "Enter a d20 result from 1 through 20.",
            ));
        }
        let die = atlas_dice::prepare("1d20").map_err(atlas_local_state::LocalStateError::from)?;
        self.with_write(move |workspace| {
            let receipt = workspace.local_state().encounters().resolve_action_check(
                &key,
                &request.request_id,
                &intent,
                &request.attempt_key,
                request.expected_attempt_revision,
                move |dc| match request.outcome {
                    EncounterActionCheckInputView::Roll => {
                        let roll = die.roll()?;
                        let success = i64::from(roll.total) >= dc;
                        Ok(ActionCheckResult {
                            roll: Some(roll),
                            table_result: None,
                            success,
                        })
                    }
                    EncounterActionCheckInputView::TableResult { result } => {
                        Ok(ActionCheckResult {
                            roll: None,
                            table_result: Some(result),
                            success: result >= dc,
                        })
                    }
                },
                complete_action,
            )?;
            Ok(receipt_view(receipt))
        })
    }
    pub fn abandon_encounter_action_attempt(
        &self,
        encounter_ref: &str,
        request: AbandonEncounterActionAttemptRequest,
    ) -> AppServiceResult<EncounterActionMutationView> {
        let intent = serde_json::to_value(&request)
            .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
        let (key, recovered) = self.casting_receipt(encounter_ref, &request.request_id, &intent)?;
        if let Some(receipt) = recovered {
            return Ok(receipt_view(receipt));
        }
        self.with_write(move |workspace| {
            Ok(receipt_view(
                workspace
                    .local_state()
                    .encounters()
                    .abandon_action_attempt(
                        &key,
                        &request.request_id,
                        &intent,
                        &request.attempt_key,
                        request.expected_attempt_revision,
                    )?,
            ))
        })
    }
    pub(super) fn casting_receipt(
        &self,
        encounter_ref: &str,
        id: &str,
        intent: &serde_json::Value,
    ) -> AppServiceResult<(String, Option<ActionAttemptReceipt>)> {
        self.with_state(|workspace| {
            let encounters = workspace.local_state().encounters();
            let encounter = encounters
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let receipt =
                encounters.action_attempt_receipt(&encounter.encounter_key, id, intent)?;
            Ok((encounter.encounter_key, receipt))
        })
    }
}
struct PreparedPreview {
    view: EncounterCastingPreviewView,
    prepared: PreparedActionAttempt,
}
fn prepare(
    workspace: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    request: &PreviewEncounterCastingAttemptRequest,
) -> AppServiceResult<PreparedPreview> {
    if request.judgments.len() > 200 {
        return Err(AppServiceError::invalid_request(
            "Too many casting judgments.",
        ));
    }
    let snapshot = workspace
        .local_state()
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
    let participant = snapshot
        .participants
        .iter()
        .find(|participant| participant.participant_key == request.participant_key)
        .ok_or_else(|| {
            AppServiceError::invalid_request("Participant is no longer in this encounter.")
        })?;
    let confusion = checked_confusion(
        &snapshot.participants,
        participant,
        request.confusion.as_ref(),
    )?;
    let target =
        effective_confusion_target(participant, confusion.as_ref(), request.target.as_ref())?;
    let automatic_self_hit = matches!(
        confusion,
        Some(atlas_local_state::ConfusionChoice::NoViableTargets)
    );
    let relationships = workspace
        .local_state()
        .encounters()
        .relationships(&snapshot.encounter.encounter_key)?;
    let selection = &request.selection;
    if participant.record_key.as_deref() != Some(selection.record_key.as_str())
        || selection.owners.len() != 1
        || selection.field.is_some()
        || selection.passage.is_some()
    {
        return Err(AppServiceError::invalid_request(
            "Select this participant's spell.",
        ));
    }
    let source =
        super::hydration::resolve_record_ref(workspace.retrieval_mut(), &selection.record_key)?;
    let fingerprint = workspace.retrieval_mut().source_fingerprint().to_owned();
    if selection.source_fingerprint.as_deref() != Some(fingerprint.as_str()) {
        return Err(AppServiceError::invalid_request(
            "Spell source changed; reopen the spell.",
        ));
    }
    let Some(SourceNodeView::Item(item @ ItemSourceView::SpellSource(_))) =
        source.source.node_at(&selection.owners)
    else {
        return Err(AppServiceError::invalid_request("Select a spell."));
    };
    let spell_id = match &selection.owners[0].identity {
        atlas_record::source_content::OwnedContentIdentity::Stable(id) => id.clone(),
        _ => {
            return Err(AppServiceError::invalid_request(
                "Spell identity is unavailable.",
            ));
        }
    };
    let traits = SourceQueryView {
        source: SourceNodeView::Item(item),
        pack_id: "",
        pack_label: "",
    }
    .traits()
    .value();
    let source_traits = traits;
    let mut facts = vec![ActionJudgment {
        source_id: None,
        fact: ActionFact::VoluntaryIngestion,
        value: false,
    }];
    if let Some(traits) = traits {
        facts.push(ActionJudgment {
            source_id: None,
            fact: ActionFact::Cantrip,
            value: traits.iter().any(|value| value == "cantrip"),
        });
        for (fact, trait_key) in [
            (ActionFact::Manipulate, "manipulate"),
            (ActionFact::Auditory, "auditory"),
            (ActionFact::Concentrate, "concentrate"),
            (ActionFact::Move, "move"),
            (ActionFact::Attack, "attack"),
        ] {
            facts.push(ActionJudgment {
                source_id: None,
                fact,
                value: traits.iter().any(|value| value == trait_key),
            });
        }
    }
    let reaction = item
        .spell_casting_form()
        .value()
        .and_then(|form| match form {
            "reaction" => Some(true),
            "free" | "one_action" | "two_actions" | "three_actions" | "one_or_two"
            | "one_to_three" | "two_or_three" | "two_rounds" => Some(false),
            _ => None,
        });
    if let Some(value) = reaction {
        facts.push(ActionJudgment {
            source_id: None,
            fact: ActionFact::Reaction,
            value,
        });
    }
    let judged = apply_judgments(
        participant,
        ActionContext {
            kind: ActionKind::Cast,
            reaction,
            other_participant: target
                .as_ref()
                .map(|target| target.participant_key.as_str()),
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
        .map(|issue| issue_view(issue, participant, &relationships))
        .collect::<Vec<_>>();
    if participant.defeated
        || participant.hazard_disabled
        || participant.mortality == atlas_local_state::MortalityState::Dead
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
    let catalog = super::spell_catalog::catalog(
        &source.source,
        &participant.spell_resources,
        &participant.expended_spells,
        Some(&fingerprint),
    );
    let spell = catalog
        .spells
        .iter()
        .find(|spell| spell.navigation.owners == selection.owners)
        .ok_or_else(|| AppServiceError::invalid_request("Spell is no longer available."))?;
    let rank = selection
        .spell_rank
        .ok_or_else(|| AppServiceError::invalid_request("Choose a casting rank."))?;
    let casting = spell
        .ranks
        .iter()
        .find(|cast| cast.rank == rank)
        .ok_or_else(|| AppServiceError::invalid_request("This casting rank is unavailable."))?;
    if let Some(reason) = &casting.unavailable_reason {
        issues.push(casting_unavailable_issue(reason));
    }
    let mut resources = participant.spell_resources.clone();
    for resource in &catalog.resources {
        if !resources.iter().any(|saved| saved.key == resource.key) {
            resources.push(SpellResource {
                key: resource.key.clone(),
                capacity: resource.capacity,
                remaining: resource.remaining,
                undo_remaining: None,
            });
        }
    }
    let cost = if let Some(key) = &casting.resource_key {
        if let Some(index) = resources.iter().position(|resource| resource.key == *key) {
            let before = resources[index].remaining;
            if before < 1 {
                if casting.unavailable_reason.is_none() {
                    issues.push(casting_unavailable_issue("No uses remaining."));
                }
                None
            } else if casting.unavailable_reason.is_some() {
                None
            } else {
                resources[index].remaining -= 1;
                let after = resources[index].remaining;
                for resource in &mut resources {
                    resource.undo_remaining = None;
                }
                let label = catalog
                    .resources
                    .iter()
                    .find(|resource| resource.key == *key)
                    .map(|resource| resource.label.clone())
                    .unwrap_or_default();
                Some(atlas_local_state::SpellResourceCost {
                    resource_key: key.clone(),
                    label,
                    before,
                    after,
                })
            }
        } else {
            issues.push(casting_unavailable_issue("Spell resource is unavailable."));
            None
        }
    } else {
        None
    };
    let disruption = evaluated
        .disruption
        .map(|check| atlas_local_state::FrozenActionCheck {
            stage: ActionCheckStage::Disruption,
            dc: check.dc,
            sources: check
                .source_ids
                .into_iter()
                .map(|id| atlas_local_state::ActionCheckSource {
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
    let generation = workspace.retrieval_mut().artifact_generation()?;
    let pending = workspace
        .local_state()
        .encounters()
        .action_attempts(&snapshot.encounter.encounter_key)?
        .into_iter()
        .find(|attempt| {
            attempt.context.participant_key == participant.participant_key
                && attempt.status == ActionAttemptStatus::AwaitingCheck
        })
        .map(|attempt| attempt.attempt_key);
    let sensory_baseline = super::rules::source_baseline(&source.source, &fingerprint, None);
    let prepared_targeting = if automatic_self_hit {
        None
    } else {
        super::action_preview::targeting_for_action(
            &snapshot.participants,
            &relationships,
            &participant.participant_key,
            target.as_ref(),
            sensory_baseline.as_ref(),
            source_traits,
        )?
    };
    let saved_target = prepared_targeting
        .as_ref()
        .map(|target| target.saved.clone());
    let targeting = prepared_targeting.map(|target| target.requirements);
    let mut context = FrozenActionContext {
        participant_key: participant.participant_key.clone(),
        source_record_key: selection.record_key.clone(),
        title: spell.title.clone(),
        action: FrozenAction::Cast { spell_id, rank },
        target: saved_target,
        confusion: confusion.clone(),
        source_fingerprint: fingerprint,
        artifact_generation: generation.clone(),
        reference_revision: workspace.reference_revision()?,
        judgments: facts
            .iter()
            .map(|fact| atlas_local_state::FrozenActionJudgment {
                source_condition_id: fact.source_id,
                fact: fact_to_saved(fact.fact),
                value: fact.value,
            })
            .collect(),
        cost,
        checks: disruption.into_iter().collect(),
    };
    if let Some(targeting) = &targeting
        && let Some(dc) = targeting.flat_check_dc
    {
        context.checks.push(super::action_preview::targeting_check(
            &snapshot.participants,
            targeting,
            dc,
        ));
    }
    let advisories = casting_advisories(
        participant,
        &snapshot.participants,
        &issues,
        context
            .checks
            .iter()
            .find(|check| check.stage == ActionCheckStage::Disruption),
    );
    let view = EncounterCastingPreviewView {
        advisories,
        encounter_key: snapshot.encounter.encounter_key,
        participant_key: participant.participant_key.clone(),
        expected_encounter_revision: snapshot.encounter.revision,
        expected_participant_revision: participant.revision,
        artifact_generation: generation,
        spell_title: context.title.clone(),
        rank,
        cost: context.cost.as_ref().map(cost_view),
        issues,
        confusion: confusion_view(
            participant,
            confusion.as_ref(),
            ActionKind::Cast,
            facts.iter().any(|fact| {
                fact.source_id.is_none() && fact.fact == ActionFact::Attack && fact.value
            }),
        ),
        disruption: context.checks.first().map(disruption_view),
        pending_attempt_key: pending,
        targeting,
    };
    let has_cost = context.cost.is_some();
    Ok(PreparedPreview {
        view,
        prepared: PreparedActionAttempt {
            context,
            resources: has_cost.then_some(resources),
        },
    })
}
pub(super) fn is_intrinsic(fact: ActionFact) -> bool {
    matches!(
        fact,
        ActionFact::Reaction
            | ActionFact::Manipulate
            | ActionFact::Auditory
            | ActionFact::Concentrate
            | ActionFact::Move
            | ActionFact::Attack
            | ActionFact::MentalOnly
            | ActionFact::VoluntaryIngestion
            | ActionFact::Cantrip
    )
}
pub(super) fn issue_view(
    issue: ActionIssue,
    participant: &atlas_local_state::EncounterParticipant,
    relationships: &[atlas_local_state::DirectedRelationship],
) -> EncounterActionIssueView {
    let (source, kind, fact, reason) = match issue {
        ActionIssue::Prohibited { source_id, rule } => (
            source_id,
            EncounterActionIssueKindView::Prohibited,
            None,
            prohibition_reason(rule).into(),
        ),
        ActionIssue::NeedsConfusion { source_id } => (
            source_id,
            EncounterActionIssueKindView::NeedsConfusion,
            None,
            "Choose how Confused directs this action.".into(),
        ),
        ActionIssue::NeedsContext { source_id, reason } => (
            source_id,
            EncounterActionIssueKindView::NeedsContext,
            None,
            reason.into(),
        ),
        ActionIssue::NeedsValue { source_id } => (
            source_id,
            EncounterActionIssueKindView::NeedsValue,
            None,
            "Enter this condition's value.".into(),
        ),
        ActionIssue::NeedsFact { source_id, fact } => (
            source_id,
            EncounterActionIssueKindView::NeedsFact,
            Some(fact_to_view(fact)),
            fact_question(fact).into(),
        ),
    };
    let control_source = participant.conditions.iter().find(|condition| {
        condition.condition_id == source
            && super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                == Some(super::conditions::ConditionRule::Controlled)
    });
    let authority = control_source.map(|source| {
        super::rules::action_context::control_direction(participant, source, relationships)
    });
    let (controller_key, instructions) = match authority {
        Some(atlas_local_state::RelationshipKnowledge::Known {
            value:
                atlas_local_state::RelationshipValue::Control {
                    controller_key,
                    instructions,
                },
            ..
        }) => (Some(controller_key), instructions),
        _ => (None, None),
    };
    let subject = participant
        .conditions
        .iter()
        .find(|condition| condition.condition_id == source)
        .and_then(|condition| match &condition.parameters {
            atlas_local_state::ConditionParameters::Fleeing { subject }
            | atlas_local_state::ConditionParameters::Fascinated { subject } => {
                subject.as_ref().map(|subject| match subject {
                    atlas_local_state::ConditionSubject::Participant { participant_key } => {
                        ConditionSubjectView::Participant {
                            participant_key: participant_key.clone(),
                        }
                    }
                    atlas_local_state::ConditionSubject::Other { description } => {
                        ConditionSubjectView::Other {
                            description: description.clone(),
                        }
                    }
                })
            }
            _ => None,
        });
    EncounterActionIssueView {
        subject,
        controller_key,
        instructions,
        source_condition_id: source,
        condition_name: participant
            .conditions
            .iter()
            .find(|condition| condition.condition_id == source)
            .map(|condition| condition.name.clone())
            .unwrap_or_default(),
        kind,
        fact,
        reason,
    }
}
fn fact_question(fact: ActionFact) -> &'static str {
    match fact {
        ActionFact::FollowsControllerInstructions => {
            "Does this action follow the controller’s instructions?"
        }
        ActionFact::Reaction => "Is this a reaction?",
        ActionFact::Manipulate => "Does this action have the manipulate trait?",
        ActionFact::Auditory => "Does this action have the auditory trait?",
        ActionFact::Concentrate => "Does this action have the concentrate trait?",
        ActionFact::Move => "Does this action have the move trait?",
        ActionFact::Attack => "Does this action have the attack trait?",
        ActionFact::MentalOnly => "Does this action require only mental effort?",
        ActionFact::OpensOwnBonds => "Does this action open these bonds?",
        ActionFact::RelatesToFascination => {
            "Does this casting relate to this fascination's subject?"
        }
        ActionFact::FurthersEscape => "Does this action help escape the named threat?",
        ActionFact::Cantrip => "Is this spell a cantrip?",
        ActionFact::VoluntaryIngestion => "Does this action voluntarily consume food or drink?",
    }
}
fn receipt_view(receipt: ActionAttemptReceipt) -> EncounterActionMutationView {
    EncounterActionMutationView {
        encounter_key: receipt.encounter_key,
        attempt_key: receipt.attempt_key,
    }
}
fn attempt_view(attempt: SavedActionAttempt) -> Option<EncounterCastingAttemptView> {
    let FrozenAction::Cast { rank, .. } = attempt.context.action else {
        return None;
    };
    let (secret, shareable_summary) = disclosure(&attempt);
    Some(EncounterCastingAttemptView {
        attempt_key: attempt.attempt_key,
        participant_key: attempt.context.participant_key,
        revision: attempt.revision,
        status: match attempt.status {
            ActionAttemptStatus::AwaitingCheck => EncounterActionAttemptStatusView::AwaitingCheck,
            ActionAttemptStatus::Ready => EncounterActionAttemptStatusView::Ready,
            ActionAttemptStatus::Disrupted => EncounterActionAttemptStatusView::Disrupted,
            ActionAttemptStatus::Abandoned => EncounterActionAttemptStatusView::Abandoned,
        },
        automatic_self_hit: matches!(
            attempt.context.confusion,
            Some(atlas_local_state::ConfusionChoice::NoViableTargets)
        ),
        secret,
        shareable_summary,
        spell_title: attempt.context.title,
        rank,
        cost: attempt.context.cost.as_ref().map(cost_view),
        checks: attempt.context.checks.iter().map(disruption_view).collect(),
        pending_check: if attempt.status == ActionAttemptStatus::AwaitingCheck {
            attempt
                .context
                .checks
                .get(attempt.outcomes.len())
                .map(disruption_view)
        } else {
            None
        },
        outcomes: attempt
            .outcomes
            .into_iter()
            .map(|outcome| EncounterActionCheckResultView {
                roll: outcome.roll,
                table_result: outcome.table_result,
                success: outcome.success,
            })
            .collect(),
    })
}
fn cost_view(cost: &atlas_local_state::SpellResourceCost) -> CastingResourceCostView {
    CastingResourceCostView {
        resource_key: cost.resource_key.clone(),
        label: cost.label.clone(),
        before: cost.before,
        after: cost.after,
    }
}
fn disruption_view(check: &atlas_local_state::FrozenActionCheck) -> EncounterActionCheckView {
    EncounterActionCheckView {
        dc: check.dc,
        sources: check
            .sources
            .iter()
            .map(|source| EncounterActionCheckSourceView {
                condition_id: source.condition_id,
                name: source.name.clone(),
            })
            .collect(),
    }
}

pub(super) fn fact_from_view(fact: EncounterActionFactView) -> ActionFact {
    match fact {
        EncounterActionFactView::FollowsControllerInstructions => {
            ActionFact::FollowsControllerInstructions
        }
        EncounterActionFactView::Reaction => ActionFact::Reaction,
        EncounterActionFactView::Manipulate => ActionFact::Manipulate,
        EncounterActionFactView::Auditory => ActionFact::Auditory,
        EncounterActionFactView::Concentrate => ActionFact::Concentrate,
        EncounterActionFactView::Move => ActionFact::Move,
        EncounterActionFactView::Attack => ActionFact::Attack,
        EncounterActionFactView::MentalOnly => ActionFact::MentalOnly,
        EncounterActionFactView::OpensOwnBonds => ActionFact::OpensOwnBonds,
        EncounterActionFactView::RelatesToFascination => ActionFact::RelatesToFascination,
        EncounterActionFactView::FurthersEscape => ActionFact::FurthersEscape,
        EncounterActionFactView::Cantrip => ActionFact::Cantrip,
        EncounterActionFactView::VoluntaryIngestion => ActionFact::VoluntaryIngestion,
    }
}

pub(super) fn fact_to_view(fact: ActionFact) -> EncounterActionFactView {
    match fact {
        ActionFact::FollowsControllerInstructions => {
            EncounterActionFactView::FollowsControllerInstructions
        }
        ActionFact::Reaction => EncounterActionFactView::Reaction,
        ActionFact::Manipulate => EncounterActionFactView::Manipulate,
        ActionFact::Auditory => EncounterActionFactView::Auditory,
        ActionFact::Concentrate => EncounterActionFactView::Concentrate,
        ActionFact::Move => EncounterActionFactView::Move,
        ActionFact::Attack => EncounterActionFactView::Attack,
        ActionFact::MentalOnly => EncounterActionFactView::MentalOnly,
        ActionFact::OpensOwnBonds => EncounterActionFactView::OpensOwnBonds,
        ActionFact::RelatesToFascination => EncounterActionFactView::RelatesToFascination,
        ActionFact::FurthersEscape => EncounterActionFactView::FurthersEscape,
        ActionFact::Cantrip => EncounterActionFactView::Cantrip,
        ActionFact::VoluntaryIngestion => EncounterActionFactView::VoluntaryIngestion,
    }
}

pub(super) fn fact_to_saved(fact: ActionFact) -> atlas_local_state::ActionFactKind {
    match fact {
        ActionFact::FollowsControllerInstructions => {
            atlas_local_state::ActionFactKind::FollowsControllerInstructions
        }
        ActionFact::Reaction => atlas_local_state::ActionFactKind::Reaction,
        ActionFact::Manipulate => atlas_local_state::ActionFactKind::Manipulate,
        ActionFact::Auditory => atlas_local_state::ActionFactKind::Auditory,
        ActionFact::Concentrate => atlas_local_state::ActionFactKind::Concentrate,
        ActionFact::Move => atlas_local_state::ActionFactKind::Move,
        ActionFact::Attack => atlas_local_state::ActionFactKind::Attack,
        ActionFact::MentalOnly => atlas_local_state::ActionFactKind::MentalOnly,
        ActionFact::OpensOwnBonds => atlas_local_state::ActionFactKind::OpensOwnBonds,
        ActionFact::RelatesToFascination => atlas_local_state::ActionFactKind::RelatesToFascination,
        ActionFact::FurthersEscape => atlas_local_state::ActionFactKind::FurthersEscape,
        ActionFact::Cantrip => atlas_local_state::ActionFactKind::Cantrip,
        ActionFact::VoluntaryIngestion => atlas_local_state::ActionFactKind::VoluntaryIngestion,
    }
}

fn prohibition_reason(rule: super::conditions::ConditionRule) -> &'static str {
    use super::conditions::ConditionRule;
    match rule {
        ConditionRule::Controlled => "The controller’s instructions prohibit this action.",
        ConditionRule::Unconscious => "Cannot act while unconscious.",
        ConditionRule::Petrified => "Cannot act while petrified.",
        ConditionRule::Stunned => "Cannot act while stunned.",
        ConditionRule::Paralyzed => "Only mental actions are allowed while paralyzed.",
        ConditionRule::Immobilized => "Cannot use actions with the move trait.",
        ConditionRule::Prone => "Move by Crawling or Standing.",
        ConditionRule::Restrained => {
            "Cannot use attack or manipulate actions except Escape or Force Open your bonds."
        }
        ConditionRule::Fascinated => {
            "Concentrate actions must relate to the subject of fascination."
        }
        ConditionRule::Fleeing => "Actions must help escape this source.",
        ConditionRule::Confused => {
            "Follow this source's confusion instruction; reactions are unavailable."
        }
        ConditionRule::Sickened => "Cannot voluntarily consume food or drink.",
        ConditionRule::Fatigued => {
            "Cannot perform exploration activities while traveling and Fatigued."
        }
        _ => "This action is prohibited by its condition.",
    }
}

/// Called only by the atomic owner after recovery and revision guards.
pub(super) fn complete_action(
    action: &atlas_local_state::FrozenAction,
) -> atlas_local_state::LocalStateResult<atlas_local_state::ActionCompletion> {
    use atlas_local_state::{ActionCompletion, ActionRollMode, FrozenAction};
    match action {
        FrozenAction::Cast { .. } => Ok(ActionCompletion::Cast),
        FrozenAction::Strike {
            retained_roll: Some(retained),
            ..
        } => Ok(ActionCompletion::StrikeRoll {
            roll: retained.outcome.clone(),
        }),
        FrozenAction::Strike {
            automatic_self_hit: true,
            ..
        } => Ok(ActionCompletion::AutomaticSelfHit),
        FrozenAction::Strike { modifier, mode, .. } => {
            let modifier =
                modifier.ok_or(atlas_local_state::LocalStateError::InvalidRollRequest(
                    "attack modifier unavailable",
                ))?;
            let die = match mode {
                ActionRollMode::Normal => "1d20",
                ActionRollMode::Fortune => "2d20kh1",
                ActionRollMode::Misfortune => "2d20kl1",
            };
            Ok(ActionCompletion::StrikeRoll {
                roll: atlas_dice::prepare(&format!("{die}{modifier:+}"))?.roll()?,
            })
        }
    }
}

pub(super) struct JudgedAction {
    pub evaluation: super::rules::action_context::ActionEvaluation,
    pub facts: Vec<ActionJudgment>,
}
pub(super) fn apply_judgments(
    participant: &atlas_local_state::EncounterParticipant,
    context: ActionContext<'_>,
    judgments: &[EncounterActionJudgmentView],
) -> AppServiceResult<JudgedAction> {
    let ActionContext {
        kind,
        mut reaction,
        other_participant: other,
        confusion,
        relationships,
        facts: known,
    } = context;
    let mut facts = known.to_vec();
    if judgments.len() > 200 {
        return Err(AppServiceError::invalid_request(
            "Too many action judgments.",
        ));
    }
    let initial = evaluate_action(
        participant,
        &ActionContext {
            kind,
            reaction,
            other_participant: other,
            confusion,
            relationships,
            facts: &facts,
        },
    );
    let mut remaining = judgments.iter().collect::<Vec<_>>();
    while !remaining.is_empty() {
        let before = remaining.len();
        let mut next = vec![];
        for judgment in remaining {
            let fact = fact_from_view(judgment.fact);
            let source_id = judgment.source_condition_id.ok_or_else(|| {
                AppServiceError::invalid_request("Choose the condition source for this judgment.")
            })?;
            if facts.iter().any(|known| {
                known.fact == fact
                    && (known.source_id.is_none()
                        || known.source_id == Some(source_id)
                        || is_intrinsic(fact))
                    && known.value != judgment.value
            }) {
                return Err(AppServiceError::invalid_request(
                    "An action judgment contradicts established facts.",
                ));
            }
            let current = evaluate_action(
                participant,
                &ActionContext {
                    kind,
                    reaction,
                    other_participant: other,
                    confusion,
                    relationships,
                    facts: &facts,
                },
            );
            if !initial.issues.iter().chain(&current.issues).any(|issue|matches!(issue,ActionIssue::NeedsFact {source_id:id,fact:needed} if *id==source_id&&*needed==fact)) {next.push(judgment);continue;}
            facts.push(ActionJudgment {
                source_id: Some(source_id),
                fact,
                value: judgment.value,
            });
            if fact == ActionFact::Reaction {
                reaction = Some(judgment.value);
            }
        }
        if next.len() == before {
            return Err(AppServiceError::invalid_request(
                "That fact is not required for this action.",
            ));
        }
        remaining = next;
    }
    let evaluation = evaluate_action(
        participant,
        &ActionContext {
            kind,
            reaction,
            other_participant: other,
            confusion,
            relationships,
            facts: &facts,
        },
    );
    Ok(JudgedAction { evaluation, facts })
}
pub(super) fn confusion_from_view(
    value: &EncounterConfusionContextView,
) -> AppServiceResult<atlas_local_state::ConfusionChoice> {
    use atlas_local_state::ConfusionChoice as C;
    Ok(match value {
        EncounterConfusionContextView::Offensive {
            viable_target_keys,
            random_target_key,
            offensive,
        } => {
            if viable_target_keys.len() > 1000
                || viable_target_keys.iter().any(|key| key.len() > 256)
                || random_target_key
                    .as_ref()
                    .is_some_and(|key| key.len() > 256)
            {
                return Err(AppServiceError::invalid_request(
                    "Too many or invalid viable targets.",
                ));
            }
            C::Offensive {
                viable_target_keys: viable_target_keys.clone(),
                random_target_key: random_target_key.clone(),
                offensive: *offensive,
            }
        }
        EncounterConfusionContextView::Facilitate { note } => {
            let note = note
                .as_ref()
                .map(|note| note.trim().to_owned())
                .filter(|note| !note.is_empty());
            if note.as_ref().is_some_and(|note| note.len() > 4096) {
                return Err(AppServiceError::invalid_request(
                    "The facilitation note is too long.",
                ));
            }
            C::Facilitate { note }
        }
        EncounterConfusionContextView::NoViableTargets => C::NoViableTargets,
        EncounterConfusionContextView::UnableToAttack => C::UnableToAttack,
    })
}
pub(super) fn validate_confusion_targets(
    participants: &[atlas_local_state::EncounterParticipant],
    actor: &str,
    choice: Option<&atlas_local_state::ConfusionChoice>,
) -> AppServiceResult<()> {
    if let Some(atlas_local_state::ConfusionChoice::Offensive {
        viable_target_keys, ..
    }) = choice
    {
        let mut unique = std::collections::BTreeSet::new();
        if viable_target_keys.iter().any(|key| {
            key == actor
                || !participants
                    .iter()
                    .any(|participant| participant.participant_key == *key)
                || !unique.insert(key)
        }) {
            return Err(AppServiceError::invalid_request(
                "Confirm distinct viable targets in this encounter; use the self-target procedure when no others are viable.",
            ));
        }
    }
    Ok(())
}
pub(super) fn effective_confusion_target(
    participant: &atlas_local_state::EncounterParticipant,
    choice: Option<&atlas_local_state::ConfusionChoice>,
    target: Option<&EncounterActionTargetView>,
) -> AppServiceResult<Option<EncounterActionTargetView>> {
    let chosen = match choice {
        Some(atlas_local_state::ConfusionChoice::Offensive {
            random_target_key: Some(key),
            ..
        }) => Some(key.as_str()),
        Some(atlas_local_state::ConfusionChoice::NoViableTargets) => {
            Some(participant.participant_key.as_str())
        }
        _ => None,
    };
    if let Some(chosen) = chosen {
        if target.is_some_and(|target| target.participant_key != chosen) {
            return Err(AppServiceError::invalid_request(
                "Use the target selected by the Confused procedure.",
            ));
        }
        return Ok(Some(target.cloned().unwrap_or_else(|| {
            EncounterActionTargetView {
                participant_key: chosen.into(),
                mode: EncounterTargetModeView::Direct,
                vision_only_precise: None,
                distance_feet: None,
                guessed_location_correct: None,
                visual_effect: None,
                auditory_effect: None,
            }
        })));
    }
    Ok(target.cloned())
}
pub(super) fn confusion_view(
    participant: &atlas_local_state::EncounterParticipant,
    choice: Option<&atlas_local_state::ConfusionChoice>,
    kind: ActionKind,
    source_attack: bool,
) -> Option<EncounterConfusionRequirementsView> {
    super::rules::action_context::confusion_requirements(participant, choice, kind, source_attack)
        .map(|value| EncounterConfusionRequirementsView {
            source_ids: value.source_ids,
            required_facts: value.required_facts,
            selected_target_key: value.selected_target_key,
            automatic_self_hit: value.automatic_self_hit,
            wastes_action: value.wastes_action,
        })
}

pub(super) fn checked_confusion(
    participants: &[atlas_local_state::EncounterParticipant],
    participant: &atlas_local_state::EncounterParticipant,
    value: Option<&EncounterConfusionContextView>,
) -> AppServiceResult<Option<atlas_local_state::ConfusionChoice>> {
    let choice = value.map(confusion_from_view).transpose()?;
    if choice.is_some()
        && !super::rules::causes::condition_causes(participant, None)
            .iter()
            .any(|cause| cause.rule == super::conditions::ConditionRule::Confused)
    {
        return Err(AppServiceError::invalid_request(
            "The Confused procedure is not required for this participant.",
        ));
    }
    validate_confusion_targets(participants, &participant.participant_key, choice.as_ref())?;
    Ok(choice)
}

/// GM detail remains visible; this summary intentionally omits secret causes and dice.
pub(super) fn disclosure(attempt: &SavedActionAttempt) -> (bool, String) {
    use atlas_local_state::{DetectionState, RelationshipKnowledge, RelationshipValue};
    let secret = attempt.context.target.as_ref().is_some_and(|target| {
        matches!(
            &target.detection,
            RelationshipKnowledge::Known {
                value: RelationshipValue::Detection(DetectionState::Undetected),
                ..
            }
        ) && target.mode != atlas_local_state::ActionTargetMode::Area
    });
    let summary = match attempt.status {
        ActionAttemptStatus::AwaitingCheck => "Awaiting resolution.",
        ActionAttemptStatus::Disrupted => "No effect.",
        ActionAttemptStatus::Abandoned => "Action abandoned.",
        ActionAttemptStatus::Ready => "Resolve the action at the table.",
    };
    (secret, summary.into())
}

/// Ordinary table use receives consequences, not an adjudication questionnaire.
fn casting_advisories(
    participant: &atlas_local_state::EncounterParticipant,
    roster: &[atlas_local_state::EncounterParticipant],
    issues: &[EncounterActionIssueView],
    check: Option<&atlas_local_state::FrozenActionCheck>,
) -> Vec<RuntimeEffectNoteView> {
    use super::conditions::ConditionRule as R;
    let mut notes = vec![];
    let unknown_stupefied = issues.iter().any(|issue| {
        issue.kind == EncounterActionIssueKindView::NeedsValue
            && participant
                .conditions
                .iter()
                .find(|condition| condition.condition_id == issue.source_condition_id)
                .is_some_and(|condition| {
                    super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                        == Some(R::Stupefied)
                })
    });
    if let Some(check) = check
        && !unknown_stupefied
    {
        let names = check
            .sources
            .iter()
            .map(|source| source.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        notes.push(RuntimeEffectNoteView {
            source: "casting-flat-check".into(),
            label: names,
            reason: format!("DC {} flat check or the spell is disrupted", check.dc),
        });
    }
    for issue in issues {
        let Some(condition) = participant
            .conditions
            .iter()
            .find(|condition| condition.condition_id == issue.source_condition_id)
        else {
            continue;
        };
        let rule = super::conditions::condition_rule_for_key(condition.condition_key.as_deref());
        let subject = issue.subject.as_ref().map(|subject| match subject {
            ConditionSubjectView::Participant { participant_key } => roster
                .iter()
                .find(|p| p.participant_key == *participant_key)
                .map(|p| p.display_name.clone())
                .unwrap_or_else(|| "the named subject".into()),
            ConditionSubjectView::Other { description } => description.clone(),
        });
        let reason=match rule {
            Some(R::Grabbed) if check.is_none()=>Some("Manipulate actions require a DC 5 flat check.".into()),
            Some(R::Deafened) if check.is_none()=>Some("Auditory actions require a DC 5 flat check.".into()),
            Some(R::Restrained)=>Some("Cannot use attack or manipulate actions while Restrained.".into()),
            Some(R::Paralyzed)=>Some("Only purely mental actions are possible while Paralyzed.".into()),
            Some(R::Confused)=>Some("Use an offensive cantrip or an action the GM permits to facilitate an attack. The GM selects the target.".into()),
            Some(R::Controlled)=>Some(issue.instructions.clone().unwrap_or_else(||"The controller chooses your actions.".into())),
            Some(R::Fascinated)=>Some(format!("Concentrate actions must relate to {}.",subject.as_deref().unwrap_or("the subject of fascination"))),
            Some(R::Fleeing)=>Some(format!("Actions must help escape {}.",subject.as_deref().unwrap_or("the source of flight"))),
            Some(R::Stupefied) if unknown_stupefied=>Some("The Stupefied value is missing; the casting flat-check DC is unknown.".into()),
            _ if issue.kind==EncounterActionIssueKindView::Prohibited=>Some(issue.reason.clone()),
            _=>None,
        };
        if let Some(reason) = reason
            && !notes.iter().any(|note| note.reason == reason)
        {
            notes.push(RuntimeEffectNoteView {
                source: format!("condition:{}", condition.condition_id),
                label: condition.name.clone(),
                reason,
            });
        }
    }
    notes
}

fn casting_unavailable_issue(reason: &str) -> EncounterActionIssueView {
    EncounterActionIssueView {
        subject: None,
        source_condition_id: 0,
        condition_name: "Spell".into(),
        controller_key: None,
        instructions: None,
        kind: EncounterActionIssueKindView::Prohibited,
        fact: None,
        reason: reason.into(),
    }
}
