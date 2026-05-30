use std::path::Path;

use atlas_domain::PackName;
use atlas_embedding::{GeneratedDocumentEmbedding, PendingDocumentEmbedding};
use atlas_record::{NormalizedRecord, RecordAlias, ReferenceEdge, RemasterLink};
use thiserror::Error;

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

    pub fn generated_record_count(&self) -> Result<usize, IndexBuildInputError> {
        let artifact_record_count = self.artifact_record_count();
        artifact_record_count
            .checked_sub(self.source_record_count)
            .ok_or(IndexBuildInputError::InconsistentRecordCounts {
                source_record_count: self.source_record_count,
                artifact_record_count,
            })
    }
}

#[derive(Debug, Error)]
pub enum IndexBuildInputError {
    #[error(
        "source record count {source_record_count} exceeds artifact record count {artifact_record_count}"
    )]
    InconsistentRecordCounts {
        source_record_count: usize,
        artifact_record_count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_record_count_rejects_inconsistent_counts() {
        let input = IndexBuildInput {
            source_signature: "fixture",
            source_record_count: 1,
            packs: Vec::new(),
            records: Vec::new(),
            references: &[],
            aliases: &[],
            remaster_links: &[],
            pending_document_embeddings: &[],
            document_embeddings: &[],
        };

        let error = input
            .generated_record_count()
            .expect_err("source count cannot exceed artifact count");

        assert_eq!(
            error.to_string(),
            "source record count 1 exceeds artifact record count 0"
        );
    }
}
