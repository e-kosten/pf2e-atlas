use super::*;
use model::Assets;
use std::fs;
use std::path::Path;

fn seed(store: &Path, id: &str) -> PathBuf {
    let directory = store.join("versions").join(id);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("atlas"), b"frozen executable").unwrap();
    fs::write(
        directory.join("workspace.sqlite"),
        b"immutable seed contents",
    )
    .unwrap();
    let manifest = Manifest {
        version: 1,
        id: id.into(),
        created_at: "2026-10-19T00:00:00Z".into(),
        build: json!({"sha256":files::hash(&directory.join("atlas")).unwrap()}),
        database: json!({"sha256":files::hash(&directory.join("workspace.sqlite")).unwrap()}),
        assets: Assets::default(),
        captured_from: "source.sqlite".into(),
        validation: json!({}),
    };
    files::replace(&directory.join("manifest.json"), &manifest).unwrap();
    files::replace(
        &store.join("current.json"),
        &Pointer {
            version: 1,
            baseline: id.into(),
        },
    )
    .unwrap();
    directory
}
#[test]
fn existing_copies_remain_independent_and_pinned_after_new_publication() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    let first = seed(&store, "first");
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    assert_eq!(initialize(&a, &store, None).unwrap()["initialized"], true);
    assert_eq!(initialize(&b, &store, None).unwrap()["initialized"], true);
    fs::write(a.join(".atlas/workspace.sqlite"), b"private edits").unwrap();
    seed(&store, "second");
    assert_eq!(initialize(&a, &store, None).unwrap()["initialized"], false);
    assert_eq!(workspace_baseline(&a, &store, None).unwrap().2.id, "first");
    assert_eq!(
        fs::read(b.join(".atlas/workspace.sqlite")).unwrap(),
        fs::read(first.join("workspace.sqlite")).unwrap()
    );
    assert_eq!(
        fs::read(a.join(".atlas/workspace.sqlite")).unwrap(),
        b"private edits"
    );
}
#[test]
fn receipt_only_and_unmanaged_workspaces_are_preserved_without_a_seed() {
    for filename in ["workspace.sqlite", "dev-workspace.json"] {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join(".atlas")).unwrap();
        let existing = temp.path().join(".atlas").join(filename);
        fs::write(&existing, b"do not touch").unwrap();
        assert_eq!(
            initialize(temp.path(), Path::new("missing-store"), None).unwrap()["initialized"],
            false
        );
        assert_eq!(fs::read(&existing).unwrap(), b"do not touch");
    }
}
#[test]
fn tampered_seeds_and_executables_are_rejected_before_installation() {
    for filename in ["workspace.sqlite", "atlas"] {
        let temp = tempfile::tempdir().unwrap();
        let store = temp.path().join("store");
        let directory = seed(&store, "first");
        fs::write(directory.join(filename), b"tampered").unwrap();
        assert!(initialize(temp.path(), &store, None).is_err());
        assert!(!temp.path().join(".atlas/workspace.sqlite").exists());
    }
}
#[test]
fn invalid_identities_and_orphan_sidecars_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    seed(&store, "first");
    for id in ["../first", "/tmp/file", "", ".hidden", "a/b"] {
        assert!(load(&store, Some(id), false).is_err());
    }
    fs::create_dir(temp.path().join(".atlas")).unwrap();
    fs::write(
        temp.path().join(".atlas/workspace.sqlite-wal"),
        b"recover me",
    )
    .unwrap();
    assert!(initialize(temp.path(), &store, None).is_err());
}
#[cfg(unix)]
#[test]
fn shared_directory_and_dangling_database_symlinks_are_never_replaced() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let shared = temp.path().join("shared");
    fs::create_dir(&shared).unwrap();
    symlink(&shared, temp.path().join(".atlas")).unwrap();
    assert!(initialize(temp.path(), Path::new("missing-store"), None).is_err());
    fs::remove_file(temp.path().join(".atlas")).unwrap();
    fs::create_dir(temp.path().join(".atlas")).unwrap();
    let database = temp.path().join(".atlas/workspace.sqlite");
    symlink("absent", &database).unwrap();
    assert_eq!(
        initialize(temp.path(), Path::new("missing-store"), None).unwrap()["initialized"],
        false
    );
    assert!(database.is_symlink());
}
#[test]
fn binary_selection_does_not_use_checkout_builds() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("target/debug")).unwrap();
    fs::write(temp.path().join("target/debug/atlas"), b"candidate").unwrap();
    let mut candidate = Candidate {
        selection: args::Selection { baseline: None },
        binary: None,
        port: 4727,
    };
    assert_eq!(
        binary(temp.path(), Path::new("frozen"), &candidate).unwrap(),
        Path::new("frozen/atlas")
    );
    candidate.binary = Some("target/debug/atlas".into());
    assert_eq!(
        binary(temp.path(), Path::new("frozen"), &candidate).unwrap(),
        temp.path()
            .join("target/debug/atlas")
            .canonicalize()
            .unwrap()
    );
}
#[cfg(unix)]
fn fake_atlas(path: &Path, failure: &str) {
    let script = format!(
        r##"#!/bin/sh
case "$*" in
  *"--version"*) echo 'atlas fixture'; exit 0;;
  *"setup --check"*) echo '{{"status":"ok","data":{{"ready":true,"search_profiles":[{{"profile_id":"one","ready":true}},{{"profile_id":"two","ready":true}}],"paths":{{"embedding_cache":"/tmp/models","source":"/tmp/source"}}}}}}';;
  *"index validate"*) {};;
  *"search healing"*) {};;
  *) echo '{{"status":"error"}}'; exit 1;;
esac
"##,
        if failure == "deep" {
            "echo '{\"status\":\"error\"}'; exit 1"
        } else {
            "echo '{\"status\":\"ok\",\"data\":{}}'"
        },
        if failure == "search" {
            "echo '{\"status\":\"ok\",\"data\":{\"page\":{\"count\":0},\"rows\":[]}}'"
        } else if failure == "retired_search_format" {
            "echo '{\"status\":\"ok\",\"data\":{\"results\":[{}]}}'"
        } else {
            "echo '{\"status\":\"ok\",\"data\":{\"page\":{\"count\":1},\"rows\":[{\"record\":{\"record_key\":\"feats-srd:healing\"},\"matches\":[]}]}}'"
        }
    );
    fs::write(path, script).unwrap();
    files::mode(path, 0o755).unwrap();
}
#[cfg(unix)]
#[test]
fn validation_requires_deep_success_and_nonempty_results_for_every_profile() {
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("atlas");
    for failure in ["deep", "search", "retired_search_format", ""] {
        fake_atlas(&executable, failure);
        let result = checks::validate(
            &executable,
            Path::new("seed"),
            &Assets::default(),
            temp.path(),
        );
        if failure.is_empty() {
            assert_eq!(
                result.unwrap()["searches"],
                json!([
                    {"profile":"one","result_count":1},
                    {"profile":"two","result_count":1}
                ])
            );
        } else {
            assert!(result.is_err());
        }
    }
}
#[test]
fn failed_publication_and_busy_publisher_preserve_default() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    seed(&store, "first");
    let before = fs::read(store.join("current.json")).unwrap();
    let options = args::PublishArgs {
        binary: store.join("versions/first/atlas"),
        index: store.join("versions/first/workspace.sqlite"),
        build_commit: "HEAD".into(),
        build_profile: "dev".into(),
        toolchain: "1.95.0".into(),
        source: None,
        embedding_cache_path: None,
    };
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(store.join("publish.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert!(
        publish::publish(&options, checkout, &store)
            .unwrap_err()
            .to_string()
            .contains("busy")
    );
    drop(lock);
    assert!(publish::publish(&options, checkout, &store).is_err());
    assert_eq!(fs::read(store.join("current.json")).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn launcher_forwards_checkout_and_arguments_to_explicit_retained_tool() {
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("tool with spaces");
    fs::write(&executable, "#!/bin/sh\nprintf '%s\\n' \"$@\"\n").unwrap();
    files::mode(&executable, 0o755).unwrap();
    let launcher =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/development/atlas-dev.sh");
    let output = Command::new("sh")
        .arg(launcher)
        .args(["status", "--checkout", "/a checkout with spaces", "--json"])
        .env("ATLAS_DEV_BINARY", &executable)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "baseline\nstatus\n--checkout\n/a checkout with spaces\n--json\n"
    );
}

#[cfg(unix)]
#[test]
fn status_reports_candidate_failure_without_invoking_migrations() {
    let temp = tempfile::tempdir().unwrap();
    let checkout = temp.path().join("checkout");
    fs::create_dir_all(&checkout).unwrap();
    assert!(
        Command::new("git")
            .arg("init")
            .arg(&checkout)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(&checkout)
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--allow-empty",
                "-m",
                "fixture"
            ])
            .output()
            .unwrap()
            .status
            .success()
    );
    let store = temp.path().join("store");
    seed(&store, "first");
    initialize(&checkout, &store, None).unwrap();
    let executable = temp.path().join("candidate");
    fs::write(&executable, "#!/bin/sh\nprintf '%s\\n' \"$*\" >> calls\necho '{\"status\":\"error\",\"message\":\"schema mismatch\"}'\nexit 1\n").unwrap();
    files::mode(&executable, 0o755).unwrap();
    let candidate = Candidate {
        selection: args::Selection { baseline: None },
        binary: Some(executable),
        port: 4727,
    };
    let report = status(&checkout, &store, &candidate).unwrap();
    assert_eq!(report["compatibility"]["exit_code"], 1);
    let calls = fs::read_to_string(checkout.join("calls")).unwrap();
    assert_eq!(calls.lines().count(), 2);
    assert!(
        calls
            .lines()
            .all(|line| line.starts_with("setup --check --offline")
                || line.starts_with("index check"))
    );
    assert_eq!(
        fs::read(checkout.join(".atlas/workspace.sqlite")).unwrap(),
        b"immutable seed contents"
    );
}
