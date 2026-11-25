use crate::error::Path;
use crate::{
    error::WebError,
    service::{AtlasWebState, call_service},
};
use atlas_app_model::PathbuilderPreviewRequest;
use atlas_app_model::SavePathbuilderPcRequest;
use atlas_app_model::SavePcAbilityLinkRequest;
use atlas_app_model::SetEncounterPcReferenceRequest;
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    response::IntoResponse,
};

#[utoipa::path(post, path = "/api/characters/pathbuilder/preview", operation_id = "previewPathbuilderPc", request_body = PathbuilderPreviewRequest,
    responses((status = 200, description = "Successful response", body = atlas_app_model::PathbuilderPreviewView),
    (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse),
    (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn preview_pathbuilder_pc(
    State(state): State<AtlasWebState>,
    payload: Result<Json<PathbuilderPreviewRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| {
            service.preview_pathbuilder_pc(&request.json, request.character_key.as_deref())
        })
        .await?,
    ))
}

#[utoipa::path(post, path = "/api/characters/pathbuilder", operation_id = "savePathbuilderPc", request_body = SavePathbuilderPcRequest,
    responses((status = 200, description = "Successful response", body = atlas_app_model::CharacterMutationView),
    (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse),
    (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn save_pathbuilder_pc(
    State(state): State<AtlasWebState>,
    payload: Result<Json<SavePathbuilderPcRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| service.save_pathbuilder_pc(request)).await?,
    ))
}
#[utoipa::path(patch, path = "/api/characters/{character_key}/reference/{ability_key}", operation_id = "savePcAbilityLink", params(("character_key" = String, Path), ("ability_key" = String, Path)), request_body = SavePcAbilityLinkRequest,
    responses((status = 200, description = "Successful response", body = atlas_app_model::PcReferenceView),
    (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse),
    (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn save_pc_ability_link(
    State(state): State<AtlasWebState>,
    Path((character_key, ability_key)): Path<(String, String)>,
    payload: Result<Json<SavePcAbilityLinkRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| {
            service.save_pc_ability_link(&character_key, &ability_key, request)
        })
        .await?,
    ))
}

#[utoipa::path(get, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/pc-reference", operation_id = "encounterPcReference", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)),
    responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterPcReferenceView),
    (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse),
    (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn encounter_pc_reference(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
) -> Result<impl IntoResponse, WebError> {
    Ok(Json(
        call_service(state, move |service| {
            service.encounter_pc_reference(&encounter_ref, &participant_key)
        })
        .await?,
    ))
}

#[utoipa::path(patch, path = "/api/encounters/{encounter_ref}/participants/{participant_key}/pc-reference", operation_id = "setEncounterPcReference", params(("encounter_ref" = String, Path), ("participant_key" = String, Path)), request_body = SetEncounterPcReferenceRequest,
    responses((status = 200, description = "Successful response", body = atlas_app_model::EncounterMutationView),
    (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse),
    (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn set_encounter_pc_reference(
    State(state): State<AtlasWebState>,
    Path((encounter_ref, participant_key)): Path<(String, String)>,
    payload: Result<Json<SetEncounterPcReferenceRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    Ok(Json(
        call_service(state, move |service| {
            service.set_encounter_pc_reference(&encounter_ref, &participant_key, request)
        })
        .await?,
    ))
}
