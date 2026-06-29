use std::io::Write;
use std::process::{Command, Stdio};

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
            "--tag",
            "arc-one",
            "--tag",
            "boss",
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
        create_data["list"]["tags"],
        serde_json::json!(["arc-one", "boss"])
    );
    assert_eq!(
        create_data["local_state_path"],
        local_state_path.display().to_string()
    );

    let duplicate_create_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "create",
            "undead-research",
            "--name",
            "Duplicate",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(duplicate_create_output.status.code(), Some(1));
    let duplicate_create_json: Value = serde_json::from_slice(&duplicate_create_output.stdout)?;
    assert_eq!(
        duplicate_create_json["error"]["code"],
        "saved_list_already_exists"
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
    assert_eq!(add_data["record_name"], "Test Action 1");

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
    assert_eq!(
        show_data["list"]["tags"],
        serde_json::json!(["arc-one", "boss"])
    );
    assert_eq!(show_data["list"]["item_count"], 1);
    assert_eq!(show_data["item_count"], 1);
    assert_eq!(show_data["items"].as_array().unwrap().len(), 1);
    assert_eq!(show_data["items"][0]["record_key"], "actions:testAction1");
    assert_eq!(show_data["items"][0]["position"], 1);
    assert_eq!(show_data["items"][0]["note"], "Check skeleton options");
    assert_eq!(show_data["items"][0]["status"], "active");
    assert_eq!(show_data["items"][0]["snapshot"]["name"], "Test Action 1");
    assert_eq!(
        show_data["items"][0]["record"]["key"],
        "actions:testAction1"
    );
    assert_eq!(show_data["items"][0]["record"]["name"], "Test Action 1");
    assert_eq!(show_data["items"][0]["record"]["kind"], "rule");
    assert!(show_data["items"][0]["record"].get("record_key").is_none());

    let ls_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "ls", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(ls_output.status.success());
    let ls_json: Value = serde_json::from_slice(&ls_output.stdout)?;
    let lists = ok_data(&ls_json)["lists"].as_array().unwrap();
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0]["name"], "Undead Research");
    assert_eq!(lists[0]["item_count"], 1);

    let summary_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["list", "show", "undead-research", "--summary", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(summary_output.status.success());
    let summary_json: Value = serde_json::from_slice(&summary_output.stdout)?;
    let summary_data = ok_data(&summary_json);
    assert_eq!(summary_data["item_count"], 1);
    assert_eq!(summary_data["items"][0]["position"], 1);
    assert_eq!(
        summary_data["items"][0]["record_key"],
        "actions:testAction1"
    );
    assert_eq!(summary_data["items"][0]["name"], "Test Action 1");
    assert_eq!(summary_data["items"][0]["kind"], "rule");
    assert_eq!(summary_data["items"][0]["note"], "Check skeleton options");
    assert!(summary_data["items"][0].get("record").is_none());
    assert!(summary_data["items"][0].get("snapshot").is_none());

    let keys_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "show", "undead-research", "--keys-only", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(keys_output.status.success());
    let keys_json: Value = serde_json::from_slice(&keys_output.stdout)?;
    assert_eq!(
        ok_data(&keys_json)["record_keys"],
        serde_json::json!(["actions:testAction1"])
    );

    let no_records_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "show",
            "undead-research",
            "--detail",
            "none",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(no_records_output.status.success());
    let no_records_json: Value = serde_json::from_slice(&no_records_output.stdout)?;
    let no_records_data = ok_data(&no_records_json);
    assert!(no_records_data["items"][0].get("record").is_none());
    assert_eq!(
        no_records_data["items"][0]["snapshot"]["name"],
        "Test Action 1"
    );

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
    let matches = ambiguous_json["error"]["data"]["matches"]
        .as_array()
        .expect("ambiguous response should include matches");
    assert!(
        matches
            .iter()
            .any(|record| record["key"] == "actions:testAction1")
    );
    assert!(
        matches
            .iter()
            .any(|record| record["key"] == "actions:testAction2")
    );
    assert!(
        matches
            .iter()
            .all(|record| record["name"] == "Duplicate Action")
    );
    assert!(matches.iter().all(|record| record["kind"] == "rule"));
    assert!(
        matches
            .iter()
            .all(|record| record.get("record_key").is_none())
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
fn lists_add_accepts_repeated_refs_and_reports_partial_failures()
-> Result<(), Box<dyn std::error::Error>> {
    let index_path = temp_index_path("lists-json-batch-add")?;
    let local_state_path = index_path.with_file_name("pf2e-local-state.sqlite");
    let _ = std::fs::remove_file(&local_state_path);
    create_valid_artifact_database(&index_path)?;
    create_list(&index_path, "batch-prep", "Batch Prep")?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "batch-prep",
            "Test Action 1",
            "No Such Record",
            "actions:testAction2",
            "--note",
            "review",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["requested_count"], 3);
    assert_eq!(data["added_count"], 2);
    assert_eq!(data["failed_count"], 1);
    assert_eq!(data["items"][0]["outcome"], "added");
    assert_eq!(data["items"][0]["record_key"], "actions:testAction1");
    assert_eq!(data["items"][0]["record_name"], "Test Action 1");
    assert_eq!(data["items"][1]["outcome"], "failed");
    assert_eq!(data["items"][1]["error"]["code"], "record_resolution_miss");
    assert_eq!(data["items"][2]["record_key"], "actions:testAction2");

    let show_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "show", "batch-prep", "--summary", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(show_output.status.success());
    let show_json: Value = serde_json::from_slice(&show_output.stdout)?;
    let show_data = ok_data(&show_json);
    assert_eq!(show_data["item_count"], 2);
    assert_eq!(show_data["items"][0]["record_key"], "actions:testAction1");
    assert_eq!(show_data["items"][0]["note"], "review");
    assert_eq!(show_data["items"][1]["record_key"], "actions:testAction2");
    assert_eq!(show_data["items"][1]["note"], "review");

    let _ = std::fs::remove_file(&local_state_path);
    Ok(())
}

#[test]
fn lists_add_reads_refs_from_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = temp_index_path("lists-json-stdin-add")?;
    let local_state_path = index_path.with_file_name("pf2e-local-state.sqlite");
    let _ = std::fs::remove_file(&local_state_path);
    create_valid_artifact_database(&index_path)?;
    create_list(&index_path, "stdin-prep", "Stdin Prep")?;

    let mut child = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "add", "stdin-prep", "--stdin", "--index"])
        .arg(&index_path)
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(b"Test Action 1\nactions:testAction2\n\n")?;
    let output = child.wait_with_output()?;
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["requested_count"], 2);
    assert_eq!(data["added_count"], 2);
    assert_eq!(data["failed_count"], 0);

    let _ = std::fs::remove_file(&local_state_path);
    Ok(())
}

#[test]
fn lists_export_import_and_edit_cover_portable_list_workflow()
-> Result<(), Box<dyn std::error::Error>> {
    let index_path = temp_index_path("lists-json-import-export")?;
    let local_state_path = index_path.with_file_name("pf2e-local-state.sqlite");
    let export_path = index_path.with_file_name("session-prep-export.json");
    let _ = std::fs::remove_file(&local_state_path);
    let _ = std::fs::remove_file(&export_path);
    create_valid_artifact_database(&index_path)?;
    create_list(&index_path, "session-prep", "Session Prep")?;
    let tag_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "edit",
            "session-prep",
            "--tag",
            "arc-one",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(tag_output.status.success());

    let add_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "add",
            "session-prep",
            "Test Action 1",
            "--note",
            "Bring up early",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(add_output.status.success());

    let stdout_export = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "export", "session-prep", "--index"])
        .arg(&index_path)
        .output()?;
    assert!(stdout_export.status.success());
    let export_json: Value = serde_json::from_slice(&stdout_export.stdout)?;
    assert!(export_json.get("status").is_none());
    assert_eq!(export_json["format"], "pf2e-atlas.saved-list");
    assert_eq!(export_json["version"], 1);
    assert_eq!(export_json["list"]["id"], "session-prep");
    assert_eq!(export_json["list"]["tags"], serde_json::json!(["arc-one"]));
    assert_eq!(export_json["items"][0]["record_key"], "actions:testAction1");
    assert_eq!(export_json["items"][0]["record_name"], "Test Action 1");

    let file_export = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "export", "session-prep", "--output"])
        .arg(&export_path)
        .arg("--index")
        .arg(&index_path)
        .output()?;
    assert!(file_export.status.success());
    assert!(file_export.stdout.is_empty());
    let file_json: Value = serde_json::from_slice(&std::fs::read(&export_path)?)?;
    assert_eq!(file_json["list"]["id"], "session-prep");

    let conflict = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "import"])
        .arg(&export_path)
        .args(["--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(conflict.status.code(), Some(1));
    let conflict_json: Value = serde_json::from_slice(&conflict.stdout)?;
    assert_eq!(conflict_json["error"]["code"], "saved_list_already_exists");

    let imported = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "import"])
        .arg(&export_path)
        .args(["--id", "session-copy", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(imported.status.success());
    let imported_json: Value = serde_json::from_slice(&imported.stdout)?;
    let imported_data = ok_data(&imported_json);
    assert_eq!(imported_data["list"]["slug"], "session-copy");
    assert_eq!(
        imported_data["list"]["tags"],
        serde_json::json!(["arc-one"])
    );
    assert_eq!(imported_data["replaced"], false);
    assert_eq!(imported_data["active_count"], 1);

    let edit = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "lists",
            "edit",
            "session-copy",
            "--id",
            "renamed-copy",
            "--name",
            "Renamed Copy",
            "--clear-description",
            "--tag",
            "arc-two",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(edit.status.success());
    let edit_json: Value = serde_json::from_slice(&edit.stdout)?;
    let edit_data = ok_data(&edit_json);
    assert_eq!(edit_data["list"]["slug"], "renamed-copy");
    assert_eq!(edit_data["list"]["name"], "Renamed Copy");
    assert_eq!(edit_data["list"]["tags"], serde_json::json!(["arc-two"]));
    assert!(edit_data["list"].get("description").is_none());

    let replaced = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["lists", "import"])
        .arg(&export_path)
        .args(["--id", "renamed-copy", "--replace", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(replaced.status.success());
    let replaced_json: Value = serde_json::from_slice(&replaced.stdout)?;
    let replaced_data = ok_data(&replaced_json);
    assert_eq!(replaced_data["list"]["slug"], "renamed-copy");
    assert_eq!(replaced_data["list"]["name"], "Session Prep");
    assert_eq!(
        replaced_data["list"]["tags"],
        serde_json::json!(["arc-one"])
    );
    assert_eq!(replaced_data["replaced"], true);

    let _ = std::fs::remove_file(&export_path);
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
