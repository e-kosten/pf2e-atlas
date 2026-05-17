use super::{Column, Table};

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
