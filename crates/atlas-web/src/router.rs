use atlas_app_service::AtlasAppService;
use axum::Router;
#[cfg(feature = "embedded-ui")]
use axum::routing::get;

#[cfg(feature = "embedded-ui")]
use crate::assets::{root, static_asset};
use crate::service::AtlasWebState;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(OpenApi)]
#[openapi(components(schemas(
    atlas_app_model::CastingEntryContextView,
    atlas_app_model::StaticSearchContext,
    atlas_domain::QueryExpression
)))]
struct OpenApiComponents;

fn documented_routes() -> OpenApiRouter<AtlasWebState> {
    OpenApiRouter::with_openapi(OpenApiComponents::openapi())
        .routes(routes!(crate::compatibility::handshake))
        .routes(routes!(crate::handlers::definition_detail))
        .routes(routes!(
            crate::handlers::encounters,
            crate::handlers::create_encounter
        ))
        .routes(routes!(crate::handlers::encounter_condition_definitions))
        .routes(routes!(
            crate::handlers::delete_encounter,
            crate::handlers::encounter,
            crate::handlers::update_encounter
        ))
        .routes(routes!(crate::handlers::advance_encounter_turn))
        .routes(routes!(crate::handlers::encounter_casting_reminders))
        .routes(routes!(
            crate::handlers::pending_encounter_condition_consequences
        ))
        .routes(routes!(
            crate::handlers::resolve_encounter_drained_consequence
        ))
        .routes(routes!(
            crate::handlers::preview_encounter_drained_consequence
        ))
        .routes(routes!(crate::handlers::apply_condition_lifecycle))
        .routes(routes!(crate::handlers::condition_lifecycle_actions))
        .routes(routes!(crate::handlers::pending_condition_recoveries))
        .routes(routes!(crate::handlers::set_encounter_difficulty))
        .routes(routes!(crate::handlers::preview_encounter_difficulty))
        .routes(routes!(crate::handlers::pending_encounter_hp_lifecycle))
        .routes(routes!(crate::handlers::preview_encounter_hp_lifecycle))
        .routes(routes!(crate::handlers::resolve_encounter_hp_lifecycle))
        .routes(routes!(
            crate::handlers::initiative_options,
            crate::handlers::roll_initiative
        ))
        .routes(routes!(crate::handlers::add_encounter_manual_participant))
        .routes(routes!(crate::parties::add_encounter_party))
        .routes(routes!(crate::handlers::add_encounter_record_participant))
        .routes(routes!(crate::handlers::reorder_encounter_participant))
        .routes(routes!(
            crate::handlers::remove_encounter_participant,
            crate::handlers::update_encounter_participant
        ))
        .routes(routes!(crate::handlers::update_encounter_ability_state))
        .routes(routes!(
            crate::handlers::participant_encounter_condition_definitions
        ))
        .routes(routes!(
            crate::handlers::add_encounter_participant_condition
        ))
        .routes(routes!(crate::handlers::preview_encounter_condition_add))
        .routes(routes!(
            crate::handlers::remove_encounter_participant_condition,
            crate::handlers::update_encounter_participant_condition
        ))
        .routes(routes!(crate::handlers::preview_encounter_condition_update))
        .routes(routes!(crate::handlers::apply_encounter_hp_action))
        .routes(routes!(crate::handlers::set_encounter_life_settings))
        .routes(routes!(crate::handlers::update_encounter_manual_tracker))
        .routes(routes!(crate::handlers::encounter_reference))
        .routes(routes!(crate::handlers::apply_encounter_spell_action))
        .routes(routes!(crate::handlers::update_turn_resources))
        .routes(routes!(crate::handlers::apply_persistent_damage))
        .routes(routes!(crate::handlers::prepare_persistent_damage))
        .routes(routes!(crate::handlers::recover_persistent_damage))
        .routes(routes!(crate::handlers::persistent_damage_review))
        .routes(routes!(crate::handlers::apply_encounter_recovery))
        .routes(routes!(crate::handlers::preview_encounter_recovery))
        .routes(routes!(crate::handlers::preview_encounter_strike_context))
        .routes(routes!(crate::handlers::encounter_strike_options))
        .routes(routes!(crate::handlers::roll_encounter_strike))
        .routes(routes!(crate::handlers::set_encounter_turn))
        .routes(routes!(crate::handlers::encounter_turn_review))
        .routes(routes!(crate::handlers::discover_filter_counts))
        .routes(routes!(crate::handlers::discover_filter_editor))
        .routes(routes!(crate::handlers::validate_filter))
        .routes(routes!(crate::handlers::discover_filter_values))
        .routes(routes!(
            crate::handlers::saved_lists,
            crate::handlers::create_saved_list
        ))
        .routes(routes!(
            crate::handlers::delete_saved_list,
            crate::handlers::saved_list,
            crate::handlers::update_saved_list
        ))
        .routes(routes!(crate::handlers::filter_saved_list))
        .routes(routes!(
            crate::handlers::remove_saved_list_item,
            crate::handlers::add_saved_list_item
        ))
        .routes(routes!(crate::handlers::reorder_saved_list_item))
        .routes(routes!(
            crate::handlers::saved_list_item_note,
            crate::handlers::update_saved_list_item_note
        ))
        .routes(routes!(
            crate::parties::parties,
            crate::parties::create_party
        ))
        .routes(routes!(
            crate::parties::delete_party,
            crate::parties::update_party
        ))
        .routes(routes!(crate::handlers::readiness))
        .routes(routes!(crate::handlers::record_detail_at))
        .routes(routes!(crate::handlers::embedded_reading))
        .routes(routes!(crate::handlers::record_detail))
        .routes(routes!(crate::handlers::related_records))
        .routes(routes!(crate::handlers::roll_content))
        .routes(routes!(crate::handlers::roll_flat_check))
        .routes(routes!(crate::handlers::roll_table))
        .routes(routes!(crate::handlers::search_page))
        .routes(routes!(crate::handlers::relationship_occurrences))
        .routes(routes!(crate::handlers::similarity_parts))
        .routes(routes!(crate::handlers::trait_definition))
        .routes(routes!(crate::handlers::trait_records))
}

// Utoipa applies Serde's deny_unknown_fields to structs but leaves tagged
// enum branches open. QueryExpression is closed at its flattened wrapper.
pub(crate) const CLOSED_TAGGED_SCHEMAS: &[&str] = &[
    "DiscoverFilterValuesRequest",
    "EmbeddedReadingSelection",
    "EncounterAbilityStateAction",
    "EncounterHpAction",
    "EncounterManualTrackerAction",
    "EncounterSpellAction",
    "FilterDiscoveryContext",
    "FilterVocabularyScope",
    "ReferenceDestinationView",
    "ResultSort",
    "SearchCriteria",
    "SimilarityComparisonKey",
    "SimilarityComponentView",
    "SimilarityOptions",
    "SimilaritySeed",
    "SourcePassageAddress",
    "StrikeRollAction",
];

pub fn openapi() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use utoipa::openapi::{
        RefOr,
        schema::{AdditionalProperties, Schema},
    };

    let mut document = documented_routes().into_openapi();
    let schemas = &mut document
        .components
        .as_mut()
        .ok_or("documented routes must supply schemas")?
        .schemas;
    for name in CLOSED_TAGGED_SCHEMAS {
        let Some(RefOr::T(Schema::OneOf(union))) = schemas.get_mut(*name) else {
            return Err(format!("{name} must remain a tagged union").into());
        };
        for variant in &mut union.items {
            match variant {
                RefOr::T(Schema::Object(object)) => {
                    object.additional_properties =
                        Some(Box::new(AdditionalProperties::FreeForm(false)));
                }
                RefOr::T(Schema::AllOf(composition)) => {
                    composition
                        .extensions
                        .get_or_insert_default()
                        .insert("unevaluatedProperties".into(), false.into());
                }
                _ => return Err(format!("{name} variants must remain objects").into()),
            }
        }
    }

    // QueryPredicate flattens a tagged expression alongside clause_id. Closing
    // each expression branch would reject the valid clause_id field.
    let Some(RefOr::T(Schema::AllOf(predicate))) = schemas.get_mut("QueryPredicate") else {
        return Err("QueryPredicate must remain a flattened object".into());
    };
    predicate
        .extensions
        .get_or_insert_default()
        .insert("unevaluatedProperties".into(), false.into());
    let mut document = serde_json::to_value(document)?;
    crate::schema_directions::describe_accepted_inputs(&mut document)?;
    Ok(document)
}

pub fn router(service: AtlasAppService) -> Router {
    router_with_state(AtlasWebState::new(service))
}

pub(crate) fn router_with_state(state: AtlasWebState) -> Router {
    let (router, _) = documented_routes().split_for_parts();
    #[cfg(feature = "embedded-ui")]
    let router = router.route("/", get(root)).fallback(get(static_asset));
    router
        .layer(axum::middleware::from_fn(crate::compatibility::check))
        .with_state(state)
}
