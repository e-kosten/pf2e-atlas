use serde::Serialize;

use atlas_foundry_model::{FoundryDocumentSource, SourceAdmission};

use super::metadata::{SourceMetadata, SourcePackMetadata};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFileProvenance {
    pub pack_name: String,
    pub document_type: String,
    /// Path relative to the supplied source root, using '/' separators.
    pub source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFoundryDocument {
    pub provenance: SourceFileProvenance,
    /// SHA-256 of the original bytes, before parsing.
    pub content_hash: String,
    /// Exact file contents, including whitespace, duplicate names and numbers.
    pub bytes: Vec<u8>,
    pub admission: SourceAdmission<FoundryDocumentSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLoadFailureStage {
    Discovery,
    Read,
    Parse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLoadFailure {
    pub provenance: SourceFileProvenance,
    pub stage: SourceLoadFailureStage,
    pub message: String,
}

/// Invalid JSON/non-object envelopes are retained separately, without invented
/// source models. Read failures have neither bytes nor a content hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantinedSourceFile {
    pub failure: SourceLoadFailure,
    pub bytes: Option<Vec<u8>>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFoundryPack {
    pub metadata: SourcePackMetadata,
    pub documents: Vec<LoadedFoundryDocument>,
    pub quarantined_files: Vec<QuarantinedSourceFile>,
    /// Missing/unreadable packs remain visible even when no files are discovered.
    pub discovery_failure: Option<SourceLoadFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFoundrySource {
    pub metadata: SourceMetadata,
    /// Manifest order; document outcomes within each pack use sorted paths.
    pub packs: Vec<LoadedFoundryPack>,
}
