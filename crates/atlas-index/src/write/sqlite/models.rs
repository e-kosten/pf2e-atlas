use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::artifact_metadata)]
pub(super) struct ArtifactMetadataRow {
    pub key: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::packs)]
pub(super) struct PackRow {
    pub name: String,
    pub label: String,
    pub document_type: String,
    pub declared_path: String,
    pub resolved_path: String,
    pub record_count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::records)]
pub(super) struct RecordRow {
    pub record_key: String,
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub record_kind: String,
    pub pack_name: String,
    pub pack_label: String,
    pub foundry_document_type: String,
    pub foundry_record_type: String,
    pub level: Option<i64>,
    pub rarity: Option<String>,
    pub traits_json: String,
    pub prerequisites_json: String,
    pub system_category: Option<String>,
    pub system_group: Option<String>,
    pub system_base_item: Option<String>,
    pub system_usage: Option<String>,
    pub system_price_json: Option<String>,
    pub system_actions_value: Option<i64>,
    pub system_time_value: Option<String>,
    pub system_duration_value: Option<String>,
    pub price_cp: Option<i64>,
    pub activation_time_kind: Option<String>,
    pub activation_time_actions: Option<i64>,
    pub activation_time_duration_value: Option<i64>,
    pub activation_time_duration_unit: Option<String>,
    pub activation_time_text: Option<String>,
    pub duration_kind: Option<String>,
    pub duration_value: Option<i64>,
    pub duration_unit: Option<String>,
    pub duration_text: Option<String>,
    pub publication_title: Option<String>,
    pub publication_remaster: bool,
    pub publication_family: String,
    pub folder_id: Option<String>,
    pub taxonomy_families_json: String,
    pub variant_group_key: Option<String>,
    pub variant_base_name: Option<String>,
    pub variant_label: Option<String>,
    pub variant_axes_json: String,
    pub variant_confidence: Option<f64>,
    pub variant_source: String,
    pub source_path: String,
    pub is_default_visible: bool,
    pub visibility_state: String,
    pub visibility_reason: String,
    pub metric_count: i64,
    pub metric_order_sha256: String,
    pub raw_json: String,
    pub record_role: String,
    pub retrieval_disposition: String,
    pub retrieval_rationale: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::record_content)]
pub(super) struct RecordContentRow {
    pub record_key: String,
    pub content_key: String,
    pub authored_order: i64,
    pub identity_stability: String,
    pub owner_kind: String,
    pub owner_record_key: Option<String>,
    pub owner_entity_id: Option<String>,
    pub owner_occurrence_id: Option<String>,
    pub owner_occurrence_authored_order: Option<i64>,
    pub owner_hazard_entity_id: Option<String>,
    pub owner_hazard_occurrence_id: Option<String>,
    pub owner_hazard_occurrence_authored_order: Option<i64>,
    pub owner_consumable_occurrence_id: Option<String>,
    pub owner_consumable_occurrence_authored_order: Option<i64>,
    pub role: String,
    pub origin_json: String,
    pub visibility: String,
    pub provenance_json: String,
    pub source_kind: String,
    pub contributes_to_search: bool,
    pub contributes_to_references: bool,
    pub label: Option<String>,
    pub content_json: String,
    pub content_hash: String,
    pub duplicate_status_json: String,
    pub diagnostics_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::record_content_exclusions)]
pub(super) struct RecordContentExclusionRow {
    pub record_key: String,
    pub content_key: String,
    pub relative_source_path: String,
    pub label: Option<String>,
    pub reason: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_creature_records)]
pub(super) struct CanonicalCreatureRecordRow {
    pub record_key: String,
    pub source_id: String,
    pub name: String,
    pub family: String,
    pub canonical_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_creature_resources)]
pub(super) struct CanonicalCreatureResourceRow {
    pub record_key: String,
    pub resource_id: String,
    pub authored_order: i64,
    pub resource_kind: String,
    pub resource_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_creature_entities)]
pub(super) struct CanonicalCreatureEntityRow {
    pub record_key: String,
    pub entity_id: String,
    pub family: String,
    pub label: String,
    pub source_identity_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_creature_occurrences)]
pub(super) struct CanonicalCreatureOccurrenceRow {
    pub record_key: String,
    pub occurrence_id: String,
    pub identity_stability: String,
    pub family: String,
    pub authored_order: i64,
    pub source_sort_json: String,
    pub source_folder_json: String,
    pub source_identity_json: String,
    pub parent_kind: String,
    pub parent_occurrence_id: Option<String>,
    pub parent_occurrence_authored_order: Option<i64>,
    pub target_kind: String,
    pub target_record_key: Option<String>,
    pub target_entity_id: Option<String>,
    pub context_json: String,
    pub capability_json: String,
    pub deltas_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_creature_relationships)]
pub(super) struct CanonicalCreatureRelationshipRow {
    pub record_key: String,
    pub relationship_order: i64,
    pub source_occurrence_id: String,
    pub source_occurrence_authored_order: i64,
    pub relationship_kind: String,
    pub target_kind: String,
    pub target_occurrence_id: Option<String>,
    pub target_occurrence_authored_order: Option<i64>,
    pub target_source_id: Option<String>,
    pub source_path: String,
    pub contextual_label_json: String,
    pub lifecycle_json: String,
    pub execution: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_hazard_records)]
pub(super) struct CanonicalHazardRecordRow {
    pub record_key: String,
    pub source_id: String,
    pub name: String,
    pub family: String,
    pub canonical_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_spell_records)]
pub(super) struct CanonicalSpellRecordRow {
    pub record_key: String,
    pub source_id: String,
    pub name: String,
    pub canonical_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_hazard_entities)]
pub(super) struct CanonicalHazardEntityRow {
    pub record_key: String,
    pub entity_id: String,
    pub family: String,
    pub label: String,
    pub image_json: String,
    pub source_identity_json: String,
    pub capability_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_hazard_occurrences)]
pub(super) struct CanonicalHazardOccurrenceRow {
    pub record_key: String,
    pub occurrence_id: String,
    pub entity_id: String,
    pub identity_stability: String,
    pub family: String,
    pub authored_order: i64,
    pub source_sort_json: String,
    pub source_folder_json: String,
    pub source_ordinal: i64,
    pub contextual_label_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_hazard_relationships)]
pub(super) struct CanonicalHazardRelationshipRow {
    pub record_key: String,
    pub relationship_id: String,
    pub authored_order: i64,
    pub source_occurrence_id: Option<String>,
    pub source_occurrence_authored_order: Option<i64>,
    pub relationship_kind: String,
    pub target_kind: String,
    pub target_entity_id: Option<String>,
    pub target_occurrence_id: Option<String>,
    pub target_occurrence_authored_order: Option<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_consumable_spell_children)]
pub(super) struct CanonicalConsumableSpellChildRow {
    pub parent_record_key: String,
    pub child_id: String,
    pub authored_order: i64,
    pub standalone_target_record_key: Option<String>,
    pub canonical_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_consumable_records)]
pub(super) struct CanonicalConsumableRecordRow {
    pub record_key: String,
    pub source_id: String,
    pub name: String,
    pub canonical_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_consumable_entities)]
pub(super) struct CanonicalConsumableEntityRow {
    pub owner_record_key: String,
    pub entity_id: String,
    pub target_record_key: Option<String>,
    pub canonical_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::canonical_consumable_occurrences)]
pub(super) struct CanonicalConsumableOccurrenceRow {
    pub owner_record_key: String,
    pub occurrence_id: String,
    pub entity_id: String,
    pub authored_order: i64,
    pub canonical_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::consumable_query_records)]
pub(super) struct ConsumableQueryRecordRow {
    pub record_key: String,
    pub category: Option<String>,
    pub usage: Option<String>,
    pub base_item: Option<String>,
    pub bulk_value: Option<f64>,
    pub hands_requirement: Option<String>,
    pub price_cp: Option<i64>,
    pub damage_types_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::spell_traditions)]
pub(super) struct SpellTraditionRow {
    pub record_key: String,
    pub authored_order: i64,
    pub tradition: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::spell_damage_types)]
pub(super) struct SpellDamageTypeRow {
    pub record_key: String,
    pub damage_key: String,
    pub damage_authored_order: i64,
    pub type_authored_order: i64,
    pub damage_type: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::record_traits)]
pub(super) struct RecordTraitRow {
    pub record_key: String,
    #[diesel(column_name = trait_)]
    pub trait_value: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::actor_records)]
pub(super) struct ActorRecordRow {
    pub record_key: String,
    pub size: Option<String>,
    pub languages_json: String,
    pub speed_types_json: String,
    pub senses_json: String,
    pub immunities_json: String,
    pub resistances_json: String,
    pub weaknesses_json: String,
    pub disable_text: Option<String>,
    pub disable_skills_json: String,
    pub is_complex: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::item_records)]
pub(super) struct ItemRecordRow {
    pub record_key: String,
    pub system_category: Option<String>,
    pub system_base_item: Option<String>,
    pub system_group: Option<String>,
    pub system_usage: Option<String>,
    pub system_price_json: Option<String>,
    pub price_cp: Option<i64>,
    pub bulk_value: Option<f64>,
    pub hands_requirement: Option<String>,
    pub damage_types_json: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::spell_records)]
pub(super) struct SpellRecordRow {
    pub record_key: String,
    pub traditions_json: String,
    pub spell_kinds_json: String,
    pub range_text: Option<String>,
    pub range_value: Option<f64>,
    pub target_text: Option<String>,
    pub area_type: Option<String>,
    pub area_value: Option<f64>,
    pub save_type: Option<String>,
    pub sustained: bool,
    pub basic_save: Option<bool>,
    pub damage_types_json: String,
    pub rank: Option<i64>,
    pub range_kind: Option<String>,
    pub range_rule: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::record_metrics)]
pub(super) struct RecordMetricRow {
    pub record_key: String,
    pub ordinal: i64,
    pub metric_domain: String,
    pub metric_key: String,
    pub value_type: String,
    pub number_value: Option<f64>,
    pub text_value: Option<String>,
    pub bool_value: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::records_fts)]
pub(super) struct RecordsFtsRow {
    pub record_key: String,
    pub title: Option<String>,
    pub aliases: Option<String>,
    pub traits: Option<String>,
    pub taxonomy_terms: Option<String>,
    pub constraint_terms: Option<String>,
    pub mechanic_terms: Option<String>,
    pub source_terms: Option<String>,
    pub metric_terms: Option<String>,
    pub headings: Option<String>,
    pub body: Option<String>,
    pub facts: Option<String>,
    pub reference_terms: Option<String>,
    pub embedded_content: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::reference_edges)]
pub(super) struct ReferenceEdgeRow {
    pub from_record_key: String,
    pub to_record_key: String,
    pub display_text: Option<String>,
    pub reference_text: String,
    pub relation_kind: String,
    pub source_kind: String,
    pub visibility: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::reference_occurrences)]
pub(super) struct ReferenceOccurrenceRow {
    pub record_key: String,
    pub content_key: String,
    pub content_authored_order: i64,
    pub occurrence_ordinal: i64,
    pub owner_kind: String,
    pub owner_record_key: Option<String>,
    pub owner_entity_id: Option<String>,
    pub owner_occurrence_id: Option<String>,
    pub owner_occurrence_authored_order: Option<i64>,
    pub owner_hazard_entity_id: Option<String>,
    pub owner_hazard_occurrence_id: Option<String>,
    pub owner_hazard_occurrence_authored_order: Option<i64>,
    pub owner_consumable_occurrence_id: Option<String>,
    pub owner_consumable_occurrence_authored_order: Option<i64>,
    pub role: String,
    pub origin_json: String,
    pub visibility: String,
    pub provenance_json: String,
    pub target_kind: String,
    pub target_record_key: Option<String>,
    pub target_json: String,
    pub label: Option<String>,
    pub relation_kind: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::record_aliases)]
pub(super) struct RecordAliasRow {
    pub canonical_record_key: String,
    pub alias_text: String,
    pub normalized_alias: String,
    pub source_kind: String,
    pub source_ref: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::remaster_links)]
pub(super) struct RemasterLinkRow {
    pub remaster_record_key: String,
    pub legacy_record_key: String,
    pub source_kind: String,
    pub source_ref: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::document_embedding_cache)]
pub(super) struct DocumentEmbeddingCacheRow {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub ordinal: i64,
    pub semantic_input_hash: String,
    pub dimensions: i64,
    pub vector_blob: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::metric_key_catalog)]
pub(super) struct MetricKeyCatalogRow {
    pub metric_domain: String,
    pub record_kind: Option<String>,
    pub namespace_prefix: String,
    pub metric_key: String,
    pub value_type: String,
    pub catalog_count: i64,
    pub numeric_min: Option<f64>,
    pub numeric_max: Option<f64>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::metric_value_catalog)]
pub(super) struct MetricValueCatalogRow {
    pub metric_domain: String,
    pub record_kind: Option<String>,
    pub metric_key: String,
    pub value: String,
    pub catalog_count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::filter_field_catalog)]
pub(super) struct FilterFieldCatalogRow {
    pub field: String,
    pub record_kind: Option<String>,
    pub field_type: String,
    pub field_group: String,
    pub value_policy: String,
    pub operators_json: String,
    pub cli_flags_json: String,
    pub applicable_kinds_json: String,
    pub value_count: i64,
    pub matching_record_count: i64,
    pub null_count: i64,
    pub distinct_count: i64,
    pub singleton_count: i64,
    pub singleton_ratio: Option<f64>,
    pub observation_singleton_ratio: Option<f64>,
    pub policy_reason: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::filter_value_catalog)]
pub(super) struct FilterValueCatalogRow {
    pub field: String,
    pub record_kind: Option<String>,
    pub value: String,
    pub catalog_count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::filter_sample_catalog)]
pub(super) struct FilterSampleCatalogRow {
    pub field: String,
    pub record_kind: Option<String>,
    pub value: String,
    pub catalog_count: i64,
    pub sample_rank: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::filter_numeric_catalog)]
pub(super) struct FilterNumericCatalogRow {
    pub field: String,
    pub record_kind: Option<String>,
    pub metric_domain: Option<String>,
    pub metric_key: Option<String>,
    pub catalog_count: i64,
    pub null_count: i64,
    pub min: Option<f64>,
    pub p05: Option<f64>,
    pub p25: Option<f64>,
    pub p50: Option<f64>,
    pub mean: Option<f64>,
    pub p75: Option<f64>,
    pub p95: Option<f64>,
    pub max: Option<f64>,
}
