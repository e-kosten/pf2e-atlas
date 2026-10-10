use atlas_ingest::{BuildArtifactOptions, build_artifact};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scratch/dev-cli-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(path.join("packs/actions")).unwrap();
        fs::create_dir_all(path.join("static/lang")).unwrap();
        fs::write(path.join("static/lang/en.json"), "{}").unwrap();
        fs::write(
            path.join("module.json"),
            include_str!("fixtures/module.json"),
        )
        .unwrap();
        fs::write(
            path.join("packs/actions/action.json"),
            include_str!("fixtures/action.json"),
        )
        .unwrap();
        Self(path)
    }
    fn artifact(&self) -> PathBuf {
        let index = self.0.join("artifact.sqlite");
        build_artifact(BuildArtifactOptions {
            source_root: self.0.clone(),
            output_path: index.clone(),
            manifest_path: None,
            locale: "en".into(),
            embedding: None,
            reuse_embeddings: false,
            embedding_batch_size: 16,
        })
        .unwrap();
        index
    }
    fn source(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
            .args(args)
            .arg("--source")
            .arg(&self.0)
            .arg("--json")
            .env("PATH", "")
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn data(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let v: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["status"], "ok");
    v["data"].clone()
}
#[test]
fn rust_source_commands_need_no_node_and_report_discovery_without_model_owner_receipts() {
    let f = Fixture::new();
    let load = data(f.source(&["source", "load"]));
    assert_eq!(load["counts"]["retained_documents"], 1);
    let analyze = data(f.source(&["source", "analyze"]));
    assert_eq!(analyze["source"]["retained_documents"], 1);
    assert!(analyze["selected_sections"].as_u64().unwrap() > 0);
    assert!(analyze.get("metrics").is_none());
    let audit = data(f.source(&["source", "audit-paths", "--limit", "5"]));
    assert!(!audit["paths"].as_array().unwrap().is_empty());
    assert!(audit["paths"][0].get("known_consumers").is_none());
    assert!(audit["paths"][0].get("coverage_status").is_none());
    assert!(!f.0.join("artifact.sqlite").exists());
}
#[test]
fn developer_inspection_decodes_checked_snapshot_and_only_reads_verified_original_on_request() {
    let f = Fixture::new();
    let index = f.artifact();
    let inspect = data(
        Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
            .args(["index", "inspect", "--index"])
            .arg(&index)
            .arg("--json")
            .output()
            .unwrap(),
    );
    assert_eq!(inspect["statistics"]["records"], 1);
    let key = "actions:aaaaaaaaaaaaaaaa";
    let record = Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
        .args(["index", "record", key, "--index"])
        .arg(&index)
        .arg("--json")
        .output()
        .unwrap();
    let v = data(record);
    assert!(v["checked_snapshot"].is_object());
    assert!(v["original_source"].is_null());
    let v = data(
        Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
            .args(["index", "record", key, "--index"])
            .arg(&index)
            .arg("--source")
            .arg(&f.0)
            .args(["--original", "--json"])
            .output()
            .unwrap(),
    );
    assert!(v["original_source"].is_object());
    fs::write(f.0.join("packs/actions/action.json"), "{}").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
        .args(["index", "record", key, "--index"])
        .arg(&index)
        .arg("--source")
        .arg(&f.0)
        .args(["--original", "--json"])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(
        String::from_utf8(o.stdout)
            .unwrap()
            .contains("hash mismatch")
    );
}
