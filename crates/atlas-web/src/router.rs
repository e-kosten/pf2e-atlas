use atlas_app_service::AtlasAppService;
use axum::Router;
use axum::routing::{get, post};

use crate::assets::{root, static_asset};
use crate::handlers::{
    add_saved_list_item, create_saved_list, delete_saved_list, discover_filter_editor,
    discover_filter_values, filter_saved_list, open_result_window, read_result_window_page,
    readiness, record_detail, remove_saved_list_item, saved_list, saved_lists, update_saved_list,
};
use crate::service::AtlasWebState;

pub fn router(service: AtlasAppService) -> Router {
    router_with_state(AtlasWebState::new(service))
}

pub(crate) fn router_with_state(state: AtlasWebState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api/readiness", get(readiness))
        .route("/api/lists", get(saved_lists).post(create_saved_list))
        .route(
            "/api/lists/{list_ref}",
            get(saved_list)
                .patch(update_saved_list)
                .delete(delete_saved_list),
        )
        .route("/api/lists/{list_ref}/filter", post(filter_saved_list))
        .route(
            "/api/lists/{list_ref}/items",
            post(add_saved_list_item).delete(remove_saved_list_item),
        )
        .route("/api/filters/editor", post(discover_filter_editor))
        .route("/api/filters/values", post(discover_filter_values))
        .route("/api/result-windows", post(open_result_window))
        .route(
            "/api/result-windows/{window_id}/page",
            post(read_result_window_page),
        )
        .route("/api/records/{record_key}", get(record_detail))
        .fallback(get(static_asset))
        .with_state(state)
}
