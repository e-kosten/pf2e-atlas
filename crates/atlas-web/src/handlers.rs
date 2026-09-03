use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, CreateEncounterRequest,
    CreateSavedListRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    EncounterSpellCastRequest, FilterSavedListRequest, OpenResultWindowRequest,
    ReadResultWindowPageRequest, RemoveSavedListItemRequest, ReorderEncounterParticipantRequest,
    ResetEncounterParticipantRequest, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest, UpdateSavedListRequest,
};
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use crate::error::{WebError, parse_window_id};
use crate::service::{AtlasWebState, call_service};

pub(crate) async fn readiness(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    Ok(Json(state.service.readiness()))
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
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.remove_encounter_participant_condition(
                &encounter_ref,
                &participant_key,
                condition_id,
            )
        })
        .await?,
    ))
}

pub(crate) async fn mutate_encounter_spell_cast(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<EncounterSpellCastRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.mutate_encounter_spell_cast(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

pub(crate) async fn reset_encounter_participant(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<ResetEncounterParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.reset_encounter_participant(&encounter_ref, &participant_key, request)
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
        call_service(state, move || service.discover_filter_values(request)).await?,
    ))
}

pub(crate) async fn open_result_window(
    State(state): State<AtlasWebState>,
    payload: Result<Json<OpenResultWindowRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.open_result_window(request)).await?,
    ))
}

pub(crate) async fn read_result_window_page(
    State(state): State<AtlasWebState>,
    Path(window_id): Path<String>,
    payload: Result<Json<ReadResultWindowPageRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let window_id = parse_window_id(&window_id)?;
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.read_result_window_page(window_id, request)
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
