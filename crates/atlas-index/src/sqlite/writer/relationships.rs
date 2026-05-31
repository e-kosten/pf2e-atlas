use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::IndexWriteError;
use atlas_record::{
    ContentBlock, ContentDocument, ContentReference, ContentReferenceLocator, ContentSourceKind,
    ContentVisibility, NormalizedRecord, RecordAlias, ReferenceEdge, RemasterLink,
    iter_content_references, render_plain_text,
};

use super::models::{RecordAliasRow, ReferenceEdgeRow, ReferenceOccurrenceRow, RemasterLinkRow};
use super::records::supplemental_content_key;

pub(super) fn write_reference_edges(
    connection: &mut SqliteConnection,
    references: &[ReferenceEdge],
) -> Result<(), IndexWriteError> {
    let rows = references
        .iter()
        .map(|reference| ReferenceEdgeRow {
            from_record_key: reference.from_record_key.to_string(),
            to_record_key: reference.to_record_key.to_string(),
            display_text: reference.display_text.clone(),
            reference_text: reference.reference_text.clone(),
            source_kind: reference.source_kind.as_str().to_string(),
            visibility: reference.visibility.as_str().to_string(),
        })
        .collect::<Vec<_>>();
    if !rows.is_empty() {
        diesel::insert_or_ignore_into(crate::schema::reference_edges::table)
            .values(&rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn write_reference_occurrences(
    connection: &mut SqliteConnection,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let mut rows = Vec::new();
    for record in records {
        if let Some(document) = &record.description {
            collect_document_reference_occurrences(
                &mut rows,
                record,
                "description",
                ContentSourceKind::Description,
                ContentVisibility::Public,
                document,
            )?;
        }
        if let Some(document) = &record.blurb {
            collect_document_reference_occurrences(
                &mut rows,
                record,
                "blurb",
                ContentSourceKind::Blurb,
                ContentVisibility::Public,
                document,
            )?;
        }
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            if supplemental.contributes_to_references {
                collect_document_reference_occurrences(
                    &mut rows,
                    record,
                    &supplemental_content_key(ordinal),
                    supplemental.source_kind,
                    supplemental.visibility,
                    &supplemental.document,
                )?;
            }
        }
    }
    if !rows.is_empty() {
        diesel::insert_or_ignore_into(crate::schema::reference_occurrences::table)
            .values(&rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn collect_document_reference_occurrences(
    rows: &mut Vec<ReferenceOccurrenceRow>,
    record: &NormalizedRecord,
    content_key: &str,
    source_kind: ContentSourceKind,
    visibility: ContentVisibility,
    document: &ContentDocument,
) -> Result<(), IndexWriteError> {
    let mut occurrence_ordinal = 0_i64;
    for reference in iter_content_references(document) {
        let Some(target_record_key) = &reference.resolved_key else {
            continue;
        };
        rows.push(ReferenceOccurrenceRow {
            record_key: record.key.to_string(),
            content_key: content_key.to_string(),
            occurrence_ordinal,
            target_record_key: target_record_key.to_string(),
            source_kind: source_kind.as_str().to_string(),
            visibility: visibility.as_str().to_string(),
            display_text: content_reference_display_text(reference),
            reference_text: content_reference_text(reference),
        });
        occurrence_ordinal = occurrence_ordinal.checked_add(1).ok_or_else(|| {
            IndexWriteError::WriteFailed(
                "reference occurrence ordinal does not fit in i64".to_string(),
            )
        })?;
    }
    Ok(())
}

pub(super) fn write_record_aliases(
    connection: &mut SqliteConnection,
    aliases: &[RecordAlias],
) -> Result<(), IndexWriteError> {
    let rows = aliases
        .iter()
        .map(|alias| RecordAliasRow {
            canonical_record_key: alias.canonical_record_key.to_string(),
            alias_text: alias.alias_text.clone(),
            normalized_alias: alias.normalized_alias.clone(),
            source_kind: alias.source.as_str().to_string(),
            source_ref: alias.source_ref.clone(),
        })
        .collect::<Vec<_>>();
    if !rows.is_empty() {
        diesel::insert_or_ignore_into(crate::schema::record_aliases::table)
            .values(&rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn write_remaster_links(
    connection: &mut SqliteConnection,
    remaster_links: &[RemasterLink],
) -> Result<(), IndexWriteError> {
    let rows = remaster_links
        .iter()
        .map(|link| RemasterLinkRow {
            remaster_record_key: link.remaster_record_key.to_string(),
            legacy_record_key: link.legacy_record_key.to_string(),
            source_kind: link.source.as_str().to_string(),
            source_ref: link.source_ref.clone(),
        })
        .collect::<Vec<_>>();
    if !rows.is_empty() {
        diesel::insert_or_ignore_into(crate::schema::remaster_links::table)
            .values(&rows)
            .execute(connection)
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
