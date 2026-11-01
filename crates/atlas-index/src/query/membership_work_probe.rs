//! Opt-in SQL-work evidence over an explicitly supplied, already-current corpus.
use super::{ValidatedQuery, compiler::compile_predicate, parse_where, validate_query};
use atlas_domain::{QueryExpression, QueryPredicate, QuerySetMatch, RecordKey};
use rusqlite::{Connection, StatementStatus, params_from_iter, types::Value};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

type ProbeResult<T> = Result<T, Box<dyn Error>>;

struct Case {
    name: &'static str,
    query: ValidatedQuery,
    keys: Option<Vec<RecordKey>>,
}

#[derive(Serialize)]
struct Work {
    vm_steps: i32,
    fullscan_steps: i32,
    sort_operations: i32,
    autoindex_rows: i32,
    reprepares: i32,
}

#[derive(Serialize)]
struct Measurement {
    round: usize,
    position: usize,
    prepare_seconds: f64,
    execute_decode_collect_seconds: f64,
    emitted_rows: usize,
    work: Work,
}

#[derive(Serialize)]
struct ExplainStep {
    id: i64,
    parent: i64,
    detail: String,
}

#[derive(Serialize)]
struct PlanEvidence {
    lowering: &'static str,
    sql: String,
    sqlite_bind_debug: Vec<String>,
    explain: Vec<ExplainStep>,
    accepted_keys: Vec<String>,
    accepted_keys_sha256: String,
    samples: Vec<Measurement>,
}

#[derive(Serialize)]
struct CaseEvidence {
    name: &'static str,
    predicate: QueryPredicate,
    key_scope: Option<Vec<RecordKey>>,
    accepted_key_parity: bool,
    repeated_results_stable: bool,
    plans: [PlanEvidence; 2],
}

fn cases(restricted_keys: &[RecordKey]) -> Result<Vec<Case>, crate::IndexError> {
    let mut cases = [
        ("frozen_ac", "defenses.ac>=40"),
        ("frozen_level_ac", "level>=15 && defenses.ac>=40"),
        ("frozen_price", "physical.unit_price_gp>=10000"),
        (
            "frozen_owned_spell",
            "embedded.spells.exists(s,s.spell.rank>=6)",
        ),
        ("ac_selective", "defenses.ac>=50"),
        ("ac_broad", "defenses.ac>=0"),
        ("level_selective", "level>=20"),
        ("level_broad", "level>=0"),
        ("price_broad", "physical.unit_price_gp>=0"),
        ("trait_selective", "'incorporeal' in traits"),
        (
            "same_child_fallback",
            "embedded.spells.exists(s,s.spell.rank>=6 && 'arcane' in s.spell.traditions)",
        ),
        ("all_roots_control", "true"),
    ]
    .into_iter()
    .map(|(name, expression)| {
        Ok(Case {
            name,
            query: parse_where(expression)?,
            keys: None,
        })
    })
    .collect::<Result<Vec<_>, crate::IndexError>>()?;
    cases.push(Case {
        name: "trait_broad_includes_common",
        query: validate_query(&QueryPredicate::new(QueryExpression::SetMatch {
            field: "traits".into(),
            op: QuerySetMatch::IncludesAny,
            values: [
                "magical",
                "attack",
                "concentrate",
                "manipulate",
                "mental",
                "humanoid",
                "auditory",
                "visual",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        }))?,
        keys: None,
    });
    for (name, keys) in [
        ("ac_restricted_keys", restricted_keys.to_vec()),
        ("ac_empty_keys", vec![]),
    ] {
        cases.push(Case {
            name,
            query: parse_where("defenses.ac>=40")?,
            keys: Some(keys),
        });
    }
    Ok(cases)
}

fn general_sql(case: &Case) -> Result<(String, Vec<Value>), crate::IndexError> {
    let compiled = compile_predicate(&case.query)?;
    // compile_predicate includes the same Macro exclusion as eligible_sql.
    let mut sql = format!(
        "SELECT r.record_id FROM records r WHERE ({}) IS TRUE",
        compiled.expression
    );
    let mut values = compiled.parameters;
    if let Some(keys) = &case.keys {
        if keys.is_empty() {
            sql.push_str(" AND 0");
        } else {
            sql.push_str(&format!(
                " AND r.key IN ({})",
                vec!["?"; keys.len()].join(",")
            ));
            values.extend(keys.iter().map(|key| Value::Text(key.to_string())));
        }
    }
    Ok((sql, values))
}

fn run_sql(
    c: &Connection,
    sql: &str,
    values: &[Value],
    round: usize,
    position: usize,
) -> rusqlite::Result<(Vec<i64>, Measurement)> {
    let start = Instant::now();
    let mut statement = c.prepare(sql)?;
    let prepare_seconds = start.elapsed().as_secs_f64();
    for counter in [
        StatementStatus::VmStep,
        StatementStatus::FullscanStep,
        StatementStatus::Sort,
        StatementStatus::AutoIndex,
        StatementStatus::RePrepare,
    ] {
        statement.reset_status(counter);
    }
    let start = Instant::now();
    let mut rows = statement.query(params_from_iter(values))?;
    let mut ids = Vec::new();
    while let Some(row) = rows.next()? {
        ids.push(row.get(0)?);
    }
    drop(rows);
    let execute_decode_collect_seconds = start.elapsed().as_secs_f64();
    let work = Work {
        vm_steps: statement.get_status(StatementStatus::VmStep),
        fullscan_steps: statement.get_status(StatementStatus::FullscanStep),
        sort_operations: statement.get_status(StatementStatus::Sort),
        autoindex_rows: statement.get_status(StatementStatus::AutoIndex),
        reprepares: statement.get_status(StatementStatus::RePrepare),
    };
    let measurement = Measurement {
        round,
        position,
        prepare_seconds,
        execute_decode_collect_seconds,
        emitted_rows: ids.len(),
        work,
    };
    Ok((ids, measurement))
}

fn accepted_keys(ids: Vec<i64>, roots: &BTreeMap<i64, String>) -> ProbeResult<Vec<String>> {
    let mut keys = ids
        .iter()
        .map(|id| {
            roots
                .get(id)
                .cloned()
                .ok_or_else(|| format!("unknown root ID {id}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    keys.sort_unstable();
    if keys.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("eligibility emitted duplicate root keys".into());
    }
    Ok(keys)
}

fn measure_case(
    c: &Connection,
    roots: &BTreeMap<i64, String>,
    case: Case,
    repeats: usize,
) -> ProbeResult<CaseEvidence> {
    let compiled = [
        crate::read_units::eligible_sql(&case.query, case.keys.as_deref())?,
        general_sql(&case)?,
    ];
    let mut plans = Vec::new();
    for (index, (sql, values)) in compiled.iter().enumerate() {
        let explain = c
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?
            .query_map(params_from_iter(values), |row| {
                Ok(ExplainStep {
                    id: row.get(0)?,
                    parent: row.get(1)?,
                    detail: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        // One warmup per plan. Sorting, hashing and key lookup are never timed.
        let (ids, _) = run_sql(c, sql, values, 0, index)?;
        let keys = accepted_keys(ids, roots)?;
        let digest = crate::codec::sha256(&serde_json::to_vec(&keys)?);
        plans.push(PlanEvidence {
            lowering: ["production_membership", "general_expression"][index],
            sql: sql.clone(),
            sqlite_bind_debug: values.iter().map(|value| format!("{value:?}")).collect(),
            explain,
            accepted_keys: keys,
            accepted_keys_sha256: digest,
            samples: Vec::with_capacity(repeats),
        });
    }
    let mut stable = true;
    for round in 0..repeats {
        for position in 0..2 {
            let index = (round + position) % 2;
            let (sql, values) = &compiled[index];
            let (ids, sample) = run_sql(c, sql, values, round + 1, position + 1)?;
            stable &= accepted_keys(ids, roots)? == plans[index].accepted_keys;
            plans[index].samples.push(sample);
        }
    }
    let [membership, general]: [PlanEvidence; 2] = plans.try_into().ok().unwrap();
    Ok(CaseEvidence {
        name: case.name,
        predicate: case.query.predicate().clone(),
        key_scope: case.keys,
        accepted_key_parity: membership.accepted_keys == general.accepted_keys,
        repeated_results_stable: stable,
        plans: [membership, general],
    })
}

fn roots(c: &Connection) -> rusqlite::Result<BTreeMap<i64, String>> {
    c.prepare("SELECT record_id,key FROM records")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

fn file_sha256(path: &Path) -> ProbeResult<String> {
    let mut input = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[test]
#[ignore = "requires ATLAS_QUERY_WORK_CORPUS, ATLAS_QUERY_WORK_OUTPUT and ATLAS_QUERY_WORK_CANDIDATE; read-only current-schema controlled corpus"]
fn controlled_corpus_eligibility_work() -> ProbeResult<()> {
    let corpus = PathBuf::from(
        std::env::var_os("ATLAS_QUERY_WORK_CORPUS").ok_or("set ATLAS_QUERY_WORK_CORPUS")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(
        std::env::var_os("ATLAS_QUERY_WORK_OUTPUT")
            .ok_or("set ATLAS_QUERY_WORK_OUTPUT to a fresh evidence file")?,
    );
    let candidate = std::env::var("ATLAS_QUERY_WORK_CANDIDATE")?;
    let repeats =
        std::env::var("ATLAS_QUERY_WORK_REPEATS").map_or(Ok(6), |value| value.parse::<usize>())?;
    assert!(
        corpus.is_file(),
        "corpus must be an existing workspace file"
    );
    assert!(!output.exists(), "evidence output must not already exist");
    assert!(
        !candidate.trim().is_empty(),
        "candidate identity must be explicit"
    );
    assert!((2..=20).contains(&repeats), "repeats must be 2..20");
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()?;
    let workspace = atlas_storage::Workspace::at(&corpus)?;
    let reader = crate::SqliteIndexReader::from_session(
        workspace.read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)?,
    )?;
    let c = reader.native()?;
    let roots = roots(c)?;
    let mut restricted = roots
        .values()
        .map(|key| RecordKey::parse(key))
        .collect::<Result<Vec<_>, _>>()?;
    restricted.sort_unstable();
    restricted.truncate(64);
    let evidence = cases(&restricted)?
        .into_iter()
        .map(|case| measure_case(c, &roots, case, repeats))
        .collect::<ProbeResult<Vec<_>>>()?;
    let parity = evidence
        .iter()
        .all(|case| case.accepted_key_parity && case.repeated_results_stable);
    let executable = std::env::current_exe()?;
    let report = serde_json::json!({
        "format": "atlas-eligibility-work/v1",
        "candidate_declared_by_caller": candidate,
        "test_executable_sha256": file_sha256(&executable)?,
        "debug_assertions": cfg!(debug_assertions),
        "measured_at_unix_seconds": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        "corpus": corpus,
        "root_count_including_macros": roots.len(),
        "workspace_key": reader.workspace_session().workspace_key()?,
        "reference_revision": reader.workspace_session().reference_revision()?,
        "context_sha256": reader.context_sha256(),
        "context": reader.context(),
        "sqlite_version": rusqlite::version(),
        "sqlite_source_id": c.query_row("SELECT sqlite_source_id()", [], |row| row.get::<_, String>(0))?,
        "warmup_runs_per_plan": 1,
        "measured_repeats_per_plan": repeats,
        "timing_scope": "fresh SQLite prepare including SQLite compilation; execute includes binding, row decoding and ID-vector allocation; predicate validation/SQL lowering, admission, EQP, key lookup, sorting, hashing and file output excluded",
        "counter_scope": "per-execution SQLite statement status; fullscan_steps counts forward full-table-scan steps, not total row visits; autoindex_rows counts inserted automatic-index rows; signed i32 counters retained without reinterpretation",
        "profile_scope": "root eligibility is profile independent; native retrieval and shipped CLI timings require separate measurements for each profile",
        "cache_scope": "one admitted read snapshot; warm SQLite/page-cache repetitions, alternating which lowering executes first; no OS-cache reset",
        "accepted_key_parity_and_stability": parity,
        "cases": evidence,
    });
    drop(reader);
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &report)?;
    writeln!(writer)?;
    writer.flush()?;
    assert!(
        parity,
        "eligibility parity/stability failed; inspect {}",
        output.display()
    );
    Ok(())
}

#[test]
fn fixture_probe_preserves_both_lowerings_and_measures_each_scoped_execution() {
    let db = super::tests::database();
    for (id, ac) in [(1, 50), (2, 0), (3, 50)] {
        super::tests::root(&db, id, "npc", "common");
        super::tests::row(
            &db,
            "actor_projection",
            &[
                ("record_id", id.into()),
                ("armor_class_state", super::tests::text("value")),
                ("armor_class", ac.into()),
            ],
        );
    }
    db.execute(
        "UPDATE records SET document_kind='Macro' WHERE record_id=3",
        [],
    )
    .unwrap();
    db.execute(
        "INSERT INTO record_traits VALUES(1,'magical'),(2,'humanoid'),(3,'magical')",
        [],
    )
    .unwrap();
    let roots = roots(&db).unwrap();
    let restricted = ["test:0000000000000002".parse().unwrap()];
    for case in cases(&restricted).unwrap() {
        let evidence = measure_case(&db, &roots, case, 2).unwrap();
        assert!(
            evidence.accepted_key_parity && evidence.repeated_results_stable,
            "{}",
            evidence.name
        );
        for plan in &evidence.plans {
            assert_eq!(plan.samples.len(), 2);
            assert!(plan.samples.iter().all(|sample| sample.work.vm_steps > 0));
            assert!(!plan.explain.is_empty());
            if ["frozen_ac", "ac_selective"].contains(&evidence.name) {
                assert_eq!(plan.accepted_keys, ["test:0000000000000001"]);
            }
            if evidence.name == "trait_broad_includes_common" {
                assert_eq!(
                    plan.accepted_keys,
                    ["test:0000000000000001", "test:0000000000000002"]
                );
            }
            if ["ac_restricted_keys", "ac_empty_keys"].contains(&evidence.name) {
                assert!(plan.accepted_keys.is_empty());
            }
        }
    }

    let input = crate::foundation_tests::fixture(false);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    let schema = crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
    let write = workspace.write(schema).unwrap();
    let original: String = write
        .connection()
        .query_row(
            "SELECT context_json FROM artifact_context WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let stored_json = format!("\n{original}\n");
    let stored_digest = crate::codec::sha256(stored_json.as_bytes());
    write
        .connection()
        .execute(
            "UPDATE artifact_context SET context_json=?,context_hash=? WHERE singleton=1",
            rusqlite::params![stored_json, stored_digest],
        )
        .unwrap();
    write.commit().unwrap();
    // Ordinary admission authenticates the original JSON bytes, including whitespace.
    let reader = crate::SqliteIndexReader::from_session(workspace.read(schema).unwrap()).unwrap();
    let reported_digest = reader.context_sha256();
    assert_eq!(reported_digest, stored_digest);
    assert_ne!(reported_digest, reader.context().sha256().unwrap());
}
