use std::path::PathBuf;

use atlas_domain::PackName;
use serde::Deserialize;

use crate::{
    GeneratedDocumentEmbedding, IngestDiagnostics, LoadedRecord, PendingDocumentEmbedding,
    RecordAlias, ReferenceEdge, RemasterLink,
};

#[derive(Debug, Clone)]
pub struct BuildArtifactOptions {
    pub source_root: PathBuf,
    pub output_path: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub embedding_cache_root: Option<PathBuf>,
    pub reuse_embeddings: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactReport {
    pub output_path: PathBuf,
    pub pack_count: usize,
    pub record_count: usize,
    pub source_record_count: usize,
    pub artifact_record_count: usize,
    pub generated_record_count: usize,
    pub pending_document_embedding_count: usize,
    pub document_embedding_count: usize,
    pub reused_document_embedding_count: usize,
    pub generated_document_embedding_count: usize,
    pub source_signature: String,
    pub diagnostics: IngestDiagnostics,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLoad {
    pub manifest_path: PathBuf,
    pub source_signature: String,
    pub source_record_count: usize,
    pub packs: Vec<LoadedPack>,
    pub records: Vec<LoadedRecord>,
    pub references: Vec<ReferenceEdge>,
    pub aliases: Vec<RecordAlias>,
    pub remaster_links: Vec<RemasterLink>,
    pub pending_document_embeddings: Vec<PendingDocumentEmbedding>,
    pub document_embeddings: Vec<GeneratedDocumentEmbedding>,
    pub diagnostics: IngestDiagnostics,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedRecord {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPack {
    pub name: PackName,
    pub label: String,
    pub document_type: String,
    pub declared_path: String,
    pub resolved_path: PathBuf,
    pub record_count: usize,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) packs: Vec<ManifestPack>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManifestPack {
    pub(crate) name: String,
    pub(crate) label: String,
    #[serde(rename = "type")]
    pub(crate) document_type: String,
    pub(crate) path: String,
}

#[derive(Debug)]
pub(crate) struct ParsedManifest {
    pub(crate) manifest: Manifest,
    pub(crate) content_hash: String,
}
