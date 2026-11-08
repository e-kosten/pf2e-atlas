use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, AdvanceEncounterTurnRequest,
    ApplyEncounterHpActionRequest, ApplyEncounterSpellActionRequest, CreateEncounterRequest,
    CreateSavedListRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterSavedListRequest, RemoveSavedListItemRequest, ReorderEncounterParticipantRequest,
    ReorderSavedListItemRequest, SavedEditPayload, SearchResultsRequest, SetEncounterTurnRequest,
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

#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/initiative", operation_id = "initiativeOptions", params(("encounter_ref" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::InitiativeOptionsView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/initiative", operation_id = "rollInitiative", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::RollInitiativeRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::InitiativeRollView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(get, path = "/api/readiness", operation_id = "readiness", responses((status = 200, description = "Successful response", body = atlas_app_model::AppReadinessView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn readiness(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || Ok(service.readiness())).await?,
    ))
}

#[utoipa::path(get, path = "/api/lists", operation_id = "savedLists", responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListIndexView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn saved_lists(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.saved_lists()).await?,
    ))
}

#[utoipa::path(get, path = "/api/lists/{list_ref}", operation_id = "savedList", params(("list_ref" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.saved_list(&list_ref)).await?,
    ))
}

#[utoipa::path(post, path = "/api/lists/{list_ref}/filter", operation_id = "filterSavedList", params(("list_ref" = String, Path)), request_body = FilterSavedListRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn filter_saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<FilterSavedListRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.filter_saved_list(&list_ref, request)).await?,
    ))
}

#[utoipa::path(post, path = "/api/lists", operation_id = "createSavedList", request_body = CreateSavedListRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListCreateView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(patch, path = "/api/lists/{list_ref}", operation_id = "updateSavedList", params(("list_ref" = String, Path)), request_body = UpdateSavedListRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListUpdateView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<SavedEditPayload<UpdateSavedListRequest>>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(SavedEditPayload(request)) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.update_saved_list(&list_ref, request)).await?,
    ))
}

#[utoipa::path(post, path = "/api/lists/{list_ref}/items", operation_id = "addSavedListItem", params(("list_ref" = String, Path)), request_body = AddSavedListItemRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListItemMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn add_saved_list_item(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<AddSavedListItemRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_saved_list_item(&list_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(delete, path = "/api/lists/{list_ref}/items", operation_id = "removeSavedListItem", params(("list_ref" = String, Path)), request_body = RemoveSavedListItemRequest, responses((status = 200, description = "Saved list item removed", body = atlas_app_model::SavedListItemMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn remove_saved_list_item(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<RemoveSavedListItemRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.remove_saved_list_item(&list_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/lists/{list_ref}/items/reorder", operation_id = "reorderSavedListItem", params(("list_ref" = String, Path)), request_body = ReorderSavedListItemRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListItemMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn reorder_saved_list_item(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
    payload: Result<Json<ReorderSavedListItemRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.reorder_saved_list_item(&list_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(delete, path = "/api/lists/{list_ref}", operation_id = "deleteSavedList", params(("list_ref" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::DeleteSavedListView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn delete_saved_list(
    State(state): State<AtlasWebState>,
    Path(list_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.delete_saved_list(&list_ref)).await?,
    ))
}

#[utoipa::path(get, path = "/api/encounters", operation_id = "encounters", responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterIndexView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn encounters(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounters()).await?,
    ))
}

#[utoipa::path(get, path = "/api/encounters/condition-definitions", operation_id = "encounterConditionDefinitions", responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterConditionCatalogView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn encounter_condition_definitions(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounter_condition_definitions()).await?,
    ))
}

#[utoipa::path(get, path = "/api/encounters/{encounter_ref}", operation_id = "encounter", params(("encounter_ref" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn encounter(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounter(&encounter_ref)).await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters", operation_id = "createEncounter", request_body = CreateEncounterRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterCreateView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(delete, path = "/api/encounters/{encounter_ref}", operation_id = "deleteEncounter", params(("encounter_ref" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::DeleteEncounterView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn delete_encounter(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.delete_encounter(&encounter_ref)).await?,
    ))
}

#[utoipa::path(patch, path = "/api/encounters/{encounter_ref}", operation_id = "updateEncounter", params(("encounter_ref" = String, Path)), request_body = UpdateEncounterRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_encounter(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<SavedEditPayload<UpdateEncounterRequest>>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(SavedEditPayload(request)) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_encounter(&encounter_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/record", operation_id = "addEncounterRecordParticipant", params(("encounter_ref" = String, Path)), request_body = AddEncounterRecordParticipantRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn add_encounter_record_participant(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<AddEncounterRecordParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_record_participant(&encounter_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/manual", operation_id = "addEncounterManualParticipant", params(("encounter_ref" = String, Path)), request_body = AddEncounterManualParticipantRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn add_encounter_manual_participant(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<AddEncounterManualParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_manual_participant(&encounter_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(patch, path = "/api/encounters/{encounter_ref}/participants/{participant_key}", operation_id = "updateEncounterParticipant", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = UpdateEncounterParticipantRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_encounter_participant(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<UpdateEncounterParticipantRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_encounter_participant(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/reorder", operation_id = "reorderEncounterParticipant", params(("encounter_ref" = String, Path)), request_body = ReorderEncounterParticipantRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/hp", operation_id = "applyEncounterHpAction", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = ApplyEncounterHpActionRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn apply_encounter_hp_action(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<ApplyEncounterHpActionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.apply_encounter_hp_action(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/spells", operation_id = "applyEncounterSpellAction", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = ApplyEncounterSpellActionRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn apply_encounter_spell_action(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<ApplyEncounterSpellActionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.apply_encounter_spell_action(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(delete, path = "/api/encounters/{encounter_ref}/participants/{participant_key}", operation_id = "removeEncounterParticipant", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/turn-review", operation_id = "encounterTurnReview", params(("encounter_ref" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterTurnReviewView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn encounter_turn_review(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.encounter_turn_review(&encounter_ref)).await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/advance", operation_id = "advanceEncounterTurn", params(("encounter_ref" = String, Path)), request_body = AdvanceEncounterTurnRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/turn", operation_id = "setEncounterTurn", params(("encounter_ref" = String, Path)), request_body = SetEncounterTurnRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn set_encounter_turn(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<SetEncounterTurnRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.set_encounter_turn(&encounter_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/conditions", operation_id = "addEncounterParticipantCondition", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = AddEncounterParticipantConditionRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn add_encounter_participant_condition(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<AddEncounterParticipantConditionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_participant_condition(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(patch, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/conditions/{condition_id}", operation_id = "updateEncounterParticipantCondition", params(("encounter_ref" = String, Path), ("participant_key" = String, Path), ("condition_id" = String, Path, pattern = "^-?[0-9]+$")), request_body = UpdateEncounterParticipantConditionRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_encounter_participant_condition(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key, condition_id)): Path<(String, String, i64)>,
    payload: Result<Json<UpdateEncounterParticipantConditionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_encounter_participant_condition(
                &encounter_ref,
                &participant_key,
                condition_id,
                request,
            )
        })
        .await?,
    ))
}

#[utoipa::path(delete, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/conditions/{condition_id}", operation_id = "removeEncounterParticipantCondition", params(("encounter_ref" = String, Path), ("participant_key" = String, Path), ("condition_id" = String, Path, pattern = "^-?[0-9]+$")), request_body = atlas_app_model::RemoveEncounterParticipantConditionRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/filters/editor", operation_id = "discoverFilterEditor", request_body = DiscoverFilterEditorRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::FilterEditorView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/filters/values", operation_id = "discoverFilterValues", request_body = DiscoverFilterValuesRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::FilterValueListView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/search/page", operation_id = "searchPage", request_body = SearchResultsRequest, responses((status = 200, description = "Search results", body = atlas_app_model::SearchResultsPage), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/search/similarity/parts", operation_id = "similarityParts", request_body = atlas_app_model::SimilarityPartsRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::SimilarityPartsView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn similarity_parts(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::SimilarityPartsRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.similarity_parts(request, &execution)
        })
        .await?,
    ))
}

#[utoipa::path(get, path = "/api/records/{record_key}", operation_id = "getRecordDetail", params(("record_key" = String, Path)), responses((status = 200, description = "Record detail", body = atlas_app_model::RecordDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn record_detail(
    State(state): State<AtlasWebState>,
    Path(record_key): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.record_detail(&record_key)).await?,
    ))
}
#[utoipa::path(post, path = "/api/records/detail", operation_id = "recordDetailAt", request_body = RecordDetailRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::RecordDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/reference", operation_id = "encounterReference", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = atlas_app_model::EncounterReferenceRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::RecordDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/records/embedded-reading", operation_id = "embeddedReading", request_body = atlas_app_model::EmbeddedReadingRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EmbeddedReadingBatchView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/filters/counts", operation_id = "discoverFilterCounts", request_body = DiscoverFilterCountsRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::FilterCountsView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/filters/validate", operation_id = "validateFilter", request_body = QueryPredicate, responses((status = 200, description = "Successful response", body = atlas_app_model::FilterValidationResult), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/related", operation_id = "relatedRecords", request_body = atlas_app_model::RelatedRecordsRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::RelatedRecordsView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/rolls/content", operation_id = "rollContent", request_body = atlas_app_model::RollContentRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::ContentRollResultView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn roll_content(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::RollContentRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.roll_content(request)).await?,
    ))
}

#[utoipa::path(post, path = "/api/rolls/flat-check", operation_id = "rollFlatCheck", request_body = atlas_app_model::RollFlatCheckRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::FlatCheckRollView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/rolls/table", operation_id = "rollTable", request_body = atlas_app_model::RollTableRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::TableRollView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/strike-options", operation_id = "encounterStrikeOptions", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::EncounterStrikeOptionsRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterStrikeOptionsView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/strike-rolls", operation_id = "rollEncounterStrike", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::RollEncounterStrikeRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterStrikeRollView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/condition-definitions", operation_id = "participantEncounterConditionDefinitions", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterConditionCatalogView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/conditions/preview", operation_id = "previewEncounterConditionAdd", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = atlas_app_model::AddEncounterParticipantConditionRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::ConditionEventPreviewView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn preview_encounter_condition_add(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<atlas_app_model::AddEncounterParticipantConditionRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_condition_add(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/conditions/{condition_id}/preview", operation_id = "previewEncounterConditionUpdate", params(("encounter_ref" = String, Path), ("participant_key" = String, Path), ("condition_id" = String, Path, pattern = "^-?[0-9]+$")), request_body = atlas_app_model::UpdateEncounterParticipantConditionRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::ConditionEventPreviewView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn preview_encounter_condition_update(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key, condition_id)): Path<(String, String, i64)>,
    payload: Result<
        Json<atlas_app_model::UpdateEncounterParticipantConditionRequest>,
        JsonRejection,
    >,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_condition_update(
                &encounter_ref,
                &participant_key,
                condition_id,
                request,
            )
        })
        .await?,
    ))
}
#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/condition-consequences", operation_id = "pendingEncounterConditionConsequences", params(("encounter_ref" = String, Path)), responses((status = 200, description = "Successful response", body = Vec<atlas_app_model::PendingConditionConsequenceView>), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/condition-consequences/preview", operation_id = "previewEncounterDrainedConsequence", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::ResolveDrainedConsequenceRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::ConditionHpChangeView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/condition-consequences/drained", operation_id = "resolveEncounterDrainedConsequence", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::ResolveDrainedConsequenceRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(patch, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/life-settings", operation_id = "setEncounterLifeSettings", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = atlas_app_model::SetEncounterLifeSettingsRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/persistent-recovery", operation_id = "recoverPersistentDamage", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::RecoverPersistentDamageRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PersistentRecoveryResultView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/hp-lifecycle", operation_id = "pendingEncounterHpLifecycle", params(("encounter_ref" = String, Path)), responses((status = 200, description = "Successful response", body = Vec<atlas_app_model::PendingHpLifecycleView>), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/hp-lifecycle/resolve", operation_id = "resolveEncounterHpLifecycle", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::ResolveEncounterHpLifecycleRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/persistent-preparation", operation_id = "preparePersistentDamage", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::PreparePersistentDamageRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PreparedPersistentDamageView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/persistent-damage", operation_id = "applyPersistentDamage", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::ApplyPersistentDamageRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PersistentDamageResultView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/strike-context", operation_id = "previewEncounterStrikeContext", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::EncounterStrikeContextRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterStrikeContextView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/casting-reminders", operation_id = "encounterCastingReminders", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::EncounterCastingRemindersRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterCastingRemindersView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/persistent-review", operation_id = "persistentDamageReview", params(("encounter_ref" = String, Path), ("participant_key" = Option<String>, Query), ("boundary_key" = Option<String>, Query)), responses((status = 200, description = "Successful response", body = atlas_app_model::PersistentReviewView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/hp-lifecycle/preview", operation_id = "previewEncounterHpLifecycle", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::ResolveEncounterHpLifecycleRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PendingHpLifecycleView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/condition-lifecycle", operation_id = "applyConditionLifecycle", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::ApplyConditionLifecycleRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::ConditionLifecycleResultView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/condition-lifecycle-actions", operation_id = "conditionLifecycleActions", params(("encounter_ref" = String, Path), ("participant_key" = Option<String>, Query), ("boundary_key" = Option<String>, Query)), responses((status = 200, description = "Successful response", body = atlas_app_model::ConditionLifecycleActionsView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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
#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/condition-recoveries", operation_id = "pendingConditionRecoveries", params(("encounter_ref" = String, Path)), responses((status = 200, description = "Successful response", body = Vec<atlas_app_model::PendingConditionRecoveryView>), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(get, path = "/api/lists/{list_ref}/items/{record_key}/note", operation_id = "savedListItemNote", params(("list_ref" = String, Path), ("record_key" = String, Path)), responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListItemNoteView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(patch, path = "/api/lists/{list_ref}/items/{record_key}/note", operation_id = "updateSavedListItemNote", params(("list_ref" = String, Path), ("record_key" = String, Path)), request_body = UpdateSavedListItemNoteRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::SavedListItemNoteView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_saved_list_item_note(
    State(state): State<AtlasWebState>,
    Path((list_ref, record_key)): Path<(String, String)>,
    payload: Result<Json<SavedEditPayload<UpdateSavedListItemNoteRequest>>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(SavedEditPayload(request)) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_saved_list_item_note(&list_ref, &record_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/definitions/detail", operation_id = "definitionDetail", request_body = atlas_app_model::DefinitionDetailRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::DefinitionDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn definition_detail(
    State(state): State<AtlasWebState>,
    payload: Result<Json<atlas_app_model::DefinitionDetailRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_read_service(state, move |execution| {
            service.definition_detail(request, &execution)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/traits/detail", operation_id = "traitDefinition", request_body = atlas_app_model::TraitDetailRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::TraitDetailView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/traits/records", operation_id = "traitRecords", request_body = atlas_app_model::TraitRecordsRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::TraitRecordsPage), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/search/relationship-occurrences", operation_id = "relationshipOccurrences", request_body = atlas_app_model::RelationshipOccurrencesRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::RelationshipOccurrencesPage), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
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

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/manual-trackers", operation_id = "updateEncounterManualTracker", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = atlas_app_model::UpdateEncounterManualTrackerRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_encounter_manual_tracker(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<atlas_app_model::UpdateEncounterManualTrackerRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_encounter_manual_tracker(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/ability-state", operation_id = "updateEncounterAbilityState", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = atlas_app_model::UpdateEncounterAbilityStateRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_encounter_ability_state(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<atlas_app_model::UpdateEncounterAbilityStateRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_encounter_ability_state(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/turn-resources", operation_id = "updateTurnResources", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = atlas_app_model::UpdateTurnResourcesRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_turn_resources(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<atlas_app_model::UpdateTurnResourcesRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.update_turn_resources(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(patch, path = "/api/encounters/{encounter_ref}/difficulty", operation_id = "setEncounterDifficulty", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::SetEncounterDifficultyRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn set_encounter_difficulty(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::SetEncounterDifficultyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.set_encounter_difficulty(&encounter_ref, request)
        })
        .await?,
    ))
}
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/difficulty/preview", operation_id = "previewEncounterDifficulty", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::PreviewEncounterDifficultyRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterDifficultyView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn preview_encounter_difficulty(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::PreviewEncounterDifficultyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_difficulty(&encounter_ref, request)
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/recovery/preview", operation_id = "previewEncounterRecovery", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::PreviewEncounterRecoveryRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterRecoveryPreviewView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn preview_encounter_recovery(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::PreviewEncounterRecoveryRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.preview_encounter_recovery(&encounter_ref, request)
        })
        .await?,
    ))
}
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/recovery", operation_id = "applyEncounterRecovery", params(("encounter_ref" = String, Path)), request_body = atlas_app_model::ApplyEncounterRecoveryRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn apply_encounter_recovery(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<atlas_app_model::ApplyEncounterRecoveryRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.apply_encounter_recovery(&encounter_ref, request)
        })
        .await?,
    ))
}
