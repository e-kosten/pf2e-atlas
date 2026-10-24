use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, AdvanceEncounterTurnRequest,
    ApplyEncounterHpActionRequest, ApplyEncounterSpellActionRequest, CreateEncounterRequest,
    CreateSavedListRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterSavedListRequest, RemoveSavedListItemRequest, ReorderEncounterParticipantRequest,
    ReorderSavedListItemRequest, SearchResultsRequest, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest, UpdateSavedListItemNoteRequest, UpdateSavedListRequest,
};
use atlas_app_model::{DiscoverFilterCountsRequest, RecordDetailRequest};
use atlas_domain::QueryPredicate;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use crate::error::WebError;
use crate::service::{AtlasWebState, call_read_service, call_service};

pub(crate) async fn initiative_options(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.encounter_initiative_options(&encounter_ref)
        })
        .await?,
    ))
}

pub(crate) async fn roll_initiative(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::RollInitiativeRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.roll_encounter_initiative(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn readiness(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || Ok(service.readiness())).await?,
    ))
}

pub(crate) async fn saved_lists(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.saved_lists()).await?,
    ))
}

pub(crate) async fn saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.saved_list(&list_ref)).await?,
    ))
}

pub(crate) async fn filter_saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<FilterSavedListRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.list_ref = list_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.filter_saved_list(request)).await?,
    ))
}

pub(crate) async fn create_saved_list(
    State(state): State<AtlasWebState>,
    payload: Result<Json<CreateSavedListRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.create_saved_list(request)).await?,
    ))
}

pub(crate) async fn update_saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<UpdateSavedListRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.list_key = list_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.update_saved_list(request)).await?,
    ))
}

pub(crate) async fn add_saved_list_item(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<AddSavedListItemRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.list_ref = list_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.add_saved_list_item(request)).await?,
    ))
}

pub(crate) async fn remove_saved_list_item(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<RemoveSavedListItemRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.list_ref = list_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.remove_saved_list_item(request)).await?,
    ))
}

pub(crate) async fn reorder_saved_list_item(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<ReorderSavedListItemRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.list_ref = list_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.reorder_saved_list_item(request)).await?,
    ))
}

pub(crate) async fn delete_saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.delete_saved_list(&list_ref)).await?,
    ))
}

pub(crate) async fn encounters(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounters()).await?,
    ))
}

pub(crate) async fn encounter_condition_definitions(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounter_condition_definitions()).await?,
    ))
}

pub(crate) async fn encounter(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounter(&encounter_ref)).await?,
    ))
}

pub(crate) async fn create_encounter(
    State(state): State<AtlasWebState>,
    payload: Result<Json<CreateEncounterRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.create_encounter(request)).await?,
    ))
}

pub(crate) async fn delete_encounter(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.delete_encounter(&encounter_ref)).await?,
    ))
}

pub(crate) async fn update_encounter(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<UpdateEncounterRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.encounter_key = encounter_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.update_encounter(request)).await?,
    ))
}

pub(crate) async fn add_encounter_record_participant(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<AddEncounterRecordParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.encounter_ref = encounter_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_record_participant(request)
        })
        .await?,
    ))
}

pub(crate) async fn add_encounter_manual_participant(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<AddEncounterManualParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.encounter_ref = encounter_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_manual_participant(request)
        })
        .await?,
    ))
}

pub(crate) async fn update_encounter_participant(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<UpdateEncounterParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.participant_key = participant_key;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_encounter_participant(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn reorder_encounter_participant(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<ReorderEncounterParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.reorder_encounter_participant(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn apply_encounter_hp_action(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<ApplyEncounterHpActionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.participant_key = participant_key;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.apply_encounter_hp_action(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn apply_encounter_spell_action(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<ApplyEncounterSpellActionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.participant_key = participant_key;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.apply_encounter_spell_action(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn remove_encounter_participant(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.remove_encounter_participant(&encounter_ref, &participant_key)
        })
        .await?,
    ))
}

pub(crate) async fn encounter_turn_review(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounter_turn_review(&encounter_ref)).await?,
    ))
}

pub(crate) async fn advance_encounter_turn(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<AdvanceEncounterTurnRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.advance_encounter_turn(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn set_encounter_turn(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<SetEncounterTurnRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.encounter_ref = encounter_ref;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.set_encounter_turn(request)).await?,
    ))
}

pub(crate) async fn add_encounter_participant_condition(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<AddEncounterParticipantConditionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.participant_key = participant_key;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_participant_condition(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn update_encounter_participant_condition(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key, condition_id)): Path<(String, String, i64)>,
    payload: Result<Json<UpdateEncounterParticipantConditionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.condition_id = condition_id;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_encounter_participant_condition(
                &encounter_ref,
                &participant_key,
                request,
            )
        })
        .await?,
    ))
}

pub(crate) async fn remove_encounter_participant_condition(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key, condition_id)): Path<(String, String, i64)>,
    payload: Result<
        Json<atlas_app_model::RemoveEncounterParticipantConditionRequest>,
        JsonRejection,
    >,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.remove_encounter_participant_condition(
                &encounter_ref,
                &participant_key,
                condition_id,
                request,
            )
        })
        .await?,
    ))
}

pub(crate) async fn discover_filter_editor(
    State(state): State<AtlasWebState>,
    payload: Result<Json<DiscoverFilterEditorRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.discover_filter_editor(request)).await?,
    ))
}

pub(crate) async fn discover_filter_values(
    State(state): State<AtlasWebState>,
    payload: Result<Json<DiscoverFilterValuesRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.discover_filter_values(request, &execution)
        })
        .await?,
    ))
}

pub(crate) async fn search_page(
    State(state): State<AtlasWebState>,
    payload: Result<Json<SearchResultsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.search_page(request, &execution)
        })
        .await?,
    ))
}

pub(crate) async fn record_detail(
    State(state): State<AtlasWebState>,
    Path(record_key): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.record_detail(&record_key)).await?,
    ))
}
pub(crate) async fn record_detail_at(
    State(state): State<AtlasWebState>,
    payload: Result<Json<RecordDetailRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.record_detail_at(request)).await?,
    ))
}
pub(crate) async fn encounter_reference(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<atlas_app_model::EncounterReferenceRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.encounter_reference(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}
pub(crate) async fn embedded_reading(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::EmbeddedReadingRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.embedded_reading(request)).await?,
    ))
}
pub(crate) async fn discover_filter_counts(
    State(state): State<AtlasWebState>,
    payload: Result<Json<DiscoverFilterCountsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.discover_filter_counts(request, &execution)
        })
        .await?,
    ))
}
pub(crate) async fn validate_filter(
    State(state): State<AtlasWebState>,
    payload: Result<Json<QueryPredicate>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.validate_filter(request)).await?,
    ))
}

pub(crate) async fn related_records(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::RelatedRecordsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.related_records(request)).await?,
    ))
}

pub(crate) async fn roll_flat_check(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::RollFlatCheckRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.roll_flat_check(request)).await?,
    ))
}

pub(crate) async fn roll_table(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::RollTableRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.roll_table(request)).await?,
    ))
}

pub(crate) async fn encounter_strike_options(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::EncounterStrikeOptionsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.encounter_strike_options(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn roll_encounter_strike(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::RollEncounterStrikeRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.roll_encounter_strike(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn participant_encounter_condition_definitions(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.participant_encounter_condition_definitions(&encounter_ref, &participant_key)
        })
        .await?,
    ))
}

pub(crate) async fn preview_encounter_condition_add(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<atlas_app_model::AddEncounterParticipantConditionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.participant_key = participant_key;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_condition_add(&encounter_ref, request)
        })
        .await?,
    ))
}
pub(crate) async fn preview_encounter_condition_update(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key, condition_id)): Path<(String, String, i64)>,
    payload: Result<
        Json<atlas_app_model::UpdateEncounterParticipantConditionRequest>,
        JsonRejection,
    >,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.condition_id = condition_id;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_condition_update(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}
pub(crate) async fn pending_encounter_condition_consequences(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.pending_encounter_condition_consequences(&encounter_ref)
        })
        .await?,
    ))
}
pub(crate) async fn preview_encounter_drained_consequence(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::ResolveDrainedConsequenceRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_drained_consequence(&encounter_ref, request)
        })
        .await?,
    ))
}
pub(crate) async fn resolve_encounter_drained_consequence(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::ResolveDrainedConsequenceRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.resolve_encounter_drained_consequence(&encounter_ref, request)
        })
        .await?,
    ))
}
pub(crate) async fn set_encounter_life_settings(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<atlas_app_model::SetEncounterLifeSettingsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.set_encounter_life_settings(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

pub(crate) async fn recover_persistent_damage(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::RecoverPersistentDamageRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.recover_persistent_damage(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn pending_encounter_hp_lifecycle(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.pending_encounter_hp_lifecycle(&encounter_ref)
        })
        .await?,
    ))
}
pub(crate) async fn resolve_encounter_hp_lifecycle(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::ResolveEncounterHpLifecycleRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.resolve_encounter_hp_lifecycle(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn prepare_persistent_damage(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::PreparePersistentDamageRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.prepare_persistent_damage(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn apply_persistent_damage(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::ApplyPersistentDamageRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.apply_persistent_damage(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn preview_encounter_strike_context(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::EncounterStrikeContextRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_strike_context(&encounter_ref, request)
        })
        .await?,
    ))
}
pub(crate) async fn encounter_casting_reminders(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::EncounterCastingRemindersRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.encounter_casting_reminders(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) type PersistentReviewQuery = std::collections::HashMap<String, String>;
pub(crate) async fn persistent_damage_review(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    axum::extract::Query(query): axum::extract::Query<PersistentReviewQuery>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.persistent_damage_review(
                &encounter_ref,
                query
                    .get("participant_key")
                    .map(String::as_str)
                    .unwrap_or(""),
                query.get("boundary_key").map(String::as_str),
            )
        })
        .await?,
    ))
}

pub(crate) async fn preview_encounter_hp_lifecycle(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::ResolveEncounterHpLifecycleRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_hp_lifecycle(&encounter_ref, request)
        })
        .await?,
    ))
}

pub(crate) async fn apply_condition_lifecycle(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    Json(request): Json<atlas_app_model::ApplyConditionLifecycleRequest>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.apply_condition_lifecycle(&encounter_ref, request)
        })
        .await?,
    ))
}
pub(crate) async fn condition_lifecycle_actions(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    axum::extract::Query(query): axum::extract::Query<PersistentReviewQuery>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.condition_lifecycle_actions(
                &encounter_ref,
                query
                    .get("participant_key")
                    .map(String::as_str)
                    .unwrap_or(""),
                query.get("boundary_key").map(String::as_str),
            )
        })
        .await?,
    ))
}
pub(crate) async fn pending_condition_recoveries(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.pending_condition_recoveries(&encounter_ref)
        })
        .await?,
    ))
}

pub(crate) async fn saved_list_item_note(
    State(state): State<AtlasWebState>,
    Path((list_ref, record_key)): Path<(String, String)>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.saved_list_item_note(&list_ref, &record_key)
        })
        .await?,
    ))
}

pub(crate) async fn update_saved_list_item_note(
    State(state): State<AtlasWebState>,
    Path((list_ref, record_key)): Path<(String, String)>,
    payload: Result<Json<UpdateSavedListItemNoteRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(mut request) = payload.map_err(WebError::invalid_request)?;
    request.list_ref = list_ref;
    request.record_key = record_key;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.update_saved_list_item_note(request)).await?,
    ))
}

pub(crate) async fn trait_definition(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::TraitDetailRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.trait_definition(request, &execution)
        })
        .await?,
    ))
}

pub(crate) async fn trait_records(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::TraitRecordsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.trait_records(request, &execution)
        })
        .await?,
    ))
}

pub(crate) async fn relationship_occurrences(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::RelationshipOccurrencesRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.relationship_occurrences(request, &execution)
        })
        .await?,
    ))
}
