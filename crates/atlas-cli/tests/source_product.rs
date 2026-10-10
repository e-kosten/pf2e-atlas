use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture {
    root: PathBuf,
    index: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scratch/cli-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(root.join("packs/actions")).unwrap();
        fs::create_dir_all(root.join("packs/macros")).unwrap();
        fs::create_dir_all(root.join("static/lang")).unwrap();
        fs::write(root.join("static/lang/en.json"), "{}").unwrap();
        fs::write(root.join("module.json"),r#"{"packs":[{"name":"actions","label":"Actions","type":"Item","path":"packs/actions"},{"name":"macros","label":"Macros","type":"Macro","path":"packs/macros"}]}"#).unwrap();
        fs::write(root.join("packs/actions/heal.json"),r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Treat Wounds","type":"action","system":{"description":{"value":"<h2>Defined Fever</h2><p>A unique explanatory prose phrase. @Check[type:fortitude|dc:23]</p>"},"traits":{"value":["healing"]}}}"#).unwrap();
        fs::write(root.join("packs/macros/tool.json"),r#"{"_id":"bbbbbbbbbbbbbbbb","name":"Treat Wounds","type":"script","command":"console.log(1)"}"#).unwrap();
        let index = root.join("index.sqlite");
        let f = Self { root, index };
        let o = f.command(&["index", "build", "--no-embeddings", "--json"], true);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        f
    }
    fn command(&self, args: &[&str], build: bool) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_atlas"));
        c.args(args)
            .arg(if build { "--output" } else { "--index" })
            .arg(&self.index);
        if build {
            c.arg("--source").arg(&self.root);
        }
        for name in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_PREFIX",
            "ATLAS_LOG",
        ] {
            c.env_remove(name);
        }
        c.output().unwrap()
    }
    fn data(&self, args: &[&str]) -> Value {
        let o = self.command(args, false);
        assert!(
            o.status.success(),
            "{args:?}: {} {}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["status"], "ok");
        v["data"].clone()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
#[test]
fn source_artifact_search_catalog_detail_and_graph_use_the_product_contract() {
    let f = Fixture::new();
    let check = f.data(&["index", "check", "--no-embeddings", "--json"]);
    assert_eq!(check["product_records"], 1);
    let validation = f.data(&["index", "validate", "--no-embeddings", "--json"]);
    assert_eq!(validation["checked_snapshots"], 2);
    let fields = f.data(&["filters", "fields", "--json"]);
    assert!(
        fields["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["path"] == "actor.hp.maximum")
    );
    let values = f.data(&["filters", "values", "--field", "traits", "--json"]);
    assert_eq!(values["values"]["options"][0]["value"], "healing");
    let results = f.data(&[
        "search",
        "Defined Fever",
        "--retrieval",
        "fts",
        "--where",
        "'healing' in traits",
        "--json",
    ]);
    assert_eq!(results["results"].as_array().unwrap().len(), 1);
    let record = &results["results"][0]["record"];
    assert!(record.get("source_path").is_none());
    assert!(record.get("content_hash").is_none());
    assert!(record.get("source_json").is_none());
    let witness = &results["results"][0]["matches"][0];
    assert!(witness["navigation"]["field"].is_string());
    assert_eq!(results["coverage"]["exhaustive"], true);
    let navigation = &witness["navigation"];
    let owners = serde_json::to_string(&navigation["owners"]).unwrap();
    let passage = serde_json::to_string(&navigation["passage"]).unwrap();
    let selected = f.data(&[
        "record",
        "get",
        "actions:aaaaaaaaaaaaaaaa",
        "--owners",
        &owners,
        "--field",
        navigation["field"].as_str().unwrap(),
        "--passage",
        &passage,
        "--json",
    ]);
    assert_eq!(selected["records"][0]["selected"], *navigation);
    let mut stale = navigation["passage"].clone();
    stale["canonical_text_sha256"] = Value::String("0".repeat(64));
    let stale = serde_json::to_string(&stale).unwrap();
    let stale = f.command(
        &[
            "record",
            "get",
            "actions:aaaaaaaaaaaaaaaa",
            "--field",
            navigation["field"].as_str().unwrap(),
            "--passage",
            &stale,
            "--json",
        ],
        false,
    );
    assert!(!stale.status.success());
    let stale: Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(stale["status"], "error");
    let detail = f.data(&["record", "get", "actions:aaaaaaaaaaaaaaaa", "--json"]);
    let detailtext = serde_json::to_string(&detail).unwrap();
    assert!(detailtext.contains("DC 23"));
    assert!(detailtext.contains("data-atlas-interaction"));
    assert!(!detailtext.contains("source_json"));
    let resolved = f.data(&["record", "resolve", "Treat Wounds", "--json"]);
    assert_eq!(resolved["resolutions"][0]["total"], 1);
    for command in ["links", "uses", "remaster", "variants"] {
        f.data(&["graph", command, "actions:aaaaaaaaaaaaaaaa", "--json"]);
    }
    let macroget = f.command(
        &["record", "get", "macros:bbbbbbbbbbbbbbbb", "--json"],
        false,
    );
    assert!(!macroget.status.success());
}
#[test]
fn ambiguous_names_require_keys_and_graph_errors_keep_candidate_details() {
    let f = Fixture::new();
    let source = fs::read_to_string(f.root.join("packs/actions/heal.json")).unwrap();
    fs::write(
        f.root.join("packs/actions/other.json"),
        source.replace("aaaaaaaaaaaaaaaa", "cccccccccccccccc"),
    )
    .unwrap();
    assert!(
        f.command(&["index", "build", "--no-embeddings", "--json"], true)
            .status
            .success()
    );
    let resolve = f.command(&["record", "resolve", "Treat Wounds", "--json"], false);
    assert!(!resolve.status.success());
    let resolve: Value = serde_json::from_slice(&resolve.stdout).unwrap();
    assert_eq!(resolve["data"]["resolutions"][0]["status"], "ambiguous");
    assert_eq!(resolve["data"]["resolutions"][0]["total"], 2);
    let graph = f.command(&["graph", "links", "Treat Wounds", "--json"], false);
    assert!(!graph.status.success());
    let graph: Value = serde_json::from_slice(&graph.stdout).unwrap();
    assert_eq!(graph["error"]["code"], "record_resolution_ambiguous");
    assert_eq!(
        graph["error"]["data"]["candidates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    f.data(&["record", "get", "actions:aaaaaaaaaaaaaaaa", "--json"]);
}
#[test]
fn saved_lists_retain_unresolved_snapshots_across_an_artifact_rebuild() {
    let f = Fixture::new();
    f.data(&[
        "lists", "create", "research", "--name", "Research", "--json",
    ]);
    f.data(&["lists", "add", "research", "Treat Wounds", "--json"]);
    let v = f.data(&["lists", "show", "research", "--json"]);
    assert_eq!(v["items"][0]["status"], "active");
    fs::remove_file(f.root.join("packs/actions/heal.json")).unwrap();
    let o = f.command(&["index", "build", "--no-embeddings", "--json"], true);
    assert!(o.status.success());
    let v = f.data(&["lists", "show", "research", "--json"]);
    assert_eq!(v["items"][0]["status"], "unresolved");
    assert_eq!(v["items"][0]["snapshot"]["title"], "Treat Wounds");
    assert!(v["items"][0]["record"].is_null());
}
#[test]
fn setup_check_and_cleanup_preview_do_not_mutate_existing_artifact() {
    let f = Fixture::new();
    let before = fs::read(&f.index).unwrap();
    let check = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--check",
            "--offline",
            "--no-embeddings",
            "--json",
            "--source",
        ])
        .arg(&f.root)
        .arg("--index")
        .arg(&f.index)
        .arg("--embedding-cache-path")
        .arg(f.root.join("models"))
        .output()
        .unwrap();
    let check: Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(check["status"], "ok");
    assert_eq!(check["data"]["check"], true);
    assert_eq!(fs::read(&f.index).unwrap(), before);
    let clean = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--index"])
        .arg(&f.index)
        .args(["clean", "--artifact", "--check", "--json"])
        .output()
        .unwrap();
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stderr)
    );
    let clean: Value = serde_json::from_slice(&clean.stdout).unwrap();
    assert_eq!(clean["status"], "ok");
    assert_eq!(fs::read(&f.index).unwrap(), before);
}
#[test]
fn unsupported_cel_and_legacy_artifacts_fail_with_rebuild_guidance() {
    let f = Fixture::new();
    let o = f.command(
        &["search", "--where", "actor.hp.maximum + 1 > 2", "--json"],
        false,
    );
    assert!(!o.status.success());
    assert!(
        String::from_utf8(o.stdout)
            .unwrap()
            .contains("invalid_filter")
    );
    let db = rusqlite::Connection::open(&f.index).unwrap();
    db.pragma_update(None, "user_version", 0).unwrap();
    drop(db);
    let o = f.command(&["index", "check", "--no-embeddings", "--json"], false);
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("rebuild")
            || String::from_utf8_lossy(&o.stdout).contains("rebuild")
    );
}
