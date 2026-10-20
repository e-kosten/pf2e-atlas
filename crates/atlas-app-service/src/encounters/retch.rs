//! Read-only retch decisions and strike targeting requirements.
use super::rules::action_context::{ActionContext, ActionFact, ActionJudgment, ActionKind};
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
            let facts = retch_facts();
            let judged = super::action_projection::apply_judgments(
                participant,
                ActionContext {
                    kind: ActionKind::Retch,
                    reaction: Some(false),
                    other_participant: None,
                    confusion: confusion.as_ref(),
                    facts: &facts,
                },
                &request.judgments,
            )?;
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
pub(super) fn retch_facts() -> Vec<ActionJudgment> {
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
