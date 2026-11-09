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
