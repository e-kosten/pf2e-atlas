use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use atlas_ingest::{
    CoverageFailureCode, PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT,
    PF2E_SOURCE_PINNED_SIGNATURE, capture_registered_source_leaf_receipt,
    evaluate_source_leaf_coverage, lint_source_leaf_ledger, parse_source_leaf_ledger,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const LEDGER: &str = include_str!("../../../contracts/source-leaf-coverage/v1/actor-npc.yaml");
const FIXTURE_ROOT: &str = "tests/fixtures/source-leaf-coverage/actor-npc";

#[derive(Debug, Deserialize)]
struct ActorFixtureManifest {
    source_contract_version: String,
    source_commit: String,
    source_signature: String,
    validator_negatives: Vec<ValidatorNegativeFixture>,
}

#[derive(Debug, Deserialize)]
struct ValidatorNegativeFixture {
    case_id: String,
    record_key: String,
    source_path: String,
    source_file_sha256: String,
    excerpt_path: String,
    excerpt_sha256: String,
    attempted_path: String,
    expected_failure: String,
}

#[derive(Debug, Deserialize)]
struct PinnedSystemManifest {
    packs: Vec<PinnedManifestPack>,
}

#[derive(Debug, Deserialize)]
struct PinnedManifestPack {
    name: String,
    path: String,
    #[serde(rename = "type")]
    document_class: String,
}

struct AuthenticatedNegative {
    document_class: String,
    type_discriminator: String,
}

fn require_pinned_repository() -> PathBuf {
    let repository = std::env::var_os("PF2E_SOURCE_REPOSITORY")
        .map(PathBuf::from)
        .expect("TEST PREREQUISITE: set PF2E_SOURCE_REPOSITORY to the accepted PF2E checkout");
    let accepted_commit = "4cbdaa37d6c33e9519561bae2c59a23e0288cbce^{commit}";
    let output = Command::new("git")
        .args(["-C"])
        .arg(&repository)
        .args(["cat-file", "-e", accepted_commit])
        .output()
        .expect("TEST PREREQUISITE: git must inspect PF2E_SOURCE_REPOSITORY");
    assert!(
        output.status.success(),
        "TEST PREREQUISITE: repository must contain {accepted_commit}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    repository
}

fn git_show(repository: &Path, path: &str) -> Vec<u8> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repository)
        .args(["show", &format!("{PF2E_SOURCE_PINNED_COMMIT}:{path}")])
        .output()
        .expect("git must read the accepted pinned source tree");
    assert!(
        output.status.success(),
        "pinned source path {path} must resolve: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn authenticate_validator_negative(
    repository: &Path,
    system_manifest: &PinnedSystemManifest,
    fixture_root: &Path,
    fixture: &ValidatorNegativeFixture,
) -> AuthenticatedNegative {
    let source_bytes = git_show(repository, &fixture.source_path);
    assert_eq!(
        format!("{:x}", Sha256::digest(&source_bytes)),
        fixture.source_file_sha256,
        "{} source blob digest",
        fixture.case_id
    );
    let source: serde_json::Value =
        serde_json::from_slice(&source_bytes).expect("pinned negative source is JSON");
    let pack = system_manifest
        .packs
        .iter()
        .find(|pack| {
            fixture
                .source_path
                .strip_prefix(&format!("{}/", pack.path.trim_end_matches('/')))
                .is_some_and(|relative| !relative.is_empty())
        })
        .expect("negative source path belongs to a pinned manifest pack");
    let source_id = source["_id"]
        .as_str()
        .expect("negative source has a string _id");
    assert_eq!(fixture.record_key, format!("{}:{source_id}", pack.name));

    let excerpt_bytes =
        std::fs::read(fixture_root.join(&fixture.excerpt_path)).expect("negative excerpt exists");
    let excerpt: serde_json::Value =
        serde_json::from_slice(&excerpt_bytes).expect("negative excerpt is JSON");
    let canonical_excerpt = serde_json::to_vec(&excerpt).expect("canonical negative excerpt");
    assert_eq!(
        format!("{:x}", Sha256::digest(canonical_excerpt)),
        fixture.excerpt_sha256,
        "{} excerpt digest",
        fixture.case_id
    );
    assert_eq!(excerpt["_id"], source["_id"]);
    assert_eq!(excerpt["type"], source["type"]);

    AuthenticatedNegative {
        document_class: pack.document_class.clone(),
        type_discriminator: source["type"]
            .as_str()
            .expect("negative source has a string type")
            .to_string(),
    }
}

#[test]
fn actor_npc_ledger_is_exact_and_source_grounded() {
    let ledger = parse_source_leaf_ledger(LEDGER).expect("Actor NPC A2 ledger parses");
    assert_eq!(ledger.type_id, "actor--npc--top-level--root--root--root");
    assert_eq!(ledger.leaves.len(), 7);
    assert_eq!(
        ledger
            .leaves
            .iter()
            .map(|leaf| leaf.normalized_path.as_str())
            .collect::<Vec<_>>(),
        [
            "$.system.abilities.str.mod",
            "$.system.abilities.dex.mod",
            "$.system.abilities.con.mod",
            "$.system.abilities.int.mod",
            "$.system.abilities.wis.mod",
            "$.system.abilities.cha.mod",
            "$.system.skills.*.base",
        ]
    );
    assert!(
        lint_source_leaf_ledger(&ledger).is_empty(),
        "{:#?}",
        lint_source_leaf_ledger(&ledger)
    );

    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT);
    for (path, expected) in [
        (
            "excerpts/night-hag.json",
            "393888a0626eda2c050251ac008405ff96a793135cba038aacb83b5d44480256",
        ),
        (
            "excerpts/giant-rat.json",
            "fd53bab318359b32e567363a72b8984118882ab6aebafb89e36bf308613c7245",
        ),
        (
            "excerpts/karumzek-priest.json",
            "90211361583ce185b51a6b0c8b7509291ae9fea0bd453a11ad3c47843ea9facb",
        ),
        (
            "excerpts/inkdrop.json",
            "494090d798933b84380554d66aff86fa705a984c265c3cf5c16382be6f31111a",
        ),
        (
            "excerpts/gray-master.json",
            "5c856483c6f3eec58ce754dadfd91d23f98df1b57ea631a63ad6dc98e7bc9b19",
        ),
        (
            "excerpts/chernasardo-ranger.json",
            "5038083d837b95b133ca01538f814ba582b599bb788d5f6425ec25eb24b791bd",
        ),
        (
            "validator-negatives/trait-prefix-volluk.json",
            "dc0ad56accf422e76c2fd2d6290ffd8afcd0893596172cf33afec748da36cd12",
        ),
        (
            "validator-negatives/spell-prefix-anadi-lore.json",
            "2f044e9c559bfa3ec96c82be74450a3c4f9ba56c105f97020e5a442af806a1b8",
        ),
    ] {
        let bytes = std::fs::read(fixture_root.join(path)).expect("checked-in source excerpt");
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("source excerpt is JSON");
        let canonical = serde_json::to_vec(&value).expect("canonical excerpt JSON");
        assert_eq!(
            format!("{:x}", Sha256::digest(canonical)),
            expected,
            "{path}"
        );
    }

    let ledger_paths = ledger
        .leaves
        .iter()
        .flat_map(|leaf| leaf.fixtures.iter())
        .map(|fixture| fixture.source_path.as_str())
        .collect::<BTreeSet<_>>();
    assert!(
        !ledger_paths
            .iter()
            .any(|path| path.contains("volluk-azrinae"))
    );
    assert!(!ledger_paths.iter().any(|path| path.contains("anadi-lore")));
}

#[test]
fn current_creature_pipeline_fails_exact_promoted_owners() {
    let repository = require_pinned_repository();
    let ledger = parse_source_leaf_ledger(LEDGER).expect("Actor NPC A2 ledger parses");
    let mut receipts = Vec::new();
    for (leaf_index, leaf) in ledger.leaves.iter().enumerate() {
        for fixture_index in 0..leaf.fixtures.len() {
            receipts.push(
                capture_registered_source_leaf_receipt(
                    &ledger,
                    leaf_index,
                    fixture_index,
                    &repository,
                )
                .expect("pinned production-pipeline receipt"),
            );
        }
    }
    let report = evaluate_source_leaf_coverage(&ledger, &receipts);
    assert!(
        !report.passed,
        "A2 must expose current mismatches, not repair B1"
    );
    assert_eq!(report.receipt_count, 21);
    let codes = report
        .failures
        .iter()
        .map(|failure| failure.code)
        .collect::<BTreeSet<_>>();
    for expected in [
        CoverageFailureCode::DtoMismatch,
        CoverageFailureCode::CanonicalMismatch,
        CoverageFailureCode::PostProjectionMismatch,
        CoverageFailureCode::ArtifactHydrationMismatch,
        CoverageFailureCode::PublicSurfaceMismatch,
    ] {
        assert!(
            codes.contains(&expected),
            "missing {expected:?}: {:#?}",
            report.failures
        );
    }

    let has_failure = |path: &str, code: CoverageFailureCode| {
        report.failures.iter().any(|failure| {
            failure.code == code
                && failure
                    .identity
                    .as_ref()
                    .is_some_and(|identity| identity.normalized_path == path)
        })
    };
    assert!(has_failure(
        "$.system.abilities.str.mod",
        CoverageFailureCode::DtoMismatch
    ));
    for code in [
        CoverageFailureCode::CanonicalMismatch,
        CoverageFailureCode::PostProjectionMismatch,
        CoverageFailureCode::ArtifactHydrationMismatch,
    ] {
        assert!(
            has_failure("$.system.abilities.str.mod", code),
            "a transient metric or container cannot satisfy the {code:?} leaf owner"
        );
    }
    assert!(has_failure(
        "$.system.skills.*.base",
        CoverageFailureCode::PublicSurfaceMismatch
    ));

    let mut broad_parent = ledger.clone();
    broad_parent.leaves[0].normalized_path = "$.system.abilities.*".to_string();
    assert!(
        lint_source_leaf_ledger(&broad_parent)
            .iter()
            .any(|failure| failure.code == CoverageFailureCode::BroadDeclaration),
        "a broad parent must not close the exact .mod leaf"
    );
}

#[test]
fn validator_negatives_authenticate_and_execute_expected_typed_failures() {
    let repository = require_pinned_repository();
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT);
    let fixture_manifest: ActorFixtureManifest = yaml_serde::from_str(
        &std::fs::read_to_string(fixture_root.join("manifest.yaml"))
            .expect("Actor fixture manifest exists"),
    )
    .expect("Actor fixture manifest parses");
    assert_eq!(
        fixture_manifest.source_contract_version,
        PF2E_SOURCE_CONTRACT_VERSION
    );
    assert_eq!(fixture_manifest.source_commit, PF2E_SOURCE_PINNED_COMMIT);
    assert_eq!(
        fixture_manifest.source_signature,
        PF2E_SOURCE_PINNED_SIGNATURE
    );
    assert_eq!(fixture_manifest.validator_negatives.len(), 2);

    let system_manifest: PinnedSystemManifest =
        serde_json::from_slice(&git_show(&repository, "static/system.json"))
            .expect("pinned system manifest parses");
    let base_ledger = parse_source_leaf_ledger(LEDGER).expect("Actor NPC A2 ledger parses");

    for fixture in &fixture_manifest.validator_negatives {
        let identity =
            authenticate_validator_negative(&repository, &system_manifest, &fixture_root, fixture);
        let mut negative = base_ledger.clone();
        negative.leaves.truncate(1);
        negative.leaves[0].normalized_path = fixture.attempted_path.clone();
        negative.selector.document_class = identity.document_class;
        negative.selector.type_discriminator = identity.type_discriminator;
        let failures = lint_source_leaf_ledger(&negative);
        let expected = match fixture.expected_failure.as_str() {
            "broad_declaration" => CoverageFailureCode::BroadDeclaration,
            "registry_binding_mismatch" => CoverageFailureCode::RegistryBindingMismatch,
            other => panic!("unrecognized validator-negative failure {other}"),
        };
        assert!(
            failures.iter().any(|failure| failure.code == expected),
            "{} must execute {expected:?}: {failures:#?}",
            fixture.case_id
        );
    }
}
