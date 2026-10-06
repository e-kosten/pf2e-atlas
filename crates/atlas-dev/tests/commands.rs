use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use atlas_ingest::{BuildArtifactOptions, build_artifact};
use serde_json::Value;

struct Source {
    root: PathBuf,
}

impl Source {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("atlas dev {label} {} {nonce}", std::process::id()));
        fs::create_dir_all(root.join("packs/actions")).unwrap();
        fs::write(
            root.join("module.json"),
            include_str!("fixtures/module.json"),
        )
        .unwrap();
        fs::write(
            root.join("packs/actions/action.json"),
            include_str!("fixtures/action.json"),
        )
        .unwrap();
        Self { root }
    }

    fn artifact(&self) -> PathBuf {
        let index = self.root.join("artifact.sqlite");
        build_artifact(BuildArtifactOptions {
            source_root: self.root.clone(),
            output_path: index.clone(),
            manifest_path: None,
            embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
            embedding_cache_root: None,
            reuse_embeddings: false,
            embedding_batch_size: 32,
        })
        .unwrap();
        index
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn dev() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas-dev"));
    for name in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_PREFIX",
        "ATLAS_PROGRESS",
        "ATLAS_LOG",
    ] {
        command.env_remove(name);
    }
    command
}

fn data(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "ok");
    json["data"].clone()
}

#[test]
fn analyze_preserves_report_without_writing_an_artifact_or_requiring_node() {
    let source = Source::new("analyze");
    let output = dev()
        .env("PATH", "")
        .args(["source", "analyze", "--source"])
        .arg(&source.root)
        .arg("--json")
        .output()
        .unwrap();
    let report = data(&output);
    assert!(output.stderr.is_empty());
    assert_eq!(report["source"]["root"], source.root.display().to_string());
    assert_eq!(
        report["source"]["manifest"],
        source.root.join("module.json").display().to_string()
    );
    let signature = report["source"]["source_signature"].as_str().unwrap();
    assert!(signature.starts_with("foundry-pf2e:sha256:"));
    assert_eq!(signature.len(), "foundry-pf2e:sha256:".len() + 64);
    for field in [
        "pack_count",
        "loaded_source_pack_count",
        "record_count",
        "loaded_source_record_count",
        "default_visible_record_count",
    ] {
        assert_eq!(report[field], 1);
    }
    for field in [
        "generated_record_count",
        "hidden_record_count",
        "skipped_record_count",
    ] {
        assert_eq!(report[field], 0);
    }
    assert_eq!(report["by_kind"]["rule"], 1);
    assert_eq!(report["by_foundry_taxonomy"]["Item|action"], 1);
    assert_eq!(report["by_publication_category"]["unknown"], 1);
    assert_eq!(report["mechanics"]["item_records"], 1);
    assert_eq!(report["text"]["records_with_description"], 1);
    assert_eq!(report["embeddings"]["pending_document_embeddings"], 1);
    assert_eq!(report["relationships"]["reference_edges"], 1);
    assert!(!source.root.join("artifact.sqlite").exists());
}

#[test]
fn analyze_preserves_manifest_override_and_human_and_progress_channels() {
    let source = Source::new("manifest");
    let manifest = source.root.join("alternate manifest.json");
    fs::copy(source.root.join("module.json"), &manifest).unwrap();
    let original_manifest = fs::read(&manifest).unwrap();
    let output = dev()
        .args(["source", "analyze", "--source"])
        .arg(&source.root)
        .arg("--manifest")
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("ok: analyzed 1 records from 1 packs"));
    assert!(text.contains("records: source=1 generated=0 default_visible=1 hidden=0"));
    assert!(text.contains("relationships: references=1 aliases=0 remaster_links=0"));
    assert!(text.contains("dropped inline macros: 0"));
    assert_eq!(fs::read(&manifest).unwrap(), original_manifest);
    let progress = dev()
        .args(["--progress", "always", "source", "analyze", "--source"])
        .arg(&source.root)
        .arg("--manifest")
        .arg(&manifest)
        .arg("--json")
        .output()
        .unwrap();
    let report = data(&progress);
    assert_eq!(report["record_count"], 1);
    assert_eq!(report["source"]["manifest"], manifest.display().to_string());
    assert!(!progress.stderr.is_empty());
    assert!(!source.root.join("artifact.sqlite").exists());
}

#[test]
fn audit_preserves_coverage_and_all_filter_flags() {
    let source = Source::new("audit");
    let manifest = source.root.join("alternate manifest.json");
    fs::copy(source.root.join("module.json"), &manifest).unwrap();
    let output = dev()
        .args(["source", "audit-paths", "--source"])
        .arg(&source.root)
        .arg("--manifest")
        .arg(&manifest)
        .args([
            "--pack-name",
            "actions",
            "--document-type",
            "Item",
            "--record-type",
            "action",
            "--limit",
            "100",
            "--json",
        ])
        .output()
        .unwrap();
    let report = data(&output);
    assert_eq!(report["source_root"], source.root.display().to_string());
    assert_eq!(report["manifest_path"], manifest.display().to_string());
    assert_eq!(report["pack_count"], 1);
    assert_eq!(report["record_count"], 1);
    assert_eq!(report["filters"]["record_type"], "action");
    assert!(report["paths"].as_array().unwrap().iter().any(|path| {
        path["path"] == "$.system.description.value"
            && path["coverage_status"] == "consumed"
            && path["known_consumers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|consumer| consumer == "rich_content")
    }));
    let limited = dev()
        .args(["source", "audit-paths", "--source"])
        .arg(&source.root)
        .args(["--limit", "1", "--json"])
        .output()
        .unwrap();
    assert_eq!(data(&limited)["paths"].as_array().unwrap().len(), 1);
    let filtered = dev()
        .args(["source", "audit-paths", "--source"])
        .arg(&source.root)
        .args(["--min-records", "2", "--json"])
        .output()
        .unwrap();
    assert!(data(&filtered)["paths"].as_array().unwrap().is_empty());
    let human = dev()
        .args(["source", "audit-paths", "--source"])
        .arg(&source.root)
        .args(["--record-type", "action"])
        .output()
        .unwrap();
    assert!(human.status.success());
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.contains("ok: audited 1 records from 1 packs"));
    assert!(text.contains("$.system.description.value"));
    assert!(text.contains("status=consumed consumers=rich_content"));
}

#[test]
fn inspect_preserves_report_and_does_not_change_artifact_bytes() {
    let source = Source::new("inspect");
    let index = source.artifact();
    let before = fs::read(&index).unwrap();
    let mut files_before: Vec<_> = fs::read_dir(&source.root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    files_before.sort();
    let output = dev()
        .env("PATH", "")
        .args(["index", "inspect", "--index"])
        .arg(&index)
        .arg("--json")
        .output()
        .unwrap();
    let report = data(&output);
    assert_eq!(report["records"]["total_records"], 1);
    assert_eq!(report["records"]["default_visible_records"], 1);
    assert_eq!(report["records"]["by_kind"]["rule"], 1);
    assert_eq!(report["records"]["by_publication_category"]["unknown"], 1);
    assert_eq!(report["tables"]["records"], 1);
    assert_eq!(report["tables"]["packs"], 1);
    assert_eq!(report["tables"]["document_embedding_cache"], 0);
    assert_eq!(report["text"]["records_with_description"], 1);
    assert_eq!(report["relationships"]["reference_edges"], 1);
    assert_eq!(report["metrics"]["metric_value_catalog_rows"], 0);
    assert_eq!(fs::read(&index).unwrap(), before);
    let mut files_after: Vec<_> = fs::read_dir(&source.root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    files_after.sort();
    assert_eq!(files_after, files_before);
    let human = dev()
        .args(["index", "inspect", "--index"])
        .arg(&index)
        .output()
        .unwrap();
    assert!(human.status.success());
    assert!(
        String::from_utf8(human.stdout)
            .unwrap()
            .contains("ok: inspected 1 records")
    );
}

#[test]
fn repo_paths_honor_explicit_source_and_index_overrides() {
    let source = Source::new("repo overrides");
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let output = dev()
        .current_dir(repo)
        .args(["source", "analyze", "--path-mode", "repo", "--source"])
        .arg(&source.root)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(
        data(&output)["source"]["root"],
        source.root.display().to_string()
    );
    let index = source.artifact();
    let output = dev()
        .current_dir(repo)
        .args(["index", "inspect", "--path-mode", "repo", "--index"])
        .arg(&index)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(data(&output)["index"], index.display().to_string());
    let outside = dev()
        .current_dir(&source.root)
        .args(["source", "analyze", "--path-mode", "repo", "--source"])
        .arg(&source.root)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(outside.status.code(), Some(2));
    assert!(outside.stdout.is_empty());
    assert!(!outside.stderr.is_empty());
}

#[test]
fn invalid_flags_and_runtime_failures_preserve_exits_and_channels() {
    let invalid = dev()
        .args([
            "source",
            "audit-paths",
            "--min-records",
            "not-a-number",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    let report: Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(report["status"], "error");
    assert_eq!(report["error"]["code"], "invalid_input");
    assert!(invalid.stderr.is_empty());
    let missing = Source::new("missing");
    for args in [
        vec!["source", "analyze", "--source"],
        vec!["index", "inspect", "--index"],
    ] {
        let output = dev()
            .args(args)
            .arg(missing.root.join("missing"))
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn developer_help_has_only_the_selected_commands_and_path_flags() {
    for (args, example) in [
        (vec![], "atlas-dev source analyze"),
        (
            vec!["source", "analyze"],
            "atlas-dev source analyze --source",
        ),
        (
            vec!["source", "audit-paths"],
            "atlas-dev source audit-paths --record-type npc",
        ),
        (vec!["index", "inspect"], "atlas-dev index inspect --json"),
    ] {
        let output = dev().args(&args).arg("--help").output().unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(example));
        if !args.is_empty() {
            assert!(text.contains("--path-mode"));
        }
    }
    for args in [["index", "build"], ["source", "schema"]] {
        assert_eq!(dev().args(args).output().unwrap().status.code(), Some(2));
    }
}
