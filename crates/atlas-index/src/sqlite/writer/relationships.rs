use rusqlite::Connection;

use crate::IndexWriteError;
use atlas_record::{RecordAlias, ReferenceEdge, RemasterLink};

pub(super) fn write_reference_edges(
    connection: &Connection,
    references: &[ReferenceEdge],
) -> Result<(), IndexWriteError> {
    let mut insert_reference = connection
        .prepare(&atlas_artifact::schema::reference_edge_insert_sql())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    for reference in references {
        insert_reference
            .execute((
                reference.from_record_key.to_string(),
                reference.to_record_key.to_string(),
                reference.display_text.as_deref(),
                reference.reference_text.as_str(),
                reference.source_kind.as_str(),
                reference.visibility.as_str(),
            ))
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn write_record_aliases(
    connection: &Connection,
    aliases: &[RecordAlias],
) -> Result<(), IndexWriteError> {
    let mut insert_alias = connection
        .prepare(&atlas_artifact::schema::record_alias_insert_sql())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    for alias in aliases {
        insert_alias
            .execute((
                alias.canonical_record_key.to_string(),
                alias.alias_text.as_str(),
                alias.normalized_alias.as_str(),
                alias.source.as_str(),
                alias.source_ref.as_str(),
            ))
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn write_remaster_links(
    connection: &Connection,
    remaster_links: &[RemasterLink],
) -> Result<(), IndexWriteError> {
    let mut insert_link = connection
        .prepare(&atlas_artifact::schema::remaster_link_insert_sql())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    for link in remaster_links {
        insert_link
            .execute((
                link.remaster_record_key.to_string(),
                link.legacy_record_key.to_string(),
                link.source.as_str(),
                link.source_ref.as_str(),
            ))
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}
