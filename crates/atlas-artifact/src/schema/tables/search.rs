use super::{Column, Table};

pub mod records_fts {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("records_fts");

    pub mod columns {
        use super::{Column, TABLE};

        pub const RECORD_KEY: Column = Column::new(TABLE, "record_key");
        pub const TITLE: Column = Column::new(TABLE, "title");
        pub const ALIASES: Column = Column::new(TABLE, "aliases");
        pub const TRAITS: Column = Column::new(TABLE, "traits");
        pub const TAXONOMY_TERMS: Column = Column::new(TABLE, "taxonomy_terms");
        pub const CONSTRAINT_TERMS: Column = Column::new(TABLE, "constraint_terms");
        pub const MECHANIC_TERMS: Column = Column::new(TABLE, "mechanic_terms");
        pub const SOURCE_TERMS: Column = Column::new(TABLE, "source_terms");
        pub const METRIC_TERMS: Column = Column::new(TABLE, "metric_terms");
        pub const HEADINGS: Column = Column::new(TABLE, "headings");
        pub const BODY: Column = Column::new(TABLE, "body");
        pub const FACTS: Column = Column::new(TABLE, "facts");
        pub const REFERENCE_TERMS: Column = Column::new(TABLE, "reference_terms");
        pub const EMBEDDED_CONTENT: Column = Column::new(TABLE, "embedded_content");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::RECORD_KEY,
        columns::TITLE,
        columns::ALIASES,
        columns::TRAITS,
        columns::TAXONOMY_TERMS,
        columns::CONSTRAINT_TERMS,
        columns::MECHANIC_TERMS,
        columns::SOURCE_TERMS,
        columns::METRIC_TERMS,
        columns::HEADINGS,
        columns::BODY,
        columns::FACTS,
        columns::REFERENCE_TERMS,
        columns::EMBEDDED_CONTENT,
    ];
}
