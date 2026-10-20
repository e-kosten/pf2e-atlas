//! Retch decisions shared by its read-only preview and guarded recovery action.
use super::rules::action_context::{
    ActionContext, ActionEvaluation, ActionFact, ActionIssue, ActionJudgment, ActionKind,
    ConfusionChoice, evaluate_action,
};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
impl AtlasAppService {
    pub fn preview_encounter_retch(
        &self,
        encounter_ref: &str,
        request: EncounterRetchPreviewRequest,
    ) -> AppServiceResult<EncounterRetchPreviewView> {
        self.with_state(|session| {
            let snapshot = session
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
            let confusion = super::action_projection::checked_confusion(
                &snapshot.participants,
                participant,
                request.confusion.as_ref(),
            )?;
            let judged = evaluate(participant, &request.judgments, confusion.as_ref())?;
            let mut issues: Vec<_> = judged
                .issues
                .into_iter()
                .map(|issue| super::action_projection::issue_view(issue, participant))
                .collect();
            issues.extend(super::action_projection::inactive_issue(participant));
            Ok(EncounterRetchPreviewView {
                issues,
                confusion: super::action_projection::confusion_view(
                    participant,
                    confusion.as_ref(),
                    ActionKind::Retch,
                    false,
                ),
            })
        })
    }
}
fn retch_facts() -> Vec<ActionJudgment> {
    [
        ActionFact::Manipulate,
        ActionFact::Auditory,
        ActionFact::Concentrate,
        ActionFact::Move,
        ActionFact::Attack,
        ActionFact::MentalOnly,
        ActionFact::VoluntaryIngestion,
    ]
    .into_iter()
    .map(|fact| ActionJudgment {
        source_id: None,
        fact,
        value: false,
    })
    .collect()
}

/// Retch has fixed traits; only source-specific behavioral decisions are editable.
/// Those questions are independent, so one evaluation identifies all valid answers.
pub(super) fn evaluate(
    participant: &atlas_local_state::EncounterParticipant,
    judgments: &[EncounterActionJudgmentView],
    confusion: Option<&ConfusionChoice>,
) -> AppServiceResult<ActionEvaluation> {
    if judgments.len() > 200 {
        return Err(AppServiceError::invalid_request(
            "Too many action judgments.",
        ));
    }
    let mut facts = retch_facts();
    let context = ActionContext {
        kind: ActionKind::Retch,
        reaction: Some(false),
        other_participant: None,
        confusion,
        facts: &[],
    };
    let initial = evaluate_action(
        participant,
        &ActionContext {
            facts: &facts,
            ..context
        },
    );
    for judgment in judgments {
        let fact = super::action_projection::fact_from_view(judgment.fact);
        let source_id = judgment.source_condition_id.ok_or_else(|| {
            AppServiceError::invalid_request("Choose the condition source for this judgment.")
        })?;
        if facts.iter().any(|known| {
            known.fact == fact
                && (known.source_id.is_none() || known.source_id == Some(source_id))
                && known.value != judgment.value
        }) {
            return Err(AppServiceError::invalid_request(
                "An action judgment contradicts established facts.",
            ));
        }
        if !initial.issues.iter().any(|issue| {
            matches!(issue, ActionIssue::NeedsFact { source_id: id, fact: needed }
                if *id == source_id && *needed == fact)
        }) {
            return Err(AppServiceError::invalid_request(
                "That fact is not required for this action.",
            ));
        }
        facts.push(ActionJudgment {
            source_id: Some(source_id),
            fact,
            value: judgment.value,
        });
    }
    Ok(evaluate_action(
        participant,
        &ActionContext {
            facts: &facts,
            ..context
        },
    ))
}
