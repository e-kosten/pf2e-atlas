//! Executable-owned reference schema, composed into the workspace migration ledger.
pub const REFERENCE_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000001_create_artifact/up.sql");

/// Model-owned vector tables use a digest rather than interpolating request identifiers.
pub(crate) fn vector_table(
    model: &crate::SourceSearchProfileIdentity,
) -> Result<String, crate::IndexError> {
    Ok(format!(
        "semantic_vectors_{}",
        crate::codec::sha256(&serde_json::to_vec(model)?)
    ))
}
pub(crate) fn vector_ddl(
    model: &crate::SourceSearchProfileIdentity,
) -> Result<String, crate::IndexError> {
    Ok(format!(
        "CREATE VIRTUAL TABLE {} USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[{}] distance_metric=cosine)",
        vector_table(model)?,
        model.dimensions
    ))
}

pub const LEXICAL_ALIAS_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000002_lexical_aliases/up.sql");
/// Consumable query fields; existing rows are refreshed from retained typed snapshots.
pub const CONSUMABLE_FILTER_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000003_consumable_filters/up.sql");

pub const REFERENCE_NUMERIC_FILTER_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000004_reference_numeric_filters/up.sql");
pub const PHYSICAL_BASE_TYPE_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000005_physical_base_type/up.sql");

/// Complete native target membership and selected numerical checksums.
pub const NATIVE_TARGETS_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000008_native_targets/up.sql");
