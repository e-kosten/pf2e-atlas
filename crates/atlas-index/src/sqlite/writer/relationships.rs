use rusqlite::Connection;

use crate::IndexWriteError;
use atlas_record::{
    ContentBlock, ContentDocument, ContentReference, ContentReferenceLocator, NormalizedRecord,
    RecordAlias, ReferenceEdge, RemasterLink, iter_content_references, render_plain_text,
};

use super::records::supplemental_content_key;

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

pub(super) fn write_reference_occurrences(
    connection: &Connection,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let mut insert_occurrence = connection
        .prepare(&atlas_artifact::schema::reference_occurrence_insert_sql())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    for record in records {
        if let Some(document) = &record.description {
            write_document_reference_occurrences(
                &mut insert_occurrence,
                record,
                "description",
                document,
            )?;
        }
        if let Some(document) = &record.blurb {
            write_document_reference_occurrences(
                &mut insert_occurrence,
                record,
                "blurb",
                document,
            )?;
        }
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            if supplemental.contributes_to_references {
                write_document_reference_occurrences(
                    &mut insert_occurrence,
                    record,
                    &supplemental_content_key(ordinal),
                    &supplemental.document,
                )?;
            }
        }
    }

    Ok(())
}

fn write_document_reference_occurrences(
    insert_occurrence: &mut rusqlite::Statement<'_>,
    record: &NormalizedRecord,
    content_key: &str,
    document: &ContentDocument,
) -> Result<(), IndexWriteError> {
    for reference in iter_content_references(document) {
        let Some(target_record_key) = &reference.resolved_key else {
            continue;
        };
        insert_occurrence
            .execute((
                record.key.to_string(),
                content_key,
                target_record_key.to_string(),
                content_reference_display_text(reference).as_deref(),
                content_reference_text(reference),
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

fn content_reference_text(reference: &ContentReference) -> String {
    match &reference.locator {
        ContentReferenceLocator::FoundryUuid { raw_target }
        | ContentReferenceLocator::Compendium { raw_target } => raw_target.clone(),
        ContentReferenceLocator::PackAndLocator { pack_name, locator } => {
            format!("{pack_name}:{locator}")
        }
        ContentReferenceLocator::Unknown { raw } => raw.clone(),
    }
}

fn content_reference_display_text(reference: &ContentReference) -> Option<String> {
    reference
        .label
        .as_ref()
        .map(|label| {
            render_plain_text(&ContentDocument::new(vec![ContentBlock::Paragraph {
                content: label.clone(),
            }]))
        })
        .filter(|label| !label.trim().is_empty())
}
