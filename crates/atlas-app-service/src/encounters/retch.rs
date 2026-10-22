//! Fixed Retch traits feed the shared action evaluator; unknown facts stay hints.
use super::rules::action_context::{
    ActionContext, ActionEvaluation, ActionFact, ActionJudgment, ActionKind, evaluate_action,
};
pub(super) fn evaluate(participant: &atlas_local_state::EncounterParticipant) -> ActionEvaluation {
    let facts = [
        ActionFact::Manipulate,
        ActionFact::Auditory,
        ActionFact::Concentrate,
        ActionFact::Move,
        ActionFact::Attack,
        ActionFact::MentalOnly,
        ActionFact::VoluntaryIngestion,
    ]
    .map(|fact| ActionJudgment {
        source_id: None,
        fact,
        value: false,
    });
    evaluate_action(
        participant,
        &ActionContext {
            kind: ActionKind::Retch,
            reaction: Some(false),
            other_participant: None,
            facts: &facts,
        },
    )
}
