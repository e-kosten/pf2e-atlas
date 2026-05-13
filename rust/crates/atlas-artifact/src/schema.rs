pub const REQUIRED_TABLES: &[&str] = &[
    "artifact_metadata",
    "packs",
    "records",
    "record_traits",
    "reference_edges",
    "record_aliases",
    "remaster_links",
    "record_metrics",
    "metric_key_catalog",
    "metric_value_catalog",
    "actor_records",
    "item_records",
    "spell_records",
    "records_fts",
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

pub const REQUIRED_COLUMNS: &[(&str, &[&str])] = &[
    ("artifact_metadata", &["key", "value"]),
    (
        "packs",
        &[
            "name",
            "label",
            "document_type",
            "declared_path",
            "resolved_path",
            "record_count",
        ],
    ),
    ("records", RECORD_COLUMNS),
    ("record_traits", &["record_key", "trait"]),
    (
        "reference_edges",
        &[
            "from_record_key",
            "to_record_key",
            "display_text",
            "reference_text",
        ],
    ),
    (
        "record_aliases",
        &[
            "canonical_record_key",
            "alias_text",
            "normalized_alias",
            "source_kind",
            "source_ref",
        ],
    ),
    (
        "remaster_links",
        &[
            "remaster_record_key",
            "legacy_record_key",
            "source_kind",
            "source_ref",
        ],
    ),
    (
        "record_metrics",
        &[
            "record_key",
            "metric_domain",
            "metric_key",
            "value_type",
            "number_value",
            "text_value",
            "bool_value",
        ],
    ),
    (
        "metric_key_catalog",
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
        "metric_value_catalog",
        &[
            "metric_domain",
            "record_family",
            "metric_key",
            "value",
            "catalog_count",
        ],
    ),
    (
        "actor_records",
        &[
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
        ],
    ),
    (
        "item_records",
        &[
            "record_key",
            "system_category",
            "system_base_item",
            "system_group",
            "system_usage",
            "price_cp",
            "bulk_value",
            "hands_requirement",
            "damage_types_json",
        ],
    ),
    (
        "spell_records",
        &[
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
        ],
    ),
    (
        "records_fts",
        &["record_key", "name", "search_text_projection"],
    ),
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
