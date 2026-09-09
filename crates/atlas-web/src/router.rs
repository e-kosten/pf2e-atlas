use atlas_app_service::AtlasAppService;
use axum::Router;
use axum::routing::{get, post};

use crate::assets::{root, static_asset};
use crate::handlers::{
    add_encounter_manual_participant, add_encounter_participant_condition,
    add_encounter_record_participant, add_saved_list_item, create_encounter, create_saved_list,
    delete_encounter, delete_saved_list, discover_filter_editor, discover_filter_values, encounter,
    encounter_condition_definitions, encounters, filter_saved_list, mutate_encounter_spell_cast,
    open_result_window, read_result_window_page, readiness, record_detail,
    remove_encounter_participant, remove_encounter_participant_condition, remove_saved_list_item,
    reorder_encounter_participant, reset_encounter_participant, roll_table, saved_list,
    saved_lists, set_encounter_turn, update_encounter, update_encounter_participant,
    update_encounter_participant_condition, update_saved_list,
};
use crate::service::AtlasWebState;

pub fn router(service: AtlasAppService) -> Router {
    router_with_state(AtlasWebState::new(service))
}

pub(crate) fn router_with_state(state: AtlasWebState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api/readiness", get(readiness))
        .route("/api/encounters", get(encounters).post(create_encounter))
        .route(
            "/api/encounters/condition-definitions",
            get(encounter_condition_definitions),
        )
        .route(
            "/api/encounters/{encounter_ref}",
            get(encounter).patch(update_encounter).delete(delete_encounter),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/record",
            post(add_encounter_record_participant),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/manual",
            post(add_encounter_manual_participant),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/reorder",
            post(reorder_encounter_participant),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/{participant_key}",
            axum::routing::patch(update_encounter_participant).delete(remove_encounter_participant),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/{participant_key}/conditions",
            post(add_encounter_participant_condition),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/{participant_key}/conditions/{condition_id}",
            axum::routing::patch(update_encounter_participant_condition)
                .delete(remove_encounter_participant_condition),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/{participant_key}/spell-casts",
            post(mutate_encounter_spell_cast),
        )
        .route(
            "/api/encounters/{encounter_ref}/participants/{participant_key}/reset",
            post(reset_encounter_participant),
        )
        .route(
            "/api/encounters/{encounter_ref}/turn",
            post(set_encounter_turn),
        )
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
        .route("/api/records/{record_key}/table-roll", post(roll_table))
        .fallback(get(static_asset))
        .with_state(state)
}
