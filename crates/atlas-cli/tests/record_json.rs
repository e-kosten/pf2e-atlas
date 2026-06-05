use std::fs;
use std::process::Command;

use serde_json::Value;

mod support;

use support::json::{ok_data, record_sections};
use support::path::temp_source_root;
use support::source::{
    write_ambiguous_action_source, write_record_search_source, write_tooling_collision_source,
};

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
    assert!(preview_get_output.status.success());
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
    let description_sections = serde_json::to_string(&description_get_data["record"]["sections"])?;
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
    assert!(successful_batch_get_output.status.success());
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
    assert!(text_resolve_description_stdout.contains("actions:testAction0001  Treat Wounds  rule"));
    assert!(text_resolve_description_stdout.contains("Source: Actions"));
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

    let offset_window_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--limit",
            "1",
            "--offset",
            "2",
            "--fts-top-k",
            "1",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(offset_window_output.status.success());
    let offset_window_json: Value = serde_json::from_slice(&offset_window_output.stdout)?;
    let offset_window_data = ok_data(&offset_window_json);
    assert_eq!(offset_window_data["candidate_windows"]["fts_top_k"], 3);

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
            "--offset",
            "5000",
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
    assert!(text_get_stdout.contains("actions:testAction0001\tTreat Wounds\trule"));
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
    assert!(text_search_preview_stdout.contains("actions:testAction0001  Treat Wounds  rule"));
    assert!(text_search_preview_stdout.contains("Source: Actions"));
    assert!(
        text_search_preview_stdout
            .contains("You spend 10 minutes treating one injured living creature with")
    );
    assert!(text_search_preview_stdout.contains("Match: filter"));

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
