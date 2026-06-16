use std::process::Command;

use rusqlite::Connection;
use serde_json::Value;

mod support;

use support::db::{create_valid_artifact_database, ok_data};

#[test]
fn lists_create_add_show_remove_and_delete() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = temp_index_path("lists-json-artifact")?;
    let local_state_path = index_path.with_file_name("pf2e-local-state.sqlite");
    let _ = std::fs::remove_file(&local_state_path);
    create_valid_artifact_database(&index_path)?;

    let create_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "create",
            "undead-research",
            "--name",
            "Undead Research",
            "--description",
            "Campaign prep",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(create_output.status.success());
    let create_json: Value = serde_json::from_slice(&create_output.stdout)?;
    let create_data = ok_data(&create_json);
    assert_eq!(create_data["list"]["slug"], "undead-research");
    assert_eq!(create_data["list"]["name"], "Undead Research");
    assert_eq!(create_data["list"]["description"], "Campaign prep");
    assert_eq!(
        create_data["local_state_path"],
        local_state_path.display().to_string()
    );

    let add_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "undead-research",
            "Test Action 1",
            "--note",
            "Check skeleton options",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(add_output.status.success());
    let add_json: Value = serde_json::from_slice(&add_output.stdout)?;
    let add_data = ok_data(&add_json);
    assert_eq!(add_data["outcome"], "added");
    assert_eq!(add_data["record_key"], "actions:testAction1");

    let duplicate_add_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "undead-research",
            "actions:testAction1",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(duplicate_add_output.status.success());
    let duplicate_add_json: Value = serde_json::from_slice(&duplicate_add_output.stdout)?;
    assert_eq!(ok_data(&duplicate_add_json)["outcome"], "already_present");

    let show_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "show", "undead-research", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(show_output.status.success());
    let show_json: Value = serde_json::from_slice(&show_output.stdout)?;
    let show_data = ok_data(&show_json);
    assert_eq!(show_data["list"]["slug"], "undead-research");
    assert_eq!(show_data["items"].as_array().unwrap().len(), 1);
    assert_eq!(show_data["items"][0]["record_key"], "actions:testAction1");
    assert_eq!(show_data["items"][0]["position"], 1);
    assert_eq!(show_data["items"][0]["note"], "Check skeleton options");
    assert_eq!(show_data["items"][0]["status"], "active");
    assert_eq!(show_data["items"][0]["snapshot"]["name"], "Test Action 1");
    assert_eq!(show_data["items"][0]["record"]["name"], "Test Action 1");

    let remove_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "remove",
            "undead-research",
            "actions:testAction1",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(remove_output.status.success());
    let remove_json: Value = serde_json::from_slice(&remove_output.stdout)?;
    assert_eq!(ok_data(&remove_json)["outcome"], "removed");

    let absent_remove_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "remove",
            "undead-research",
            "actions:testAction2",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(absent_remove_output.status.success());
    let absent_remove_json: Value = serde_json::from_slice(&absent_remove_output.stdout)?;
    assert_eq!(ok_data(&absent_remove_json)["outcome"], "not_present");

    let delete_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "delete", "undead-research", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(delete_output.status.success());
    let delete_json: Value = serde_json::from_slice(&delete_output.stdout)?;
    assert_eq!(ok_data(&delete_json)["deleted"], true);

    let deleted_show_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "show", "undead-research", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(deleted_show_output.status.code(), Some(1));
    let deleted_show_json: Value = serde_json::from_slice(&deleted_show_output.stdout)?;
    assert_eq!(deleted_show_json["error"]["code"], "saved_list_not_found");

    let _ = std::fs::remove_file(&local_state_path);
    Ok(())
}

#[test]
fn lists_show_preserves_unresolved_items_after_artifact_change()
-> Result<(), Box<dyn std::error::Error>> {
    let index_path = temp_index_path("lists-json-unresolved")?;
    let local_state_path = index_path.with_file_name("pf2e-local-state.sqlite");
    let _ = std::fs::remove_file(&local_state_path);
    create_valid_artifact_database(&index_path)?;
    create_list(&index_path, "stale-research", "Stale Research")?;

    let add_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "stale-research",
            "Test Action 1",
            "--note",
            "Keep this even if stale",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(add_output.status.success());

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "DELETE FROM records WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    drop(connection);

    let show_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "show", "stale-research", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(show_output.status.success());
    let show_json: Value = serde_json::from_slice(&show_output.stdout)?;
    let show_data = ok_data(&show_json);
    assert_eq!(show_data["items"].as_array().unwrap().len(), 1);
    assert_eq!(show_data["items"][0]["record_key"], "actions:testAction1");
    assert_eq!(show_data["items"][0]["position"], 1);
    assert_eq!(show_data["items"][0]["status"], "unresolved");
    assert_eq!(show_data["items"][0]["note"], "Keep this even if stale");
    assert_eq!(show_data["items"][0]["snapshot"]["name"], "Test Action 1");
    assert_eq!(show_data["items"][0]["snapshot"]["kind"], "rule");
    assert!(show_data["items"][0].get("record").is_none());

    let _ = std::fs::remove_file(&local_state_path);
    Ok(())
}

#[test]
fn lists_ls_orders_by_slug() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = temp_index_path("lists-json-ls")?;
    let local_state_path = index_path.with_file_name("pf2e-local-state.sqlite");
    let _ = std::fs::remove_file(&local_state_path);
    create_valid_artifact_database(&index_path)?;

    create_list(&index_path, "z-last", "Z Last")?;
    create_list(&index_path, "a-first", "A First")?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "ls", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["lists"][0]["slug"], "a-first");
    assert_eq!(data["lists"][1]["slug"], "z-last");

    let _ = std::fs::remove_file(&local_state_path);
    Ok(())
}

#[test]
fn lists_add_rejects_miss_and_ambiguity_without_inserting() -> Result<(), Box<dyn std::error::Error>>
{
    let index_path = temp_index_path("lists-json-resolution")?;
    let local_state_path = index_path.with_file_name("pf2e-local-state.sqlite");
    let _ = std::fs::remove_file(&local_state_path);
    create_valid_artifact_database(&index_path)?;
    create_list(&index_path, "resolution-checks", "Resolution Checks")?;

    let miss_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "resolution-checks",
            "No Such Record",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(miss_output.status.code(), Some(1));
    let miss_json: Value = serde_json::from_slice(&miss_output.stdout)?;
    assert_eq!(miss_json["error"]["code"], "record_resolution_miss");

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "UPDATE records
         SET name = 'Duplicate Action',
             normalized_name = 'duplicate action'
         WHERE record_key IN ('actions:testAction1', 'actions:testAction2')",
        [],
    )?;
    drop(connection);

    let ambiguous_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "resolution-checks",
            "Duplicate Action",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(ambiguous_output.status.code(), Some(1));
    let ambiguous_json: Value = serde_json::from_slice(&ambiguous_output.stdout)?;
    assert_eq!(
        ambiguous_json["error"]["code"],
        "record_resolution_ambiguous"
    );
    assert!(
        ambiguous_json["error"]["data"]["matches"]
            .as_array()
            .is_some_and(|matches| matches.len() >= 2)
    );

    let show_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "show", "resolution-checks", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(show_output.status.success());
    let show_json: Value = serde_json::from_slice(&show_output.stdout)?;
    assert_eq!(ok_data(&show_json)["items"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_file(&local_state_path);
    Ok(())
}

#[test]
fn lists_json_setup_failures_return_domain_exit_without_extra_stderr()
-> Result<(), Box<dyn std::error::Error>> {
    let missing_index_path = temp_index_path("lists-json-missing-index")?;
    let index_open_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "add", "runtime-checks", "Test Action 1", "--index"])
        .arg(&missing_index_path)
        .arg("--json")
        .output()?;
    assert_eq!(index_open_output.status.code(), Some(3));
    assert!(index_open_output.stderr.is_empty());
    let index_open_json: Value = serde_json::from_slice(&index_open_output.stdout)?;
    assert_eq!(index_open_json["error"]["code"], "index_unavailable");

    let local_state_index_path = temp_index_path("lists-json-local-state-open")?;
    let local_state_path = local_state_index_path.with_file_name("pf2e-local-state.sqlite");
    create_valid_artifact_database(&local_state_index_path)?;
    std::fs::create_dir(&local_state_path)?;

    let local_state_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "ls", "--index"])
        .arg(&local_state_index_path)
        .arg("--json")
        .output()?;
    assert_eq!(local_state_output.status.code(), Some(3));
    assert!(local_state_output.stderr.is_empty());
    let local_state_json: Value = serde_json::from_slice(&local_state_output.stdout)?;
    assert_eq!(local_state_json["error"]["code"], "local_state_error");
    std::fs::remove_dir(&local_state_path)?;

    let hydration_index_path = temp_index_path("lists-json-hydration-failure")?;
    let hydration_local_state_path = hydration_index_path.with_file_name("pf2e-local-state.sqlite");
    create_valid_artifact_database(&hydration_index_path)?;
    create_list(&hydration_index_path, "broken-artifact", "Broken Artifact")?;

    let add_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "broken-artifact",
            "Test Action 1",
            "--index",
        ])
        .arg(&hydration_index_path)
        .arg("--json")
        .output()?;
    assert!(add_output.status.success());

    let connection = Connection::open(&hydration_index_path)?;
    connection.execute("DROP TABLE records", [])?;
    drop(connection);

    let hydration_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "show", "broken-artifact", "--index"])
        .arg(&hydration_index_path)
        .arg("--json")
        .output()?;
    assert_eq!(hydration_output.status.code(), Some(3));
    assert!(hydration_output.stderr.is_empty());
    let hydration_json: Value = serde_json::from_slice(&hydration_output.stdout)?;
    assert_eq!(hydration_json["error"]["code"], "query_failed");

    let _ = std::fs::remove_file(&hydration_local_state_path);
    Ok(())
}

fn create_list(
    index_path: &std::path::Path,
    slug: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "create", slug, "--name", name, "--index"])
        .arg(index_path)
        .arg("--json")
        .output()?;
    assert!(output.status.success());
    Ok(())
}

fn temp_index_path(name: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "atlas-cli-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("artifact.sqlite"))
}
