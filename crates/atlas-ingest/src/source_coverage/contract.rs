use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE,
};

use super::registry::accepted_registry;
use super::{CoverageFailure, CoverageFailureCode};

pub const ATLAS_SOURCE_LEAF_COVERAGE_VERSION: &str = "atlas-source-leaf-coverage/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafCoverageLedger {
    pub contract_version: String,
    pub type_id: String,
    pub source_pin: SourcePin,
    pub selector: SourceLeafSelector,
    pub leaves: Vec<SourceLeafContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePin {
    pub upstream_commit: String,
    pub source_signature: String,
}

impl SourcePin {
    pub fn pinned() -> Self {
        Self {
            upstream_commit: PF2E_SOURCE_PINNED_COMMIT.to_string(),
            source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafSelector {
    pub source_contract_version: String,
    pub document_class: String,
    pub type_discriminator: String,
    pub role: SourceDocumentRole,
    pub parent_context: SourceParentContextSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceParentContextSelector {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_discriminator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship_path: Option<String>,
}

impl SourceParentContextSelector {
    pub const fn root() -> Self {
        Self {
            document_class: None,
            type_discriminator: None,
            relationship_path: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDocumentRole {
    TopLevel,
    Embedded,
    Child,
    Generated,
    Container,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafContract {
    pub normalized_path: String,
    pub expected_shapes: Vec<ExpectedSourceShape>,
    pub source_prevalence: SourcePrevalence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_key_policy: Option<MapKeyPolicy>,
    pub disposition: SourceLeafDisposition,
    pub reader: ReaderContract,
    pub fixtures: Vec<FixtureContract>,
    pub final_owners: Vec<FinalOwnerContract>,
    pub surfaces: SurfaceContract,
    pub rationale: String,
    pub owner: String,
    pub acceptance_checkpoint: String,
    #[serde(default)]
    pub future_owner: Option<String>,
    #[serde(default)]
    pub future_task: Option<String>,
    #[serde(default)]
    pub prerequisite: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedSourceShape {
    Missing,
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
    ArrayMember,
    MapMember,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePrevalence {
    pub record_count: usize,
    pub occurrence_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MapKeyPolicy {
    ClosedVocabulary { keys: Vec<String> },
    OpenVocabularyRetainedIdentity,
    TypedUnsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLeafDisposition {
    Promoted,
    ProvenanceOnly,
    Ignored,
    Deferred,
    Unconsumed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReaderContract {
    #[serde(default)]
    pub reader_id: Option<String>,
    #[serde(default)]
    pub parity_case_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureContract {
    pub case_id: String,
    pub record_key: String,
    pub source_path: String,
    pub source_file_digest: String,
    pub excerpt_digest: String,
    pub provenance: FixtureProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureProvenance {
    PinnedSource,
    ContractOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalOwnerContract {
    pub stage: FinalOwnerStage,
    pub destination: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalOwnerStage {
    SourceDto,
    Canonical,
    PostProjection,
    ArtifactHydration,
    PublicSurface,
    DurableProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceDecision {
    pub disposition: SurfaceDisposition,
    pub rationale: String,
}

impl Default for SurfaceDecision {
    fn default() -> Self {
        Self {
            disposition: SurfaceDisposition::NotApplicable,
            rationale: "not selected for this surface".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceDisposition {
    Promoted,
    NotApplicable,
    Deferred,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceContract {
    pub artifact: SurfaceDecision,
    pub search: SurfaceDecision,
    pub cli: SurfaceDecision,
    pub app: SurfaceDecision,
    pub ui: SurfaceDecision,
    pub runtime: SurfaceDecision,
}

impl SurfaceContract {
    pub(crate) fn decisions(&self) -> [&SurfaceDecision; 6] {
        [
            &self.artifact,
            &self.search,
            &self.cli,
            &self.app,
            &self.ui,
            &self.runtime,
        ]
    }

    pub(crate) fn artifact_is_promoted(&self) -> bool {
        self.artifact.disposition == SurfaceDisposition::Promoted
    }

    pub(crate) fn public_surface_is_promoted(&self) -> bool {
        [&self.search, &self.cli, &self.app, &self.ui, &self.runtime]
            .into_iter()
            .any(|decision| decision.disposition == SurfaceDisposition::Promoted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafIdentity {
    pub type_id: String,
    pub selector: SourceLeafSelector,
    pub normalized_path: String,
}

impl SourceLeafCoverageLedger {
    pub fn identity_for(&self, leaf: &SourceLeafContract) -> SourceLeafIdentity {
        SourceLeafIdentity {
            type_id: self.type_id.clone(),
            selector: self.selector.clone(),
            normalized_path: leaf.normalized_path.clone(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{code:?}: {message}")]
pub struct CoverageContractError {
    pub code: CoverageFailureCode,
    pub message: String,
}

pub fn parse_source_leaf_ledger(
    input: &str,
) -> Result<SourceLeafCoverageLedger, CoverageContractError> {
    yaml_serde::from_str(input).map_err(|error| CoverageContractError {
        code: CoverageFailureCode::InvalidContract,
        message: error.to_string(),
    })
}

pub fn lint_source_leaf_ledger(ledger: &SourceLeafCoverageLedger) -> Vec<CoverageFailure> {
    let mut failures = Vec::new();
    if ledger.contract_version != ATLAS_SOURCE_LEAF_COVERAGE_VERSION {
        failures.push(CoverageFailure::ledger(
            CoverageFailureCode::InvalidContract,
            format!(
                "contract_version must be {ATLAS_SOURCE_LEAF_COVERAGE_VERSION}, found {}",
                ledger.contract_version
            ),
        ));
    }
    if ledger.source_pin != SourcePin::pinned()
        || ledger.selector.source_contract_version != PF2E_SOURCE_CONTRACT_VERSION
    {
        failures.push(CoverageFailure::ledger(
            CoverageFailureCode::SourcePinMismatch,
            format!(
                "ledger must bind source contract {PF2E_SOURCE_CONTRACT_VERSION}, commit {PF2E_SOURCE_PINNED_COMMIT}, and signature {PF2E_SOURCE_PINNED_SIGNATURE}"
            ),
        ));
    }
    for (field, value) in [
        (
            "selector.document_class",
            ledger.selector.document_class.as_str(),
        ),
        (
            "selector.type_discriminator",
            ledger.selector.type_discriminator.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            failures.push(CoverageFailure::ledger(
                CoverageFailureCode::InvalidContract,
                format!("{field} must be non-empty"),
            ));
        }
    }
    match accepted_registry() {
        Ok(registry) => {
            let exact_match = registry.entries.iter().any(|entry| {
                entry.type_id == ledger.type_id
                    && entry.document_class == ledger.selector.document_class
                    && entry.type_discriminator == ledger.selector.type_discriminator
                    && entry.role == ledger.selector.role
                    && entry.parent_context == ledger.selector.parent_context
            });
            if !exact_match {
                failures.push(CoverageFailure::ledger(
                    CoverageFailureCode::RegistryBindingMismatch,
                    "type_id and exact document/discriminator/role/parent-context selector must match one accepted registry entry",
                ));
            }
        }
        Err(error) => failures.push(CoverageFailure::ledger(
            CoverageFailureCode::RegistryBindingMismatch,
            error,
        )),
    }
    if ledger.leaves.is_empty() {
        failures.push(CoverageFailure::ledger(
            CoverageFailureCode::InvalidContract,
            "a source-leaf ledger must contain at least one exact leaf",
        ));
    }

    let mut owners = BTreeSet::new();
    for leaf in &ledger.leaves {
        let identity = ledger.identity_for(leaf);
        if !owners.insert(identity.clone()) {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::DuplicateOwnership,
                identity.clone(),
                "the exact selector and leaf are declared more than once",
            ));
        }
        lint_leaf(leaf, identity, &mut failures);
    }
    failures
}

pub fn lint_source_leaf_ledgers(ledgers: &[SourceLeafCoverageLedger]) -> Vec<CoverageFailure> {
    let mut failures = ledgers
        .iter()
        .flat_map(lint_source_leaf_ledger)
        .collect::<Vec<_>>();
    let mut owners = BTreeMap::<SourceLeafIdentity, usize>::new();
    for (ledger_index, ledger) in ledgers.iter().enumerate() {
        for leaf in &ledger.leaves {
            let identity = ledger.identity_for(leaf);
            if owners
                .insert(identity.clone(), ledger_index)
                .is_some_and(|owner_index| owner_index != ledger_index)
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::DuplicateOwnership,
                    identity,
                    "the exact selector and leaf are owned by more than one ledger",
                ));
            }
        }
    }
    failures
}

fn lint_leaf(
    leaf: &SourceLeafContract,
    identity: SourceLeafIdentity,
    failures: &mut Vec<CoverageFailure>,
) {
    if is_broad_path(&leaf.normalized_path)
        || leaf.normalized_path.matches('*').count() > 0 && leaf.map_key_policy.is_none()
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::BroadDeclaration,
            identity.clone(),
            "recursive, prefix, partial-wildcard, or ungoverned map declarations are forbidden",
        ));
    }
    if leaf.normalized_path.contains('*') && !has_exact_map_wildcard(&leaf.normalized_path) {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::BroadDeclaration,
            identity.clone(),
            "map wildcards must occupy one complete path segment",
        ));
    }
    let has_map_wildcard = leaf
        .normalized_path
        .split('.')
        .any(|segment| segment == "*");
    if has_map_wildcard
        != leaf
            .expected_shapes
            .contains(&ExpectedSourceShape::MapMember)
        || has_map_wildcard != leaf.map_key_policy.is_some()
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::BroadDeclaration,
            identity.clone(),
            "map paths, map-member shapes, and map key policies must agree exactly",
        ));
    }
    let has_array_member = leaf.normalized_path.contains("[]");
    if has_array_member
        != leaf
            .expected_shapes
            .contains(&ExpectedSourceShape::ArrayMember)
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::BroadDeclaration,
            identity.clone(),
            "array-member paths must declare array_member and only array-member paths may do so",
        ));
    }
    if let Some(MapKeyPolicy::ClosedVocabulary { keys }) = &leaf.map_key_policy
        && (keys.is_empty()
            || keys.iter().any(|key| key.trim().is_empty())
            || keys.iter().collect::<BTreeSet<_>>().len() != keys.len())
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::BroadDeclaration,
            identity.clone(),
            "closed map key policies require a non-empty explicit vocabulary",
        ));
    }
    if leaf.expected_shapes.is_empty() {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::InvalidContract,
            identity.clone(),
            "expected_shapes must name the allowed source states and shapes",
        ));
    }
    if leaf.expected_shapes.iter().collect::<BTreeSet<_>>().len() != leaf.expected_shapes.len() {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::InvalidContract,
            identity.clone(),
            "expected_shapes must not contain duplicates",
        ));
    }
    if leaf
        .reader
        .parity_case_ids
        .iter()
        .any(|case_id| case_id.trim().is_empty())
        || leaf
            .reader
            .parity_case_ids
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != leaf.reader.parity_case_ids.len()
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity.clone(),
            "parity_case_ids must be unique non-empty fixture identities",
        ));
    }
    let fixture_cases = leaf
        .fixtures
        .iter()
        .map(|fixture| fixture.case_id.as_str())
        .collect::<BTreeSet<_>>();
    let parity_cases = leaf
        .reader
        .parity_case_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if fixture_cases.len() != leaf.fixtures.len()
        || fixture_cases != parity_cases
        || leaf.fixtures.iter().any(|fixture| {
            fixture.case_id.trim().is_empty()
                || fixture.record_key.trim().is_empty()
                || fixture.source_path.trim().is_empty()
                || !valid_sha256_digest(&fixture.source_file_digest)
                || !valid_sha256_digest(&fixture.excerpt_digest)
        })
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity.clone(),
            "fixture contracts must uniquely and exactly bind every parity case to record/path/sha256 provenance",
        ));
    }
    if leaf.source_prevalence.occurrence_count < leaf.source_prevalence.record_count {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::InvalidContract,
            identity.clone(),
            "source occurrence count cannot be smaller than record count",
        ));
    }
    if leaf.owner.trim().is_empty() || leaf.acceptance_checkpoint.trim().is_empty() {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::InvalidContract,
            identity.clone(),
            "owner and acceptance_checkpoint must be exact non-empty identities",
        ));
    }
    if leaf.rationale.trim().is_empty() {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::InvalidContract,
            identity.clone(),
            "every source-leaf declaration requires a product rationale",
        ));
    }
    for decision in leaf.surfaces.decisions() {
        if decision.rationale.trim().is_empty() {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::InvalidContract,
                identity.clone(),
                "every surface decision requires a rationale",
            ));
            break;
        }
    }

    let mut stages = BTreeMap::new();
    for owner in &leaf.final_owners {
        if owner.destination.trim().is_empty() {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::InvalidContract,
                identity.clone(),
                "final-owner destinations must be exact and non-empty",
            ));
        }
        if stages.insert(owner.stage, &owner.destination).is_some() {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::DuplicateOwnership,
                identity.clone(),
                format!(
                    "final-owner stage {:?} is declared more than once",
                    owner.stage
                ),
            ));
        }
    }

    match leaf.disposition {
        SourceLeafDisposition::Promoted => lint_promoted(leaf, &stages, identity.clone(), failures),
        SourceLeafDisposition::ProvenanceOnly => {
            if leaf.reader.reader_id.as_deref().is_none_or(str::is_empty)
                || leaf.final_owners.len() != 1
                || !stages.contains_key(&FinalOwnerStage::DurableProvenance)
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::ProvenanceNotDurable,
                    identity.clone(),
                    "provenance_only requires an exact provenance reader and durable_provenance as its sole final owner",
                ));
            }
            if leaf
                .surfaces
                .decisions()
                .into_iter()
                .any(|surface| surface.disposition == SurfaceDisposition::Promoted)
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::ProvenanceNotDurable,
                    identity.clone(),
                    "provenance_only cannot declare a promoted semantic surface",
                ));
            }
        }
        SourceLeafDisposition::Ignored => {
            if !leaf.final_owners.is_empty()
                || leaf
                    .surfaces
                    .decisions()
                    .into_iter()
                    .any(|surface| surface.disposition == SurfaceDisposition::Promoted)
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::IgnoredPromoted,
                    identity.clone(),
                    "ignored leaves cannot declare semantic final owners",
                ));
            }
        }
        SourceLeafDisposition::Deferred => {
            if [
                leaf.future_owner.as_deref(),
                leaf.future_task.as_deref(),
                leaf.prerequisite.as_deref(),
            ]
            .into_iter()
            .any(|value| value.is_none_or(str::is_empty))
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::UnresolvedDeferred,
                    identity.clone(),
                    "deferred leaves require an exact future owner, task, and prerequisite",
                ));
            }
        }
        SourceLeafDisposition::Unconsumed => {}
    }

    if matches!(
        leaf.disposition,
        SourceLeafDisposition::Promoted
            | SourceLeafDisposition::ProvenanceOnly
            | SourceLeafDisposition::Ignored
    ) && leaf.reader.parity_case_ids.is_empty()
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity,
            "accepted dispositions require at least one source-grounded parity case",
        ));
    }
}

fn valid_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn lint_promoted(
    leaf: &SourceLeafContract,
    stages: &BTreeMap<FinalOwnerStage, &String>,
    identity: SourceLeafIdentity,
    failures: &mut Vec<CoverageFailure>,
) {
    if leaf.reader.reader_id.as_deref().is_none_or(str::is_empty) {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ReaderNotObserved,
            identity.clone(),
            "promoted leaves require an exact reader_id",
        ));
    }
    for stage in [FinalOwnerStage::SourceDto, FinalOwnerStage::Canonical] {
        if !stages.contains_key(&stage) {
            failures.push(CoverageFailure::for_identity(
                stage.mismatch_code(),
                identity.clone(),
                format!("promoted leaves require a {stage:?} final owner"),
            ));
        }
    }
    if leaf.surfaces.artifact_is_promoted() {
        for stage in [
            FinalOwnerStage::PostProjection,
            FinalOwnerStage::ArtifactHydration,
        ] {
            if !stages.contains_key(&stage) {
                failures.push(CoverageFailure::for_identity(
                    stage.mismatch_code(),
                    identity.clone(),
                    format!("artifact promotion requires a {stage:?} final owner"),
                ));
            }
        }
    }
    if leaf.surfaces.public_surface_is_promoted()
        && !stages.contains_key(&FinalOwnerStage::PublicSurface)
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::PublicSurfaceMismatch,
            identity,
            "a selected public surface requires an exact public_surface final owner",
        ));
    }
}

fn is_broad_path(path: &str) -> bool {
    path.trim().is_empty()
        || path == "$"
        || !path.starts_with("$.")
        || path.contains("**")
        || path.ends_with('.')
}

fn has_exact_map_wildcard(path: &str) -> bool {
    path.split('.').any(|segment| segment == "*")
        && path
            .split('.')
            .all(|segment| !segment.contains('*') || segment == "*")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEDGER: &str = r#"
contract_version: atlas-source-leaf-coverage/v1
type_id: actor--npc--top-level--root--root--root
source_pin:
  upstream_commit: 4cbdaa37d6c33e9519561bae2c59a23e0288cbce
  source_signature: foundry-pf2e:sha256:dd78d67f5b6d25bf65e30ca4da66af76e7a31e1e7d990562f139154b1752603a
selector:
  source_contract_version: pf2e-serialized-source/v1
  document_class: Actor
  type_discriminator: npc
  role: top_level
  parent_context: {}
leaves:
  - normalized_path: $.system.abilities.*.mod
    expected_shapes: [missing, null, number, map_member]
    source_prevalence: { record_count: 1, occurrence_count: 6 }
    map_key_policy:
      kind: closed_vocabulary
      keys: [str, dex, con, int, wis, cha]
    disposition: promoted
    reader:
      reader_id: source::dto::NpcLegacyAbilitySource::mod
      parity_case_ids: [dense-npc]
    fixtures:
      - case_id: dense-npc
        record_key: pf2e.pathfinder-bestiary:fixture
        source_path: packs/pathfinder-bestiary/fixture.json
        source_file_digest: sha256:14191cbe7fa302bf633734290cbb660ae8e955bef24d587cc909533ad6bf68c5
        excerpt_digest: sha256:14191cbe7fa302bf633734290cbb660ae8e955bef24d587cc909533ad6bf68c5
        provenance: pinned_source
    final_owners:
      - { stage: source_dto, destination: NpcLegacyAbilitySource.mod }
      - { stage: canonical, destination: CreatureRecord.legacy_abilities }
    surfaces:
      artifact: { disposition: not_applicable, rationale: canonical-only fixture }
      search: { disposition: not_applicable, rationale: not selected }
      cli: { disposition: not_applicable, rationale: not selected }
      app: { disposition: not_applicable, rationale: not selected }
      ui: { disposition: not_applicable, rationale: not selected }
      runtime: { disposition: not_applicable, rationale: not selected }
    rationale: canonical creature ability modifiers
    owner: atlas-ingest::source::npc_core
    acceptance_checkpoint: A1
"#;

    #[test]
    fn parser_accepts_exact_selector_and_governed_true_map() {
        let ledger = parse_source_leaf_ledger(LEDGER).expect("valid ledger");
        assert_eq!(ledger.leaves.len(), 1);
        assert!(lint_source_leaf_ledger(&ledger).is_empty());
        assert_eq!(
            ledger.selector.parent_context,
            SourceParentContextSelector::root()
        );
    }

    #[test]
    fn checked_in_schema_is_valid_and_version_bound() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/source-leaf-coverage/v1/schema.json"
        ))
        .expect("valid JSON schema");
        assert_eq!(
            schema["properties"]["contract_version"]["const"],
            ATLAS_SOURCE_LEAF_COVERAGE_VERSION
        );
        assert_eq!(schema["additionalProperties"], false);
    }

    #[test]
    fn parser_rejects_unknown_fields_with_exact_code() {
        let error = parse_source_leaf_ledger(&LEDGER.replace(
            "type_id: actor--npc--top-level--root--root--root",
            "type_id: actor--npc--top-level--root--root--root\npath_family: $.system.**",
        ))
        .expect_err("unknown broad field");
        assert_eq!(error.code, CoverageFailureCode::InvalidContract);
    }

    #[test]
    fn lint_rejects_recursive_prefix_intent_and_ungoverned_map() {
        let mut ledger = parse_source_leaf_ledger(LEDGER).expect("valid ledger");
        ledger.leaves[0].normalized_path = "$.system.**".to_string();
        ledger.leaves[0].map_key_policy = None;
        let failures = lint_source_leaf_ledger(&ledger);
        assert_eq!(failures[0].code, CoverageFailureCode::BroadDeclaration);
    }

    #[test]
    fn lint_rejects_duplicate_exact_ownership() {
        let mut ledger = parse_source_leaf_ledger(LEDGER).expect("valid ledger");
        ledger.leaves.push(ledger.leaves[0].clone());
        let failures = lint_source_leaf_ledger(&ledger);
        assert!(
            failures
                .iter()
                .any(|failure| { failure.code == CoverageFailureCode::DuplicateOwnership })
        );
    }

    #[test]
    fn lint_rejects_duplicate_ownership_across_ledgers() {
        let ledger = parse_source_leaf_ledger(LEDGER).expect("valid ledger");
        let failures = lint_source_leaf_ledgers(&[ledger.clone(), ledger]);
        assert!(failures.iter().any(|failure| {
            failure.code == CoverageFailureCode::DuplicateOwnership
                && failure.message.contains("more than one ledger")
        }));
    }

    #[test]
    fn lint_rejects_wrong_pin_and_registry_tuple_with_exact_codes() {
        let mut wrong_pin = parse_source_leaf_ledger(LEDGER).expect("valid ledger");
        wrong_pin.source_pin.upstream_commit = "not-the-pinned-source".to_string();
        assert!(
            lint_source_leaf_ledger(&wrong_pin)
                .iter()
                .any(|failure| { failure.code == CoverageFailureCode::SourcePinMismatch })
        );

        let mut wrong_tuple = parse_source_leaf_ledger(LEDGER).expect("valid ledger");
        wrong_tuple.selector.parent_context = SourceParentContextSelector {
            document_class: Some("Actor".to_string()),
            type_discriminator: Some("npc".to_string()),
            relationship_path: Some("Actor.items".to_string()),
        };
        assert!(
            lint_source_leaf_ledger(&wrong_tuple)
                .iter()
                .any(|failure| { failure.code == CoverageFailureCode::RegistryBindingMismatch })
        );
    }

    #[test]
    fn provenance_only_rejects_semantic_final_owners() {
        let mut ledger = parse_source_leaf_ledger(LEDGER).expect("valid ledger");
        let leaf = &mut ledger.leaves[0];
        leaf.disposition = SourceLeafDisposition::ProvenanceOnly;
        leaf.final_owners = vec![
            FinalOwnerContract {
                stage: FinalOwnerStage::DurableProvenance,
                destination: "AtlasRecord.raw_json".to_string(),
            },
            FinalOwnerContract {
                stage: FinalOwnerStage::Canonical,
                destination: "CreatureRecord.semantic".to_string(),
            },
        ];
        leaf.surfaces = SurfaceContract::default();
        assert!(
            lint_source_leaf_ledger(&ledger)
                .iter()
                .any(|failure| { failure.code == CoverageFailureCode::ProvenanceNotDurable })
        );
    }
}
