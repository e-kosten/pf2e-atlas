use super::{Column, Table};

macro_rules! table_inventory {
    ($module:ident, $table:literal, [$($column:ident => $name:literal),+ $(,)?]) => {
        pub mod $module {
            use super::{Column, Table};
            pub const TABLE: Table = Table::new($table);
            pub mod columns {
                use super::{Column, TABLE};
                $(pub const $column: Column = Column::new(TABLE, $name);)+
            }
            pub const ALL_COLUMNS: &[Column] = &[$(columns::$column),+];
        }
    };
}

table_inventory!(canonical_creature_records, "canonical_creature_records", [
    RECORD_KEY => "record_key", SOURCE_ID => "source_id", NAME => "name",
    FAMILY => "family", CANONICAL_JSON => "canonical_json"
]);
table_inventory!(canonical_creature_resources, "canonical_creature_resources", [
    RECORD_KEY => "record_key", RESOURCE_ID => "resource_id", AUTHORED_ORDER => "authored_order",
    RESOURCE_KIND => "resource_kind", RESOURCE_JSON => "resource_json"
]);
table_inventory!(canonical_creature_entities, "canonical_creature_entities", [
    RECORD_KEY => "record_key", ENTITY_ID => "entity_id", FAMILY => "family",
    LABEL => "label", SOURCE_IDENTITY_JSON => "source_identity_json"
]);
table_inventory!(canonical_creature_occurrences, "canonical_creature_occurrences", [
    RECORD_KEY => "record_key", OCCURRENCE_ID => "occurrence_id",
    IDENTITY_STABILITY => "identity_stability", FAMILY => "family",
    AUTHORED_ORDER => "authored_order", SOURCE_SORT_JSON => "source_sort_json",
    SOURCE_FOLDER_JSON => "source_folder_json", SOURCE_IDENTITY_JSON => "source_identity_json",
    PARENT_KIND => "parent_kind",
    PARENT_OCCURRENCE_ID => "parent_occurrence_id", TARGET_KIND => "target_kind",
    TARGET_RECORD_KEY => "target_record_key", TARGET_ENTITY_ID => "target_entity_id",
    CONTEXT_JSON => "context_json", CAPABILITY_JSON => "capability_json", DELTAS_JSON => "deltas_json"
]);
table_inventory!(canonical_creature_relationships, "canonical_creature_relationships", [
    RECORD_KEY => "record_key", RELATIONSHIP_ORDER => "relationship_order",
    SOURCE_OCCURRENCE_ID => "source_occurrence_id",
    SOURCE_OCCURRENCE_AUTHORED_ORDER => "source_occurrence_authored_order",
    RELATIONSHIP_KIND => "relationship_kind", TARGET_KIND => "target_kind",
    TARGET_OCCURRENCE_ID => "target_occurrence_id",
    TARGET_OCCURRENCE_AUTHORED_ORDER => "target_occurrence_authored_order",
    TARGET_SOURCE_ID => "target_source_id", SOURCE_PATH => "source_path",
    CONTEXTUAL_LABEL_JSON => "contextual_label_json", LIFECYCLE_JSON => "lifecycle_json",
    EXECUTION => "execution"
]);
table_inventory!(canonical_hazard_records, "canonical_hazard_records", [
    RECORD_KEY => "record_key", SOURCE_ID => "source_id", NAME => "name",
    FAMILY => "family", CANONICAL_JSON => "canonical_json"
]);
table_inventory!(canonical_hazard_entities, "canonical_hazard_entities", [
    RECORD_KEY => "record_key", ENTITY_ID => "entity_id", FAMILY => "family",
    LABEL => "label", IMAGE_JSON => "image_json", SOURCE_IDENTITY_JSON => "source_identity_json",
    CAPABILITY_JSON => "capability_json"
]);
table_inventory!(canonical_hazard_occurrences, "canonical_hazard_occurrences", [
    RECORD_KEY => "record_key", OCCURRENCE_ID => "occurrence_id", ENTITY_ID => "entity_id",
    IDENTITY_STABILITY => "identity_stability", FAMILY => "family",
    AUTHORED_ORDER => "authored_order", SOURCE_SORT_JSON => "source_sort_json",
    SOURCE_FOLDER_JSON => "source_folder_json", SOURCE_ORDINAL => "source_ordinal",
    CONTEXTUAL_LABEL_JSON => "contextual_label_json"
]);
table_inventory!(canonical_hazard_relationships, "canonical_hazard_relationships", [
    RECORD_KEY => "record_key", RELATIONSHIP_ID => "relationship_id",
    AUTHORED_ORDER => "authored_order", SOURCE_OCCURRENCE_ID => "source_occurrence_id",
    SOURCE_OCCURRENCE_AUTHORED_ORDER => "source_occurrence_authored_order",
    RELATIONSHIP_KIND => "relationship_kind", TARGET_KIND => "target_kind",
    TARGET_ENTITY_ID => "target_entity_id", TARGET_OCCURRENCE_ID => "target_occurrence_id",
    TARGET_OCCURRENCE_AUTHORED_ORDER => "target_occurrence_authored_order"
]);
table_inventory!(canonical_spell_records, "canonical_spell_records", [
    RECORD_KEY => "record_key", SOURCE_ID => "source_id", NAME => "name",
    CANONICAL_JSON => "canonical_json"
]);
table_inventory!(canonical_journal_records, "canonical_journal_records", [
    RECORD_KEY => "record_key", SOURCE_ID => "source_id", NAME => "name",
    CANONICAL_JSON => "canonical_json"
]);
table_inventory!(canonical_roll_table_records, "canonical_roll_table_records", [
    RECORD_KEY => "record_key", SOURCE_ID => "source_id", NAME => "name",
    CANONICAL_JSON => "canonical_json"
]);
table_inventory!(canonical_consumable_spell_children, "canonical_consumable_spell_children", [
    PARENT_RECORD_KEY => "parent_record_key", CHILD_ID => "child_id",
    AUTHORED_ORDER => "authored_order",
    STANDALONE_TARGET_RECORD_KEY => "standalone_target_record_key",
    CANONICAL_JSON => "canonical_json"
]);
table_inventory!(spell_traditions, "spell_traditions", [
    RECORD_KEY => "record_key", AUTHORED_ORDER => "authored_order", TRADITION => "tradition"
]);
table_inventory!(spell_damage_types, "spell_damage_types", [
    RECORD_KEY => "record_key", DAMAGE_KEY => "damage_key",
    DAMAGE_AUTHORED_ORDER => "damage_authored_order",
    TYPE_AUTHORED_ORDER => "type_authored_order", DAMAGE_TYPE => "damage_type"
]);
table_inventory!(record_content_exclusions, "record_content_exclusions", [
    RECORD_KEY => "record_key", CONTENT_KEY => "content_key",
    RELATIVE_SOURCE_PATH => "relative_source_path", LABEL => "label", REASON => "reason"
]);
