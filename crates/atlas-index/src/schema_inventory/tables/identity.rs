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
