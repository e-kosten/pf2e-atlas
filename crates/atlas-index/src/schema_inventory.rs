//! Validation inventory for the SQLite artifact schema.
//!
//! Diesel migrations are the physical schema source of truth. This module is
//! retained only for artifact validation requirements and the few raw-SQL
//! paths that still need stable table and column names.

mod contract_checks;
mod names;
mod tables;

pub use contract_checks::{
    REQUIRED_REFERENCES, boolean_columns, invalid_boolean_column_sql, orphan_reference_sql,
};
pub use names::{Column, Table};
pub use tables::{
    TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_METRIC_VALUE_CATALOG, TABLE_PACKS, TABLE_RECORD_ALIASES,
    TABLE_RECORD_VECTOR_INDEX, TABLE_RECORDS, TABLE_RECORDS_FTS, TABLE_REFERENCE_EDGES,
    TABLE_REFERENCE_OCCURRENCES, TABLE_REMASTER_LINKS, actor_records, document_embedding_cache,
    item_records, packs, record_aliases, record_content, record_metrics, record_traits, records,
    reference_edges, reference_occurrences, remaster_links, required_columns, required_tables,
    spell_records,
};
