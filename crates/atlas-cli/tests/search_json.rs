use std::fs;
use std::process::Command;

use serde_json::Value;

mod support;

use support::json::{ok_data, record_sections};
use support::path::temp_source_root;
use support::source::{write_ambiguous_action_source, write_creature_preview_source};

#[test]
fn filter_search_reports_pagination_and_random_seed() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-search-pagination-random");
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

    let paged_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--limit", "1", "--offset", "1", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(paged_output.status.success());
    let paged_json: Value = serde_json::from_slice(&paged_output.stdout)?;
    let paged_data = ok_data(&paged_json);
    assert_eq!(paged_data["pagination"]["offset"], 1);
    assert_eq!(paged_data["pagination"]["limit"], 1);
    assert_eq!(paged_data["pagination"]["count"], 1);
    assert_eq!(paged_data["pagination"]["total"], 2);
    assert_eq!(paged_data["pagination"]["has_more"], false);
    assert!(paged_data["pagination"].get("next_offset").is_none());

    let seeded_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--sort", "random", "--seed", "1234", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(seeded_output.status.success());
    let seeded_json: Value = serde_json::from_slice(&seeded_output.stdout)?;
    let seeded_data = ok_data(&seeded_json);
    assert_eq!(seeded_data["sort"]["kind"], "random");
    assert_eq!(seeded_data["sort"]["seed"], 1234);

    let generated_seed_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--sort", "random", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(generated_seed_output.status.success());
    let generated_seed_json: Value = serde_json::from_slice(&generated_seed_output.stdout)?;
    let generated_seed_data = ok_data(&generated_seed_json);
    assert_eq!(generated_seed_data["sort"]["kind"], "random");
    assert!(generated_seed_data["sort"]["seed"].as_u64().is_some());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn search_preview_prints_family_metric_facts() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-search-preview-facts");
    write_creature_preview_source(&root)?;
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
        .args([
            "search",
            "--family",
            "creature",
            "--metric",
            "ac.value>=25",
            "--detail",
            "preview",
            "--index",
        ])
        .arg(&index_path)
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("creatures:testCreature001  Test Guardian  creature 5"));
    assert!(
        stdout.contains("Summary: Size Med; Languages Common; Senses Darkvision; Perception +12")
    );
    assert!(stdout.contains("Defense: AC 25; HP 80; Saves Fort +14, Ref +11, Will +12"));
    assert!(stdout.contains("Movement: Speed Land 25 feet; Speed Types Land"));

    let json_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--family",
            "creature",
            "--metric",
            "ac.value>=25",
            "--detail",
            "preview",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(json_output.status.success());
    let json: Value = serde_json::from_slice(&json_output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(
        record_sections(&data["results"][0]["record"]),
        vec!["summary", "defense", "movement"]
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn search_rejects_invalid_filter_json_before_runtime_loading()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--index",
            "missing.sqlite",
            "--filter-json",
            "{",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_filter_json");
    Ok(())
}

#[test]
fn search_rejects_filter_json_with_convenience_flags_before_runtime_loading()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--index",
            "missing.sqlite",
            "--filter-json",
            r#"{"kind":"record_family","value":"rule"}"#,
            "--family",
            "rule",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_filter");
    Ok(())
}

#[test]
fn search_print_filter_lowers_convenience_flags_before_runtime_loading()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--family",
            "spell",
            "--references",
            "spells:fireball",
            "--referenced-by",
            "actions:activate",
            "--price",
            "100..500",
            "--metric",
            "ac.value>=18",
            "--print-filter",
            "--json",
        ])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["filter"]["kind"], "all_of");
    assert_eq!(data["filter"]["children"][0]["kind"], "record_kind");
    assert_eq!(
        data["filter"]["children"][1]["predicate"]["field"],
        "price_cp"
    );
    assert_eq!(data["filter"]["children"][2]["kind"], "links_to");
    assert_eq!(data["filter"]["children"][3]["kind"], "linked_from");
    assert_eq!(data["filter"]["children"][4]["kind"], "metric");
    assert_eq!(data["filter"]["children"][4]["metric"], "ac.value");
    assert_eq!(data["filter"]["children"][4]["match"]["kind"], "gte");
    assert_eq!(data["filter"]["children"][4]["match"]["value"], 18.0);
    Ok(())
}

#[test]
fn search_rejects_unweighted_rrf_with_lane_weights() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--fusion",
            "rrf",
            "--fts-weight",
            "2",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_option");
    Ok(())
}
