use crate::{
    error::WebError,
    service::{AtlasWebState, call_service},
};
use atlas_app_model::*;
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    response::IntoResponse,
};
#[utoipa::path(get, path="/api/characters",operation_id="characters",responses((status=200,description="Successful response",body=CharacterIndexView),(status=409,description="Application or browser contract conflict",body=crate::error::ConflictResponse),(status="default",description="Atlas error",body=AppError)))]
pub(crate) async fn characters(
    State(state): State<AtlasWebState>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.characters()).await?,
    ))
}
#[utoipa::path(post, path="/api/characters",operation_id="createCharacter",request_body=CreateCharacterRequest,responses((status=200,description="Successful response",body=CharacterMutationView),(status=409,description="Application or browser contract conflict",body=crate::error::ConflictResponse),(status="default",description="Atlas error",body=AppError)))]
pub(crate) async fn create_character(
    State(state): State<AtlasWebState>,
    payload: Result<Json<CreateCharacterRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.create_character(request)).await?,
    ))
}
#[utoipa::path(get, path="/api/characters/{character_key}",operation_id="character",params(("character_key"=String,Path)), responses((status=200,description="Successful response",body=PcReferenceView),(status=409,description="Application or browser contract conflict",body=crate::error::ConflictResponse),(status="default",description="Atlas error",body=AppError)))]
pub(crate) async fn character(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.character(&key)).await?,
    ))
}
#[utoipa::path(patch, path="/api/characters/{character_key}",operation_id="updateCharacter",params(("character_key"=String,Path)), request_body=UpdateCharacterRequest,responses((status=200,description="Successful response",body=CharacterMutationView),(status=409,description="Application or browser contract conflict",body=crate::error::ConflictResponse),(status="default",description="Atlas error",body=AppError)))]
pub(crate) async fn update_character(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<UpdateCharacterRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.update_character(&key, request)).await?,
    ))
}
#[utoipa::path(delete, path="/api/characters/{character_key}",operation_id="deleteCharacter",params(("character_key"=String,Path)), request_body=DeleteCharacterRequest,responses((status=200,description="Successful response",body=CharacterMutationView),(status=409,description="Application or browser contract conflict",body=crate::error::ConflictResponse),(status="default",description="Atlas error",body=AppError)))]
pub(crate) async fn delete_character(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<DeleteCharacterRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.delete_character(&key, request)).await?,
    ))
}
#[utoipa::path(post, path="/api/encounters/{encounter_ref}/participants/character",operation_id="addEncounterCharacter",params(("encounter_ref"=String,Path)), request_body=AddEncounterCharacterRequest,responses((status=200,description="Successful response",body=EncounterCharacterMutationView),(status=409,description="Application or browser contract conflict",body=crate::error::ConflictResponse),(status="default",description="Atlas error",body=AppError)))]
pub(crate) async fn add_encounter_character(
    State(state): State<AtlasWebState>,
    Path(key): Path<String>,
    payload: Result<Json<AddEncounterCharacterRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || {
            service.add_encounter_character(&key, request)
        })
        .await?,
    ))
}
