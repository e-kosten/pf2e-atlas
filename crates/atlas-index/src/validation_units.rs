//! Reconstruct selected search fields and verify evidence kinds and whole section coverage.
use crate::{IndexError, SourceArtifactBuildContext};
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use atlas_record::source_record::{SourceBackedRecord, SourceContentOutcome, select_record_search};
use rusqlite::Connection;
use std::collections::BTreeMap;
fn invalid(s: &str) -> IndexError {
    IndexError::Invalid(s.into())
}
type SectionKey = (String, String, usize);
pub(crate) fn validate_selection(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    context: &SourceArtifactBuildContext,
    label: &str,
    content: &[SourceContentOutcome],
) -> Result<(), IndexError> {
    // The caller has checked these exact field bytes and sidecars in this same
    // transaction. Reuse them instead of decoding and validating every field twice.
    let selection = select_record_search(
        record,
        content,
        context.audience,
        label,
        &context.used_trait_labels,
    )
    .map_err(|e| IndexError::Invalid(e.to_string()))?;
    crate::selected_sections::validate_record(c, id, record, context, &selection)?;
    if atlas_record::source_record::SourceNodeView::from(record.source()).document_kind() != "Macro"
    {
        crate::core_projection::validate_record_core(c, id, record, &selection)?;
    }
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
            let address = field.section_address(section);
            let owners = serde_json::to_string(&field.locator.owners)?;
            sections.insert(
                (owners, field.locator.field.clone(), section.section_ordinal),
                (address, section.text.as_str()),
            );
        }
    }
    let mut statement=c.prepare(&format!("SELECT {},lu.identity_terms,lu.definition_terms,lu.structured_terms,lu.alias_terms FROM lexical_units lu JOIN records r USING(record_id) LEFT JOIN selected_sections ss ON ss.section_id=lu.section_id AND ss.record_id=lu.record_id WHERE lu.record_id=?",crate::read_units::LEXICAL_LOCATION_COLUMNS))?;
    let mut rows = statement.query([id])?;
    while let Some(row) = rows.next()? {
        let location = crate::read_units::location(row)?;
        let unit = crate::SourceLexicalUnitInput {
            record: location.record,
            owners: location.owners,
            field: location.field,
            address: location.address,
            kind: serde_json::from_value(serde_json::Value::String(row.get(13)?))?,
            identity_terms: row.get("identity_terms")?,
            definition_terms: row.get("definition_terms")?,
            structured_terms: row.get("structured_terms")?,
            alias_terms: row.get("alias_terms")?,
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
        let mut identity=c.prepare(&format!("SELECT {} FROM semantic_units su JOIN semantic_locations sl USING(location_id) JOIN records r ON r.record_id=su.record_id LEFT JOIN selected_sections ss ON ss.section_id=sl.section_id AND ss.record_id=su.record_id WHERE su.record_id=? AND su.profile_id=? AND su.unit_kind='identity'",crate::read_units::SEMANTIC_LOCATION_COLUMNS))?;
        let mut rows = identity.query(rusqlite::params![id, profile.profile_id.as_str()])?;
        while let Some(row) = rows.next()? {
            let location = crate::read_units::location(row)?;
            if selection.identity_required.is_empty()
                || !location.owners.is_empty()
                || location.field.is_some()
                || row.get::<_, usize>("location_chunk_ordinal")? != 0
                || location.address != Some(SourcePassageAddress::Identity {})
            {
                return Err(invalid("invalid semantic identity attribution"));
            }
        }
        let mut covered = BTreeMap::<SectionKey, Vec<(usize, SourceByteRange)>>::new();
        let mut statement=c.prepare(&format!("SELECT {} FROM semantic_units su JOIN semantic_locations sl USING(location_id) JOIN records r ON r.record_id=su.record_id LEFT JOIN selected_sections ss ON ss.section_id=sl.section_id AND ss.record_id=su.record_id WHERE su.record_id=? AND su.profile_id=? AND su.unit_kind='passage'",crate::read_units::SEMANTIC_LOCATION_COLUMNS))?;
        let mut rows = statement.query(rusqlite::params![id, profile.profile_id.as_str()])?;
        while let Some(row) = rows.next()? {
            let location = crate::read_units::location(row)?;
            let mut address = location
                .address
                .ok_or_else(|| invalid("semantic passage address unavailable"))?;
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
            let key = (
                serde_json::to_string(&location.owners)?,
                location
                    .field
                    .ok_or_else(|| invalid("semantic field unavailable"))?,
                ordinal,
            );
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
            covered
                .entry(key)
                .or_default()
                .push((row.get("location_chunk_ordinal")?, range));
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
