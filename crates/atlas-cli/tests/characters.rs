use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    process::{Command, Output, Stdio},
};

struct Fixture {
    directory: tempfile::TempDir,
    index: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        fs::create_dir_all(source.join("static/lang")).unwrap();
        fs::write(source.join("static/lang/en.json"), "{}").unwrap();
        let mut packs = Vec::new();
        for (name, kind, records) in [
            (
                "feats",
                "Item",
                vec![
                    json!({"_id":"counterspell0001","name":"Counterspell","type":"feat","system":{"description":{"value":"First Counterspell rule."}}}),
                    json!({"_id":"counterspell0002","name":"Counterspell","type":"feat","system":{"description":{"value":"Second Counterspell rule."}}}),
                ],
            ),
            (
                "equipment",
                "Item",
                vec![
                    json!({"_id":"rope000000000001","name":"Rope","type":"equipment","system":{"description":{"value":"Useful rope."},"price":{"value":{"gp":1}}}}),
                ],
            ),
            (
                "creatures",
                "Actor",
                vec![
                    json!({"_id":"guard00000000001","name":"Test Guard","type":"npc","system":{"details":{"level":{"value":7}},"attributes":{"hp":{"max":70,"value":70}}}}),
                ],
            ),
        ] {
            fs::create_dir_all(source.join("packs").join(name)).unwrap();
            packs.push(json!({"name":name,"label":name,"type":kind,"system":"pf2e","path":format!("packs/{name}")}));
            for (i, record) in records.iter().enumerate() {
                fs::write(
                    source.join("packs").join(name).join(format!("{i}.json")),
                    record.to_string(),
                )
                .unwrap();
            }
        }
        fs::write(
            source.join("module.json"),
            json!({"id":"pf2e","version":atlas_runtime::desktop_reference_baseline().1,"packs":packs}).to_string(),
        )
        .unwrap();
        let index = directory.path().join("workspace.sqlite");
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(["index", "build", "--no-embeddings", "--json", "--source"])
            .arg(source)
            .arg("--output")
            .arg(&index)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        Self { directory, index }
    }
    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
        command.args(args).arg("--index").arg(&self.index);
        for key in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_PREFIX",
            "ATLAS_LOG",
        ] {
            command.env_remove(key);
        }
        command
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }
    fn data(&self, args: &[&str]) -> Value {
        let mut args = args.to_vec();
        args.push("--json");
        let output = self.run(&args);
        assert!(
            output.status.success(),
            "{args:?}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["status"], "ok");
        result["data"].clone()
    }
    fn export(&self) -> std::path::PathBuf {
        let path = self.directory.path().join("character.json");
        fs::write(
            &path,
            r#" {
          "success":true,"build":{"name":"Mira","level":7,
          "feats":[["Counterspell",null,"Class Feat",2]],"specials":[],"spellCasters":[],
          "class":"Wizard","proficiencies":{"arcana":4},
          "equipment":[["Rope",2,"Not Invested"],["Rope",1,"bag","Invested"]],
          "equipmentContainers":{"bag":{"containerName":"Backpack"}},
          "armor":[{"name":"Clothing","worn":false}],"money":{"gp":50,"sp":null}},
          "custom":{"precise":123456789012345678901234567890.012345678901}
        }
"#,
        )
        .unwrap();
        path
    }
    fn imported(&self) -> (String, std::path::PathBuf, Value) {
        let path = self.export();
        let path_text = path.to_str().unwrap();
        let preview = self.data(&["characters", "preview", path_text]);
        let args = [
            "characters",
            "import",
            path_text,
            "--request-id",
            "import-mira",
            "--expected-source-sha256",
            preview["source_sha256"].as_str().unwrap(),
            "--expected-generation",
            preview["artifact_generation"].as_str().unwrap(),
        ];
        let created = self.data(&args);
        assert_eq!(
            self.data(&args),
            created,
            "identical import retry should replay"
        );
        (
            created["character_key"].as_str().unwrap().into(),
            path,
            preview,
        )
    }
}

#[test]
fn one_character_read_selects_lossless_equipment_and_exports_original_source() {
    let f = Fixture::new();
    let (key, path, _) = f.imported();
    let selected = f.data(&[
        "characters",
        "show",
        "Mira",
        "--section",
        "equipment",
        "--section",
        "money",
    ]);
    assert_eq!(selected["character"]["character_key"], key);
    assert_eq!(selected["selected_sections"], json!(["equipment", "money"]));
    assert_eq!(selected["sections"].as_object().unwrap().len(), 2);
    let gear = &selected["sections"]["equipment"]["fields"]["equipment"]["value"];
    assert_eq!(gear.as_array().unwrap().len(), 2);
    assert_eq!(gear[0]["value"]["quantity"]["value"], 2);
    assert_eq!(gear[1]["value"]["container_id"]["value"], "bag");
    assert_eq!(
        selected["sections"]["money"]["fields"]["money"]["value"]["sp"]["state"],
        "null"
    );
    let full = f.data(&["characters", "show", &key]);
    assert!(full["sections"].get("source").is_none());
    let raw = f.data(&["characters", "show", &key, "--section", "source"]);
    assert_eq!(
        raw["sections"]["source"]["source_json"],
        fs::read_to_string(&path).unwrap()
    );
    let output = f.directory.path().join("original.json");
    let args = [
        "characters",
        "show",
        &key,
        "--section",
        "source",
        "--output",
        output.to_str().unwrap(),
    ];
    f.data(&args);
    assert_eq!(fs::read(&output).unwrap(), fs::read(&path).unwrap());
    assert_eq!(
        f.run(&[&args[..], &["--json"]].concat()).status.code(),
        Some(2)
    );
    assert_eq!(fs::read(&output).unwrap(), fs::read(&path).unwrap());
    let human = f.run(&["characters", "show", &key, "--section", "equipment"]);
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.contains("quantity: 2") && text.contains("Rope") && text.contains("Backpack"));
}

#[test]
fn agent_can_repair_an_ability_and_refresh_without_losing_its_correction() {
    let f = Fixture::new();
    let (key, path, preview) = f.imported();
    let unresolved = f.data(&[
        "characters",
        "show",
        &key,
        "--section",
        "abilities",
        "--needs-review",
    ]);
    let ability = unresolved["sections"]["abilities"]["entries"][0]["ability_key"]
        .as_str()
        .unwrap();
    assert_eq!(
        unresolved["sections"]["abilities"]["entries"][0]["link_state"],
        "ambiguous"
    );
    let resolved = f.run(&["record", "resolve", "Counterspell", "--json"]);
    assert_eq!(resolved.status.code(), Some(1));
    let resolved: Value = serde_json::from_slice(&resolved.stdout).unwrap();
    assert_eq!(
        resolved["data"]["results"][0]["error"]["details"]["total"],
        2
    );
    f.data(&["record", "get", "feats:counterspell0001"]);
    f.data(&[
        "characters",
        "link",
        &key,
        "--ability",
        ability,
        "--record",
        "feats:counterspell0001",
        "--expected-generation",
        preview["artifact_generation"].as_str().unwrap(),
        "--expected-revision",
        "0",
    ]);
    let reviewed = f.data(&[
        "characters",
        "show",
        &key,
        "--section",
        "abilities",
        "--needs-review",
    ]);
    assert_eq!(reviewed["sections"]["abilities"]["shown"], 0);
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, original.replace("\"level\":7", "\"level\":8")).unwrap();
    let refresh = f.data(&[
        "characters",
        "preview",
        path.to_str().unwrap(),
        "--character",
        &key,
    ]);
    assert_eq!(refresh["current"]["character"]["revision"], 1);
    assert_eq!(refresh["changes"]["retained_corrections"], 1);
    let args = [
        "characters",
        "refresh",
        &key,
        path.to_str().unwrap(),
        "--expected-revision",
        "1",
        "--expected-source-sha256",
        refresh["source_sha256"].as_str().unwrap(),
        "--expected-generation",
        refresh["artifact_generation"].as_str().unwrap(),
    ];
    f.data(&args);
    assert_eq!(
        f.run(&[&args[..], &["--json"]].concat()).status.code(),
        Some(2)
    );
    let after = f.data(&["characters", "show", &key, "--section", "abilities"]);
    assert_eq!(after["character"]["level"], 8);
    assert_eq!(
        after["sections"]["abilities"]["entries"][0]["link_state"],
        "manual"
    );
}

#[test]
fn party_context_drives_encounter_and_saved_loot_planning_with_replay_guards() {
    let f = Fixture::new();
    let (key, _, _) = f.imported();
    let manual = f.data(&[
        "characters",
        "create",
        "Bran",
        "--level",
        "7",
        "--request-id",
        "create-bran",
    ]);
    let bran = manual["character_key"].as_str().unwrap();
    let party = f.data(&[
        "parties",
        "create",
        "Actual party",
        "--member",
        &format!("{key}=0"),
        "--member",
        &format!("{bran}=0"),
    ]);
    let party_key = party["party_key"].as_str().unwrap();
    let context = f.data(&[
        "parties",
        "context",
        party_key,
        "--section",
        "equipment",
        "--section",
        "proficiencies",
    ]);
    assert_eq!(context["members"][0]["character"]["character_key"], key);
    assert_eq!(context["members"][1]["source_state"], "no_import");
    let reorder = f.data(&[
        "parties",
        "reorder",
        party_key,
        "--member",
        &format!("{bran}=0"),
        "--member",
        &format!("{key}=0"),
        "--expected-revision",
        "0",
    ]);
    assert_eq!(reorder["members"][0]["character_key"], bran);
    let levelled = f.data(&[
        "parties",
        "set-level",
        party_key,
        "--member",
        &format!("{bran}=0"),
        "--member",
        &format!("{key}=0"),
        "--level",
        "8",
        "--expected-revision",
        "1",
    ]);
    assert_eq!(levelled["members"][1]["character_revision"], 1);
    let encounter = f.data(&["encounters", "create", "Guard scene"]);
    let encounter_key = encounter["encounter"]["encounter_key"].as_str().unwrap();
    let add = [
        "encounters",
        "add-party",
        encounter_key,
        "--party",
        party_key,
        "--expected-party-revision",
        "2",
        "--expected-revision",
        "0",
        "--member",
        &format!("{bran}=1"),
        "--member",
        &format!("{key}=1"),
        "--request-id",
        "scene-party",
    ];
    let first = f.data(&add);
    assert_eq!(f.data(&add), first);
    f.data(&[
        "encounters",
        "add",
        encounter_key,
        "--record",
        "creatures:guard00000000001",
    ]);
    let planned = f.data(&["encounters", "show", encounter_key]);
    assert_eq!(planned["participants"].as_array().unwrap().len(), 3);
    assert_eq!(planned["difficulty"]["pc_count"], 2);
    assert_eq!(planned["difficulty"]["party_level"], 8);
    f.data(&[
        "search",
        "Rope",
        "--retrieval",
        "fts",
        "--kind",
        "equipment",
    ]);
    f.data(&["lists", "create", "Party loot", "--slug", "party-loot"]);
    f.data(&[
        "lists",
        "add",
        "party-loot",
        "--record",
        "equipment:rope000000000001",
        "--note",
        "Extra rope for the current party loadout.",
    ]);
    let loot = f.data(&["lists", "show", "party-loot"]);
    assert_eq!(loot["items"].as_array().unwrap().len(), 1);
    let stale = f.run(&[
        "parties",
        "remove",
        party_key,
        "--character",
        bran,
        "--expected-revision",
        "0",
        "--json",
    ]);
    assert_eq!(stale.status.code(), Some(2));
    assert_eq!(
        f.data(&["parties", "show", party_key])["members"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn invalid_selection_and_changed_source_fail_before_mutation_and_stdin_is_bounded() {
    let f = Fixture::new();
    let path = f.export();
    let preview = f.data(&["characters", "preview", path.to_str().unwrap()]);
    let bad = f.run(&[
        "characters",
        "show",
        "missing",
        "--section",
        "equipment",
        "--needs-review",
        "--json",
    ]);
    assert_eq!(bad.status.code(), Some(2));
    let mut args = vec![
        "characters",
        "import",
        "-",
        "--request-id",
        "stdin-import",
        "--expected-source-sha256",
        preview["source_sha256"].as_str().unwrap(),
        "--expected-generation",
        preview["artifact_generation"].as_str().unwrap(),
        "--json",
    ];
    let mut child = f
        .command(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"changed\":true}")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        f.data(&["characters", "list"])["characters"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    args = vec!["characters", "preview", "-", "--json"];
    let mut child = f
        .command(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child
        .stdin
        .take()
        .unwrap()
        .write_all(&vec![b' '; 512 * 1024 + 1]);
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("512 KiB"));
}
