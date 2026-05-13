mod contract_checks;
mod descriptors;
mod sql;
mod tables;

pub use contract_checks::{
    BOOLEAN_COLUMNS, BooleanColumn, REQUIRED_REFERENCES, RequiredReference,
    invalid_boolean_column_sql, orphan_reference_sql,
};
pub use descriptors::{Column, Table};
pub use sql::{
    CREATE_ARTIFACT_SCHEMA_SQL, actor_record_insert_sql, actor_record_select_sql, insert_sql,
    item_record_insert_sql, item_record_select_sql, ordered_select_sql,
    persisted_record_select_sql, record_alias_select_sql, record_insert_sql,
    record_metric_insert_sql, record_metric_select_sql, record_trait_insert_sql,
    records_fts_insert_sql, reference_edge_select_sql, remaster_link_select_sql,
    spell_record_insert_sql, spell_record_select_sql,
};
pub use tables::{
    ACTOR_RECORD_COLUMNS, ITEM_RECORD_COLUMNS, PERSISTED_RECORD_COLUMNS, RECORD_ALIAS_COLUMNS,
    RECORD_COLUMNS, RECORD_METRIC_COLUMNS, RECORD_TRAIT_COLUMNS, RECORDS_FTS_COLUMNS,
    REFERENCE_EDGE_COLUMNS, REMASTER_LINK_COLUMNS, REQUIRED_COLUMNS, REQUIRED_TABLES,
    SPELL_RECORD_COLUMNS, TABLE_ACTOR_RECORDS, TABLE_ARTIFACT_METADATA, TABLE_ITEM_RECORDS,
    TABLE_METRIC_KEY_CATALOG, TABLE_METRIC_VALUE_CATALOG, TABLE_PACKS, TABLE_RECORD_ALIASES,
    TABLE_RECORD_METRICS, TABLE_RECORD_TRAITS, TABLE_RECORDS, TABLE_RECORDS_FTS,
    TABLE_REFERENCE_EDGES, TABLE_REMASTER_LINKS, TABLE_SPELL_RECORDS, actor_records,
    artifact_metadata, item_records, metric_key_catalog, metric_value_catalog, packs,
    record_aliases, record_metrics, record_traits, records, records_fts, reference_edges,
    remaster_links, spell_records,
};
