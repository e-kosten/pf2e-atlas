use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::IndexWriteError;
use atlas_record::{
    AtlasRecord, ContentSourceKind, ContentVisibility, FoundryLink, FoundryLinkBehavior,
    RecordAlias, ReferenceEdge, ReferenceRelationKind, RemasterLink, RichDocument, RichLinkTarget,
    iter_foundry_links, render_plain_text,
};

use super::models::{RecordAliasRow, ReferenceEdgeRow, ReferenceOccurrenceRow, RemasterLinkRow};
use super::records::allocated_content_keys;

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
            relation_kind: reference.relation_kind.as_str().to_string(),
            source_kind: reference.source_kind.as_str().to_string(),
            visibility: reference.visibility.as_str().to_string(),
        })
        .collect::<Vec<_>>();
    for rows in rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_or_ignore_into(crate::schema::reference_edges::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn write_reference_occurrences(
    connection: &mut SqliteConnection,
    records: &[AtlasRecord],
    canonical_record_keys: &std::collections::BTreeSet<String>,
) -> Result<(), IndexWriteError> {
    let mut rows = Vec::new();
    for record in records {
        if canonical_record_keys.contains(&record.identity.key.to_string()) {
            continue;
        }
        let mut content_inputs = Vec::new();
        for (ordinal, content) in record.content.documents.iter().enumerate() {
            let content_json = serde_json::to_string(&content.document)
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
            content_inputs.push((ordinal, content, content_json));
        }
        let content_keys = allocated_content_keys(&content_inputs);
        for ((ordinal, content, _), content_key) in content_inputs.into_iter().zip(content_keys) {
            if !content.contributes_to_reference_occurrences() {
                continue;
            }
            collect_document_reference_occurrences(
                &mut rows,
                record,
                &content_key,
                ordinal as i64,
                content.source_kind,
                content.visibility(),
                &content.document,
            )?;
        }
    }
    for rows in rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_or_ignore_into(crate::schema::reference_occurrences::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn collect_document_reference_occurrences(
    rows: &mut Vec<ReferenceOccurrenceRow>,
    record: &AtlasRecord,
    content_key: &str,
    content_authored_order: i64,
    source_kind: ContentSourceKind,
    visibility: ContentVisibility,
    document: &RichDocument,
) -> Result<(), IndexWriteError> {
    let mut occurrence_ordinal = 0_i64;
    for link in iter_foundry_links(document) {
        let Some(target_record_key) = link.target.record_key() else {
            continue;
        };
        rows.push(ReferenceOccurrenceRow {
            record_key: record.identity.key.to_string(),
            content_key: content_key.to_string(),
            content_authored_order,
            occurrence_ordinal,
            owner_kind: "record".to_string(),
            owner_record_key: Some(record.identity.key.to_string()),
            owner_entity_id: None,
            owner_occurrence_id: None,
            owner_occurrence_authored_order: None,
            owner_hazard_entity_id: None,
            owner_hazard_occurrence_id: None,
            owner_hazard_occurrence_authored_order: None,
            role: legacy_content_role(source_kind).to_string(),
            origin_json: crate::artifact::canonical_json::encode(
                &atlas_record::ContentOrigin::RecordField {
                    source_kind,
                    relative_source_path: source_kind.as_str().to_string(),
                },
            )
            .map_err(IndexWriteError::WriteFailed)?,
            visibility: visibility.as_str().to_string(),
            provenance_json: crate::artifact::canonical_json::encode(
                &atlas_record::ContentProvenance {
                    source_record_key: record.identity.key.clone(),
                    relative_source_path: source_kind.as_str().to_string(),
                    field_or_pointer_family: source_kind.as_str().to_string(),
                    nested_source_id: None,
                    authored_ordinal_or_range: None,
                    authored_label: None,
                },
            )
            .map_err(IndexWriteError::WriteFailed)?,
            target_kind: "record".to_string(),
            target_record_key: Some(target_record_key.to_string()),
            target_json: serde_json::to_string(&link.target)
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
            label: foundry_link_display_text(link),
            relation_kind: foundry_link_relation_kind(link).as_str().to_string(),
        });
        occurrence_ordinal = occurrence_ordinal.checked_add(1).ok_or_else(|| {
            IndexWriteError::WriteFailed(
                "reference occurrence ordinal does not fit in i64".to_string(),
            )
        })?;
    }
    Ok(())
}

fn legacy_content_role(source_kind: ContentSourceKind) -> &'static str {
    match source_kind {
        ContentSourceKind::Description => "primary_description",
        ContentSourceKind::Blurb => "summary",
        ContentSourceKind::EmbeddedItemDescription
        | ContentSourceKind::EmbeddedSpellDescription => "embedded_capability",
        ContentSourceKind::GeneratedAffliction => "generated_narrative",
        _ => "supplemental_rules",
    }
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
    for rows in rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_or_ignore_into(crate::schema::record_aliases::table)
            .values(rows)
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
    for rows in rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_or_ignore_into(crate::schema::remaster_links::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn foundry_link_display_text(link: &FoundryLink) -> Option<String> {
    link.label
        .as_ref()
        .map(|label| render_plain_text(&RichDocument::new(label.clone())))
        .filter(|label| !label.trim().is_empty())
        .or_else(|| match &link.target {
            RichLinkTarget::Record { name, .. } => Some(name.clone()),
            RichLinkTarget::LocalContent { label, .. } | RichLinkTarget::External { label, .. } => {
                label.clone()
            }
            RichLinkTarget::Unresolved { fallback_label, .. } => Some(fallback_label.clone()),
        })
}

fn foundry_link_relation_kind(link: &FoundryLink) -> ReferenceRelationKind {
    match link.behavior {
        FoundryLinkBehavior::Reference => ReferenceRelationKind::Reference,
        FoundryLinkBehavior::Embed { .. } => ReferenceRelationKind::Embed,
    }
}
