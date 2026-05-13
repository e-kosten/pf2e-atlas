use rusqlite::Connection;

use crate::{IngestError, RecordAlias, ReferenceEdge, RemasterLink};

pub(super) fn write_reference_edges(
    connection: &Connection,
    references: &[ReferenceEdge],
) -> Result<(), IngestError> {
    let mut insert_reference = connection
        .prepare(
            "INSERT OR IGNORE INTO reference_edges (
              from_record_key, to_record_key, display_text, reference_text
            ) VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for reference in references {
        insert_reference
            .execute((
                reference.from_record_key.to_string(),
                reference.to_record_key.to_string(),
                reference.display_text.as_deref(),
                reference.reference_text.as_str(),
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn write_record_aliases(
    connection: &Connection,
    aliases: &[RecordAlias],
) -> Result<(), IngestError> {
    let mut insert_alias = connection
        .prepare(
            "INSERT OR IGNORE INTO record_aliases (
              canonical_record_key, alias_text, normalized_alias, source_kind, source_ref
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for alias in aliases {
        insert_alias
            .execute((
                alias.canonical_record_key.to_string(),
                alias.alias_text.as_str(),
                alias.normalized_alias.as_str(),
                alias.source.as_str(),
                alias.source_ref.as_str(),
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn write_remaster_links(
    connection: &Connection,
    remaster_links: &[RemasterLink],
) -> Result<(), IngestError> {
    let mut insert_link = connection
        .prepare(
            "INSERT OR IGNORE INTO remaster_links (
              remaster_record_key, legacy_record_key, source_kind, source_ref
            ) VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for link in remaster_links {
        insert_link
            .execute((
                link.remaster_record_key.to_string(),
                link.legacy_record_key.to_string(),
                link.source.as_str(),
                link.source_ref.as_str(),
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}
