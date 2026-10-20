use crate::{
    error::WebError,
    service::{AtlasWebState, call_service},
};
use atlas_app_model::{
    AddEncounterPartyRequest, CreatePartyRequest, DeletePartyRequest, UpdatePartyRequest,
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    response::IntoResponse,
};

pub(crate) async fn parties(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(call_service(state, move || service.parties()).await?))
}
pub(crate) async fn create_party(
    State(state): State<AtlasWebState>,
    payload: Result<Json<CreatePartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.create_party(request)).await?,
    ))
}
pub(crate) async fn update_party(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<UpdatePartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.update_party(&key, request)).await?,
    ))
}
pub(crate) async fn delete_party(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<DeletePartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.delete_party(&key, request)).await?,
    ))
}
pub(crate) async fn add_encounter_party(
    State(state): State<AtlasWebState>,
    Path(encounter_ref): Path<String>,
    payload: Result<Json<AddEncounterPartyRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_party(&encounter_ref, request)
        })
        .await?,
    ))
}
