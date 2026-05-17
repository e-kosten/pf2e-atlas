use super::{Column, Table};

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
