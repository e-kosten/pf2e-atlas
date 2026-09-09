use super::{Column, Table};

pub mod reference_edges {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("reference_edges");

    pub mod columns {
        use super::{Column, TABLE};

        pub const FROM_RECORD_KEY: Column = Column::new(TABLE, "from_record_key");
        pub const TO_RECORD_KEY: Column = Column::new(TABLE, "to_record_key");
        pub const DISPLAY_TEXT: Column = Column::new(TABLE, "display_text");
        pub const REFERENCE_TEXT: Column = Column::new(TABLE, "reference_text");
        pub const RELATION_KIND: Column = Column::new(TABLE, "relation_kind");
        pub const SOURCE_KIND: Column = Column::new(TABLE, "source_kind");
        pub const VISIBILITY: Column = Column::new(TABLE, "visibility");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::FROM_RECORD_KEY,
        columns::TO_RECORD_KEY,
        columns::DISPLAY_TEXT,
        columns::REFERENCE_TEXT,
        columns::RELATION_KIND,
        columns::SOURCE_KIND,
        columns::VISIBILITY,
    ];
}

pub mod reference_occurrences {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("reference_occurrences");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const CONTENT_KEY: Column = Column::new(TABLE, "content_key");
        pub const CONTENT_AUTHORED_ORDER: Column = Column::new(TABLE, "content_authored_order");
        pub const OCCURRENCE_ORDINAL: Column = Column::new(TABLE, "occurrence_ordinal");
        pub const OWNER_KIND: Column = Column::new(TABLE, "owner_kind");
        pub const OWNER_RECORD_KEY: Column = Column::new(TABLE, "owner_record_key");
        pub const OWNER_ENTITY_ID: Column = Column::new(TABLE, "owner_entity_id");
        pub const OWNER_OCCURRENCE_ID: Column = Column::new(TABLE, "owner_occurrence_id");
        pub const OWNER_OCCURRENCE_AUTHORED_ORDER: Column =
            Column::new(TABLE, "owner_occurrence_authored_order");
        pub const OWNER_HAZARD_ENTITY_ID: Column = Column::new(TABLE, "owner_hazard_entity_id");
        pub const OWNER_HAZARD_OCCURRENCE_ID: Column =
            Column::new(TABLE, "owner_hazard_occurrence_id");
        pub const OWNER_HAZARD_OCCURRENCE_AUTHORED_ORDER: Column =
            Column::new(TABLE, "owner_hazard_occurrence_authored_order");
        pub const OWNER_CONSUMABLE_OCCURRENCE_ID: Column =
            Column::new(TABLE, "owner_consumable_occurrence_id");
        pub const OWNER_CONSUMABLE_OCCURRENCE_AUTHORED_ORDER: Column =
            Column::new(TABLE, "owner_consumable_occurrence_authored_order");
        pub const ROLE: Column = Column::new(TABLE, "role");
        pub const ORIGIN_JSON: Column = Column::new(TABLE, "origin_json");
        pub const VISIBILITY: Column = Column::new(TABLE, "visibility");
        pub const PROVENANCE_JSON: Column = Column::new(TABLE, "provenance_json");
        pub const TARGET_KIND: Column = Column::new(TABLE, "target_kind");
        pub const TARGET_RECORD_KEY: Column = Column::new(TABLE, "target_record_key");
        pub const TARGET_JSON: Column = Column::new(TABLE, "target_json");
        pub const LABEL: Column = Column::new(TABLE, "label");
        pub const RELATION_KIND: Column = Column::new(TABLE, "relation_kind");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::CONTENT_KEY,
        columns::CONTENT_AUTHORED_ORDER,
        columns::OCCURRENCE_ORDINAL,
        columns::OWNER_KIND,
        columns::OWNER_RECORD_KEY,
        columns::OWNER_ENTITY_ID,
        columns::OWNER_OCCURRENCE_ID,
        columns::OWNER_OCCURRENCE_AUTHORED_ORDER,
        columns::OWNER_HAZARD_ENTITY_ID,
        columns::OWNER_HAZARD_OCCURRENCE_ID,
        columns::OWNER_HAZARD_OCCURRENCE_AUTHORED_ORDER,
        columns::OWNER_CONSUMABLE_OCCURRENCE_ID,
        columns::OWNER_CONSUMABLE_OCCURRENCE_AUTHORED_ORDER,
        columns::ROLE,
        columns::ORIGIN_JSON,
        columns::VISIBILITY,
        columns::PROVENANCE_JSON,
        columns::TARGET_KIND,
        columns::TARGET_RECORD_KEY,
        columns::TARGET_JSON,
        columns::LABEL,
        columns::RELATION_KIND,
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
