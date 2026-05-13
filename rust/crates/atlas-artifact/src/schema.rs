pub const TABLE_ARTIFACT_METADATA: &str = "artifact_metadata";
pub const TABLE_PACKS: &str = "packs";
pub const TABLE_RECORDS: &str = "records";
pub const TABLE_RECORD_TRAITS: &str = "record_traits";
pub const TABLE_REFERENCE_EDGES: &str = "reference_edges";
pub const TABLE_RECORD_ALIASES: &str = "record_aliases";
pub const TABLE_REMASTER_LINKS: &str = "remaster_links";
pub const TABLE_RECORD_METRICS: &str = "record_metrics";
pub const TABLE_METRIC_KEY_CATALOG: &str = "metric_key_catalog";
pub const TABLE_METRIC_VALUE_CATALOG: &str = "metric_value_catalog";
pub const TABLE_ACTOR_RECORDS: &str = "actor_records";
pub const TABLE_ITEM_RECORDS: &str = "item_records";
pub const TABLE_SPELL_RECORDS: &str = "spell_records";
pub const TABLE_RECORDS_FTS: &str = "records_fts";

pub const REQUIRED_TABLES: &[&str] = &[
    TABLE_ARTIFACT_METADATA,
    TABLE_PACKS,
    TABLE_RECORDS,
    TABLE_RECORD_TRAITS,
    TABLE_REFERENCE_EDGES,
    TABLE_RECORD_ALIASES,
    TABLE_REMASTER_LINKS,
    TABLE_RECORD_METRICS,
    TABLE_METRIC_KEY_CATALOG,
    TABLE_METRIC_VALUE_CATALOG,
    TABLE_ACTOR_RECORDS,
    TABLE_ITEM_RECORDS,
    TABLE_SPELL_RECORDS,
    TABLE_RECORDS_FTS,
];

pub const RECORD_COLUMNS: &[&str] = &[
    "record_key",
    "id",
    "name",
    "normalized_name",
    "record_family",
    "pack_name",
    "pack_label",
    "foundry_document_type",
    "foundry_record_type",
    "level",
    "rarity",
    "traits_json",
    "system_category",
    "system_group",
    "system_base_item",
    "system_usage",
    "system_price_json",
    "system_actions_value",
    "system_time_value",
    "system_duration_value",
    "price_cp",
    "activation_time_kind",
    "activation_time_actions",
    "activation_time_duration_value",
    "activation_time_duration_unit",
    "activation_time_text",
    "duration_kind",
    "duration_value",
    "duration_unit",
    "duration_text",
    "publication_title",
    "publication_remaster",
    "description_text",
    "blurb_text",
    "description_snippet",
    "publication_family",
    "folder_id",
    "taxonomy_families_json",
    "variant_group_key",
    "variant_base_name",
    "variant_label",
    "variant_axes_json",
    "variant_confidence",
    "variant_source",
    "source_path",
    "is_default_visible",
    "search_text_projection",
    "raw_json",
];

pub const PERSISTED_RECORD_COLUMNS: &[&str] = &[
    "record_key",
    "id",
    "name",
    "normalized_name",
    "record_family",
    "pack_name",
    "pack_label",
    "foundry_document_type",
    "foundry_record_type",
    "level",
    "rarity",
    "traits_json",
    "system_category",
    "system_group",
    "system_base_item",
    "system_usage",
    "system_price_json",
    "system_actions_value",
    "system_time_value",
    "system_duration_value",
    "price_cp",
    "activation_time_kind",
    "activation_time_actions",
    "activation_time_duration_value",
    "activation_time_duration_unit",
    "activation_time_text",
    "duration_kind",
    "duration_value",
    "duration_unit",
    "duration_text",
    "publication_title",
    "publication_remaster",
    "description_text",
    "blurb_text",
    "publication_family",
    "folder_id",
    "taxonomy_families_json",
    "variant_group_key",
    "variant_base_name",
    "variant_label",
    "variant_axes_json",
    "variant_confidence",
    "variant_source",
    "source_path",
    "is_default_visible",
    "search_text_projection",
    "raw_json",
];

pub const RECORD_TRAIT_COLUMNS: &[&str] = &["record_key", "trait"];

pub const RECORD_METRIC_COLUMNS: &[&str] = &[
    "record_key",
    "metric_domain",
    "metric_key",
    "value_type",
    "number_value",
    "text_value",
    "bool_value",
];

pub const ACTOR_RECORD_COLUMNS: &[&str] = &[
    "record_key",
    "size",
    "languages_json",
    "speed_types_json",
    "senses_json",
    "immunities_json",
    "resistances_json",
    "weaknesses_json",
    "disable_text",
    "disable_skills_json",
    "is_complex",
];

pub const ITEM_RECORD_COLUMNS: &[&str] = &[
    "record_key",
    "system_category",
    "system_base_item",
    "system_group",
    "system_usage",
    "price_cp",
    "bulk_value",
    "hands_requirement",
    "damage_types_json",
];

pub const SPELL_RECORD_COLUMNS: &[&str] = &[
    "record_key",
    "traditions_json",
    "spell_kinds_json",
    "range_text",
    "range_value",
    "target_text",
    "area_type",
    "area_value",
    "save_type",
    "sustained",
    "basic_save",
    "damage_types_json",
];

pub const RECORDS_FTS_COLUMNS: &[&str] = &["record_key", "name", "search_text_projection"];

pub fn record_insert_sql() -> String {
    insert_sql(TABLE_RECORDS, RECORD_COLUMNS)
}

pub fn record_trait_insert_sql() -> String {
    insert_sql(TABLE_RECORD_TRAITS, RECORD_TRAIT_COLUMNS)
}

pub fn record_metric_insert_sql() -> String {
    insert_sql(TABLE_RECORD_METRICS, RECORD_METRIC_COLUMNS)
}

pub fn actor_record_insert_sql() -> String {
    insert_sql(TABLE_ACTOR_RECORDS, ACTOR_RECORD_COLUMNS)
}

pub fn item_record_insert_sql() -> String {
    insert_sql(TABLE_ITEM_RECORDS, ITEM_RECORD_COLUMNS)
}

pub fn spell_record_insert_sql() -> String {
    insert_sql(TABLE_SPELL_RECORDS, SPELL_RECORD_COLUMNS)
}

pub fn records_fts_insert_sql() -> String {
    insert_sql(TABLE_RECORDS_FTS, RECORDS_FTS_COLUMNS)
}

pub fn persisted_record_select_sql() -> String {
    ordered_select_sql(TABLE_RECORDS, PERSISTED_RECORD_COLUMNS, "record_key")
}

pub fn insert_sql(table: &str, columns: &[&str]) -> String {
    let placeholders = (1..=columns.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "INSERT INTO {table} ({columns}) VALUES ({placeholders})",
        columns = columns.join(", ")
    )
}

pub fn ordered_select_sql(table: &str, columns: &[&str], order_by: &str) -> String {
    format!(
        "SELECT {columns} FROM {table} ORDER BY {order_by}",
        columns = columns.join(", ")
    )
}

pub const REQUIRED_COLUMNS: &[(&str, &[&str])] = &[
    (TABLE_ARTIFACT_METADATA, &["key", "value"]),
    (
        TABLE_PACKS,
        &[
            "name",
            "label",
            "document_type",
            "declared_path",
            "resolved_path",
            "record_count",
        ],
    ),
    (TABLE_RECORDS, RECORD_COLUMNS),
    (TABLE_RECORD_TRAITS, RECORD_TRAIT_COLUMNS),
    (
        TABLE_REFERENCE_EDGES,
        &[
            "from_record_key",
            "to_record_key",
            "display_text",
            "reference_text",
        ],
    ),
    (
        TABLE_RECORD_ALIASES,
        &[
            "canonical_record_key",
            "alias_text",
            "normalized_alias",
            "source_kind",
            "source_ref",
        ],
    ),
    (
        TABLE_REMASTER_LINKS,
        &[
            "remaster_record_key",
            "legacy_record_key",
            "source_kind",
            "source_ref",
        ],
    ),
    (TABLE_RECORD_METRICS, RECORD_METRIC_COLUMNS),
    (
        TABLE_METRIC_KEY_CATALOG,
        &[
            "metric_domain",
            "record_family",
            "namespace_prefix",
            "metric_key",
            "value_type",
            "catalog_count",
            "numeric_min",
            "numeric_max",
        ],
    ),
    (
        TABLE_METRIC_VALUE_CATALOG,
        &[
            "metric_domain",
            "record_family",
            "metric_key",
            "value",
            "catalog_count",
        ],
    ),
    (TABLE_ACTOR_RECORDS, ACTOR_RECORD_COLUMNS),
    (TABLE_ITEM_RECORDS, ITEM_RECORD_COLUMNS),
    (TABLE_SPELL_RECORDS, SPELL_RECORD_COLUMNS),
    (TABLE_RECORDS_FTS, RECORDS_FTS_COLUMNS),
];

pub const CREATE_ARTIFACT_SCHEMA_SQL: &str = r#"            PRAGMA foreign_keys = ON;

            CREATE TABLE artifact_metadata (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );

            CREATE TABLE packs (
              name TEXT PRIMARY KEY,
              label TEXT NOT NULL,
              document_type TEXT NOT NULL,
              declared_path TEXT NOT NULL,
              resolved_path TEXT NOT NULL,
              record_count INTEGER NOT NULL
            );

            CREATE TABLE records (
              record_key TEXT PRIMARY KEY,
              id TEXT NOT NULL,
              name TEXT NOT NULL,
              normalized_name TEXT NOT NULL,
              record_family TEXT NOT NULL,
              pack_name TEXT NOT NULL,
              pack_label TEXT NOT NULL,
              foundry_document_type TEXT NOT NULL,
              foundry_record_type TEXT NOT NULL,
              level INTEGER,
              rarity TEXT,
              traits_json TEXT NOT NULL,
              system_category TEXT,
              system_group TEXT,
              system_base_item TEXT,
              system_usage TEXT,
              system_price_json TEXT,
              system_actions_value INTEGER,
              system_time_value TEXT,
              system_duration_value TEXT,
              price_cp INTEGER,
              activation_time_kind TEXT,
              activation_time_actions INTEGER,
              activation_time_duration_value INTEGER,
              activation_time_duration_unit TEXT,
              activation_time_text TEXT,
              duration_kind TEXT,
              duration_value INTEGER,
              duration_unit TEXT,
              duration_text TEXT,
              publication_title TEXT,
              publication_remaster INTEGER NOT NULL CHECK (publication_remaster IN (0, 1)),
              description_text TEXT,
              blurb_text TEXT,
              description_snippet TEXT,
              publication_family TEXT NOT NULL,
              folder_id TEXT,
              taxonomy_families_json TEXT NOT NULL,
              variant_group_key TEXT,
              variant_base_name TEXT,
              variant_label TEXT,
              variant_axes_json TEXT NOT NULL,
              variant_confidence REAL,
              variant_source TEXT NOT NULL,
              source_path TEXT NOT NULL,
              is_default_visible INTEGER NOT NULL CHECK (is_default_visible IN (0, 1)),
              search_text_projection TEXT NOT NULL,
              raw_json TEXT NOT NULL
            );

            CREATE TABLE record_traits (
              record_key TEXT NOT NULL,
              trait TEXT NOT NULL,
              PRIMARY KEY (record_key, trait),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE reference_edges (
              from_record_key TEXT NOT NULL,
              to_record_key TEXT NOT NULL,
              display_text TEXT,
              reference_text TEXT NOT NULL,
              PRIMARY KEY (from_record_key, to_record_key, reference_text),
              FOREIGN KEY (from_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
              FOREIGN KEY (to_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE record_aliases (
              canonical_record_key TEXT NOT NULL,
              alias_text TEXT NOT NULL,
              normalized_alias TEXT NOT NULL,
              source_kind TEXT NOT NULL CHECK (source_kind IN ('remaster_journal', 'migration', 'compendium_source')),
              source_ref TEXT NOT NULL,
              PRIMARY KEY (canonical_record_key, normalized_alias, source_kind, source_ref),
              FOREIGN KEY (canonical_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE remaster_links (
              remaster_record_key TEXT NOT NULL,
              legacy_record_key TEXT NOT NULL,
              source_kind TEXT NOT NULL CHECK (source_kind IN ('remaster_journal', 'migration')),
              source_ref TEXT NOT NULL,
              PRIMARY KEY (remaster_record_key, legacy_record_key, source_kind, source_ref),
              FOREIGN KEY (remaster_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
              FOREIGN KEY (legacy_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE record_metrics (
              record_key TEXT NOT NULL,
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              metric_key TEXT NOT NULL,
              value_type TEXT NOT NULL CHECK (value_type IN ('number', 'text', 'boolean')),
              number_value REAL,
              text_value TEXT,
              bool_value INTEGER CHECK (bool_value IN (0, 1)),
              PRIMARY KEY (record_key, metric_domain, metric_key),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE metric_key_catalog (
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              record_family TEXT NOT NULL,
              namespace_prefix TEXT NOT NULL,
              metric_key TEXT NOT NULL,
              value_type TEXT NOT NULL CHECK (value_type IN ('number', 'text', 'boolean')),
              catalog_count INTEGER NOT NULL,
              numeric_min REAL,
              numeric_max REAL,
              PRIMARY KEY (metric_domain, record_family, metric_key)
            );

            CREATE TABLE metric_value_catalog (
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              record_family TEXT NOT NULL,
              metric_key TEXT NOT NULL,
              value TEXT NOT NULL,
              catalog_count INTEGER NOT NULL,
              PRIMARY KEY (metric_domain, record_family, metric_key, value)
            );

            CREATE TABLE actor_records (
              record_key TEXT PRIMARY KEY,
              size TEXT,
              languages_json TEXT NOT NULL,
              speed_types_json TEXT NOT NULL,
              senses_json TEXT NOT NULL,
              immunities_json TEXT NOT NULL,
              resistances_json TEXT NOT NULL,
              weaknesses_json TEXT NOT NULL,
              disable_text TEXT,
              disable_skills_json TEXT NOT NULL,
              is_complex INTEGER NOT NULL CHECK (is_complex IN (0, 1)),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE item_records (
              record_key TEXT PRIMARY KEY,
              system_category TEXT,
              system_base_item TEXT,
              system_group TEXT,
              system_usage TEXT,
              price_cp INTEGER,
              bulk_value REAL,
              hands_requirement TEXT,
              damage_types_json TEXT NOT NULL,
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE spell_records (
              record_key TEXT PRIMARY KEY,
              traditions_json TEXT NOT NULL,
              spell_kinds_json TEXT NOT NULL,
              range_text TEXT,
              range_value REAL,
              target_text TEXT,
              area_type TEXT,
              area_value REAL,
              save_type TEXT,
              sustained INTEGER NOT NULL CHECK (sustained IN (0, 1)),
              basic_save INTEGER NOT NULL CHECK (basic_save IN (0, 1)),
              damage_types_json TEXT NOT NULL,
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE VIRTUAL TABLE records_fts USING fts5(
              record_key UNINDEXED,
              name,
              search_text_projection
            );

            CREATE INDEX records_pack_name_idx ON records(pack_name);
            CREATE INDEX records_default_visible_idx ON records(is_default_visible);
            CREATE INDEX reference_edges_from_idx ON reference_edges(from_record_key);
            CREATE INDEX reference_edges_to_idx ON reference_edges(to_record_key);
            CREATE INDEX record_aliases_canonical_idx ON record_aliases(canonical_record_key);
            CREATE INDEX record_aliases_normalized_alias_idx ON record_aliases(normalized_alias);
            CREATE INDEX remaster_links_remaster_idx ON remaster_links(remaster_record_key);
            CREATE INDEX remaster_links_legacy_idx ON remaster_links(legacy_record_key);
            CREATE INDEX record_metrics_record_idx ON record_metrics(record_key);
            CREATE INDEX record_metrics_catalog_source_idx ON record_metrics(metric_domain, metric_key, value_type);
            CREATE INDEX metric_key_catalog_coverage_idx ON metric_key_catalog(metric_domain, record_family, metric_key);
            CREATE INDEX metric_value_catalog_coverage_idx ON metric_value_catalog(metric_domain, record_family, metric_key, value);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_insert_sql_uses_record_column_descriptor_order() {
        assert_eq!(
            record_insert_sql(),
            format!(
                "INSERT INTO records ({}) VALUES ({})",
                RECORD_COLUMNS.join(", "),
                (1..=RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
    }

    #[test]
    fn side_table_insert_sql_uses_column_descriptor_order() {
        assert_eq!(
            record_trait_insert_sql(),
            "INSERT INTO record_traits (record_key, trait) VALUES (?1, ?2)"
        );
        assert_eq!(
            record_metric_insert_sql(),
            format!(
                "INSERT INTO record_metrics ({}) VALUES ({})",
                RECORD_METRIC_COLUMNS.join(", "),
                (1..=RECORD_METRIC_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            actor_record_insert_sql(),
            format!(
                "INSERT INTO actor_records ({}) VALUES ({})",
                ACTOR_RECORD_COLUMNS.join(", "),
                (1..=ACTOR_RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            item_record_insert_sql(),
            format!(
                "INSERT INTO item_records ({}) VALUES ({})",
                ITEM_RECORD_COLUMNS.join(", "),
                (1..=ITEM_RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            spell_record_insert_sql(),
            format!(
                "INSERT INTO spell_records ({}) VALUES ({})",
                SPELL_RECORD_COLUMNS.join(", "),
                (1..=SPELL_RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            records_fts_insert_sql(),
            "INSERT INTO records_fts (record_key, name, search_text_projection) VALUES (?1, ?2, ?3)"
        );
    }

    #[test]
    fn persisted_record_select_sql_uses_persisted_column_descriptor_order() {
        assert_eq!(
            persisted_record_select_sql(),
            format!(
                "SELECT {} FROM records ORDER BY record_key",
                PERSISTED_RECORD_COLUMNS.join(", ")
            )
        );
        assert!(!PERSISTED_RECORD_COLUMNS.contains(&"description_snippet"));
        assert!(RECORD_COLUMNS.contains(&"description_snippet"));
    }
}
