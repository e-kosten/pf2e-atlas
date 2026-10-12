//! Complete selected references composed from one checked parent and current local state.
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{AppErrorCode, EncounterReferenceRequest, RecordBodyView, RecordDetailView};
use atlas_record::source_record::{SourceFieldView, SourceQueryView};

impl AtlasAppService {
    pub fn encounter_reference(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: EncounterReferenceRequest,
    ) -> AppServiceResult<RecordDetailView> {
        let encounter = self
            .local_state_store()?
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        let participant = encounter
            .participants
            .into_iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    "participant is no longer in this encounter",
                )
            })?;
        if participant.record_key.as_deref() != Some(request.selection.record_key.as_str()) {
            return Err(AppServiceError::invalid_request(
                "selected reference does not belong to this participant",
            ));
        }
        self.submit_retrieval(move |r| {
            if r.artifact_generation()? != request.artifact_generation {
                return Err(AppServiceError::invalid_request(
                    "the reference collection has changed; reload the encounter and reopen the reference",
                ));
            }
            let (source, mut selected) = crate::records::read_record_detail(r, request.selection)?;
            let root_node = source.source.node_at(&[]).ok_or_else(|| {
                AppServiceError::invalid_request("participant source is unavailable")
            })?;
            let root = crate::presentation::record_presentation(
                &source.source, root_node, &selected.record, &[], vec![], r,
            );
            let baseline = super::mechanics::arithmetic_baseline(&root).ok_or_else(|| {
                AppServiceError::invalid_request("this participant has no actor runtime context")
            })?;
            let adjustment_applicable = !matches!(
                SourceQueryView::new(source.source.source(), source.source.key().pack().as_str(), "")
                    .actor().authored_adjustment(),
                SourceFieldView::NotApplicable
            );
            let block = super::mechanics::participant_stat_block(
                &participant, &baseline, adjustment_applicable,
            );
            let current = crate::presentation::compose_participant(root, &block);
            compose_selected(&mut selected, current)?;
            Ok(selected)
        })
    }
}

fn compose_selected(
    selected: &mut RecordDetailView,
    current: atlas_app_model::RecordPresentationView,
) -> AppServiceResult<()> {
    if selected.selected.owners.is_empty() {
        // Retain selected root content; hydration deliberately has none.
        let content = std::mem::take(&mut selected.presentation.content);
        selected.presentation = current;
        selected.presentation.content = content;
        return Ok(());
    }
    let actor = match &current.body {
        RecordBodyView::Creature(actor) => actor.as_ref(),
        RecordBodyView::Hazard(hazard) => &hazard.actor,
        _ => {
            return Err(AppServiceError::invalid_request(
                "participant actor is unavailable",
            ));
        }
    };
    let activities = actor.activities.value.as_deref().unwrap_or_default();
    let matching = activities
        .iter()
        .find(|activity| activity.navigation.owners == selected.selected.owners);
    match &mut selected.presentation.body {
        RecordBodyView::Activity(activity) => {
            let matching = matching.ok_or_else(|| {
                AppServiceError::invalid_request(
                    "selected activity is no longer available on this participant",
                )
            })?;
            let mut context = activity.casting_context.take();
            if let Some(context) = &mut context {
                context.spells = atlas_app_model::FactView {
                    state: actor.activities.state,
                    value: actor.activities.value.as_ref().map(|all| {
                        all.iter()
                            .filter(|spell| {
                                spell.casting_entry.as_ref() == Some(&matching.navigation)
                            })
                            .cloned()
                            .collect()
                    }),
                };
            }
            **activity = matching.clone();
            activity.casting_context = context;
        }
        RecordBodyView::Spell(spell) => {
            let matching = matching.ok_or_else(|| {
                AppServiceError::invalid_request(
                    "selected spell is no longer available on this participant",
                )
            })?;
            spell.runtime_notes = Some(matching.notes.clone());
        }
        // Equipment and other owned references preserve their complete authored view.
        _ => {}
    }
    Ok(())
}
