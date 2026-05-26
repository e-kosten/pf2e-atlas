use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use atlas_artifact::test_support::{
    create_minimal_contract_schema, insert_contract_metadata_entries, insert_minimal_contract_rows,
    legacy_minilm_metadata_entries,
};
use rusqlite::Connection;
use serde_json::Value;

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
fn graph_variants_json_returns_group_siblings() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-variants");
    create_contract_database(&path)?;
    insert_variant_group(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "Test Action 1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert_eq!(data["variant_group_key"], "test-action");
    let variants = data["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0]["record"]["key"], "actions:testAction1");
    assert_eq!(variants[0]["is_seed"], true);
    assert_eq!(variants[0]["variant_label"], "Lesser");
    assert_eq!(variants[0]["variant_axes"][0], "grade");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_variants_json_resolves_variant_base_name() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-variant-base");
    create_contract_database(&path)?;
    insert_variant_group(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "Test Action", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert!(data.get("seed").is_none());
    assert_eq!(data["variant_group_key"], "test-action");
    let variants = data["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0]["record"]["key"], "actions:testAction1");
    assert_eq!(variants[0]["is_seed"], false);

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_remaster_json_returns_legacy_and_remaster_links() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-graph-remaster");
    create_contract_database(&path)?;
    insert_remaster_link(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "remaster", "actions:testAction1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    let links = data["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0]["direction"], "legacy_to_remaster");
    assert_eq!(links[0]["legacy"]["key"], "actions:testAction1");
    assert_eq!(links[0]["remaster"]["key"], "actions:testAction2");
    assert_eq!(links[0]["source"]["kind"], "migration");

    fs::remove_file(path)?;
    Ok(())
}

fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
}

fn insert_variant_group(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    for (record_key, level, label) in [
        ("actions:testAction1", 1_i64, "Lesser"),
        ("actions:testAction2", 3_i64, "Moderate"),
        ("actions:testAction3", 5_i64, "Greater"),
    ] {
        connection.execute(
            "UPDATE records
             SET variant_group_key = 'test-action',
                 variant_base_name = 'Test Action',
                 variant_label = ?1,
                 variant_axes_json = '[\"grade\"]',
                 variant_confidence = 1.0,
                 variant_source = 'test',
                 level = ?2
             WHERE record_key = ?3",
            (label, level, record_key),
        )?;
    }
    Ok(())
}

fn insert_remaster_link(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "INSERT INTO remaster_links (
           remaster_record_key, legacy_record_key, source_kind, source_ref
         ) VALUES (?1, ?2, ?3, ?4)",
        (
            "actions:testAction2",
            "actions:testAction1",
            "migration",
            "test migration",
        ),
    )?;
    Ok(())
}

fn assert_section_edges_point_to_returned_records(section: &Value, neighbor_field: &str) {
    let record_keys = section["records"]
        .as_array()
        .expect("records should be an array")
        .iter()
        .map(|record| {
            record["key"]
                .as_str()
                .expect("record key should be a string")
        })
        .collect::<Vec<_>>();
    for edge in section["edges"]
        .as_array()
        .expect("edges should be an array")
    {
        let neighbor = edge[neighbor_field]
            .as_str()
            .expect("edge endpoint should be a string");
        assert!(
            record_keys.contains(&neighbor),
            "edge endpoint {neighbor} should be present in returned records"
        );
    }
}

fn create_contract_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata_entries(&connection, legacy_minilm_metadata_entries(), None)?;
    insert_minimal_contract_rows(&connection)?;
    Ok(())
}

fn insert_graph_edges(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    for (from, to, display, reference, source_kind, visibility) in [
        (
            "actions:testAction1",
            "actions:testAction2",
            Some("Alpha"),
            "@UUID[Compendium.pf2e.actions.Item.testAction2]{Alpha}",
            "description",
            "public",
        ),
        (
            "actions:testAction1",
            "actions:testAction2",
            Some("Beta"),
            "@UUID[Compendium.pf2e.actions.Item.testAction2]{Beta}",
            "description",
            "public",
        ),
        (
            "actions:testAction1",
            "actions:testAction3",
            Some("Gamma"),
            "@UUID[Compendium.pf2e.actions.Item.testAction3]{Gamma}",
            "description",
            "public",
        ),
        (
            "actions:testAction1",
            "actions:testAction3",
            Some("Private"),
            "@UUID[Compendium.pf2e.actions.Item.testAction3]{Private}",
            "description",
            "private",
        ),
        (
            "actions:testAction2",
            "actions:testAction1",
            Some("Incoming A"),
            "@UUID[Compendium.pf2e.actions.Item.testAction1]{Incoming A}",
            "description",
            "public",
        ),
        (
            "actions:testAction3",
            "actions:testAction1",
            Some("Incoming B"),
            "@UUID[Compendium.pf2e.actions.Item.testAction1]{Incoming B}",
            "description",
            "public",
        ),
    ] {
        insert_reference_edge(
            &connection,
            from,
            to,
            display,
            reference,
            source_kind,
            visibility,
        )?;
    }
    Ok(())
}

fn insert_reference_edge(
    connection: &Connection,
    from: &str,
    to: &str,
    display: Option<&str>,
    reference: &str,
    source_kind: &str,
    visibility: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute(
        "INSERT INTO reference_edges (
           from_record_key, to_record_key, display_text, reference_text, source_kind, visibility
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (from, to, display, reference, source_kind, visibility),
    )?;
    Ok(())
}

fn set_record_visibility(
    path: &Path,
    record_key: &str,
    visible: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "UPDATE records SET is_default_visible = ?1 WHERE record_key = ?2",
        (if visible { 1_i64 } else { 0_i64 }, record_key),
    )?;
    Ok(())
}

fn temp_db_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-cli-{name}-{}-{}.sqlite",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_file(&path);
    path
}
