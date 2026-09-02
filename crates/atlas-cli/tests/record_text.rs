use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

mod support;

use support::path::temp_source_root;

const SOURCE_ROOT_ENV: &str = "PF2E_ATLAS_B5_SOURCE_ROOT";
const UPDATE_ENV: &str = "PF2E_ATLAS_UPDATE_RECORD_TEXT_GOLDENS";
const REMEDIATION_BASE: &str = "d595966a11502f6eca0d4580a500e003619f1613";
const CLARIFICATION_NOTE_PATH: &str = "/Users/ekosten/.ao/data/handoffs/pathfinder-2e-foundry-mcp/cli-presentation-audit/2026-09-02-stage-h-b5-cli-product-decision-clarification-note.md";
const CLARIFICATION_NOTE_SHA256: &str =
    "91ab3f81f6716e3c4a0b8a921baf508b78d564e7e6453ea1ebee1d6dc77ca5d2";
const ROUTE_PATH: &str = "/Users/ekosten/.ao/data/worktrees/pathfinder-2e-foundry-mcp/pathfinder-2e-foundry-mcp-60/scratch/plans/2026-09-02-stage-h-pre-b6-creature-cli-routing-final-addendum.md";
const ROUTE_SHA256: &str = "c91d442157c712401f52fe582e657cb7e0f6e73ba5f80153d380916c6a3a519a";
const AUDIT_PATH: &str = "/Users/ekosten/.ao/data/handoffs/pathfinder-2e-foundry-mcp/cli-presentation-audit/2026-09-02-non-json-creature-cli-presentation-audit-final.md";
const AUDIT_SHA256: &str = "94298824b37d96f4e6ce48705b93cf0838d2be4b07575f84f2d3aa28c07b6182";
const DETAILS: [&str; 5] = ["summary", "preview", "description", "standard", "full"];
const WIDTHS: [usize; 3] = [40, 80, 120];

#[test]
fn creature_golden_manifest_is_complete_and_current() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/record_text/creature");
    let manifest_path = root
        .parent()
        .ok_or("golden root needs a parent")?
        .join("manifest.sha256");
    let manifest = fs::read_to_string(&manifest_path)?;
    let mut listed = BTreeSet::new();
    for line in manifest.lines() {
        let (expected, relative) = line
            .split_once("  ")
            .ok_or_else(|| format!("malformed manifest row: {line}"))?;
        let relative = relative
            .strip_prefix("./")
            .ok_or_else(|| format!("manifest path must be relative: {relative}"))?;
        let bytes = fs::read(root.join(relative))?;
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if actual != expected {
            return Err(format!("stale golden manifest row for {relative}").into());
        }
        if !listed.insert(relative.to_string()) {
            return Err(format!("duplicate golden manifest row for {relative}").into());
        }
    }

    let actual = CREATURES
        .iter()
        .flat_map(|creature| {
            DETAILS.into_iter().flat_map(move |detail| {
                WIDTHS
                    .into_iter()
                    .map(move |width| format!("{}/{detail}-{width}.txt", creature.slug))
            })
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(listed, actual);
    assert_eq!(listed.len(), 135);
    Ok(())
}

#[test]
fn snapshot_evidence_manifest_is_complete_and_honest() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/record_text");
    let evidence: Value = serde_json::from_slice(&fs::read(root.join("snapshot-evidence.json"))?)?;
    assert_eq!(evidence["remediation_base"], REMEDIATION_BASE);
    assert_eq!(
        evidence["base_tree"],
        "c215416437d52ad0ad3bf85a8affa391c8f1d2c9"
    );
    assert_eq!(evidence["settings"]["matrix_cells"], 135);
    assert_eq!(
        evidence["controlling_clarification"]["path"],
        CLARIFICATION_NOTE_PATH
    );
    assert_eq!(
        evidence["controlling_clarification"]["sha256"],
        CLARIFICATION_NOTE_SHA256
    );
    assert_eq!(evidence["controlling_clarification"]["mode"], "0444");
    assert_eq!(evidence["controlling_clarification"]["bytes"], 7242);
    assert_eq!(
        evidence["controlling_clarification"]["file_type"],
        "regular"
    );
    assert_eq!(evidence["controlling_route"]["path"], ROUTE_PATH);
    assert_eq!(evidence["controlling_route"]["sha256"], ROUTE_SHA256);
    assert_eq!(evidence["durable_audit"]["path"], AUDIT_PATH);
    assert_eq!(evidence["durable_audit"]["sha256"], AUDIT_SHA256);
    assert_eq!(
        evidence["bounded_artifact"]["artifact_contract"],
        atlas_index::ARTIFACT_CONTRACT_VERSION
    );
    assert_eq!(
        evidence["bounded_artifact"]["artifact_schema"],
        atlas_index::ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(
        evidence["bounded_artifact"]["manifest_contract"],
        atlas_index::ARTIFACT_MANIFEST_VERSION
    );
    assert_eq!(
        evidence["verified_edition_lookups"]
            .as_array()
            .ok_or("edition evidence must be an array")?
            .len(),
        9
    );
    assert!(
        evidence["verified_edition_lookups"]
            .as_array()
            .is_some_and(|rows| rows
                .iter()
                .all(|row| row["counterpart_lookup"] == "verified"))
    );
    let captures = evidence["captures"]
        .as_array()
        .ok_or("captures must be an array")?;
    assert_eq!(captures.len(), 138);
    for capture in captures {
        assert_eq!(capture["exit_status"], 0);
        assert_eq!(capture["stderr_bytes"], 0);
        assert_eq!(
            capture["stderr_sha256"],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        retained_capture_matches(&root, capture)?;
    }
    assert!(captures.iter().any(|capture| {
        capture["stdout_path"] == "authentic-unmodeled/areelu-vorlesh-provenance.json"
    }));
    assert!(captures.iter().any(|capture| {
        capture["stdout_path"] == "authentic-unmodeled/areelu-vorlesh-full-80.txt"
    }));

    let first = captures.first().ok_or("at least one retained capture")?;
    let mut bad_bytes = first.clone();
    bad_bytes["stdout_bytes"] =
        Value::from(first["stdout_bytes"].as_u64().ok_or("stdout byte count")? + 1);
    assert!(retained_capture_matches(&root, &bad_bytes).is_err());
    let mut bad_hash = first.clone();
    bad_hash["stdout_sha256"] = Value::String("0".repeat(64));
    assert!(retained_capture_matches(&root, &bad_hash).is_err());
    assert!(evidence.get("candidate_commit").is_none());
    Ok(())
}

fn retained_capture_matches(root: &Path, capture: &Value) -> Result<(), String> {
    let relative = capture["stdout_path"]
        .as_str()
        .ok_or_else(|| "retained capture needs stdout_path".to_string())?;
    let bytes = fs::read(root.join(relative))
        .map_err(|error| format!("failed to read retained capture {relative}: {error}"))?;
    let expected_bytes = capture["stdout_bytes"]
        .as_u64()
        .ok_or_else(|| format!("retained capture {relative} needs stdout_bytes"))?;
    if bytes.len() as u64 != expected_bytes {
        return Err(format!("retained capture {relative} byte count differs"));
    }
    let expected_hash = capture["stdout_sha256"]
        .as_str()
        .ok_or_else(|| format!("retained capture {relative} needs stdout_sha256"))?;
    if sha256_bytes(&bytes) != expected_hash {
        return Err(format!("retained capture {relative} hash differs"));
    }
    Ok(())
}

struct CreatureFixture {
    slug: &'static str,
    name: &'static str,
    key: &'static str,
    source_path: &'static str,
    source_sha256: &'static str,
}

#[derive(Debug, Serialize)]
struct CaptureEvidence {
    command: String,
    width: Option<usize>,
    profile: Option<String>,
    color: &'static str,
    progress: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout_path: Option<String>,
    stdout_bytes: usize,
    stdout_sha256: String,
    stderr_bytes: usize,
    stderr_sha256: String,
    exit_status: i32,
}

struct CapturedOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: i32,
}

const CREATURES: [CreatureFixture; 9] = [
    CreatureFixture {
        slug: "night-hag",
        name: "Night Hag",
        key: "pathfinder-bestiary:WQy7HBUcgDLsfVJd",
        source_path: "packs/pf2e/pathfinder-bestiary/night-hag.json",
        source_sha256: "ba2a7da4c23b78a4add8ccdc7976c0e8d6a971d21208df58a33f9bd21df76940",
    },
    CreatureFixture {
        slug: "giant-rat",
        name: "Giant Rat",
        key: "pathfinder-monster-core:iIJPJcDT8wlJ8z5M",
        source_path: "packs/pf2e/pathfinder-monster-core/giant-rat.json",
        source_sha256: "aa9cc319293195f87c7c59732915f7820ccb0ac617fa52f0c04d252a1a6c817c",
    },
    CreatureFixture {
        slug: "calcifda",
        name: "Calcifda",
        key: "quest-for-the-frozen-flame-bestiary:kxO7gXbpcmY1pi7p",
        source_path: "packs/pf2e/quest-for-the-frozen-flame-bestiary/book-3-burning-tundra/calcifda.json",
        source_sha256: "89a287cdd849ce347b1e8a1f01515c0cf1c45caa53a55c3406352cb88dbd4b76",
    },
    CreatureFixture {
        slug: "magical-forge",
        name: "Magical Forge",
        key: "triumph-of-the-tusk-bestiary:EBpwhiVtWKqi8M3n",
        source_path: "packs/pf2e/triumph-of-the-tusk-bestiary/book-2-hoof-cinder-and-storm/magical-forge.json",
        source_sha256: "96a76d3c5da6fbe3373c3865dc3f10a4d0f43d79f5a15ffcb0323061638252be",
    },
    CreatureFixture {
        slug: "balor",
        name: "Balor",
        key: "pathfinder-bestiary:9vNYtJZiseCEf4wt",
        source_path: "packs/pf2e/pathfinder-bestiary/balor.json",
        source_sha256: "f2b1ef8e87eda3665c2416f56f632bc718f77a943dcdb39202a7a60ab0e1b88d",
    },
    CreatureFixture {
        slug: "gray-master",
        name: "Gray Master",
        key: "curtain-call-bestiary:1eX4Csnv3psAsfLf",
        source_path: "packs/pf2e/curtain-call-bestiary/book-3-bring-the-house-down/gray-master.json",
        source_sha256: "8ccd7b926140b52caa66db0294a44f1aa002e3f9005a42dcd6043f86fddd1289",
    },
    CreatureFixture {
        slug: "beluthus",
        name: "Beluthus",
        key: "abomination-vaults-bestiary:3ry9WSvMMXHUe3kE",
        source_path: "packs/pf2e/abomination-vaults-bestiary/abomination-vaults-hardcover-compilation/beluthus.json",
        source_sha256: "47c674ade1ba92d231bd1fdb45fcd06b6ed5648af2ebb2a6960ac91cc9848afa",
    },
    CreatureFixture {
        slug: "air-mephit",
        name: "Air Mephit",
        key: "pathfinder-bestiary:KDRlxdIUADWHI6Vr",
        source_path: "packs/pf2e/pathfinder-bestiary/air-mephit.json",
        source_sha256: "a5e628ad9f52baac8cce14fafe2dda543d1c3455797f373160c50358d75478ee",
    },
    CreatureFixture {
        slug: "air-scamp",
        name: "Air Scamp",
        key: "pathfinder-monster-core:MSm1im7lZA5i82rz",
        source_path: "packs/pf2e/pathfinder-monster-core/air-scamp.json",
        source_sha256: "f2c11588000587cd78ba86a2d8e3d8fac209c282da487fe08ec15f5c9d3c63a8",
    },
];

const REMASTER_JOURNAL_PATH: &str = "packs/pf2e/journals/remaster-changes.json";
const REMASTER_JOURNAL_SHA256: &str =
    "838700b9242191e8ea0221bb895eff1726d9705a586415e59343fc552bee1b61";
const AUTHENTIC_UNMODELED: CreatureFixture = CreatureFixture {
    slug: "areelu-vorlesh",
    name: "Areelu Vorlesh",
    key: "revenge-of-the-runelords-bestiary:igeNfVBZBcIkCzcZ",
    source_path: "packs/pf2e/revenge-of-the-runelords-bestiary/book-3-into-the-apocalypse-archive/areelu-vorlesh.json",
    source_sha256: "483e9d78ed24093f4c2025cd7d5579394a396d93712b6e6e5131005f180973ca",
};

#[test]
fn source_backed_creature_terminal_matrix() -> Result<(), Box<dyn std::error::Error>> {
    let Some(source_root) = source_root() else {
        eprintln!("skipping source-backed matrix; set {SOURCE_ROOT_ENV} to the PF2e checkout");
        return Ok(());
    };
    verify_source_identity(&source_root)?;
    let root = temp_source_root("record-text-matrix");
    prepare_bounded_source(&source_root, &root)?;
    let artifact = root.join("artifact.sqlite");
    let build_capture = build_bounded_artifact(&root, &artifact)?;

    let update = std::env::var_os(UPDATE_ENV).is_some();
    let mut captures = Vec::new();
    for creature in &CREATURES {
        let mut normalized_by_detail = Vec::new();
        for detail in DETAILS {
            let mut normalized_by_width = Vec::new();
            for width in WIDTHS {
                let captured = capture_record(&artifact, creature.key, detail, width)?;
                let output = String::from_utf8(captured.stdout.clone())?;
                if detail != "summary" {
                    assert_eq!(
                        output.lines().next(),
                        Some(creature.name),
                        "{detail} width {width} header for {}",
                        creature.name
                    );
                }
                assert_golden(creature.slug, detail, width, &output, update)?;
                captures.push(capture_evidence(
                    &captured,
                    format!(
                        "atlas --progress never record get {} --detail {detail} --index $BOUNDED_ARTIFACT",
                        creature.key
                    ),
                    Some(width),
                    Some(detail),
                    Some(format!("creature/{}/{detail}-{width}.txt", creature.slug)),
                ));
                if detail != "summary" {
                    let overlong = output
                        .lines()
                        .filter(|line| {
                            line.len() > width && !has_indented_unbreakable_token(line, width)
                        })
                        .collect::<Vec<_>>();
                    assert!(
                        overlong.is_empty(),
                        "wrappable line exceeded width {width} for {} {detail}: {overlong:?}",
                        creature.name
                    );
                    assert_spell_damage_hierarchy(&output, creature, detail, width)?;
                    if width == 120 {
                        assert_no_dangling_compound_punctuation(&output, creature, detail)?;
                    }
                }
                normalized_by_width.push(normalize_layout(&output));
            }
            assert!(
                normalized_by_width
                    .windows(2)
                    .all(|pair| pair[0] == pair[1]),
                "width changed selected content for {} {detail}",
                creature.name
            );
            normalized_by_detail.push((detail, normalized_by_width.remove(0)));
        }
        assert_detail_contract(creature, &normalized_by_detail)?;
    }

    update_matrix_manifest(update)?;
    let authentic_captures = assert_authentic_unmodeled(&artifact, update)?;
    captures.extend(authentic_captures);
    assert_snapshot_evidence(&root, &artifact, &build_capture, &captures, update)?;

    fs::remove_dir_all(root)?;
    Ok(())
}

fn has_indented_unbreakable_token(line: &str, width: usize) -> bool {
    let indent = line.len() - line.trim_start().len();
    line.split_whitespace()
        .any(|token| indent + token.len() > width)
}

fn assert_spell_damage_hierarchy(
    output: &str,
    creature: &CreatureFixture,
    detail: &str,
    width: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut in_spellcasting = false;
    for line in output.lines() {
        if line == "Spellcasting" {
            in_spellcasting = true;
            continue;
        }
        if in_spellcasting && !line.is_empty() && !line.starts_with(' ') {
            in_spellcasting = false;
        }
        if in_spellcasting && line.trim_start().starts_with("Damage:") {
            let indent = line.len() - line.trim_start().len();
            if indent < 8 {
                return Err(format!(
                    "spell damage lost hierarchy for {} {detail} width {width}: {line:?}",
                    creature.name
                )
                .into());
            }
        }
    }
    Ok(())
}

fn assert_no_dangling_compound_punctuation(
    output: &str,
    creature: &CreatureFixture,
    detail: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for line in output.lines().filter(|line| line.starts_with("  ")) {
        let trimmed = line.trim_end();
        if trimmed.contains(": ;")
            || trimmed.contains("; ;")
            || (trimmed.contains(':') && trimmed.ends_with(';'))
        {
            return Err(format!(
                "dangling compound punctuation for {} {detail}: {line:?}",
                creature.name
            )
            .into());
        }
    }
    Ok(())
}

fn source_root() -> Option<PathBuf> {
    std::env::var_os(SOURCE_ROOT_ENV)
        .map(PathBuf::from)
        .or_else(|| {
            let candidate = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pf2e");
            candidate.is_dir().then_some(candidate)
        })
}

fn verify_source_identity(source_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for creature in &CREATURES {
        verify_file_hash(
            &source_root.join(creature.source_path),
            creature.source_sha256,
        )?;
    }
    verify_file_hash(
        &source_root.join(REMASTER_JOURNAL_PATH),
        REMASTER_JOURNAL_SHA256,
    )?;
    verify_file_hash(
        &source_root.join(AUTHENTIC_UNMODELED.source_path),
        AUTHENTIC_UNMODELED.source_sha256,
    )
}

fn verify_file_hash(path: &Path, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        return Err(format!(
            "source identity mismatch for {}: expected {expected}, got {actual}",
            path.display()
        )
        .into());
    }
    Ok(())
}

fn prepare_bounded_source(
    source_root: &Path,
    target: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(target)?;
    let manifest: Value = serde_json::from_slice(&fs::read(source_root.join("system.pf2e.json"))?)?;
    let selected_packs = manifest["packs"]
        .as_array()
        .ok_or("PF2e manifest packs must be an array")?
        .iter()
        .filter(|pack| {
            pack["name"].as_str().is_some_and(|name| {
                [
                    "abomination-vaults-bestiary",
                    "curtain-call-bestiary",
                    "journals",
                    "pathfinder-bestiary",
                    "pathfinder-monster-core",
                    "quest-for-the-frozen-flame-bestiary",
                    "revenge-of-the-runelords-bestiary",
                    "triumph-of-the-tusk-bestiary",
                ]
                .contains(&name)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if selected_packs.len() != 8 {
        return Err(format!("expected 8 bounded packs, found {}", selected_packs.len()).into());
    }
    fs::write(
        target.join("module.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"packs": selected_packs}))?,
    )?;
    for source_path in CREATURES
        .iter()
        .map(|creature| creature.source_path)
        .chain(std::iter::once(AUTHENTIC_UNMODELED.source_path))
        .chain(std::iter::once(REMASTER_JOURNAL_PATH))
    {
        let relative = source_path
            .strip_prefix("packs/pf2e/")
            .ok_or("bounded source path must begin with packs/pf2e")?;
        let destination = target.join("packs").join(relative);
        fs::create_dir_all(destination.parent().ok_or("source file needs a parent")?)?;
        fs::copy(source_root.join(source_path), destination)?;
    }
    Ok(())
}

fn build_bounded_artifact(
    source_root: &Path,
    artifact: &Path,
) -> Result<CaptureEvidence, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["--progress", "never", "index", "build", "--source"])
        .arg(source_root)
        .arg("--output")
        .arg(artifact)
        .args(["--no-embeddings", "--json"])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "bounded artifact build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let captured = CapturedOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        status: output.status.code().unwrap_or(-1),
    };
    Ok(capture_evidence(
        &captured,
        "atlas --progress never index build --source $BOUNDED_SOURCE --output $BOUNDED_ARTIFACT --no-embeddings --json".into(),
        None,
        None,
        None,
    ))
}

fn capture_record(
    artifact: &Path,
    key: &str,
    detail: &str,
    width: usize,
) -> Result<CapturedOutput, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "--progress",
            "never",
            "record",
            "get",
            key,
            "--detail",
            detail,
        ])
        .arg("--index")
        .arg(artifact)
        .env("COLUMNS", width.to_string())
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "record capture failed for {key} {detail} width {width}: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    if !output.stderr.is_empty() {
        return Err(format!(
            "record capture wrote stderr for {key} {detail} width {width}: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(CapturedOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        status: output.status.code().unwrap_or(-1),
    })
}

fn capture_evidence(
    output: &CapturedOutput,
    command: String,
    width: Option<usize>,
    profile: Option<&str>,
    stdout_path: Option<String>,
) -> CaptureEvidence {
    CaptureEvidence {
        command,
        width,
        profile: profile.map(str::to_string),
        color: "NO_COLOR=1",
        progress: "never",
        stdout_path,
        stdout_bytes: output.stdout.len(),
        stdout_sha256: sha256_bytes(&output.stdout),
        stderr_bytes: output.stderr.len(),
        stderr_sha256: sha256_bytes(&output.stderr),
        exit_status: output.status,
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_golden(
    slug: &str,
    detail: &str,
    width: usize,
    actual: &str,
    update: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/record_text/creature")
        .join(slug)
        .join(format!("{detail}-{width}.txt"));
    if update {
        fs::create_dir_all(path.parent().ok_or("golden path needs a parent")?)?;
        fs::write(path, actual)?;
        return Ok(());
    }
    let expected = fs::read_to_string(&path)?;
    if actual != expected {
        return Err(format!("record text golden mismatch: {}", path.display()).into());
    }
    Ok(())
}

fn assert_authentic_unmodeled(
    artifact: &Path,
    update: bool,
) -> Result<Vec<CaptureEvidence>, Box<dyn std::error::Error>> {
    let text = capture_record(artifact, AUTHENTIC_UNMODELED.key, "standard", 80)?;
    let text_value = String::from_utf8(text.stdout.clone())?;
    assert!(text_value.contains("Key: craft"));
    assert!(text_value.contains("Modifier: +36"));
    assert!(!text_value.contains("Modifier: +0"));
    assert_authentic_golden("areelu-vorlesh-standard-80.txt", &text.stdout, update)?;

    let full = capture_record(artifact, AUTHENTIC_UNMODELED.key, "full", 80)?;
    let full_value = String::from_utf8(full.stdout.clone())?;
    assert!(full_value.contains("Key: craft"));
    assert!(full_value.contains("Modifier: +36"));
    assert!(full_value.contains("The source supplied an unrecognized skill key."));
    assert_eq!(full_value.matches("Key: craft").count(), 1);
    assert_eq!(full_value.matches("Modifier: +36").count(), 1);
    assert_eq!(
        full_value
            .matches("The source supplied an unrecognized skill key.")
            .count(),
        1
    );
    for forbidden in [
        "Source path:",
        "Foundry:",
        "Contract:",
        "System:",
        "Upstream commit:",
    ] {
        assert!(
            !full_value.contains(forbidden),
            "ordinary Full leaked {forbidden}"
        );
    }
    assert_authentic_golden("areelu-vorlesh-full-80.txt", &full.stdout, update)?;

    let provenance = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "--progress",
            "never",
            "record",
            "provenance",
            AUTHENTIC_UNMODELED.key,
            "--json",
        ])
        .arg("--index")
        .arg(artifact)
        .env("COLUMNS", "80")
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .output()?;
    let provenance = CapturedOutput {
        stdout: provenance.stdout,
        stderr: provenance.stderr,
        status: provenance.status.code().unwrap_or(-1),
    };
    if provenance.status != 0 || !provenance.stderr.is_empty() {
        return Err(format!(
            "authentic provenance capture failed: {}",
            String::from_utf8_lossy(&provenance.stderr)
        )
        .into());
    }
    let json: Value = serde_json::from_slice(&provenance.stdout)?;
    let unmodeled = json["data"]["unmodeled_skills"]
        .as_array()
        .ok_or("provenance must expose unmodeled skills")?
        .iter()
        .find(|row| row["authored_key"] == "craft")
        .ok_or("authentic craft evidence missing")?;
    assert_eq!(unmodeled["base"]["state"], "value");
    assert_eq!(unmodeled["base"]["value"], 36);
    assert_authentic_golden("areelu-vorlesh-provenance.json", &provenance.stdout, update)?;
    update_authentic_manifest(update)?;

    Ok(vec![
        capture_evidence(
            &text,
            format!(
                "atlas --progress never record get {} --detail standard --index $BOUNDED_ARTIFACT",
                AUTHENTIC_UNMODELED.key
            ),
            Some(80),
            Some("standard"),
            Some("authentic-unmodeled/areelu-vorlesh-standard-80.txt".into()),
        ),
        capture_evidence(
            &full,
            format!(
                "atlas --progress never record get {} --detail full --index $BOUNDED_ARTIFACT",
                AUTHENTIC_UNMODELED.key
            ),
            Some(80),
            Some("full"),
            Some("authentic-unmodeled/areelu-vorlesh-full-80.txt".into()),
        ),
        capture_evidence(
            &provenance,
            format!(
                "atlas --progress never record provenance {} --json --index $BOUNDED_ARTIFACT",
                AUTHENTIC_UNMODELED.key
            ),
            Some(80),
            Some("provenance-json"),
            Some("authentic-unmodeled/areelu-vorlesh-provenance.json".into()),
        ),
    ])
}

fn assert_authentic_golden(
    name: &str,
    actual: &[u8],
    update: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/record_text/authentic-unmodeled")
        .join(name);
    if update {
        fs::create_dir_all(path.parent().ok_or("authentic path needs a parent")?)?;
        fs::write(path, actual)?;
    } else if fs::read(&path)? != actual {
        return Err(format!("authentic golden mismatch: {}", path.display()).into());
    }
    Ok(())
}

fn update_authentic_manifest(update: bool) -> Result<(), Box<dyn std::error::Error>> {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/record_text/authentic-unmodeled");
    let manifest = root.join("manifest.sha256");
    let mut rows = fs::read_dir(&root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file() && entry.file_name() != "manifest.sha256")
        .map(|entry| {
            let bytes = fs::read(entry.path())?;
            Ok(format!(
                "{}  ./{}",
                sha256_bytes(&bytes),
                entry.file_name().to_string_lossy()
            ))
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    rows.sort();
    let actual = format!("{}\n", rows.join("\n"));
    if update {
        fs::write(manifest, actual)?;
    } else if fs::read_to_string(manifest)? != actual {
        return Err("authentic unmodeled checksum manifest is stale".into());
    }
    Ok(())
}

fn update_matrix_manifest(update: bool) -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/record_text/creature");
    let mut rows = Vec::new();
    for creature in &CREATURES {
        for detail in DETAILS {
            for width in WIDTHS {
                let relative = format!("{}/{detail}-{width}.txt", creature.slug);
                rows.push(format!(
                    "{}  ./{relative}",
                    sha256_bytes(&fs::read(root.join(&relative))?)
                ));
            }
        }
    }
    rows.sort();
    let actual = format!("{}\n", rows.join("\n"));
    let manifest = root
        .parent()
        .ok_or("matrix root needs parent")?
        .join("manifest.sha256");
    if update {
        fs::write(manifest, actual)?;
    } else if fs::read_to_string(manifest)? != actual {
        return Err("creature matrix checksum manifest is stale".into());
    }
    Ok(())
}

fn assert_snapshot_evidence(
    bounded_root: &Path,
    artifact: &Path,
    build_capture: &CaptureEvidence,
    captures: &[CaptureEvidence],
    update: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let artifact_manifest_path = artifact
        .parent()
        .ok_or("artifact needs parent")?
        .join("manifest.json");
    let artifact_bytes = fs::read(artifact)?;
    let artifact_manifest_bytes = fs::read(&artifact_manifest_path)?;
    let mut artifact_manifest: Value = serde_json::from_slice(&artifact_manifest_bytes)?;
    if let Some(root) = artifact_manifest.pointer_mut("/source/root") {
        *root = Value::String("$BOUNDED_SOURCE".into());
    }
    let (edition_lookups, edition_commands) = verified_edition_evidence(artifact)?;

    let golden_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/record_text");
    let matrix_manifest = fs::read(golden_root.join("manifest.sha256"))?;
    let authentic_manifest = fs::read(golden_root.join("authentic-unmodeled/manifest.sha256"))?;
    let source_files = CREATURES
        .iter()
        .chain(std::iter::once(&AUTHENTIC_UNMODELED))
        .map(|creature| {
            serde_json::json!({
                "record_key": creature.key,
                "relative_path": creature.source_path,
                "sha256": creature.source_sha256,
            })
        })
        .chain(std::iter::once(serde_json::json!({
            "relative_path": REMASTER_JOURNAL_PATH,
            "sha256": REMASTER_JOURNAL_SHA256,
        })))
        .collect::<Vec<_>>();

    let evidence = serde_json::json!({
        "format": "pf2e-atlas-cli-text-snapshot-evidence/v1",
        "remediation_base": REMEDIATION_BASE,
        "base_tree": "c215416437d52ad0ad3bf85a8affa391c8f1d2c9",
        "accepted_first_parent_ancestry": [
            "142b4eba993fcf34750d7b8a4b601586a1bf63bd",
            "329125b414a592d60c5e0bb742e8954b11d7bab1",
            "fe97ea0c52697c2c3f863746bb24a7d3d0f98afa",
            "2485a21d60d78a39928ea4f64dccaf2b579d1235",
            "a124c37179ffd15698485cd530307b1a5af33406",
            REMEDIATION_BASE,
        ],
        "controlling_clarification": {
            "path": CLARIFICATION_NOTE_PATH,
            "sha256": CLARIFICATION_NOTE_SHA256,
            "mode": "0444",
            "bytes": 7242,
            "file_type": "regular",
            "supersession_scope": [
                "preview_relationships_and_availability_omission",
                "standard_and_full_opaque_relationship_exclusion",
                "full_debug_provenance_exclusion",
                "dedicated_provenance_command_ownership",
                "natural_profile_headings",
            ],
        },
        "controlling_route": {
            "path": ROUTE_PATH,
            "sha256": ROUTE_SHA256,
            "mode": "0444",
            "bytes": 29216,
            "file_type": "regular",
        },
        "durable_audit": {
            "path": AUDIT_PATH,
            "sha256": AUDIT_SHA256,
            "mode": "0444",
            "bytes": 46729,
            "file_type": "regular",
        },
        "pinned_source": {
            "source_contract": atlas_ingest::PF2E_SOURCE_CONTRACT_VERSION,
            "upstream_commit": atlas_ingest::PF2E_SOURCE_PINNED_COMMIT,
            "upstream_signature": atlas_ingest::PF2E_SOURCE_PINNED_SIGNATURE,
            "files": source_files,
            "bounded_module_sha256": sha256_bytes(&fs::read(bounded_root.join("module.json"))?),
        },
        "bounded_artifact": {
            "embeddings": false,
            "artifact_sha256": sha256_bytes(&artifact_bytes),
            "manifest_sha256": sha256_bytes(&artifact_manifest_bytes),
            "artifact_contract": atlas_index::ARTIFACT_CONTRACT_VERSION,
            "artifact_schema": atlas_index::ARTIFACT_SCHEMA_VERSION,
            "manifest_contract": atlas_index::ARTIFACT_MANIFEST_VERSION,
            "manifest_identity": artifact_manifest,
            "build_capture": build_capture,
        },
        "settings": {
            "profiles": DETAILS,
            "widths": WIDTHS,
            "color": "NO_COLOR=1",
            "term": "dumb",
            "progress": "never",
            "matrix_cells": 135,
        },
        "captures": captures,
        "edition_verification_commands": edition_commands,
        "verified_edition_lookups": edition_lookups,
        "snapshot_hashes": {
            "matrix_manifest_sha256": sha256_bytes(&matrix_manifest),
            "authentic_unmodeled_manifest_sha256": sha256_bytes(&authentic_manifest),
        },
    });
    let mut bytes = serde_json::to_vec_pretty(&evidence)?;
    bytes.push(b'\n');
    let path = golden_root.join("snapshot-evidence.json");
    if update {
        fs::write(path, bytes)?;
    } else {
        let expected: Value = serde_json::from_slice(&fs::read(&path)?)?;
        let mut actual = evidence;
        actual["bounded_artifact"] = expected["bounded_artifact"].clone();
        if actual != expected {
            return Err(format!("snapshot evidence mismatch: {}", path.display()).into());
        }
    }
    Ok(())
}

fn verified_edition_evidence(
    artifact: &Path,
) -> Result<(Vec<Value>, Vec<CaptureEvidence>), Box<dyn std::error::Error>> {
    let mut lookups = Vec::new();
    let mut commands = Vec::new();
    for creature in &CREATURES {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args([
                "--progress",
                "never",
                "record",
                "get",
                creature.key,
                "--detail",
                "full",
                "--json",
            ])
            .arg("--index")
            .arg(artifact)
            .env("COLUMNS", "80")
            .env("NO_COLOR", "1")
            .env("TERM", "dumb")
            .output()?;
        let output = CapturedOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status: output.status.code().unwrap_or(-1),
        };
        if output.status != 0 || !output.stderr.is_empty() {
            return Err(format!("edition verification failed for {}", creature.key).into());
        }
        let json: Value = serde_json::from_slice(&output.stdout)?;
        let edition = &json["data"]["record"]["edition"];
        let state = edition["counterpart_lookup"]["state"]
            .as_str()
            .ok_or("edition lookup state missing")?;
        if state != "verified" {
            return Err(format!("edition lookup was not verified for {}", creature.key).into());
        }
        lookups.push(serde_json::json!({
            "record_key": creature.key,
            "status": edition["status"],
            "counterpart_lookup": state,
            "counterpart_count": edition["counterpart_lookup"]["counterparts"]
                .as_array()
                .map_or(0, Vec::len),
        }));
        commands.push(capture_evidence(
            &output,
            format!(
                "atlas --progress never record get {} --detail full --json --index $BOUNDED_ARTIFACT",
                creature.key
            ),
            Some(80),
            Some("full-json-edition-verification"),
            None,
        ));
    }
    Ok((lookups, commands))
}

fn normalize_layout(output: &str) -> Vec<String> {
    output
        .lines()
        .flat_map(str::split_whitespace)
        .map(str::to_string)
        .collect()
}

fn assert_detail_contract(
    creature: &CreatureFixture,
    values: &[(&str, Vec<String>)],
) -> Result<(), Box<dyn std::error::Error>> {
    let text = |detail| {
        values
            .iter()
            .find(|(candidate, _)| *candidate == detail)
            .map(|(_, words)| words.join(" "))
            .ok_or_else(|| format!("missing {detail} output for {}", creature.name))
    };
    let summary = text("summary")?;
    let preview = text("preview")?;
    let description = text("description")?;
    let standard = text("standard")?;
    let full = text("full")?;
    if summary != format!("{} {} creature", creature.key, creature.name) {
        return Err(format!("summary is not identity-only for {}", creature.name).into());
    }
    for ordinary in [&preview, &description, &standard, &full] {
        if ordinary.contains("record-owned")
            || ordinary.contains("entity-owned")
            || ordinary.contains("occurrence-owned")
            || ordinary.contains("Prepared slot locator")
        {
            return Err(format!(
                "ordinary output leaked placement internals for {}",
                creature.name
            )
            .into());
        }
        for forbidden in [
            "serialized_value_is_provenance_only",
            "serialized 1",
            "PT1M",
            "per-moon",
            "; ;",
            "Component:",
            "Component ID:",
            "Source value: {}",
            "Source value: []",
            "Source value: false",
            "Source value: null",
            "Source value: 0",
            "actor-owned:",
            "Source path:",
            "Foundry:",
            "Contract:",
            "System:",
            "Upstream commit:",
        ] {
            if ordinary.contains(forbidden) {
                return Err(
                    format!("ordinary output leaked {forbidden:?} for {}", creature.name).into(),
                );
            }
        }
    }
    if preview.contains("Data availability") || preview.contains("Relationships") {
        return Err(format!("preview is too noisy for {}", creature.name).into());
    }
    if !standard.contains("Defenses") || !full.contains("Source and edition") {
        return Err(format!("mechanics/provenance profile gap for {}", creature.name).into());
    }
    if creature.slug == "night-hag"
        && (!standard.contains("Focus: maximum 1")
            || !standard.contains("Perception: +18; darkvision"))
    {
        return Err("Night Hag lost maximum-only resources or clean Perception senses".into());
    }
    if creature.slug == "gray-master" && !standard.contains("Frequency: 1 per minute") {
        return Err("Gray Master lost the typed human frequency display".into());
    }
    Ok(())
}
