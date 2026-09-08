use std::fs;
use std::process::Command;

use serde_json::Value;

mod support;

use support::json::{ok_data, record_sections};
use support::path::temp_source_root;
use support::source::{
    write_ambiguous_action_source, write_creature_preview_source, write_hazard_source,
    write_record_search_source, write_tooling_collision_source,
};

#[test]
fn hazard_record_uses_typed_json_text_search_filters_and_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-hazard-record-contract");
    write_hazard_source(&root)?;
    let index_path = root.join("artifact.sqlite");
    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .args(["--no-embeddings", "--json"])
        .output()?;
    assert!(
        build_output.status.success(),
        "{}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "hazards:BHq5wpQU8hQEke8D",
            "--detail",
            "full",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(get_output.status.success());
    let get_json: Value = serde_json::from_slice(&get_output.stdout)?;
    let record = &ok_data(&get_json)["record"];
    assert_eq!(record["presentation_type"], "hazard");
    assert_eq!(record["level"], 0);
    assert_eq!(record["rarity"], "common");
    assert_eq!(record["source"]["publication_title"], "Pathfinder GM Core");
    assert_eq!(record["source"]["publication_remaster"], true);
    assert!(record.get("provenance").is_none());
    assert!(record.get("migration").is_none());
    let rendered = serde_json::to_string(record)?;
    for expected in [
        "Detection DC",
        "Broken Threshold",
        "Fortitude",
        "Pitfall",
        "Trigger",
        "Effect",
        "Reset",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected}: {rendered}"
        );
    }
    assert!(!rendered.contains("publication_license"));
    assert!(!rendered.contains("actor_effects"));
    assert!(!rendered.contains("img"));
    let availability = record["availability"]
        .as_array()
        .expect("hazard availability");
    assert!(
        availability
            .iter()
            .any(|row| { row["field"] == "action.actions" && row["state"] == "null" })
    );
    assert!(availability.iter().all(|row| {
        !row["field"].as_str().is_some_and(|field| {
            field.contains("source_metadata")
                || field == "provenance.token.name"
                || field == "entity.unsupported.action.unexpected.traits.rarity"
        })
    }));

    let text_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "hazards:BHq5wpQU8hQEke8D",
            "--detail",
            "full",
            "--index",
        ])
        .arg(&index_path)
        .output()?;
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout)?;
    for expected in [
        "Detection DC: 18",
        "AC: 10",
        "HP: 12",
        "Hardness: 3",
        "BT: 6",
        "Fort: +1",
        "Ref: +1",
        "Will: +0",
        "Pitfall: Reaction",
        "Trigger",
        "Effect",
    ] {
        assert!(text.contains(expected), "missing {expected:?}:\n{text}");
    }
    for hidden in ["Damage 0", "Occurrence ", "Entity ", "Temporary HP: 0"] {
        assert!(!text.contains(hidden), "leaked {hidden:?}:\n{text}");
    }

    let search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "mechanical",
            "--retrieval",
            "fts",
            "--kind",
            "hazard",
            "--metric",
            "ac.value=10",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(
        search_output.status.success(),
        "{}",
        String::from_utf8_lossy(&search_output.stdout)
    );
    let search_json: Value = serde_json::from_slice(&search_output.stdout)?;
    let search = ok_data(&search_json);
    assert_eq!(search["pagination"]["total"], 1);
    assert_eq!(
        search["results"][0]["record"]["presentation_type"],
        "hazard"
    );

    let provenance_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "provenance",
            "hazards:BHq5wpQU8hQEke8D",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(provenance_output.status.success());
    let provenance_json: Value = serde_json::from_slice(&provenance_output.stdout)?;
    let provenance = ok_data(&provenance_json);
    assert_eq!(
        provenance["hazard_provenance"]["convenience_rule_id"],
        "pf2e-hazard-conveniences"
    );
    assert_eq!(
        provenance["hazard_provenance"]["publication_license"],
        "ORC"
    );
    assert_eq!(
        provenance["hazard_provenance"]["occurrences"][0]["family"],
        "action"
    );
    assert!(
        provenance["hazard_provenance"]["content"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
    );
    let source_metadata = provenance["hazard_provenance"]["source_metadata"]
        .as_array()
        .expect("hazard source metadata");
    assert_eq!(
        source_metadata
            .iter()
            .map(|fact| fact["field"].as_str().expect("source metadata field"))
            .collect::<Vec<_>>(),
        vec![
            "token_name",
            "has_health",
            "temporary_maximum",
            "save_detail",
            "save_detail",
            "save_detail",
            "item_rarity",
            "item_lineage",
        ]
    );
    let has_health = &source_metadata[1];
    assert_eq!(has_health["value"]["value"]["state"], "value");
    assert_eq!(has_health["value"]["value"]["value"]["support"], "typed");
    assert_eq!(has_health["value"]["value"]["value"]["value"], true);
    assert_eq!(
        has_health["value"]["provenance"]["relativeSourcePath"],
        "/system/attributes/hasHealth"
    );
    assert_eq!(source_metadata[2]["value"]["value"]["value"]["value"], 0);
    assert_eq!(source_metadata[3]["save"], "fortitude");
    assert_eq!(source_metadata[3]["value"]["value"]["value"]["value"], "");
    assert_eq!(source_metadata[6]["entity_id"], "lY83oUjx0DLxDByK");
    assert_eq!(
        source_metadata[6]["value"]["value"]["value"]["value"],
        "common"
    );
    assert_eq!(provenance["references"]["lookup_performed"], true);
    assert!(
        provenance["references"]["edges"]
            .as_array()
            .is_some_and(|edges| edges.iter().any(|edge| {
                edge["direction"] == "outgoing"
                    && edge["to_record_key"] == "actionspf2e:grabEdgeTest0001"
            }))
    );
    assert!(!serde_json::to_string(provenance)?.contains("image"));

    fs::remove_dir_all(root)?;
    Ok(())
}

fn assert_no_internal_creature_locators(record: &Value) {
    fn reject_named_debug_fields(value: &Value) {
        match value {
            Value::Object(object) => {
                for forbidden in [
                    "target_entity_id",
                    "parent_entry_id",
                    "slot",
                    "owner",
                    "provenance",
                ] {
                    assert!(
                        !object.contains_key(forbidden),
                        "ordinary creature JSON exposed {forbidden}"
                    );
                }
                object.values().for_each(reject_named_debug_fields);
            }
            Value::Array(values) => values.iter().for_each(reject_named_debug_fields),
            _ => {}
        }
    }

    reject_named_debug_fields(record);
    for family in ["strikes", "actions", "equipment", "lore"] {
        for row in record[family].as_array().into_iter().flatten() {
            assert!(row.get("id").is_none(), "ordinary {family} row exposed id");
        }
    }
    if let Some(spellcasting) = record.get("spellcasting") {
        for entry in spellcasting["entries"].as_array().into_iter().flatten() {
            assert!(
                entry.get("id").is_none(),
                "ordinary spellcasting entry exposed id"
            );
            for spell in entry["spells"].as_array().into_iter().flatten() {
                assert!(spell.get("id").is_none(), "ordinary spell row exposed id");
            }
            for slot in entry["slots"].as_array().into_iter().flatten() {
                for prepared in slot["prepared"].as_array().into_iter().flatten() {
                    assert!(
                        prepared.get("id").is_none(),
                        "ordinary prepared spell exposed item id"
                    );
                }
            }
        }
        for spell in spellcasting["standalone_spells"]
            .as_array()
            .into_iter()
            .flatten()
        {
            assert!(
                spell.get("id").is_none(),
                "ordinary standalone spell exposed id"
            );
        }
    }
}

#[test]
fn record_get_resolve_and_filter_search_use_shared_record_shape()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-search");
    write_record_search_source(&root)?;
    let index_path = root.join("artifact.sqlite");

    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;
    assert!(build_output.status.success());

    let get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "actions:testAction0001", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(get_output.status.success());
    let get_json: Value = serde_json::from_slice(&get_output.stdout)?;
    let get_data = ok_data(&get_json);
    assert_eq!(get_data["detail"], "standard");
    assert_eq!(get_data["record"]["key"], "actions:testAction0001");
    assert_eq!(get_data["record"]["name"], "Treat Wounds");
    assert_eq!(get_data["record"]["kind"], "rule");
    assert_eq!(get_data["record"]["presentation_type"], "unmigrated");
    assert_eq!(get_data["record"]["migration"]["plan_id"], "H7");
    assert!(record_sections(&get_data["record"]).contains(&"description"));
    assert!(!record_sections(&get_data["record"]).contains(&"description_preview"));
    assert!(get_data["record"].get("source_json").is_none());

    let preview_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "--detail",
            "preview",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(
        preview_get_output.status.success(),
        "preview command failed: status={}; stdout={}; stderr={}",
        preview_get_output.status,
        String::from_utf8_lossy(&preview_get_output.stdout),
        String::from_utf8_lossy(&preview_get_output.stderr)
    );
    let preview_get_json: Value = serde_json::from_slice(&preview_get_output.stdout)?;
    let preview_get_data = ok_data(&preview_get_json);
    assert_eq!(preview_get_data["detail"], "preview");
    assert!(record_sections(&preview_get_data["record"]).contains(&"description_preview"));
    assert!(!record_sections(&preview_get_data["record"]).contains(&"description"));

    let description_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "--detail",
            "description",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(description_get_output.status.success());
    let description_get_json: Value = serde_json::from_slice(&description_get_output.stdout)?;
    let description_get_data = ok_data(&description_get_json);
    assert_eq!(description_get_data["detail"], "description");
    assert!(record_sections(&description_get_data["record"]).contains(&"description"));
    assert!(!record_sections(&description_get_data["record"]).contains(&"description_preview"));
    assert!(!record_sections(&description_get_data["record"]).contains(&"details"));
    let description_sections =
        serde_json::to_string(&description_get_data["record"]["supplementary_sections"])?;
    assert!(description_sections.contains("\"label\":\"Treat Wounds\""));
    assert!(description_sections.contains("\"record_key\":\"actions:testAction0001\""));

    let full_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "--detail",
            "full",
            "--include-raw",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(full_get_output.status.success());
    let full_get_json: Value = serde_json::from_slice(&full_get_output.stdout)?;
    let full_get_data = ok_data(&full_get_json);
    assert_eq!(full_get_data["detail"], "full");
    assert!(full_get_data["record"]["source_json"].as_str().is_some());
    assert_eq!(
        full_get_data["record"]["source"]["foundry"]["document_type"],
        "Item"
    );
    assert!(record_sections(&full_get_data["record"]).contains(&"description"));
    assert!(!record_sections(&full_get_data["record"]).contains(&"description_preview"));

    let preview_raw_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "--detail",
            "preview",
            "--include-raw",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(preview_raw_get_output.status.success());
    let preview_raw_get_json: Value = serde_json::from_slice(&preview_raw_get_output.stdout)?;
    assert!(
        ok_data(&preview_raw_get_json)["record"]["source_json"]
            .as_str()
            .is_some()
    );

    let batch_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "actions:missingAction999",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(batch_get_output.status.code(), Some(1));
    let batch_get_json: Value = serde_json::from_slice(&batch_get_output.stdout)?;
    let batch_get_data = ok_data(&batch_get_json);
    assert_eq!(batch_get_data["counts"]["requested"], 2);
    assert_eq!(batch_get_data["counts"]["matched"], 1);
    assert_eq!(batch_get_data["counts"]["failed"], 1);
    assert_eq!(batch_get_data["partial"], true);
    assert_eq!(
        batch_get_data["results"][1]["error"]["code"],
        "record_not_found"
    );

    let successful_batch_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "actions:testAction0001",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(
        successful_batch_get_output.status.success(),
        "batch get failed: stdout={} stderr={}",
        String::from_utf8_lossy(&successful_batch_get_output.stdout),
        String::from_utf8_lossy(&successful_batch_get_output.stderr),
    );
    let successful_batch_get_json: Value =
        serde_json::from_slice(&successful_batch_get_output.stdout)?;
    let successful_batch_get_data = ok_data(&successful_batch_get_json);
    assert_eq!(successful_batch_get_data["counts"]["failed"], 0);
    assert_eq!(successful_batch_get_data["partial"], false);

    let resolve_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "resolve", "Treat Wounds", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(resolve_output.status.success());
    let resolve_json: Value = serde_json::from_slice(&resolve_output.stdout)?;
    let resolve_data = ok_data(&resolve_json);
    assert_eq!(
        resolve_data["result"]["record"]["key"],
        "actions:testAction0001"
    );
    assert_eq!(resolve_data["detail"], "standard");
    assert!(record_sections(&resolve_data["result"]["record"]).contains(&"description"));
    assert!(!record_sections(&resolve_data["result"]["record"]).contains(&"description_preview"));
    assert_eq!(resolve_data["result"]["resolution"]["match_kind"], "name");

    let batch_resolve_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Treat Wounds",
            "No Such Record",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(batch_resolve_output.status.code(), Some(1));
    let batch_resolve_json: Value = serde_json::from_slice(&batch_resolve_output.stdout)?;
    let batch_resolve_data = ok_data(&batch_resolve_json);
    assert_eq!(batch_resolve_data["counts"]["requested"], 2);
    assert_eq!(batch_resolve_data["counts"]["matched"], 1);
    assert_eq!(batch_resolve_data["counts"]["failed"], 1);
    assert_eq!(batch_resolve_data["partial"], true);
    assert_eq!(
        batch_resolve_data["results"][1]["error"]["code"],
        "record_resolution_miss"
    );

    let successful_batch_resolve_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Treat Wounds",
            "Treat Wounds",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(successful_batch_resolve_output.status.success());
    let successful_batch_resolve_json: Value =
        serde_json::from_slice(&successful_batch_resolve_output.stdout)?;
    let successful_batch_resolve_data = ok_data(&successful_batch_resolve_json);
    assert_eq!(successful_batch_resolve_data["counts"]["failed"], 0);
    assert_eq!(successful_batch_resolve_data["partial"], false);

    let text_resolve_description_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Treat Wounds",
            "--detail",
            "description",
            "--index",
        ])
        .arg(&index_path)
        .output()?;
    assert!(text_resolve_description_output.status.success());
    let text_resolve_description_stdout =
        String::from_utf8(text_resolve_description_output.stdout)?;
    assert!(text_resolve_description_stdout.starts_with("Treat Wounds\n"));
    assert!(text_resolve_description_stdout.contains("Type: rule"));
    assert!(text_resolve_description_stdout.contains("Key: actions:testAction0001"));
    assert!(text_resolve_description_stdout.contains("Pack: Actions"));
    assert!(
        text_resolve_description_stdout
            .contains("You spend 10 minutes treating one injured living creature with")
    );
    assert!(text_resolve_description_stdout.contains("Public Notes"));
    assert!(text_resolve_description_stdout.contains("Bring a healer's kit."));
    assert!(text_resolve_description_stdout.contains("Match: name"));

    let search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(search_output.status.success());
    let search_json: Value = serde_json::from_slice(&search_output.stdout)?;
    let search_data = ok_data(&search_json);
    assert_eq!(search_data["sort"]["kind"], "alphabetical");
    assert_eq!(search_data["pagination"]["total"], 1);
    assert_eq!(
        search_data["results"][0]["record"]["key"],
        "actions:testAction0001"
    );
    assert_eq!(search_data["results"][0]["match"]["kind"], "filter");

    let fts_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(fts_search_output.status.success());
    assert_eq!(String::from_utf8(fts_search_output.stderr)?, "");
    let fts_search_json: Value = serde_json::from_slice(&fts_search_output.stdout)?;
    let fts_search_data = ok_data(&fts_search_json);
    assert_eq!(fts_search_data["retrieval"], "fts");
    assert_eq!(
        fts_search_data["query_analysis"]["fts_tokens"][0],
        "healing"
    );
    assert_eq!(fts_search_data["fusion"]["method"], "weighted-rrf");
    assert_eq!(fts_search_data["fusion"]["fts_policy"], "demote-weak");
    assert_eq!(fts_search_data["candidate_windows"]["fts_top_k"], 200);
    assert_eq!(fts_search_data["candidate_windows"]["vector_top_k"], 200);
    assert_eq!(fts_search_data["sort"]["kind"], "ranked");
    assert_eq!(fts_search_data["pagination"]["total"], 1);
    assert_eq!(
        fts_search_data["results"][0]["record"]["key"],
        "actions:testAction0001"
    );
    assert_eq!(fts_search_data["results"][0]["match"]["kind"], "ranked");
    assert_eq!(fts_search_data["results"][0]["match"]["retrieval"], "fts");
    assert_eq!(
        fts_search_data["results"][0]["match"]["explain"]["fts"]["fts_rank"],
        1
    );
    assert_eq!(
        fts_search_data["results"][0]["match"]["explain"]["fts"]["fts_lane"],
        "facet"
    );
    assert_eq!(
        fts_search_data["results"][0]["match"]["explain"]["fts"]["fts_confidence"],
        "strong-lexical"
    );

    let body_only_fts_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "treating",
            "--retrieval",
            "fts",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(body_only_fts_output.status.success());
    let body_only_fts_json: Value = serde_json::from_slice(&body_only_fts_output.stdout)?;
    let body_only_fts_data = ok_data(&body_only_fts_json);
    assert_eq!(body_only_fts_data["pagination"]["total"], 0);

    let excluded_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--exclude",
            "treating",
            "--retrieval",
            "fts",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(excluded_search_output.status.success());
    let excluded_search_json: Value = serde_json::from_slice(&excluded_search_output.stdout)?;
    let excluded_search_data = ok_data(&excluded_search_json);
    assert_eq!(excluded_search_data["pagination"]["total"], 1);
    assert_eq!(
        excluded_search_data["query_analysis"]["exclude_tokens"][0],
        "treating"
    );

    let page_window_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--limit",
            "1",
            "--page",
            "3",
            "--fts-top-k",
            "1",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(page_window_output.status.success());
    let page_window_json: Value = serde_json::from_slice(&page_window_output.stdout)?;
    let page_window_data = ok_data(&page_window_json);
    assert_eq!(page_window_data["candidate_windows"]["fts_top_k"], 3);

    let oversized_window_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--fts-top-k",
            "5001",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(oversized_window_output.status.code(), Some(2));
    let oversized_window_json: Value = serde_json::from_slice(&oversized_window_output.stdout)?;
    assert_eq!(oversized_window_json["status"], "error");
    assert_eq!(oversized_window_json["error"]["code"], "invalid_option");
    assert!(
        oversized_window_json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("ranked search candidate windows must be at most 5000")
    );

    for args in [
        vec![
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--vector-top-k",
            "5001",
        ],
        vec![
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--page",
            "5001",
            "--limit",
            "1",
        ],
    ] {
        let oversized_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(args)
            .arg("--index")
            .arg(&index_path)
            .arg("--json")
            .output()?;
        assert_eq!(oversized_output.status.code(), Some(2));
        let oversized_json: Value = serde_json::from_slice(&oversized_output.stdout)?;
        assert_eq!(oversized_json["status"], "error");
        assert_eq!(oversized_json["error"]["code"], "invalid_option");
        assert!(
            oversized_json["error"]["message"]
                .as_str()
                .unwrap()
                .contains("ranked search candidate windows must be at most 5000")
        );
    }

    let default_hybrid_without_embeddings = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "healing", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(default_hybrid_without_embeddings.status.code(), Some(3));
    let hybrid_error: Value = serde_json::from_slice(&default_hybrid_without_embeddings.stdout)?;
    assert_eq!(hybrid_error["status"], "error");
    assert_eq!(hybrid_error["error"]["code"], "vector_readiness_required");
    assert!(
        hybrid_error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("rerun the search with --retrieval fts")
    );

    let text_hybrid_without_embeddings = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "healing", "--index"])
        .arg(&index_path)
        .output()?;
    assert_eq!(text_hybrid_without_embeddings.status.code(), Some(2));
    assert!(
        String::from_utf8(text_hybrid_without_embeddings.stderr)?
            .contains("rerun the search with --retrieval fts")
    );

    let text_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "actions:testAction0001", "--index"])
        .arg(&index_path)
        .output()?;
    assert!(text_get_output.status.success());
    let text_get_stdout = String::from_utf8(text_get_output.stdout)?;
    assert!(text_get_stdout.starts_with("Treat Wounds\n"));
    assert!(text_get_stdout.contains("Type: rule"));
    assert!(text_get_stdout.contains("Key: actions:testAction0001"));
    assert!(!text_get_stdout.contains("\"status\""));

    let text_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--index"])
        .arg(&index_path)
        .output()?;
    assert!(text_search_output.status.success());
    assert_eq!(String::from_utf8(text_search_output.stderr)?, "");
    let text_search_stdout = String::from_utf8(text_search_output.stdout)?;
    assert!(text_search_stdout.contains("showing 1 of 1 records"));
    assert!(text_search_stdout.contains("key"));
    assert!(text_search_stdout.contains("kind"));
    assert!(text_search_stdout.contains("name"));
    assert!(text_search_stdout.contains("actions:testAction0001  rule  Treat Wounds"));
    assert!(!text_search_stdout.contains("\"status\""));

    let text_search_preview_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--detail", "preview", "--index"])
        .arg(&index_path)
        .output()?;
    assert!(text_search_preview_output.status.success());
    let text_search_preview_stdout = String::from_utf8(text_search_preview_output.stdout)?;
    assert!(text_search_preview_stdout.contains("Treat Wounds\n"));
    assert!(text_search_preview_stdout.contains("Type: rule"));
    assert!(text_search_preview_stdout.contains("Key: actions:testAction0001"));
    assert!(text_search_preview_stdout.contains("Pack: Actions"));
    assert!(
        text_search_preview_stdout
            .contains("You spend 10 minutes treating one injured living creature with")
    );
    assert!(text_search_preview_stdout.contains("Match: filter"));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn creature_record_uses_direct_tagged_fields_at_each_detail()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-creature-record-contract");
    write_creature_preview_source(&root)?;
    let index_path = root.join("artifact.sqlite");
    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .arg("--output")
        .arg(&index_path)
        .args(["--no-embeddings", "--json"])
        .output()?;
    assert!(build_output.status.success());

    for detail in ["summary", "preview", "description", "standard", "full"] {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args([
                "record",
                "get",
                "creatures:testCreature001",
                "--detail",
                detail,
                "--index",
            ])
            .arg(&index_path)
            .arg("--json")
            .output()?;
        assert!(output.status.success(), "detail {detail}");
        let json: Value = serde_json::from_slice(&output.stdout)?;
        let record = &ok_data(&json)["record"];
        assert_eq!(record["presentation_type"], "creature");
        assert_no_internal_creature_locators(record);
        assert!(record.get("sections").is_none());
        let includes_scan_fields = matches!(detail, "preview" | "standard" | "full");
        for field in ["defenses", "perception", "languages", "movement"] {
            assert_eq!(
                record.get(field).is_some(),
                includes_scan_fields,
                "detail {detail} field {field}"
            );
        }
        for field in ["skills", "resources", "strikes", "actions", "spellcasting"] {
            assert!(
                record.get(field).is_none(),
                "detail {detail} must omit source-missing field {field}"
            );
        }
        if includes_scan_fields {
            assert!(record["defenses"].is_object());
            assert!(record["movement"].is_object());
        }
    }

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "creatures:testCreature001",
            "--detail",
            "standard",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let record = &ok_data(&json)["record"];
    assert_eq!(record["defenses"]["ac"]["value"], 25);
    assert_eq!(record["defenses"]["hp"]["maximum"], 80);
    assert_eq!(record["defenses"]["saves"]["fortitude"]["value"], 14);
    assert_eq!(record["perception"]["modifier"], 12);
    assert_eq!(record["languages"][0], "common");
    assert_eq!(record["movement"]["modes"][0]["value_feet"], 25);
    let forbidden_generic_body = ["creature", "mechanics"].join("_");
    assert!(!serde_json::to_string(record)?.contains(&forbidden_generic_body));

    for detail in ["summary", "preview", "description", "standard", "full"] {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args([
                "record",
                "get",
                "creatures:WQy7HBUcgDLsfVJd",
                "--detail",
                detail,
                "--index",
            ])
            .arg(&index_path)
            .arg("--json")
            .output()?;
        assert!(output.status.success(), "Night Hag detail {detail}");
        let json: Value = serde_json::from_slice(&output.stdout)?;
        let record = &ok_data(&json)["record"];
        assert_eq!(record["presentation_type"], "creature");
        assert_no_internal_creature_locators(record);
        let includes_scan_fields = matches!(detail, "preview" | "standard" | "full");
        assert_eq!(record.get("defenses").is_some(), includes_scan_fields);
        assert_eq!(record.get("strikes").is_some(), includes_scan_fields);
        assert_eq!(record.get("actions").is_some(), includes_scan_fields);
        assert_eq!(record.get("spellcasting").is_some(), includes_scan_fields);
        for forbidden in ["relationships", "provenance", "availability_evidence"] {
            assert!(
                record.get(forbidden).is_none(),
                "ordinary {detail} exposed provenance-only {forbidden}"
            );
        }
        if detail == "preview" {
            assert!(record["strikes"][0].get("rolls").is_none());
            assert!(record["strikes"][0].get("damage").is_none());
            assert!(record["strikes"][0].get("modes").is_none());
            assert!(record["actions"][0].get("rolls").is_none());
            assert!(record["spellcasting"]["spells"][0].get("damage").is_none());
        }
        if matches!(detail, "standard" | "full") {
            assert_eq!(record["defenses"]["ac"]["value"], 28);
            assert_eq!(record["defenses"]["hp"]["maximum"], 170);
            assert_eq!(record["defenses"]["saves"]["fortitude"]["value"], 19);
            assert_eq!(record["defenses"]["immunities"][0]["iwr_type"], "sleep");
            assert_eq!(record["defenses"]["resistances"][0]["iwr_type"], "mental");
            assert_eq!(record["defenses"]["weaknesses"][0]["iwr_type"], "cold-iron");
            assert_eq!(record["perception"]["modifier"], 18);
            assert_eq!(record["languages"][0], "aklo");
            assert_eq!(record["skills"][0]["slug"], "occultism");
            assert_eq!(record["skills"][0]["note"], "ancient soul lore");
            assert_eq!(record["skills"][1]["slug"], "lore");
            assert_eq!(record["movement"]["modes"][0]["mode"], "land");
            assert_eq!(record["movement"]["modes"][1]["mode"], "fly");
            assert_eq!(record["resources"][0]["maximum"], 1);
            assert_eq!(record["resources"][0]["serialized_value"], 1);
            assert_eq!(record["defenses"]["resistances"][0]["value"], 10);
            assert_eq!(record["defenses"]["weaknesses"][0]["value"], 10);
            let change_shape = record["actions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|action| action["label"] == "Change Shape")
                .expect("Change Shape action");
            assert_eq!(change_shape["action_cost"]["kind"], "actions");
            assert_eq!(change_shape["action_cost"]["actions"], 1);
            assert_eq!(record["strikes"].as_array().unwrap().len(), 2);
            assert_eq!(record["actions"].as_array().unwrap().len(), 2);
            assert_eq!(
                record["spellcasting"]["entries"].as_array().unwrap().len(),
                2
            );
            let spells = record["spellcasting"]["entries"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|entry| entry["spells"].as_array().into_iter().flatten())
                .collect::<Vec<_>>();
            assert_eq!(spells.len(), 2);
            assert!(record["strikes"][0]["rolls"].is_array());
            assert!(record["strikes"][0]["damage"].is_array());
            assert!(record["actions"][0]["rolls"].is_array());
            assert!(spells[0]["damage"].is_array());
            assert_eq!(spells[0]["context"]["rank"], 3);
            assert!(spells[0].get("parent_entry_id").is_none());
            if detail == "full" {
                assert!(serde_json::to_string(record)?.contains("Heartstones"));
            }
        }
        if detail == "full" {
            assert!(record["source"].get("source_path").is_none());
            assert!(record["source"].get("foundry").is_none());
            assert!(record["strikes"][0].get("provenance").is_none());
        }
    }

    let resolve_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Night Hag",
            "--kind",
            "creature",
            "--detail",
            "standard",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(resolve_output.status.success());
    let resolved_json: Value = serde_json::from_slice(&resolve_output.stdout)?;
    let resolved = &ok_data(&resolved_json)["result"]["record"];
    assert_eq!(resolved["presentation_type"], "creature");
    assert_eq!(resolved["strikes"].as_array().unwrap().len(), 2);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn record_commands_return_standard_error_codes() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-errors");
    write_record_search_source(&root)?;
    let index_path = root.join("artifact.sqlite");
    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;
    assert!(build_output.status.success());

    let invalid_key = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "not-a-key", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(invalid_key.status.code(), Some(2));
    let invalid_key_json: Value = serde_json::from_slice(&invalid_key.stdout)?;
    assert_eq!(invalid_key_json["status"], "error");
    assert_eq!(invalid_key_json["error"]["code"], "invalid_record_key");

    let miss = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "resolve", "No Such Record", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(miss.status.code(), Some(1));
    let miss_json: Value = serde_json::from_slice(&miss.stdout)?;
    assert_eq!(miss_json["status"], "error");
    assert_eq!(miss_json["error"]["code"], "record_resolution_miss");

    let invalid_filter = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Treat Wounds",
            "--filter-json",
            "{",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(invalid_filter.status.code(), Some(2));
    let invalid_filter_json: Value = serde_json::from_slice(&invalid_filter.stdout)?;
    assert_eq!(invalid_filter_json["status"], "error");
    assert_eq!(invalid_filter_json["error"]["code"], "invalid_filter_json");

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn record_resolve_reports_ambiguity() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-ambiguity");
    write_ambiguous_action_source(&root)?;
    let index_path = root.join("artifact.sqlite");
    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;
    assert!(build_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "resolve", "Duplicate Action", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "record_resolution_ambiguous");
    let data = &json["error"]["data"];
    assert_eq!(
        data["result"]["error"]["code"],
        "record_resolution_ambiguous"
    );
    assert!(
        data["result"]["alternatives"]
            .as_array()
            .expect("ambiguity alternatives")
            .iter()
            .all(|alternative| alternative["record"]["presentation_type"] == "unmigrated")
    );
    assert_eq!(data["result"]["alternatives"].as_array().unwrap().len(), 2);
    assert_eq!(
        data["result"]["alternatives"][0]["record"]["key"],
        "actions:duplicateAction1"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn tooling_records_are_hidden_from_default_resolution_and_search_but_gettable_by_key()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-tooling-collision");
    write_tooling_collision_source(&root)?;
    let index_path = root.join("artifact.sqlite");
    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;
    assert!(build_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "resolve", "Treat Wounds", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["result"]["record"]["key"], "actions:testAction0001");
    assert_eq!(data["result"]["record"]["kind"], "rule");

    let search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(search_output.status.success());
    let search_json: Value = serde_json::from_slice(&search_output.stdout)?;
    let search_data = ok_data(&search_json);
    assert_eq!(search_data["pagination"]["total"], 1);
    assert_eq!(
        search_data["results"][0]["record"]["key"],
        "actions:testAction0001"
    );

    let filtered_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--kind",
            "rule",
            "--pack-name",
            "actions",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(filtered_search_output.status.success());
    let filtered_search_json: Value = serde_json::from_slice(&filtered_search_output.stdout)?;
    let filtered_search_data = ok_data(&filtered_search_json);
    assert_eq!(filtered_search_data["pagination"]["total"], 1);
    assert_eq!(filtered_search_data["filter"]["kind"], "all_of");
    assert_eq!(
        filtered_search_data["filter"]["children"][1]["predicate"]["field"],
        "pack_name"
    );

    let filtered_resolve_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Treat Wounds",
            "--pack-name",
            "actions",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(filtered_resolve_output.status.success());
    let filtered_resolve_json: Value = serde_json::from_slice(&filtered_resolve_output.stdout)?;
    let filtered_resolve_data = ok_data(&filtered_resolve_json);
    assert_eq!(
        filtered_resolve_data["result"]["record"]["key"],
        "actions:testAction0001"
    );

    let get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "pf2e-macros:macroTreatWounds", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(get_output.status.success());
    let get_json: Value = serde_json::from_slice(&get_output.stdout)?;
    let get_data = ok_data(&get_json);
    assert_eq!(get_data["record"]["key"], "pf2e-macros:macroTreatWounds");
    assert_eq!(get_data["record"]["kind"], "tooling");

    fs::remove_dir_all(root)?;
    Ok(())
}
