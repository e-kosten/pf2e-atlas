use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use sha2::{Digest, Sha256};

mod support;

use support::path::temp_source_root;

const SOURCE_ROOT_ENV: &str = "PF2E_SOURCE_REPOSITORY";
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
        source_path: "packs/pathfinder-bestiary/night-hag.json",
        source_sha256: "9b7697e6ea8a367b432c9f2d11fdaadf1f32a58b6a6ee5ec2e5ec3ca517829a8",
    },
    CreatureFixture {
        slug: "giant-rat",
        name: "Giant Rat",
        key: "pathfinder-monster-core:iIJPJcDT8wlJ8z5M",
        source_path: "packs/pathfinder-monster-core/giant-rat.json",
        source_sha256: "f8399003c84dff77ec500f39a1e4996bf4a0eadb71be606152ae34f088570b4f",
    },
    CreatureFixture {
        slug: "calcifda",
        name: "Calcifda",
        key: "quest-for-the-frozen-flame-bestiary:kxO7gXbpcmY1pi7p",
        source_path: "packs/quest-for-the-frozen-flame-bestiary/book-3-burning-tundra/calcifda.json",
        source_sha256: "83a68237aac93565f41e75d6ce6e89d5a1e88a468623e1c7e3c34aa3def71c7c",
    },
    CreatureFixture {
        slug: "magical-forge",
        name: "Magical Forge",
        key: "triumph-of-the-tusk-bestiary:EBpwhiVtWKqi8M3n",
        source_path: "packs/triumph-of-the-tusk-bestiary/book-2-hoof-cinder-and-storm/magical-forge.json",
        source_sha256: "1b61ab9c5014d98617d2cae9dd614b109b7806a32f8c37a128be3bdc0d64a831",
    },
    CreatureFixture {
        slug: "balor",
        name: "Balor",
        key: "pathfinder-bestiary:9vNYtJZiseCEf4wt",
        source_path: "packs/pathfinder-bestiary/balor.json",
        source_sha256: "0ef5eaa7137946177208127edf08387491d005877fbe0d8d0ade38d1471a9e8a",
    },
    CreatureFixture {
        slug: "gray-master",
        name: "Gray Master",
        key: "curtain-call-bestiary:1eX4Csnv3psAsfLf",
        source_path: "packs/curtain-call-bestiary/book-3-bring-the-house-down/gray-master.json",
        source_sha256: "497ccd4fe79f6579d7ac33bd9f48bc941bb10182ac82b54ed9c60179208e20f6",
    },
    CreatureFixture {
        slug: "beluthus",
        name: "Beluthus",
        key: "abomination-vaults-bestiary:3ry9WSvMMXHUe3kE",
        source_path: "packs/abomination-vaults-bestiary/abomination-vaults-hardcover-compilation/beluthus.json",
        source_sha256: "bbee3086eaf0745d4feda720a5e15eb7a4db4434f3526dc7e514e76f8096a358",
    },
    CreatureFixture {
        slug: "air-mephit",
        name: "Air Mephit",
        key: "pathfinder-bestiary:KDRlxdIUADWHI6Vr",
        source_path: "packs/pathfinder-bestiary/air-mephit.json",
        source_sha256: "60d5c44540c0a8795bba01271150022a6edb6d8bd07129f585f0c5edaba63264",
    },
    CreatureFixture {
        slug: "air-scamp",
        name: "Air Scamp",
        key: "pathfinder-monster-core:MSm1im7lZA5i82rz",
        source_path: "packs/pathfinder-monster-core/air-scamp.json",
        source_sha256: "e7e3508b648dc311e21d43c3bf369c9e18bc96c2a07ac7b8c79d721701a0dd05",
    },
];

const REMASTER_JOURNAL_PATH: &str = "packs/journals/remaster-changes.json";
const REMASTER_JOURNAL_SHA256: &str =
    "f422ed137d9f15ca320c19b5675c94ab26c0fa1f5b3e2ba6e157d39fe836a826";
const AUTHENTIC_UNMODELED: CreatureFixture = CreatureFixture {
    slug: "chernasardo-ranger",
    name: "Chernasardo Ranger",
    key: "claws-of-the-tyrant-bestiary:0mllbU5aBEcVUj3E",
    source_path: "packs/claws-of-the-tyrant-bestiary/2-ashes-for-ozem/chernasardo-ranger.json",
    source_sha256: "b655a11ea8928936ca586f13fd74a0f87c795454e618e8cb4d22d54a15bd566c",
};

#[test]
#[ignore = "requires pinned PF2e fixtures; CI runs this comparison explicitly"]
fn source_backed_creature_terminal_matrix() -> Result<(), Box<dyn std::error::Error>> {
    let source_root = source_root()
        .ok_or_else(|| format!("rendering comparison requires {SOURCE_ROOT_ENV} or vendor/pf2e"))?;
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
    assert_authentic_unmodeled(&artifact, update)?;
    assert_verified_editions(&artifact)?;

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
    let manifest: Value =
        serde_json::from_slice(&fs::read(source_root.join("static/system.json"))?)?;
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
                    "claws-of-the-tyrant-bestiary",
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
            .strip_prefix("packs/")
            .ok_or("bounded source path must begin with packs")?;
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
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
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
    Ok(output)
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
) -> Result<(), Box<dyn std::error::Error>> {
    let text = capture_record(artifact, AUTHENTIC_UNMODELED.key, "standard", 80)?;
    let text_value = String::from_utf8(text.stdout.clone())?;
    assert!(text_value.contains("Key: acrobatics+13"));
    assert!(!text_value.contains("Modifier:"));
    assert!(!text_value.contains("Modifier: +0"));
    assert_authentic_golden("chernasardo-ranger-standard-80.txt", &text.stdout, update)?;

    let full = capture_record(artifact, AUTHENTIC_UNMODELED.key, "full", 80)?;
    let full_value = String::from_utf8(full.stdout.clone())?;
    assert!(full_value.contains("Key: acrobatics+13"));
    assert!(!full_value.contains("Modifier:"));
    assert!(full_value.contains("The source supplied an unrecognized skill key."));
    assert_eq!(full_value.matches("Key: acrobatics+13").count(), 1);
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
    assert_authentic_golden("chernasardo-ranger-full-80.txt", &full.stdout, update)?;

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
    if !provenance.status.success() || !provenance.stderr.is_empty() {
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
        .find(|row| row["authored_key"] == "acrobatics+13")
        .ok_or("authentic acrobatics+13 evidence missing")?;
    assert_eq!(unmodeled["base"]["state"], "null");
    assert!(unmodeled["base"].get("value").is_none());
    assert_authentic_golden(
        "chernasardo-ranger-provenance.json",
        &provenance.stdout,
        update,
    )?;
    update_authentic_manifest(update)?;

    Ok(())
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

fn assert_verified_editions(artifact: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
        if !output.status.success() || !output.stderr.is_empty() {
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
