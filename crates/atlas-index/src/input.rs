//! Immutable build inputs. Source admission and filesystem discovery remain ingest-owned.
use atlas_domain::{RecordKey, SourcePassageAddress};
use atlas_record::source_content::{
    ContentAudience, ContentInteraction, ContentReferenceResolution, ContentVisibilityRule,
    OwnedContentLocator, SourceContentLocator,
};
use atlas_record::source_record::{
    SourceBackedRecord, SourceContentOutcome, SourceRelationshipOccurrence,
};
use serde::{Deserialize, Serialize};

pub const ARTIFACT_FORMAT_VERSION: u32 = 2;
pub const ARTIFACT_CONTRACT_VERSION: &str = "pf2e-atlas-source-artifact/v2";
pub const ARTIFACT_SCHEMA_VERSION: &str = "2";
pub const EXPECTED_SOURCE_KIND: &str = "foundry-pf2e";
pub const SOURCE_PROJECTION_VERSION: &str = "authored-query/v1";
pub const SOURCE_LOOKUP_VERSION: &str = "nfc-lowercase-whitespace/v1";
pub const LEXICAL_SELECTION_VERSION: &str = "curated-lexical/v1";
pub const LOCALIZATION_POLICY_VERSION: &str = "selected-source-catalog/english-fallback/v1";
pub const ASSET_POLICY_VERSION: &str = "safe-remote-images/local-unavailable/v1";
pub const RELATIONSHIP_POLICY_VERSION: &str = "typed-authored-occurrences/v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexBuildPack {
    pub pack_id: String,
    pub label: String,
    pub document_kind: String,
    pub declared_path: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSemanticModelIdentity {
    pub provider: String,
    pub model_id: String,
    pub revision: String,
    pub model_sha256: String,
    pub tokenizer_sha256: String,
    pub dimensions: usize,
    pub pooling: String,
    pub normalization: String,
    pub distance_metric: String,
    pub query_prefix: String,
    pub document_prefix: String,
    pub token_limit: usize,
    pub unit_policy_version: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifactBuildContext {
    pub format_version: u32,
    pub snapshot_version: u32,
    pub source_contract: String,
    pub source_revision: Option<String>,
    pub source_fingerprint: String,
    pub source_file_count: usize,
    pub indexing_locale: String,
    pub locale_catalog_sha256: String,
    pub english_catalog_sha256: String,
    pub used_trait_labels: std::collections::BTreeMap<String, String>,
    pub localization_policy: String,
    pub audience: ContentAudience,
    pub content_interpretation_version: String,
    pub content_selection_version: String,
    pub asset_policy: String,
    pub relationship_policy: String,
    pub query_projection_version: String,
    pub query_catalog_version: u32,
    pub lookup_version: String,
    pub lexical_selection_version: String,
    pub fts_tokenizer: String,
    pub semantic_model: Option<SourceSemanticModelIdentity>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifactDiagnostic {
    pub owners: Option<Vec<OwnedContentLocator>>,
    pub field: Option<String>,
    pub stage: String,
    pub code: String,
    pub message: String,
}
#[derive(Debug)]
pub struct SourceArtifactRecordInput {
    pub record: SourceBackedRecord,
    pub source_path: String,
    pub content_hash: String,
    pub preparation_context_hash: String,
    pub content: Vec<SourceContentOutcome>,
    pub relationships: Vec<SourceRelationshipOccurrence>,
    pub diagnostics: Vec<SourceArtifactDiagnostic>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAliasInput {
    pub record: RecordKey,
    pub alias: String,
    pub evidence: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRemasterPairInput {
    pub legacy: RecordKey,
    pub remaster: RecordKey,
    pub evidence: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLexicalUnitKind {
    RootName,
    OwnedName,
    Heading,
    DefinitionLabel,
}
impl SourceLexicalUnitKind {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::RootName => "root_name",
            Self::OwnedName => "owned_name",
            Self::Heading => "heading",
            Self::DefinitionLabel => "definition_label",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLexicalUnitInput {
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub address: Option<SourcePassageAddress>,
    pub kind: SourceLexicalUnitKind,
    pub identity_terms: String,
    pub alias_terms: String,
    pub structured_terms: String,
    pub definition_terms: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSemanticUnitInput {
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub address: SourcePassageAddress,
    pub chunk_ordinal: usize,
    pub input_token_count: usize,
    pub input_hash: String,
    pub vector: Vec<f32>,
}
#[derive(Debug)]
pub struct IndexBuildInput {
    pub records: Vec<SourceArtifactRecordInput>,
    pub packs: Vec<IndexBuildPack>,
    pub context: SourceArtifactBuildContext,
    pub aliases: Vec<SourceAliasInput>,
    pub remaster_pairs: Vec<SourceRemasterPairInput>,
    pub lexical_units: Vec<SourceLexicalUnitInput>,
    pub semantic_units: Vec<SourceSemanticUnitInput>,
}
impl IndexBuildInput {
    pub fn artifact_record_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePreparedContent {
    pub locator: SourceContentLocator,
    pub role: String,
    pub visibility: ContentVisibilityRule,
    pub outcome: String,
    pub html: Option<String>,
    pub interactions: Vec<ContentInteraction>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceStoredRelationship {
    pub locator: SourceContentLocator,
    pub ordinal: usize,
    pub origin: String,
    pub kind: String,
    pub authored_target: Option<String>,
    pub occurrence_path: Option<String>,
    pub resolution: ContentReferenceResolution,
    pub details: SourceRelationshipDetails,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRelationshipDetails {
    pub availability: Option<atlas_record::source_record::FieldAvailability>,
    pub reference_kind: Option<atlas_record::source_content::ContentReferenceKind>,
    pub audiences: Vec<String>,
}

impl SourceArtifactBuildContext {
    pub fn sha256(&self) -> Result<String, crate::IndexError> {
        Ok(crate::codec::sha256(&serde_json::to_vec(self)?))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceUnitLocation {
    pub unit_id: i64,
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub address: Option<SourcePassageAddress>,
}
