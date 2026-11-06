//! Checked remaster statements and rebuildable current-reference projections.
use crate::{IndexError, PublicationSpool, SqliteIndexReader};
use atlas_domain::{OwnedContentLocator, RecordKey, normalize_record_name};
use atlas_record::{
    remaster::*,
    source_content::{CONTENT_INTERPRETATION_VERSION, PreparedSourceContent, SourceContentLocator},
    source_record::SourceBackedRecord,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

pub const REMASTER_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000030_remaster_information/up.sql");
#[derive(Debug, Clone)]
pub struct SourceRemasterInformation {
    pub declarations: Vec<SourceRemasterDeclaration>,
    pub truncated: bool,
}
#[derive(Debug, Clone)]
pub struct SourceRemasterDeclaration {
    pub declaration: RemasterDeclaration,
    pub bindings: Vec<RemasterBinding>,
    pub citations: Vec<SourceRemasterCitation>,
}
#[derive(Debug, Clone)]
pub struct SourceRemasterCitation {
    pub origin: RemasterOrigin,
    pub attribution: String,
    pub revision: String,
    pub statement: String,
    pub locator: Option<SourceContentLocator>,
    pub path: Option<String>,
}
fn invalid(e: impl std::fmt::Display) -> IndexError {
    IndexError::Invalid(e.to_string())
}
fn side(value: RemasterSideKind) -> &'static str {
    match value {
        RemasterSideKind::Before => "before",
        RemasterSideKind::After => "after",
    }
}
fn current_facts(
    c: &Connection,
    live: Option<&BTreeMap<RecordKey, i64>>,
) -> Result<BTreeMap<RecordKey, RemasterRecordFacts>, IndexError> {
    let mut result = BTreeMap::new();
    for row in c
        .prepare("SELECT key,name,source_type FROM records WHERE document_kind<>'Macro'")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?
    {
        let (key, name, source_type) = row?;
        let key = RecordKey::parse(&key).map_err(invalid)?;
        if live.is_none_or(|live| live.contains_key(&key)) {
            result.insert(key, RemasterRecordFacts { name, source_type });
        }
    }
    Ok(result)
}
fn selected_journals(
    c: &Connection,
    facts: &BTreeMap<RecordKey, RemasterRecordFacts>,
) -> Result<(Vec<RemasterDeclaration>, Vec<RemasterSelectedField>), IndexError> {
    let mut declarations = vec![];
    let mut fields = vec![];
    let mut q=c.prepare("SELECT r.record_id,r.key,b.snapshot FROM records r JOIN record_bodies b USING(record_id) WHERE r.document_kind='JournalEntry' AND r.name='Remaster Changes' ORDER BY r.key")?;
    let rows = q.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Vec<u8>>(2)?,
        ))
    })?;
    for row in rows {
        let (id, key, bytes) = row?;
        let key = RecordKey::parse(&key).map_err(invalid)?;
        if !facts.contains_key(&key) {
            continue;
        }
        let record = SourceBackedRecord::new(
            key.pack().as_str(),
            atlas_foundry_model::decode_snapshot(&crate::codec::gunzip(&bytes)?)?,
        )
        .map_err(|_| invalid("remaster journal snapshot identity differs"))?;
        let references = crate::read_content::content_references_for_record(c, id)?;
        let mut query=c.prepare("SELECT owners_json,field_path,authored_markup_sha256,html_gzip FROM prepared_content WHERE record_id=? AND published=1 AND outcome='prepared' ORDER BY owners_json,field_path")?;
        for row in query.query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Vec<u8>>(3)?,
            ))
        })? {
            let (owners, field, hash, html) = row?;
            let locator = SourceContentLocator {
                record: key.clone(),
                owners: serde_json::from_str(&owners)?,
                field: field.clone(),
            };
            let authored = record
                .authored_content_at(&locator)
                .value()
                .ok_or_else(|| invalid("remaster citation field unavailable"))?;
            if hash != crate::codec::sha256(authored.as_bytes()) {
                return Err(invalid("remaster selected field hash differs"));
            }
            let section = record
                .node_at(&locator.owners)
                .and_then(|n| n.name().value().cloned())
                .ok_or_else(|| invalid("remaster source section unavailable"))?;
            let refs = references
                .get(&(key.to_string(), owners, field))
                .cloned()
                .unwrap_or_default();
            let prepared = PreparedSourceContent {
                locator: locator.clone(),
                interpretation_version: CONTENT_INTERPRETATION_VERSION.into(),
                authored_markup_sha256: hash.clone(),
                html: String::from_utf8(crate::codec::gunzip(&html)?).map_err(invalid)?,
                text: String::new(),
                references: refs.clone(),
                interactions: vec![],
                diagnostics: vec![],
            };
            declarations.extend(journal_declarations(&section, &prepared).map_err(invalid)?);
            fields.push(RemasterSelectedField {
                locator,
                authored_sha256: hash,
                references: refs,
            });
        }
    }
    Ok((declarations, fields))
}
struct State {
    sources: Vec<RemasterSourceInput>,
    declarations: Vec<RemasterDeclaration>,
    bindings: Vec<RemasterBinding>,
    projection: RemasterProjection,
}
impl State {
    fn derive(
        c: &Connection,
        mut sources: Vec<RemasterSourceInput>,
        facts: &BTreeMap<RecordKey, RemasterRecordFacts>,
    ) -> Result<Self, IndexError> {
        sources.sort_by(|a, b| a.key.cmp(&b.key));
        if sources.windows(2).any(|v| v[0].key == v[1].key) {
            return Err(invalid("duplicate remaster source capture"));
        }
        let (mut declarations, fields) = selected_journals(c, facts)?;
        for source in &sources {
            declarations.extend(file_declarations(source).map_err(invalid)?);
        }
        declarations.sort_by(|a, b| a.id.cmp(&b.id));
        if declarations.windows(2).any(|v| v[0].id == v[1].id) {
            return Err(invalid("duplicate remaster declaration"));
        }
        let bindings =
            bind_remaster_declarations(&declarations, facts, &fields).map_err(invalid)?;
        let projection = derive_remaster_projection(&declarations, &bindings, facts, &sources);
        Ok(Self {
            sources,
            declarations,
            bindings,
            projection,
        })
    }
    fn digest(&self) -> Result<String, IndexError> {
        Ok(crate::codec::sha256(
            serde_json::to_vec(&(
                REMASTER_RECIPE,
                &self.sources,
                &self.declarations,
                &self.bindings,
                &self.projection.aliases,
                &self.projection.preferences,
            ))?
            .as_slice(),
        ))
    }
}
pub(crate) fn publish(
    c: &Connection,
    input: &PublicationSpool,
    products: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let mut sources = vec![];
    input.remaster_sources(&mut |row| {
        sources.push(row.clone());
        Ok(())
    })?;
    let state = State::derive(c, sources, &current_facts(c, Some(products))?)?;
    let mut supplied = vec![];
    input.remaster_declarations(&mut |row| {
        supplied.push(row.clone());
        Ok(())
    })?;
    supplied.sort_by(|a, b| a.id.cmp(&b.id));
    if supplied != state.declarations {
        return Err(invalid(
            "remaster statements differ from selected captured source",
        ));
    }
    let digest = state.digest()?;
    if c.query_row(
        "SELECT digest FROM remaster_metadata WHERE singleton=1",
        [],
        |r| r.get::<_, String>(0),
    )
    .optional()?
    .as_deref()
        == Some(&digest)
    {
        return Ok(());
    }
    c.execute("DELETE FROM remaster_declarations", [])?;
    c.execute("DELETE FROM remaster_source_inputs", [])?;
    c.execute("DELETE FROM remaster_aliases", [])?;
    c.execute("DELETE FROM remaster_preferences", [])?;
    for source in &state.sources {
        c.execute(
            "INSERT INTO remaster_source_inputs VALUES(?,?)",
            params![source.key, serde_json::to_string(source)?],
        )?;
    }
    for declaration in &state.declarations {
        c.execute(
            "INSERT INTO remaster_declarations VALUES(?,?)",
            params![declaration.id, serde_json::to_string(declaration)?],
        )?;
        for (ordinal, citation) in declaration.citations.iter().enumerate() {
            let (input_key, record) = match citation {
                RemasterCitation::CapturedFile { input_key, .. } => {
                    (Some(input_key.as_str()), None)
                }
                RemasterCitation::Journal { locator, .. } => (
                    None,
                    Some(
                        *products
                            .get(&locator.record)
                            .ok_or_else(|| invalid("remaster citation root unavailable"))?,
                    ),
                ),
            };
            c.execute(
                "INSERT INTO remaster_citations VALUES(?,?,?,?,?)",
                params![
                    declaration.id,
                    ordinal as i64,
                    input_key,
                    record,
                    serde_json::to_string(citation)?
                ],
            )?;
        }
        let terms = declaration
            .before
            .iter()
            .chain(&declaration.after)
            .flat_map(|side| &side.terms)
            .map(|t| normalize_record_name(t))
            .filter(|t| !t.is_empty())
            .collect::<BTreeSet<_>>();
        for term in terms {
            c.execute(
                "INSERT INTO remaster_terms VALUES(?,?,?)",
                params![term, declaration.context.namespace, declaration.id],
            )?;
        }
    }
    for binding in &state.bindings {
        let (key, record, owners) = match &binding.resolution {
            RemasterBindingResolution::Current { record_key, owners } => (
                Some(record_key.to_string()),
                Some(
                    *products
                        .get(record_key)
                        .ok_or_else(|| invalid("remaster binding root unavailable"))?,
                ),
                serde_json::to_string(owners)?,
            ),
            RemasterBindingResolution::MissingExplicitKey { record_key } => {
                (Some(record_key.to_string()), None, "[]".into())
            }
            _ => (None, None, "[]".into()),
        };
        c.execute(
            "INSERT INTO remaster_bindings VALUES(?,?,?,?,?,?,?,?)",
            params![
                binding.declaration,
                side(binding.side),
                binding.group as i64,
                binding.reference as i64,
                key,
                record,
                owners,
                serde_json::to_string(binding)?
            ],
        )?;
    }
    for alias in &state.projection.aliases {
        let id = products
            .get(&alias.record)
            .ok_or_else(|| invalid("alias root unavailable"))?;
        c.execute(
            "INSERT INTO remaster_aliases VALUES(?,?,?)",
            params![id, alias.alias, normalize_record_name(&alias.alias)],
        )?;
        for source in &alias.declarations {
            c.execute(
                "INSERT INTO remaster_alias_support VALUES(?,?,?)",
                params![id, alias.alias, source],
            )?;
        }
    }
    for preference in &state.projection.preferences {
        let old = products
            .get(&preference.predecessor)
            .ok_or_else(|| invalid("predecessor missing"))?;
        let new = products
            .get(&preference.successor)
            .ok_or_else(|| invalid("successor missing"))?;
        c.execute(
            "INSERT INTO remaster_preferences VALUES(?,?)",
            params![old, new],
        )?;
        for source in &preference.declarations {
            c.execute(
                "INSERT INTO remaster_preference_support VALUES(?,?)",
                params![old, source],
            )?;
        }
    }
    c.execute("INSERT INTO remaster_metadata VALUES(1,?) ON CONFLICT(singleton) DO UPDATE SET digest=excluded.digest",[digest])?;
    Ok(())
}
fn read_sources(c: &Connection) -> Result<Vec<RemasterSourceInput>, IndexError> {
    c.prepare("SELECT payload_json FROM remaster_source_inputs ORDER BY input_key")?
        .query_map([], |r| r.get::<_, String>(0))?
        .map(|r| Ok(serde_json::from_str(&r?)?))
        .collect()
}
fn require_rows(c: &Connection, sql: &str, expected: BTreeSet<String>) -> Result<(), IndexError> {
    let actual = c
        .prepare(sql)?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<BTreeSet<_>, _>>()?;
    if actual != expected {
        return Err(invalid(format!("remaster projection differs: {sql}")));
    }
    Ok(())
}
fn encoded<T: serde::Serialize>(value: T) -> Result<String, IndexError> {
    Ok(serde_json::to_string(&value)?)
}
pub(crate) fn validate(c: &Connection) -> Result<(), IndexError> {
    let state = State::derive(c, read_sources(c)?, &current_facts(c, None)?)?;
    require_rows(
        c,
        "SELECT json_array(input_key,payload_json) FROM remaster_source_inputs",
        state
            .sources
            .iter()
            .map(|v| encoded((&v.key, serde_json::to_string(v)?)))
            .collect::<Result<_, IndexError>>()?,
    )?;
    require_rows(
        c,
        "SELECT json_array(declaration_id,payload_json) FROM remaster_declarations",
        state
            .declarations
            .iter()
            .map(|v| encoded((&v.id, serde_json::to_string(v)?)))
            .collect::<Result<_, IndexError>>()?,
    )?;
    let ids = c
        .prepare("SELECT key,record_id FROM records")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut citations = BTreeSet::new();
    let mut terms = BTreeSet::new();
    for declaration in &state.declarations {
        for (ordinal, citation) in declaration.citations.iter().enumerate() {
            let (source, record) = match citation {
                RemasterCitation::CapturedFile { input_key, .. } => {
                    (Some(input_key.as_str()), None)
                }
                RemasterCitation::Journal { locator, .. } => {
                    (None, ids.get(&locator.record.to_string()).copied())
                }
            };
            citations.insert(encoded((
                &declaration.id,
                ordinal,
                source,
                record,
                serde_json::to_string(citation)?,
            ))?);
        }
        for term in declaration
            .before
            .iter()
            .chain(&declaration.after)
            .flat_map(|s| &s.terms)
            .map(|s| normalize_record_name(s))
            .filter(|s| !s.is_empty())
        {
            terms.insert(encoded((
                term,
                &declaration.context.namespace,
                &declaration.id,
            ))?);
        }
    }
    require_rows(
        c,
        "SELECT json_array(declaration_id,ordinal,input_key,source_record_id,payload_json) FROM remaster_citations",
        citations,
    )?;
    require_rows(
        c,
        "SELECT json_array(term,namespace,declaration_id) FROM remaster_terms",
        terms,
    )?;
    let mut bindings = BTreeSet::new();
    for b in &state.bindings {
        let (key, id, owners) = match &b.resolution {
            RemasterBindingResolution::Current { record_key, owners } => (
                Some(record_key.to_string()),
                ids.get(&record_key.to_string()).copied(),
                serde_json::to_string(owners)?,
            ),
            RemasterBindingResolution::MissingExplicitKey { record_key } => {
                (Some(record_key.to_string()), None, "[]".into())
            }
            _ => (None, None, "[]".into()),
        };
        bindings.insert(encoded((
            &b.declaration,
            side(b.side),
            b.group,
            b.reference,
            key,
            id,
            owners,
            serde_json::to_string(b)?,
        ))?);
    }
    require_rows(
        c,
        "SELECT json_array(declaration_id,side,group_ordinal,reference_ordinal,record_key,record_id,owners_json,payload_json) FROM remaster_bindings",
        bindings,
    )?;
    let mut aliases = BTreeSet::new();
    let mut alias_support = BTreeSet::new();
    let mut prefs = BTreeSet::new();
    let mut pref_support = BTreeSet::new();
    for a in &state.projection.aliases {
        aliases.insert(encoded((
            a.record.to_string(),
            &a.alias,
            normalize_record_name(&a.alias),
        ))?);
        for d in &a.declarations {
            alias_support.insert(encoded((a.record.to_string(), &a.alias, d))?);
        }
    }
    for p in &state.projection.preferences {
        prefs.insert(encoded((
            p.predecessor.to_string(),
            p.successor.to_string(),
        ))?);
        for d in &p.declarations {
            pref_support.insert(encoded((p.predecessor.to_string(), d))?);
        }
    }
    require_rows(
        c,
        "SELECT json_array(r.key,a.alias,a.alias_lookup_key) FROM remaster_aliases a JOIN records r USING(record_id)",
        aliases,
    )?;
    require_rows(
        c,
        "SELECT json_array(r.key,a.alias,a.declaration_id) FROM remaster_alias_support a JOIN records r USING(record_id)",
        alias_support,
    )?;
    require_rows(
        c,
        "SELECT json_array(l.key,r.key) FROM remaster_preferences p JOIN records l ON l.record_id=p.legacy_record_id JOIN records r ON r.record_id=p.remaster_record_id",
        prefs,
    )?;
    require_rows(
        c,
        "SELECT json_array(r.key,p.declaration_id) FROM remaster_preference_support p JOIN records r ON r.record_id=p.legacy_record_id",
        pref_support,
    )?;
    let digest: Option<String> = c
        .query_row(
            "SELECT digest FROM remaster_metadata WHERE singleton=1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if digest.as_deref() != Some(&state.digest()?) {
        return Err(invalid("remaster metadata digest differs"));
    }
    Ok(())
}

mod read;
