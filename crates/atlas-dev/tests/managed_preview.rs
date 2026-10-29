#![cfg(unix)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

// Re-executed by tiny fake Atlas scripts, so atlas-dev tests need no Node/Python.
#[test]
#[ignore = "fake HTTP application subprocess for managed-preview tests"]
fn fixture_http_server() {
    let port = std::env::var("ATLAS_PREVIEW_FIXTURE_PORT").unwrap();
    let html = std::env::var("ATLAS_PREVIEW_FIXTURE_HTML").unwrap();
    let mode = std::env::var("ATLAS_PREVIEW_FIXTURE_MODE").unwrap_or_default();
    let readiness = match mode.as_str() {
        "missing-profiles" => json!({"status":"ready"}),
        "malformed-profiles" => json!({"status":"ready","search_profiles":{}}),
        "empty-profiles" => json!({"status":"ready","search_profiles":[]}),
        "unusable-profile" => json!({"status":"ready","search_profiles":[{"usable":false}]}),
        _ => json!({"status":"ready","search_profiles":[{"usable":true}]}),
    }
    .to_string();
    let server = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    for stream in server.incoming() {
        let mut stream = stream.unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut first = String::new();
        if BufReader::new(&stream).read_line(&mut first).is_err() {
            continue;
        }
        let body = if first.contains(" /api/readiness ") {
            if mode == "slow-probe" {
                thread::sleep(Duration::from_secs(60));
            }
            &readiness
        } else {
            &html
        };
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
    }
}

struct Fixture {
    _temporary: tempfile::TempDir,
    checkout: PathBuf,
    port: u16,
    owner: Option<Child>,
}
impl Fixture {
    fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let checkout = temporary.path().canonicalize().unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&checkout)
                .status()
                .unwrap()
                .success()
        );
        let version = checkout.join("store/versions/seed");
        fs::create_dir_all(&version).unwrap();
        fs::create_dir(checkout.join(".atlas")).unwrap();
        fs::write(
            checkout.join(".atlas/workspace.sqlite"),
            b"table edits stay here",
        )
        .unwrap();
        let binary = version.join("atlas");
        fs::write(&binary, b"unused baseline binary").unwrap();
        fs::write(version.join("manifest.json"), serde_json::to_vec(&json!({
            "version":1,"id":"seed","created_at":"fixture","build":{"sha256":digest(&fs::read(&binary).unwrap()),"commit":"baseline-source"},
            "database":{},"assets":{},"captured_from":"fixture","validation":{}
        })).unwrap()).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        Self {
            _temporary: temporary,
            checkout,
            port,
            owner: None,
        }
    }
    fn app(&self, name: &str, mode: &str) -> (PathBuf, PathBuf) {
        let binary = self.checkout.join(format!("{name} app"));
        let index = self.checkout.join(format!("{name}.html"));
        let html = format!("<html>{name}</html>");
        fs::write(&index, &html).unwrap();
        let index_check = if mode == "preflight-failure" {
            "echo '{\"status\":\"error\",\"message\":\"requires explicit setup\"}'; exit 2"
        } else {
            "echo '{\"status\":\"ok\",\"data\":{}}'"
        };
        let web = format!(
            "export ATLAS_PREVIEW_FIXTURE_HTML='{html}'\nexport ATLAS_PREVIEW_FIXTURE_MODE='{mode}'\nwhile [ \"$#\" -gt 0 ]; do\n if [ \"$1\" = --port ]; then shift; export ATLAS_PREVIEW_FIXTURE_PORT=$1; fi\n shift\ndone\nprintf '%s %s %s\\n' '{name}' \"$ATLAS_PREVIEW_FIXTURE_PORT\" \"$$\" >> '{}';\nif [ '{mode}' = startup-failure ] && [ \"$ATLAS_PREVIEW_FIXTURE_PORT\" = '{}' ]; then exit 9; fi\nif [ '{mode}' = ledger-failure ]; then echo 'migration ledger mismatch' >&2; exit 8; fi\nexec \"$ATLAS_PREVIEW_TEST_EXECUTABLE\" --exact fixture_http_server --ignored --nocapture",
            self.checkout.join(".atlas/web-starts").display(),
            self.port
        );
        fs::write(&binary,format!("#!/bin/sh\ncase \"$1\" in\n setup) echo 'unexpected maintenance check' >&2; exit 98;;\n index) {index_check};;\n web) {web};;\n *) exit 99;;\nesac\n")).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        (binary, index)
    }
    fn command(&self, action: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_atlas-dev"));
        command
            .args(["baseline", action, "--checkout"])
            .arg(&self.checkout)
            .args(["--store", "store", "--json"])
            .current_dir(&self.checkout)
            .env(
                "ATLAS_PREVIEW_TEST_EXECUTABLE",
                std::env::current_exe().unwrap(),
            );
        command
    }
    fn start(&mut self, binary: &Path, index: &Path) {
        self.owner = Some(
            self.command("web")
                .args([
                    "--managed",
                    "--baseline",
                    "seed",
                    "--port",
                    &self.port.to_string(),
                    "--binary",
                ])
                .arg(binary)
                .arg("--frontend-index")
                .arg(index)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if self.report().is_some_and(|value| value["state"] == "ready") {
                return;
            }
            if let Some(status) = self.owner.as_mut().unwrap().try_wait().unwrap() {
                panic!("Preview exited unexpectedly: {status}");
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("Preview did not become ready");
    }
    fn report(&self) -> Option<Value> {
        fs::read(self.checkout.join(".atlas/dev-preview/status.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    }
    fn refresh(&self, binary: &Path, index: &Path) -> Output {
        self.command("refresh")
            .arg("--binary")
            .arg(binary)
            .arg("--frontend-index")
            .arg(index)
            .output()
            .unwrap()
    }
    fn starts(&self) -> Vec<(String, u16, u32)> {
        fs::read_to_string(self.checkout.join(".atlas/web-starts"))
            .unwrap_or_default()
            .lines()
            .map(|line| {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                (
                    fields[0].into(),
                    fields[1].parse().unwrap(),
                    fields[2].parse().unwrap(),
                )
            })
            .collect()
    }
    fn html(&self) -> String {
        ureq::get(format!("http://127.0.0.1:{}/", self.port))
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(mut owner) = self.owner.take()
            && owner.try_wait().ok().flatten().is_none()
        {
            let _ = self.command("stop").output();
            let _ = owner.wait();
        }
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn refresh_keeps_url_and_table_edits_and_retires_the_previous_runtime_copy() {
    let mut fixture = Fixture::new();
    let (first, index) = fixture.app("first", "ok");
    fixture.start(&first, &index);
    let old = fixture.report().unwrap();
    let old_runtime = PathBuf::from(old["selected_build"]["running_binary"].as_str().unwrap());
    let (second, index) = fixture.app("second", "ok");
    let output = fixture.refresh(&second, &index);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["url"], old["url"]);
    assert_eq!(fixture.html(), "<html>second</html>");
    let starts = fixture.starts();
    assert_eq!(starts.len(), 3, "initial server, cold probe, replacement");
    assert_eq!((starts[0].1, starts[2].1), (fixture.port, fixture.port));
    assert_ne!(starts[1].1, fixture.port);
    assert!(
        !Command::new("kill")
            .args(["-0", &starts[1].2.to_string()])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success(),
        "probe must be reaped before replacement"
    );
    assert_eq!(
        fs::read(fixture.checkout.join(".atlas/workspace.sqlite")).unwrap(),
        b"table edits stay here"
    );
    assert!(!old_runtime.exists());
    assert_eq!(report["baseline_build"]["commit"], "baseline-source");
    assert_eq!(report["selected_build"]["provenance"]["kind"], "unrecorded");
    let stopped = fixture.command("stop").output().unwrap();
    assert!(
        stopped.status.success(),
        "{}",
        String::from_utf8_lossy(&stopped.stderr)
    );
    assert!(fixture.owner.as_mut().unwrap().wait().unwrap().success());
    assert_eq!(fixture.report().unwrap()["state"], "stopped");
    assert!(TcpListener::bind(("127.0.0.1", fixture.port)).is_ok());
}

#[test]
fn failed_preflight_and_bad_receipts_preserve_the_existing_server() {
    let mut fixture = Fixture::new();
    let (first, index) = fixture.app("first", "ok");
    fixture.start(&first, &index);
    let (bad, index) = fixture.app("bad", "preflight-failure");
    assert!(!fixture.refresh(&bad, &index).status.success());
    assert_eq!(fixture.html(), "<html>first</html>");
    let (next, index) = fixture.app("next", "ok");
    let receipt = fixture.checkout.join("bad-receipt.json");
    fs::write(
        &receipt,
        r#"{"version":1,"binary_sha256":"incorrect","frontend_sha256":"incorrect"}"#,
    )
    .unwrap();
    let output = fixture
        .command("refresh")
        .arg("--binary")
        .arg(&next)
        .arg("--frontend-index")
        .arg(&index)
        .arg("--build-provenance")
        .arg(receipt)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fixture.html(), "<html>first</html>");
}

#[test]
fn startup_failure_restores_the_frozen_previous_binary_even_after_a_rebuild() {
    let mut fixture = Fixture::new();
    let (first, index) = fixture.app("first", "ok");
    fixture.start(&first, &index);
    // Warm rebuild may replace the path originally supplied to the supervisor.
    fs::write(&first, b"no longer an executable").unwrap();
    let (bad, index) = fixture.app("bad", "startup-failure");
    let output = fixture.refresh(&bad, &index);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("previous build restored"),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fixture.html(), "<html>first</html>");
    assert_eq!(
        fixture.report().unwrap()["outcome"]["previous_build_restored"],
        true
    );
}

#[test]
fn concurrent_refreshes_are_serialized_and_never_mix_artifacts() {
    let mut fixture = Fixture::new();
    let (first, index) = fixture.app("first", "ok");
    fixture.start(&first, &index);
    let (a, index_a) = fixture.app("a", "ok");
    let (b, index_b) = fixture.app("b", "ok");
    let mut a = fixture
        .command("refresh")
        .arg("--binary")
        .arg(a)
        .arg("--frontend-index")
        .arg(index_a)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut b = fixture
        .command("refresh")
        .arg("--binary")
        .arg(b)
        .arg("--frontend-index")
        .arg(index_b)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert!(a.wait().unwrap().success());
    assert!(b.wait().unwrap().success());
    assert!(["<html>a</html>", "<html>b</html>"].contains(&fixture.html().as_str()));
}

#[test]
fn foreign_listener_and_stale_receipts_never_authorize_process_control() {
    let fixture = Fixture::new();
    let listener = TcpListener::bind(("127.0.0.1", fixture.port)).unwrap();
    let (binary, index) = fixture.app("first", "ok");
    let output = fixture
        .command("web")
        .args([
            "--managed",
            "--baseline",
            "seed",
            "--port",
            &fixture.port.to_string(),
            "--binary",
        ])
        .arg(binary)
        .arg("--frontend-index")
        .arg(index)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("occupied"),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.command("stop").output().unwrap().status.success());
    assert_eq!(listener.local_addr().unwrap().port(), fixture.port);
    assert!(fixture.checkout.join(".atlas/workspace.sqlite").exists());
}

#[test]
fn mismatched_served_ui_is_rejected_and_previous_preview_returns() {
    let mut fixture = Fixture::new();
    let (first, index) = fixture.app("first", "ok");
    fixture.start(&first, &index);
    let (next, index) = fixture.app("next", "ok");
    fs::write(&index, "<html>another build</html>").unwrap();
    let output = fixture.refresh(&next, &index);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Served UI does not match"));
    assert_eq!(fixture.html(), "<html>first</html>");
}

#[test]
fn unchanged_candidate_reuses_child_and_termination_cleans_its_listener() {
    let mut fixture = Fixture::new();
    let (binary, index) = fixture.app("first", "ok");
    fixture.start(&binary, &index);
    let before = fixture.report().unwrap();
    let starts = fixture.starts();
    assert_eq!(starts.len(), 1);
    let receipt = fixture.checkout.join("build.json");
    fs::write(
        &receipt,
        serde_json::to_vec(&json!({
            "version":1,
            "binary_sha256":digest(&fs::read(&binary).unwrap()),
            "frontend_sha256":digest(&fs::read(&index).unwrap()),
            "source":{"commit":"declared-checkpoint"}
        }))
        .unwrap(),
    )
    .unwrap();
    let output = fixture
        .command("refresh")
        .arg("--binary")
        .arg(&binary)
        .arg("--frontend-index")
        .arg(&index)
        .arg("--build-provenance")
        .arg(receipt)
        .output()
        .unwrap();
    assert!(output.status.success());
    let after: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        after["selected_build"]["running_binary"],
        before["selected_build"]["running_binary"]
    );
    assert_eq!(after["outcome"]["refreshed"], false);
    assert_eq!(
        fixture.starts(),
        starts,
        "unchanged refresh must reuse the live owner"
    );
    assert_eq!(
        after["selected_build"]["provenance"]["kind"],
        "build_receipt"
    );
    assert_eq!(
        after["selected_build"]["provenance"]["receipt"]["source"]["commit"],
        "declared-checkpoint"
    );
    let owner = fixture.owner.as_mut().unwrap();
    assert!(
        Command::new("kill")
            .args(["-TERM", &owner.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(owner.wait().unwrap().success());
    assert_eq!(fixture.report().unwrap()["state"], "stopped");
    assert!(TcpListener::bind(("127.0.0.1", fixture.port)).is_ok());
}

#[test]
fn incomplete_profile_preflight_never_stops_or_restarts_the_working_preview() {
    let mut fixture = Fixture::new();
    let (first, index) = fixture.app("first", "ok");
    fixture.start(&first, &index);
    let before = fixture.report().unwrap();
    let log = fixture.checkout.join(".atlas/dev-preview/server.log");
    let original_log = fs::read(&log).unwrap();
    for mode in [
        "missing-profiles",
        "malformed-profiles",
        "empty-profiles",
        "unusable-profile",
        "ledger-failure",
    ] {
        let (binary, index) = fixture.app(mode, mode);
        let output = fixture.refresh(&binary, &index);
        assert!(!output.status.success(), "{mode}");
        assert_eq!(fixture.html(), "<html>first</html>", "{mode}");
        assert_eq!(
            fixture.report().unwrap()["selected_build"]["running_binary"],
            before["selected_build"]["running_binary"],
            "{mode}"
        );
        assert_eq!(
            fs::read(&log).unwrap(),
            original_log,
            "{mode}: a stable-port replacement or rollback must never have started"
        );
    }
}

#[test]
fn capabilities_do_not_require_an_existing_checkout_or_workspace() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
        .args([
            "baseline",
            "capabilities",
            "--checkout",
            "/atlas-nonexistent-checkout",
        ])
        .output()
        .expect("capability command");
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capability JSON");
    assert_eq!(report["version"], 1);
    assert_eq!(report["managed_preview"], true);
    assert_eq!(
        report["workspace_schema_version"],
        atlas_runtime::WORKSPACE_SCHEMA_VERSION
    );
}

#[test]
fn interrupting_a_cold_probe_reaps_probe_and_current_server() {
    let mut fixture = Fixture::new();
    let (first, index) = fixture.app("first", "ok");
    fixture.start(&first, &index);
    let (next, index) = fixture.app("next", "slow-probe");
    let mut request = fixture
        .command("refresh")
        .arg("--binary")
        .arg(next)
        .arg("--frontend-index")
        .arg(index)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while fixture.starts().len() < 2 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    let starts = fixture.starts();
    assert_eq!(starts.len(), 2, "cold probe must have started");
    let owner = fixture.owner.as_mut().unwrap();
    assert!(
        Command::new("kill")
            .args(["-TERM", &owner.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(owner.wait().unwrap().success());
    assert!(!request.wait().unwrap().success());
    for (_, port, pid) in starts {
        assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
        assert!(
            !Command::new("kill")
                .args(["-0", &pid.to_string()])
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success()
        );
    }
}
