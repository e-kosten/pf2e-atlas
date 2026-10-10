//! Checked publication and explicit whole-artifact validation. Opening a reader is cheap.
use crate::{
    IndexError, SourceArtifactBuildContext, SourceArtifactRecordInput,
    codec::{checked_hash, gunzip, sha256},
    input::ARTIFACT_FORMAT_VERSION,
    projections::{ProjectionIds, root_rows},
};
use atlas_foundry_model::{SNAPSHOT_VERSION, decode_snapshot};
use atlas_record::{
    source_content::{
        ContentInteraction, OwnedContentLocator, SourceContentLocator, validate_prepared_markers,
        validate_prepared_reference_bindings,
    },
    source_record::SourceBackedRecord,
};
use rusqlite::{Connection, OpenFlags, params_from_iter};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactValidationReport {
    pub roots: usize,
    pub checked_snapshots: usize,
    pub projection_rows: usize,
    pub prepared_fields: usize,
    pub relationships: usize,
    pub lexical_units: usize,
    pub semantic_units: usize,
    pub source_fingerprint: String,
}

pub(crate) fn validate_open(c: &Connection) -> Result<SourceArtifactBuildContext, IndexError> {
    let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version != i64::from(ARTIFACT_FORMAT_VERSION) {
        return Err(IndexError::Unsupported(format!(
            "artifact schema version {version}"
        )));
    }
    let (format,hash,json):(i64,String,String)=c.query_row("SELECT format_version,context_hash,context_json FROM artifact_context WHERE singleton=1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|IndexError::Invalid(format!("artifact context: {e}")))?;
    if format != version || !checked_hash(&hash) || sha256(json.as_bytes()) != hash {
        return Err(IndexError::Invalid(
            "artifact context digest/version mismatch".into(),
        ));
    }
    let context: SourceArtifactBuildContext = serde_json::from_str(&json)?;
    crate::writer::validate_context(&context)?;
    // Reference schema is compiled code, never SQL supplied by artifact metadata.
    let expected = Connection::open_in_memory()?;
    expected.execute_batch(include_str!(
        "../migrations/00000000000001_create_artifact/up.sql"
    ))?;
    let schema = |c: &Connection| -> Result<BTreeMap<String, String>, IndexError> {
        Ok(c.prepare("SELECT name,sql FROM sqlite_master WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'lexical_fts_%' AND name NOT LIKE 'semantic_vectors%' ORDER BY name")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<Result<_,_>>()?)
    };
    if schema(c)? != schema(&expected)? {
        return Err(IndexError::Unsupported(
            "artifact tables/indexes differ from executable schema".into(),
        ));
    }
    let mut statement =
        c.prepare("SELECT field,definition_json FROM query_field_catalog ORDER BY field")?;
    let actual = statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let wanted = crate::query::query_capabilities()?
        .fields
        .into_iter()
        .map(|d| Ok((d.id.clone(), serde_json::to_string(&d)?)))
        .collect::<Result<BTreeMap<_, _>, IndexError>>()?;
    if actual != wanted {
        return Err(IndexError::Unsupported(
            "filter catalog differs from executable bindings".into(),
        ));
    }
    if let Some(model) = &context.semantic_model {
        let actual:(String,usize,i64)=c.query_row("SELECT identity_json,dimensions,unit_policy_version FROM semantic_models WHERE model_id=1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|bad(format!("active model metadata: {e}")))?;
        if serde_json::from_str::<crate::SourceSemanticModelIdentity>(&actual.0)? != *model
            || actual.1 != model.dimensions
            || actual.2 != 1
            || count(c, "semantic_models")? != 1
        {
            return Err(bad("active model metadata differs from context"));
        }
        let vector_sql: String = c
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name='semantic_vectors'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| bad(format!("active vector table: {e}")))?;
        if vector_sql
            != format!(
                "CREATE VIRTUAL TABLE semantic_vectors USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[{}] distance_metric=cosine)",
                model.dimensions
            )
        {
            return Err(bad(
                "vector table model dimensions or distance policy differs",
            ));
        }
    } else if count(c, "semantic_models")? != 0
        || count(c, "semantic_units")? != 0
        || c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='semantic_vectors')",
            [],
            |r| r.get::<_, bool>(0),
        )?
    {
        return Err(bad("semantic storage exists without context model"));
    }
    Ok(context)
}

pub fn validate_artifact(path: impl AsRef<Path>) -> Result<ArtifactValidationReport, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    c.set_prepared_statement_cache_capacity(128);
    let context = validate_open(&c)?;
    let integrity: String = c.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" {
        return Err(IndexError::Invalid(format!(
            "SQLite integrity: {integrity}"
        )));
    }
    if c.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(IndexError::Invalid("foreign key violation".into()));
    }
    let mut counts = BTreeMap::<&'static str, usize>::new();
    let mut ids = ProjectionIds::default();
    let mut roots = 0;
    let mut prepared_fields = 0;
    let mut relationships = 0;
    let mut root_query=c.prepare("SELECT r.record_id,r.key,r.pack_id,p.label,r.source_path,r.content_hash,b.codec_version,b.encoding,b.snapshot FROM records r JOIN packs p USING(pack_id) LEFT JOIN record_bodies b USING(record_id) ORDER BY r.record_id")?;
    // Retain only the existing resolver's compact checked identity/name index.
    // No source bodies or prose survive this first pass; later edge checks use
    // that same source-owner resolver instead of decoding a target per edge.
    let mut reference_index = atlas_record::source_record::SourceReferenceIndex::default();
    {
        let mut cursor = root_query.query([])?;
        while let Some(row) = cursor.next()? {
            let record = checked_record_row(row)?;
            reference_index.insert_source(record.key(), record.source());
        }
    }
    let mut cursor = root_query.query([])?;
    while let Some(row) = cursor.next()? {
        roots += 1;
        let id: i64 = row.get(0)?;
        let key: String = row.get(1)?;
        let label: String = row.get(3)?;
        let path: String = row.get(4)?;
        let hash: String = row.get(5)?;
        if !checked_hash(&hash)
            || Path::new(&path).is_absolute()
            || path.is_empty()
            || Path::new(&path)
                .components()
                .any(|p| !matches!(p, std::path::Component::Normal(_)))
        {
            return Err(IndexError::Invalid("invalid source provenance".into()));
        }
        let record = checked_record_row(row)?;
        let input = SourceArtifactRecordInput {
            record,
            source_path: path,
            content_hash: hash,
            preparation_context_hash: context.sha256()?,
            content: vec![],
            relationships: vec![],
            diagnostics: vec![],
        };
        for expected in root_rows(&input, &label, id, &mut ids)? {
            let sql = format!(
                "SELECT count(*) FROM {} WHERE {}",
                expected.table,
                expected
                    .columns
                    .iter()
                    .zip(&expected.values)
                    .map(|(s, v)| match v {
                        rusqlite::types::Value::Integer(_) =>
                            format!("{s} IS ? AND typeof({s})='integer'"),
                        rusqlite::types::Value::Real(_) =>
                            format!("{s} IS ? AND typeof({s})='real'"),
                        _ => format!("{s} IS ?"),
                    })
                    .collect::<Vec<_>>()
                    .join(" AND ")
            );
            let count: i64 = c
                .prepare_cached(&sql)?
                .query_row(params_from_iter(&expected.values), |r| r.get(0))?;
            if count != 1 {
                return Err(IndexError::Invalid(format!(
                    "typed source projection mismatch: {} root {key}",
                    expected.table
                )));
            }
            *counts.entry(expected.table).or_default() += 1;
        }
        prepared_fields += validate_content(&c, id, &input.record, &context)?;
        relationships += validate_relationships(&c, id, &input.record, &reference_index)?;
        crate::validation_units::validate_selection(&c, id, &input.record, &context, &label)?;
    }
    for table in PROJECTION_TABLES {
        if count(&c, table)? != counts.get(table).copied().unwrap_or(0) {
            return Err(IndexError::Invalid(format!(
                "extra/missing typed rows in {table}"
            )));
        }
    }
    if roots != count(&c, "record_bodies")? || roots > context.source_file_count {
        return Err(IndexError::Invalid(
            "snapshot/root/source count mismatch".into(),
        ));
    }
    let product_count: usize = c.query_row(
        "SELECT count(*) FROM records WHERE document_kind<>'Macro'",
        [],
        |r| r.get(0),
    )?;
    let root_lexical:usize=c.query_row("SELECT count(*) FROM records r WHERE r.document_kind<>'Macro' AND (SELECT count(*) FROM lexical_units u WHERE u.record_id=r.record_id AND u.unit_kind='root_name' AND u.owners_json='[]' AND u.field_path IS NULL AND u.section_json IS NULL)=1",[],|r|r.get(0))?;
    if root_lexical != product_count {
        return Err(IndexError::Invalid(
            "lexical identity coverage is incomplete".into(),
        ));
    }
    validate_fts(&c)?;
    validate_vectors(&c, &context, product_count)?;
    validate_aliases(&c)?;
    let mut urls=c.prepare("SELECT target_url FROM relationship_occurrences WHERE resolution IN ('resolved_url','unverified_url')")?;
    for url in urls.query_map([], |r| r.get::<_, String>(0))? {
        if !atlas_record::source_content::is_safe_content_url(&url?) {
            return Err(bad("relationship URL violates content policy"));
        }
    }
    Ok(ArtifactValidationReport {
        roots,
        checked_snapshots: roots,
        projection_rows: counts.values().sum(),
        prepared_fields,
        relationships,
        lexical_units: count(&c, "lexical_units")?,
        semantic_units: count(&c, "semantic_units")?,
        source_fingerprint: context.source_fingerprint,
    })
}
fn checked_record_row(row: &rusqlite::Row<'_>) -> Result<SourceBackedRecord, IndexError> {
    let key: String = row.get(1)?;
    let pack: String = row.get(2)?;
    let version: Option<i64> = row.get(6)?;
    let encoding: Option<String> = row.get(7)?;
    let body: Option<Vec<u8>> = row.get(8)?;
    if version != Some(i64::from(SNAPSHOT_VERSION)) || encoding.as_deref() != Some("gzip6") {
        return Err(bad(format!(
            "missing/unsupported checked snapshot for {key}"
        )));
    }
    let source = decode_snapshot(&gunzip(&body.ok_or_else(|| bad("missing snapshot"))?)?)?;
    let record = SourceBackedRecord::new(&pack, source)
        .map_err(|e| bad(format!("source identity {:?}", e.reason)))?;
    if record.key().to_string() != key {
        return Err(bad("body key differs from record row"));
    }
    Ok(record)
}
const PROJECTION_TABLES: &[&str] = &[
    "records",
    "record_traits",
    "actor_projection",
    "actor_items",
    "actor_item_traits",
    "actor_iwr_entries",
    "actor_languages",
    "actor_speeds",
    "actor_senses",
    "spell_projection",
    "spell_traditions",
    "spell_damage_entries",
    "spell_damage_kinds",
    "physical_projection",
    "weapon_projection",
    "armor_projection",
    "shield_projection",
    "ability_projection",
    "heritage_projection",
    "effect_projection",
    "condition_projection",
    "deity_projection",
    "deity_domains",
    "deity_fonts",
];
fn count(c: &Connection, table: &str) -> Result<usize, IndexError> {
    Ok(c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?)
}
fn bad(message: impl Into<String>) -> IndexError {
    IndexError::Invalid(message.into())
}
fn owners(s: &str) -> Result<Vec<OwnedContentLocator>, IndexError> {
    Ok(serde_json::from_str(s)?)
}
fn validate_content(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    context: &SourceArtifactBuildContext,
) -> Result<usize, IndexError> {
    let mut references = crate::read_content::content_references_for_record(c, id)?;
    let mut expected = std::collections::BTreeMap::new();
    let mut owner_error = None;
    record.visit_content_selections(|owners, selection| {
        if selection.text.value().is_some()
            && !matches!(
                selection.format,
                atlas_record::source_record::SourceFieldView::Value(
                    atlas_record::source_record::SourceContentFormat::Plain
                )
            )
            && selection
                .visibility
                .value()
                .is_some_and(|v| crate::writer::permits(context.audience, v))
        {
            match serde_json::to_string(owners) {
                Ok(owners) => {
                    expected.insert((owners, selection.field.to_owned()), selection);
                }
                Err(error) => owner_error = Some(error),
            }
        }
    });
    if let Some(error) = owner_error {
        return Err(error.into());
    }
    let mut s=c.prepare("SELECT owners_json,field_path,authored_markup_sha256,preparation_context_hash,outcome,html_gzip,interactions_json,visibility,role FROM prepared_content WHERE record_id=?")?;
    let mut rows = s.query([id])?;
    let mut n = 0;
    while let Some(r) = rows.next()? {
        n += 1;
        let chain = owners(&r.get::<_, String>(0)?)?;
        let field: String = r.get(1)?;
        let locator = SourceContentLocator {
            record: record.key().clone(),
            owners: chain,
            field,
        };
        let field_references = references
            .remove(&crate::read_content::content_reference_key(&locator)?)
            .unwrap_or_default();
        let selection = expected
            .remove(&(
                serde_json::to_string(&locator.owners)?,
                locator.field.clone(),
            ))
            .ok_or_else(|| bad("unexpected prepared cache field"))?;
        let authored = selection
            .text
            .value()
            .ok_or_else(|| bad("cache does not address admitted selected content"))?;
        let hash: String = r.get(2)?;
        let preparation: String = r.get(3)?;
        if hash != sha256(authored.as_bytes()) || preparation != context.sha256()? {
            return Err(bad("prepared cache input/context mismatch"));
        }
        let outcome: String = r.get(4)?;
        let bytes: Option<Vec<u8>> = r.get(5)?;
        let controls: Vec<ContentInteraction> = serde_json::from_str(&r.get::<_, String>(6)?)?;
        let visibility: atlas_record::source_content::ContentVisibilityRule =
            serde_json::from_str(&format!("\"{}\"", r.get::<_, String>(7)?))?;
        if !crate::writer::permits(context.audience, visibility) {
            return Err(bad("hidden cache in product content"));
        }
        let role: String = r.get(8)?;
        if selection.visibility.value() != Some(visibility)
            || role != serde_json::from_str::<String>(&serde_json::to_string(&selection.role)?)?
        {
            return Err(bad("cache declared role/visibility mismatch"));
        }
        use atlas_record::source_record::{SourceContentFormat, SourceFieldView};
        let correct_status = match selection.format {
            SourceFieldView::Value(SourceContentFormat::Html) => matches!(
                outcome.as_str(),
                "prepared" | "empty" | "preparation_failed"
            ),
            SourceFieldView::Value(SourceContentFormat::Plain) => false,
            SourceFieldView::Value(_) => outcome == "unsupported_format",
            _ => outcome == "format_unavailable",
        };
        if !correct_status {
            return Err(bad("cache outcome disagrees with authored format"));
        }
        match (outcome.as_str(), bytes) {
            ("prepared", Some(b)) => {
                let html = String::from_utf8(gunzip(&b)?).map_err(|e| bad(e.to_string()))?;
                validate_prepared_markers(&html, &controls).map_err(|e| bad(e.to_string()))?;
                validate_prepared_reference_bindings(&html, &locator, &hash, &field_references)
                    .map_err(|e| bad(e.to_string()))?;
            }
            ("prepared", None) => return Err(bad("missing prepared HTML")),
            (_, None) if controls.is_empty() && field_references.is_empty() => {}
            _ => return Err(bad("failed cache has HTML or controls")),
        }
    }
    if !references.is_empty() {
        return Err(bad("content reference has no selected prepared field"));
    }
    if !expected.is_empty() {
        return Err(bad("selected content cache coverage is incomplete"));
    }
    Ok(n)
}
fn validate_relationships(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    reference_index: &atlas_record::source_record::SourceReferenceIndex,
) -> Result<usize, IndexError> {
    use atlas_record::source_content::ContentReferenceResolution;
    let expected =
        atlas_record::source_record::resolve_source_relationships(record, Some(reference_index));
    let mut seen = vec![false; expected.len()];
    let mut statement = c.prepare(&format!(
        "{} WHERE o.record_id=?",
        crate::read_content::RELATIONSHIP_SELECT
    ))?;
    let mut n = 0;
    for stored in statement.query_map([id], crate::read_content::relationship_row)? {
        let stored = stored?;
        n += 1;
        if record.node_at(&stored.locator.owners).is_none() {
            return Err(bad("relationship owner is absent"));
        }
        if stored.origin == "content" {
            // Field-local complete coverage and digest binding were already
            // proved in validate_content; retain this shared kind decoder.
            crate::read_content::content_reference(stored)?;
            continue;
        }
        let source = expected
            .get(stored.ordinal)
            .ok_or_else(|| bad("unexpected structured relationship"))?;
        if seen[stored.ordinal]
            || source.locator != stored.locator
            || source.authored_target != stored.authored_target
            || stored.details.availability.as_ref() != Some(&source.availability)
            || stored.details.reference_kind.is_some()
            || stored.kind != serde_json::from_str::<String>(&serde_json::to_string(&source.kind)?)?
        {
            return Err(bad("structured edge attribution differs from typed source"));
        }
        // The build may have excluded unaddressable roots whose names would
        // otherwise look unique here. Do not promote an original unresolved
        // edge, but every stored resolved destination must match the existing
        // resolver reconstructed from admitted snapshot identities/names.
        if stored.resolution != ContentReferenceResolution::Unresolved
            && stored.resolution != source.resolution
        {
            return Err(bad(
                "resolved structured target differs from checked source resolver",
            ));
        }
        seen[stored.ordinal] = true;
    }
    if seen.iter().any(|v| !*v) {
        return Err(bad("structured relationship coverage is incomplete"));
    }
    // Validate incoming owned targets with this already-decoded target root.
    let mut statement=c.prepare("SELECT target_owners_json FROM relationship_occurrences WHERE target_record_id=? AND resolution='resolved_owned'")?;
    for chain in statement.query_map([id], |r| r.get::<_, String>(0))? {
        let chain = owners(&chain?)?;
        if chain.is_empty() || record.node_at(&chain).is_none() {
            return Err(bad("relationship owned target is absent"));
        }
    }
    Ok(n)
}
fn validate_aliases(c: &Connection) -> Result<(), IndexError> {
    let mut s=c.prepare("SELECT a.alias,a.alias_lookup_key,a.evidence_json,r.document_kind FROM verified_aliases a JOIN records r USING(record_id)")?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    })? {
        let (alias, key, evidence, kind) = row?;
        let evidence: String = serde_json::from_str(&evidence)?;
        if alias.trim().is_empty()
            || key != atlas_domain::normalize_record_name(&alias)
            || evidence.trim().is_empty()
            || kind == "Macro"
        {
            return Err(bad("invalid verified alias"));
        }
    }
    let invalid:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM remaster_pairs p JOIN records l ON l.record_id=p.legacy_record_id JOIN records r ON r.record_id=p.remaster_record_id WHERE l.document_kind='Macro' OR r.document_kind='Macro' OR json_type(p.evidence_json)<>'text' OR trim(json_extract(p.evidence_json,'$'))='')",[],|r|r.get(0))?;
    if invalid {
        return Err(bad("invalid remaster evidence"));
    }
    Ok(())
}
fn validate_vectors(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    product_count: usize,
) -> Result<(), IndexError> {
    let Some(model) = &context.semantic_model else {
        if count(c, "semantic_models")? != 0
            || count(c, "semantic_units")? != 0
            || c.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='semantic_vectors')",
                [],
                |r| r.get::<_, bool>(0),
            )?
        {
            return Err(bad("vectors exist without active model"));
        }
        return Ok(());
    };
    if count(c, "semantic_models")? != 1 {
        return Err(bad("active model identity missing"));
    }
    let (json, dimensions, policy): (String, usize, i64) = c.query_row(
        "SELECT identity_json,dimensions,unit_policy_version FROM semantic_models WHERE model_id=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    if serde_json::from_str::<crate::SourceSemanticModelIdentity>(&json)? != *model
        || dimensions != model.dimensions
        || policy != 1
    {
        return Err(bad("model identity metadata mismatch"));
    }
    let covered:usize=c.query_row("SELECT count(*) FROM records r WHERE r.document_kind<>'Macro' AND (SELECT count(*) FROM semantic_units s WHERE s.record_id=r.record_id AND s.unit_kind='identity' AND s.model_id=1)=1",[],|r|r.get(0))?;
    if covered != product_count {
        return Err(bad("semantic identity coverage is incomplete"));
    }
    if count(c, "semantic_vectors")? != count(c, "semantic_units")? {
        return Err(bad("vector/unit coverage mismatch"));
    }
    let mut s=c.prepare("SELECT u.input_token_count,u.input_hash,u.model_id,v.embedding FROM semantic_units u LEFT JOIN semantic_vectors v USING(unit_id)")?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, usize>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, Option<Vec<u8>>>(3)?,
        ))
    })? {
        let (tokens, hash, id, bytes) = row?;
        if tokens == 0 || tokens > model.token_limit || !checked_hash(&hash) || id != 1 {
            return Err(bad("invalid semantic input metadata"));
        }
        let vector = crate::read_units::decode_vector(
            &bytes.ok_or_else(|| bad("semantic unit vector missing"))?,
        )?;
        check_vector(&vector, dimensions)?;
    }
    Ok(())
}
pub(crate) fn check_vector(vector: &[f32], dimensions: usize) -> Result<(), IndexError> {
    if dimensions == 0 || vector.len() != dimensions || vector.iter().any(|v| !v.is_finite()) {
        return Err(bad("vector dimensions/finiteness mismatch"));
    }
    let squared = vector.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>();
    if !squared.is_finite() || (squared - 1.0).abs() > 1e-4 {
        return Err(bad("vector is not L2 normalized"));
    }
    Ok(())
}
// Compare index postings against SQLite's own tokenizer output, without writing the artifact.
fn validate_fts(c: &Connection) -> Result<(), IndexError> {
    let expected = Connection::open_in_memory()?;
    expected.execute_batch("CREATE VIRTUAL TABLE expected_fts USING fts5(identity_terms,alias_terms,structured_terms,definition_terms,tokenize='unicode61 remove_diacritics 2'); CREATE VIRTUAL TABLE expected_vocab USING fts5vocab(expected_fts,instance);")?;
    let mut s=c.prepare("SELECT unit_id,identity_terms,alias_terms,structured_terms,definition_terms FROM lexical_units")?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })? {
        let (id, a, b, d, e) = row?;
        expected.execute("INSERT INTO expected_fts(rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES(?,?,?,?,?)",rusqlite::params![id,a,b,d,e])?;
    }
    c.execute_batch(
        "CREATE VIRTUAL TABLE temp.actual_vocab USING fts5vocab(main,lexical_fts,instance);",
    )?;
    let mut actual =
        c.prepare("SELECT term,doc,col,offset FROM actual_vocab ORDER BY term,doc,col,offset")?;
    let mut wanted = expected
        .prepare("SELECT term,doc,col,offset FROM expected_vocab ORDER BY term,doc,col,offset")?;
    let mut a = actual.query([])?;
    let mut b = wanted.query([])?;
    loop {
        match (a.next()?, b.next()?) {
            (None, None) => break,
            (Some(a), Some(b)) => {
                let get = |r: &rusqlite::Row<'_>| -> rusqlite::Result<(String, i64, String, i64)> {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                };
                if get(a)? != get(b)? {
                    return Err(bad("FTS postings mismatch"));
                }
            }
            _ => return Err(bad("FTS postings coverage mismatch")),
        }
    }
    Ok(())
}
