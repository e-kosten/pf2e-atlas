#![cfg(any(target_os = "macos", windows))]

use std::process::Command;

#[cfg(any(target_os = "macos", feature = "web"))]
#[test]
fn app_selection_is_the_cli_default_and_explicit_paths_bypass_it() {
    use atlas_desktop_protocol::workspace_selection::{Config, Role, SelectionSnapshot};
    use std::{io::Write, process::Stdio};
    let home = tempfile::tempdir().unwrap();
    let command = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_atlas"));
        cmd.env("HOME", home.path())
            .env("LOCALAPPDATA", home.path());
        cmd
    };
    let output = command()
        .args(["desktop-probe", "--workspace-selection", "get"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let state: SelectionSnapshot = serde_json::from_value(envelope["data"].clone()).unwrap();
    let original = state
        .workspaces
        .iter()
        .find(|w| w.role == Role::Original)
        .unwrap()
        .path
        .clone();
    let separate = state
        .workspaces
        .iter()
        .find(|w| w.role == Role::Separate)
        .unwrap()
        .path
        .clone();
    atlas_runtime::initialize_workspace(&original).unwrap();
    atlas_runtime::initialize_workspace(&separate).unwrap();
    let save = |config: &Config| {
        let mut child = command()
            .args(["desktop-probe", "--workspace-selection", "save"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut pipe = child.stdin.take().unwrap();
        serde_json::to_writer(&mut pipe, config).unwrap();
        pipe.write_all(b"\n").unwrap();
        drop(pipe);
        child.wait_with_output().unwrap()
    };
    let mut selection = state.config;
    selection.active = Role::Separate;
    assert!(save(&selection).status.success());
    // A stale desktop cannot redirect subsequent commands back to the original.
    selection.active = Role::Original;
    assert!(!save(&selection).status.success());
    let probe = command().arg("desktop-probe").output().unwrap();
    assert!(probe.status.success());
    let report: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
    assert_eq!(
        report["data"]["workspace"],
        separate.canonicalize().unwrap().to_str().unwrap()
    );
    assert!(original.is_file());

    let config_path = original.with_file_name("workspace-selection.json");
    std::fs::write(&config_path, "invalid selection").unwrap();
    assert!(
        !command()
            .arg("desktop-probe")
            .output()
            .unwrap()
            .status
            .success()
    );
    let explicit = command()
        .args(["desktop-probe", "--index"])
        .arg(&original)
        .output()
        .unwrap();
    assert!(explicit.status.success());
    let report: serde_json::Value = serde_json::from_slice(&explicit.stdout).unwrap();
    assert_eq!(
        report["data"]["workspace"],
        original.canonicalize().unwrap().to_str().unwrap()
    );
    // Repo mode also bypasses the corrupt global selection without changing it.
    let repo = command()
        .args(["setup", "--check", "--json", "--path-mode", "repo"])
        .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&repo.stdout).unwrap();
    assert_eq!(report["status"], "ok", "{report}");
    assert_eq!(
        std::fs::read_to_string(&config_path).unwrap(),
        "invalid selection"
    );
}

#[cfg(any(target_os = "macos", all(windows, feature = "web")))]
#[test]
fn probe_missing_workspace_is_read_only() {
    let home = tempfile::tempdir().unwrap();
    let index = home.path().join("missing").join("workspace.sqlite");
    let result = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["desktop-probe", "--index"])
        .arg(&index)
        .env("HOME", home.path())
        .env("LOCALAPPDATA", home.path())
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["data"]["version"], 1);
    assert_eq!(report["data"]["status"], "needs_setup");
    assert!(!index.parent().unwrap().exists());
}

#[cfg(target_os = "macos")]
#[test]
fn open_rejects_a_non_atlas_url_before_asking_macos_to_open_it() {
    let result = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["open", "https://example.invalid/"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Expected an Atlas v1 link"));
}

#[test]
fn setup_stream_reports_progress_without_changing_json_result() {
    let home = tempfile::tempdir().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--check", "--json", "--desktop-progress"])
        .env("HOME", home.path())
        .env("LOCALAPPDATA", home.path())
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
