use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use sha2::{Digest, Sha256};

mod support;

use support::path::temp_source_root;

const SOURCE_ROOT_ENV: &str = "PF2E_ATLAS_B5_SOURCE_ROOT";
const UPDATE_ENV: &str = "PF2E_ATLAS_UPDATE_RECORD_TEXT_GOLDENS";
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

struct CreatureFixture {
    slug: &'static str,
    name: &'static str,
    key: &'static str,
    source_path: &'static str,
    source_sha256: &'static str,
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
    build_bounded_artifact(&root, &artifact)?;

    let update = std::env::var_os(UPDATE_ENV).is_some();
    for creature in &CREATURES {
        let mut normalized_by_detail = Vec::new();
        for detail in DETAILS {
            let mut normalized_by_width = Vec::new();
            for width in WIDTHS {
                let output = capture_record(&artifact, creature.key, detail, width)?;
                if detail != "summary" {
                    assert_eq!(
                        output.lines().next(),
                        Some(creature.name),
                        "{detail} width {width} header for {}",
                        creature.name
                    );
                }
                assert_golden(creature.slug, detail, width, &output, update)?;
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

    fs::remove_dir_all(root)?;
    Ok(())
}

fn has_indented_unbreakable_token(line: &str, width: usize) -> bool {
    let indent = line.len() - line.trim_start().len();
    line.split_whitespace()
        .any(|token| indent + token.len() > width)
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
                    "triumph-of-the-tusk-bestiary",
                ]
                .contains(&name)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if selected_packs.len() != 7 {
        return Err(format!("expected 7 bounded packs, found {}", selected_packs.len()).into());
    }
    fs::write(
        target.join("module.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"packs": selected_packs}))?,
    )?;
    for source_path in CREATURES
        .iter()
        .map(|creature| creature.source_path)
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
) -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

fn capture_record(
    artifact: &Path,
    key: &str,
    detail: &str,
    width: usize,
) -> Result<String, Box<dyn std::error::Error>> {
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
    Ok(String::from_utf8(output.stdout)?)
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
    if !standard.contains("Defenses") || !full.contains("Provenance and edition") {
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
