use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[test]
fn setup_default_mode_uses_global_install_paths() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("setup-auto-user");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--source"])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["path_mode"], "global");
    assert!(actual["repo_root"].is_null());
    assert_eq!(actual["paths"]["source"], source.display().to_string());
    assert_eq!(actual["paths"]["index"], index.display().to_string());
    assert!(
        actual["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| { check["kind"] == "validate_index" && check["status"] == "done" })
    );
    assert!(!actual["actions"].as_array().unwrap().iter().any(|action| {
        action["kind"] == "validate_index" || action["kind"] == "analyze_source"
    }));

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_repo_path_resolution_json_errors_use_standard_envelope()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("setup-repo-path-json-error");
    fs::create_dir_all(&root)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .current_dir(&root)
        .args(["setup", "--path-mode", "repo", "--json", "--check"])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(actual["status"], "error");
    assert_eq!(actual["error"]["code"], "invalid_input");
    assert!(
        actual["error"]["message"]
            .as_str()
            .unwrap()
            .contains("--path-mode repo requires")
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_json_reports_overridden_paths_and_default_model() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_root("cli-setup");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--path-mode", "global", "--source"])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], true);
    assert_eq!(actual["target"], "records");
    assert_eq!(actual["path_mode"], "global");
    assert_eq!(actual["paths"]["source"], source.display().to_string());
    assert_eq!(actual["embedding"]["model"], "bge-small-en-v1.5");
    assert_eq!(
        actual["embedding"]["cache_root"],
        cache.display().to_string()
    );
    assert_eq!(actual["embedding"]["ready"], false);
    assert_eq!(actual["paths"]["index"], index.display().to_string());
    assert_eq!(actual["readiness"]["records"]["status"], "ready");
    assert_eq!(actual["readiness"]["semantic_search"]["status"], "skipped");
    assert_eq!(actual["build"]["source_record_count"], 1);
    assert_eq!(actual["build"]["artifact_record_count"], 1);
    assert_eq!(actual["build"]["generated_record_count"], 0);
    assert_eq!(actual["build"]["pending_document_embedding_count"], 1);
    assert_eq!(actual["build"]["document_embedding_count"], 0);
    assert_eq!(actual["build"]["reused_document_embedding_count"], 0);
    assert_eq!(actual["build"]["generated_document_embedding_count"], 0);
    assert!(actual["build"]["build_duration_ms"].as_u64().is_some());
    assert_eq!(actual["build"]["embedding_tokenization_duration_ms"], 0);
    assert_eq!(actual["build"]["embedding_model_load_duration_ms"], 0);
    assert_eq!(actual["build"]["embedding_generation_duration_ms"], 0);
    assert!(index.is_file());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_human_output_reports_build_timing_summary() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-human-build");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let output = atlas_command()
        .args([
            "setup",
            "--path-mode",
            "global",
            "--offline",
            "--no-embeddings",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("build:"));
    let duration = value_after_prefix(&stdout, "  duration: ");
    assert_human_duration(duration);
    assert!(stdout.contains("  records: source=1 generated=0 artifact=1"));
    assert!(stdout.contains("  embeddings: pending_document=1 document=0 reused=0 generated=0"));
    assert!(stdout.contains("  embedding timing: tokenization=0ms model_load=0ms generation=0ms"));

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_records_second_run_skips_rebuild() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-idempotent");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let first = setup_records_offline(&source, &cache, &index)?;
    assert!(first.status.success());
    assert!(index.is_file());
    assert!(index.parent().unwrap().join("manifest.json").is_file());

    let second = setup_records_offline(&source, &cache, &index)?;
    assert!(second.status.success());
    let actual: Value = serde_json::from_slice(&second.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], true);
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "skipped" })
    );
    assert!(actual["checks"].as_array().unwrap().iter().any(|check| {
        check["kind"] == "analyze_source"
            && check["status"] == "skipped"
            && check["reason"] == "source position matched adjacent artifact manifest"
    }));

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_records_git_source_manifest_tracks_git_commit() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-git-position");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;
    let git_commit = initialize_git_source(&source)?;

    let output = setup_records_offline(&source, &cache, &index)?;
    assert!(output.status.success());

    let manifest_path = index.parent().unwrap().join("manifest.json");
    let manifest: Value = serde_json::from_slice(&fs::read(manifest_path)?)?;
    assert_eq!(manifest["source"]["git_commit"], git_commit);
    assert!(manifest["source"].get("fingerprint").is_none());
    assert_eq!(
        manifest["source"]["signature"],
        ok_data(&serde_json::from_slice::<Value>(&output.stdout)?)["build"]["source_signature"]
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_check_falls_back_to_source_analysis_when_manifest_source_position_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-position-stale");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let first = setup_records_offline(&source, &cache, &index)?;
    assert!(first.status.success());
    write_changed_fixture_record(&source)?;

    let output = setup_records_check(&source, &cache, &index)?;

    assert_eq!(output.status.code(), Some(1));
    assert_plans_rebuild_after_source_analysis(&output)?;

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_check_falls_back_to_source_analysis_for_dirty_git_source()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-dirty-git-position");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;
    initialize_git_source(&source)?;

    let first = setup_records_offline(&source, &cache, &index)?;
    assert!(first.status.success());
    write_changed_fixture_record(&source)?;

    let output = setup_records_check(&source, &cache, &index)?;

    assert_eq!(output.status.code(), Some(1));
    assert_plans_rebuild_after_source_analysis(&output)?;

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_check_falls_back_to_source_analysis_for_untracked_git_source()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-untracked-git-position");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;
    initialize_git_source(&source)?;

    let first = setup_records_offline(&source, &cache, &index)?;
    assert!(first.status.success());
    fs::write(
        source.join("packs/actions/new-untracked-action.json"),
        r#"{
          "_id": "newAction0001",
          "name": "Untracked Action",
          "type": "action",
          "system": {
            "description": { "value": "<p>An untracked action.</p>" },
            "traits": { "value": ["exploration"] }
          }
        }"#,
    )?;

    let output = setup_records_check(&source, &cache, &index)?;

    assert_eq!(output.status.code(), Some(1));
    assert_plans_rebuild_after_source_analysis(&output)?;

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_check_force_reports_not_ready() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-check-force");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let build_output = setup_records_offline(&source, &cache, &index)?;
    assert!(build_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "global",
            "--offline",
            "--check",
            "--force",
            "--no-embeddings",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert!(actual["actions"].as_array().unwrap().iter().any(|action| {
        action["kind"] == "build_index"
            && action["status"] == "planned"
            && action["reason"] == "force rebuild requested"
    }));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_rejects_removed_force_rebuild_flag() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--force-rebuild", "--check"])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("unexpected argument '--force-rebuild'"));
    assert!(stderr.contains("--force"));
    assert!(output.stdout.is_empty());
    Ok(())
}

#[test]
fn setup_full_check_reports_record_ready_when_vectors_missing()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-base-artifact-full-check");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let build_output = setup_records_offline(&source, &cache, &index)?;
    assert!(build_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "global",
            "--offline",
            "--check",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["records"]["status"], "ready");
    assert_eq!(
        actual["readiness"]["semantic_search"]["status"],
        "not_ready"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_failed_source_update_is_runtime_failure() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-fetch-fail");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let build_output = setup_records_offline(&source, &cache, &index)?;
    assert!(build_output.status.success());
    fs::create_dir(source.join(".git"))?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "global",
            "--no-embeddings",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["source"]["status"], "not_ready");
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "fetch_source" && action["status"] == "failed" })
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_failed_source_update_blocks_missing_index_build() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_root("cli-setup-fetch-fail-missing-index");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;
    fs::create_dir(source.join(".git"))?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "global",
            "--no-embeddings",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    assert!(!index.exists());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["source"]["status"], "not_ready");
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "blocked" })
    );
    assert!(
        !actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "done" })
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_check_offline_missing_source_uses_json_status_vocabulary()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-missing-source");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "global",
            "--offline",
            "--check",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["source"]["status"], "not_ready");
    assert_eq!(
        actual["readiness"]["embedding_model"]["status"],
        "not_ready"
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_check_online_missing_assets_plans_dependent_build()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("cli-setup-online-check-missing");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--path-mode", "global", "--check", "--source"])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "fetch_source" && action["status"] == "planned" })
    );
    assert!(actual["actions"].as_array().unwrap().iter().any(|action| {
        action["kind"] == "prepare_embedding_model" && action["status"] == "planned"
    }));
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "planned" })
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn assert_plans_rebuild_after_source_analysis(
    output: &std::process::Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert!(
        actual["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| { check["kind"] == "analyze_source" && check["status"] == "done" })
    );
    assert!(actual["actions"].as_array().unwrap().iter().any(|action| {
        action["kind"] == "build_index"
            && action["status"] == "planned"
            && action["reason"] == "source signature changed since the artifact was built"
    }));
    assert!(
        actual["not_ready_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| {
                reason["code"] == "artifact_stale_source_signature"
                    && reason["action"] == "build_index"
                    && reason["status"] == "planned"
                    && reason["message"] == "source signature changed since the artifact was built"
            })
    );
    Ok(())
}

fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
}

fn value_after_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .expect("expected output line")
}

fn assert_human_duration(value: &str) {
    assert!(
        value.ends_with("ms")
            || value.ends_with('s')
            || value.contains("m ")
            || value.contains("h "),
        "expected human-readable duration, got {value:?}"
    );
}

fn temp_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-cli-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn setup_records_offline(
    source: &Path,
    cache: &Path,
    index: &Path,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    Ok(atlas_command()
        .args([
            "setup",
            "--path-mode",
            "global",
            "--offline",
            "--no-embeddings",
            "--source",
        ])
        .arg(source)
        .args(["--embedding-cache-path"])
        .arg(cache)
        .args(["--index"])
        .arg(index)
        .arg("--json")
        .output()?)
}

fn setup_records_check(
    source: &Path,
    cache: &Path,
    index: &Path,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    Ok(atlas_command()
        .args([
            "setup",
            "--path-mode",
            "global",
            "--offline",
            "--check",
            "--no-embeddings",
            "--source",
        ])
        .arg(source)
        .args(["--embedding-cache-path"])
        .arg(cache)
        .args(["--index"])
        .arg(index)
        .arg("--json")
        .output()?)
}

fn initialize_git_source(source: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let init = git_command()
        .args(["init", "--quiet"])
        .current_dir(source)
        .output()?;
    assert!(init.status.success());
    let add = git_command()
        .args(["add", "."])
        .current_dir(source)
        .output()?;
    assert!(add.status.success());
    let commit = git_command()
        .args(["commit", "--quiet", "-m", "fixture"])
        .env("GIT_AUTHOR_NAME", "Atlas Test")
        .env("GIT_AUTHOR_EMAIL", "atlas@example.invalid")
        .env("GIT_COMMITTER_NAME", "Atlas Test")
        .env("GIT_COMMITTER_EMAIL", "atlas@example.invalid")
        .current_dir(source)
        .output()?;
    assert!(commit.status.success());
    let head = git_command()
        .args(["rev-parse", "HEAD"])
        .current_dir(source)
        .output()?;
    assert!(head.status.success());
    Ok(String::from_utf8(head.stdout)?.trim().to_string())
}

fn atlas_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
    clear_git_hook_env(&mut command);
    command
}

fn git_command() -> Command {
    let mut command = Command::new("git");
    clear_git_hook_env(&mut command);
    command
}

fn clear_git_hook_env(command: &mut Command) {
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
}

fn write_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/treat-wounds.json"),
        r#"{
          "_id": "testAction0001",
          "name": "Treat Wounds",
          "type": "action",
          "system": {
            "description": { "value": "<p>You treat wounds.</p>" },
            "traits": { "value": ["healing"] }
          }
        }"#,
    )?;
    Ok(())
}

fn write_changed_fixture_record(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        root.join("packs/actions/treat-wounds.json"),
        r#"{
          "_id": "testAction0001",
          "name": "Treat Wounds",
          "type": "action",
          "system": {
            "description": { "value": "<p>You treat changed wounds.</p>" },
            "traits": { "value": ["healing"] }
          }
        }"#,
    )?;
    Ok(())
}
