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
    let selected_text = f.command(
        &[
            "record",
            "get",
            "actions:aaaaaaaaaaaaaaaa",
            "--field",
            navigation["field"].as_str().unwrap(),
            "--passage",
            &passage,
            "--detail",
            "full",
        ],
        false,
    );
    assert!(selected_text.status.success());
    let selected_text = String::from_utf8(selected_text.stdout).unwrap();
    assert!(selected_text.contains("Defined Fever"));
    assert!(!selected_text.contains("Action cost:"));
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
fn root_terminal_summary_does_not_read_source_bodies() {
    let f = Fixture::new();
    let db = rusqlite::Connection::open(&f.index).unwrap();
    db.execute("UPDATE record_bodies SET snapshot = X'00'", [])
        .unwrap();
    drop(db);
    let summary = f.command(
        &[
            "record",
            "get",
            "actions:aaaaaaaaaaaaaaaa",
            "--detail",
            "summary",
        ],
        false,
    );
    assert!(
        summary.status.success(),
        "{}",
        String::from_utf8_lossy(&summary.stderr)
    );
    let text = String::from_utf8(summary.stdout).unwrap();
    assert!(text.contains("Treat Wounds"));
    assert!(!text.contains("Defined Fever"));
    let detail = f.command(&["record", "get", "actions:aaaaaaaaaaaaaaaa"], false);
    assert!(!detail.status.success());
    let json = f.command(
        &["record", "get", "actions:aaaaaaaaaaaaaaaa", "--json"],
        false,
    );
    assert!(
        !json.status.success(),
        "JSON still requests the selected detail DTO"
    );
}

#[test]
fn creature_and_customized_owned_spell_terminal_and_json_preserve_semantic_facts() {
    let f = Fixture::new();
    fs::create_dir_all(f.root.join("packs/creatures")).unwrap();
    fs::write(f.root.join("module.json"), r#"{"packs":[{"name":"actions","label":"Actions","type":"Item","path":"packs/actions"},{"name":"macros","label":"Macros","type":"Macro","path":"packs/macros"},{"name":"creatures","label":"Creatures","type":"Actor","path":"packs/creatures"}]}"#).unwrap();
    fs::write(f.root.join("packs/creatures/caster.json"), r#"{
      "_id":"dddddddddddddddd","name":"Local Caster","type":"npc",
      "system":{
        "details":{"level":{"value":7},"languages":{"value":["common"]},"publicNotes":"<p>Flavor before mechanics.</p>"},
        "traits":{"value":["humanoid"],"rarity":"uncommon"},
        "attributes":{"ac":{"value":25},"hp":{"max":80,"value":80},"speed":{"value":25},
          "resistances":[{"type":"all-damage","value":10,"exceptions":["force","ghost-touch","vitality"],"doubleVs":["non-magical"]}]},
        "perception":{"mod":12},"saves":{"fortitude":{"value":0},"reflex":{"value":-2},"will":{"value":10}}
      },
      "items":[{
        "_id":"eeeeeeeeeeeeeeee","name":"Local Fireball","type":"spell",
        "system":{
          "level":{"value":2},"time":{"value":"2"},"range":{"value":"120 feet"},
          "traits":{"rarity":"uncommon","traditions":["arcane"],"value":["fire"]},
          "area":{"type":"burst","value":20},"defense":{"save":{"statistic":"reflex","basic":true}},
          "duration":{"value":"1 minute","sustained":false},
          "damage":{"local":{"formula":"2d6+1","type":"fire","kinds":["damage"],"materials":[],"applyMod":false}},
          "heightening":{"type":"interval","interval":1,"damage":{"local":"1d6"}},
          "location":{"value":"ffffffffffffffff"},
          "overlays":{"living":{"name":"Local healing option","overlayType":"override","sort":1,"system":{"time":{"value":"2"},"defense":null,"damage":{"local":{"kinds":["healing"]}},"heightening":{"damage":{"local":"1d8+8"}}}}},
          "description":{"value":"<p>This customized spell uses local damage. @UUID[Compendium.pf2e.actions.Item.aaaaaaaaaaaaaaaa] @Unknown[do not execute]</p>"}
        }
      },{
        "_id":"ffffffffffffffff","name":"Arcane Prepared Spells","type":"spellcastingEntry",
        "system":{"spelldc":{"dc":33,"value":23},"prepared":{"value":"prepared"},"tradition":{"value":"arcane"}}
      },{
        "_id":"hhhhhhhhhhhhhhhh","name":"Local Ability","type":"action",
        "system":{"actionType":{"value":"reaction"},"description":{"value":"<p>Embedded ability prose stays with its caster.</p>"}}
      },{
        "_id":"gggggggggggggggg","name":"River Lore","type":"lore","system":{"mod":{"value":8}}
      }]
    }"#).unwrap();
    let built = f.command(&["index", "build", "--no-embeddings", "--json"], true);
    assert!(
        built.status.success(),
        "{} {}",
        String::from_utf8_lossy(&built.stdout),
        String::from_utf8_lossy(&built.stderr)
    );
    let actor = f.command(&["record", "get", "creatures:dddddddddddddddd"], false);
    assert!(
        actor.status.success(),
        "{}",
        String::from_utf8_lossy(&actor.stderr)
    );
    let actor = String::from_utf8(actor.stdout).unwrap();
    for expected in [
        "Flavor before mechanics.",
        "Embedded ability prose stays with its caster.",
        "Rank 2: Local Fireball",
        "Perception: +12",
        "Fortitude: +0",
        "Reflex: -2",
        "Speed: 25 feet",
        "all-damage 10 (except force, ghost-touch, vitality) (double against non-magical)",
        "Local Fireball",
        "Spell attack: +23",
        "DC: 33",
        "Tradition: arcane",
        "Preparation: prepared",
        "Lore modifier: +8",
    ] {
        assert!(actor.contains(expected), "missing {expected:?} in {actor}");
    }
    assert!(actor.find("Flavor before mechanics.").unwrap() < actor.find("Perception:").unwrap());
    for internal in [
        "atlas record get",
        "Association:",
        "--owners",
        "Authored",
        "Damage [",
    ] {
        assert!(!actor.contains(internal), "{internal}: {actor}");
    }
    let root = f.data(&["record", "get", "creatures:dddddddddddddddd", "--json"]);
    let expanded = f.command(
        &[
            "record",
            "get",
            "creatures:dddddddddddddddd",
            "--detail",
            "full",
        ],
        false,
    );
    assert!(
        expanded.status.success(),
        "{}",
        String::from_utf8_lossy(&expanded.stderr)
    );
    let expanded = String::from_utf8(expanded.stdout).unwrap();
    assert!(expanded.contains("This customized spell uses local damage"));
    assert!(expanded.contains("Range: 120 feet"));
    assert!(expanded.contains("From Local Caster"));
    assert_eq!(
        expanded
            .matches("Embedded ability prose stays with its caster.")
            .count(),
        1
    );
    assert!(!actor.contains("This customized spell uses local damage"));
    let spell = root["records"][0]["presentation"]["body"]["value"]["activities"]["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|activity| activity["title"] == "Local Fireball")
        .unwrap();
    let navigation = &spell["navigation"];
    assert_eq!(navigation["owners"][0]["collection"], "/items");
    assert_eq!(
        navigation["owners"][0]["identity"]["Stable"],
        "eeeeeeeeeeeeeeee"
    );
    let owners = serde_json::to_string(&navigation["owners"]).unwrap();
    let selected = f.command(
        &[
            "record",
            "get",
            "creatures:dddddddddddddddd",
            "--owners",
            &owners,
        ],
        false,
    );
    assert!(
        selected.status.success(),
        "{}",
        String::from_utf8_lossy(&selected.stderr)
    );
    let selected = String::from_utf8(selected.stdout).unwrap();
    for expected in [
        "Local Fireball",
        "From Local Caster",
        "Traditions: arcane",
        "Range: 120 feet",
        "Area: 20 feet burst",
        "2d6+1; fire",
        "Heightening (+1)",
        "Damage increase: 1d6",
        "This customized spell uses local damage",
        "Casting: 2 actions",
        "Changes to base",
        "Defense: cleared (null)",
        "Damage component changes: kinds: healing",
        "Damage increase: 1d8+8",
    ] {
        assert!(
            selected.contains(expected),
            "missing {expected:?} in {selected}"
        );
    }
    assert!(!selected.contains("data-atlas"));
    let json = f.data(&[
        "record",
        "get",
        "creatures:dddddddddddddddd",
        "--owners",
        &owners,
        "--json",
    ]);
    assert_eq!(json["records"][0]["record"]["title"], "Local Caster");
    assert_eq!(
        json["records"][0]["presentation"]["identity"]["title"],
        "Local Fireball"
    );
    assert_eq!(json["records"][0]["presentation"]["body"]["kind"], "spell");
    assert!(json["records"][0].get("surface").is_none());
    assert!(json["records"][0].get("source_json").is_none());
    let entry = root["records"][0]["presentation"]["body"]["value"]["activities"]["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|activity| activity["title"] == "Arcane Prepared Spells")
        .unwrap();
    let entry_owners = serde_json::to_string(&entry["navigation"]["owners"]).unwrap();
    let output = f.command(
        &[
            "record",
            "get",
            "creatures:dddddddddddddddd",
            "--owners",
            &entry_owners,
        ],
        false,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Arcane Prepared Spells",
        "Spell attack: +23",
        "DC: 33",
        "Tradition: arcane",
        "Preparation: prepared",
        "From Local Caster",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in {text}");
    }
}
#[test]
fn source_dump_retains_unpresented_data_and_works_without_the_clone() {
    let f = Fixture::new();
    let path = f.root.join("packs/actions/heal.json");
    let authored = fs::read_to_string(&path).unwrap().replace(
        "\"system\":{",
        "\"futureField\":{\"oneOff\":42},\"system\":{",
    );
    fs::write(&path, &authored).unwrap();
    let built = f.command(&["index", "build", "--no-embeddings", "--json"], true);
    assert!(built.status.success());
    // A source dump uses the checked artifact, even after the original file is gone.
    fs::remove_file(&path).unwrap();
    let source = f.data(&["record", "source", "actions:aaaaaaaaaaaaaaaa", "--json"]);
    assert_eq!(source["key"], "actions:aaaaaaaaaaaaaaaa");
    assert!(
        source["provenance"]["path"]
            .as_str()
            .unwrap()
            .ends_with("heal.json")
    );
    assert_eq!(source["provenance"]["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(source["checked_snapshot"]["version"], 1);
    assert!(source["checked_snapshot"]["source_contract"].is_string());
    let snapshot = serde_json::to_string(&source["checked_snapshot"]).unwrap();
    assert!(snapshot.contains("futureField"));
    assert!(snapshot.contains("oneOff"));
    assert!(snapshot.contains("@Check[type:fortitude|dc:23]"));
    assert!(!snapshot.contains("data-atlas-interaction"));
    let reference = f.data(&["record", "get", "actions:aaaaaaaaaaaaaaaa", "--json"]);
    assert!(
        !serde_json::to_string(&reference)
            .unwrap()
            .contains("futureField")
    );
    let human = f.command(&["record", "source", "actions:aaaaaaaaaaaaaaaa"], false);
    assert!(human.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&human.stdout).unwrap(),
        source
    );
    for key in ["actions:absent", "macros:bbbbbbbbbbbbbbbb", "not-a-key"] {
        let output = f.command(&["record", "source", key, "--json"], false);
        assert!(!output.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap()["status"],
            "error"
        );
    }
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

#[test]
#[ignore = "requires pinned local Base/Gemma assets; bounded normal CLI setup/search/progress"]
fn actual_profile_setup_preserves_manifest_defaults_and_reports_cache_heavy_progress() {
    use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig};
    let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|p| p.join(".cache/hf-models"))
        .find(|p| {
            EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, p)
                .model_dir()
                .join("onnx/model.onnx_data")
                .is_file()
        })
        .unwrap();
    let f = Fixture::new();
    let isolated_root = f.root.join("gemma-only-cache");
    let gemma = EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, &isolated_root);
    let original = EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, &cache);
    for file in atlas_embedding::required_embedding_model_cache_files(&gemma) {
        fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
        fs::hard_link(original.model_dir().join(file.source_path), file.local_path).unwrap();
    }
    let baseline = atlas_embedding::baseline_tokenizer_cache_file(&isolated_root);
    fs::create_dir_all(baseline.local_path.parent().unwrap()).unwrap();
    fs::copy(
        atlas_embedding::baseline_tokenizer_cache_file(&cache).local_path,
        baseline.local_path,
    )
    .unwrap();
    assert!(
        !EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeBaseEnV15, &isolated_root)
            .model_dir()
            .join("onnx/model.onnx")
            .exists()
    );
    let setup = |model_cache: &std::path::Path, profiles: &[&str], mode: &str| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
        command
            .args(["--progress", mode, "setup", "--offline", "--json"])
            .arg("--source")
            .arg(&f.root)
            .arg("--index")
            .arg(&f.index)
            .arg("--embedding-cache-path")
            .arg(model_cache);
        for profile in profiles {
            command.args(["--search-profile", profile]);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let json: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(json["status"], "ok");
        (
            json["data"].clone(),
            String::from_utf8(output.stderr).unwrap(),
        )
    };
    let (first, progress) = setup(&isolated_root, &["embeddinggemma"], "always");
    assert_eq!(first["search_profiles"].as_array().unwrap().len(), 1);
    assert_eq!(first["search_profiles"][0]["profile_id"], "embeddinggemma");
    assert!(first["build"]["inferred_inputs"].as_u64().unwrap() > 0);
    eprintln!("Gemma-only normal setup: {}", first["build"]["profiles"]);
    assert!(progress.contains("EmbeddingGemma [embeddinggemma]"));
    assert!(progress.contains("unique inputs") && progress.contains("occurrences"));
    assert!(
        progress.lines().count() < 100,
        "tiny build progress must be bounded"
    );
    let cache_text = isolated_root.to_str().unwrap();
    let result = f.data(&[
        "search",
        "Defined Fever",
        "--retrieval",
        "vector",
        "--embedding-cache-path",
        cache_text,
        "--json",
    ]);
    assert_eq!(result["search_profile"], "embeddinggemma");
    eprintln!("Gemma-only omitted search selection resolved embeddinggemma");
    let missing = f.command(
        &[
            "search",
            "Defined Fever",
            "--retrieval",
            "vector",
            "--search-profile",
            "bge-base",
            "--embedding-cache-path",
            cache_text,
            "--json",
        ],
        false,
    );
    assert!(
        !missing.status.success(),
        "explicit absent profile cannot fall back"
    );
    assert!(String::from_utf8_lossy(&missing.stdout).contains("bge-base"));
    let (preserved, _) = setup(&isolated_root, &[], "always");
    assert_eq!(
        preserved["search_profiles"][0]["profile_id"],
        "embeddinggemma"
    );
    assert!(preserved["build"].is_null());
    let silent = f.command(
        &["--progress", "never", "index", "validate", "--json"],
        false,
    );
    assert!(silent.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&silent.stdout).unwrap()["status"],
        "ok"
    );
    let silent = String::from_utf8(silent.stderr).unwrap();
    assert!(
        !silent.contains("INFO"),
        "disabled progress must stay off stderr"
    );
    let (both, progress) = setup(&cache, &["bge-base", "embeddinggemma"], "always");
    assert_eq!(both["search_profiles"].as_array().unwrap().len(), 2);
    let reused = both["build"]["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["profile_id"] == "embeddinggemma")
        .unwrap();
    assert_eq!(reused["inferred_inputs"], 0);
    assert!(reused["reused_inputs"].as_u64().unwrap() > 0);
    assert!(progress.contains("0/0") && progress.contains("reused unique inputs"));
    eprintln!(
        "Both-profile setup with Gemma zero-inference reuse: {}",
        both["build"]["profiles"]
    );
    let source = f.root.join("packs/actions/heal.json");
    let before = fs::read_to_string(&source).unwrap();
    fs::write(
        source,
        before.replace(
            "unique explanatory prose phrase",
            "changed source prose with additional mechanics",
        ),
    )
    .unwrap();
    let (updated, _) = setup(&cache, &[], "always");
    assert_eq!(updated["search_profiles"].as_array().unwrap().len(), 2);
    assert_eq!(updated["build"]["profiles"].as_array().unwrap().len(), 2);
    assert!(
        updated["build"]["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["inferred_inputs"].as_u64().unwrap() > 0)
    );
    eprintln!(
        "Omitted-profile source update preserved both: {}",
        updated["build"]["profiles"]
    );
    let implicit = f.data(&[
        "search",
        "additional mechanics",
        "--retrieval",
        "vector",
        "--embedding-cache-path",
        cache.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(implicit["search_profile"], "bge-base");
}
