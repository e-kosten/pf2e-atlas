use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use atlas_domain::{DetailLevel, PackName};
use atlas_record::{RecordBody, RecordJsonOptions, record_json};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::SourcePresence;
use crate::diagnostics::IngestDiagnostics;
use crate::index_build_input::index_build_input;
use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE,
    SourceIdentity, parse_item_source, pinned_source_version_metadata,
};
use crate::source::normalize::normalize_record;
use crate::source::{LoadedPack, ManifestPack, SourceLoad};

use super::{
    CoverageContractError, CoverageFailureCode, FinalOwnerStage, FixtureContract,
    FixtureProvenance, PF2E_TYPE_REGISTRY_SHA256, SourceLeafCoverageLedger, SourceLeafIdentity,
    lint_source_leaf_ledger,
};

const ITEM_NAME_READER: &str = "source::dto::FullItemSource::name";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) enum SourceAccessorPurpose {
    Parser,
    ProvenanceReader,
    Inventory,
}

/// Opaque evidence emitted only by the sealed exact-accessor registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafReceipt {
    identity: SourceLeafIdentity,
    fixture: FixtureReference,
    source: SourcePresence<SourceLeafValue>,
    reader: ActualReadEvidence,
    observations: Vec<StageObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_output: Option<SemanticOutputObservation>,
    evidence_digest: String,
}

impl SourceLeafReceipt {
    pub fn identity(&self) -> &SourceLeafIdentity {
        &self.identity
    }
    pub(crate) fn fixture(&self) -> &FixtureReference {
        &self.fixture
    }
    pub(crate) fn source(&self) -> &SourcePresence<SourceLeafValue> {
        &self.source
    }
    pub(crate) fn integrity_is_valid(&self) -> bool {
        self.evidence_digest
            == evidence_digest(
                &self.identity,
                &self.fixture,
                &self.source,
                &self.reader,
                &self.observations,
                &self.semantic_output,
            )
    }
    pub(crate) fn reader_id(&self) -> &str {
        &self.reader.reader_id
    }
    pub(crate) fn reader_binding(&self) -> &str {
        &self.reader.accessor_binding
    }
    pub(crate) fn reader_purpose(&self) -> SourceAccessorPurpose {
        self.reader.purpose
    }
    pub(crate) fn reader_mutation_digest(&self) -> &str {
        &self.reader.mutation_digest
    }
    pub(crate) fn observations(&self) -> &[StageObservation] {
        &self.observations
    }
    pub(crate) fn semantic_output(&self) -> Option<(bool, bool, &str)> {
        self.semantic_output.as_ref().map(|proof| {
            (
                proof.observed,
                proof.mutation_observed,
                proof.accessor_binding.as_str(),
            )
        })
    }
}

/// Loads a fixture from the pinned Git tree and executes its registered parser
/// and selected real owner stages. Callers supply no bytes, accessor types,
/// owner values, mutation sentinels, or success flags.
///
/// Declaration-equivalent accessor and builder APIs do not exist:
///
/// ```compile_fail
/// use atlas_ingest::{SourceLeafAccessor, SourceLeafReceiptBuilder};
/// ```
///
/// Fixture references cannot be self-authenticated from caller bytes:
///
/// ```compile_fail
/// use atlas_ingest::FixtureReference;
/// ```
///
/// A cloned caller-owned value cannot be submitted as every owner stage:
///
/// ```compile_fail
/// use atlas_ingest::SourceLeafReceipt;
/// struct OwnerValue(serde_json::Value);
/// let same_owner = OwnerValue(serde_json::json!({"value": 4}));
/// let _ = SourceLeafReceipt::capture(same_owner);
/// ```
pub fn capture_registered_source_leaf_receipt(
    ledger: &SourceLeafCoverageLedger,
    leaf_index: usize,
    fixture_index: usize,
    source_repository: &Path,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    if let Some(failure) = lint_source_leaf_ledger(ledger).into_iter().next() {
        return Err(error(failure.code, failure.message));
    }
    let leaf = ledger.leaves.get(leaf_index).ok_or_else(|| {
        error(
            CoverageFailureCode::InvalidContract,
            format!("leaf index {leaf_index} is outside the ledger"),
        )
    })?;
    let fixture = leaf.fixtures.get(fixture_index).ok_or_else(|| {
        error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            format!("fixture index {fixture_index} is outside the leaf"),
        )
    })?;
    let identity = ledger.identity_for(leaf);
    let registration = registration_for(&identity, leaf.reader.reader_id.as_deref())?;
    registration.capture(identity, ResolvedFixture::load(fixture, source_repository)?)
}

#[derive(Debug, Clone, Copy)]
enum RegisteredAccessor {
    ItemActionName,
}

fn registration_for(
    identity: &SourceLeafIdentity,
    reader_id: Option<&str>,
) -> Result<RegisteredAccessor, CoverageContractError> {
    if identity.type_id == "item--action--top-level--root--root--root"
        && identity.selector.document_class == "Item"
        && identity.selector.type_discriminator == "action"
        && identity.normalized_path == "$.name"
        && reader_id == Some(ITEM_NAME_READER)
    {
        Ok(RegisteredAccessor::ItemActionName)
    } else {
        Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!(
                "no sealed semantic accessor is registered for {} {}",
                identity.normalized_path,
                reader_id.unwrap_or("<missing reader>")
            ),
        ))
    }
}

impl RegisteredAccessor {
    fn capture(
        self,
        identity: SourceLeafIdentity,
        fixture: ResolvedFixture,
    ) -> Result<SourceLeafReceipt, CoverageContractError> {
        match self {
            Self::ItemActionName => capture_item_name(identity, fixture),
        }
    }
}

fn capture_item_name(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let source_name = fixture
        .raw
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "$.name is not a string",
            )
        })?;
    let excerpt = serde_json::to_vec(&Value::String(source_name.to_string()))
        .expect("JSON string serialization cannot fail");
    if digest_bytes(&excerpt) != fixture.reference.excerpt_digest {
        return Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            "registered $.name excerpt does not match its declaration digest",
        ));
    }
    let actual = run_item_name_pipeline(&fixture, fixture.raw.clone())?;
    let mut mutated_raw = fixture.raw.clone();
    mutated_raw["name"] = Value::String(format!("{source_name}\u{241f}source-leaf-mutation"));
    let mutation = run_item_name_pipeline(&fixture, mutated_raw)?;
    if actual == mutation {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            "registered pipeline did not observe its internal source mutation",
        ));
    }

    let source = string_leaf(actual.source_dto.clone());
    let observations = vec![
        stage(
            FinalOwnerStage::SourceDto,
            "FullItemSource.name",
            "source::dto::parse_item_source",
            actual.source_dto,
            &mutation.source_dto,
        ),
        stage(
            FinalOwnerStage::Canonical,
            "AtlasRecord.identity.name",
            "source::normalize::normalize_record",
            actual.canonical,
            &mutation.canonical,
        ),
        stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.records[].identity.name",
            "index_build_input::index_build_input",
            actual.post_projection,
            &mutation.post_projection,
        ),
        stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.record.identity.name",
            "atlas_index::hydrate_record_parts",
            actual.hydration,
            &mutation.hydration,
        ),
        stage(
            FinalOwnerStage::PublicSurface,
            "RecordJson.name",
            "atlas_record::record_json",
            actual.public_surface,
            &mutation.public_surface,
        ),
    ];
    let reader = ActualReadEvidence {
        reader_id: ITEM_NAME_READER.to_string(),
        accessor_binding: "sealed::ItemActionName".to_string(),
        purpose: SourceAccessorPurpose::Parser,
        mutation_digest: digest_serializable(&string_leaf(mutation.source_dto)),
    };
    let semantic_output = None;
    let evidence_digest = evidence_digest(
        &identity,
        &fixture.reference,
        &source,
        &reader,
        &observations,
        &semantic_output,
    );
    Ok(SourceLeafReceipt {
        identity,
        fixture: fixture.reference,
        source,
        reader,
        observations,
        semantic_output,
        evidence_digest,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct ItemNamePipeline {
    source_dto: String,
    canonical: String,
    post_projection: String,
    hydration: String,
    public_surface: String,
}

fn run_item_name_pipeline(
    fixture: &ResolvedFixture,
    raw: Value,
) -> Result<ItemNamePipeline, CoverageContractError> {
    let pack = fixture
        .reference
        .record_key
        .split_once(':')
        .map(|(pack, _)| pack)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "invalid record key",
            )
        })?;
    let pack_name = PackName::new(pack.to_string()).map_err(|message| {
        error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            message.to_string(),
        )
    })?;
    let manifest_pack = ManifestPack {
        name: pack.strip_prefix("pf2e.").unwrap_or(pack).to_string(),
        label: "source-leaf fixture".to_string(),
        document_type: "Item".to_string(),
        path: fixture
            .reference
            .source_path
            .rsplit_once('/')
            .map_or("packs", |(parent, _)| parent)
            .to_string(),
    };
    let source_dto = parse_item_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(
            fixture.reference.record_key.clone(),
            fixture.reference.source_path.clone(),
        ),
        None,
        raw.clone(),
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?
    .source
    .source()
    .name
    .clone();
    let loaded = normalize_record(
        &manifest_pack,
        &pack_name,
        Path::new(&fixture.reference.source_path),
        Path::new("."),
        raw,
        None,
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let canonical = loaded.record.identity.name.clone();
    let source_load = SourceLoad {
        manifest_path: PathBuf::from("static/system.json"),
        source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
        source_record_count: 1,
        packs: vec![LoadedPack {
            name: pack_name,
            label: manifest_pack.label,
            document_type: manifest_pack.document_type,
            declared_path: manifest_pack.path.clone(),
            resolved_path: PathBuf::from(manifest_pack.path),
            record_count: 1,
        }],
        records: vec![loaded],
        references: Vec::new(),
        aliases: Vec::new(),
        remaster_links: Vec::new(),
        pending_document_embeddings: Vec::new(),
        document_embeddings: Vec::new(),
        document_embedding_tokenization: Default::default(),
        diagnostics: IngestDiagnostics::default(),
        skipped_records: Vec::new(),
        warnings: Vec::new(),
    };
    let input = index_build_input(source_load);
    let post_projection = input.records[0].identity.name.clone();
    let bodies = input
        .canonical_bodies
        .into_iter()
        .map(|body| {
            let RecordBody::Creature(creature) = &body;
            (creature.identity.record_key.clone(), body)
        })
        .collect::<BTreeMap<_, _>>();
    let hydrated = atlas_index::hydrate_record_parts(input.records, bodies)
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let hydration = hydrated[0].record.identity.name.clone();
    let public = record_json(
        &hydrated[0],
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: true,
        },
    )
    .map_err(|message| error(CoverageFailureCode::PublicSurfaceMismatch, message))?;
    Ok(ItemNamePipeline {
        source_dto,
        canonical,
        post_projection,
        hydration,
        public_surface: public.name.clone(),
    })
}

#[derive(Debug)]
struct ResolvedFixture {
    reference: FixtureReference,
    raw: Value,
}

impl ResolvedFixture {
    fn load(
        contract: &FixtureContract,
        source_repository: &Path,
    ) -> Result<Self, CoverageContractError> {
        if contract.provenance != FixtureProvenance::PinnedSource
            || Path::new(&contract.source_path).is_absolute()
            || contract
                .source_path
                .split('/')
                .any(|segment| segment == "..")
        {
            return Err(error(
                CoverageFailureCode::ReceiptProvenanceInvalid,
                "registered receipts require a safe pinned-source relative path",
            ));
        }
        git_success(
            source_repository,
            &[
                "cat-file",
                "-e",
                &format!("{PF2E_SOURCE_PINNED_COMMIT}^{{commit}}"),
            ],
        )?;
        let bytes = git_output(
            source_repository,
            &[
                "show",
                &format!("{PF2E_SOURCE_PINNED_COMMIT}:{}", contract.source_path),
            ],
        )?;
        if digest_bytes(&bytes) != contract.source_file_digest {
            return Err(error(
                CoverageFailureCode::ReceiptProvenanceInvalid,
                "source digest does not match the blob from the pinned Git tree",
            ));
        }
        let raw: Value = serde_json::from_slice(&bytes)
            .map_err(|message| error(CoverageFailureCode::FixtureNotSourceGrounded, message))?;
        let source_id = raw.get("_id").and_then(Value::as_str).ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "pinned fixture has no string _id",
            )
        })?;
        let pack = contract
            .source_path
            .strip_prefix("packs/")
            .and_then(|path| path.split('/').next())
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::FixtureNotSourceGrounded,
                    "source_path must identify a pinned PF2e pack record",
                )
            })?;
        let resolved_record_key = format!("pf2e.{pack}:{source_id}");
        if resolved_record_key != contract.record_key {
            return Err(error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                format!(
                    "fixture record_key {} does not match {resolved_record_key}",
                    contract.record_key
                ),
            ));
        }
        Ok(Self {
            reference: FixtureReference {
                case_id: contract.case_id.clone(),
                record_key: contract.record_key.clone(),
                source_path: contract.source_path.clone(),
                source_file_digest: contract.source_file_digest.clone(),
                excerpt_digest: contract.excerpt_digest.clone(),
                provenance: contract.provenance,
                source_contract_version: PF2E_SOURCE_CONTRACT_VERSION,
                source_commit: PF2E_SOURCE_PINNED_COMMIT,
                source_signature: PF2E_SOURCE_PINNED_SIGNATURE,
                registry_sha256: PF2E_TYPE_REGISTRY_SHA256,
            },
            raw,
        })
    }
}

fn git_success(repository: &Path, args: &[&str]) -> Result<(), CoverageContractError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .map_err(|message| error(CoverageFailureCode::ReceiptProvenanceInvalid, message))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            "source repository does not contain the accepted pinned commit",
        ))
    }
}

fn git_output(repository: &Path, args: &[&str]) -> Result<Vec<u8>, CoverageContractError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .map_err(|message| error(CoverageFailureCode::ReceiptProvenanceInvalid, message))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            String::from_utf8_lossy(&output.stderr).trim(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixtureReference {
    case_id: String,
    record_key: String,
    source_path: String,
    source_file_digest: String,
    excerpt_digest: String,
    provenance: FixtureProvenance,
    source_contract_version: &'static str,
    source_commit: &'static str,
    source_signature: &'static str,
    registry_sha256: &'static str,
}

impl FixtureReference {
    pub(crate) fn case_id(&self) -> &str {
        &self.case_id
    }
    pub(crate) fn record_key(&self) -> &str {
        &self.record_key
    }
    pub(crate) fn source_path(&self) -> &str {
        &self.source_path
    }
    pub(crate) fn excerpt_digest(&self) -> &str {
        &self.excerpt_digest
    }
    pub(crate) fn source_file_digest(&self) -> &str {
        &self.source_file_digest
    }
    pub(crate) fn provenance(&self) -> FixtureProvenance {
        self.provenance
    }
    pub(crate) fn has_authenticated_source_binding(&self) -> bool {
        self.source_contract_version == PF2E_SOURCE_CONTRACT_VERSION
            && self.source_commit == PF2E_SOURCE_PINNED_COMMIT
            && self.source_signature == PF2E_SOURCE_PINNED_SIGNATURE
            && self.registry_sha256 == PF2E_TYPE_REGISTRY_SHA256
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct ActualReadEvidence {
    reader_id: String,
    accessor_binding: String,
    purpose: SourceAccessorPurpose,
    mutation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageObservation {
    stage: FinalOwnerStage,
    destination: String,
    accessor_binding: String,
    mutation_digest: String,
    value: SourcePresence<SourceLeafValue>,
}

impl StageObservation {
    pub(crate) fn stage(&self) -> FinalOwnerStage {
        self.stage
    }
    pub(crate) fn destination(&self) -> &str {
        &self.destination
    }
    pub(crate) fn accessor_binding(&self) -> &str {
        &self.accessor_binding
    }
    pub(crate) fn mutation_digest(&self) -> &str {
        &self.mutation_digest
    }
    pub(crate) fn value(&self) -> &SourcePresence<SourceLeafValue> {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct SemanticOutputObservation {
    accessor_binding: String,
    mutation_observed: bool,
    observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceLeafValue {
    pub(crate) json_type: SourceJsonType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) stable_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) member_kind: Option<SourceMemberKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) member_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ordinal: Option<usize>,
    pub(crate) multiplicity: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) unsupported: Option<TypedUnsupportedValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) enum SourceJsonType {
    Boolean,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) enum SourceMemberKind {
    Array,
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TypedUnsupportedValue {
    pub(crate) value: Value,
    pub(crate) reason: String,
}

fn string_leaf(value: String) -> SourcePresence<SourceLeafValue> {
    SourcePresence::Value(SourceLeafValue {
        json_type: SourceJsonType::String,
        value: Some(Value::String(value)),
        stable_digest: None,
        member_kind: None,
        member_identity: None,
        ordinal: None,
        multiplicity: 1,
        unsupported: None,
    })
}

fn stage(
    stage: FinalOwnerStage,
    destination: &str,
    accessor_binding: &str,
    value: String,
    mutation: &str,
) -> StageObservation {
    StageObservation {
        stage,
        destination: destination.to_string(),
        accessor_binding: accessor_binding.to_string(),
        mutation_digest: digest_serializable(&string_leaf(mutation.to_string())),
        value: string_leaf(value),
    }
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn digest_serializable(value: &impl Serialize) -> String {
    digest_bytes(&serde_json::to_vec(value).expect("source-leaf evidence is serializable"))
}
fn evidence_digest(
    identity: &SourceLeafIdentity,
    fixture: &FixtureReference,
    source: &SourcePresence<SourceLeafValue>,
    reader: &ActualReadEvidence,
    observations: &[StageObservation],
    semantic_output: &Option<SemanticOutputObservation>,
) -> String {
    digest_serializable(&(
        identity,
        fixture,
        source,
        reader,
        observations,
        semantic_output,
    ))
}
fn error(code: CoverageFailureCode, message: impl ToString) -> CoverageContractError {
    CoverageContractError {
        code,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::source_coverage::{
        ExpectedSourceShape, FinalOwnerContract, FixturePrevalence, MapKeyPolicy, ReaderContract,
        SourceDocumentRole, SourceLeafContract, SourceLeafDisposition, SourceLeafKind,
        SourceLeafSelector, SourceParentContextSelector, SourcePin, SourcePrevalence,
        SurfaceContract, SurfaceDecision, SurfaceDisposition, evaluate_source_leaf_coverage,
    };

    fn promoted() -> SurfaceDecision {
        SurfaceDecision {
            disposition: SurfaceDisposition::Promoted,
            rationale: "selected by this engine fixture".to_string(),
        }
    }

    fn ledger() -> SourceLeafCoverageLedger {
        SourceLeafCoverageLedger {
            contract_version: "atlas-source-leaf-coverage/v1".to_string(),
            type_id: "item--action--top-level--root--root--root".to_string(),
            source_pin: SourcePin::pinned(),
            selector: SourceLeafSelector {
                source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
                document_class: "Item".to_string(),
                type_discriminator: "action".to_string(),
                role: SourceDocumentRole::TopLevel,
                parent_context: SourceParentContextSelector::root(),
            },
            leaves: vec![SourceLeafContract {
                normalized_path: "$.name".to_string(),
                leaf_kind: SourceLeafKind::Scalar,
                expected_shapes: vec![ExpectedSourceShape::String],
                source_prevalence: SourcePrevalence {
                    source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
                    source_commit: PF2E_SOURCE_PINNED_COMMIT.to_string(),
                    source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
                    registry_sha256: PF2E_TYPE_REGISTRY_SHA256.to_string(),
                    // The authenticated registry reports 1,169 top-level action Items;
                    // required `name` occurs exactly once per record at this pin.
                    record_count: 1_169,
                    occurrence_count: 1_169,
                },
                fixture_prevalence: FixturePrevalence {
                    record_count: 1,
                    occurrence_count: 1,
                },
                map_key_policy: None::<MapKeyPolicy>,
                disposition: SourceLeafDisposition::Promoted,
                reader: ReaderContract {
                    reader_id: Some(ITEM_NAME_READER.to_string()),
                    parity_case_ids: vec!["registered-item-name".to_string()],
                },
                fixtures: vec![FixtureContract {
                    case_id: "registered-item-name".to_string(),
                    record_key: "pf2e.actions:c40APnn4a7bWhtcZ".to_string(),
                    source_path: "packs/actions/a-challenge-for-heroes.json".to_string(),
                    source_file_digest:
                        "sha256:b71bc2d31b2a10c232a01ef57b5943b60e4747770b7f35d8ebeb27bf4b281d43"
                            .to_string(),
                    excerpt_digest:
                        "sha256:d3c3aced9d8c6e0b39d40df1f17917eac5f850a34abf4b2a76915d0bd2c4140c"
                            .to_string(),
                    provenance: FixtureProvenance::PinnedSource,
                }],
                final_owners: vec![
                    owner(FinalOwnerStage::SourceDto, "FullItemSource.name"),
                    owner(FinalOwnerStage::Canonical, "AtlasRecord.identity.name"),
                    owner(
                        FinalOwnerStage::PostProjection,
                        "IndexBuildInput.records[].identity.name",
                    ),
                    owner(
                        FinalOwnerStage::ArtifactHydration,
                        "RetrievedRecord.record.identity.name",
                    ),
                    owner(FinalOwnerStage::PublicSurface, "RecordJson.name"),
                ],
                surfaces: SurfaceContract {
                    artifact: promoted(),
                    app: promoted(),
                    ..SurfaceContract::default()
                },
                rationale: "exercise the sealed A1 engine over a pinned source leaf".to_string(),
                owner: "atlas-ingest::source_coverage".to_string(),
                acceptance_checkpoint: "A1 engine test only".to_string(),
                future_owner: None,
                future_task: None,
                prerequisite: None,
            }],
        }
    }

    fn owner(stage: FinalOwnerStage, destination: &str) -> FinalOwnerContract {
        FinalOwnerContract {
            stage,
            destination: destination.to_string(),
        }
    }

    fn pinned_repository() -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("PF2E_SOURCE_REPOSITORY").map(PathBuf::from) {
            return Some(path);
        }
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .output()
            .ok()?;
        String::from_utf8(output.stdout)
            .ok()?
            .lines()
            .filter_map(|line| line.strip_prefix("worktree "))
            .map(|root| Path::new(root).join("vendor/pf2e"))
            .find(|candidate| candidate.is_dir())
    }

    #[test]
    fn registered_receipt_executes_real_pipeline_and_does_not_equate_global_prevalence() {
        let Some(repository) = pinned_repository() else {
            eprintln!(
                "pinned source repository unavailable; fixture authentication tested separately"
            );
            return;
        };
        let ledger = ledger();
        let receipt = capture_registered_source_leaf_receipt(&ledger, 0, 0, &repository)
            .expect("sealed registered receipt");
        let report = evaluate_source_leaf_coverage(&ledger, &[receipt]);
        assert!(report.passed, "{:#?}", report.failures);
        assert_eq!(ledger.leaves[0].source_prevalence.occurrence_count, 1_169);
        assert_eq!(ledger.leaves[0].fixture_prevalence.occurrence_count, 1);
    }

    #[test]
    fn fake_repository_and_self_stamped_fixture_cannot_authenticate() {
        let ledger = ledger();
        let error = capture_registered_source_leaf_receipt(&ledger, 0, 0, Path::new("."))
            .expect_err("Atlas repository is not the pinned PF2e source repository");
        assert_eq!(error.code, CoverageFailureCode::ReceiptProvenanceInvalid);

        let mut fake = ledger.clone();
        fake.leaves[0].fixtures[0].source_file_digest = format!("sha256:{}", "0".repeat(64));
        let error = capture_registered_source_leaf_receipt(&fake, 0, 0, Path::new("."))
            .expect_err("caller digest cannot self-stamp source identity");
        assert_eq!(error.code, CoverageFailureCode::ReceiptProvenanceInvalid);
    }

    #[test]
    fn unregistered_direct_json_shape_has_no_receipt_path() {
        let mut ledger = ledger();
        ledger.leaves[0].reader.reader_id = Some("tests::DirectJsonAccessor".to_string());
        let error = capture_registered_source_leaf_receipt(&ledger, 0, 0, Path::new("."))
            .expect_err("unregistered caller accessors are rejected before source loading");
        assert_eq!(error.code, CoverageFailureCode::ReaderNotObserved);
    }
}
