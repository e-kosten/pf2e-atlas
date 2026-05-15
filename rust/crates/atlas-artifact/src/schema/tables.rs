use super::{Column, Table};

pub mod artifact_metadata {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("artifact_metadata");

    pub mod columns {
        use super::{Column, TABLE};

        pub const KEY: Column = Column::new(TABLE, "key");
        pub const VALUE: Column = Column::new(TABLE, "value");
    }

    pub const ALL_COLUMNS: &[Column] = &[columns::KEY, columns::VALUE];
}

pub mod packs {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("packs");

    pub mod columns {
        use super::{Column, TABLE};

        pub const NAME: Column = Column::new(TABLE, "name");
        pub const LABEL: Column = Column::new(TABLE, "label");
        pub const DOCUMENT_TYPE: Column = Column::new(TABLE, "document_type");
        pub const DECLARED_PATH: Column = Column::new(TABLE, "declared_path");
        pub const RESOLVED_PATH: Column = Column::new(TABLE, "resolved_path");
        pub const RECORD_COUNT: Column = Column::new(TABLE, "record_count");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::NAME,
        columns::LABEL,
        columns::DOCUMENT_TYPE,
        columns::DECLARED_PATH,
        columns::RESOLVED_PATH,
        columns::RECORD_COUNT,
    ];
}

pub mod records {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("records");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const ID: Column = Column::new(TABLE, "id");
        pub const NAME: Column = Column::new(TABLE, "name");
        pub const NORMALIZED_NAME: Column = Column::new(TABLE, "normalized_name");
        pub const RECORD_FAMILY: Column = Column::new(TABLE, "record_family");
        pub const PACK_NAME: Column = Column::new(TABLE, "pack_name");
        pub const PACK_LABEL: Column = Column::new(TABLE, "pack_label");
        pub const FOUNDRY_DOCUMENT_TYPE: Column = Column::new(TABLE, "foundry_document_type");
        pub const FOUNDRY_RECORD_TYPE: Column = Column::new(TABLE, "foundry_record_type");
        pub const LEVEL: Column = Column::new(TABLE, "level");
        pub const RARITY: Column = Column::new(TABLE, "rarity");
        pub const TRAITS_JSON: Column = Column::new(TABLE, "traits_json");
        pub const SYSTEM_CATEGORY: Column = Column::new(TABLE, "system_category");
        pub const SYSTEM_GROUP: Column = Column::new(TABLE, "system_group");
        pub const SYSTEM_BASE_ITEM: Column = Column::new(TABLE, "system_base_item");
        pub const SYSTEM_USAGE: Column = Column::new(TABLE, "system_usage");
        pub const SYSTEM_PRICE_JSON: Column = Column::new(TABLE, "system_price_json");
        pub const SYSTEM_ACTIONS_VALUE: Column = Column::new(TABLE, "system_actions_value");
        pub const SYSTEM_TIME_VALUE: Column = Column::new(TABLE, "system_time_value");
        pub const SYSTEM_DURATION_VALUE: Column = Column::new(TABLE, "system_duration_value");
        pub const PRICE_CP: Column = Column::new(TABLE, "price_cp");
        pub const ACTIVATION_TIME_KIND: Column = Column::new(TABLE, "activation_time_kind");
        pub const ACTIVATION_TIME_ACTIONS: Column = Column::new(TABLE, "activation_time_actions");
        pub const ACTIVATION_TIME_DURATION_VALUE: Column =
            Column::new(TABLE, "activation_time_duration_value");
        pub const ACTIVATION_TIME_DURATION_UNIT: Column =
            Column::new(TABLE, "activation_time_duration_unit");
        pub const ACTIVATION_TIME_TEXT: Column = Column::new(TABLE, "activation_time_text");
        pub const DURATION_KIND: Column = Column::new(TABLE, "duration_kind");
        pub const DURATION_VALUE: Column = Column::new(TABLE, "duration_value");
        pub const DURATION_UNIT: Column = Column::new(TABLE, "duration_unit");
        pub const DURATION_TEXT: Column = Column::new(TABLE, "duration_text");
        pub const PUBLICATION_TITLE: Column = Column::new(TABLE, "publication_title");
        pub const PUBLICATION_REMASTER: Column = Column::new(TABLE, "publication_remaster");
        pub const DESCRIPTION_TEXT: Column = Column::new(TABLE, "description_text");
        pub const BLURB_TEXT: Column = Column::new(TABLE, "blurb_text");
        pub const DESCRIPTION_SNIPPET: Column = Column::new(TABLE, "description_snippet");
        pub const PUBLICATION_FAMILY: Column = Column::new(TABLE, "publication_family");
        pub const FOLDER_ID: Column = Column::new(TABLE, "folder_id");
        pub const TAXONOMY_FAMILIES_JSON: Column = Column::new(TABLE, "taxonomy_families_json");
        pub const VARIANT_GROUP_KEY: Column = Column::new(TABLE, "variant_group_key");
        pub const VARIANT_BASE_NAME: Column = Column::new(TABLE, "variant_base_name");
        pub const VARIANT_LABEL: Column = Column::new(TABLE, "variant_label");
        pub const VARIANT_AXES_JSON: Column = Column::new(TABLE, "variant_axes_json");
        pub const VARIANT_CONFIDENCE: Column = Column::new(TABLE, "variant_confidence");
        pub const VARIANT_SOURCE: Column = Column::new(TABLE, "variant_source");
        pub const SOURCE_PATH: Column = Column::new(TABLE, "source_path");
        pub const IS_DEFAULT_VISIBLE: Column = Column::new(TABLE, "is_default_visible");
        pub const SEARCH_TEXT_PROJECTION: Column = Column::new(TABLE, "search_text_projection");
        pub const RAW_JSON: Column = Column::new(TABLE, "raw_json");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::ID,
        columns::NAME,
        columns::NORMALIZED_NAME,
        columns::RECORD_FAMILY,
        columns::PACK_NAME,
        columns::PACK_LABEL,
        columns::FOUNDRY_DOCUMENT_TYPE,
        columns::FOUNDRY_RECORD_TYPE,
        columns::LEVEL,
        columns::RARITY,
        columns::TRAITS_JSON,
        columns::SYSTEM_CATEGORY,
        columns::SYSTEM_GROUP,
        columns::SYSTEM_BASE_ITEM,
        columns::SYSTEM_USAGE,
        columns::SYSTEM_PRICE_JSON,
        columns::SYSTEM_ACTIONS_VALUE,
        columns::SYSTEM_TIME_VALUE,
        columns::SYSTEM_DURATION_VALUE,
        columns::PRICE_CP,
        columns::ACTIVATION_TIME_KIND,
        columns::ACTIVATION_TIME_ACTIONS,
        columns::ACTIVATION_TIME_DURATION_VALUE,
        columns::ACTIVATION_TIME_DURATION_UNIT,
        columns::ACTIVATION_TIME_TEXT,
        columns::DURATION_KIND,
        columns::DURATION_VALUE,
        columns::DURATION_UNIT,
        columns::DURATION_TEXT,
        columns::PUBLICATION_TITLE,
        columns::PUBLICATION_REMASTER,
        columns::DESCRIPTION_TEXT,
        columns::BLURB_TEXT,
        columns::DESCRIPTION_SNIPPET,
        columns::PUBLICATION_FAMILY,
        columns::FOLDER_ID,
        columns::TAXONOMY_FAMILIES_JSON,
        columns::VARIANT_GROUP_KEY,
        columns::VARIANT_BASE_NAME,
        columns::VARIANT_LABEL,
        columns::VARIANT_AXES_JSON,
        columns::VARIANT_CONFIDENCE,
        columns::VARIANT_SOURCE,
        columns::SOURCE_PATH,
        columns::IS_DEFAULT_VISIBLE,
        columns::SEARCH_TEXT_PROJECTION,
        columns::RAW_JSON,
    ];

    pub const PERSISTED_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::ID,
        columns::NAME,
        columns::NORMALIZED_NAME,
        columns::RECORD_FAMILY,
        columns::PACK_NAME,
        columns::PACK_LABEL,
        columns::FOUNDRY_DOCUMENT_TYPE,
        columns::FOUNDRY_RECORD_TYPE,
        columns::LEVEL,
        columns::RARITY,
        columns::TRAITS_JSON,
        columns::SYSTEM_CATEGORY,
        columns::SYSTEM_GROUP,
        columns::SYSTEM_BASE_ITEM,
        columns::SYSTEM_USAGE,
        columns::SYSTEM_PRICE_JSON,
        columns::SYSTEM_ACTIONS_VALUE,
        columns::SYSTEM_TIME_VALUE,
        columns::SYSTEM_DURATION_VALUE,
        columns::PRICE_CP,
        columns::ACTIVATION_TIME_KIND,
        columns::ACTIVATION_TIME_ACTIONS,
        columns::ACTIVATION_TIME_DURATION_VALUE,
        columns::ACTIVATION_TIME_DURATION_UNIT,
        columns::ACTIVATION_TIME_TEXT,
        columns::DURATION_KIND,
        columns::DURATION_VALUE,
        columns::DURATION_UNIT,
        columns::DURATION_TEXT,
        columns::PUBLICATION_TITLE,
        columns::PUBLICATION_REMASTER,
        columns::DESCRIPTION_TEXT,
        columns::BLURB_TEXT,
        columns::PUBLICATION_FAMILY,
        columns::FOLDER_ID,
        columns::TAXONOMY_FAMILIES_JSON,
        columns::VARIANT_GROUP_KEY,
        columns::VARIANT_BASE_NAME,
        columns::VARIANT_LABEL,
        columns::VARIANT_AXES_JSON,
        columns::VARIANT_CONFIDENCE,
        columns::VARIANT_SOURCE,
        columns::SOURCE_PATH,
        columns::IS_DEFAULT_VISIBLE,
        columns::SEARCH_TEXT_PROJECTION,
        columns::RAW_JSON,
    ];
}

pub mod record_traits {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("record_traits");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const TRAIT: Column = Column::new(TABLE, "trait");
    }

    pub const ALL_COLUMNS: &[Column] = &[columns::RECORD_KEY, columns::TRAIT];
}

pub mod reference_edges {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("reference_edges");

    pub mod columns {
        use super::{Column, TABLE};

        pub const FROM_RECORD_KEY: Column = Column::new(TABLE, "from_record_key");
        pub const TO_RECORD_KEY: Column = Column::new(TABLE, "to_record_key");
        pub const DISPLAY_TEXT: Column = Column::new(TABLE, "display_text");
        pub const REFERENCE_TEXT: Column = Column::new(TABLE, "reference_text");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::FROM_RECORD_KEY,
        columns::TO_RECORD_KEY,
        columns::DISPLAY_TEXT,
        columns::REFERENCE_TEXT,
    ];
}

pub mod record_aliases {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("record_aliases");

    pub mod columns {
        use super::{Column, TABLE};

        pub const CANONICAL_RECORD_KEY: Column = Column::new(TABLE, "canonical_record_key");
        pub const ALIAS_TEXT: Column = Column::new(TABLE, "alias_text");
        pub const NORMALIZED_ALIAS: Column = Column::new(TABLE, "normalized_alias");
        pub const SOURCE_KIND: Column = Column::new(TABLE, "source_kind");
        pub const SOURCE_REF: Column = Column::new(TABLE, "source_ref");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::CANONICAL_RECORD_KEY,
        columns::ALIAS_TEXT,
        columns::NORMALIZED_ALIAS,
        columns::SOURCE_KIND,
        columns::SOURCE_REF,
    ];
}

pub mod remaster_links {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("remaster_links");

    pub mod columns {
        use super::{Column, TABLE};

        pub const REMASTER_RECORD_KEY: Column = Column::new(TABLE, "remaster_record_key");
        pub const LEGACY_RECORD_KEY: Column = Column::new(TABLE, "legacy_record_key");
        pub const SOURCE_KIND: Column = Column::new(TABLE, "source_kind");
        pub const SOURCE_REF: Column = Column::new(TABLE, "source_ref");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::REMASTER_RECORD_KEY,
        columns::LEGACY_RECORD_KEY,
        columns::SOURCE_KIND,
        columns::SOURCE_REF,
    ];
}

pub mod record_metrics {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("record_metrics");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const METRIC_DOMAIN: Column = Column::new(TABLE, "metric_domain");
        pub const METRIC_KEY: Column = Column::new(TABLE, "metric_key");
        pub const VALUE_TYPE: Column = Column::new(TABLE, "value_type");
        pub const NUMBER_VALUE: Column = Column::new(TABLE, "number_value");
        pub const TEXT_VALUE: Column = Column::new(TABLE, "text_value");
        pub const BOOL_VALUE: Column = Column::new(TABLE, "bool_value");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::METRIC_DOMAIN,
        columns::METRIC_KEY,
        columns::VALUE_TYPE,
        columns::NUMBER_VALUE,
        columns::TEXT_VALUE,
        columns::BOOL_VALUE,
    ];
}

pub mod metric_key_catalog {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("metric_key_catalog");

    pub mod columns {
        use super::{Column, TABLE};

        pub const METRIC_DOMAIN: Column = Column::new(TABLE, "metric_domain");
        pub const RECORD_FAMILY: Column = Column::new(TABLE, "record_family");
        pub const NAMESPACE_PREFIX: Column = Column::new(TABLE, "namespace_prefix");
        pub const METRIC_KEY: Column = Column::new(TABLE, "metric_key");
        pub const VALUE_TYPE: Column = Column::new(TABLE, "value_type");
        pub const CATALOG_COUNT: Column = Column::new(TABLE, "catalog_count");
        pub const NUMERIC_MIN: Column = Column::new(TABLE, "numeric_min");
        pub const NUMERIC_MAX: Column = Column::new(TABLE, "numeric_max");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::METRIC_DOMAIN,
        columns::RECORD_FAMILY,
        columns::NAMESPACE_PREFIX,
        columns::METRIC_KEY,
        columns::VALUE_TYPE,
        columns::CATALOG_COUNT,
        columns::NUMERIC_MIN,
        columns::NUMERIC_MAX,
    ];
}

pub mod metric_value_catalog {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("metric_value_catalog");

    pub mod columns {
        use super::{Column, TABLE};

        pub const METRIC_DOMAIN: Column = Column::new(TABLE, "metric_domain");
        pub const RECORD_FAMILY: Column = Column::new(TABLE, "record_family");
        pub const METRIC_KEY: Column = Column::new(TABLE, "metric_key");
        pub const VALUE: Column = Column::new(TABLE, "value");
        pub const CATALOG_COUNT: Column = Column::new(TABLE, "catalog_count");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::METRIC_DOMAIN,
        columns::RECORD_FAMILY,
        columns::METRIC_KEY,
        columns::VALUE,
        columns::CATALOG_COUNT,
    ];
}

pub mod actor_records {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("actor_records");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const SIZE: Column = Column::new(TABLE, "size");
        pub const LANGUAGES_JSON: Column = Column::new(TABLE, "languages_json");
        pub const SPEED_TYPES_JSON: Column = Column::new(TABLE, "speed_types_json");
        pub const SENSES_JSON: Column = Column::new(TABLE, "senses_json");
        pub const IMMUNITIES_JSON: Column = Column::new(TABLE, "immunities_json");
        pub const RESISTANCES_JSON: Column = Column::new(TABLE, "resistances_json");
        pub const WEAKNESSES_JSON: Column = Column::new(TABLE, "weaknesses_json");
        pub const DISABLE_TEXT: Column = Column::new(TABLE, "disable_text");
        pub const DISABLE_SKILLS_JSON: Column = Column::new(TABLE, "disable_skills_json");
        pub const IS_COMPLEX: Column = Column::new(TABLE, "is_complex");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::SIZE,
        columns::LANGUAGES_JSON,
        columns::SPEED_TYPES_JSON,
        columns::SENSES_JSON,
        columns::IMMUNITIES_JSON,
        columns::RESISTANCES_JSON,
        columns::WEAKNESSES_JSON,
        columns::DISABLE_TEXT,
        columns::DISABLE_SKILLS_JSON,
        columns::IS_COMPLEX,
    ];
}

pub mod item_records {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("item_records");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const SYSTEM_CATEGORY: Column = Column::new(TABLE, "system_category");
        pub const SYSTEM_BASE_ITEM: Column = Column::new(TABLE, "system_base_item");
        pub const SYSTEM_GROUP: Column = Column::new(TABLE, "system_group");
        pub const SYSTEM_USAGE: Column = Column::new(TABLE, "system_usage");
        pub const PRICE_CP: Column = Column::new(TABLE, "price_cp");
        pub const BULK_VALUE: Column = Column::new(TABLE, "bulk_value");
        pub const HANDS_REQUIREMENT: Column = Column::new(TABLE, "hands_requirement");
        pub const DAMAGE_TYPES_JSON: Column = Column::new(TABLE, "damage_types_json");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::SYSTEM_CATEGORY,
        columns::SYSTEM_BASE_ITEM,
        columns::SYSTEM_GROUP,
        columns::SYSTEM_USAGE,
        columns::PRICE_CP,
        columns::BULK_VALUE,
        columns::HANDS_REQUIREMENT,
        columns::DAMAGE_TYPES_JSON,
    ];
}

pub mod spell_records {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("spell_records");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const TRADITIONS_JSON: Column = Column::new(TABLE, "traditions_json");
        pub const SPELL_KINDS_JSON: Column = Column::new(TABLE, "spell_kinds_json");
        pub const RANGE_TEXT: Column = Column::new(TABLE, "range_text");
        pub const RANGE_VALUE: Column = Column::new(TABLE, "range_value");
        pub const TARGET_TEXT: Column = Column::new(TABLE, "target_text");
        pub const AREA_TYPE: Column = Column::new(TABLE, "area_type");
        pub const AREA_VALUE: Column = Column::new(TABLE, "area_value");
        pub const SAVE_TYPE: Column = Column::new(TABLE, "save_type");
        pub const SUSTAINED: Column = Column::new(TABLE, "sustained");
        pub const BASIC_SAVE: Column = Column::new(TABLE, "basic_save");
        pub const DAMAGE_TYPES_JSON: Column = Column::new(TABLE, "damage_types_json");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::TRADITIONS_JSON,
        columns::SPELL_KINDS_JSON,
        columns::RANGE_TEXT,
        columns::RANGE_VALUE,
        columns::TARGET_TEXT,
        columns::AREA_TYPE,
        columns::AREA_VALUE,
        columns::SAVE_TYPE,
        columns::SUSTAINED,
        columns::BASIC_SAVE,
        columns::DAMAGE_TYPES_JSON,
    ];
}

pub mod records_fts {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("records_fts");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const NAME: Column = Column::new(TABLE, "name");
        pub const SEARCH_TEXT_PROJECTION: Column = Column::new(TABLE, "search_text_projection");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::NAME,
        columns::SEARCH_TEXT_PROJECTION,
    ];
}

pub mod document_embedding_cache {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("document_embedding_cache");

    pub mod columns {
        use super::{Column, TABLE};

        pub const EMBEDDING_UNIT_KEY: Column = Column::new(TABLE, "embedding_unit_key");
        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const UNIT_KIND: Column = Column::new(TABLE, "unit_kind");
        pub const LABEL: Column = Column::new(TABLE, "label");
        pub const ORDINAL: Column = Column::new(TABLE, "ordinal");
        pub const SEMANTIC_INPUT_HASH: Column = Column::new(TABLE, "semantic_input_hash");
        pub const DIMENSIONS: Column = Column::new(TABLE, "dimensions");
        pub const VECTOR_BLOB: Column = Column::new(TABLE, "vector_blob");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::EMBEDDING_UNIT_KEY,
        columns::RECORD_KEY,
        columns::UNIT_KIND,
        columns::LABEL,
        columns::ORDINAL,
        columns::SEMANTIC_INPUT_HASH,
        columns::DIMENSIONS,
        columns::VECTOR_BLOB,
    ];
}

pub mod record_vector_index {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("record_vector_index");

    pub mod columns {
        use super::{Column, TABLE};

        pub const EMBEDDING: Column = Column::new(TABLE, "embedding");
    }

    pub const ALL_COLUMNS: &[Column] = &[columns::EMBEDDING];
}

pub const TABLE_ARTIFACT_METADATA: &str = artifact_metadata::TABLE.name();
pub const TABLE_PACKS: &str = packs::TABLE.name();
pub const TABLE_RECORDS: &str = records::TABLE.name();
pub const TABLE_RECORD_TRAITS: &str = record_traits::TABLE.name();
pub const TABLE_REFERENCE_EDGES: &str = reference_edges::TABLE.name();
pub const TABLE_RECORD_ALIASES: &str = record_aliases::TABLE.name();
pub const TABLE_REMASTER_LINKS: &str = remaster_links::TABLE.name();
pub const TABLE_RECORD_METRICS: &str = record_metrics::TABLE.name();
pub const TABLE_METRIC_KEY_CATALOG: &str = metric_key_catalog::TABLE.name();
pub const TABLE_METRIC_VALUE_CATALOG: &str = metric_value_catalog::TABLE.name();
pub const TABLE_ACTOR_RECORDS: &str = actor_records::TABLE.name();
pub const TABLE_ITEM_RECORDS: &str = item_records::TABLE.name();
pub const TABLE_SPELL_RECORDS: &str = spell_records::TABLE.name();
pub const TABLE_RECORDS_FTS: &str = records_fts::TABLE.name();
pub const TABLE_DOCUMENT_EMBEDDING_CACHE: &str = document_embedding_cache::TABLE.name();
pub const TABLE_RECORD_VECTOR_INDEX: &str = record_vector_index::TABLE.name();

pub const REQUIRED_TABLES: &[Table] = &[
    artifact_metadata::TABLE,
    packs::TABLE,
    records::TABLE,
    record_traits::TABLE,
    reference_edges::TABLE,
    record_aliases::TABLE,
    remaster_links::TABLE,
    record_metrics::TABLE,
    metric_key_catalog::TABLE,
    metric_value_catalog::TABLE,
    actor_records::TABLE,
    item_records::TABLE,
    spell_records::TABLE,
    records_fts::TABLE,
    document_embedding_cache::TABLE,
];

pub const RECORD_COLUMNS: &[Column] = records::ALL_COLUMNS;
pub const PERSISTED_RECORD_COLUMNS: &[Column] = records::PERSISTED_COLUMNS;
pub const RECORD_TRAIT_COLUMNS: &[Column] = record_traits::ALL_COLUMNS;
pub const REFERENCE_EDGE_COLUMNS: &[Column] = reference_edges::ALL_COLUMNS;
pub const RECORD_ALIAS_COLUMNS: &[Column] = record_aliases::ALL_COLUMNS;
pub const REMASTER_LINK_COLUMNS: &[Column] = remaster_links::ALL_COLUMNS;
pub const RECORD_METRIC_COLUMNS: &[Column] = record_metrics::ALL_COLUMNS;
pub const ACTOR_RECORD_COLUMNS: &[Column] = actor_records::ALL_COLUMNS;
pub const ITEM_RECORD_COLUMNS: &[Column] = item_records::ALL_COLUMNS;
pub const SPELL_RECORD_COLUMNS: &[Column] = spell_records::ALL_COLUMNS;
pub const RECORDS_FTS_COLUMNS: &[Column] = records_fts::ALL_COLUMNS;
pub const DOCUMENT_EMBEDDING_CACHE_COLUMNS: &[Column] = document_embedding_cache::ALL_COLUMNS;
pub const RECORD_VECTOR_INDEX_COLUMNS: &[Column] = record_vector_index::ALL_COLUMNS;

pub const REQUIRED_COLUMNS: &[(Table, &[Column])] = &[
    (artifact_metadata::TABLE, artifact_metadata::ALL_COLUMNS),
    (packs::TABLE, packs::ALL_COLUMNS),
    (records::TABLE, RECORD_COLUMNS),
    (record_traits::TABLE, RECORD_TRAIT_COLUMNS),
    (reference_edges::TABLE, REFERENCE_EDGE_COLUMNS),
    (record_aliases::TABLE, RECORD_ALIAS_COLUMNS),
    (remaster_links::TABLE, REMASTER_LINK_COLUMNS),
    (record_metrics::TABLE, RECORD_METRIC_COLUMNS),
    (metric_key_catalog::TABLE, metric_key_catalog::ALL_COLUMNS),
    (
        metric_value_catalog::TABLE,
        metric_value_catalog::ALL_COLUMNS,
    ),
    (actor_records::TABLE, ACTOR_RECORD_COLUMNS),
    (item_records::TABLE, ITEM_RECORD_COLUMNS),
    (spell_records::TABLE, SPELL_RECORD_COLUMNS),
    (records_fts::TABLE, RECORDS_FTS_COLUMNS),
    (
        document_embedding_cache::TABLE,
        DOCUMENT_EMBEDDING_CACHE_COLUMNS,
    ),
];
