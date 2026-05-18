use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[test]
fn setup_auto_mode_uses_user_install_paths_even_from_repo() -> Result<(), Box<dyn std::error::Error>>
{
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
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["path_mode"], "user");
    assert!(actual["repo_root"].is_null());
    assert_eq!(actual["paths"]["source"], source.display().to_string());
    assert_eq!(actual["paths"]["index"], index.display().to_string());

    fs::remove_dir_all(root)?;
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

    fs::remove_dir_all(root)?;
    Ok(())
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
