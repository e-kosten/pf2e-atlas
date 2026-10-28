//! Input validation and DTO projection shared by live condition workflows.
use super::rules::action_context::{ActionFact, ActionIssue, KnownActionFact};
use atlas_app_model::*;

pub(super) fn issue_view(
    issue: ActionIssue,
    participant: &atlas_local_state::EncounterParticipant,
    sources: &super::rules::source_conditions::SourceConditions,
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
            "Confused: use offensive actions and choose targets randomly.".into(),
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
        .find(|condition| Some(condition.condition_id) == source.saved_id());
    let (controller_key, instructions) = source_condition
        .filter(|condition| {
            super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                == Some(super::conditions::ConditionRule::Controlled)
        })
        .and_then(|source| {
            super::rules::action_context::control_direction(participant, source, sources)
        })
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
        source_key: source.key(),
        source_condition_id: source.saved_id(),
        condition_name: source_condition
            .map(|condition| condition.name.clone())
            .unwrap_or_else(|| source.label(std::slice::from_ref(participant))),
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
        ActionFact::VoluntaryIngestion => "Does this action voluntarily consume food or drink?",
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

pub(super) fn strike_facts(
    activity: &super::stat_block::MechanicActivityView,
) -> Vec<KnownActionFact> {
    let mut facts = vec![
        KnownActionFact {
            fact: ActionFact::Attack,
            value: true,
        },
        KnownActionFact {
            fact: ActionFact::MentalOnly,
            value: false,
        },
        KnownActionFact {
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
            facts.push(KnownActionFact {
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
            source_key: "participant-status".into(),
            source_condition_id: None,
            condition_name: "Participant".into(),
            controller_key: None,
            instructions: None,
            kind: EncounterActionIssueKindView::Prohibited,
            fact: None,
            reason: super::rules::capabilities::cannot_act_reason(
                participant,
                super::rules::source_conditions::SourceConditions::empty(),
            )
            .unwrap_or("This participant cannot act.")
            .into(),
        })
}

pub(super) fn check_view(
    dc: i64,
    source_ids: Vec<super::rules::causes::ConditionSourceKey>,
    participants: &[atlas_local_state::EncounterParticipant],
) -> EncounterActionCheckView {
    EncounterActionCheckView {
        dc,
        sources: source_ids
            .into_iter()
            .map(|id| EncounterActionCheckSourceView {
                condition_id: id.saved_id(),
                source_key: id.key(),
                name: id.label(participants),
            })
            .collect(),
    }
}
