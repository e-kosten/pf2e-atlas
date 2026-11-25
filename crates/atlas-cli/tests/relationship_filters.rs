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
                "GM's relationship {}-{}",
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
                json!({"_id":"guard00000000000","name":"Fixture Guard","type":"npc","system":{"details":{"level":{"value":2}},"traits":{"value":["humanoid"]}},"items":[{"_id":"owned00000000000","name":"Guard Armor","type":"armor","_stats":{"compendiumSource":"Compendium.pf2e.items.Item.armor00000000000"},"system":{"description":{"value":format!("<p>{armor} {stunned} {armor} {armor} {armor}</p>")}}}]}),
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
        "--detail=preview",
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
    let mut keys = lexical["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["record"]["record_key"].as_str().unwrap())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(keys, ["actors:guard00000000000", "actors:scout00000000000"]);
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
        "--detail=preview",
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
        "--detail=preview",
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
        "--detail=preview",
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
    let mut keys = outgoing["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["record"]["record_key"].as_str().unwrap())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(keys, ["items:armor00000000000", "items:stun000000000000"]);
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

fn assert_armor_destination(occurrence: &Value) {
    assert_eq!(occurrence["destination"]["kind"], "resolved_record");
    assert_eq!(
        occurrence["destination"]["target"]["record_key"],
        "items:armor00000000000"
    );
    for legacy in ["status", "target", "url"] {
        assert!(occurrence.get(legacy).is_none(), "{occurrence}");
    }
}

#[test]
fn graph_navigation_reopens_at_both_detail_levels() {
    let fixture = Fixture::new();
    let scout = "actors:scout00000000000";
    let armor = "items:armor00000000000";
    for detail in ["standard", "full"] {
        let outgoing = fixture.data(&["graph", "links", scout, "--detail", detail, "--json"]);
        let generation = outgoing["artifact_generation"].as_str().unwrap();
        let incoming = fixture.data(&[
            "graph",
            "links",
            armor,
            "--backlinks",
            "8",
            "--detail",
            detail,
            "--json",
        ]);
        let page = fixture.data(&[
            "graph",
            "occurrences",
            scout,
            "--links-to",
            armor,
            "--artifact-generation",
            generation,
            "--detail",
            detail,
            "--json",
        ]);
        for occurrence in [
            &outgoing["outgoing"]["occurrences"][0],
            &incoming["backlinks"]["occurrences"][0],
            &page["occurrences"][0],
        ] {
            for locator in [
                &occurrence["source_navigation"],
                &occurrence["destination"]["target"],
            ] {
                assert_eq!(locator["artifact_generation"], generation);
                let opened = fixture.data(&[
                    "record",
                    "get",
                    "--navigation",
                    &locator.to_string(),
                    "--json",
                ]);
                assert_eq!(
                    opened["results"][0]["value"]["selected"]["record_key"],
                    locator["record_key"]
                );
            }
        }
        let mut retired = page["occurrences"][0]["source_navigation"].clone();
        retired["artifact_generation"] = json!("retired");
        let output = fixture.command(
            &[
                "record",
                "get",
                "--navigation",
                &retired.to_string(),
                "--json",
            ],
            false,
        );
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            response["data"]["results"][0]["error"]["code"],
            "artifact_generation_changed"
        );
    }
}

#[test]
fn occurrence_pages_require_generation_and_relevant_positive_proof() {
    let fixture = Fixture::new();
    let result = fixture.data(&[
        "search",
        "--detail=preview",
        "--links-to",
        "Fixture Armor",
        "--json",
    ]);
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
    assert_armor_destination(&page["occurrences"][0]);

    let scout = "actors:scout00000000000";
    let row = result["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["record"]["record_key"] == scout)
        .unwrap();
    assert_armor_destination(&row["relationship_evidence"][0]["occurrences"][0]);

    let detail = fixture.data(&["record", "get", scout, "--detail", "full", "--json"]);
    assert_armor_destination(&detail["results"][0]["value"]["relationships"][0]);

    let graph = fixture.data(&["graph", "links", scout, "--json"]);
    assert_armor_destination(&graph["outgoing"]["occurrences"][0]);
    let stale = fixture.command(
        &[
            "graph",
            "occurrences",
            "actors:scout00000000000",
            "--links-to",
            "Fixture Armor",
            "--artifact-generation",
            "stale",
            "--links-to",
            "An old name now gone",
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

// Execute the argument vector exactly as returned. Adding the fixture's path here
// would hide a continuation that accidentally selects the user's global workspace.
fn follow(args: &Value) -> Value {
    let args: Vec<_> = args
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    assert_eq!(args[0], "atlas");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(&args[1..])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    response["data"].clone()
}

#[test]
fn followups_reopen_exact_hits_and_continue_seeded_and_multiclause_pages() {
    let f = Fixture::new();
    let first = f.data(&[
        "search",
        "Fixture",
        "--retrieval",
        "fts",
        "--links-to",
        "Fixture Armor",
        "--sort",
        "random",
        "--limit",
        "1",
        "--json",
    ]);
    let second = follow(&first["next_args"]);
    assert_eq!(first["artifact_generation"], second["artifact_generation"]);
    assert_eq!(first["sort"], second["sort"]);
    assert_eq!(second["page"]["number"], 2);
    assert_ne!(
        first["rows"][0]["record"]["record_key"],
        second["rows"][0]["record"]["record_key"]
    );
    assert_eq!(first["result_detail"], "records");
    assert_eq!(second["result_detail"], "records");
    let first_row = &first["rows"][0];
    assert!(first_row.get("open_args").is_none());
    let selected = f.data(&[
        "record",
        "get",
        first_row["record"]["record_key"].as_str().unwrap(),
        "--json",
    ]);
    assert_eq!(selected["results"][0]["status"], "ok");

    for (detail, effective) in [("preview", "compact"), ("full", "evidence")] {
        let first = f.data(&[
            "search", "Fixture", "--detail", detail, "--limit", "1", "--json",
        ]);
        let second = follow(&first["next_args"]);
        assert_eq!(second["result_detail"], effective);
        assert_eq!(second["artifact_generation"], first["artifact_generation"]);
    }
    let guarded = f.data(&[
        "search",
        "--detail=preview",
        "--links-to",
        "Fixture Armor",
        "--links-to",
        "Fixture Stunned",
        "--json",
    ]);
    let row = &guarded["rows"][0];
    let proof = row["relationship_evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["next_args"].is_array())
        .unwrap();
    let more = follow(&proof["next_args"]);
    assert_eq!(more["clause"]["clause_id"], proof["clause_id"]);
    assert_eq!(more["page"]["number"], 2);
    assert_eq!(more["page"]["total"], proof["page"]["total"]);

    let edges = f.data(&["graph", "links", "Fixture Guard", "--json"]);
    assert_eq!(edges["backlinks"]["enabled"], false);
    let provenance = edges["outgoing"]["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "CompendiumProvenance")
        .unwrap();
    assert_eq!(provenance["owner_title"], "Fixture Guard");
    assert_eq!(provenance["source_title"], "Guard Armor");
    assert_eq!(provenance["target_title"], "Fixture Armor");
    let text = f.command(&["graph", "links", "Fixture Guard"], false);
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(
        text.contains("Fixture Guard / Guard Armor → Fixture Armor (compendium source"),
        "{text}"
    );

    let stale = f.command(
        &[
            "search",
            "Fixture",
            "--artifact-generation",
            "stale",
            "--links-to",
            "An old name now gone",
            "--json",
        ],
        false,
    );
    assert_eq!(stale.status.code(), Some(3));
    let response: Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(response["error"]["code"], "artifact_generation_changed");
    assert_eq!(response["error"]["recoverable_action"], "refresh_results");
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
        "--text",
        "hum",
        "--json",
    ]);
    assert_eq!(queried["values"]["options"][0]["value"], "humanoid");
    assert_eq!(
        queried["values"]["options"][0]["distinct_roots"], 2,
        "value-name search must not narrow structured record eligibility"
    );
    let stale = fixture.command(
        &[
            "filters",
            "values",
            "--field",
            "traits",
            "--links-to",
            "Fixture Armor",
            "--artifact-generation",
            "stale",
            "--links-to",
            "An old name now gone",
            "--json",
        ],
        false,
    );
    let stale: Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(
        stale["error"]["code"], "artifact_generation_changed",
        "generation is checked before discovery"
    );
    fixture.data(&[
        "lists",
        "create",
        "Relationship test",
        "--slug",
        "relationship-test",
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
