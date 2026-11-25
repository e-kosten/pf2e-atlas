use crate::error::Path;
use crate::{
    error::WebError,
    service::{AtlasWebState, call_service},
};
use atlas_app_model::{
    AddEncounterPartyRequest, CreatePartyRequest, DeletePartyRequest, UpdatePartyRequest,
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    response::IntoResponse,
};

#[utoipa::path(get, path = "/api/parties", operation_id = "parties", responses((status = 200, description = "Successful response", body = atlas_app_model::PartyDirectoryView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn parties(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    Ok(Json(
        call_service(state, move |service| service.party_directory()).await?,
    ))
}
#[utoipa::path(get, path = "/api/parties/rosters", operation_id = "partyRosters", responses((status = 200, description = "Successful response", body = atlas_app_model::PartyIndexView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn party_rosters(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    Ok(Json(
        call_service(state, |service| service.parties()).await?,
    ))
}
#[utoipa::path(get, path = "/api/parties/{party_key}", operation_id = "partyRoster", params(("party_key" = String, Path, description = "Stable party key")), responses((status = 200, description = "Successful response", body = atlas_app_model::PartyView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn party_roster(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(Json(
        call_service(state, move |service| service.party_roster(&key)).await?,
    ))
}
#[utoipa::path(patch, path = "/api/parties/{party_key}/name", operation_id = "renameParty", params(("party_key" = String, Path, description = "Stable party key")), request_body = atlas_app_model::RenamePartyRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PartyView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn rename_party(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<atlas_app_model::RenamePartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| {
            service.edit_party_name(&key, request.expected_revision, request.name)
        })
        .await?,
    ))
}
#[utoipa::path(post, path = "/api/parties", operation_id = "createParty", request_body = CreatePartyRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PartyView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn create_party(
    State(state): State<AtlasWebState>,
    payload: Result<Json<CreatePartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| service.create_party(request)).await?,
    ))
}
#[utoipa::path(patch, path = "/api/parties/{party_key}", operation_id = "updateParty", params(("party_key" = String, Path)), request_body = UpdatePartyRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PartyView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn update_party(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<UpdatePartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| service.update_party(&key, request)).await?,
    ))
}
#[utoipa::path(delete, path = "/api/parties/{party_key}", operation_id = "deleteParty", params(("party_key" = String, Path)), request_body = DeletePartyRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::PartyMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn delete_party(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<DeletePartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| service.delete_party(&key, request)).await?,
    ))
}
#[utoipa::path(post, path = "/api/encounters/{encounter_ref}/participants/party", operation_id = "addEncounterParty", params(("encounter_ref" = String, Path)), request_body = AddEncounterPartyRequest, responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn add_encounter_party(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<AddEncounterPartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| {
            service.add_encounter_party(&encounter_ref, request)
        })
        .await?,
    ))
}
