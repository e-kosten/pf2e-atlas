//! Executable-owned reference schema, composed into the workspace migration ledger.
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
