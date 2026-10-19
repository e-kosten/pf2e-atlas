#![cfg(unix)]

use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn startup_identifies_the_executable_actually_launched_for_each_selection() {
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
    let workspace = checkout.join(".atlas/workspace.sqlite");
    fs::write(&workspace, b"private fixture").unwrap();
    let script = b"#!/bin/sh\ncase \"$1\" in\nsetup) echo '{\"status\":\"ok\",\"data\":{\"ready\":true}}';;\nweb) printf '%s\\n' \"$0\"; exit 7;;\n*) exit 99;;\nesac\n";
    let baseline = version.join("atlas");
    let candidate = checkout.join("candidate with spaces");
    for binary in [&baseline, &candidate] {
        fs::write(binary, script).unwrap();
        fs::set_permissions(binary, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(
        version.join("manifest.json"),
        serde_json::to_vec(&json!({
            "version":1,"id":"seed","created_at":"fixture",
            "build":{"sha256":format!("{:x}",Sha256::digest(script))},
            "database":{},"assets":{},"captured_from":"fixture","validation":{}
        }))
        .unwrap(),
    )
    .unwrap();
    for (explicit, binary, label) in [
        (false, &baseline, "frozen baseline"),
        (true, &candidate, "selected candidate"),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_atlas-dev"));
        command
            .args(["baseline", "web", "--store", "store", "--baseline", "seed"])
            .current_dir(&checkout);
        if explicit {
            command.args(["--binary", "candidate with spaces"]);
        }
        let output = command.output().unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(output.status.code(), Some(7), "{stderr}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{}\n", binary.display())
        );
        let banner = format!(
            "Running: {label}\nBinary: {}\nWorkspace: {}\n",
            binary.display(),
            workspace.display()
        );
        assert!(stderr.contains(&banner), "{stderr}");
        assert_eq!(fs::read(&workspace).unwrap(), b"private fixture");
    }
}
