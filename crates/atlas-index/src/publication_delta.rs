//! Reconciliation of complete prepared input with the active reference revision.
//! Only changed owned rows are written; identity and mutable-domain rows survive.
use crate::{
    IndexBuildInput, IndexError, SourceArtifactRecordInput,
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
    Ok(sha256(&serde_json::to_vec(&(
        encode_snapshot(source.record.source())?,
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
    input: &IndexBuildInput,
    ids: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let mut kept = BTreeSet::new();
    for alias in &input.aliases {
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
    }
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
    for pair in &input.remaster_pairs {
        if pair.evidence.trim().is_empty() {
            return Err(IndexError::Invalid("remaster pair has no evidence".into()));
        }
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
        row.text("evidence_json", serde_json::to_string(&pair.evidence)?);
        upsert(c, &row, "legacy_record_id")?;
    }
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
fn nullable(value: Option<String>) -> Value {
    value.map(Value::Text).unwrap_or(Value::Null)
}

pub(crate) fn search_units(
    c: &Connection,
    input: &IndexBuildInput,
    ids: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let mut lexical = BTreeSet::new();
    for unit in &input.lexical_units {
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
            continue;
        }
        let mut row = ProjectionRow::new("lexical_units");
        row.text("input_hash", hash);
        row.id("record_id", id);
        row.text("owners_json", serde_json::to_string(&unit.owners)?);
        row.push("field_path", nullable(unit.field.clone()));
        row.push(
            "section_json",
            nullable(
                unit.address
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
            ),
        );
        row.text("unit_kind", unit.kind.as_str());
        row.text("identity_terms", &unit.identity_terms);
        row.text("alias_terms", &unit.alias_terms);
        row.text("structured_terms", &unit.structured_terms);
        row.text("definition_terms", &unit.definition_terms);
        row.insert(c)?;
        lexical.insert(c.last_insert_rowid());
    }
    remove_unkept(c, "lexical_units", "unit_id", &lexical)?;

    let old_profiles = c
        .prepare("SELECT profile_id,identity_json FROM search_profiles")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut current_profiles = BTreeMap::new();
    for model in input.context.canonical_profiles()? {
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
    for model in &input.context.search_profiles {
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
    let mut pending = Vec::new();
    for unit in &input.semantic_units {
        let model = input.context.search_profile(&unit.profile_id)?;
        crate::validation::check_vector(&unit.vector, model.dimensions)?;
        let id = crate::writer::product_id(ids, &unit.record)?;
        let bytes = crate::read_units::vector_bytes(&unit.vector);
        let table = crate::schema::vector_table(model)?;
        let candidates = if old_profiles.get(unit.profile_id.as_str())
            == current_profiles.get(unit.profile_id.as_str())
        {
            c.prepare_cached(&format!("SELECT u.unit_id,v.embedding FROM semantic_units u JOIN {table} v USING(unit_id) WHERE u.profile_id=? AND u.input_hash=? AND u.input_token_count=? AND u.record_id=? AND u.owners_json=? AND u.field_path IS ? AND u.section_json=? AND u.chunk_ordinal=? ORDER BY u.unit_id"))?.query_map(rusqlite::params![unit.profile_id.as_str(),unit.input_hash,unit.input_token_count as i64,id,serde_json::to_string(&unit.owners)?,unit.field,serde_json::to_string(&unit.address)?,unit.chunk_ordinal as i64],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Vec<u8>>(1)?)))?.collect::<Result<Vec<_>,_>>()?
        } else {
            Vec::new()
        };
        let reused = candidates
            .into_iter()
            .find(|(id, vector)| !kept.contains(id) && *vector == bytes)
            .map(|(id, _)| id);
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
        row.text("owners_json", serde_json::to_string(&unit.owners)?);
        row.push("field_path", nullable(unit.field.clone()));
        row.text("section_json", serde_json::to_string(&unit.address)?);
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
        row.id("chunk_ordinal", unit.chunk_ordinal as i64);
        row.id("input_token_count", unit.input_token_count as i64);
        row.text("input_hash", &unit.input_hash);
        pending.push((row, table, reused.is_none().then_some(bytes)));
    }
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
    for (row, table, vector) in pending {
        let unit_id = row.values[0].clone();
        upsert(c, &row, "unit_id")?;
        if let Some(vector) = vector {
            execute(
                c,
                &format!("INSERT INTO {table}(unit_id,embedding) VALUES(?,?)"),
                &[unit_id, Value::Blob(vector)],
            )?;
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
    Ok(())
}
