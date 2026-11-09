//! Executable-owned reference schema, composed into the workspace migration ledger.
#[cfg(any(test, feature = "test-support"))]
pub const REFERENCE_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000001_create_artifact/up.sql");

/// Model-owned vector tables use a digest rather than interpolating request identifiers.
#[cfg(any(test, feature = "test-support"))]
pub(crate) fn vector_table(
    model: &crate::SourceSearchProfileIdentity,
) -> Result<String, crate::IndexError> {
    Ok(format!(
        "semantic_vectors_{}",
        crate::codec::sha256(&serde_json::to_vec(model)?)
    ))
}
#[cfg(any(test, feature = "test-support"))]
pub(crate) fn vector_ddl(
    model: &crate::SourceSearchProfileIdentity,
) -> Result<String, crate::IndexError> {
    Ok(format!(
        "CREATE VIRTUAL TABLE {} USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[{}] distance_metric=cosine)",
        vector_table(model)?,
        model.dimensions
    ))
}

#[cfg(any(test, feature = "test-support"))]
pub const LEXICAL_ALIAS_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000002_lexical_aliases/up.sql");
/// Consumable query fields; existing rows are refreshed from retained typed snapshots.
#[cfg(any(test, feature = "test-support"))]
pub const CONSUMABLE_FILTER_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000003_consumable_filters/up.sql");

#[cfg(any(test, feature = "test-support"))]
pub const REFERENCE_NUMERIC_FILTER_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000004_reference_numeric_filters/up.sql");
#[cfg(any(test, feature = "test-support"))]
pub const PHYSICAL_BASE_TYPE_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000005_physical_base_type/up.sql");

/// Current numerical authority; historical migration10 keeps vector_table above.
pub(crate) fn native_vector_table(
    model: &crate::SourceSearchProfileIdentity,
) -> Result<String, crate::IndexError> {
    Ok(format!(
        "native_vectors_{}",
        crate::codec::sha256(&serde_json::to_vec(model)?)
    ))
}
pub(crate) fn native_vector_ddl(
    model: &crate::SourceSearchProfileIdentity,
) -> Result<String, crate::IndexError> {
    Ok(format!(
        "CREATE VIRTUAL TABLE {} USING vec0(input_id INTEGER PRIMARY KEY,embedding FLOAT[{}] distance_metric=cosine)",
        native_vector_table(model)?,
        model.dimensions
    ))
}
