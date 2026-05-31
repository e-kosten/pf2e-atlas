use super::{Column, Table};

pub mod filter_field_catalog {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("filter_field_catalog");

    pub mod columns {
        use super::{Column, TABLE};

        pub const FIELD: Column = Column::new(TABLE, "field");
        pub const RECORD_FAMILY: Column = Column::new(TABLE, "record_family");
        pub const FIELD_TYPE: Column = Column::new(TABLE, "field_type");
        pub const FIELD_GROUP: Column = Column::new(TABLE, "field_group");
        pub const VALUE_POLICY: Column = Column::new(TABLE, "value_policy");
        pub const OPERATORS_JSON: Column = Column::new(TABLE, "operators_json");
        pub const CLI_FLAGS_JSON: Column = Column::new(TABLE, "cli_flags_json");
        pub const APPLICABLE_FAMILIES_JSON: Column = Column::new(TABLE, "applicable_families_json");
        pub const VALUE_COUNT: Column = Column::new(TABLE, "value_count");
        pub const MATCHING_RECORD_COUNT: Column = Column::new(TABLE, "matching_record_count");
        pub const NULL_COUNT: Column = Column::new(TABLE, "null_count");
        pub const DISTINCT_COUNT: Column = Column::new(TABLE, "distinct_count");
        pub const SINGLETON_COUNT: Column = Column::new(TABLE, "singleton_count");
        pub const SINGLETON_RATIO: Column = Column::new(TABLE, "singleton_ratio");
        pub const OBSERVATION_SINGLETON_RATIO: Column =
            Column::new(TABLE, "observation_singleton_ratio");
        pub const POLICY_REASON: Column = Column::new(TABLE, "policy_reason");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::FIELD,
        columns::RECORD_FAMILY,
        columns::FIELD_TYPE,
        columns::FIELD_GROUP,
        columns::VALUE_POLICY,
        columns::OPERATORS_JSON,
        columns::CLI_FLAGS_JSON,
        columns::APPLICABLE_FAMILIES_JSON,
        columns::VALUE_COUNT,
        columns::MATCHING_RECORD_COUNT,
        columns::NULL_COUNT,
        columns::DISTINCT_COUNT,
        columns::SINGLETON_COUNT,
        columns::SINGLETON_RATIO,
        columns::OBSERVATION_SINGLETON_RATIO,
        columns::POLICY_REASON,
    ];
}

pub mod filter_value_catalog {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("filter_value_catalog");

    pub mod columns {
        use super::{Column, TABLE};

        pub const FIELD: Column = Column::new(TABLE, "field");
        pub const RECORD_FAMILY: Column = Column::new(TABLE, "record_family");
        pub const VALUE: Column = Column::new(TABLE, "value");
        pub const CATALOG_COUNT: Column = Column::new(TABLE, "catalog_count");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::FIELD,
        columns::RECORD_FAMILY,
        columns::VALUE,
        columns::CATALOG_COUNT,
    ];
}

pub mod filter_sample_catalog {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("filter_sample_catalog");

    pub mod columns {
        use super::{Column, TABLE};

        pub const FIELD: Column = Column::new(TABLE, "field");
        pub const RECORD_FAMILY: Column = Column::new(TABLE, "record_family");
        pub const VALUE: Column = Column::new(TABLE, "value");
        pub const CATALOG_COUNT: Column = Column::new(TABLE, "catalog_count");
        pub const SAMPLE_RANK: Column = Column::new(TABLE, "sample_rank");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::FIELD,
        columns::RECORD_FAMILY,
        columns::VALUE,
        columns::CATALOG_COUNT,
        columns::SAMPLE_RANK,
    ];
}

pub mod filter_numeric_catalog {
    use super::{Column, Table};

    pub const TABLE: Table = Table::new("filter_numeric_catalog");

    pub mod columns {
        use super::{Column, TABLE};

        pub const FIELD: Column = Column::new(TABLE, "field");
        pub const RECORD_FAMILY: Column = Column::new(TABLE, "record_family");
        pub const METRIC_DOMAIN: Column = Column::new(TABLE, "metric_domain");
        pub const METRIC_KEY: Column = Column::new(TABLE, "metric_key");
        pub const CATALOG_COUNT: Column = Column::new(TABLE, "catalog_count");
        pub const NULL_COUNT: Column = Column::new(TABLE, "null_count");
        pub const MIN: Column = Column::new(TABLE, "min");
        pub const P05: Column = Column::new(TABLE, "p05");
        pub const P25: Column = Column::new(TABLE, "p25");
        pub const P50: Column = Column::new(TABLE, "p50");
        pub const MEAN: Column = Column::new(TABLE, "mean");
        pub const P75: Column = Column::new(TABLE, "p75");
        pub const P95: Column = Column::new(TABLE, "p95");
        pub const MAX: Column = Column::new(TABLE, "max");
    }

    pub const ALL_COLUMNS: &[Column] = &[
        columns::FIELD,
        columns::RECORD_FAMILY,
        columns::METRIC_DOMAIN,
        columns::METRIC_KEY,
        columns::CATALOG_COUNT,
        columns::NULL_COUNT,
        columns::MIN,
        columns::P05,
        columns::P25,
        columns::P50,
        columns::MEAN,
        columns::P75,
        columns::P95,
        columns::MAX,
    ];
}
