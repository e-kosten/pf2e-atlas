use std::any::type_name;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::SourcePresence;
use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE,
};

use super::{
    CoverageContractError, CoverageFailureCode, FinalOwnerStage, FixtureContract,
    FixtureProvenance, PF2E_TYPE_REGISTRY_SHA256, SourceLeafIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAccessorPurpose {
    Parser,
    ProvenanceReader,
    Inventory,
}

pub trait SourceLeafAccessor {
    const ID: &'static str;
    const PURPOSE: SourceAccessorPurpose;

    fn read(excerpt: &[u8]) -> Result<SourcePresence<SourceLeafValue>, String>;
}

pub trait FinalOwnerAccessor<T> {
    const STAGE: FinalOwnerStage;
    const DESTINATION: &'static str;

    fn observe(value: &T) -> SourcePresence<SourceLeafValue>;
}

pub trait SemanticOutputAccessor<T> {
    fn semantic_output_observed(value: &T) -> bool;
}

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
    pub fn capture<A: SourceLeafAccessor>(
        identity: SourceLeafIdentity,
        fixture: FixtureReference,
        mutation_sentinel_excerpt: &[u8],
    ) -> Result<SourceLeafReceiptBuilder, CoverageContractError> {
        if fixture.excerpt == mutation_sentinel_excerpt {
            return Err(CoverageContractError {
                code: CoverageFailureCode::ReceiptProvenanceInvalid,
                message: "mutation sentinel must change the authenticated source excerpt"
                    .to_string(),
            });
        }
        let source = A::read(&fixture.excerpt).map_err(|message| CoverageContractError {
            code: CoverageFailureCode::ReaderNotObserved,
            message,
        })?;
        let sentinel =
            A::read(mutation_sentinel_excerpt).map_err(|message| CoverageContractError {
                code: CoverageFailureCode::ReaderNotObserved,
                message: format!("mutation sentinel reader failed: {message}"),
            })?;
        if source == sentinel {
            return Err(CoverageContractError {
                code: CoverageFailureCode::ReaderNotObserved,
                message: format!(
                    "accessor {} did not observe the source mutation sentinel",
                    A::ID
                ),
            });
        }
        Ok(SourceLeafReceiptBuilder {
            identity,
            fixture,
            source,
            reader: ActualReadEvidence {
                reader_id: A::ID.to_string(),
                accessor_binding: type_name::<A>().to_string(),
                purpose: A::PURPOSE,
                mutation_digest: digest_bytes(mutation_sentinel_excerpt),
            },
            observations: Vec::new(),
            semantic_output: None,
        })
    }

    pub fn identity(&self) -> &SourceLeafIdentity {
        &self.identity
    }

    pub fn fixture(&self) -> &FixtureReference {
        &self.fixture
    }

    pub fn source(&self) -> &SourcePresence<SourceLeafValue> {
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

    #[cfg(test)]
    pub(crate) fn corrupt_identity_path(&mut self, path: &str) {
        self.identity.normalized_path = path.to_string();
    }

    #[cfg(test)]
    pub(crate) fn corrupt_evidence_digest(&mut self) {
        self.evidence_digest = "sha256:tampered".to_string();
    }
}

#[derive(Debug)]
pub struct SourceLeafReceiptBuilder {
    identity: SourceLeafIdentity,
    fixture: FixtureReference,
    source: SourcePresence<SourceLeafValue>,
    reader: ActualReadEvidence,
    observations: Vec<StageObservation>,
    semantic_output: Option<SemanticOutputObservation>,
}

impl SourceLeafReceiptBuilder {
    pub fn observe_final_owner<T, A: FinalOwnerAccessor<T>>(
        &mut self,
        value: &T,
        mutation_sentinel_value: &T,
    ) -> Result<&mut Self, CoverageContractError> {
        let observed = A::observe(value);
        let sentinel = A::observe(mutation_sentinel_value);
        if observed == sentinel {
            return Err(CoverageContractError {
                code: A::STAGE.mismatch_code(),
                message: format!(
                    "final-owner accessor {} did not observe its typed mutation sentinel",
                    type_name::<A>()
                ),
            });
        }
        self.observations.push(StageObservation {
            stage: A::STAGE,
            destination: A::DESTINATION.to_string(),
            accessor_binding: type_name::<A>().to_string(),
            mutation_digest: digest_serializable(&sentinel),
            value: observed,
        });
        Ok(self)
    }

    pub fn observe_semantic_output<T, A: SemanticOutputAccessor<T>>(
        &mut self,
        value: &T,
        mutation_sentinel_value: &T,
    ) -> Result<&mut Self, CoverageContractError> {
        let observed = A::semantic_output_observed(value);
        let sentinel = A::semantic_output_observed(mutation_sentinel_value);
        if observed == sentinel {
            return Err(CoverageContractError {
                code: CoverageFailureCode::ReceiptProvenanceInvalid,
                message: format!(
                    "semantic-output accessor {} did not observe its typed mutation sentinel",
                    type_name::<A>()
                ),
            });
        }
        self.semantic_output = Some(SemanticOutputObservation {
            accessor_binding: type_name::<A>().to_string(),
            mutation_observed: sentinel,
            observed,
        });
        Ok(self)
    }

    pub fn finish(self) -> SourceLeafReceipt {
        let evidence_digest = evidence_digest(
            &self.identity,
            &self.fixture,
            &self.source,
            &self.reader,
            &self.observations,
            &self.semantic_output,
        );
        SourceLeafReceipt {
            identity: self.identity,
            fixture: self.fixture,
            source: self.source,
            reader: self.reader,
            observations: self.observations,
            semantic_output: self.semantic_output,
            evidence_digest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureReference {
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
    #[serde(skip)]
    excerpt: Vec<u8>,
}

impl FixtureReference {
    pub fn authenticate(
        contract: &FixtureContract,
        source_file: &[u8],
        excerpt: &[u8],
    ) -> Result<Self, CoverageContractError> {
        if excerpt.is_empty() {
            return Err(CoverageContractError {
                code: CoverageFailureCode::ReceiptProvenanceInvalid,
                message: format!("fixture {} excerpt cannot be empty", contract.case_id),
            });
        }
        let actual_source_file_digest = digest_bytes(source_file);
        let actual_digest = digest_bytes(excerpt);
        if actual_source_file_digest != contract.source_file_digest
            || actual_digest != contract.excerpt_digest
            || !source_file
                .windows(excerpt.len())
                .any(|candidate| candidate == excerpt)
        {
            return Err(CoverageContractError {
                code: CoverageFailureCode::ReceiptProvenanceInvalid,
                message: format!(
                    "fixture {} is not an exact excerpt of the declaration-bound source file",
                    contract.case_id
                ),
            });
        }
        Ok(Self {
            case_id: contract.case_id.clone(),
            record_key: contract.record_key.clone(),
            source_path: contract.source_path.clone(),
            source_file_digest: actual_source_file_digest,
            excerpt_digest: actual_digest,
            provenance: contract.provenance,
            source_contract_version: PF2E_SOURCE_CONTRACT_VERSION,
            source_commit: PF2E_SOURCE_PINNED_COMMIT,
            source_signature: PF2E_SOURCE_PINNED_SIGNATURE,
            registry_sha256: PF2E_TYPE_REGISTRY_SHA256,
            excerpt: excerpt.to_vec(),
        })
    }

    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    pub fn record_key(&self) -> &str {
        &self.record_key
    }

    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    pub fn excerpt_digest(&self) -> &str {
        &self.excerpt_digest
    }

    pub fn source_file_digest(&self) -> &str {
        &self.source_file_digest
    }

    pub fn provenance(&self) -> FixtureProvenance {
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
pub struct StageObservation {
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
pub(crate) struct SemanticOutputObservation {
    accessor_binding: String,
    mutation_observed: bool,
    observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafValue {
    pub json_type: SourceJsonType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_kind: Option<SourceMemberKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinal: Option<usize>,
    pub multiplicity: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<TypedUnsupportedValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceJsonType {
    Boolean,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMemberKind {
    Array,
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypedUnsupportedValue {
    pub value: Value,
    pub reason: String,
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
    let encoded = serde_json::to_vec(&(
        identity,
        fixture,
        source,
        reader,
        observations,
        semantic_output,
    ))
    .expect("source-leaf receipt evidence is serializable");
    digest_bytes(&encoded)
}
