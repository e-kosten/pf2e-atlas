use std::path::Path;

use atlas_domain::PackName;
use atlas_embedding::{GeneratedDocumentEmbedding, PendingDocumentEmbedding};
use atlas_record::{NormalizedRecord, RecordAlias, ReferenceEdge, RemasterLink};

#[derive(Debug, Clone, Copy)]
pub struct IndexBuildPack<'a> {
    pub name: &'a PackName,
    pub label: &'a str,
    pub document_type: &'a str,
    pub declared_path: &'a str,
    pub resolved_path: &'a Path,
    pub record_count: usize,
}

#[derive(Debug)]
pub struct IndexBuildInput<'a> {
    pub source_signature: &'a str,
    pub source_record_count: usize,
    pub packs: Vec<IndexBuildPack<'a>>,
    pub records: Vec<&'a NormalizedRecord>,
    pub references: &'a [ReferenceEdge],
    pub aliases: &'a [RecordAlias],
    pub remaster_links: &'a [RemasterLink],
    pub pending_document_embeddings: &'a [PendingDocumentEmbedding],
    pub document_embeddings: &'a [GeneratedDocumentEmbedding],
}

impl IndexBuildInput<'_> {
    pub fn artifact_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn generated_record_count(&self) -> usize {
        self.artifact_record_count() - self.source_record_count
    }
}
