//! Reconciliation of complete prepared input with the active reference revision.
//! Only changed owned rows are written; identity and mutable-domain rows survive.
use crate::{
    IndexError,
    codec::sha256,
    persistence::{ProjectionRow, execute},
    projections::ProjectionIds,
};
use atlas_domain::RecordKey;
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

pub(crate) fn allocate(
    c: &Connection,
    id: i64,
    required: i64,
) -> Result<ProjectionIds, IndexError> {
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
    c.execute("INSERT INTO projection_allocations(record_id,seed,end_id) VALUES(?,?,?) ON CONFLICT(record_id) DO UPDATE SET seed=excluded.seed,end_id=excluded.end_id",rusqlite::params![id,seed,end])?;
    Ok(ProjectionIds::seeded(seed))
}

pub(crate) fn clear_query(c: &Connection, id: i64) -> Result<(), IndexError> {
    // Family leaves cascade from their declared owner; incoming edges and semantic
    // occurrences are reconciled separately and never cascade with a replacement.
    c.execute("DELETE FROM actor_projection WHERE record_id=?", [id])?;
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
    input: &crate::PublicationSpool,
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

fn previous_semantic_slot(
    c: &Connection,
    slot: PreviousSemanticSlot<'_>,
    kept: &BTreeSet<i64>,
) -> Result<Option<i64>, IndexError> {
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
            if !kept.contains(&candidate) {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}

pub(crate) fn search_units(
    c: &Connection,
    input: &crate::PublicationSpool,
    ids: &BTreeMap<RecordKey, i64>,
    values: &crate::prepared_publication::PublicationValues<'_, '_>,
) -> Result<BTreeSet<i64>, IndexError> {
    let mut changed = lexical_stream(c, ids, &mut |visitor| input.lexical(visitor))?;
    let old_profiles = c
        .prepare("SELECT profile_id,model_id FROM search_profiles")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut current_profiles = BTreeMap::new();
    for model in input.context.canonical_profiles()? {
        let json = serde_json::to_string(&model)?;
        let model_id: i64 = c.query_row(
            "SELECT model_id FROM embedding_models WHERE identity_json=?",
            [json],
            |row| row.get(0),
        )?;
        current_profiles.insert(model.profile_id.to_string(), model_id);
        let mut row = ProjectionRow::new("search_profiles");
        row.text("profile_id", model.profile_id.as_str());
        row.id("model_id", model_id);
        upsert(c, &row, "profile_id")?;
    }
    if old_profiles != current_profiles {
        changed.extend(ids.values().copied());
    }
    let mut kept = BTreeSet::new();
    let mut next: i64 = c.query_row(
        "SELECT coalesce(max(unit_id),0) FROM semantic_units",
        [],
        |r| r.get(0),
    )?;
    for (profile, model_id) in &old_profiles {
        if current_profiles.get(profile) != Some(model_id) {
            c.execute("DELETE FROM semantic_units WHERE profile_id=?", [profile])?;
        }
    }
    input.semantic(&mut |unit| {
        // Values were installed once before reconciliation. Occurrences own only
        // attribution and a stable input handle, so this path never fetches BLOBs.
        let native = values.input_id(&unit.profile_id, &unit.input)?;
        let id = crate::writer::product_id(ids, &unit.record)?;
        let location = crate::semantic_locations::location_id(c, id, unit)?;
        let previous = if old_profiles.get(unit.profile_id.as_str())
            == current_profiles.get(unit.profile_id.as_str())
        {
            let section: Option<i64> = c.query_row(
                "SELECT section_id FROM semantic_locations WHERE location_id=?",
                [location],
                |r| r.get(0),
            )?;
            previous_semantic_slot(
                c,
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
        let unit_id = if let Some(id) = previous {
            id
        } else {
            next = next
                .checked_add(1)
                .ok_or_else(|| IndexError::Invalid("semantic ID overflow".into()))?;
            next
        };
        if !kept.insert(unit_id) {
            return Err(IndexError::Invalid("duplicate semantic occurrence".into()));
        }
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
        let before = c.total_changes();
        upsert(c, &row, "unit_id")?;
        if c.total_changes() != before {
            changed.insert(id);
        }
        Ok(())
    })?;
    remove_unkept_units(c, "semantic_units", &kept, &mut changed)?;
    for profile in old_profiles.keys() {
        if !current_profiles.contains_key(profile) {
            c.execute("DELETE FROM search_profiles WHERE profile_id=?", [profile])?;
        }
    }
    Ok(changed)
}

type LexicalVisitor<'a> = dyn FnMut(&crate::SourceLexicalUnitInput) -> Result<(), IndexError> + 'a;
type LexicalStream<'a> = dyn FnMut(&mut LexicalVisitor<'_>) -> Result<(), IndexError> + 'a;

pub(crate) fn lexical_stream(
    c: &Connection,
    ids: &BTreeMap<RecordKey, i64>,
    emit: &mut LexicalStream<'_>,
) -> Result<BTreeSet<i64>, IndexError> {
    let mut changed = BTreeSet::new();
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
        changed.insert(id);
        lexical.insert(c.last_insert_rowid());
        Ok(())
    })?;
    remove_unkept_units(c, "lexical_units", &lexical, &mut changed)?;

    Ok(changed)
}

fn remove_unkept_units(
    c: &Connection,
    table: &str,
    kept: &BTreeSet<i64>,
    changed: &mut BTreeSet<i64>,
) -> Result<(), IndexError> {
    let rows = c
        .prepare(&format!("SELECT unit_id,record_id FROM {table}"))?
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    for (unit, record) in rows {
        if !kept.contains(&unit) {
            c.execute(&format!("DELETE FROM {table} WHERE unit_id=?"), [unit])?;
            changed.insert(record);
        }
    }
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
    fn metadata_slot_priority_scope_kept_exclusion_and_null_sections_are_preserved() {
        let c = fixture();
        assert_eq!(
            previous_semantic_slot(&c, slot(), &BTreeSet::new()).unwrap(),
            Some(30)
        );
        assert_eq!(
            previous_semantic_slot(&c, slot(), &BTreeSet::from([30])).unwrap(),
            Some(10)
        );
        assert_eq!(
            previous_semantic_slot(&c, slot(), &BTreeSet::from([30, 10])).unwrap(),
            Some(20)
        );
        assert_eq!(
            previous_semantic_slot(&c, slot(), &BTreeSet::from([30, 10, 20])).unwrap(),
            None
        );
        for (record, profile, section, chunk, expected) in [
            (9, "profile", Some(8), 0, Some(1)),
            (7, "profile", Some(9), 0, Some(2)),
            (7, "profile", Some(8), 1, Some(3)),
            (7, "other-profile", Some(8), 0, Some(4)),
            (7, "profile", None, 0, None),
        ] {
            assert_eq!(
                previous_semantic_slot(
                    &c,
                    PreviousSemanticSlot {
                        record,
                        profile,
                        section,
                        chunk_ordinal: chunk,
                        location: 3
                    },
                    &BTreeSet::new()
                )
                .unwrap(),
                expected
            );
        }
        c.execute_batch("INSERT INTO semantic_locations VALUES(7,7,NULL,0),(8,9,NULL,0); INSERT INTO semantic_units VALUES(40,7,'profile'),(41,8,'profile'); DROP TABLE stored_vectors;").unwrap();
        assert_eq!(
            previous_semantic_slot(
                &c,
                PreviousSemanticSlot {
                    record: 7,
                    profile: "profile",
                    section: None,
                    chunk_ordinal: 0,
                    location: 7
                },
                &BTreeSet::new()
            )
            .unwrap(),
            Some(40)
        );
        // Numerical rows have separate ownership; metadata enumeration cannot
        // depend on, decode or silently skip a missing occurrence payload.
        assert_eq!(
            previous_semantic_slot(&c, slot(), &BTreeSet::new()).unwrap(),
            Some(30)
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
        let table = crate::schema::native_vector_table(model).unwrap();
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
        let identity: i64 = c.query_row("SELECT u.input_id FROM semantic_units u JOIN records r USING(record_id) WHERE r.key=? AND u.profile_id=? AND u.unit_kind='identity'", rusqlite::params![unit.record.to_string(), unit.profile_id.as_str()], |row| row.get(0)).unwrap();
        c.execute(
            &format!("UPDATE {table} SET embedding=? WHERE input_id=?"),
            rusqlite::params![vec![0_u8; model.dimensions * 4], identity],
        )
        .unwrap();
        drop(c);
        publish_input(&path, &input).unwrap();
        crate::validate_artifact(&path).unwrap();
        let c = Connection::open(&path).unwrap();
        let bytes: Vec<u8> = c
            .query_row(
                &format!("SELECT embedding FROM {table} WHERE input_id=?"),
                [identity],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(bytes, crate::read_units::vector_bytes(&unit.vector));
        let item = input.records[1].record.key();
        let obsolete: i64 = c.query_row("SELECT u.unit_id FROM semantic_units u JOIN records r USING(record_id) WHERE r.key=? AND u.profile_id=? AND u.unit_kind='passage' ORDER BY u.unit_id LIMIT 1", rusqlite::params![item.to_string(), model.profile_id.as_str()], |row| row.get(0)).unwrap();
        let obsolete_input: i64 = c
            .query_row(
                "SELECT input_id FROM semantic_units WHERE unit_id=?",
                [obsolete],
                |r| r.get(0),
            )
            .unwrap();
        c.execute(
            &format!("UPDATE {table} SET embedding=? WHERE input_id=?"),
            rusqlite::params![vec![0_u8; model.dimensions * 4], obsolete_input],
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
    fn unchanged_location_probe_uses_production_unique_keys_with_bounded_metadata_work() {
        use rusqlite::StatementStatus;
        let values=(1..=512).map(|id|("items","Item",serde_json::json!({
            "_id":format!("{id:016x}"),"name":format!("Record {id}"),"type":"effect",
            "system":{"description":{"value":"<h2>Rules</h2><p>Independent source prose.</p>"}}
        }))).collect();
        let input = crate::core_tests::fixture(values, false);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let model = &input.context.search_profiles[0];
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
            let found = previous_semantic_slot(
                &c,
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
            assert_eq!(found, unit);
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
