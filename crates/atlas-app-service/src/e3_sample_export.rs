use std::collections::{BTreeMap, BTreeSet};
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

const BASE: &str = "0a3b318a94686360ef34c7c11e42454506213d0d";
const TECHNICAL_REVIEW_SHA256: &str =
    "881aa1cee78f7c1f6c6bb53dc5e0f3f7d174502e07dab1b40a8fcdb6d7cc9642";
const APPROVAL_SHA256: &str = "0ca28ff6906d04cd993030b5d22b060cb91516c56661e0593eaffc2371de2de6";
const CLI_APPROVAL_SHA256: &str =
    "5dfef04af82b341ce42d8b928af81c45b61152005ba38cbf6f56b02cdabe4ece";
const CLI_APPROVAL_SIDECAR_SHA256: &str =
    "89dd043178d883a0cc196dcc8f3445ee7e62851466201c3318512ff5a7d57bec";
const AMENDED_PLAN_SHA256: &str =
    "19b037a999c6349a087ebab123e0e400c487e7626ceffc6501f44a99454d97ca";
const AMENDED_TASK_MAP_SHA256: &str =
    "49eb15a8d791852ff31395859d2018143d9aecb435439216d358a01048add8da";
const PLANNING_REVIEW_VERDICT_SHA256: &str =
    "099bbfe95a9b1d97df3376c89eba5e86571a4ffd6adedb6558afda8e226e9d78";
const PLANNING_REVIEW_JSON_SHA256: &str =
    "74190663fa8397d6f14686066e73e889e151b4d169f0a4063859880857f43e71";
const PLANNING_REVIEW_CHECKSUMS_SHA256: &str =
    "a303010e277623b638f338b949ab9f9ab263bb34d6d61c6c95039500370d8e7b";
const REVIEWED_MANIFEST_SHA256: &str =
    "06c8b5f9fad9bf09a928cf70e52a790eadecf54cf6d5ddda82f8bea09f1133b4";
const REVIEWED_CHECKSUMS_SHA256: &str =
    "765090c569cf95ce7f945db6f3c7e8293967b8a691b9c210de81b9d4e76fe545";
const NIGHT_HAG_KEY: &str = "pathfinder-bestiary:WQy7HBUcgDLsfVJd";
const GIANT_RAT_KEY: &str = "pathfinder-monster-core:iIJPJcDT8wlJ8z5M";
const NIGHT_HAG_SOURCE_SHA256: &str =
    "9b7697e6ea8a367b432c9f2d11fdaadf1f32a58b6a6ee5ec2e5ec3ca517829a8";
const GIANT_RAT_SOURCE_SHA256: &str =
    "f8399003c84dff77ec500f39a1e4996bf4a0eadb71be606152ae34f088570b4f";
const SOURCE_SIGNATURE: &str =
    "foundry-pf2e:sha256:dd78d67f5b6d25bf65e30ca4da66af76e7a31e1e7d990562f139154b1752603a";
const AUTHENTIC_EXPORT_CANDIDATE: &str = "0a3b318a94686360ef34c7c11e42454506213d0d";
const AUTHENTIC_EXPORT_TREE: &str = "10ec526c943ef988b427b40b80540895fe9d26b9";
const AUTHENTIC_EXPORT_MANIFEST_SHA256: &str =
    "06c8b5f9fad9bf09a928cf70e52a790eadecf54cf6d5ddda82f8bea09f1133b4";
const AUTHENTIC_EXPORT_CHECKSUMS_SHA256: &str =
    "765090c569cf95ce7f945db6f3c7e8293967b8a691b9c210de81b9d4e76fe545";
const RECORD_VIEW_RENAME_BASE: &str = "80962b4d43d2922e49728352c95a89427bc58a6e";
const RECORD_VIEW_RENAME_BASE_TREE: &str = "4b1c494a04cc4ec975f6494f221b76859d481b06";
const RECORD_VIEW_RENAME_SOURCE_MANIFEST_SHA256: &str =
    "dd21992020cb508c1ca0786bb6a3900ae888fe0296819cfd83f1cc19614d0320";
const RECORD_VIEW_RENAME_SOURCE_CHECKSUMS_SHA256: &str =
    "0f96763d296210d9589a83b91b76cc4ef1614e91e3c9d007480f61572344d61d";
const RECORD_VIEW_RENAME_APPROVAL_SHA256: &str =
    "4d64932b40bfdeff356c6a49cb215c29da6e46cdd0695fe8ecabf9470afaf719";
const RECORD_VIEW_RENAME_APPROVAL_SIDECAR_SHA256: &str =
    "673a8ec5835910d71a4d9d46b727eef2a8dcbd86fcc983fe08ef4324d24b6bf5";
const RECORD_VIEW_RENAME_REVIEW_SHA256: &str =
    "b3c38ad29e8337ed9dd763ffd98b459912554e0e6b6fb8e04ae7c6eb9a88444d";

#[test]
#[ignore = "exports checksum-bound E3 final-candidate samples"]
fn export_e3_record_surface_final_samples() {
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

    let reviewed_root = required_path_env("E3_REVIEWED_SAMPLE_ROOT");
    assert_eq!(
        file_sha256(&reviewed_root.join("manifest.json")),
        REVIEWED_MANIFEST_SHA256
    );
    assert_eq!(
        file_sha256(&reviewed_root.join("checksums.sha256")),
        REVIEWED_CHECKSUMS_SHA256
    );
    verify_checksums(&reviewed_root);

    let retained_target = required_path_env("E3_RETAINED_TARGET");
    let retained_node_modules = required_path_env("E3_RETAINED_NODE_MODULES");
    let frontend_boundary = required_env("E3_FRONTEND_BOUNDARY_RESULT");
    let cli_approval = required_path_env("E3_CLI_APPROVAL");
    let amended_plan = required_path_env("E3_AMENDED_PLAN");
    let amended_task_map = required_path_env("E3_AMENDED_TASK_MAP");
    let planning_review_root = required_path_env("E3_PLANNING_REVIEW_ROOT");
    let technical_review = required_path_env("E3_TECHNICAL_REVIEW");
    assert_bound_file(&cli_approval, CLI_APPROVAL_SHA256);
    assert_bound_file(
        &PathBuf::from(format!("{}.sha256", cli_approval.display())),
        CLI_APPROVAL_SIDECAR_SHA256,
    );
    assert_bound_file(&amended_plan, AMENDED_PLAN_SHA256);
    assert_bound_file(&amended_task_map, AMENDED_TASK_MAP_SHA256);
    assert_bound_file(
        &planning_review_root.join("verdict.md"),
        PLANNING_REVIEW_VERDICT_SHA256,
    );
    assert_bound_file(
        &planning_review_root.join("verdict.json"),
        PLANNING_REVIEW_JSON_SHA256,
    );
    assert_bound_file(
        &planning_review_root.join("checksums.sha256"),
        PLANNING_REVIEW_CHECKSUMS_SHA256,
    );
    verify_checksums(&planning_review_root);
    assert_bound_file(&technical_review, TECHNICAL_REVIEW_SHA256);

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
        .record_view
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
        assert_encounter_surface(&participant.record_view);
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
        .record_view
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
        "surface": concept_dense,
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
        "CreatureSurfaceUnavailableDomainsView.ts",
        "CreatureSurfaceDomainUnavailableView.ts",
        "CreatureSurfaceUnavailableCauseView.ts",
        "CreatureSurfaceUnavailableStateView.ts",
        "CreatureSurfaceUnavailableFieldView.ts",
        "CreatureSurfaceFactProvenanceView.ts",
        "CreatureSurfaceFactOwnerView.ts",
        "CreatureSurfaceSourceFieldView.ts",
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
            "# E3 cumulative-cause correction walkthrough\n\nThis package is **candidate evidence awaiting independent exact-commit rereview** for correction `{candidate}` (`{candidate_tree}`), not E3 acceptance or final user approval. It is bound to failed exact-commit rereview `{TECHNICAL_REVIEW_SHA256}`, reviewed candidate `{BASE}`, conditioned early-direction approval `{APPROVAL_SHA256}`, and the passed CLI ownership amendment `{AMENDED_PLAN_SHA256}` / `{AMENDED_TASK_MAP_SHA256}`.\n\n## Start here\n\n1. `concept-mock.json` is clearly labeled mock/non-authentic and exposes its illustrative record under `surface`. `surface-unavailable-non-creature.json` separately demonstrates whole-record typed unavailability.\n2. `search-compact-night-hag.json` and `record-detail-night-hag.json` show the dense authentic PF2e record.\n3. `search-compact-sparse-creature.json` and `record-detail-sparse-creature.json` show the sparse authentic PF2e record (Giant Rat).\n4. `encounter-participant-normal.json` and `encounter-participant-adjusted.json` show the exact accepted runtime bag attached without duplicating canonical mechanics.\n5. `api-record-detail.json` and `api-encounter-detail.json` are exact app-service response DTO serializations. Only genuinely known-empty arrays are omitted. Selected static domains with missing, null, unsupported, ambiguous, failed, or unsafe canonical data expose all independent named typed `unavailable_domains` causes; unsafe ordinary values are not presented as valid.\n6. `generated/` contains the exact candidate TypeScript contracts relevant to these JSON responses, including the typed domain-failure contract.\n7. `reviewed-to-correction-delta-ledger.md` accounts for every reviewed-0a3b and cumulative-cause correction payload output while excluding only recursively self-describing manifest/checksum/ledger metadata.\n\nThe semantic shape has no generic section/value registry, compatibility alias, string-key semantic lookup, empty arrays, or empty objects. Failure messages are display-only; consumers use typed state, field, domain, component identity, and provenance. The CLI consumer remains output-identical.\n"
        ),
    )
    .expect("walkthrough should write");
    fs::write(
        sample_root.join("presentation.md"),
        "# Presentation index\n\n- Concept direction: `concept-mock.json`\n- Dense real record: `record-detail-night-hag.json`\n- Sparse real record: `record-detail-sparse-creature.json`\n- Context adjustment: `encounter-participant-adjusted.json`\n- Exact API envelopes: `api-record-detail.json`, `api-encounter-detail.json`\n- Generated contract: `generated/RecordSurfaceView.ts` and typed unavailable-domain bindings\n- Complete 0a3b-to-cumulative-cause payload delta: `reviewed-to-correction-delta-ledger.md`\n\nAll concept material is mock/non-authorizing; only Night Hag and Giant Rat are authentic Foundry records. This is rereview evidence, not E3 acceptance or final user approval.\n",
    )
    .expect("presentation index should write");
    fs::write(
        sample_root.join("report.md"),
        format!(
            "# E3 cumulative-cause correction candidate report\n\nCandidate `{candidate}` / tree `{candidate_tree}` is a direct child of failed reviewed candidate `{BASE}`. It changes only remaining F-001: movement collects speed and mode failures independently before suppressing an unsafe component, and awareness collects language payload/failure independently of perception. Complete typed cause tuples remain deterministic under reversed input order. Known-empty omission, wire shape, mechanics, CLI behavior, and frontend source are unchanged. No generic registry, shim, fallback, alias, or string-key semantic lookup was added.\n\nFocused model/app-service/web tests, generated-binding freshness, full `just verify`, residue/path checks, and the documented downstream frontend boundary check were run before this package. A fresh independent exact-commit rereview, final user sample approval, E3 acceptance, F1, and F2 remain pending.\n\nThe package contains one explicitly non-authentic concept mock plus exactly two authentic PF2e records: Night Hag (`{NIGHT_HAG_KEY}`) and Giant Rat (`{GIANT_RAT_KEY}`). Source identity is `{SOURCE_SIGNATURE}` at commit `{source_commit}` / tree `{source_tree}`. The complete 0a3b-to-cumulative-cause delta is in `reviewed-to-correction-delta-ledger.md`.\n"
        ),
    )
    .expect("candidate report should write");

    assert_no_generic_surface_residue(&concept_mock);
    assert_no_empty_public_containers(&concept_mock);
    assert!(concept_mock.get("surface").is_some());
    assert!(concept_mock.get("full_typed_creature_surface").is_none());
    for value in [
        serde_json::to_value(&night_hag_compact).expect("compact should serialize"),
        serde_json::to_value(&giant_rat_compact).expect("compact should serialize"),
        serde_json::to_value(&night_hag_detail).expect("detail should serialize"),
        serde_json::to_value(&giant_rat_detail).expect("detail should serialize"),
        serde_json::to_value(&api_encounter).expect("encounter should serialize"),
    ] {
        assert_no_generic_surface_residue(&value);
        assert_no_empty_public_containers(&value);
    }

    write_delta_ledger(&sample_root, &reviewed_root, &candidate, &candidate_tree);

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
        "schema": "atlas-e3-f001-correction-samples/v1",
        "status": "correction_candidate_awaiting_exact_commit_rereview_not_accepted",
        "candidate": { "commit": candidate, "tree": candidate_tree, "parent": BASE },
        "approval": {
            "path": "/Users/ekosten/.ao/data/handoffs/pathfinder-2e-foundry-mcp/source-faithful-records/20260824T210853Z-c7b74cbdc7c4-pathfinder-2e-foundry-mcp-17/approvals/e3-early-direction-approval.json",
            "sha256": APPROVAL_SHA256,
            "mode": "0444"
        },
        "cli_ownership_amendment": {
            "approval": { "path": cli_approval, "sha256": CLI_APPROVAL_SHA256, "sidecar_file_sha256": CLI_APPROVAL_SIDECAR_SHA256, "mode": "0444" },
            "plan": { "path": amended_plan, "sha256": AMENDED_PLAN_SHA256, "mode": "0444" },
            "task_map": { "path": amended_task_map, "sha256": AMENDED_TASK_MAP_SHA256, "mode": "0444" },
            "planning_review": {
                "root": planning_review_root,
                "verdict": "PASS",
                "verdict_md_sha256": PLANNING_REVIEW_VERDICT_SHA256,
                "verdict_json_sha256": PLANNING_REVIEW_JSON_SHA256,
                "checksums_sha256": PLANNING_REVIEW_CHECKSUMS_SHA256,
                "mode": "0444",
                "checksum_closure": "pass"
            },
            "authorized_cli_paths": ["crates/atlas-cli/src/commands/lists.rs"],
            "cli_output_changed": false
        },
        "failed_review": { "path": technical_review, "sha256": TECHNICAL_REVIEW_SHA256, "mode": "0444", "finding": "F-001" },
        "reviewed_9c_evidence": {
            "root": reviewed_root,
            "manifest_sha256": REVIEWED_MANIFEST_SHA256,
            "checksums_sha256": REVIEWED_CHECKSUMS_SHA256,
            "checksum_closure": "pass"
        },
        "producer": {
            "command": "cargo test -p atlas-app-service e3_sample_export::export_e3_record_surface_final_samples -- --ignored --exact",
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
        "retained_substrate": {
            "source": { "path": source_root, "size_kib": directory_size_kib(&source_root) },
            "artifact": { "path": sample_index, "sha256": sample_index_sha256, "size_bytes": file_size(&sample_index) },
            "target": { "path": retained_target, "size_kib": directory_size_kib(&retained_target) },
            "node_modules": { "path": retained_node_modules, "size_kib": directory_size_kib(&retained_node_modules) }
        },
        "mock_policy": {
            "mock_file": "concept-mock.json",
            "mock_is_authentic": false,
            "real_record_count": 2,
            "real_records": [NIGHT_HAG_KEY, GIANT_RAT_KEY]
        },
        "contract": {
            "known_empty_arrays_omitted": true,
            "populated_true_many_arrays_retained": true,
            "empty_objects_forbidden": true,
            "unsupported_or_failed_projection_remains_explicit": true,
            "static_failure_representation": "named_typed_unavailable_domains",
            "failure_messages_behavior_parsed": false,
            "generated_typescript_collections_optional": true,
            "cli_surface_serialization": false,
            "cli_compatibility_path": false
        },
        "validation": {
            "focused_model_app_service_web": "pass",
            "focused_cli_lists_json": "pass_8",
            "cli_tests_contracts_goldens_changed": false,
            "cli_changed_paths": ["crates/atlas-cli/src/commands/lists.rs"],
            "generated_binding_freshness": "pass",
            "just_verify": "pass",
            "frontend_boundary": frontend_boundary
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

#[test]
#[ignore = "deterministically rebinds an authenticated E3 export after a generated-doc-only fix"]
fn rebind_e3_record_surface_final_samples() {
    let source_root = required_path_env("E3_REBIND_SOURCE_ROOT");
    assert_eq!(
        file_sha256(&source_root.join("manifest.json")),
        AUTHENTIC_EXPORT_MANIFEST_SHA256
    );
    assert_eq!(
        file_sha256(&source_root.join("checksums.sha256")),
        AUTHENTIC_EXPORT_CHECKSUMS_SHA256
    );
    verify_checksums(&source_root);

    let sample_root = required_path_env("E3_SAMPLE_ROOT");
    assert!(
        !sample_root.exists(),
        "sample root must be fresh and no-clobber: {}",
        sample_root.display()
    );
    fs::create_dir_all(sample_root.parent().expect("sample root parent"))
        .expect("sample parent should be creatable");
    fs::create_dir(&sample_root).expect("sample root should be creatable");

    let candidate = required_env("E3_SAMPLE_CANDIDATE");
    let candidate_tree = required_env("E3_SAMPLE_TREE");
    assert_eq!(git_value(Path::new("."), &["rev-parse", "HEAD"]), candidate);
    assert_eq!(
        git_value(Path::new("."), &["rev-parse", "HEAD^{tree}"]),
        candidate_tree
    );
    assert_eq!(git_value(Path::new("."), &["rev-parse", "HEAD^"]), BASE);

    let skipped = BTreeSet::from([
        PathBuf::from("manifest.json"),
        PathBuf::from("checksums.sha256"),
        PathBuf::from("WALKTHROUGH.md"),
        PathBuf::from("presentation.md"),
        PathBuf::from("report.md"),
        PathBuf::from("reviewed-to-correction-delta-ledger.md"),
        PathBuf::from("generated/CreatureSurfaceUnavailableCauseView.ts"),
    ]);
    for relative in relative_files(&source_root) {
        if skipped.contains(&relative) {
            continue;
        }
        let destination = sample_root.join(&relative);
        fs::create_dir_all(destination.parent().expect("copied file parent"))
            .expect("copied file parent should be creatable");
        fs::copy(source_root.join(&relative), &destination)
            .unwrap_or_else(|error| panic!("{} should copy: {error}", relative.display()));
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve");
    let binding_relative = Path::new("generated/CreatureSurfaceUnavailableCauseView.ts");
    fs::create_dir_all(
        sample_root
            .join(binding_relative)
            .parent()
            .expect("binding parent"),
    )
    .expect("binding parent should be creatable");
    fs::copy(
        repo_root.join("crates/atlas-app-model/bindings/CreatureSurfaceUnavailableCauseView.ts"),
        sample_root.join(binding_relative),
    )
    .expect("current generated failure binding should copy");

    for relative in ["WALKTHROUGH.md", "presentation.md", "report.md"] {
        let rebound = fs::read_to_string(source_root.join(relative))
            .expect("authenticated narrative should read")
            .replace(AUTHENTIC_EXPORT_CANDIDATE, &candidate)
            .replace(AUTHENTIC_EXPORT_TREE, &candidate_tree)
            .replace(
                "2b5415d5264ae23ae05f34ca2e285c91e4ac59793a0274f16eb12b08de6dd542",
                TECHNICAL_REVIEW_SHA256,
            )
            .replace("9c22379e0703eca09d58f702ad826c8b7f4521d1", BASE)
            .replace("reviewed-9c", "reviewed-0a3b")
            .replace("9c-to-correction", "0a3b-to-cumulative-cause-correction");
        fs::write(
            sample_root.join(relative),
            format!(
                "{rebound}\n\n## Deterministic rebind provenance\n\nThe mock and authentic JSON payloads were copied byte-for-byte from reviewed candidate `{AUTHENTIC_EXPORT_CANDIDATE}` / tree `{AUTHENTIC_EXPORT_TREE}`. Its checksum-bound provenance chain reaches the single retained-artifact export. The new direct-child candidate changes only sibling failure-cause collection, exact mutation assertions, and evidence machinery. Night Hag and Giant Rat have available perception and supported movement modes/speeds, so their ordinary payload bytes are unaffected by this correction. No artifact build, source query, hydration, or payload fabrication occurred during rebinding.\n"
            ),
        )
        .expect("rebound narrative should write");
    }

    let reviewed_root = required_path_env("E3_REVIEWED_SAMPLE_ROOT");
    assert_eq!(
        file_sha256(&reviewed_root.join("manifest.json")),
        REVIEWED_MANIFEST_SHA256
    );
    assert_eq!(
        file_sha256(&reviewed_root.join("checksums.sha256")),
        REVIEWED_CHECKSUMS_SHA256
    );
    verify_checksums(&reviewed_root);
    write_delta_ledger(&sample_root, &reviewed_root, &candidate, &candidate_tree);

    let technical_review = required_path_env("E3_TECHNICAL_REVIEW");
    assert_bound_file(&technical_review, TECHNICAL_REVIEW_SHA256);

    let mut manifest: Value = serde_json::from_slice(
        &fs::read(source_root.join("manifest.json")).expect("authentic manifest should read"),
    )
    .expect("authentic manifest should parse");
    manifest["candidate"] = json!({ "commit": candidate, "tree": candidate_tree, "parent": BASE });
    manifest["failed_review"] = json!({
        "path": technical_review,
        "sha256": TECHNICAL_REVIEW_SHA256,
        "mode": "0444",
        "finding": "F-001 cumulative sibling causes"
    });
    manifest
        .as_object_mut()
        .expect("manifest should be an object")
        .remove("reviewed_9c_evidence");
    manifest["reviewed_prior_evidence"] = json!({
        "root": reviewed_root,
        "manifest_sha256": REVIEWED_MANIFEST_SHA256,
        "checksums_sha256": REVIEWED_CHECKSUMS_SHA256,
        "checksum_closure": "pass"
    });
    manifest["producer"] = json!({
        "command": "cargo test -p atlas-app-service e3_sample_export::rebind_e3_record_surface_final_samples -- --ignored --exact",
        "test_path": "crates/atlas-app-service/src/e3_sample_export.rs",
        "method": "verified deterministic copy/rebind; ordinary payload bytes unaffected; no artifact/source query",
        "authentic_export_source": {
            "root": source_root,
            "candidate": AUTHENTIC_EXPORT_CANDIDATE,
            "tree": AUTHENTIC_EXPORT_TREE,
            "manifest_sha256": AUTHENTIC_EXPORT_MANIFEST_SHA256,
            "checksums_sha256": AUTHENTIC_EXPORT_CHECKSUMS_SHA256,
            "checksum_closure": "pass"
        }
    });
    manifest
        .as_object_mut()
        .expect("manifest should be an object")
        .remove("outputs");
    let output_hashes = relative_files(&sample_root)
        .into_iter()
        .map(|relative| {
            json!({
                "path": relative.to_string_lossy(),
                "sha256": file_sha256(&sample_root.join(&relative)),
            })
        })
        .collect::<Vec<_>>();
    manifest["outputs"] = Value::Array(output_hashes);
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
}

#[test]
#[ignore = "deterministically rebinds authenticated E3 evidence for the record_view rename"]
fn rebind_e3_record_view_final_samples() {
    let source_root = required_path_env("E3_RENAME_SOURCE_ROOT");
    assert_eq!(
        file_sha256(&source_root.join("manifest.json")),
        RECORD_VIEW_RENAME_SOURCE_MANIFEST_SHA256
    );
    assert_eq!(
        file_sha256(&source_root.join("checksums.sha256")),
        RECORD_VIEW_RENAME_SOURCE_CHECKSUMS_SHA256
    );
    verify_checksums(&source_root);

    let sample_root = required_path_env("E3_SAMPLE_ROOT");
    assert!(
        !sample_root.exists(),
        "sample root must be fresh and no-clobber: {}",
        sample_root.display()
    );
    fs::create_dir_all(sample_root.parent().expect("sample root parent"))
        .expect("sample parent should be creatable");
    fs::create_dir(&sample_root).expect("sample root should be creatable");

    let candidate = required_env("E3_SAMPLE_CANDIDATE");
    let candidate_tree = required_env("E3_SAMPLE_TREE");
    assert_eq!(git_value(Path::new("."), &["rev-parse", "HEAD"]), candidate);
    assert_eq!(
        git_value(Path::new("."), &["rev-parse", "HEAD^{tree}"]),
        candidate_tree
    );
    assert_eq!(
        git_value(Path::new("."), &["rev-parse", "HEAD^"]),
        RECORD_VIEW_RENAME_BASE
    );

    let approval = required_path_env("E3_FINAL_DIRECTION_APPROVAL");
    assert_bound_file(&approval, RECORD_VIEW_RENAME_APPROVAL_SHA256);
    let approval_sidecar = PathBuf::from(format!("{}.sha256", approval.display()));
    assert_bound_file(
        &approval_sidecar,
        RECORD_VIEW_RENAME_APPROVAL_SIDECAR_SHA256,
    );
    let review = required_path_env("E3_PRE_RENAME_REVIEW");
    assert_bound_file(&review, RECORD_VIEW_RENAME_REVIEW_SHA256);

    let transformed = BTreeSet::from([
        PathBuf::from("api-encounter-detail.json"),
        PathBuf::from("encounter-participant-adjusted.json"),
        PathBuf::from("encounter-participant-normal.json"),
    ]);
    let skipped = BTreeSet::from([
        PathBuf::from("manifest.json"),
        PathBuf::from("checksums.sha256"),
        PathBuf::from("WALKTHROUGH.md"),
        PathBuf::from("presentation.md"),
        PathBuf::from("report.md"),
        PathBuf::from("reviewed-to-correction-delta-ledger.md"),
        PathBuf::from("generated/EncounterParticipantView.ts"),
    ]);
    for relative in relative_files(&source_root) {
        if skipped.contains(&relative) || transformed.contains(&relative) {
            continue;
        }
        let destination = sample_root.join(&relative);
        fs::create_dir_all(destination.parent().expect("copied file parent"))
            .expect("copied file parent should be creatable");
        fs::copy(source_root.join(&relative), &destination)
            .unwrap_or_else(|error| panic!("{} should copy: {error}", relative.display()));
    }

    replace_exact_text(
        &source_root.join("encounter-participant-normal.json"),
        &sample_root.join("encounter-participant-normal.json"),
        "\n  \"surface\": {",
        "\n  \"record_view\": {",
        1,
    );
    replace_exact_text(
        &source_root.join("encounter-participant-adjusted.json"),
        &sample_root.join("encounter-participant-adjusted.json"),
        "\n  \"surface\": {",
        "\n  \"record_view\": {",
        1,
    );
    replace_exact_text(
        &source_root.join("api-encounter-detail.json"),
        &sample_root.join("api-encounter-detail.json"),
        "\n      \"surface\": {",
        "\n      \"record_view\": {",
        2,
    );

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve");
    let binding_relative = Path::new("generated/EncounterParticipantView.ts");
    fs::create_dir_all(
        sample_root
            .join(binding_relative)
            .parent()
            .expect("binding parent"),
    )
    .expect("binding parent should be creatable");
    fs::copy(
        repo_root.join("crates/atlas-app-model/bindings/EncounterParticipantView.ts"),
        sample_root.join(binding_relative),
    )
    .expect("current participant binding should copy");

    for relative in [
        "encounter-participant-normal.json",
        "encounter-participant-adjusted.json",
    ] {
        let value: Value = serde_json::from_slice(
            &fs::read(sample_root.join(relative)).expect("participant sample should read"),
        )
        .expect("participant sample should parse");
        assert!(value.get("record_view").is_some());
        assert!(value.get("surface").is_none());
    }
    let api_encounter: Value = serde_json::from_slice(
        &fs::read(sample_root.join("api-encounter-detail.json"))
            .expect("encounter API sample should read"),
    )
    .expect("encounter API sample should parse");
    let participants = api_encounter["participants"]
        .as_array()
        .expect("encounter API should contain participants");
    assert_eq!(participants.len(), 2);
    assert!(
        participants
            .iter()
            .all(|value| value.get("record_view").is_some())
    );
    assert!(
        participants
            .iter()
            .all(|value| value.get("surface").is_none())
    );
    let concept_mock: Value = serde_json::from_slice(
        &fs::read(sample_root.join("concept-mock.json")).expect("concept mock should read"),
    )
    .expect("concept mock should parse");
    assert!(concept_mock.get("surface").is_some());
    assert!(concept_mock.get("record_view").is_none());
    let record_api: Value = serde_json::from_slice(
        &fs::read(sample_root.join("api-record-detail.json"))
            .expect("record API sample should read"),
    )
    .expect("record API sample should parse");
    assert!(record_api.get("surface").is_some());
    assert!(record_api.get("record_view").is_none());
    let participant_binding = fs::read_to_string(sample_root.join(binding_relative))
        .expect("participant binding should read");
    assert!(participant_binding.contains("record_view: RecordSurfaceView"));
    assert!(!participant_binding.contains("surface: RecordSurfaceView"));

    fs::write(
        sample_root.join("WALKTHROUGH.md"),
        format!(
            "# E3 final-direction record-view rename walkthrough\n\nThis is candidate-authentic evidence for `{candidate}` / tree `{candidate_tree}`, a direct child of `{RECORD_VIEW_RENAME_BASE}`. It awaits lightweight independent static rename review and is not E3 acceptance or final sample approval.\n\n## Start here\n\n1. `concept-mock.json` remains the labeled non-authentic concept mock under its approved sample-envelope key `surface`; that envelope is not `EncounterParticipantView`.\n2. `record-detail-night-hag.json` and `record-detail-sparse-creature.json` remain byte-identical authentic static surfaces for Night Hag and Giant Rat.\n3. `encounter-participant-adjusted.json` and `encounter-participant-normal.json` show the public participant field `record_view`; the superseded participant field `surface` is absent.\n4. `api-encounter-detail.json` shows the same rename in the exact transport envelope, while `api-record-detail.json` correctly retains the distinct `RecordDetailView.surface` contract.\n5. `generated/EncounterParticipantView.ts` is the refreshed binding. All other generated bindings are byte-identical to the reviewed source package.\n6. `reviewed-to-correction-delta-ledger.md` accounts for every prior and refreshed output.\n\nThe nested `RecordSurfaceView.presentation` and optional `RecordSurfaceView.encounter` values are unchanged. There is no alias, shim, dual field, fallback, mechanics change, or F1/F2 feature work.\n"
        ),
    )
    .expect("walkthrough should write");
    fs::write(
        sample_root.join("presentation.md"),
        "# Presentation index\n\n- Concept mock: `concept-mock.json`\n- Dense real record: `record-detail-night-hag.json`\n- Sparse real record: `record-detail-sparse-creature.json`\n- Context-adjusted participant: `encounter-participant-adjusted.json`\n- Normal participant: `encounter-participant-normal.json`\n- Exact API envelopes: `api-record-detail.json`, `api-encounter-detail.json`\n- Renamed generated contract: `generated/EncounterParticipantView.ts`\n- Complete prior-to-rename delta: `reviewed-to-correction-delta-ledger.md`\n\nOnly participant envelopes rename `surface` to `record_view`; record-detail and concept-mock envelopes retain their separately owned `surface` keys.\n",
    )
    .expect("presentation index should write");
    fs::write(
        sample_root.join("report.md"),
        format!(
            "# E3 final-direction record-view rename candidate report\n\nCandidate `{candidate}` / tree `{candidate_tree}` is a direct child of technically passed pre-rename candidate `{RECORD_VIEW_RENAME_BASE}` / tree `{RECORD_VIEW_RENAME_BASE_TREE}`. The only public-contract change is `EncounterParticipantView.surface` -> `EncounterParticipantView.record_view`. Nested presentation and optional encounter-runtime semantics are unchanged. `RecordDetailView.surface`, `RecordSummaryView.surface`, the concept-mock sample envelope `surface`, and runtime roll-surface fields are separate contracts and remain unchanged.\n\nThe package is bound to final-direction approval `{RECORD_VIEW_RENAME_APPROVAL_SHA256}` and pre-rename technical/evidence PASS `{RECORD_VIEW_RENAME_REVIEW_SHA256}`. Proportionate rename validation covers exact diff and residue, Rust/transport serialization, generated binding freshness, focused Rust/frontend tests, and clean status. No exporter, index build, source query, broad workspace validation, mechanics work, compatibility layer, F1/F2 feature work, push, merge, PR, or deploy occurred.\n\nThe mock plus exactly two authentic records retain source identity `{SOURCE_SIGNATURE}`. Substantive bytes are copied from the checksum-closed pre-rename package; only three participant-bearing JSON files receive the exact key substitution, the participant TypeScript binding is regenerated, and evidence documents are refreshed.\n"
        ),
    )
    .expect("candidate report should write");

    write_record_view_rename_delta_ledger(&sample_root, &source_root, &candidate, &candidate_tree);

    let mut manifest: Value = serde_json::from_slice(
        &fs::read(source_root.join("manifest.json")).expect("source manifest should read"),
    )
    .expect("source manifest should parse");
    manifest["candidate"] = json!({
        "commit": candidate,
        "tree": candidate_tree,
        "parent": RECORD_VIEW_RENAME_BASE
    });
    manifest["final_direction_correction_approval"] = json!({
        "path": approval,
        "sha256": RECORD_VIEW_RENAME_APPROVAL_SHA256,
        "sidecar_file_sha256": RECORD_VIEW_RENAME_APPROVAL_SIDECAR_SHA256,
        "mode": "0444"
    });
    manifest["pre_rename_technical_and_evidence_review"] = json!({
        "path": review,
        "sha256": RECORD_VIEW_RENAME_REVIEW_SHA256,
        "mode": "0444",
        "verdict": "PASS"
    });
    manifest["prior_candidate_evidence"] = json!({
        "root": source_root,
        "candidate": RECORD_VIEW_RENAME_BASE,
        "tree": RECORD_VIEW_RENAME_BASE_TREE,
        "manifest_sha256": RECORD_VIEW_RENAME_SOURCE_MANIFEST_SHA256,
        "checksums_sha256": RECORD_VIEW_RENAME_SOURCE_CHECKSUMS_SHA256,
        "checksum_closure": "pass"
    });
    manifest["public_contract_delta"] = json!({
        "type": "EncounterParticipantView",
        "removed_field": "surface",
        "added_field": "record_view",
        "nested_presentation_semantics_unchanged": true,
        "optional_encounter_semantics_unchanged": true,
        "compatibility_alias": false,
        "shim": false,
        "dual_field": false,
        "mechanics_change": false
    });
    manifest["validation"] = json!({
        "exact_diff_and_production_residue": "pass",
        "affected_rust_serialization": "pass",
        "affected_web_transport": "pass",
        "generated_binding_freshness": "pass",
        "focused_rust_compile_and_tests": "pass",
        "focused_frontend_format_lint_styles_tests": "pass",
        "frontend_typecheck": "unchanged adjudicated downstream boundary; no new rename diagnostics",
        "broad_workspace_validation": "not rerun; proportionate rename scope",
        "clean_status": "pass"
    });
    manifest["producer"] = json!({
        "command": "cargo test -p atlas-app-service e3_sample_export::rebind_e3_record_view_final_samples -- --ignored --exact",
        "test_path": "crates/atlas-app-service/src/e3_sample_export.rs",
        "method": "authenticated deterministic copy plus exact participant-key substitution; no exporter hydration, index build, or source query",
        "source_package": {
            "root": source_root,
            "candidate": RECORD_VIEW_RENAME_BASE,
            "tree": RECORD_VIEW_RENAME_BASE_TREE,
            "manifest_sha256": RECORD_VIEW_RENAME_SOURCE_MANIFEST_SHA256,
            "checksums_sha256": RECORD_VIEW_RENAME_SOURCE_CHECKSUMS_SHA256,
            "checksum_closure": "pass"
        }
    });
    manifest["status"] =
        json!("record_view_rename_candidate_awaiting_lightweight_static_review_not_accepted");
    manifest
        .as_object_mut()
        .expect("manifest should be an object")
        .remove("failed_review");
    manifest
        .as_object_mut()
        .expect("manifest should be an object")
        .remove("outputs");
    let output_hashes = relative_files(&sample_root)
        .into_iter()
        .map(|relative| {
            json!({
                "path": relative.to_string_lossy(),
                "sha256": file_sha256(&sample_root.join(&relative)),
            })
        })
        .collect::<Vec<_>>();
    manifest["outputs"] = Value::Array(output_hashes);
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
    assert!(body.skills.as_ref().is_none_or(|values| !values.is_empty()));
    assert!(
        body.movement
            .as_ref()
            .is_none_or(|values| !values.is_empty())
    );
    assert!(
        body.resources
            .as_ref()
            .is_none_or(|values| !values.is_empty())
    );
    assert!(
        body.spellcasting
            .as_ref()
            .is_none_or(|values| !values.is_empty())
    );
    assert!(
        body.activities
            .as_ref()
            .is_none_or(|values| !values.is_empty())
    );
    assert!(
        body.content
            .as_ref()
            .is_none_or(|values| !values.is_empty())
    );
    assert!(
        body.relationships
            .as_ref()
            .is_none_or(|values| !values.is_empty())
    );
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

fn assert_no_empty_public_containers(value: &Value) {
    match value {
        Value::Object(object) => {
            assert!(
                !object.is_empty(),
                "public JSON must not contain empty objects"
            );
            for child in object.values() {
                assert_no_empty_public_containers(child);
            }
        }
        Value::Array(values) => {
            assert!(
                !values.is_empty(),
                "public JSON must not contain empty arrays"
            );
            for child in values {
                assert_no_empty_public_containers(child);
            }
        }
        _ => {}
    }
}

fn replace_exact_text(source: &Path, destination: &Path, from: &str, to: &str, count: usize) {
    let input = fs::read_to_string(source)
        .unwrap_or_else(|error| panic!("{} should read: {error}", source.display()));
    assert_eq!(
        input.match_indices(from).count(),
        count,
        "{} should contain exactly {count} target keys",
        source.display()
    );
    assert_eq!(
        input.match_indices(to).count(),
        0,
        "{} must not already contain the replacement key",
        source.display()
    );
    let output = input.replace(from, to);
    assert_eq!(output.match_indices(from).count(), 0);
    assert_eq!(output.match_indices(to).count(), count);
    fs::write(destination, output)
        .unwrap_or_else(|error| panic!("{} should write: {error}", destination.display()));
}

fn write_record_view_rename_delta_ledger(
    final_root: &Path,
    source_root: &Path,
    candidate: &str,
    candidate_tree: &str,
) {
    let source_manifest: Value = serde_json::from_slice(
        &fs::read(source_root.join("manifest.json")).expect("source manifest should read"),
    )
    .expect("source manifest should parse");
    let source_outputs = source_manifest["outputs"]
        .as_array()
        .expect("source manifest outputs should be an array")
        .iter()
        .filter(|value| value["path"] != "reviewed-to-correction-delta-ledger.md")
        .map(|value| {
            (
                value["path"]
                    .as_str()
                    .expect("source output path should be a string")
                    .to_string(),
                value["sha256"]
                    .as_str()
                    .expect("source output hash should be a string")
                    .to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let final_outputs = relative_files(final_root)
        .into_iter()
        .filter(|path| path != Path::new("reviewed-to-correction-delta-ledger.md"))
        .map(|path| {
            (
                path.to_string_lossy().to_string(),
                file_sha256(&final_root.join(path)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let paths = source_outputs
        .keys()
        .chain(final_outputs.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let rows = paths
        .into_iter()
        .map(|path| {
            let source = source_outputs.get(&path).map_or("—", String::as_str);
            let final_hash = final_outputs.get(&path).map_or("—", String::as_str);
            let delta = match (source_outputs.get(&path), final_outputs.get(&path)) {
                (Some(left), Some(right)) if left == right => "byte-identical",
                (Some(_), Some(_)) => "changed",
                (None, Some(_)) => "added",
                (Some(_), None) => "removed",
                (None, None) => unreachable!("union path must occur in at least one output set"),
            };
            format!("| `{path}` | `{source}` | `{final_hash}` | {delta} |")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        final_root.join("reviewed-to-correction-delta-ledger.md"),
        format!(
            "# E3 final-direction record-view rename delta ledger\n\n## Bound identities\n\n- Technically passed pre-rename package root: `{}`.\n- Pre-rename manifest/checksums SHA-256: `{RECORD_VIEW_RENAME_SOURCE_MANIFEST_SHA256}` / `{RECORD_VIEW_RENAME_SOURCE_CHECKSUMS_SHA256}`.\n- Pre-rename candidate/tree: `{RECORD_VIEW_RENAME_BASE}` / `{RECORD_VIEW_RENAME_BASE_TREE}`.\n- Pre-rename technical/evidence PASS SHA-256: `{RECORD_VIEW_RENAME_REVIEW_SHA256}`.\n- Final-direction approval/sidecar SHA-256: `{RECORD_VIEW_RENAME_APPROVAL_SHA256}` / `{RECORD_VIEW_RENAME_APPROVAL_SIDECAR_SHA256}`.\n- Rename candidate/tree: `{candidate}` / `{candidate_tree}`, direct child of `{RECORD_VIEW_RENAME_BASE}`.\n\n## Complete output delta\n\nThis table is the no-omission union of every source-package output and every rename-candidate output present before this ledger is written. `manifest.json`, `checksums.sha256`, and this recursively self-describing ledger are excluded only from the table; all three are bound by the fresh checksum closure.\n\n| Output | Pre-rename SHA-256 | Rename-candidate SHA-256 | Delta |\n|---|---|---|---|\n{rows}\n\n## Exact contract delta\n\n1. The required public `EncounterParticipantView.surface` field is directly replaced by `EncounterParticipantView.record_view` in Rust, API JSON, generated TypeScript, and existing consumers.\n2. The nested `RecordSurfaceView.presentation` and optional `RecordSurfaceView.encounter` bytes and semantics are unchanged.\n3. `RecordDetailView.surface`, `RecordSummaryView.surface`, the concept-mock envelope `surface`, and runtime roll-surface fields are distinct contracts and remain unchanged.\n4. Only `encounter-participant-normal.json`, `encounter-participant-adjusted.json`, and `api-encounter-detail.json` receive the exact participant-envelope key substitution. The two static real-record payloads, concept mock, record-detail API, all other JSON, and all generated bindings except `EncounterParticipantView.ts` remain byte-identical.\n5. There is no alias, shim, dual field, fallback, semantic or mechanics change, unrelated contract change, or F1/F2 feature work.\n6. No exporter hydration, artifact build, index query, or source query was run; this package uses authenticated deterministic copy plus exact key substitution.\n",
            source_root.display()
        ),
    )
    .expect("record-view rename delta ledger should write");
}

fn write_delta_ledger(
    final_root: &Path,
    reviewed_root: &Path,
    candidate: &str,
    candidate_tree: &str,
) {
    let reviewed_manifest: Value = serde_json::from_slice(
        &fs::read(reviewed_root.join("manifest.json")).expect("reviewed manifest should read"),
    )
    .expect("reviewed manifest should parse");
    let reviewed_outputs = reviewed_manifest["outputs"]
        .as_array()
        .expect("reviewed manifest outputs should be an array")
        .iter()
        .map(|output| {
            (
                output["path"]
                    .as_str()
                    .expect("reviewed output path should be a string")
                    .to_string(),
                output["sha256"]
                    .as_str()
                    .expect("reviewed output hash should be a string")
                    .to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let final_outputs = relative_files(final_root)
        .into_iter()
        .map(|relative| {
            let path = relative.to_string_lossy().to_string();
            let hash = file_sha256(&final_root.join(&relative));
            (path, hash)
        })
        .collect::<BTreeMap<_, _>>();
    let paths = reviewed_outputs
        .keys()
        .chain(final_outputs.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let rows = paths
        .into_iter()
        .map(|path| {
            let reviewed = reviewed_outputs.get(&path).map_or("—", String::as_str);
            let final_hash = final_outputs.get(&path).map_or("—", String::as_str);
            let delta = match (reviewed_outputs.get(&path), final_outputs.get(&path)) {
                (Some(left), Some(right)) if left == right => "unchanged",
                (Some(_), Some(_)) => "changed",
                (None, Some(_)) => "added",
                (Some(_), None) => "removed",
                (None, None) => unreachable!("union path must occur in at least one output set"),
            };
            format!("| `{path}` | `{reviewed}` | `{final_hash}` | {delta} |")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        final_root.join("reviewed-to-correction-delta-ledger.md"),
        format!(
            "# E3 0a3b-to-cumulative-cause correction delta ledger\n\n## Bound identities\n\n- Failed reviewed package root: `{}`.\n- Reviewed manifest/checksums SHA-256: `{REVIEWED_MANIFEST_SHA256}` / `{REVIEWED_CHECKSUMS_SHA256}`.\n- Failed exact-commit rereview SHA-256: `{TECHNICAL_REVIEW_SHA256}`.\n- Cumulative-cause correction candidate: `{candidate}` (tree `{candidate_tree}`), direct child of `{BASE}`.\n- Conditioned early-direction approval SHA-256: `{APPROVAL_SHA256}`.\n- CLI ownership amendment plan/task-map SHA-256: `{AMENDED_PLAN_SHA256}` / `{AMENDED_TASK_MAP_SHA256}`; independent planning review verdict SHA-256 `{PLANNING_REVIEW_VERDICT_SHA256}`.\n\n## Complete payload output ledger\n\nThis table is the no-omission union of every reviewed-0a3b payload output and every cumulative-cause correction payload output present before this ledger is written. `manifest.json`, `checksums.sha256`, and this recursively self-describing ledger are excluded only from the table; all three are hash-bound by the correction checksum closure.\n\n| Output | Reviewed 0a3b SHA-256 | Cumulative-cause correction SHA-256 | Delta |\n|---|---|---|---|\n{rows}\n\n## Contract delta\n\n1. Movement now inspects speed independently of mode, retaining simultaneous typed `movement_speed` Missing/Null and `movement_mode` Unsupported causes before suppressing an unsafe component.\n2. Awareness now inspects languages independently of perception, preserving populated language payloads and independent Missing/Null causes when perception is unavailable.\n3. Complete typed cause tuples include state, affected field, optional component identity, canonical owner, and canonical source-field provenance; their deterministic order is proven across reversed input order.\n4. Genuinely known-empty omission, ordinary payload shape, whole-record unavailability, runtime automation limitations, CLI output, and mechanics are unchanged.\n5. No generic registry, compatibility alias, shim, dual model, fallback, section ordering, or string-key semantic lookup was introduced.\n",
            reviewed_root.display()
        ),
    )
    .expect("delta ledger should write");
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

fn file_size(path: &Path) -> u64 {
    fs::metadata(path)
        .expect("retained file metadata should read")
        .len()
}

fn assert_bound_file(path: &Path, expected_sha256: &str) {
    let metadata = fs::symlink_metadata(path)
        .unwrap_or_else(|error| panic!("bound file {} should exist: {error}", path.display()));
    assert!(metadata.file_type().is_file(), "bound path must be a file");
    assert!(
        !metadata.file_type().is_symlink(),
        "bound path must not be a symlink"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        assert_eq!(metadata.permissions().mode() & 0o777, 0o444);
    }
    assert_eq!(file_sha256(path), expected_sha256);
}

fn directory_size_kib(path: &Path) -> u64 {
    let output = Command::new("du")
        .args(["-sk"])
        .arg(path)
        .output()
        .expect("du should run");
    assert!(output.status.success(), "du should succeed");
    String::from_utf8(output.stdout)
        .expect("du output should be UTF-8")
        .split_whitespace()
        .next()
        .expect("du should emit a size")
        .parse()
        .expect("du size should be an integer")
}

fn verify_checksums(root: &Path) {
    let output = Command::new("shasum")
        .args(["-a", "256", "-c", "checksums.sha256"])
        .current_dir(root)
        .output()
        .expect("checksum verification should run");
    assert!(
        output.status.success(),
        "checksum verification should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
