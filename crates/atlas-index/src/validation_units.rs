//! Reconstruct selected search fields and verify evidence kinds and whole section coverage.
use crate::{IndexError, SourceArtifactBuildContext};
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use atlas_record::{
    source_content::{CONTENT_INTERPRETATION_VERSION, PreparedSourceContent, SourceContentLocator},
    source_record::{
        FieldAvailability, SOURCE_CONTENT_SELECTION_VERSION, SourceBackedRecord,
        SourceContentOutcome, SourceContentStatus, select_record_search,
    },
};
use rusqlite::Connection;
use std::collections::BTreeMap;
fn invalid(s: &str) -> IndexError {
    IndexError::Invalid(s.into())
}
type SectionKey = (String, String, usize);
/// Shared prepared-cache decoding for selection reconstruction. Column order is
/// owners, field, role, visibility, authored hash, HTML, controls, context hash.
pub(crate) fn stored_content_outcome(
    row: &rusqlite::Row<'_>,
    record: &SourceBackedRecord,
    context: &SourceArtifactBuildContext,
    references: Vec<atlas_record::source_content::ContentReferenceOccurrence>,
) -> Result<SourceContentOutcome, IndexError> {
    let locator = SourceContentLocator {
        record: record.key().clone(),
        owners: serde_json::from_str(&row.get::<_, String>(0)?)?,
        field: row.get(1)?,
    };
    let selection = record
        .content_selection_at(&locator)
        .ok_or_else(|| invalid("cache has no checked authored selection"))?;
    let hash: String = row.get(4)?;
    let context_hash: String = row.get(7)?;
    let authored = selection
        .text
        .value()
        .ok_or_else(|| invalid("cache authored text unavailable"))?;
    if hash != crate::codec::sha256(authored.as_bytes())
        || context_hash != context.preparation_sha256()?
    {
        return Err(invalid("prepared cache input/context mismatch"));
    }
    let role = serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(2)?))?;
    let visibility = serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?))?;
    if role != selection.role
        || selection.visibility.value() != Some(visibility)
        || !crate::writer::permits(context.audience, visibility)
        || !matches!(
            selection.format,
            atlas_record::source_record::SourceFieldView::Value(
                atlas_record::source_record::SourceContentFormat::Html
            )
        )
    {
        return Err(invalid("prepared cache selection metadata mismatch"));
    }
    let html = String::from_utf8(crate::codec::gunzip(&row.get::<_, Vec<u8>>(5)?)?)
        .map_err(|e| IndexError::Invalid(e.to_string()))?;
    let interactions: Vec<atlas_record::source_content::ContentInteraction> =
        serde_json::from_str(&row.get::<_, String>(6)?)?;
    atlas_record::source_content::validate_prepared_markers(&html, &interactions)
        .map_err(|e| IndexError::Invalid(e.to_string()))?;
    atlas_record::source_content::validate_prepared_reference_bindings(
        &html,
        &locator,
        &hash,
        &references,
    )
    .map_err(|e| IndexError::Invalid(e.to_string()))?;
    Ok(SourceContentOutcome {
        role,
        visibility,
        visibility_availability: FieldAvailability::Value,
        status: SourceContentStatus::Prepared(Box::new(PreparedSourceContent {
            locator,
            interpretation_version: CONTENT_INTERPRETATION_VERSION.into(),
            authored_markup_sha256: hash,
            html,
            text: String::new(),
            references,
            interactions,
            diagnostics: vec![],
        })),
    })
}
pub(crate) fn stored_selection(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    context: &SourceArtifactBuildContext,
    label: &str,
) -> Result<atlas_record::source_record::SourceSearchSelection, IndexError> {
    let mut outcomes = vec![];
    let mut references = crate::read_content::content_references_for_record(c, id)?;
    let mut s=c.prepare("SELECT owners_json,field_path,role,visibility,authored_markup_sha256,html_gzip,interactions_json,preparation_context_hash FROM prepared_content WHERE record_id=? AND outcome='prepared'")?;
    let mut rows = s.query([id])?;
    while let Some(row) = rows.next()? {
        let locator = SourceContentLocator {
            record: record.key().clone(),
            owners: serde_json::from_str(&row.get::<_, String>(0)?)?,
            field: row.get(1)?,
        };
        let field_references = references
            .remove(&crate::read_content::content_reference_key(&locator)?)
            .unwrap_or_default();
        outcomes.push(stored_content_outcome(
            row,
            record,
            context,
            field_references,
        )?);
    }
    if !references.is_empty() {
        return Err(invalid("content reference has no prepared search field"));
    }
    select_record_search(
        record,
        &outcomes,
        context.audience,
        label,
        &context.used_trait_labels,
    )
    .map_err(|e| IndexError::Invalid(e.to_string()))
}
pub(crate) fn validate_selection(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    context: &SourceArtifactBuildContext,
    label: &str,
) -> Result<(), IndexError> {
    let selection = stored_selection(c, id, record, context, label)?;
    let mut aliases =
        c.prepare("SELECT alias FROM verified_aliases WHERE record_id=? ORDER BY alias")?;
    let aliases = aliases
        .query_map([id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut lexical = crate::project_lexical_units(&selection, &aliases)
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let mut sections = BTreeMap::<SectionKey, (SourcePassageAddress, &str)>::new();
    for field in &selection.fields {
        for section in &field.sections {
            let range = SourceByteRange {
                start: 0,
                end: section.text.len(),
            };
            let address = if let Some(html_hash) = &field.prepared_html_sha256 {
                SourcePassageAddress::HtmlSection {
                    prepared_html_sha256: html_hash.clone(),
                    canonical_text_sha256: crate::codec::sha256(section.text.as_bytes()),
                    selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                    section_ordinal: section.section_ordinal,
                    label: section.label.clone(),
                    chunk_bytes: range,
                }
            } else {
                SourcePassageAddress::PlainSection {
                    source_text_sha256: crate::codec::sha256(section.text.as_bytes()),
                    selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                    section_ordinal: section.section_ordinal,
                    label: section.label.clone(),
                    chunk_bytes: range,
                }
            };
            let owners = serde_json::to_string(&field.locator.owners)?;
            sections.insert(
                (owners, field.locator.field.clone(), section.section_ordinal),
                (address, section.text.as_str()),
            );
        }
    }
    let mut s=c.prepare("SELECT owners_json,field_path,section_json,unit_kind,identity_terms,definition_terms,structured_terms,alias_terms FROM lexical_units WHERE record_id=?")?;
    let mut rows = s.query([id])?;
    while let Some(row) = rows.next()? {
        let address: Option<String> = row.get(2)?;
        let unit = crate::SourceLexicalUnitInput {
            record: record.key().clone(),
            owners: serde_json::from_str(&row.get::<_, String>(0)?)?,
            field: row.get(1)?,
            address: address
                .map(|value| serde_json::from_str(&value))
                .transpose()?,
            kind: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?))?,
            identity_terms: row.get(4)?,
            definition_terms: row.get(5)?,
            structured_terms: row.get(6)?,
            alias_terms: row.get(7)?,
        };
        if !lexical.remove(&serde_json::to_string(&unit)?) {
            return Err(invalid(
                "lexical row differs from selected source policy or is duplicated",
            ));
        }
    }
    if !lexical.is_empty() {
        return Err(invalid("lexical evidence coverage is incomplete"));
    }
    for profile in &context.search_profiles {
        let mut identity=c.prepare("SELECT owners_json,field_path,section_json,chunk_ordinal FROM semantic_units WHERE record_id=? AND profile_id=? AND unit_kind='identity'")?;
        for row in identity.query_map(rusqlite::params![id, profile.profile_id.as_str()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, usize>(3)?,
            ))
        })? {
            let (owners, field, address, ordinal) = row?;
            if selection.identity_required.is_empty()
                || owners != "[]"
                || field.is_some()
                || ordinal != 0
                || serde_json::from_str::<SourcePassageAddress>(&address)?
                    != (SourcePassageAddress::Identity {})
            {
                return Err(invalid("invalid semantic identity attribution"));
            }
        }
        let mut covered = BTreeMap::<SectionKey, Vec<(usize, SourceByteRange)>>::new();
        let mut s=c.prepare("SELECT owners_json,field_path,section_json,chunk_ordinal FROM semantic_units WHERE record_id=? AND profile_id=? AND unit_kind='passage'")?;
        let mut rows = s.query(rusqlite::params![id, profile.profile_id.as_str()])?;
        while let Some(row) = rows.next()? {
            let mut address: SourcePassageAddress =
                serde_json::from_str(&row.get::<_, String>(2)?)?;
            let (ordinal, range) = match &address {
                SourcePassageAddress::HtmlSection {
                    section_ordinal,
                    chunk_bytes,
                    ..
                }
                | SourcePassageAddress::PlainSection {
                    section_ordinal,
                    chunk_bytes,
                    ..
                } => (*section_ordinal, *chunk_bytes),
                SourcePassageAddress::Identity {} => {
                    return Err(invalid("identity stored as passage"));
                }
            };
            let key = (row.get::<_, String>(0)?, row.get::<_, String>(1)?, ordinal);
            let (expected, text) = sections
                .get(&key)
                .ok_or_else(|| invalid("unselected semantic section"))?;
            match &mut address {
                SourcePassageAddress::HtmlSection { chunk_bytes, .. }
                | SourcePassageAddress::PlainSection { chunk_bytes, .. } => {
                    *chunk_bytes = SourceByteRange {
                        start: 0,
                        end: text.len(),
                    }
                }
                SourcePassageAddress::Identity {} => {
                    return Err(invalid("identity stored as passage"));
                }
            };
            if &address != expected {
                return Err(invalid("semantic section source identity mismatch"));
            }
            range
                .extract(text)
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
            covered.entry(key).or_default().push((row.get(3)?, range));
        }
        for (key, (_, text)) in sections.clone() {
            let mut chunks = covered
                .remove(&key)
                .ok_or_else(|| invalid("semantic section coverage is incomplete"))?;
            chunks.sort_by_key(|v| v.0);
            let mut end = 0;
            for (index, (ordinal, range)) in chunks.iter().enumerate() {
                if *ordinal != index
                    || range.start > end
                    || range.end <= range.start
                    || range.end <= end
                {
                    return Err(invalid(
                        "semantic chunk sequence is duplicated, empty or has gaps",
                    ));
                }
                end = range.end;
            }
            if end != text.len() {
                return Err(invalid("semantic section tail is uncovered"));
            }
        }
        if !covered.is_empty() {
            return Err(invalid("unaccounted semantic sections"));
        }
    }
    Ok(())
}
