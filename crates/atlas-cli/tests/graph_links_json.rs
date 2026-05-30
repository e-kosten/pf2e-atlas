use std::fs;
use std::process::Command;

use rusqlite::Connection;
use serde_json::Value;

mod support;

use support::db::{create_contract_database, ok_data, temp_db_path};
use support::graph::{
    assert_section_edges_point_to_returned_records, insert_graph_edges, insert_reference_edge,
    set_record_visibility,
};

#[test]
fn graph_links_json_returns_bounded_context() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "graph",
            "links",
            "actions:testAction1",
            "--outgoing",
            "1",
            "--backlinks",
            "1",
            "--index",
        ])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["detail"], "summary");
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert_eq!(data["outgoing"]["total_records"], 2);
    assert_eq!(data["outgoing"]["total_edges"], 3);
    assert_eq!(data["outgoing"]["truncated"], true);
    assert_eq!(data["outgoing"]["records"].as_array().unwrap().len(), 1);
    assert_eq!(data["outgoing"]["records"][0]["key"], "actions:testAction2");
    assert_eq!(data["outgoing"]["edges"].as_array().unwrap().len(), 2);
    assert_eq!(data["outgoing"]["edges"][0]["from"], "actions:testAction1");
    assert_eq!(data["outgoing"]["edges"][0]["to"], "actions:testAction2");
    assert_eq!(
        data["outgoing"]["edges"][0]["source"]["kind"],
        "description"
    );
    assert_eq!(
        data["outgoing"]["edges"][0]["source"]["visibility"],
        "public"
    );
    assert!(data.get("edges").is_none());
    assert_section_edges_point_to_returned_records(&data["outgoing"], "to");

    assert_eq!(data["backlinks"]["total_records"], 2);
    assert_eq!(data["backlinks"]["total_edges"], 2);
    assert_eq!(data["backlinks"]["truncated"], true);
    assert_eq!(data["backlinks"]["records"].as_array().unwrap().len(), 1);
    assert_eq!(
        data["backlinks"]["records"][0]["key"],
        "actions:testAction2"
    );
    assert_eq!(data["backlinks"]["edges"].as_array().unwrap().len(), 1);
    assert_section_edges_point_to_returned_records(&data["backlinks"], "from");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_keeps_empty_sections_stable() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-empty");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "graph",
            "links",
            "actions:testAction1",
            "--outgoing",
            "0",
            "--backlinks",
            "0",
            "--index",
        ])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["outgoing"]["total_records"], 0);
    assert_eq!(data["outgoing"]["total_edges"], 0);
    assert_eq!(data["outgoing"]["records"].as_array().unwrap().len(), 0);
    assert_eq!(data["outgoing"]["edges"].as_array().unwrap().len(), 0);
    assert_eq!(data["backlinks"]["total_records"], 0);
    assert_eq!(data["backlinks"]["total_edges"], 0);
    assert_eq!(data["backlinks"]["records"].as_array().unwrap().len(), 0);
    assert_eq!(data["backlinks"]["edges"].as_array().unwrap().len(), 0);

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_defaults_to_outgoing_only() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-default");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "links", "actions:testAction1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["outgoing"]["total_records"], 2);
    assert_eq!(data["outgoing"]["total_edges"], 3);
    assert_eq!(data["backlinks"]["total_records"], 0);
    assert_eq!(data["backlinks"]["total_edges"], 0);
    assert_eq!(data["backlinks"]["records"].as_array().unwrap().len(), 0);
    assert_eq!(data["backlinks"]["edges"].as_array().unwrap().len(), 0);

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_supports_backlinks_only() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-backlinks-only");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "graph",
            "links",
            "actions:testAction1",
            "--outgoing",
            "0",
            "--backlinks",
            "1",
            "--index",
        ])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["outgoing"]["total_records"], 0);
    assert_eq!(data["outgoing"]["total_edges"], 0);
    assert_eq!(data["backlinks"]["total_records"], 2);
    assert_eq!(data["backlinks"]["total_edges"], 2);
    assert_eq!(data["backlinks"]["records"].as_array().unwrap().len(), 1);
    assert_section_edges_point_to_returned_records(&data["backlinks"], "from");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_reports_missing_seed_like_record_get() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-graph-missing");
    create_contract_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "links", "actions:missing", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "record_not_found");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_rejects_invalid_limit() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-miss");
    create_contract_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "links", "not-a-key", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "record_resolution_miss");

    for (flag, value) in [
        ("--outgoing", "51"),
        ("--backlinks", "51"),
        ("--outgoing", "-1"),
        ("--backlinks", "not-a-number"),
    ] {
        let limit_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args([
                "graph",
                "links",
                "actions:testAction1",
                flag,
                value,
                "--json",
            ])
            .output()?;

        assert_eq!(limit_output.status.code(), Some(2));
        let limit_json: Value = serde_json::from_slice(&limit_output.stdout)?;
        assert_eq!(limit_json["status"], "error");
        assert_eq!(limit_json["error"]["code"], "invalid_input");
    }
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_reports_ambiguous_name_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-ambiguous");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET name = 'Shared Action', normalized_name = 'shared action'
         WHERE record_key IN ('actions:testAction1', 'actions:testAction2')",
        [],
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "links", "Shared Action", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "record_resolution_ambiguous");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_reports_resolution_query_failures_as_operational()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-resolution-failure");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute("DROP TABLE record_aliases", [])?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "links", "Test Action 1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "query_failed");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_accepts_upper_bound_and_detail() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-upper-bound");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "graph",
            "links",
            "actions:testAction1",
            "--outgoing",
            "50",
            "--detail",
            "description",
            "--index",
        ])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["detail"], "description");
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_human_output_is_summary_oriented() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-human");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "graph",
            "links",
            "actions:testAction1",
            "--outgoing",
            "1",
            "--index",
        ])
        .arg(&path)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("actions:testAction1"));
    assert!(stdout.contains("Outgoing: 1 records, 2 edges (of 2 records, 3 edges)"));
    assert!(stdout.contains("Backlinks: disabled"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_preserves_localized_and_null_display_text()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-localized");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_reference_edge(
        &connection,
        "actions:testAction1",
        "actions:testAction2",
        Some("Échapper"),
        "@UUID[Compendium.pf2e.actions.Item.testAction2]{Échapper}",
        "description",
        "public",
    )?;
    insert_reference_edge(
        &connection,
        "actions:testAction1",
        "actions:testAction3",
        None,
        "@UUID[Compendium.pf2e.actions.Item.testAction3]",
        "description",
        "public",
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "links", "actions:testAction1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert!(data["outgoing"]["edges"][0].get("display_text").is_none());
    assert_eq!(data["outgoing"]["edges"][1]["display_text"], "Échapper");
    assert_eq!(
        data["outgoing"]["edges"][1]["reference_text"],
        "@UUID[Compendium.pf2e.actions.Item.testAction2]{Échapper}"
    );

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_links_json_accepts_non_default_visible_seed() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-hidden-seed");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;
    set_record_visibility(&path, "actions:testAction1", false)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "links", "actions:testAction1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert_eq!(data["outgoing"]["total_records"], 2);
    assert_eq!(data["outgoing"]["total_edges"], 3);

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_get_command_is_removed() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "get", "actions:testAction1", "--json"])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_input");
    Ok(())
}
