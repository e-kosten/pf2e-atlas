use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use atlas_app_model::{
    AddEncounterRecordParticipantRequest, CreateEncounterRequest, EncounterParticipantSideView,
    EncounterParticipantVariantView, RecordSurfaceMetadataView, RecordSurfacePresentationView,
    RecordSurfaceProfileView, RecordSurfaceView, SurfaceUnavailableReasonView,
    SurfaceUnavailableView, UpdateEncounterParticipantRequest,
};
use atlas_domain::RecordKey;
use atlas_record::RecordBody;
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntimeOptions};
use serde_json::{Value, json};

use crate::service::{AtlasAppService, RetrievalBackend};
use crate::surface::record_surface;

const BASE: &str = "80a6d222cec02acc1445ac817f5ee646ed147c44";
const APPROVAL_SHA256: &str = "24363e7c74026991d56a4994cd58a9a1f886c0b3f94e59076c10659cf9333807";
const NIGHT_HAG_KEY: &str = "pathfinder-bestiary:WQy7HBUcgDLsfVJd";
const GIANT_RAT_KEY: &str = "pathfinder-monster-core:iIJPJcDT8wlJ8z5M";
const NIGHT_HAG_SOURCE_SHA256: &str =
    "9b7697e6ea8a367b432c9f2d11fdaadf1f32a58b6a6ee5ec2e5ec3ca517829a8";
const GIANT_RAT_SOURCE_SHA256: &str =
    "f8399003c84dff77ec500f39a1e4996bf4a0eadb71be606152ae34f088570b4f";
const SOURCE_SIGNATURE: &str =
    "foundry-pf2e:sha256:dd78d065ea686efedcdb70edb702de1c22c245db27d32824157473886d34ed12";

#[test]
#[ignore = "exports checksum-bound E3 early-direction samples"]
fn export_e3_record_surface_early_samples() {
    let sample_root = required_path_env("E3_SAMPLE_ROOT");
    assert!(
        !sample_root.exists(),
        "sample root must be fresh and no-clobber: {}",
        sample_root.display()
    );
    fs::create_dir_all(
        sample_root
            .parent()
            .expect("sample root should have a parent"),
    )
    .expect("sample parent should be creatable");
    fs::create_dir(&sample_root).expect("fresh sample root should be creatable");
    fs::create_dir(sample_root.join("generated"))
        .expect("generated binding sample directory should be creatable");

    let candidate = required_env("E3_SAMPLE_CANDIDATE");
    let candidate_tree = required_env("E3_SAMPLE_TREE");
    assert_eq!(git_value(Path::new("."), &["rev-parse", "HEAD"]), candidate);
    assert_eq!(
        git_value(Path::new("."), &["rev-parse", "HEAD^{tree}"]),
        candidate_tree
    );
    assert_eq!(git_value(Path::new("."), &["rev-parse", "HEAD^"]), BASE);

    let source_root = required_path_env("E3_SAMPLE_SOURCE_ROOT");
    let source_commit = required_env("E3_SAMPLE_SOURCE_COMMIT");
    let source_tree = required_env("E3_SAMPLE_SOURCE_TREE");
    assert_eq!(
        git_value(&source_root, &["rev-parse", "HEAD"]),
        source_commit
    );
    assert_eq!(
        git_value(&source_root, &["rev-parse", "HEAD^{tree}"]),
        source_tree
    );
    let night_hag_source = source_root.join("packs/pathfinder-bestiary/night-hag.json");
    let giant_rat_source = source_root.join("packs/pathfinder-monster-core/giant-rat.json");
    assert_eq!(file_sha256(&night_hag_source), NIGHT_HAG_SOURCE_SHA256);
    assert_eq!(file_sha256(&giant_rat_source), GIANT_RAT_SOURCE_SHA256);

    let sample_index = required_path_env("E3_SAMPLE_INDEX");
    let sample_index_sha256 = required_env("E3_SAMPLE_INDEX_SHA256");
    assert_eq!(file_sha256(&sample_index), sample_index_sha256);

    let local_state = std::env::temp_dir().join(format!(
        "atlas-e3-early-state-{}-{}.sqlite",
        std::process::id(),
        unique_suffix()
    ));
    let service = AtlasAppService::new(
        RetrievalBackend::OnDemandNoEmbeddings,
        AtlasRuntimeOptions {
            path_mode: AtlasPathMode::Global,
            overrides: AtlasPathOverrides {
                source_root: Some(source_root.clone()),
                embedding_cache_root: None,
                index_path: Some(sample_index.clone()),
            },
        },
        local_state.clone(),
    )
    .expect("sample service should start");

    let night_hag_key = RecordKey::parse(NIGHT_HAG_KEY).expect("Night Hag key should parse");
    let giant_rat_key = RecordKey::parse(GIANT_RAT_KEY).expect("Giant Rat key should parse");
    let records = service
        .get_records(vec![night_hag_key.clone(), giant_rat_key.clone()])
        .expect("real records should hydrate from retained artifact");
    assert_eq!(records.len(), 2);
    let record_named = |key: &RecordKey| {
        records
            .iter()
            .find(|record| &record.record.identity.key == key)
            .unwrap_or_else(|| panic!("real record {key} should hydrate"))
    };
    let night_hag = record_named(&night_hag_key);
    let giant_rat = record_named(&giant_rat_key);
    assert!(matches!(night_hag.body, Some(RecordBody::Creature(_))));
    assert!(matches!(giant_rat.body, Some(RecordBody::Creature(_))));

    let night_hag_compact =
        record_surface(night_hag, RecordSurfaceProfileView::SearchCompact, None);
    let giant_rat_compact =
        record_surface(giant_rat, RecordSurfaceProfileView::SearchCompact, None);
    assert_compact_surface(&night_hag_compact);
    assert_compact_surface(&giant_rat_compact);

    let night_hag_detail = service
        .record_detail(NIGHT_HAG_KEY)
        .expect("Night Hag detail should project");
    let giant_rat_detail = service
        .record_detail(GIANT_RAT_KEY)
        .expect("Giant Rat detail should project");
    assert_detail_surface(&night_hag_detail.surface);
    assert_detail_surface(&giant_rat_detail.surface);
    assert_eq!(
        night_hag_compact.metadata,
        night_hag_detail.surface.metadata
    );
    assert_eq!(
        giant_rat_compact.metadata,
        giant_rat_detail.surface.metadata
    );

    let encounter = service
        .create_encounter(CreateEncounterRequest {
            name: "E3 Early Record Surface".to_string(),
            description: Some(
                "Candidate-authentic transport sample from exactly two retained PF2e records"
                    .to_string(),
            ),
            note: None,
        })
        .expect("sample encounter should create")
        .encounter;
    let after_night_hag = service
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: NIGHT_HAG_KEY.to_string(),
            quantity: 1,
            initiative: Some(19),
        })
        .expect("Night Hag participant should add");
    let night_hag_participant = after_night_hag
        .participants
        .iter()
        .find(|participant| participant.record_key.as_deref() == Some(NIGHT_HAG_KEY))
        .expect("Night Hag participant should be present");
    let night_hag_vitals = night_hag_participant
        .surface
        .encounter
        .as_ref()
        .and_then(|runtime| runtime.vitals.as_ref())
        .expect("Night Hag runtime vitals should be present");
    let maximum_hp = night_hag_vitals
        .maximum_hp
        .as_ref()
        .map(|value| value.adjusted_value);
    service
        .update_encounter_participant(
            &encounter.slug,
            UpdateEncounterParticipantRequest {
                participant_key: night_hag_participant.participant_key.clone(),
                display_name: "Night Hag — Elite".to_string(),
                side: EncounterParticipantSideView::Enemy,
                participant_variant: EncounterParticipantVariantView::Elite,
                initiative: night_hag_participant.initiative,
                max_hp: maximum_hp,
                current_hp: maximum_hp,
                temporary_hp: 0,
                defeated: false,
                hidden: false,
                note: Some("Real context-adjusted candidate serialization".to_string()),
            },
        )
        .expect("Night Hag participant should adjust to elite");
    service
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: GIANT_RAT_KEY.to_string(),
            quantity: 1,
            initiative: Some(12),
        })
        .expect("Giant Rat participant should add");
    let api_encounter = service
        .encounter(&encounter.slug)
        .expect("encounter API detail should project");
    assert_eq!(api_encounter.participants.len(), 2);
    for participant in &api_encounter.participants {
        assert_encounter_surface(&participant.surface);
    }
    let adjusted_night_hag = api_encounter
        .participants
        .iter()
        .find(|participant| participant.record_key.as_deref() == Some(NIGHT_HAG_KEY))
        .expect("adjusted Night Hag should remain present");
    assert_eq!(
        adjusted_night_hag.participant_variant,
        EncounterParticipantVariantView::Elite
    );
    let level = adjusted_night_hag
        .surface
        .encounter
        .as_ref()
        .and_then(|runtime| runtime.level.as_ref())
        .expect("adjusted Night Hag level should be available");
    assert_ne!(level.base_value, level.adjusted_value);
    let normal_giant_rat = api_encounter
        .participants
        .iter()
        .find(|participant| participant.record_key.as_deref() == Some(GIANT_RAT_KEY))
        .expect("normal Giant Rat should remain present");

    let mut concept_dense = night_hag_detail.surface.clone();
    concept_dense.metadata.record_key = None;
    concept_dense.metadata.title = "CONCEPT MOCK — Full Typed Creature Surface".to_string();
    concept_dense.metadata.source = None;
    let concept_unavailable = RecordSurfaceView {
        metadata: RecordSurfaceMetadataView {
            record_key: None,
            title: "CONCEPT MOCK — Unmigrated Record Family".to_string(),
            kind: "hazard".to_string(),
            kind_label: "Hazard".to_string(),
            level: None,
            rarity: None,
            traits: Vec::new(),
            source: None,
        },
        profile: RecordSurfaceProfileView::RecordDetail,
        presentation: RecordSurfacePresentationView::Unavailable {
            unavailable: SurfaceUnavailableView {
                reason: SurfaceUnavailableReasonView::RecordFamilyNotMigrated,
                requested_kind: "hazard".to_string(),
                message: "Typed record presentation is unavailable for this record family."
                    .to_string(),
            },
        },
        encounter: None,
    };
    let concept_mock = json!({
        "sample_classification": "concept_mock_non_authentic_non_authorizing",
        "authentic_foundry_record": false,
        "purpose": "Inspect the complete named-object direction; values are illustrative and must not be treated as source evidence.",
        "full_typed_creature_surface": concept_dense,
    });

    write_json(&sample_root.join("concept-mock.json"), &concept_mock);
    write_json(
        &sample_root.join("search-compact-night-hag.json"),
        &night_hag_compact,
    );
    write_json(
        &sample_root.join("search-compact-sparse-creature.json"),
        &giant_rat_compact,
    );
    write_json(
        &sample_root.join("record-detail-night-hag.json"),
        &night_hag_detail.surface,
    );
    write_json(
        &sample_root.join("record-detail-sparse-creature.json"),
        &giant_rat_detail.surface,
    );
    write_json(
        &sample_root.join("encounter-participant-normal.json"),
        normal_giant_rat,
    );
    write_json(
        &sample_root.join("encounter-participant-adjusted.json"),
        adjusted_night_hag,
    );
    write_json(
        &sample_root.join("api-record-detail.json"),
        &night_hag_detail,
    );
    write_json(
        &sample_root.join("api-encounter-detail.json"),
        &api_encounter,
    );
    write_json(
        &sample_root.join("surface-unavailable-non-creature.json"),
        &concept_unavailable,
    );

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve");
    for binding in [
        "RecordSurfaceView.ts",
        "RecordSurfacePresentationView.ts",
        "CreatureSurfaceView.ts",
        "EncounterParticipantView.ts",
        "EncounterRuntimeView.ts",
    ] {
        fs::copy(
            repo_root
                .join("crates/atlas-app-model/bindings")
                .join(binding),
            sample_root.join("generated").join(binding),
        )
        .unwrap_or_else(|error| panic!("binding {binding} should copy: {error}"));
    }

    fs::write(
        sample_root.join("WALKTHROUGH.md"),
        format!(
            "# E3 early record-surface walkthrough\n\nThis package is **early-direction evidence only** for candidate `{candidate}` (`{candidate_tree}`), not E3 acceptance.\n\n## Start here\n\n1. `concept-mock.json` is clearly labeled mock/non-authentic and shows the full named creature-body direction. `surface-unavailable-non-creature.json` is the separately labeled conceptual unmigrated-family boundary.\n2. `search-compact-night-hag.json` and `record-detail-night-hag.json` show the dense PF2e record.\n3. `search-compact-sparse-creature.json` and `record-detail-sparse-creature.json` show the sparse PF2e record (Giant Rat).\n4. `encounter-participant-normal.json` shows the real Giant Rat and `encounter-participant-adjusted.json` shows the real Night Hag elite adjustment under `surface.encounter`; canonical mechanics are omitted from each encounter-profile creature body.\n5. `api-record-detail.json` and `api-encounter-detail.json` are exact app-service response DTO serializations.\n6. `generated/` contains the exact candidate TypeScript contracts used by these JSON responses.\n\nThe semantic shape has no generic section/value registry, compatibility alias, or target-string lookup. Zero-or-one domains are optional named objects; arrays are used only for repeated domain entities.\n"
        ),
    )
    .expect("walkthrough should write");
    fs::write(
        sample_root.join("presentation.md"),
        "# Presentation index\n\n- Concept direction: `concept-mock.json`\n- Dense real record: `record-detail-night-hag.json`\n- Sparse real record: `record-detail-sparse-creature.json`\n- Context adjustment: `encounter-participant-adjusted.json`\n- Exact API envelopes: `api-record-detail.json`, `api-encounter-detail.json`\n- Generated contract: `generated/RecordSurfaceView.ts` and its referenced bindings\n\nAll concept material is mock/non-authorizing; only Night Hag and Giant Rat are authentic Foundry records.\n",
    )
    .expect("presentation index should write");
    fs::write(
        sample_root.join("report.md"),
        format!(
            "# E3 first-representative report\n\nCandidate `{candidate}` / tree `{candidate_tree}` is a direct child of `{BASE}`. Focused model, app-service, and atlas-web transport tests were run before this package. This root records early direction only and deliberately stops before polish, full workspace validation, independent technical review, final sample approval, E3 acceptance, or F1/F2.\n\nThe package contains one explicitly non-authentic concept mock plus exactly two authentic PF2e records: Night Hag (`{NIGHT_HAG_KEY}`) and Giant Rat (`{GIANT_RAT_KEY}`).\n"
        ),
    )
    .expect("candidate report should write");

    assert_no_generic_surface_residue(&concept_mock);
    for value in [
        serde_json::to_value(&night_hag_compact).expect("compact should serialize"),
        serde_json::to_value(&giant_rat_compact).expect("compact should serialize"),
        serde_json::to_value(&night_hag_detail).expect("detail should serialize"),
        serde_json::to_value(&giant_rat_detail).expect("detail should serialize"),
        serde_json::to_value(&api_encounter).expect("encounter should serialize"),
    ] {
        assert_no_generic_surface_residue(&value);
    }

    let output_hashes = relative_files(&sample_root)
        .into_iter()
        .map(|relative| {
            json!({
                "path": relative.to_string_lossy(),
                "sha256": file_sha256(&sample_root.join(&relative)),
            })
        })
        .collect::<Vec<_>>();
    let manifest = json!({
        "schema": "atlas-e3-early-direction-samples/v1",
        "status": "early_direction_only_not_accepted",
        "candidate": { "commit": candidate, "tree": candidate_tree, "parent": BASE },
        "approval": { "sha256": APPROVAL_SHA256 },
        "producer": {
            "command": "cargo test -p atlas-app-service e3_sample_export::export_e3_record_surface_early_samples -- --ignored --exact",
            "test_path": "crates/atlas-app-service/src/e3_sample_export.rs"
        },
        "source": {
            "root": source_root,
            "commit": source_commit,
            "tree": source_tree,
            "signature": SOURCE_SIGNATURE,
            "records": [
                { "classification": "real_dense", "record_key": NIGHT_HAG_KEY, "source_path": "packs/pathfinder-bestiary/night-hag.json", "sha256": NIGHT_HAG_SOURCE_SHA256 },
                { "classification": "real_sparse", "record_key": GIANT_RAT_KEY, "source_path": "packs/pathfinder-monster-core/giant-rat.json", "sha256": GIANT_RAT_SOURCE_SHA256 }
            ]
        },
        "retained_artifact": { "path": sample_index, "sha256": sample_index_sha256 },
        "mock_policy": {
            "mock_file": "concept-mock.json",
            "mock_is_authentic": false,
            "real_record_count": 2,
            "real_records": [NIGHT_HAG_KEY, GIANT_RAT_KEY]
        },
        "outputs": output_hashes,
    });
    write_json(&sample_root.join("manifest.json"), &manifest);

    let mut checksum_lines = relative_files(&sample_root)
        .into_iter()
        .map(|relative| {
            format!(
                "{}  {}",
                file_sha256(&sample_root.join(&relative)),
                relative.to_string_lossy()
            )
        })
        .collect::<Vec<_>>();
    checksum_lines.sort();
    fs::write(
        sample_root.join("checksums.sha256"),
        format!("{}\n", checksum_lines.join("\n")),
    )
    .expect("checksums should write");
    for relative in relative_files_including_checksums(&sample_root) {
        let path = sample_root.join(relative);
        let mut permissions = fs::metadata(&path)
            .expect("sample metadata should read")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions).expect("sample file should seal read-only");
    }

    drop(service);
    let _ = fs::remove_file(local_state);
}

fn assert_compact_surface(surface: &RecordSurfaceView) {
    assert_eq!(surface.profile, RecordSurfaceProfileView::SearchCompact);
    assert!(surface.encounter.is_none());
    let RecordSurfacePresentationView::Creature { body } = &surface.presentation else {
        panic!("real creature compact surface should be typed")
    };
    assert!(body.vitals.is_some());
    assert!(body.defenses.is_some());
    assert!(body.awareness.is_some());
    assert!(body.saves.is_none());
    assert!(body.abilities.is_none());
    assert!(body.skills.is_none());
    assert!(body.activities.is_none());
    assert!(body.content.is_none());
}

fn assert_detail_surface(surface: &RecordSurfaceView) {
    assert_eq!(surface.profile, RecordSurfaceProfileView::RecordDetail);
    assert!(surface.encounter.is_none());
    let RecordSurfacePresentationView::Creature { body } = &surface.presentation else {
        panic!("real creature detail surface should be typed")
    };
    assert!(body.vitals.is_some());
    assert!(body.defenses.is_some());
    assert!(body.saves.is_some());
    assert!(body.awareness.is_some());
    assert!(body.abilities.is_some());
    assert!(body.skills.is_some());
    assert!(body.movement.is_some());
    assert!(body.activities.is_some());
    assert!(body.content.is_some());
    assert!(body.provenance.is_some());
    assert!(body.skills.as_ref().is_none_or(|values| {
        values
            .windows(2)
            .all(|pair| pair[0].authored_order <= pair[1].authored_order)
    }));
    assert!(body.activities.as_ref().is_none_or(|values| {
        values
            .windows(2)
            .all(|pair| pair[0].authored_order <= pair[1].authored_order)
    }));
    assert!(body.content.as_ref().is_none_or(|values| {
        values
            .windows(2)
            .all(|pair| pair[0].authored_order <= pair[1].authored_order)
    }));
}

fn assert_encounter_surface(surface: &RecordSurfaceView) {
    assert_eq!(
        surface.profile,
        RecordSurfaceProfileView::EncounterParticipant
    );
    assert!(surface.encounter.is_some());
    let RecordSurfacePresentationView::Creature { body } = &surface.presentation else {
        panic!("real encounter creature surface should be typed")
    };
    assert!(body.vitals.is_none());
    assert!(body.defenses.is_none());
    assert!(body.saves.is_none());
    assert!(body.awareness.is_none());
    assert!(body.abilities.is_none());
    assert!(body.skills.is_none());
    assert!(body.movement.is_none());
    assert!(body.resources.is_none());
    assert!(body.spellcasting.is_none());
    assert!(body.activities.is_none());
    assert!(body.content.is_some());
    assert!(body.provenance.is_some());
}

fn assert_no_generic_surface_residue(value: &Value) {
    match value {
        Value::Object(object) => {
            for key in object.keys() {
                assert_ne!(key, &["sec", "tions"].concat());
                assert_ne!(key, &["section", "order"].join("_"));
            }
            for child in object.values() {
                assert_no_generic_surface_residue(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_no_generic_surface_residue(child);
            }
        }
        _ => {}
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(value).expect("sample should serialize")
        ),
    )
    .unwrap_or_else(|error| panic!("sample {} should write: {error}", path.display()));
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set"))
}

fn required_path_env(name: &str) -> PathBuf {
    PathBuf::from(required_env(name))
}

fn git_value(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git should run");
    assert!(output.status.success(), "git command should succeed");
    String::from_utf8(output.stdout)
        .expect("git output should be UTF-8")
        .trim()
        .to_string()
}

fn file_sha256(path: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .expect("shasum should run");
    assert!(output.status.success(), "shasum should succeed");
    String::from_utf8(output.stdout)
        .expect("shasum output should be UTF-8")
        .split_whitespace()
        .next()
        .expect("shasum should emit a digest")
        .to_string()
}

fn relative_files(root: &Path) -> Vec<PathBuf> {
    relative_files_filtered(root, true)
}

fn relative_files_including_checksums(root: &Path) -> Vec<PathBuf> {
    relative_files_filtered(root, false)
}

fn relative_files_filtered(root: &Path, skip_checksums: bool) -> Vec<PathBuf> {
    fn visit(root: &Path, path: &Path, skip_checksums: bool, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(path).expect("sample directory should read") {
            let entry = entry.expect("sample entry should read");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, skip_checksums, files);
            } else if !(skip_checksums
                && path.file_name().and_then(|name| name.to_str()) == Some("checksums.sha256"))
            {
                files.push(
                    path.strip_prefix(root)
                        .expect("sample path should be relative")
                        .to_path_buf(),
                );
            }
        }
    }
    let mut files = Vec::new();
    visit(root, root, skip_checksums, &mut files);
    files.sort();
    files
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should follow unix epoch")
        .as_nanos()
}
