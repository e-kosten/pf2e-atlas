//! Exercise the published CLI against a checked, lexical-only reference artifact.
use serde_json::{Value, json};
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
                "relationship-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        for directory in ["packs/items", "packs/actors", "static/lang"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        fs::write(root.join("module.json"),r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"},{"name":"actors","label":"Actors","type":"Actor","path":"packs/actors"}]}"#).unwrap();
        fs::write(root.join("static/lang/en.json"), "{}").unwrap();
        let armor = "@UUID[Compendium.pf2e.items.Item.armor00000000000]";
        let stunned = "@UUID[Compendium.pf2e.items.Item.stun000000000000]";
        let records = [
            (
                "items/armor",
                json!({"_id":"armor00000000000","name":"Fixture Armor","type":"armor"}),
            ),
            (
                "items/stunned",
                json!({"_id":"stun000000000000","name":"Fixture Stunned","type":"condition"}),
            ),
            (
                "items/duplicate-a",
                json!({"_id":"duplicate0000001","name":"Duplicate Target","type":"action"}),
            ),
            (
                "items/duplicate-b",
                json!({"_id":"duplicate0000002","name":"Duplicate Target","type":"action"}),
            ),
            (
                "actors/guard",
                json!({"_id":"guard00000000000","name":"Fixture Guard","type":"npc","system":{"details":{"level":{"value":2}},"traits":{"value":["humanoid"]}},"items":[{"_id":"owned00000000000","name":"Guard Armor","type":"armor","_stats":{"compendiumSource":"Compendium.pf2e.items.Item.armor00000000000"},"system":{"description":{"value":format!("<p>{armor} {stunned}</p>")}}}]}),
            ),
            (
                "actors/scout",
                json!({"_id":"scout00000000000","name":"Fixture Scout","type":"npc","system":{"details":{"level":{"value":1},"publicNotes":format!("<p>{armor} {armor}</p>")},"traits":{"value":["humanoid"]}}}),
            ),
            (
                "actors/sage",
                json!({"_id":"sage000000000000","name":"Fixture Sage","type":"npc","system":{"details":{"level":{"value":3},"publicNotes":format!("<p>{stunned}</p>")},"traits":{"value":["humanoid"]}}}),
            ),
            (
                "actors/unlinked",
                json!({"_id":"none000000000000","name":"Fixture Unlinked","type":"npc","system":{"details":{"level":{"value":4}},"traits":{"value":["humanoid"]}}}),
            ),
        ];
        for (path, record) in records {
            fs::write(
                root.join(format!("packs/{path}.json")),
                serde_json::to_vec(&record).unwrap(),
            )
            .unwrap();
        }
        let fixture = Self {
            index: root.join("workspace.sqlite"),
            root,
        };
        let output = fixture.command(&["index", "build", "--no-embeddings", "--json"], true);
        assert!(
            output.status.success(),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fixture
    }
    fn command(&self, args: &[&str], build: bool) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
        command
            .args(args)
            .arg(if build { "--output" } else { "--index" })
            .arg(&self.index);
        if build {
            command.arg("--source").arg(&self.root);
        }
        for name in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_PREFIX",
            "ATLAS_LOG",
        ] {
            command.env_remove(name);
        }
        command.output().unwrap()
    }
    fn data(&self, args: &[&str]) -> Value {
        let output = self.command(args, false);
        assert!(
            output.status.success(),
            "{args:?}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["status"], "ok");
        response["data"].clone()
    }
    fn names(&self, expression: &str) -> Vec<String> {
        self.data(&[
            "search", "--kind", "creature", "--where", expression, "--json",
        ])["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["record"]["title"].as_str().unwrap().to_owned())
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn boolean_links_share_root_scope_with_ordinary_fields_and_preserve_occurrence_evidence() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.names(r#"links_to("items:armor00000000000")"#),
        ["Fixture Guard", "Fixture Scout"]
    );
    assert_eq!(
        fixture.names(r#"links_to("items:armor00000000000", "main_entry")"#),
        ["Fixture Scout"]
    );
    assert_eq!(
        fixture
            .names(r#"links_to("items:armor00000000000") && links_to("items:stun000000000000")"#),
        ["Fixture Guard"]
    );
    assert_eq!(
        fixture
            .names(r#"links_to("items:armor00000000000") || links_to("items:stun000000000000")"#),
        ["Fixture Guard", "Fixture Sage", "Fixture Scout"]
    );
    assert_eq!(
        fixture.names(r#"!links_to("items:armor00000000000")"#),
        ["Fixture Sage", "Fixture Unlinked"]
    );
    let lexical = fixture.data(&[
        "search",
        "Fixture",
        "--retrieval",
        "fts",
        "--kind",
        "creature",
        "--links-to",
        "Fixture Armor",
        "--json",
    ]);
    assert_eq!(lexical["page"]["total"], 2);
    assert_eq!(lexical["coverage"]["exhaustive"], true);
    assert!(
        lexical["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| !row["relationship_evidence"].as_array().unwrap().is_empty())
    );
    let alternatives = fixture.data(&[
        "search",
        "--kind",
        "creature",
        "--where",
        r#"links_to("items:armor00000000000") || links_to("items:stun000000000000")"#,
        "--json",
    ]);
    let sage = alternatives["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["record"]["title"] == "Fixture Sage")
        .unwrap();
    assert_eq!(
        sage["relationship_evidence"].as_array().unwrap().len(),
        1,
        "an unmatched OR branch has no link evidence"
    );
    let negatives = fixture.data(&[
        "search",
        "--kind",
        "creature",
        "--where",
        r#"!links_to("items:armor00000000000")"#,
        "--json",
    ]);
    assert!(
        negatives["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["relationship_evidence"].as_array().unwrap().is_empty())
    );
    let result = fixture.data(&[
        "search",
        "--kind",
        "creature",
        "--links-to",
        "Fixture Armor",
        "--limit",
        "1",
        "--page",
        "2",
        "--json",
    ]);
    assert_eq!(result["page"]["total"], 2);
    assert_eq!(result["rows"][0]["record"]["title"], "Fixture Scout");
    assert_eq!(
        result["rows"][0]["relationship_evidence"][0]["page"]["total"],
        2
    );
    assert!(result["relationship_clauses"][0]["clause_id"].is_string());
    assert!(
        result.get("results").is_none(),
        "direct shared page shape has no legacy summaries alias"
    );
}

#[test]
fn strict_named_flags_resolve_outside_candidate_filters_and_fail_on_ambiguity() {
    let fixture = Fixture::new();
    let result = fixture.data(&[
        "search",
        "--kind",
        "creature",
        "--links-to",
        "Fixture Armor",
        "--links-to",
        "Fixture Stunned",
        "--json",
    ]);
    assert_eq!(result["rows"][0]["record"]["title"], "Fixture Guard");
    let main = fixture.data(&["search", "--links-to-main-entry", "Fixture Armor", "--json"]);
    assert_eq!(main["page"]["total"], 1);
    let outgoing = fixture.data(&["search", "--linked-from", "Fixture Guard", "--json"]);
    assert_eq!(outgoing["page"]["total"], 2);
    let outgoing_main = fixture.data(&[
        "search",
        "--linked-from-main-entry",
        "Fixture Guard",
        "--json",
    ]);
    assert_eq!(outgoing_main["page"]["total"], 0);
    let output = fixture.command(
        &[
            "search",
            "--kind",
            "creature",
            "--links-to",
            "Duplicate Target",
            "--json",
        ],
        false,
    );
    assert_eq!(output.status.code(), Some(1));
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], "record_resolution_ambiguous");
    assert_eq!(
        error["error"]["details"]["candidates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn similar_and_record_resolution_use_the_same_link_filter_and_resolution_errors() {
    let fixture = Fixture::new();
    let resolved = fixture.data(&[
        "record",
        "resolve",
        "Fixture Guard",
        "--links-to",
        "Fixture Armor",
        "--json",
    ]);
    assert_eq!(resolved["results"][0]["value"]["total"], 1);
    let output = fixture.command(
        &[
            "similar",
            "Fixture Guard",
            "--links-to",
            "Duplicate Target",
            "--json",
        ],
        false,
    );
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        error["error"]["code"], "record_resolution_ambiguous",
        "link resolution fails before Similar vector retrieval"
    );
    let output = fixture.command(
        &[
            "similar",
            "Fixture Guard",
            "--where",
            "missing.field == 1",
            "--json",
        ],
        false,
    );
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], "filter_invalid");
}

#[test]
fn exact_embedded_cel_selection_retains_owner_scope_and_checks_fingerprint() {
    let fixture = Fixture::new();
    let exact = r#"linked_from({"record_key":"actors:guard00000000000","embedded_path":[{"collection":"/items","identity":{"Stable":"owned00000000000"}}]}, "main_entry")"#;
    let result = fixture.data(&["search", "--where", exact, "--json"]);
    assert_eq!(result["page"]["total"], 2);
    assert_eq!(result["relationship_clauses"][0]["title"], "Guard Armor");
    let stale = r#"linked_from({"record_key":"actors:guard00000000000","embedded_path":[{"collection":"/items","identity":{"SnapshotLocal":{"index":0}}}],"source_fingerprint":"stale"})"#;
    let output = fixture.command(&["search", "--where", stale, "--json"], false);
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], "invalid_request");
}

#[test]
fn occurrence_pages_require_generation_and_relevant_positive_proof() {
    let fixture = Fixture::new();
    let result = fixture.data(&["search", "--links-to", "Fixture Armor", "--json"]);
    let generation = result["artifact_generation"].as_str().unwrap();
    let page = fixture.data(&[
        "graph",
        "occurrences",
        "actors:scout00000000000",
        "--links-to",
        "Fixture Armor",
        "--artifact-generation",
        generation,
        "--page",
        "2",
        "--limit",
        "1",
        "--json",
    ]);
    assert_eq!(page["page"]["total"], 2);
    assert_eq!(page["occurrences"].as_array().unwrap().len(), 1);
    assert_eq!(page["occurrences"][0]["source_title"], "Fixture Scout");
    let stale = fixture.command(
        &[
            "graph",
            "occurrences",
            "actors:scout00000000000",
            "--links-to",
            "Fixture Armor",
            "--artifact-generation",
            "stale",
            "--json",
        ],
        false,
    );
    let stale: Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(stale["error"]["code"], "artifact_generation_changed");
    let negative = fixture.command(
        &[
            "graph",
            "occurrences",
            "actors:sage000000000000",
            "--where",
            r#"!links_to("items:armor00000000000")"#,
            "--artifact-generation",
            generation,
            "--json",
        ],
        false,
    );
    let negative: Value = serde_json::from_slice(&negative.stdout).unwrap();
    assert_eq!(negative["error"]["code"], "filter_invalid");
}

#[test]
fn contextual_discovery_retains_link_constraints_and_saved_membership() {
    let fixture = Fixture::new();
    let context = fixture.data(&[
        "filters",
        "values",
        "--field",
        "traits",
        "--kind",
        "creature",
        "--links-to",
        "Fixture Armor",
        "--json",
    ]);
    let humanoid = context["values"]["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["value"] == "humanoid")
        .unwrap();
    assert_eq!(humanoid["distinct_roots"], 2);
    let queried = fixture.data(&[
        "filters",
        "values",
        "--field",
        "traits",
        "--kind",
        "creature",
        "--links-to",
        "Fixture Armor",
        "--query",
        "Fixture Scout",
        "--retrieval",
        "fts",
        "--text",
        "hum",
        "--json",
    ]);
    assert_eq!(queried["values"]["options"][0]["value"], "humanoid");
    assert_eq!(
        queried["values"]["options"][0]["distinct_roots"], 1,
        "record query and value text have separate purposes"
    );
    let stale = fixture.command(
        &[
            "filters",
            "values",
            "--field",
            "traits",
            "--links-to",
            "Fixture Armor",
            "--query",
            "would require inference",
            "--retrieval",
            "vector",
            "--artifact-generation",
            "stale",
            "--json",
        ],
        false,
    );
    let stale: Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(
        stale["error"]["code"], "artifact_generation_changed",
        "generation fails before query inference"
    );
    fixture.data(&[
        "lists",
        "create",
        "relationship-test",
        "--name",
        "Relationship test",
        "--record",
        "Fixture Scout",
        "--record",
        "Fixture Sage",
        "--json",
    ]);
    let context = fixture.data(&[
        "filters",
        "values",
        "--field",
        "traits",
        "--kind",
        "creature",
        "--links-to",
        "Fixture Armor",
        "--list",
        "relationship-test",
        "--json",
    ]);
    let humanoid = context["values"]["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["value"] == "humanoid")
        .unwrap();
    assert_eq!(humanoid["distinct_roots"], 1);
}

#[test]
fn canonical_print_filter_is_artifact_free_and_named_flags_do_not_open_workspace() {
    let missing = std::env::temp_dir().join(format!(
        "atlas-unopened-links-{}.sqlite",
        std::process::id()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--links-to",
            "items:armor00000000000",
            "--where",
            r#"!links_to("items:stun000000000000")"#,
            "--print-filter",
            "--json",
            "--index",
        ])
        .arg(&missing)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["data"]["predicate"]["kind"], "all_of");
    assert!(!missing.exists());
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--links-to",
            "Fixture Armor",
            "--print-filter",
            "--json",
            "--index",
        ])
        .arg(&missing)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("atlas record resolve")
    );
    assert!(!missing.exists());
}
