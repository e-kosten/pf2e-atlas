//! Input validation and DTO projection shared by live condition workflows.
use super::rules::action_context::{ActionFact, ActionIssue, ActionJudgment, ActionKind};
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::*;

pub(super) fn issue_view(
    issue: ActionIssue,
    participant: &atlas_local_state::EncounterParticipant,
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
    let source_condition = participant
        .conditions
        .iter()
        .find(|condition| condition.condition_id == source);
    let (controller_key, instructions) = source_condition
        .filter(|condition| {
            super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                == Some(super::conditions::ConditionRule::Controlled)
        })
        .and_then(|source| super::rules::action_context::control_direction(participant, source))
        .map(|(key, instructions)| (Some(key), instructions))
        .unwrap_or((None, None));
    let subject = source_condition.and_then(|condition| match &condition.parameters {
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
        condition_name: source_condition
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
        ActionFact::RelatesToFascination => {
            "Does this casting relate to this fascination's subject?"
        }
        ActionFact::FurthersEscape => "Does this action help escape the named threat?",
        ActionFact::Cantrip => "Is this spell a cantrip?",
        ActionFact::VoluntaryIngestion => "Does this action voluntarily consume food or drink?",
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
        ActionFact::RelatesToFascination => EncounterActionFactView::RelatesToFascination,
        ActionFact::FurthersEscape => EncounterActionFactView::FurthersEscape,
        ActionFact::Cantrip => EncounterActionFactView::Cantrip,
        ActionFact::VoluntaryIngestion => EncounterActionFactView::VoluntaryIngestion,
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
        _ => "This action is prohibited by its condition.",
    }
}

pub(super) fn confusion_from_view(
    value: &EncounterConfusionContextView,
) -> AppServiceResult<super::rules::action_context::ConfusionChoice> {
    use super::rules::action_context::ConfusionChoice as C;
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
    choice: Option<&super::rules::action_context::ConfusionChoice>,
) -> AppServiceResult<()> {
    if let Some(super::rules::action_context::ConfusionChoice::Offensive {
        viable_target_keys,
        ..
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

pub(super) fn confusion_view(
    participant: &atlas_local_state::EncounterParticipant,
    choice: Option<&super::rules::action_context::ConfusionChoice>,
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
) -> AppServiceResult<Option<super::rules::action_context::ConfusionChoice>> {
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

pub(super) fn strike_facts(
    activity: &super::stat_block::MechanicActivityView,
) -> Vec<ActionJudgment> {
    let mut facts = vec![
        ActionJudgment {
            source_id: None,
            fact: ActionFact::Attack,
            value: true,
        },
        ActionJudgment {
            source_id: None,
            fact: ActionFact::MentalOnly,
            value: false,
        },
        ActionJudgment {
            source_id: None,
            fact: ActionFact::VoluntaryIngestion,
            value: false,
        },
    ];
    if activity.traits_known {
        for (fact, name) in [
            (ActionFact::Manipulate, "manipulate"),
            (ActionFact::Auditory, "auditory"),
            (ActionFact::Move, "move"),
            (ActionFact::Concentrate, "concentrate"),
        ] {
            facts.push(ActionJudgment {
                source_id: None,
                fact,
                value: activity.traits.iter().any(|item| item == name),
            });
        }
    }
    facts
}

pub(super) fn inactive_issue(
    participant: &atlas_local_state::EncounterParticipant,
) -> Option<EncounterActionIssueView> {
    (participant.defeated
        || participant.hazard_disabled
        || participant.mortality == atlas_local_state::MortalityState::Dead)
        .then(|| EncounterActionIssueView {
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
        })
}

pub(super) fn check_view(
    dc: i64,
    source_ids: Vec<i64>,
    participants: &[atlas_local_state::EncounterParticipant],
) -> EncounterActionCheckView {
    EncounterActionCheckView {
        dc,
        sources: source_ids
            .into_iter()
            .map(|id| EncounterActionCheckSourceView {
                condition_id: id,
                name: participants
                    .iter()
                    .flat_map(|participant| &participant.conditions)
                    .find(|source| source.condition_id == id)
                    .map(|source| source.name.clone())
                    .unwrap_or_default(),
            })
            .collect(),
    }
}
