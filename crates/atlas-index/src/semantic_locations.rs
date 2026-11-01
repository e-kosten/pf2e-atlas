//! Shared section references and chunk ranges across native search profiles.
use crate::{IndexError, SourceSemanticUnitInput};
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) fn location_id(
    c: &Connection,
    record: i64,
    unit: &SourceSemanticUnitInput,
) -> Result<i64, IndexError> {
    let (section, ordinal, start, end) = match &unit.address {
        SourcePassageAddress::Identity {} => {
            if !unit.owners.is_empty() || unit.field.is_some() || unit.chunk_ordinal != 0 {
                return Err(IndexError::Invalid(
                    "invalid semantic identity attribution".into(),
                ));
            }
            let previous:Option<i64>=c.query_row("SELECT location_id FROM semantic_locations WHERE record_id=? AND section_id IS NULL AND chunk_ordinal=0",[record],|r|r.get(0)).optional()?;
            if let Some(id) = previous {
                return Ok(id);
            }
            (None, 0, None, None)
        }
        SourcePassageAddress::HtmlSection {
            section_ordinal,
            chunk_bytes,
            ..
        }
        | SourcePassageAddress::PlainSection {
            section_ordinal,
            chunk_bytes,
            ..
        } => {
            let field = unit
                .field
                .as_deref()
                .ok_or_else(|| IndexError::Invalid("semantic passage has no field".into()))?;
            let section = crate::selected_sections::find(
                c,
                record,
                &serde_json::to_string(&unit.owners)?,
                field,
                *section_ordinal,
            )?;
            if section.chunk_address(*chunk_bytes)? != unit.address {
                return Err(IndexError::Invalid(
                    "semantic passage identity differs from its canonical section".into(),
                ));
            }
            (
                Some(section.id),
                unit.chunk_ordinal,
                Some(chunk_bytes.start),
                Some(chunk_bytes.end),
            )
        }
    };
    c.execute("INSERT INTO semantic_locations(record_id,section_id,chunk_ordinal,chunk_start,chunk_end) VALUES(?,?,?,?,?) ON CONFLICT(section_id,chunk_ordinal,chunk_start,chunk_end) DO NOTHING",params![record,section,ordinal,start,end])?;
    Ok(match section {
        Some(section)=>c.query_row("SELECT location_id FROM semantic_locations WHERE section_id=? AND chunk_ordinal=? AND chunk_start=? AND chunk_end=?",params![section,ordinal,start,end],|r|r.get(0))?,
        None=>c.query_row("SELECT location_id FROM semantic_locations WHERE record_id=? AND section_id IS NULL AND chunk_ordinal=0",[record],|r|r.get(0))?,
    })
}
pub(crate) fn validate(c: &Connection) -> Result<(), IndexError> {
    let invalid:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM semantic_units u JOIN semantic_locations l USING(location_id) LEFT JOIN selected_sections s USING(section_id) JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) JOIN search_profiles p USING(profile_id) WHERE u.record_id<>l.record_id OR m.identity_json<>p.identity_json OR (u.unit_kind='identity' AND (l.section_id IS NOT NULL OR l.chunk_ordinal IS NOT 0 OR l.chunk_start IS NOT NULL OR l.chunk_end IS NOT NULL)) OR (u.unit_kind='passage' AND (s.section_id IS NULL OR s.record_id<>u.record_id OR l.chunk_ordinal IS NULL OR l.chunk_start IS NULL OR l.chunk_end IS NULL OR l.chunk_start<0 OR l.chunk_end<=l.chunk_start OR l.chunk_end>s.byte_length)))",[],|r|r.get(0))?;
    if invalid {
        return Err(IndexError::Invalid(
            "semantic unit model, section or chunk attribution mismatch".into(),
        ));
    }
    Ok(())
}
pub(crate) fn range(
    row: &rusqlite::Row<'_>,
    start: usize,
    end: usize,
) -> rusqlite::Result<SourceByteRange> {
    Ok(SourceByteRange {
        start: row.get(start)?,
        end: row.get(end)?,
    })
}
