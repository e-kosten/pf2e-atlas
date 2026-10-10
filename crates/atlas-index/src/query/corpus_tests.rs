//! Independent authored JSON oracle, exclusively for local corpus verification.
//! Production projection and querying continue to borrow checked DTOs only.
use super::*;
use crate::{
    input::SourceArtifactRecordInput,
    persistence::ProjectionRow,
    projections::{ProjectionIds, root_rows},
};
use atlas_foundry_model::{SourceContext, admit_document_source};
use atlas_record::source_record::SourceBackedRecord;
use rusqlite::{Connection, params_from_iter, types::Value};
use serde_json::Value as Json;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn value<'a>(row: &'a ProjectionRow, column: &str) -> &'a Value {
    &row.values[row
        .columns
        .iter()
        .position(|c| c == column)
        .unwrap_or_else(|| panic!("{}.{}", row.table, column))]
}
fn scalar(row: &ProjectionRow, column: &str, raw: &Json, pointer: &str, counts: &mut [usize; 2]) {
    let state = value(row, &format!("{column}_state"));
    if state == &Value::Text("not_applicable".into()) {
        return;
    }
    let source = raw.pointer(pointer);
    if state == &Value::Text("value".into()) {
        let expected =
            source.unwrap_or_else(|| panic!("known {}.{column} absent at {pointer}", row.table));
        match (value(row, column), expected) {
            (Value::Text(a), Json::String(b)) => assert_eq!(a, b, "{} {pointer}", row.table),
            (Value::Integer(a), Json::Bool(b)) => assert_eq!(*a, i64::from(*b), "{pointer}"),
            (Value::Integer(a), Json::Number(b)) => assert_eq!(Some(*a), b.as_i64(), "{pointer}"),
            (Value::Real(a), Json::Number(b)) => assert_eq!(Some(*a), b.as_f64(), "{pointer}"),
            (a, b) => panic!("{}.{column} {pointer}: {a:?} != {b:?}", row.table),
        }
        counts[0] += 1;
    } else {
        assert_eq!(
            value(row, column),
            &Value::Null,
            "unknown value is not stored"
        );
        // Only infer terminal absence/null after every ancestor is a known
        // object. Invalid ancestors are covered by portable five-state fixtures.
        let (parent, leaf) = pointer.rsplit_once('/').unwrap();
        if raw.pointer(parent).is_some_and(Json::is_object) {
            let parent = raw.pointer(parent).unwrap();
            if parent.get(leaf).is_none() {
                assert_eq!(state, &Value::Text("missing".into()), "{pointer}");
                counts[1] += 1;
            } else if source == Some(&Json::Null) {
                assert_eq!(state, &Value::Text("null".into()), "{pointer}");
                counts[1] += 1;
            } else if source.is_some_and(|v| v.is_string())
                && matches!(
                    column,
                    "level"
                        | "rank"
                        | "bulk"
                        | "area_size"
                        | "range"
                        | "action_count"
                        | "ac_bonus"
                        | "dex_cap"
                        | "hardness"
                        | "hp_maximum"
                        | "duration_value"
                )
            {
                assert_eq!(state, &Value::Text("invalid".into()), "{pointer}");
                counts[1] += 1;
            }
        }
    }
}
fn authored_node<'a>(row: &ProjectionRow, rows: &[ProjectionRow], raw: &'a Json) -> &'a Json {
    let owner = if row.columns.iter().any(|c| c == "actor_item_id") {
        Some(value(row, "actor_item_id"))
    } else if row.table == "actor_items" {
        Some(value(row, "id"))
    } else if row.columns.iter().any(|c| c == "physical_id") {
        let physical = rows
            .iter()
            .find(|p| {
                p.table == "physical_projection" && value(p, "id") == value(row, "physical_id")
            })
            .unwrap();
        Some(value(physical, "actor_item_id"))
    } else {
        None
    };
    match owner {
        Some(Value::Integer(id)) => {
            let child = rows
                .iter()
                .find(|p| p.table == "actor_items" && value(p, "id") == &Value::Integer(*id))
                .unwrap();
            let Value::Integer(index) = value(child, "original_index") else {
                panic!("child index")
            };
            let source = &raw["items"][*index as usize];
            let selector: Json = serde_json::from_str(match value(child, "owner_selector_json") {
                Value::Text(s) => s,
                _ => panic!("owner"),
            })
            .unwrap();
            assert_eq!(selector["collection"], "/items");
            assert_eq!(
                value(child, "authored_id"),
                &Value::Text(source["_id"].as_str().unwrap().to_owned())
            );
            source
        }
        _ => raw,
    }
}
fn oracle(rows: &[ProjectionRow], raw: &Json, counts: &mut [usize; 2]) {
    for row in rows {
        let raw = authored_node(row, rows, raw);
        let fields: &[(&str, &str)] = match row.table {
            "records" | "actor_items" => &[
                ("source_type", "/type"),
                ("rarity", "/system/traits/rarity"),
                ("publication_title", "/system/publication/title"),
                ("publication_remaster", "/system/publication/remaster"),
            ],
            "spell_projection" => &[
                ("rank", "/system/level/value"),
                ("casting_time", "/system/time/value"),
                ("save", "/system/defense/save/statistic"),
                ("basic_save", "/system/defense/save/basic"),
                ("passive_defense", "/system/defense/passive/statistic"),
                ("area_type", "/system/area/type"),
                ("area_size", "/system/area/value"),
                ("duration_text", "/system/duration/value"),
                ("sustained", "/system/duration/sustained"),
            ],
            "physical_projection" => &[
                ("bulk", "/system/bulk/value"),
                ("usage", "/system/usage/value"),
                ("consumable_category", "/system/category"),
            ],
            "weapon_projection" => &[
                ("category", "/system/category"),
                ("weapon_group", "/system/group"),
                ("damage_type", "/system/damage/damageType"),
                ("range", "/system/range"),
                ("reload", "/system/reload/value"),
            ],
            "armor_projection" => &[
                ("category", "/system/category"),
                ("ac_bonus", "/system/acBonus"),
                ("dex_cap", "/system/dexCap"),
            ],
            "shield_projection" => &[
                ("hardness", "/system/hardness"),
                ("hp_maximum", "/system/hp/max"),
            ],
            "ability_projection" => &[
                ("action_type", "/system/actionType/value"),
                ("action_count", "/system/actions/value"),
                ("category", "/system/category"),
            ],
            "heritage_projection" => &[
                ("ancestry_uuid", "/system/ancestry/uuid"),
                ("ancestry_slug", "/system/ancestry/slug"),
            ],
            "effect_projection" => &[
                ("duration_unit", "/system/duration/unit"),
                ("duration_value", "/system/duration/value"),
            ],
            "condition_projection" => &[("is_valued", "/system/value/isValued")],
            _ => &[],
        };
        for (column, path) in fields {
            let pointer = match (*column, raw["type"].as_str()) {
                ("publication_title", Some("npc" | "hazard" | "vehicle")) => {
                    "/system/details/publication/title"
                }
                ("publication_remaster", Some("npc" | "hazard" | "vehicle")) => {
                    "/system/details/publication/remaster"
                }
                _ => path,
            };
            scalar(row, column, raw, pointer, counts);
        }
        if matches!(row.table, "records" | "actor_items") {
            let actor = matches!(
                raw["type"].as_str(),
                Some(
                    "npc"
                        | "hazard"
                        | "character"
                        | "army"
                        | "vehicle"
                        | "loot"
                        | "familiar"
                        | "party"
                )
            );
            scalar(
                row,
                "level",
                raw,
                if actor {
                    "/system/details/level/value"
                } else {
                    "/system/level/value"
                },
                counts,
            );
            scalar(
                row,
                "size",
                raw,
                if actor {
                    "/system/traits/size/value"
                } else {
                    "/system/size"
                },
                counts,
            );
            if value(row, "traits_state") == &Value::Text("value".into()) {
                let (table, key, id) = if row.table == "records" {
                    ("record_traits", "record_id", value(row, "record_id"))
                } else {
                    ("actor_item_traits", "item_id", value(row, "id"))
                };
                let actual = rows
                    .iter()
                    .filter(|r| r.table == table && value(r, key) == id)
                    .map(|r| match value(r, "value") {
                        Value::Text(s) => s.clone(),
                        _ => panic!("trait"),
                    })
                    .collect::<std::collections::BTreeSet<_>>();
                let expected = raw["system"]["traits"]["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| s.as_str().unwrap().to_owned())
                    .collect::<std::collections::BTreeSet<_>>();
                assert_eq!(actual, expected);
                counts[0] += 1;
            }
        }
    }
}
fn files(path: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, out);
        } else if path.extension().is_some_and(|x| x == "json") {
            out.push(path);
        }
    }
}

#[test]
#[ignore = "requires pinned PF2e export; independent full Item/common extraction oracle"]
fn pinned_corpus_item_and_common_projection_oracle() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|p| {
            let source = p.join("scratch/source-contracts-full/pf2e");
            source
                .join("static/system.json")
                .is_file()
                .then_some(source)
        })
        .unwrap();
    let manifest: Json =
        serde_json::from_slice(&fs::read(source.join("static/system.json")).unwrap()).unwrap();
    assert_eq!(manifest["version"], "6.12.4");
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!(
        "../../migrations/00000000000001_create_artifact/up.sql"
    ))
    .unwrap();
    let mut families = BTreeMap::new();
    let mut counts = [0; 2];
    let mut roots = 0;
    let mut item_roots = 0;
    let mut children = 0;
    let mut ids = ProjectionIds::default();
    for pack in manifest["packs"].as_array().unwrap() {
        let name = pack["name"].as_str().unwrap();
        let kind = pack["type"].as_str().unwrap();
        let label = pack["label"].as_str().unwrap();
        let dir = source.join(pack["path"].as_str().unwrap());
        if !dir.is_dir() {
            continue;
        }
        db.execute("INSERT INTO packs VALUES(?1,?2)", [name, label])
            .unwrap();
        let mut paths = Vec::new();
        files(&dir, &mut paths);
        for path in paths {
            if path.file_name().is_some_and(|name| name == "_folders.json") {
                continue;
            }
            let bytes = fs::read(&path).unwrap();
            let raw: Json = serde_json::from_slice(&bytes).unwrap();
            let admission = admit_document_source(
                kind,
                SourceContext::new(path.display().to_string(), name, "$"),
                &bytes,
            )
            .unwrap();
            let Some(model) = admission.model else {
                continue;
            };
            let Ok(record) = SourceBackedRecord::new(name, model) else {
                continue;
            };
            roots += 1;
            item_roots += usize::from(kind == "Item");
            let input = SourceArtifactRecordInput {
                record,
                source_path: path.display().to_string(),
                content_hash: "fixture".to_owned(),
                preparation_context_hash: "fixture".to_owned(),
                content: vec![],
                relationships: vec![],
                diagnostics: vec![],
            };
            let rows = root_rows(&input, label, roots as i64, &mut ids)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            children += rows.iter().filter(|r| r.table == "actor_items").count();
            oracle(&rows, &raw, &mut counts);
            // Independently execute every descriptor against one real checked
            // root per family, including the source-authoritative child rows.
            let family = format!("{kind}:{}", raw["type"].as_str().unwrap_or(""));
            if let std::collections::btree_map::Entry::Vacant(entry) = families.entry(family) {
                for row in &rows {
                    let slots = (1..=row.values.len())
                        .map(|n| format!("?{n}"))
                        .collect::<Vec<_>>()
                        .join(",");
                    db.execute(
                        &format!(
                            "INSERT INTO {} ({}) VALUES ({slots})",
                            row.table,
                            row.columns.join(",")
                        ),
                        params_from_iter(&row.values),
                    )
                    .unwrap();
                }
                for d in query_capabilities().unwrap().fields {
                    if d.scope.is_none() {
                        let b = compiler::bound_field(
                            catalog::descriptor(None, &d.path).unwrap().unwrap(),
                            None,
                        );
                        let sql = format!("SELECT {} FROM records r WHERE r.record_id=?1", b.state);
                        let state: String =
                            db.query_row(&sql, [roots as i64], |r| r.get(0)).unwrap();
                        assert!(
                            ["value", "missing", "null", "invalid", "not_applicable"]
                                .contains(&state.as_str()),
                            "{} {}: {state}",
                            path.display(),
                            d.id
                        );
                    }
                }
                entry.insert(path);
            }
        }
    }
    assert_eq!(roots, 25_641, "pinned corpus roots changed");
    assert_eq!(item_roots, 18_634, "pinned corpus Item roots changed");
    assert_eq!(children, 79_818, "pinned corpus Actor Items changed");
    assert!(counts[0] > 100_000);
    assert!(counts[1] > 1_000);
    eprintln!(
        "corpus roots={roots} item_roots={item_roots} immediate_actor_items={children} known_fact_checks={} unavailable_terminal_checks={} family_sql_oracles={}",
        counts[0],
        counts[1],
        families.len()
    );
}
