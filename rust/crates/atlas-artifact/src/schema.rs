mod contract_checks;
mod descriptors;
mod sql;
mod tables;

pub use contract_checks::{
    BooleanColumn, REQUIRED_REFERENCES, RequiredReference, boolean_columns,
    invalid_boolean_column_sql, orphan_reference_sql,
};
pub use descriptors::{
    Column, ColumnCheck, ColumnDescriptor, ForeignKeyAction, SqlType, Table, TableConstraint,
    TableDescriptor, TableKind,
};
pub use sql::{
    actor_record_insert_sql, actor_record_select_sql, artifact_metadata_insert_sql,
    create_artifact_schema_sql, document_embedding_cache_insert_sql,
    filter_field_catalog_insert_sql, filter_numeric_catalog_insert_sql,
    filter_sample_catalog_insert_sql, filter_value_catalog_insert_sql, insert_or_ignore_sql,
    insert_sql, item_record_insert_sql, item_record_select_sql,
    metric_key_catalog_insert_select_sql, metric_value_catalog_insert_select_sql,
    ordered_select_sql, pack_insert_sql, persisted_record_select_sql, record_alias_insert_sql,
    record_alias_select_sql, record_content_insert_sql, record_content_select_sql,
    record_insert_sql, record_metric_insert_sql, record_metric_select_sql, record_trait_insert_sql,
    record_vector_index_create_sql, record_vector_index_insert_sql, records_fts_insert_sql,
    reference_edge_insert_sql, reference_edge_select_sql, remaster_link_insert_sql,
    remaster_link_select_sql, spell_record_insert_sql, spell_record_select_sql,
};
pub use tables::{
    ACTOR_RECORD_COLUMNS, DOCUMENT_EMBEDDING_CACHE_COLUMNS, FILTER_FIELD_CATALOG_COLUMNS,
    FILTER_NUMERIC_CATALOG_COLUMNS, FILTER_SAMPLE_CATALOG_COLUMNS, FILTER_VALUE_CATALOG_COLUMNS,
    ITEM_RECORD_COLUMNS, PERSISTED_RECORD_COLUMNS, RECORD_ALIAS_COLUMNS, RECORD_COLUMNS,
    RECORD_CONTENT_COLUMNS, RECORD_METRIC_COLUMNS, RECORD_TRAIT_COLUMNS,
    RECORD_VECTOR_INDEX_COLUMNS, RECORDS_FTS_COLUMNS, REFERENCE_EDGE_COLUMNS,
    REMASTER_LINK_COLUMNS, SPELL_RECORD_COLUMNS, TABLE_ACTOR_RECORDS, TABLE_ARTIFACT_METADATA,
    TABLE_DESCRIPTORS, TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_FILTER_FIELD_CATALOG,
    TABLE_FILTER_NUMERIC_CATALOG, TABLE_FILTER_SAMPLE_CATALOG, TABLE_FILTER_VALUE_CATALOG,
    TABLE_ITEM_RECORDS, TABLE_METRIC_KEY_CATALOG, TABLE_METRIC_VALUE_CATALOG, TABLE_PACKS,
    TABLE_RECORD_ALIASES, TABLE_RECORD_CONTENT, TABLE_RECORD_METRICS, TABLE_RECORD_TRAITS,
    TABLE_RECORD_VECTOR_INDEX, TABLE_RECORDS, TABLE_RECORDS_FTS, TABLE_REFERENCE_EDGES,
    TABLE_REMASTER_LINKS, TABLE_SPELL_RECORDS, actor_records, artifact_metadata,
    document_embedding_cache, filter_field_catalog, filter_numeric_catalog, filter_sample_catalog,
    filter_value_catalog, item_records, metric_key_catalog, metric_value_catalog, packs,
    record_aliases, record_content, record_metrics, record_traits, record_vector_index, records,
    records_fts, reference_edges, remaster_links, required_columns, required_tables, spell_records,
    table_descriptor,
};
