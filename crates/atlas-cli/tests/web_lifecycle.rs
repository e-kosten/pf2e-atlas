#![cfg(feature = "web")]

use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

struct Launcher(Child);

impl Launcher {
    fn stop(&mut self) {
        self.0.kill().unwrap();
        self.0.wait().unwrap();
    }
}

impl Drop for Launcher {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn launch(home: &std::path::Path) -> (Launcher, mpsc::Receiver<String>) {
    let mut process = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["web", "--port", "0", "--index"])
        .arg(home.join("workspace.sqlite"))
        .env("HOME", home)
        .env("LOCALAPPDATA", home)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = process.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        if let Some(Ok(line)) = BufReader::new(stdout).lines().next() {
            let _ = sender.send(line);
        }
    });
    (Launcher(process), receiver)
}

#[test]
fn simultaneous_launchers_share_a_service_and_exit_releases_ownership() {
    let home = tempfile::tempdir().unwrap();
    let (mut first, first_ready) = launch(home.path());
    let (mut second, second_ready) = launch(home.path());
    let first_url = first_ready.recv_timeout(Duration::from_secs(20)).unwrap();
    let second_url = second_ready.recv_timeout(Duration::from_secs(20)).unwrap();
    assert_eq!(first_url, second_url);
    let url = first_url
        .strip_prefix("Atlas web service listening at ")
        .unwrap();
    let endpoint = format!("{url}/api/compatibility");
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .proxy(None)
        .timeout_global(Some(Duration::from_secs(2)))
        .build()
        .into();

    assert_eq!(agent.get(&endpoint).call().unwrap().status(), 200);
    assert_eq!(
        agent
            .get(&endpoint)
            .header("Origin", "https://example.invalid")
            .call()
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        agent
            .get(&endpoint)
            .header("Host", "example.invalid")
            .call()
            .unwrap()
            .status(),
        403
    );
    first.stop();
    assert_eq!(agent.get(&endpoint).call().unwrap().status(), 200);
    second.stop();

    let deadline = Instant::now() + Duration::from_secs(8);
    while agent.get(&endpoint).call().is_ok() {
        assert!(
            Instant::now() < deadline,
            "backend remained after both launchers exited"
        );
        std::thread::sleep(Duration::from_millis(40));
    }
}

#[test]
fn running_service_and_direct_search_return_the_same_projection() {
    use atlas_search::test_support::{input, record};
    use serde_json::{Value, json};

    let home = tempfile::tempdir().unwrap();
    let index = home.path().join("workspace.sqlite");
    let workspace = atlas_runtime::initialize_workspace(&index).unwrap();
    let source = record(
        "actions",
        "Item",
        json!({
            "_id":"aaaaaaaaaaaaaaaa","name":"Treat Wounds","type":"action",
            "system":{"description":{"value":"<h2>Fever Signs</h2><p>Chills.</p>"}}
        }),
    );
    atlas_index::test_support::publish_input_in(
        &workspace,
        atlas_runtime::WORKSPACE_SCHEMA_VERSION,
        &input(vec![source], false),
    )
    .unwrap();
    let search = |detail, execution| {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(["search", "fever", "--detail", detail, "--json", "--index"])
            .arg(&index)
            .env("HOME", home.path())
            .env("LOCALAPPDATA", home.path())
            .env("ATLAS_LOG", "info")
            .output()
            .unwrap();
        let log = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "{log}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(log.contains(execution), "{log}");
        let data: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(data["data"]["rows"][0]["record"]["title"], "Treat Wounds");
        data
    };
    let details = ["records", "preview", "full"];
    let direct = details.map(|d| search(d, "using direct Atlas search"));
    let (_launcher, ready) = launch(home.path());
    ready.recv_timeout(Duration::from_secs(20)).unwrap();
    for (detail, expected) in details.into_iter().zip(direct) {
        assert_eq!(
            search(detail, "using the running Atlas service for search"),
            expected
        );
    }
}

#[test]
fn desktop_launcher_releases_the_service_when_its_input_pipe_closes() {
    let home = tempfile::tempdir().unwrap();
    let mut process = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["web", "--desktop-launcher", "--port", "0", "--index"])
        .arg(home.path().join("workspace.sqlite"))
        .env("HOME", home.path())
        .env("LOCALAPPDATA", home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = process.stdout.take().unwrap();
    let mut launcher = Launcher(process);
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        if reader.read_line(&mut line).is_ok() {
            let _ = sender.send(line);
        }
        let _ = std::io::copy(&mut reader, &mut std::io::sink());
    });
    let line = receiver.recv_timeout(Duration::from_secs(20)).unwrap();
    let report: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(report["data"]["version"], 1);
    assert!(report["data"]["api_contract"].as_str().is_some());
    let url = report["data"]["endpoint"].as_str().unwrap();
    drop(launcher.0.stdin.take());
    let deadline = Instant::now() + Duration::from_secs(8);
    let status = loop {
        if let Some(status) = launcher.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "desktop launcher did not exit after EOF"
        );
        std::thread::sleep(Duration::from_millis(30));
    };
    assert!(status.success());
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .timeout_global(Some(Duration::from_secs(1)))
        .build()
        .into();
    while agent.get(format!("{url}/api/compatibility")).call().is_ok() {
        assert!(
            Instant::now() < deadline,
            "backend remained after the desktop launcher exited"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
}

#[test]
fn desktop_launcher_exits_if_its_input_closes_before_readiness() {
    let home = tempfile::tempdir().unwrap();
    let index = home.path().join("workspace.sqlite");
    let workspace = atlas_runtime::Workspace::at(&index).unwrap();
    let _guard = workspace.service_guard().unwrap();
    let mut launcher = Launcher(
        Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(["web", "--desktop-launcher", "--port", "0", "--index"])
            .arg(&index)
            .env("HOME", home.path())
            .env("LOCALAPPDATA", home.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    drop(launcher.0.stdin.take());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = launcher.0.try_wait().unwrap() {
            assert!(status.success(), "{status:?}");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "launcher waited for startup after EOF"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
}
