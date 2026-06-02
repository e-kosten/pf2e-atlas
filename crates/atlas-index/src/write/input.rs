use std::path::PathBuf;

use atlas_domain::PackName;
use atlas_embedding::{GeneratedDocumentEmbedding, PendingDocumentEmbedding};
use atlas_record::{AtlasRecord, RecordAlias, ReferenceEdge, RemasterLink};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexBuildPack {
    pub name: PackName,
    pub label: String,
    pub document_type: String,
    pub declared_path: String,
    pub resolved_path: PathBuf,
    pub record_count: usize,
}

#[derive(Debug)]
pub struct IndexBuildInput {
    pub source_signature: String,
    pub source_record_count: usize,
    pub packs: Vec<IndexBuildPack>,
    pub records: Vec<AtlasRecord>,
    pub references: Vec<ReferenceEdge>,
    pub aliases: Vec<RecordAlias>,
    pub remaster_links: Vec<RemasterLink>,
    pub pending_document_embeddings: Vec<PendingDocumentEmbedding>,
    pub document_embeddings: Vec<GeneratedDocumentEmbedding>,
}

impl IndexBuildInput {
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
            source_signature: "fixture".to_string(),
            source_record_count: 1,
            packs: Vec::new(),
            records: Vec::new(),
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
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
