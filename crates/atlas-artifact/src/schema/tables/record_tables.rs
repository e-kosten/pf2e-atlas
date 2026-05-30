use super::{Column, Table};

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
        pub const PREREQUISITES_JSON: Column = Column::new(TABLE, "prerequisites_json");
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
        pub const DESCRIPTION_JSON: Column = Column::new(TABLE, "description_json");
        pub const BLURB_JSON: Column = Column::new(TABLE, "blurb_json");
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
        columns::PREREQUISITES_JSON,
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
        columns::DESCRIPTION_JSON,
        columns::BLURB_JSON,
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
        columns::RAW_JSON,
    ];

    pub const PERSISTED_COLUMNS: &[Column] = ALL_COLUMNS;
}

pub mod record_content {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("record_content");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const CONTENT_KEY: Column = Column::new(TABLE, "content_key");
        pub const ORDINAL: Column = Column::new(TABLE, "ordinal");
        pub const SOURCE_KIND: Column = Column::new(TABLE, "source_kind");
        pub const VISIBILITY: Column = Column::new(TABLE, "visibility");
        pub const CONTRIBUTES_TO_SEARCH: Column = Column::new(TABLE, "contributes_to_search");
        pub const CONTRIBUTES_TO_REFERENCES: Column =
            Column::new(TABLE, "contributes_to_references");
        pub const LABEL: Column = Column::new(TABLE, "label");
        pub const CONTENT_JSON: Column = Column::new(TABLE, "content_json");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::CONTENT_KEY,
        columns::ORDINAL,
        columns::SOURCE_KIND,
        columns::VISIBILITY,
        columns::CONTRIBUTES_TO_SEARCH,
        columns::CONTRIBUTES_TO_REFERENCES,
        columns::LABEL,
        columns::CONTENT_JSON,
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
