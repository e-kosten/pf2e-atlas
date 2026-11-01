//! Metadata-only Core eligibility over existing checked units and locations.
use crate::{IndexError, SourceArtifactBuildContext, SourceArtifactRecordInput};
use atlas_domain::CORE_SECTIONS_RECIPE;
use atlas_record::source_record::{
    SourceBackedRecord, SourceSearchSelection, core_search_selection, select_record_search,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeSet;

pub const CORE_SECTIONS_SCHEMA_SQL: &str = "
CREATE TABLE core_roots(
 record_id INTEGER PRIMARY KEY REFERENCES records ON DELETE CASCADE,
 family TEXT NOT NULL,
 modelled INTEGER NOT NULL CHECK(modelled IN (0,1)),
 fallback_reason TEXT CHECK(fallback_reason IS NULL OR fallback_reason IN ('unmodelled_kind','no_core_prose')),
 recipe TEXT NOT NULL
) STRICT;
CREATE TABLE core_parts(
 selector TEXT PRIMARY KEY REFERENCES similarity_parts ON DELETE CASCADE,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE
) STRICT;
CREATE INDEX core_parts_root ON core_parts(record_id,selector);
CREATE TABLE core_lexical_units(
 unit_id INTEGER PRIMARY KEY REFERENCES lexical_units ON DELETE CASCADE,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE
) STRICT;
CREATE INDEX core_lexical_units_root ON core_lexical_units(record_id,unit_id);
CREATE TABLE core_semantic_locations(
 location_id INTEGER PRIMARY KEY REFERENCES semantic_locations ON DELETE CASCADE,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE
) STRICT;
CREATE INDEX core_semantic_locations_root ON core_semantic_locations(record_id,location_id);
";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct CoreMembership {
    pub(super) family: String,
    modelled: bool,
    fallback_reason: Option<String>,
    pub(super) parts: BTreeSet<i64>,
    pub(super) lexical_units: BTreeSet<i64>,
    pub(super) semantic_locations: BTreeSet<i64>,
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
    let mut parts = c.prepare_cached(
        "SELECT section_id,owners_json,field_path FROM selected_sections WHERE record_id=?",
    )?;
    let mut parts = parts.query([id])?;
    while let Some(row) = parts.next()? {
        let owner: String = row.get(1)?;
        let field: String = row.get(2)?;
        if fields.contains(&(owner.clone(), field.clone())) {
            output.parts.insert(row.get(0)?);
        }
    }
    let mut lexical = c.prepare_cached(
        "SELECT unit_id,owners_json,section_id,unit_kind FROM lexical_units WHERE record_id=?",
    )?;
    let mut lexical = lexical.query([id])?;
    while let Some(row) = lexical.next()? {
        let kind: String = row.get(3)?;
        let owner: Option<String> = row.get(1)?;
        let allowed = match kind.as_str() {
            "root_name" | "root_alias" => true,
            "owned_name" => owner.is_some_and(|owner| owners.contains(&owner)),
            "heading" | "definition_label" => row
                .get::<_, Option<i64>>(2)?
                .is_some_and(|section| output.parts.contains(&section)),
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
    let mut semantic=c.prepare_cached("SELECT sl.location_id,sl.section_id,sl.chunk_ordinal FROM semantic_locations sl WHERE sl.record_id=? AND EXISTS(SELECT 1 FROM semantic_units su WHERE su.location_id=sl.location_id)")?;
    let mut rows = semantic.query([id])?;
    while let Some(row) = rows.next()? {
        let section: Option<i64> = row.get(1)?;
        let allowed = match section {
            Some(section) => output.parts.contains(&section),
            None => row.get::<_, Option<usize>>(2)? == Some(0),
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
    let actual: Option<(String, bool, Option<String>, String)> = c
        .query_row(
            "SELECT family,modelled,fallback_reason,recipe FROM core_roots WHERE record_id=?",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    if actual
        == Some((
            expected.family.clone(),
            expected.modelled,
            expected.fallback_reason.clone(),
            CORE_SECTIONS_RECIPE.into(),
        ))
    {
        let read_set = |table: &str, column: &str| -> Result<BTreeSet<i64>, IndexError> {
            Ok(
                c.prepare(&format!("SELECT {column} FROM {table} WHERE record_id=?"))?
                    .query_map([id], |r| r.get(0))?
                    .collect::<Result<BTreeSet<_>, _>>()?,
            )
        };
        if read_set("core_parts", "section_id")? == expected.parts
            && read_set("core_lexical_units", "unit_id")? == expected.lexical_units
            && read_set("core_semantic_locations", "location_id")? == expected.semantic_locations
        {
            return Ok(());
        }
    }
    c.execute("DELETE FROM core_roots WHERE record_id=?", [id])?;
    c.execute("INSERT INTO core_roots(record_id,family,modelled,fallback_reason,recipe) VALUES(?,?,?,?,?)", params![id,expected.family,expected.modelled,expected.fallback_reason,CORE_SECTIONS_RECIPE])?;
    let mut parts = c.prepare_cached("INSERT INTO core_parts(section_id,record_id) VALUES(?,?)")?;
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

/// The caller publishes the policy/context and complete metadata in one maintenance transaction.
pub(crate) fn backfill_core_sections(
    c: &Connection,
    context: &SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    let mut after = 0_i64;
    loop {
        let roots = c.prepare("SELECT record_id,key FROM records WHERE record_id>? AND document_kind<>'Macro' ORDER BY record_id LIMIT 128")?.query_map([after],|row| Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
        if roots.is_empty() {
            break;
        }
        for (id, key) in roots {
            after = id;
            let key = key.parse().map_err(|error| {
                IndexError::Invalid(format!("invalid Core record key: {error}"))
            })?;
            let source = crate::persisted_inputs::read_attested_persisted_input(c, context, &key)?
                .ok_or_else(|| {
                    IndexError::Invalid("Core backfill lost its checked source".into())
                })?;
            write_record_core(c, context, &source)?;
        }
    }
    Ok(())
}

pub(crate) fn validate_record_core(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    selection: &SourceSearchSelection,
) -> Result<(), IndexError> {
    let expected = membership(c, id, record, selection)?;
    let (family, modelled, fallback_reason, recipe): (String, bool, Option<String>, String) = c
        .query_row(
            "SELECT family,modelled,fallback_reason,recipe FROM core_roots WHERE record_id=?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
    let read_ids = |table: &str, column: &str| -> Result<BTreeSet<i64>, IndexError> {
        Ok(
            c.prepare(&format!("SELECT {column} FROM {table} WHERE record_id=?"))?
                .query_map([id], |row| row.get(0))?
                .collect::<Result<_, _>>()?,
        )
    };
    let actual = CoreMembership {
        family,
        modelled,
        fallback_reason,
        parts: c
            .prepare("SELECT section_id FROM core_parts WHERE record_id=?")?
            .query_map([id], |row| row.get(0))?
            .collect::<Result<_, _>>()?,
        lexical_units: read_ids("core_lexical_units", "unit_id")?,
        semantic_locations: read_ids("core_semantic_locations", "location_id")?,
    };
    if recipe != CORE_SECTIONS_RECIPE || actual != expected {
        return Err(IndexError::Invalid(
            "Core eligibility differs from its checked source policy".into(),
        ));
    }
    Ok(())
}
