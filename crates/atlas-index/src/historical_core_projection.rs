//! Metadata-only Core eligibility over existing checked units and locations.
use crate::{IndexError, SourceArtifactBuildContext, SourceArtifactRecordInput};
use atlas_domain::{CORE_SECTIONS_RECIPE, SourcePassageAddress};
use atlas_record::source_record::{
    SourceBackedRecord, SourceSearchSelection, core_search_selection, select_record_search,
};
use rusqlite::{Connection, params};
use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct CoreMembership {
    pub(super) family: String,
    modelled: bool,
    fallback_reason: Option<String>,
    pub(super) parts: BTreeSet<String>,
    pub(super) lexical_units: BTreeSet<i64>,
    pub(super) semantic_locations: BTreeSet<i64>,
}

type SectionKey = (String, String, usize, String);

fn section_key(
    owners: String,
    field: Option<String>,
    address: SourcePassageAddress,
) -> Option<SectionKey> {
    let (ordinal, hash) = match address {
        SourcePassageAddress::HtmlSection {
            section_ordinal,
            canonical_text_sha256,
            ..
        } => (section_ordinal, canonical_text_sha256),
        SourcePassageAddress::PlainSection {
            section_ordinal,
            source_text_sha256,
            ..
        } => (section_ordinal, source_text_sha256),
        SourcePassageAddress::Identity {} => return None,
    };
    Some((owners, field?, ordinal, hash))
}

pub(super) fn membership(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    selection: &SourceSearchSelection,
) -> Result<CoreMembership, IndexError> {
    let core = core_search_selection(record, selection)
        .map_err(|error| IndexError::Invalid(error.to_string()))?;
    let owners = core
        .owners
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<BTreeSet<_>, _>>()?;
    let fields = core
        .fields
        .iter()
        .map(|field| Ok((serde_json::to_string(&field.owners)?, field.field.clone())))
        .collect::<Result<BTreeSet<_>, IndexError>>()?;
    let mut output = CoreMembership {
        family: serde_json::to_value(core.family)?
            .as_str()
            .ok_or_else(|| IndexError::Invalid("invalid Core family encoding".into()))?
            .to_owned(),
        modelled: core.family.is_modelled(),
        fallback_reason: core
            .fallback_reason()
            .map(|reason| {
                serde_json::to_value(reason)
                    .map(|value| value.as_str().unwrap_or_default().to_owned())
            })
            .transpose()?,
        parts: BTreeSet::new(),
        lexical_units: BTreeSet::new(),
        semantic_locations: BTreeSet::new(),
    };
    let mut sections = BTreeSet::new();
    let mut parts = c.prepare_cached("SELECT selector,owners_json,field_path,section_ordinal,body_hash FROM similarity_parts WHERE record_id=?")?;
    let mut parts = parts.query([id])?;
    while let Some(row) = parts.next()? {
        let owner: String = row.get(1)?;
        let field: String = row.get(2)?;
        if fields.contains(&(owner.clone(), field.clone())) {
            output.parts.insert(row.get(0)?);
            sections.insert((owner, field, row.get(3)?, row.get(4)?));
        }
    }
    let mut lexical = c.prepare_cached("SELECT unit_id,owners_json,field_path,section_json,unit_kind FROM lexical_units WHERE record_id=?")?;
    let mut lexical = lexical.query([id])?;
    while let Some(row) = lexical.next()? {
        let kind: String = row.get(4)?;
        let owner: String = row.get(1)?;
        let allowed = match kind.as_str() {
            "root_name" | "root_alias" => true,
            "owned_name" => owners.contains(&owner),
            "heading" | "definition_label" => {
                let address: Option<String> = row.get(3)?;
                address
                    .map(|address| -> Result<bool, IndexError> {
                        Ok(
                            section_key(owner, row.get(2)?, serde_json::from_str(&address)?)
                                .is_some_and(|key| sections.contains(&key)),
                        )
                    })
                    .transpose()?
                    .unwrap_or(false)
            }
            _ => {
                return Err(IndexError::Invalid(
                    "unknown lexical unit kind in Core projection".into(),
                ));
            }
        };
        if allowed {
            output.lexical_units.insert(row.get(0)?);
        }
    }
    let mut semantic = c.prepare_cached("SELECT sl.location_id,sl.owners_json,sl.field_path,sl.section_json FROM semantic_locations sl WHERE sl.record_id=? AND EXISTS(SELECT 1 FROM semantic_units su WHERE su.location_id=sl.location_id)")?;
    let mut semantic = semantic.query([id])?;
    while let Some(row) = semantic.next()? {
        let owner: String = row.get(1)?;
        let field: Option<String> = row.get(2)?;
        let address: SourcePassageAddress = serde_json::from_str(&row.get::<_, String>(3)?)?;
        let allowed = match address {
            SourcePassageAddress::Identity {} => owner == "[]" && field.is_none(),
            address => {
                section_key(owner, field, address).is_some_and(|key| sections.contains(&key))
            }
        };
        if allowed {
            output.semantic_locations.insert(row.get(0)?);
        }
    }
    Ok(output)
}

pub(super) fn write_membership(
    c: &Connection,
    id: i64,
    expected: &CoreMembership,
) -> Result<(), IndexError> {
    c.execute("DELETE FROM core_roots WHERE record_id=?", [id])?;
    c.execute("INSERT INTO core_roots(record_id,family,modelled,fallback_reason,recipe) VALUES(?,?,?,?,?)", params![id,expected.family,expected.modelled,expected.fallback_reason,CORE_SECTIONS_RECIPE])?;
    let mut parts = c.prepare_cached("INSERT INTO core_parts(selector,record_id) VALUES(?,?)")?;
    for selector in &expected.parts {
        parts.execute(params![selector, id])?;
    }
    let mut lexical =
        c.prepare_cached("INSERT INTO core_lexical_units(unit_id,record_id) VALUES(?,?)")?;
    for unit in &expected.lexical_units {
        lexical.execute(params![unit, id])?;
    }
    let mut semantic =
        c.prepare_cached("INSERT INTO core_semantic_locations(location_id,record_id) VALUES(?,?)")?;
    for location in &expected.semantic_locations {
        semantic.execute(params![location, id])?;
    }
    Ok(())
}

pub(crate) fn write_record_core(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    source: &SourceArtifactRecordInput,
) -> Result<(), IndexError> {
    let (id, label): (i64, String) = c.query_row(
        "SELECT r.record_id,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.key=?",
        [source.record.key().to_string()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let selection = select_record_search(
        &source.record,
        &source.content,
        context.audience,
        &label,
        &context.used_trait_labels,
    )
    .map_err(|error| IndexError::Invalid(error.to_string()))?;
    write_membership(c, id, &membership(c, id, &source.record, &selection)?)
}

pub(crate) fn write_core_sections(
    c: &Connection,
    input: &crate::IndexBuildInput,
) -> Result<(), IndexError> {
    for source in &input.records {
        if atlas_record::source_record::SourceNodeView::from(source.record.source()).document_kind()
            != "Macro"
        {
            write_record_core(c, &input.context, source)?;
        }
    }
    Ok(())
}
