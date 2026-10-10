use std::path::PathBuf;

use serde::Serialize;

/// Source identity and filesystem context shared by loading and preparation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceMetadata {
    pub source_root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_content_hash: String,
}

/// Manifest pack context shared by stages; document outcomes remain stage-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePackMetadata {
    pub name: String,
    pub label: String,
    pub document_type: String,
    pub declared_path: String,
    pub resolved_path: PathBuf,
    /// Every discovered document file has a retained or quarantined outcome.
    pub discovered_file_count: usize,
}
