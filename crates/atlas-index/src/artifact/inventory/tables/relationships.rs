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
        pub const OCCURRENCE_ORDINAL: Column = Column::new(TABLE, "occurrence_ordinal");
        pub const TARGET_RECORD_KEY: Column = Column::new(TABLE, "target_record_key");
        pub const SOURCE_KIND: Column = Column::new(TABLE, "source_kind");
        pub const VISIBILITY: Column = Column::new(TABLE, "visibility");
        pub const DISPLAY_TEXT: Column = Column::new(TABLE, "display_text");
        pub const REFERENCE_TEXT: Column = Column::new(TABLE, "reference_text");
        pub const RELATION_KIND: Column = Column::new(TABLE, "relation_kind");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::CONTENT_KEY,
        columns::OCCURRENCE_ORDINAL,
        columns::TARGET_RECORD_KEY,
        columns::SOURCE_KIND,
        columns::VISIBILITY,
        columns::DISPLAY_TEXT,
        columns::REFERENCE_TEXT,
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
