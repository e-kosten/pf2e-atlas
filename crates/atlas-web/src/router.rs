use atlas_app_service::AtlasAppService;
use axum::Router;
use axum::routing::{get, post};

use crate::assets::{root, static_asset};
use crate::handlers::{
    add_encounter_manual_participant, add_encounter_participant_condition,
    add_encounter_record_participant, add_saved_list_item, advance_encounter_turn,
    create_encounter, create_saved_list, delete_encounter, delete_saved_list,
    discover_filter_editor, discover_filter_values, encounter, encounter_condition_definitions,
    encounter_turn_review, encounters, filter_saved_list, readiness, record_detail,
    remove_encounter_participant, remove_encounter_participant_condition, remove_saved_list_item,
    reorder_encounter_participant, reorder_saved_list_item, saved_list, saved_list_item_note,
    saved_lists, search_page, set_encounter_turn, update_encounter, update_encounter_participant,
    update_encounter_participant_condition, update_saved_list, update_saved_list_item_note,
};
use crate::service::AtlasWebState;

pub fn router(service: AtlasAppService) -> Router {
    router_with_state(AtlasWebState::new(service))
}

pub(crate) fn router_with_state(state: AtlasWebState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api/readiness", get(readiness))
        .route("/api/parties", get(crate::parties::parties).post(crate::parties::create_party))
        .route("/api/parties/{party_key}", axum::routing::patch(crate::parties::update_party).delete(crate::parties::delete_party))
        .route("/api/encounters/{encounter_ref}/participants/party", post(crate::parties::add_encounter_party))
        .route("/api/traits/detail", post(crate::handlers::trait_definition))
        .route("/api/traits/records", post(crate::handlers::trait_records))
        .route("/api/related",post(crate::handlers::related_records))
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
            "/api/encounters/{encounter_ref}/participants/{participant_key}/reference",
            post(crate::handlers::encounter_reference),
        )
        .route("/api/encounters/{encounter_ref}/participants/{participant_key}/spells", post(crate::handlers::apply_encounter_spell_action))
        .route("/api/encounters/{encounter_ref}/initiative", get(crate::handlers::initiative_options).post(crate::handlers::roll_initiative))
        .route(
            "/api/encounters/{encounter_ref}/participants/{participant_key}/hp",
            post(crate::handlers::apply_encounter_hp_action),
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
        .route("/api/encounters/{encounter_ref}/turn-review", get(encounter_turn_review))
        .route("/api/encounters/{encounter_ref}/advance", post(advance_encounter_turn))
        .route(
            "/api/encounters/{encounter_ref}/turn",
            post(set_encounter_turn),
        )
        .route("/api/lists/{list_ref}/items/{record_key}/note", get(saved_list_item_note).patch(update_saved_list_item_note))
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
        .route("/api/lists/{list_ref}/items/reorder", post(reorder_saved_list_item))
        .route("/api/filters/editor", post(discover_filter_editor))
        .route("/api/filters/values", post(discover_filter_values))
        .route("/api/filters/counts", post(crate::handlers::discover_filter_counts))
        .route("/api/filters/validate", post(crate::handlers::validate_filter))
        .route("/api/search/page", post(search_page))
        .route("/api/records/{record_key}", get(record_detail))
        .route("/api/encounters/{encounter_ref}/strike-options", post(crate::handlers::encounter_strike_options))
        .route("/api/encounters/{encounter_ref}/strike-rolls", post(crate::handlers::roll_encounter_strike))
        .route("/api/rolls/table", post(crate::handlers::roll_table))
        .route("/api/records/detail", post(crate::handlers::record_detail_at))
        .route("/api/records/embedded-reading", post(crate::handlers::embedded_reading))
        .fallback(get(static_asset))
        .with_state(state)
}
