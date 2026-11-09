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
        let workspace = atlas_runtime::initialize_workspace(&index).unwrap();
        build_artifact(
            &workspace,
            atlas_runtime::WORKSPACE_SCHEMA_VERSION,
            BuildArtifactOptions {
                source_root: self.0.clone(),
                output_path: index.clone(),
                manifest_path: None,
                locale: "en".into(),
                search_profiles: vec![],
                reuse_passage_layouts: false,
                embedding_batch_size: 16,
            },
        )
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
fn repo_mode_requires_a_checkout_while_global_default_and_source_override_work_without_node() {
    let f = Fixture::new();
    // Outside a checkout, explicit repo mode must not silently become global.
    let invoke = |flags: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
            .current_dir(std::env::temp_dir())
            .args(["source", "load", "--json"])
            .args(flags)
            .arg("--source")
            .arg(&f.0)
            .env("PATH", "")
            .output()
            .unwrap()
    };
    for flags in [vec![], vec!["--path-mode", "global"]] {
        let value = data(invoke(&flags));
        assert_eq!(value["counts"]["retained_documents"], 1);
    }
    let repo = invoke(&["--path-mode", "repo"]);
    assert_eq!(repo.status.code(), Some(2));
    let envelope: Value = serde_json::from_slice(&repo.stdout).unwrap();
    assert_eq!(envelope["error"]["code"], "command_failed");
    assert!(
        envelope["error"]["message"]
            .as_str()
            .unwrap()
            .contains("checkout")
    );

    let repo = Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "source",
            "load",
            "--path-mode",
            "repo",
            "--json",
            "--source",
        ])
        .arg(&f.0)
        .output()
        .unwrap();
    assert_eq!(data(repo)["counts"]["retained_documents"], 1);
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
    assert_eq!(v["key"], "actions:aaaaaaaaaaaaaaaa");
    assert_eq!(v["provenance"]["path"], "packs/actions/action.json");
    assert_eq!(v["checked_snapshot"]["version"], 1);
    assert!(
        serde_json::to_string(&v["checked_snapshot"])
            .unwrap()
            .contains("Treat Wounds")
    );
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
    let original: Value = serde_json::from_str(include_str!("fixtures/action.json")).unwrap();
    assert_eq!(v["original_source"], original);
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

#[test]
fn family_audit_enumerates_equipment_and_proposes_grades_without_external_tools() {
    let fixture = Fixture::new();
    let equipment = fixture.0.join("packs/equipment");
    fs::create_dir(&equipment).unwrap();
    let manifest = fixture.0.join("module.json");
    let mut module: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    module["packs"].as_array_mut().unwrap().push(serde_json::json!({
        "name": "equipment-srd", "label": "Equipment", "type": "Item", "path": "packs/equipment",
    }));
    fs::write(manifest, serde_json::to_vec(&module).unwrap()).unwrap();

    let write_item = |id: String, name: String, level| {
        let item = serde_json::json!({
            "_id": id,
            "name": name,
            "type": "equipment",
            "system": {
                "level": {"value": level},
                "publication": {"title": "Fixture source", "remaster": true, "license": "ORC"},
                "description": {"value": "<p>Fixture equipment.</p>"}
            }
        });
        fs::write(
            equipment.join(format!("{id}.json")),
            serde_json::to_vec(&item).unwrap(),
        )
        .unwrap();
    };
    for number in 0..100 {
        write_item(format!("{number:016}"), format!("Filler item {number}"), 1);
    }
    write_item("zzzzzzzzzzzzzzz1".into(), "Tonic (Lesser)".into(), 1);
    write_item("zzzzzzzzzzzzzzz2".into(), "Tonic (Greater)".into(), 5);

    let index = fixture.artifact();
    let original_database = fs::read(&index).unwrap();
    let invoke = |flags: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
            .args(["families", "audit", "--json", "--index"])
            .arg(&index)
            .args(flags)
            .env("PATH", "")
            .output()
            .unwrap()
    };
    let audit = data(invoke(&[]));
    let records = audit["records"].as_array().unwrap();
    assert_eq!(records.len(), 102);
    assert_eq!(records[0]["key"], "equipment-srd:0000000000000000");
    assert_eq!(records[100]["key"], "equipment-srd:zzzzzzzzzzzzzzz1");
    assert_eq!(records[101]["key"], "equipment-srd:zzzzzzzzzzzzzzz2");
    assert_eq!(records[100]["outcome"]["verified"], false);

    let proposal = data(invoke(&["--propose-grades"]));
    assert_eq!(proposal["proposals"].as_array().unwrap().len(), 1);
    let tonic = &proposal["proposals"][0];
    assert_eq!(tonic["family"], "Tonic");
    assert_eq!(tonic["members"][0]["grade"], "Lesser");
    assert_eq!(tonic["members"][1]["grade"], "Greater");
    assert_eq!(tonic["members"][1]["level"], 5);
    assert_eq!(fs::read(index).unwrap(), original_database);
}

#[test]
fn family_audit_help_and_missing_workspace_use_developer_cli_conventions() {
    let help = Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
        .args(["families", "audit", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    for flag in ["--index", "--path-mode", "--propose-grades", "--json"] {
        assert!(help.contains(flag));
    }

    let fixture = Fixture::new();
    let missing = fixture.0.join("missing.sqlite");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-dev"))
        .args(["families", "audit", "--json", "--index"])
        .arg(&missing)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["status"], "error");
    assert_eq!(response["error"]["code"], "command_failed");
    assert!(!missing.exists());
}
