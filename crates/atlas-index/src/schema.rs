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
