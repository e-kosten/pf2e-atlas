//! Complete selected references composed from one checked parent and current local state.
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AppErrorCode, AppRecoverableAction, EncounterReferenceRequest, RecordBodyView, RecordDetailView,
};
use atlas_record::source_record::{SourceFieldView, SourceQueryView};

impl AtlasAppService {
    pub fn encounter_reference(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: EncounterReferenceRequest,
    ) -> AppServiceResult<RecordDetailView> {
        let encounter_ref = encounter_ref.to_owned();
        let participant_key = participant_key.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| {
                read_encounter_reference(workspace, &encounter_ref, &participant_key, request)
            },
        )
    }
}

pub(super) fn read_encounter_reference(
    workspace: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    participant_key: &str,
    request: EncounterReferenceRequest,
) -> AppServiceResult<RecordDetailView> {
    let encounter = workspace
        .local_state()
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
    let participant = encounter
        .participants
        .iter()
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
    let r = workspace.retrieval_mut();

    let generation = r.artifact_generation()?;
    if generation != request.artifact_generation {
        return Err(AppServiceError::artifact_generation_changed(
            &request.artifact_generation,
            &generation,
            AppRecoverableAction::ReloadRecord,
        ));
    }
    let (source, mut selected) = crate::records::read_record_detail(r, request.selection)?;
    let root_node = source
        .source
        .node_at(&[])
        .ok_or_else(|| AppServiceError::invalid_request("participant source is unavailable"))?;
    let root = crate::presentation::record_presentation(
        &source.source,
        root_node,
        &selected.record,
        &[],
        vec![],
        r,
    )?;
    let baseline = super::rules::source_baseline(
        &source.source,
        r.source_fingerprint(),
        selected
            .selected
            .spell_rank
            .map(|rank| (selected.selected.owners.as_slice(), rank)),
    )
    .ok_or_else(|| {
        AppServiceError::invalid_request("this participant has no actor runtime context")
    })?;
    let adjustment_applicable = !matches!(
        SourceQueryView::new(
            source.source.source(),
            source.source.key().pack().as_str(),
            ""
        )
        .actor()
        .authored_adjustment(),
        SourceFieldView::NotApplicable
    );
    let relationship_causes = super::rules::relationships::observer_defense_causes(
        participant_key,
        &encounter.participants,
    );
    let block = super::rules::project_participant(
        participant,
        &baseline,
        adjustment_applicable,
        &relationship_causes,
    );
    let current = crate::presentation::compose_participant(root, &block);
    compose_selected(&mut selected, current, &block)?;
    Ok(selected)
}

fn compose_selected(
    selected: &mut RecordDetailView,
    current: atlas_app_model::RecordPresentationView,
    block: &super::stat_block::StatBlockView,
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
            if let Some(evaluated) = block
                .activities
                .iter()
                .find(|activity| activity.navigation.owners == selected.selected.owners)
            {
                for component in spell.damage.value.iter_mut().flatten() {
                    if let Some(damage) = evaluated
                        .damage
                        .iter()
                        .find(|damage| damage.damage_id == component.id)
                    {
                        if damage.unavailable_reason.is_some() {
                            component.formula.value = None;
                            component.formula.state = atlas_domain::QueryFieldState::Missing;
                        } else {
                            component.formula.value = Some(
                                damage
                                    .adjusted_formula
                                    .as_ref()
                                    .unwrap_or(&damage.formula)
                                    .clone(),
                            );
                        }
                    }
                }
            }
            spell.runtime_notes = Some(matching.notes.clone());
            spell.runtime_attack = Some(matching.attack.clone());
            spell.runtime_difficulty_class = Some(matching.difficulty_class.clone());
        }
        // Equipment and other owned references preserve their complete authored view.
        _ => {}
    }
    Ok(())
}
