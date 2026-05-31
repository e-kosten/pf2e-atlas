use crate::artifact_metadata::artifact_metadata_keys;
use crate::artifact_storage::f32_vector_blob_len;
use crate::schema_inventory::TABLE_DOCUMENT_EMBEDDING_CACHE;
use atlas_embedding::EmbeddingUnitKind;
use rusqlite::Connection;
use std::collections::BTreeMap;

use crate::sql::{count_rows, count_sql};
use crate::{
    ArtifactValidationDiagnostic, ArtifactValidationFamily, IndexValidationError,
    artifact_validation::artifact_validation_diagnostic,
};

pub(super) fn validate_document_embedding_cache(
    connection: &Connection,
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let cache_rows = count_rows(connection, TABLE_DOCUMENT_EMBEDDING_CACHE)?;
    if cache_rows == 0 {
        return Ok(());
    }

    let default_visible_records = count_sql(
        connection,
        "SELECT COUNT(*) FROM records WHERE is_default_visible = 1",
    )?;
    let parent_units = count_sql(
        connection,
        "SELECT COUNT(*) FROM document_embedding_cache WHERE unit_kind = 'parent'",
    )?;
    if parent_units != default_visible_records {
        diagnostics.push(artifact_validation_diagnostic(
            ArtifactValidationFamily::Embedding,
            "document embedding cache parent unit count must match default-visible record count"
                .to_string(),
            Some("document_embedding_cache:default_visible_count".to_string()),
            Some(default_visible_records.to_string()),
            Some(parent_units.to_string()),
        ));
    }

    let expected_dimensions = metadata
        .get(artifact_metadata_keys::EMBEDDING_DIMENSIONS)
        .and_then(|value| value.parse::<usize>().ok());
    if let Some(expected_dimensions) = expected_dimensions {
        let invalid_dimensions = count_sql(
            connection,
            &format!(
                "SELECT COUNT(*) FROM document_embedding_cache WHERE dimensions <> {expected_dimensions}"
            ),
        )?;
        if invalid_dimensions > 0 {
            diagnostics.push(artifact_validation_diagnostic(
                ArtifactValidationFamily::Embedding,
                "document embedding cache dimensions must match embedding metadata".to_string(),
                Some("document_embedding_cache:dimensions".to_string()),
                Some(expected_dimensions.to_string()),
                Some(format!("{invalid_dimensions} invalid rows")),
            ));
        }

        let expected_bytes = f32_vector_blob_len(expected_dimensions);
        let invalid_blob_bytes = count_sql(
            connection,
            &format!(
                "SELECT COUNT(*) FROM document_embedding_cache WHERE length(vector_blob) <> {expected_bytes}"
            ),
        )?;
        if invalid_blob_bytes > 0 {
            diagnostics.push(artifact_validation_diagnostic(
                ArtifactValidationFamily::Embedding,
                "document embedding cache vector blob length must match embedding dimensions"
                    .to_string(),
                Some("document_embedding_cache:vector_blob".to_string()),
                Some(format!("{expected_bytes} bytes")),
                Some(format!("{invalid_blob_bytes} invalid rows")),
            ));
        }
    }

    for (key, sql, expected) in [
        (
            "document_embedding_cache:hidden_rows",
            "SELECT COUNT(*)
             FROM document_embedding_cache e
             JOIN records r ON r.record_key = e.record_key
             WHERE r.is_default_visible <> 1",
            "no hidden records in document embedding cache",
        ),
        (
            "document_embedding_cache:missing_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT record_key FROM records WHERE is_default_visible = 1
               EXCEPT
               SELECT record_key FROM document_embedding_cache WHERE unit_kind = 'parent'
             )",
            "every default-visible record has a parent document embedding",
        ),
        (
            "document_embedding_cache:stale_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT record_key FROM document_embedding_cache WHERE unit_kind = 'parent'
               EXCEPT
               SELECT record_key FROM records WHERE is_default_visible = 1
             )",
            "every parent document embedding row belongs to a default-visible record",
        ),
        (
            "document_embedding_cache:invalid_unit_kind",
            &valid_embedding_unit_kind_sql(),
            "every document embedding row has a known unit kind",
        ),
    ] {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(artifact_validation_diagnostic(
                ArtifactValidationFamily::Embedding,
                format!("document embedding cache coverage check `{key}` failed"),
                Some(key.to_string()),
                Some(expected.to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }

    Ok(())
}

fn valid_embedding_unit_kind_sql() -> String {
    format!(
        "SELECT COUNT(*)
         FROM document_embedding_cache
         WHERE unit_kind NOT IN ('{}', '{}', '{}')",
        EmbeddingUnitKind::Parent.as_str(),
        EmbeddingUnitKind::HeadingSection.as_str(),
        EmbeddingUnitKind::TitledOption.as_str()
    )
}
