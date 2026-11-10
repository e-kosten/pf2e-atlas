#![cfg(target_os = "macos")]

use std::process::Command;

#[test]
fn probe_missing_workspace_is_read_only() {
    let home = tempfile::tempdir().unwrap();
    let index = home.path().join("missing").join("workspace.sqlite");
    let result = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["desktop-probe", "--index"])
        .arg(&index)
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["data"]["version"], 1);
    assert_eq!(report["data"]["status"], "needs_setup");
    assert!(!index.parent().unwrap().exists());
}

#[test]
fn open_rejects_a_non_atlas_url_before_asking_macos_to_open_it() {
    let result = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["open", "https://example.invalid/"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Expected an Atlas v1 record link"));
}

#[test]
fn setup_stream_reports_progress_without_changing_json_result() {
    let home = tempfile::tempdir().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--check", "--json", "--desktop-progress"])
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["status"], "ok");
    assert_eq!(report["data"]["ready"], false);
    let events: Vec<serde_json::Value> = String::from_utf8(result.stderr)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(events.iter().all(|event| event["version"] == 1));
    assert!(events.iter().any(|event| event["phase"] == "check_setup"));
}
