//! Immutable build inputs. Source admission and filesystem discovery remain ingest-owned.
use atlas_domain::OwnedContentLocator;
use atlas_domain::{RecordKey, SearchProfileId, SourcePassageAddress};
use atlas_record::source_content::{
    ContentAudience, ContentInteraction, ContentReferenceResolution, ContentVisibilityRule,
    SourceContentLocator,
};
use atlas_record::source_record::{
    SourceBackedRecord, SourceContentOutcome, SourceRelationshipOccurrence,
};
use serde::{Deserialize, Serialize};

pub const ARTIFACT_FORMAT_VERSION: u32 = 3;
pub const ARTIFACT_CONTRACT_VERSION: &str = "pf2e-atlas-source-artifact/v3";
pub const EXPECTED_SOURCE_KIND: &str = "foundry-pf2e";
pub const SOURCE_PROJECTION_VERSION: &str = "authored-query/v3";
pub const SOURCE_LOOKUP_VERSION: &str = "nfc-lowercase-whitespace/v1";
pub const LEXICAL_SELECTION_VERSION: &str = "curated-lexical/v2";
pub const LOCALIZATION_POLICY_VERSION: &str = "selected-source-catalog/english-fallback/v2";
pub const ASSET_POLICY_VERSION: &str = "safe-remote-images/local-unavailable/v1";
pub const RELATIONSHIP_POLICY_VERSION: &str = "typed-authored-occurrences/v6";

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
pub struct SourceSearchProfileIdentity {
    pub profile_id: SearchProfileId,
    pub provider: String,
    pub model_id: String,
    pub revision: String,
    pub model_sha256: String,
    pub tokenizer_sha256: String,
    pub segmentation_tokenizer_sha256: String,
    pub dimensions: usize,
    pub pooling: String,
    pub normalization: String,
    pub distance_metric: String,
    pub query_prefix: String,
    pub document_prefix: String,
    pub token_limit: usize,
    pub unit_policy_version: String,
    pub tokenizer_id: String,
    pub output_name: String,
    pub dtype: String,
    pub execution_policy: String,
    pub input_recipe: String,
    pub assets: std::collections::BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifactBuildContext {
    #[serde(default)]
    pub core_sections_recipe: String,
    #[serde(default)]
    pub trait_policy: String,
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
    pub search_profiles: Vec<SourceSearchProfileIdentity>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifactDiagnostic {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_diagnostic: Option<atlas_foundry_model::SourceDiagnostic>,
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
    RootAlias,
    OwnedName,
    Heading,
    DefinitionLabel,
}
impl SourceLexicalUnitKind {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::RootName => "root_name",
            Self::RootAlias => "root_alias",
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
    pub profile_id: SearchProfileId,
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
    pub traits: Vec<crate::TraitDefinition>,
    pub trait_vectors: Vec<crate::TraitVectorInput>,
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
    pub label: Option<atlas_record::source_content::ContentReferenceLabel>,
    pub availability: Option<atlas_record::source_record::FieldAvailability>,
    pub reference_kind: Option<atlas_record::source_content::ContentReferenceKind>,
    pub audiences: Vec<String>,
}

impl SourceArtifactBuildContext {
    pub fn canonical_profiles(
        &self,
    ) -> Result<Vec<SourceSearchProfileIdentity>, crate::IndexError> {
        let mut profiles = self.search_profiles.clone();
        profiles.sort_by(|a, b| a.profile_id.cmp(&b.profile_id));
        if profiles
            .windows(2)
            .any(|pair| pair[0].profile_id == pair[1].profile_id)
        {
            return Err(crate::IndexError::Invalid(
                "duplicate search profile identity".into(),
            ));
        }
        Ok(profiles)
    }
    pub fn sha256(&self) -> Result<String, crate::IndexError> {
        let mut canonical = self.clone();
        canonical.search_profiles = self.canonical_profiles()?;
        Ok(crate::codec::sha256(&serde_json::to_vec(&canonical)?))
    }
    /// Only dependencies of prepared content belong here. Corpus inventory,
    /// query projections and embedding recipes have separate invalidation keys.
    pub fn preparation_sha256(&self) -> Result<String, crate::IndexError> {
        let shared = serde_json::json!({
            "snapshot_version": self.snapshot_version,
            "source_contract": self.source_contract,
            "indexing_locale": self.indexing_locale,
            "locale_catalog_sha256": self.locale_catalog_sha256,
            "english_catalog_sha256": self.english_catalog_sha256,
            "localization_policy": self.localization_policy,
            "audience": self.audience,
            "content_interpretation_version": self.content_interpretation_version,
            "content_selection_version": self.content_selection_version,
            "asset_policy": self.asset_policy,
            "relationship_policy": self.relationship_policy,
        });
        Ok(crate::codec::sha256(&serde_json::to_vec(&shared)?))
    }
    pub fn search_profile(
        &self,
        id: &SearchProfileId,
    ) -> Result<&SourceSearchProfileIdentity, crate::IndexError> {
        self.search_profiles.iter().find(|profile|&profile.profile_id == id).ok_or_else(||crate::IndexError::Unavailable(format!("search profile `{id}` is not stored in this artifact; rebuild with --search-profile {id}")))
    }
}

#[cfg(test)]
mod preparation_tests {
    #[test]
    fn corpus_and_query_changes_do_not_invalidate_prepared_content() {
        let original = crate::foundation_tests::fixture(false).context;
        let mut changed = original.clone();
        changed.source_revision = Some("new revision".into());
        changed.source_fingerprint = "new inventory".into();
        changed.source_file_count += 1;
        changed.format_version += 1;
        changed.query_catalog_version += 1;
        changed.query_projection_version.push_str("-next");
        changed.lookup_version.push_str("-next");
        changed.lexical_selection_version.push_str("-next");
        changed.core_sections_recipe.push_str("-next");
        changed.fts_tokenizer.push_str(" changed");
        assert_eq!(
            original.preparation_sha256().unwrap(),
            changed.preparation_sha256().unwrap()
        );
        assert_ne!(original.sha256().unwrap(), changed.sha256().unwrap());
        changed.content_interpretation_version.push_str("-next");
        assert_ne!(
            original.preparation_sha256().unwrap(),
            changed.preparation_sha256().unwrap()
        );
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
