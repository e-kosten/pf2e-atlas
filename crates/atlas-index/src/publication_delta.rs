//! Reconciliation of complete prepared input with the active reference revision.
//! Only changed owned rows are written; identity and mutable-domain rows survive.
use crate::{
    IndexError, SourceArtifactRecordInput,
    codec::sha256,
    persistence::{ProjectionRow, execute},
    projections::{ProjectionIds, root_rows},
};
use atlas_domain::RecordKey;
use atlas_foundry_model::encode_snapshot;
use rusqlite::{Connection, OptionalExtension, types::Value};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn upsert(c: &Connection, row: &ProjectionRow, key: &str) -> Result<(), IndexError> {
    let changes = row
        .columns
        .iter()
        .filter(|column| !key.split(',').any(|k| k == column.as_str()))
        .map(|s| format!("{s}=excluded.{s}"))
        .collect::<Vec<_>>()
        .join(",");
    let different = row
        .columns
        .iter()
        .map(|s| format!("{}.{} IS NOT excluded.{s}", row.table, s))
        .collect::<Vec<_>>()
        .join(" OR ");
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT({key}) DO UPDATE SET {changes} WHERE {different}",
        row.table,
        row.columns.join(","),
        vec!["?"; row.values.len()].join(",")
    );
    execute(c, &sql, &row.values)?;
    Ok(())
}

pub(crate) fn root_digest(
    source: &SourceArtifactRecordInput,
    label: &str,
) -> Result<String, IndexError> {
    root_digest_encoded(source, label, &encode_snapshot(source.record.source())?)
}
pub(crate) fn root_digest_encoded(
    source: &SourceArtifactRecordInput,
    label: &str,
    snapshot: &[u8],
) -> Result<String, IndexError> {
    Ok(sha256(&serde_json::to_vec(&(
        snapshot,
        &source.source_path,
        &source.content_hash,
        &source.preparation_context_hash,
        &source.content,
        &source.relationships,
        &source.diagnostics,
        label,
        crate::input::SOURCE_PROJECTION_VERSION,
    ))?))
}

pub(crate) fn allocate(
    c: &Connection,
    source: &SourceArtifactRecordInput,
    label: &str,
    id: i64,
    digest: &str,
) -> Result<ProjectionIds, IndexError> {
    let mut counter = ProjectionIds::default();
    root_rows(source, label, id, &mut counter)?;
    let required = counter.end();
    let previous: Option<(i64, i64)> = c
        .query_row(
            "SELECT seed,end_id FROM projection_allocations WHERE record_id=?",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (seed, end) = match previous {
        Some((seed, end)) if end - seed >= required => (seed, end),
        _ => {
            let seed: i64 = c.query_row(
                "SELECT coalesce(max(end_id),0) FROM projection_allocations",
                [],
                |r| r.get(0),
            )?;
            (
                seed,
                seed.checked_add(required)
                    .ok_or_else(|| IndexError::Invalid("projection ID overflow".into()))?,
            )
        }
    };
    c.execute("INSERT INTO projection_allocations(record_id,seed,end_id,input_hash) VALUES(?,?,?,?) ON CONFLICT(record_id) DO UPDATE SET seed=excluded.seed,end_id=excluded.end_id,input_hash=excluded.input_hash",rusqlite::params![id,seed,end,digest])?;
    Ok(ProjectionIds::seeded(seed))
}

pub(crate) fn clear_query(c: &Connection, id: i64) -> Result<(), IndexError> {
    // Family leaves cascade from their declared owner; incoming edges and semantic
    // occurrences are reconciled separately and never cascade with a replacement.
    for table in ["record_traits", "actor_projection"] {
        c.execute(&format!("DELETE FROM {table} WHERE record_id=?"), [id])?;
    }
    for table in [
        "spell_projection",
        "physical_projection",
        "ability_projection",
        "heritage_projection",
        "effect_projection",
        "condition_projection",
        "deity_projection",
    ] {
        c.execute(&format!("DELETE FROM {table} WHERE root_record_id=?"), [id])?;
    }
    Ok(())
}

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn clear_root_derived(c: &Connection, id: i64) -> Result<(), IndexError> {
    // Family leaves cascade from their declared owner; incoming edges and semantic
    // occurrences are reconciled separately and never cascade with a replacement.
    for table in [
        "record_traits",
        "actor_projection",
        "record_bodies",
        "prepared_content",
        "developer_diagnostics",
        "relationship_occurrences",
    ] {
        c.execute(&format!("DELETE FROM {table} WHERE record_id=?"), [id])?;
    }
    for table in [
        "spell_projection",
        "physical_projection",
        "ability_projection",
        "heritage_projection",
        "effect_projection",
        "condition_projection",
        "deity_projection",
    ] {
        c.execute(&format!("DELETE FROM {table} WHERE root_record_id=?"), [id])?;
    }
    Ok(())
}

pub(crate) fn remove_absent_roots(c: &Connection, live: &BTreeSet<i64>) -> Result<(), IndexError> {
    let ids = c
        .prepare("SELECT record_id FROM records")?
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in ids {
        if !live.contains(&id) {
            // These associations reference the removable reference row, while
            // saved state references the enduring identity registry. Prune the
            // exact removed key before its deferred parent FK is deleted.
            c.execute("DELETE FROM trait_memberships WHERE record_key=(SELECT key FROM records WHERE record_id=?)",[id])?;
            c.execute("DELETE FROM trait_references WHERE record_key=(SELECT key FROM records WHERE record_id=?)",[id])?;
            c.execute("DELETE FROM records WHERE record_id=?", [id])?;
            c.execute(
                "UPDATE record_identity SET availability='unavailable' WHERE id=?",
                [id],
            )?;
        }
    }
    Ok(())
}

pub(crate) fn aliases(
    c: &Connection,
    input: &dyn crate::PublicationSource,
    ids: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let mut kept = BTreeSet::new();
    input.aliases(&mut |alias| {
        if alias.alias.trim().is_empty() || alias.evidence.trim().is_empty() {
            return Err(IndexError::Invalid("alias needs text and evidence".into()));
        }
        let id = crate::writer::product_id(ids, &alias.record)?;
        if !kept.insert((id, alias.alias.clone())) {
            return Err(IndexError::Invalid("duplicate alias".into()));
        }
        let mut row = ProjectionRow::new("verified_aliases");
        row.id("record_id", id);
        row.text("alias", &alias.alias);
        row.text(
            "alias_lookup_key",
            atlas_domain::normalize_record_name(&alias.alias),
        );
        row.text("evidence_json", serde_json::to_string(&alias.evidence)?);
        upsert(c, &row, "record_id,alias")?;
        Ok(())
    })?;
    for pair in c
        .prepare("SELECT record_id,alias FROM verified_aliases")?
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?
    {
        if !kept.contains(&pair) {
            c.execute(
                "DELETE FROM verified_aliases WHERE record_id=? AND alias=?",
                rusqlite::params![pair.0, pair.1],
            )?;
        }
    }
    let mut kept = BTreeSet::new();
    input.remaster_pairs(&mut |pair| {
        let legacy = crate::writer::product_id(ids, &pair.legacy)?;
        if !kept.insert(legacy) {
            return Err(IndexError::Invalid("duplicate remaster pair".into()));
        }
        let mut row = ProjectionRow::new("remaster_pairs");
        row.id("legacy_record_id", legacy);
        row.id(
            "remaster_record_id",
            crate::writer::product_id(ids, &pair.remaster)?,
        );
        row.text("evidence_json", serde_json::to_string(&pair.provenance)?);
        upsert(c, &row, "legacy_record_id")?;
        Ok(())
    })?;
    remove_unkept(c, "remaster_pairs", "legacy_record_id", &kept)?;
    Ok(())
}
fn remove_unkept(
    c: &Connection,
    table: &str,
    key: &str,
    kept: &BTreeSet<i64>,
) -> Result<(), IndexError> {
    let ids = c
        .prepare(&format!("SELECT {key} FROM {table}"))?
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in ids {
        if !kept.contains(&id) {
            c.execute(&format!("DELETE FROM {table} WHERE {key}=?"), [id])?;
        }
    }
    Ok(())
}

const PREVIOUS_EXACT_LOCATION_SQL: &str = "SELECT u.unit_id FROM semantic_units u JOIN semantic_locations l USING(location_id) WHERE u.location_id=?5 AND u.profile_id=?2 AND l.record_id=?1 AND l.section_id IS ?3 AND l.chunk_ordinal=?4 ORDER BY u.unit_id";
const PREVIOUS_OTHER_LOCATION_SQL: &str = "SELECT u.unit_id FROM semantic_locations l JOIN semantic_units u USING(location_id) WHERE l.record_id=?1 AND u.profile_id=?2 AND l.section_id IS ?3 AND l.chunk_ordinal=?4 AND l.location_id<>?5 ORDER BY u.unit_id";

struct PreviousSemanticSlot<'a> {
    record: i64,
    profile: &'a str,
    section: Option<i64>,
    chunk_ordinal: usize,
    location: i64,
}

fn previous_semantic_vector(
    c: &Connection,
    table: &str,
    slot: PreviousSemanticSlot<'_>,
    kept: &BTreeSet<i64>,
) -> Result<Option<(i64, Vec<u8>)>, IndexError> {
    let mut vector = c.prepare_cached(&format!("SELECT embedding FROM {table} WHERE unit_id=?"))?;
    for sql in [PREVIOUS_EXACT_LOCATION_SQL, PREVIOUS_OTHER_LOCATION_SQL] {
        let mut candidates = c.prepare_cached(sql)?;
        let mut rows = candidates.query(rusqlite::params![
            slot.record,
            slot.profile,
            slot.section,
            slot.chunk_ordinal,
            slot.location
        ])?;
        while let Some(row) = rows.next()? {
            let candidate: i64 = row.get(0)?;
            if kept.contains(&candidate) {
                continue;
            }
            let bytes = vector
                .query_row([candidate], |row| row.get::<_, Vec<u8>>(0))
                .optional()?;
            if let Some(bytes) = bytes {
                return Ok(Some((candidate, bytes)));
            }
        }
    }
    Ok(None)
}

pub(crate) fn search_units(
    c: &Connection,
    input: &dyn crate::PublicationSource,
    ids: &BTreeMap<RecordKey, i64>,
    values: &dyn crate::NativeValueSource,
) -> Result<(), IndexError> {
    lexical_stream(c, ids, &mut |visitor| input.lexical(visitor))?;

    let old_profiles = c
        .prepare("SELECT profile_id,identity_json FROM search_profiles")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut current_profiles = BTreeMap::new();
    for model in input.context().canonical_profiles()? {
        let json = serde_json::to_string(&model)?;
        current_profiles.insert(model.profile_id.to_string(), json.clone());
        let mut row = ProjectionRow::new("search_profiles");
        row.text("profile_id", model.profile_id.as_str());
        row.text("identity_json", json);
        row.id("dimensions", model.dimensions as i64);
        upsert(c, &row, "profile_id")?;
    }
    let mut old_tables = BTreeMap::new();
    for (profile, json) in &old_profiles {
        let model = serde_json::from_str::<crate::SourceSearchProfileIdentity>(json)?;
        old_tables.insert(profile.clone(), crate::schema::vector_table(&model)?);
    }
    let mut current_tables = BTreeSet::new();
    for model in &input.context().search_profiles {
        let table = crate::schema::vector_table(model)?;
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?)",
            [&table],
            |row| row.get(0),
        )?;
        if !exists {
            c.execute_batch(&crate::schema::vector_ddl(model)?)?;
        }
        current_tables.insert(table);
    }
    let mut kept = BTreeSet::new();
    let mut next: i64 = c.query_row(
        "SELECT coalesce(max(unit_id),0) FROM semantic_units",
        [],
        |r| r.get(0),
    )?;
    for (profile, json) in &old_profiles {
        if current_profiles.get(profile) != Some(json) {
            let table = &old_tables[profile];
            c.execute(&format!("DELETE FROM {table}"), [])?;
            c.execute("DELETE FROM semantic_units WHERE profile_id=?", [profile])?;
        }
    }
    input.semantic(&mut |occurrence| {
        let model = input.context().search_profile(&occurrence.profile_id)?;
        let value = values.value(model, &occurrence.input)?;
        let unit = occurrence;
        crate::validation::check_vector(value.as_ref(), model.dimensions)?;
        let id = crate::writer::product_id(ids, &unit.record)?;
        let table = crate::schema::vector_table(model)?;
        let location = crate::semantic_locations::location_id(c, id, unit)?;
        let native = crate::embedding_values::publish_input(
            c,
            model,
            &unit.input.input_sha256,
            unit.input.token_count,
            value.as_ref(),
        )?;
        let previous: Option<(i64, Vec<u8>)> = if old_profiles.get(unit.profile_id.as_str())
            == current_profiles.get(unit.profile_id.as_str())
        {
            let section: Option<i64> = c.query_row(
                "SELECT section_id FROM semantic_locations WHERE location_id=?",
                [location],
                |row| row.get(0),
            )?;
            previous_semantic_vector(
                c,
                &table,
                PreviousSemanticSlot {
                    record: id,
                    profile: unit.profile_id.as_str(),
                    section,
                    chunk_ordinal: unit.chunk_ordinal,
                    location,
                },
                &kept,
            )?
        } else {
            None
        };
        let reused = previous.as_ref().map(|(id, _)| *id);
        let bytes = crate::read_units::vector_bytes(value.as_ref());
        let changed = previous.as_ref().is_none_or(|(_, old)| *old != bytes);
        let unit_id = if let Some(id) = reused {
            id
        } else {
            next = next
                .checked_add(1)
                .ok_or_else(|| IndexError::Invalid("semantic ID overflow".into()))?;
            next
        };
        kept.insert(unit_id);
        let mut row = ProjectionRow::new("semantic_units");
        row.id("unit_id", unit_id);
        row.id("record_id", id);
        row.text("profile_id", unit.profile_id.as_str());
        row.id("location_id", location);
        row.id("input_id", native);
        row.text(
            "unit_kind",
            if matches!(
                unit.address,
                atlas_domain::SourcePassageAddress::Identity {}
            ) {
                "identity"
            } else {
                "passage"
            },
        );
        upsert(c, &row, "unit_id")?;
        if changed {
            execute(
                c,
                &if reused.is_some() {
                    format!("UPDATE {table} SET embedding=?2 WHERE unit_id=?1")
                } else {
                    format!("INSERT INTO {table}(unit_id,embedding) VALUES(?1,?2)")
                },
                &[Value::Integer(unit_id), Value::Blob(bytes)],
            )?;
        }
        Ok(())
    })?;
    let old_ids = c
        .prepare("SELECT unit_id,profile_id FROM semantic_units")?
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, profile) in old_ids {
        if !kept.contains(&id) {
            let table = old_tables.get(&profile).ok_or_else(|| {
                IndexError::Invalid("semantic profile has no vector table".into())
            })?;
            c.execute(&format!("DELETE FROM {table} WHERE unit_id=?"), [id])?;
            c.execute("DELETE FROM semantic_units WHERE unit_id=?", [id])?;
        }
    }
    for profile in old_profiles.keys() {
        if !current_profiles.contains_key(profile) {
            c.execute("DELETE FROM search_profiles WHERE profile_id=?", [profile])?;
        }
    }
    for table in old_tables.values() {
        if !current_tables.contains(table) {
            c.execute_batch(&format!("DROP TABLE {table}"))?;
        }
    }
    for model in &input.context().search_profiles {
        crate::vector_allocation::compact(c, model)?;
    }
    Ok(())
}

type LexicalVisitor<'a> = dyn FnMut(&crate::SourceLexicalUnitInput) -> Result<(), IndexError> + 'a;
type LexicalStream<'a> = dyn FnMut(&mut LexicalVisitor<'_>) -> Result<(), IndexError> + 'a;

pub(crate) fn lexical_stream(
    c: &Connection,
    ids: &BTreeMap<RecordKey, i64>,
    emit: &mut LexicalStream<'_>,
) -> Result<(), IndexError> {
    let mut lexical = BTreeSet::new();
    emit(&mut |unit| {
        let id = crate::writer::product_id(ids, &unit.record)?;
        let hash = sha256(&serde_json::to_vec(unit)?);
        let existing: Option<i64> = c
            .query_row(
                "SELECT unit_id FROM lexical_units WHERE input_hash=?",
                [&hash],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(unit_id) = existing {
            if !lexical.insert(unit_id) {
                return Err(IndexError::Invalid("duplicate lexical unit".into()));
            }
            return Ok(());
        }
        let mut row = ProjectionRow::new("lexical_units");
        row.text("input_hash", hash);
        row.id("record_id", id);
        let section = match &unit.address {
            Some(address) => {
                let ordinal = match address {
                    atlas_domain::SourcePassageAddress::HtmlSection {
                        section_ordinal, ..
                    }
                    | atlas_domain::SourcePassageAddress::PlainSection {
                        section_ordinal, ..
                    } => *section_ordinal,
                    atlas_domain::SourcePassageAddress::Identity {} => {
                        return Err(IndexError::Invalid(
                            "lexical identity has a fabricated section".into(),
                        ));
                    }
                };
                let field = unit
                    .field
                    .as_deref()
                    .ok_or_else(|| IndexError::Invalid("lexical section field missing".into()))?;
                let section = crate::selected_sections::find(
                    c,
                    id,
                    &serde_json::to_string(&unit.owners)?,
                    field,
                    ordinal,
                )?;
                if section.address != *address {
                    return Err(IndexError::Invalid(
                        "lexical section differs from canonical whole address".into(),
                    ));
                }
                Some(section.id)
            }
            None => None,
        };
        row.push(
            "section_id",
            section.map(Value::Integer).unwrap_or(Value::Null),
        );
        row.push(
            "owners_json",
            if section.is_none() {
                Value::Text(serde_json::to_string(&unit.owners)?)
            } else {
                Value::Null
            },
        );
        row.text("unit_kind", unit.kind.as_str());
        row.text("identity_terms", &unit.identity_terms);
        row.text("alias_terms", &unit.alias_terms);
        row.text("structured_terms", &unit.structured_terms);
        row.text("definition_terms", &unit.definition_terms);
        row.insert(c)?;
        lexical.insert(c.last_insert_rowid());
        Ok(())
    })?;
    remove_unkept(c, "lexical_units", "unit_id", &lexical)?;

    Ok(())
}

#[cfg(test)]
mod previous_semantic_tests {
    use super::*;

    fn fixture() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE semantic_locations(location_id INTEGER PRIMARY KEY,record_id INTEGER,section_id INTEGER,chunk_ordinal INTEGER);
             CREATE TABLE semantic_units(unit_id INTEGER PRIMARY KEY,location_id INTEGER,profile_id TEXT);
             CREATE TABLE stored_vectors(unit_id INTEGER PRIMARY KEY,embedding BLOB);
             INSERT INTO semantic_locations VALUES(1,7,8,0),(2,7,8,0),(3,7,8,0),(4,9,8,0),(5,7,9,0),(6,7,8,1);
             INSERT INTO semantic_units VALUES(10,1,'profile'),(20,2,'profile'),(30,3,'profile'),(1,4,'profile'),(2,5,'profile'),(3,6,'profile'),(4,1,'other-profile');
             INSERT INTO stored_vectors VALUES(10,x'10'),(20,x'20'),(30,x'30'),(1,x'01'),(2,x'02'),(3,x'03'),(4,x'04');"
        ).unwrap();
        c
    }

    fn slot() -> PreviousSemanticSlot<'static> {
        PreviousSemanticSlot {
            record: 7,
            profile: "profile",
            section: Some(8),
            chunk_ordinal: 0,
            location: 3,
        }
    }

    #[test]
    fn exact_location_precedes_lower_ids_and_keeps_scope() {
        let c = fixture();
        assert_eq!(
            previous_semantic_vector(&c, "stored_vectors", slot(), &BTreeSet::new()).unwrap(),
            Some((30, vec![0x30]))
        );
        assert_eq!(
            previous_semantic_vector(&c, "stored_vectors", slot(), &BTreeSet::from([30])).unwrap(),
            Some((10, vec![0x10]))
        );
    }

    #[test]
    fn kept_and_later_candidates_do_not_evaluate_vector_payloads() {
        let c = fixture();
        c.execute_batch("CREATE VIEW observed_vectors AS SELECT unit_id, CASE WHEN unit_id IN (20,30) THEN json('invalid-json') ELSE embedding END AS embedding FROM stored_vectors;").unwrap();
        assert!(
            c.query_row(
                "SELECT embedding FROM observed_vectors WHERE unit_id=30",
                [],
                |row| row.get::<_, Vec<u8>>(0)
            )
            .is_err()
        );
        assert_eq!(
            previous_semantic_vector(&c, "observed_vectors", slot(), &BTreeSet::from([30]))
                .unwrap(),
            Some((10, vec![0x10]))
        );
    }

    #[test]
    fn missing_vectors_fall_back_in_order_and_all_missing_returns_none() {
        let c = fixture();
        for (removed, expected) in [
            (30, Some((10, vec![0x10]))),
            (10, Some((20, vec![0x20]))),
            (20, None),
        ] {
            c.execute("DELETE FROM stored_vectors WHERE unit_id=?", [removed])
                .unwrap();
            assert_eq!(
                previous_semantic_vector(&c, "stored_vectors", slot(), &BTreeSet::new()).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn candidate_comparison_reads_exact_bytes_across_native_chunks() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let c = fixture();
        c.execute_batch("CREATE VIRTUAL TABLE native_vectors USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[8],chunk_size=8)").unwrap();
        for unit in 1..=33_u8 {
            let mut vector = [0_f32; 8];
            vector[usize::from(unit) % 8] = 1.;
            c.execute(
                "INSERT INTO native_vectors(unit_id,embedding) VALUES(?,?)",
                rusqlite::params![i64::from(unit), crate::read_units::vector_bytes(&vector)],
            )
            .unwrap();
        }
        let mut expected = [0_f32; 8];
        expected[30 % 8] = 1.;
        assert_eq!(
            previous_semantic_vector(&c, "native_vectors", slot(), &BTreeSet::new()).unwrap(),
            Some((30, crate::read_units::vector_bytes(&expected)))
        );
        c.execute("DELETE FROM native_vectors WHERE unit_id=30", [])
            .unwrap();
        expected.fill(0.);
        expected[10 % 8] = 1.;
        assert_eq!(
            previous_semantic_vector(&c, "native_vectors", slot(), &BTreeSet::new()).unwrap(),
            Some((10, crate::read_units::vector_bytes(&expected)))
        );
    }
    #[test]
    fn publication_repairs_selected_old_bytes_and_removes_obsolete_corrupt_slots() {
        use crate::test_support::*;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("native.sqlite");
        let input = crate::foundation_tests::fixture(true);
        publish_input(&path, &input).unwrap();
        let model = &input.context.search_profiles[0];
        let table = crate::schema::vector_table(model).unwrap();
        let unit = input
            .semantic_units
            .iter()
            .find(|unit| {
                matches!(
                    unit.address,
                    atlas_domain::SourcePassageAddress::Identity {}
                )
            })
            .unwrap();
        let c = Connection::open(&path).unwrap();
        let identity: i64 = c.query_row("SELECT u.unit_id FROM semantic_units u JOIN records r USING(record_id) WHERE r.key=? AND u.profile_id=? AND u.unit_kind='identity'", rusqlite::params![unit.record.to_string(), unit.profile_id.as_str()], |row| row.get(0)).unwrap();
        c.execute(
            &format!("UPDATE {table} SET embedding=? WHERE unit_id=?"),
            rusqlite::params![vec![0_u8; model.dimensions * 4], identity],
        )
        .unwrap();
        drop(c);
        publish_input(&path, &input).unwrap();
        crate::validate_artifact(&path).unwrap();
        let c = Connection::open(&path).unwrap();
        let bytes: Vec<u8> = c
            .query_row(
                &format!("SELECT embedding FROM {table} WHERE unit_id=?"),
                [identity],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(bytes, crate::read_units::vector_bytes(&unit.vector));
        let item = input.records[1].record.key();
        let obsolete: i64 = c.query_row("SELECT u.unit_id FROM semantic_units u JOIN records r USING(record_id) WHERE r.key=? AND u.profile_id=? AND u.unit_kind='passage' ORDER BY u.unit_id LIMIT 1", rusqlite::params![item.to_string(), model.profile_id.as_str()], |row| row.get(0)).unwrap();
        c.execute(
            &format!("UPDATE {table} SET embedding=? WHERE unit_id=?"),
            rusqlite::params![vec![0_u8; model.dimensions * 4], obsolete],
        )
        .unwrap();
        drop(c);
        let next = crate::foundation_tests::fixture_with_description(true, "");
        publish_input(&path, &next).unwrap();
        crate::validate_artifact(&path).unwrap();
        let c = Connection::open(&path).unwrap();
        let retained: usize = c
            .query_row(
                "SELECT count(*) FROM semantic_units WHERE unit_id=?",
                [obsolete],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(retained, 0);
    }
    #[test]
    fn exact_location_scope_and_remaining_group_preserve_all_original_predicates() {
        let c = fixture();
        for (record, profile, section, chunk, expected) in [
            (9, "profile", Some(8), 0, Some(1)),
            (7, "profile", Some(9), 0, Some(2)),
            (7, "profile", Some(8), 1, Some(3)),
            (7, "other-profile", Some(8), 0, Some(4)),
            (7, "profile", None, 0, None),
        ] {
            let candidate = previous_semantic_vector(
                &c,
                "stored_vectors",
                PreviousSemanticSlot {
                    record,
                    profile,
                    section,
                    chunk_ordinal: chunk,
                    location: 3,
                },
                &BTreeSet::new(),
            )
            .unwrap();
            assert_eq!(candidate.map(|(id, _)| id), expected);
        }
        c.execute_batch("INSERT INTO semantic_locations VALUES(7,7,NULL,0),(8,9,NULL,0); INSERT INTO semantic_units VALUES(40,7,'profile'),(41,8,'profile'); INSERT INTO stored_vectors VALUES(40,x'40'),(41,x'41');").unwrap();
        for (record, expected) in [(7, 40), (9, 41)] {
            assert_eq!(
                previous_semantic_vector(
                    &c,
                    "stored_vectors",
                    PreviousSemanticSlot {
                        record,
                        profile: "profile",
                        section: None,
                        chunk_ordinal: 0,
                        location: 7,
                    },
                    &BTreeSet::new()
                )
                .unwrap()
                .unwrap()
                .0,
                expected
            );
        }
    }

    #[test]
    fn exact_group_keeps_unit_order_before_any_remaining_location() {
        let c = fixture();
        c.execute_batch("INSERT INTO semantic_units VALUES(35,3,'profile'); INSERT INTO stored_vectors VALUES(35,x'35');").unwrap();
        for (kept, expected) in [
            (BTreeSet::new(), 30),
            (BTreeSet::from([30]), 35),
            (BTreeSet::from([30, 35]), 10),
        ] {
            assert_eq!(
                previous_semantic_vector(&c, "stored_vectors", slot(), &kept)
                    .unwrap()
                    .unwrap()
                    .0,
                expected
            );
        }
        c.execute("DELETE FROM stored_vectors WHERE unit_id=30", [])
            .unwrap();
        assert_eq!(
            previous_semantic_vector(&c, "stored_vectors", slot(), &BTreeSet::new())
                .unwrap()
                .unwrap()
                .0,
            35
        );
        c.execute("DELETE FROM stored_vectors WHERE unit_id=35", [])
            .unwrap();
        assert_eq!(
            previous_semantic_vector(&c, "stored_vectors", slot(), &BTreeSet::new())
                .unwrap()
                .unwrap()
                .0,
            10
        );
    }

    #[test]
    fn unchanged_location_probe_uses_production_unique_keys_with_bounded_metadata_work() {
        use rusqlite::StatementStatus;
        let values=(1..=512).map(|id|("items","Item",serde_json::json!({
            "_id":format!("{id:016x}"),"name":format!("Record {id}"),"type":"effect",
            "system":{"description":{"value":"<h2>Rules</h2><p>Independent source prose.</p>"}}
        }))).collect();
        let input = crate::core_tests::fixture(values, false);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let model = &input.context.search_profiles[0];
        let table = crate::schema::vector_table(model).unwrap();
        let key = input.records[256].record.key().to_string();
        for (kind, section_expected) in [("identity", false), ("passage", true)] {
            // A fresh connection gives counters for this single real helper call.
            let c = Connection::open(&path).unwrap();
            let (unit,location,section,ordinal,record):(i64,i64,Option<i64>,usize,i64)=c.query_row(
                "SELECT u.unit_id,l.location_id,l.section_id,l.chunk_ordinal,l.record_id FROM semantic_units u JOIN semantic_locations l USING(location_id) JOIN records r ON r.record_id=l.record_id WHERE r.key=?1 AND u.profile_id=?2 AND u.unit_kind=?3 ORDER BY u.unit_id LIMIT 1",
                rusqlite::params![key,model.profile_id.as_str(),kind],
                |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?))
            ).unwrap();
            assert_eq!(section.is_some(), section_expected);
            let plan = c
                .prepare(&format!("EXPLAIN QUERY PLAN {PREVIOUS_EXACT_LOCATION_SQL}"))
                .unwrap()
                .query_map(
                    rusqlite::params![
                        record,
                        model.profile_id.as_str(),
                        section,
                        ordinal,
                        location
                    ],
                    |row| row.get::<_, String>(3),
                )
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert!(
                plan.iter().any(|detail| detail.contains("SEARCH")
                    && detail.contains("location_id=?")
                    && detail.contains("profile_id=?")),
                "expected existing location/profile index access: {plan:?}"
            );
            let found = previous_semantic_vector(
                &c,
                &table,
                PreviousSemanticSlot {
                    record,
                    profile: model.profile_id.as_str(),
                    section,
                    chunk_ordinal: ordinal,
                    location,
                },
                &BTreeSet::new(),
            )
            .unwrap()
            .unwrap();
            assert_eq!(found.0, unit);
            let exact = c.prepare_cached(PREVIOUS_EXACT_LOCATION_SQL).unwrap();
            let vm = exact.get_status(StatementStatus::VmStep);
            assert!(
                vm > 0 && vm <= 200,
                "exact metadata probe must execute bounded point work: {vm}"
            );
            assert_eq!(exact.get_status(StatementStatus::FullscanStep), 0);
            assert_eq!(exact.get_status(StatementStatus::Sort), 0);
            assert_eq!(exact.get_status(StatementStatus::AutoIndex), 0);
            drop(exact);
            let fallback = c.prepare_cached(PREVIOUS_OTHER_LOCATION_SQL).unwrap();
            assert_eq!(
                fallback.get_status(StatementStatus::VmStep),
                0,
                "unchanged exact hit must not execute the broad fallback"
            );
            eprintln!(
                "candidate_metadata_probe kind={kind} exact_vm={vm} fullscan=0 sort=0 autoindex=0 fallback_vm={}",
                fallback.get_status(StatementStatus::VmStep)
            );
        }
    }
}
