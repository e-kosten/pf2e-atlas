use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[test]
fn setup_clean_check_reports_targets_without_removing() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("setup-clean-check");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    fs::create_dir_all(&source)?;
    fs::create_dir_all(&cache)?;
    fs::write(&index, "sqlite")?;
    fs::write(wal_path(&index), "wal")?;

    let output = atlas_setup_clean(&source, &cache, &index, &["--all", "--check"])?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["check"], true);
    assert_eq!(actual["targets"].as_array().unwrap().len(), 3);
    assert!(
        actual["targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|target| target["status"] == "planned")
    );
    assert!(source.is_dir());
    assert!(cache.is_dir());
    assert!(index.is_file());
    assert!(wal_path(&index).is_file());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_clean_help_lists_cleanup_targets() -> Result<(), Box<dyn std::error::Error>> {
    let setup_help = help_output(&["setup"])?;
    assert!(setup_help.contains("atlas setup clean --artifact"));

    let setup_clean_help = help_output(&["setup", "clean"])?;
    assert!(setup_clean_help.contains("--artifact"));
    assert!(setup_clean_help.contains("--embeddings"));
    assert!(setup_clean_help.contains("--source-checkout"));
    assert!(setup_clean_help.contains("--all"));
    assert!(setup_clean_help.contains("--yes"));
    Ok(())
}

#[test]
fn setup_clean_all_removes_runtime_data_with_confirmation() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_root("setup-clean-all");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    fs::create_dir_all(&source)?;
    fs::create_dir_all(&cache)?;
    fs::write(&index, "sqlite")?;
    fs::write(wal_path(&index), "wal")?;
    fs::write(shm_path(&index), "shm")?;

    let output = atlas_setup_clean(&source, &cache, &index, &["--all", "--yes"])?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["check"], false);
    assert!(
        actual["targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|target| target["status"] == "removed")
    );
    assert!(!source.exists());
    assert!(!cache.exists());
    assert!(!index.exists());
    assert!(!wal_path(&index).exists());
    assert!(!shm_path(&index).exists());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_clean_all_requires_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "clean", "--all"])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)?.contains("requires --yes"));
    Ok(())
}

#[test]
fn setup_clean_all_json_errors_use_standard_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--json", "clean", "--all"])
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
            .contains("requires --yes")
    );
    Ok(())
}

#[test]
fn setup_clean_missing_target_json_errors_use_standard_envelope()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--json", "clean"])
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
            .contains("requires at least one")
    );
    Ok(())
}

#[test]
fn setup_clean_individual_all_targets_require_confirmation()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "clean",
            "--artifact",
            "--embeddings",
            "--source-checkout",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)?.contains("requires --yes"));
    Ok(())
}

#[test]
fn setup_clean_rejects_parent_install_check_without_removing()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("setup-clean-parent-check");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    fs::create_dir_all(&source)?;
    fs::create_dir_all(&cache)?;
    fs::write(&index, "sqlite")?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--source"])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .args(["--json", "--check", "clean", "--artifact"])
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
            .contains("setup install options")
    );
    assert!(index.is_file());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_clean_repo_path_resolution_json_errors_use_standard_envelope()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("setup-clean-repo-path-json-error");
    fs::create_dir_all(&root)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .current_dir(&root)
        .args([
            "setup",
            "--path-mode",
            "repo",
            "--json",
            "clean",
            "--artifact",
            "--check",
        ])
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

    fs::remove_dir_all(root)?;
    Ok(())
}

fn atlas_setup_clean(
    source: &Path,
    cache: &Path,
    index: &Path,
    clean_args: &[&str],
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--source"])
        .arg(source)
        .args(["--embedding-cache-path"])
        .arg(cache)
        .args(["--index"])
        .arg(index)
        .args(["--json", "clean"])
        .args(clean_args)
        .output()?)
}

fn help_output(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(args)
        .arg("--help")
        .output()?;
    assert!(output.status.success());
    Ok(String::from_utf8(output.stdout)?)
}

fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
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

fn wal_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}-wal", path.display()))
}

fn shm_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}-shm", path.display()))
}
