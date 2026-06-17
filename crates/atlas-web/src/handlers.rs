use atlas_app_model::{
    AddSavedListItemRequest, CreateSavedListRequest, DiscoverFilterEditorRequest,
    DiscoverFilterValuesRequest, OpenResultWindowRequest, ReadResultWindowPageRequest,
    RemoveSavedListItemRequest, UpdateSavedListRequest,
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
