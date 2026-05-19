use std::path::PathBuf;
use std::str::FromStr;

use atlas_domain::PackName;
use atlas_embedding::{
    DEFAULT_EMBEDDING_MODEL, DocumentEmbeddingRecordTruncationCoverage,
    DocumentEmbeddingSectionTruncation, DocumentEmbeddingTokenizationTelemetry,
    DocumentEmbeddingTruncationExample, DocumentEmbeddingUnitKindTruncation, EmbeddingModelId,
    GeneratedDocumentEmbedding, PendingDocumentEmbedding,
};
use serde::Deserialize;

use crate::diagnostics::IngestDiagnostics;
use crate::error::IngestError;
use crate::records::{LoadedSourceRecord, RecordAlias, ReferenceEdge, RemasterLink};

#[derive(Debug, Clone)]
pub struct BuildArtifactOptions {
    pub source_root: PathBuf,
    pub output_path: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub embedding_model_id: String,
    pub embedding_cache_root: Option<PathBuf>,
    pub reuse_embeddings: bool,
    pub embedding_batch_size: usize,
}

impl BuildArtifactOptions {
    pub fn default_embedding_model_id() -> String {
        DEFAULT_EMBEDDING_MODEL.to_string()
    }

    pub(crate) fn embedding_model(&self) -> Result<EmbeddingModelId, IngestError> {
        EmbeddingModelId::from_str(&self.embedding_model_id).map_err(|error| {
            IngestError::InvalidEmbeddingModel {
                model: self.embedding_model_id.clone(),
                message: error.to_string(),
            }
        })
    }
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
    pub document_embedding_tokenization: DocumentEmbeddingTokenizationReport,
    pub embedding_timing: EmbeddingTimingReport,
    pub build_duration_ms: u128,
    pub source_signature: String,
    pub diagnostics: IngestDiagnostics,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmbeddingTimingReport {
    pub tokenization_duration_ms: u128,
    pub model_load_duration_ms: u128,
    pub generation_duration_ms: u128,
    pub batch_count: usize,
    pub batch_duration_min_ms: Option<u128>,
    pub batch_duration_p50_ms: Option<u128>,
    pub batch_duration_p95_ms: Option<u128>,
    pub batch_duration_max_ms: Option<u128>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentEmbeddingTokenizationReport {
    pub document_count: usize,
    pub truncated_document_count: usize,
    pub max_token_count: Option<usize>,
    pub max_observed_token_count: usize,
    pub total_observed_token_count: usize,
    pub total_tokens_over_limit: usize,
    pub unit_kind_truncations: Vec<DocumentEmbeddingUnitKindTruncationReport>,
    pub record_truncation_coverage: DocumentEmbeddingRecordTruncationCoverageReport,
    pub section_truncations: Vec<DocumentEmbeddingSectionTruncationReport>,
    pub truncated_examples: Vec<DocumentEmbeddingTruncationExampleReport>,
}

impl DocumentEmbeddingTokenizationReport {
    pub(crate) fn from_embedding_telemetry(
        telemetry: DocumentEmbeddingTokenizationTelemetry,
    ) -> Self {
        Self {
            document_count: telemetry.document_count,
            truncated_document_count: telemetry.truncated_document_count,
            max_token_count: telemetry.max_token_count,
            max_observed_token_count: telemetry.max_observed_token_count,
            total_observed_token_count: telemetry.total_observed_token_count,
            total_tokens_over_limit: telemetry.total_tokens_over_limit,
            unit_kind_truncations: telemetry
                .unit_kind_truncations
                .into_iter()
                .map(DocumentEmbeddingUnitKindTruncationReport::from_embedding_truncation)
                .collect(),
            record_truncation_coverage:
                DocumentEmbeddingRecordTruncationCoverageReport::from_embedding_coverage(
                    telemetry.record_truncation_coverage,
                ),
            section_truncations: telemetry
                .section_truncations
                .into_iter()
                .map(DocumentEmbeddingSectionTruncationReport::from_embedding_section)
                .collect(),
            truncated_examples: telemetry
                .truncated_examples
                .into_iter()
                .map(DocumentEmbeddingTruncationExampleReport::from_embedding_example)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingTruncationExampleReport {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub token_count: usize,
    pub max_token_count: usize,
    pub truncated_sections: Vec<String>,
}

impl DocumentEmbeddingTruncationExampleReport {
    fn from_embedding_example(example: DocumentEmbeddingTruncationExample) -> Self {
        Self {
            embedding_unit_key: example.embedding_unit_key,
            record_key: example.record_key,
            unit_kind: example.unit_kind.as_str().to_string(),
            label: example.label,
            token_count: example.token_count,
            max_token_count: example.max_token_count,
            truncated_sections: example.truncated_sections,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentEmbeddingUnitKindTruncationReport {
    pub unit_kind: String,
    pub unit_count: usize,
    pub record_count: usize,
    pub total_tokens_over_limit: usize,
    pub max_observed_token_count: usize,
    pub examples: Vec<DocumentEmbeddingTruncationExampleReport>,
}

impl DocumentEmbeddingUnitKindTruncationReport {
    fn from_embedding_truncation(truncation: DocumentEmbeddingUnitKindTruncation) -> Self {
        Self {
            unit_kind: truncation.unit_kind,
            unit_count: truncation.unit_count,
            record_count: truncation.record_count,
            total_tokens_over_limit: truncation.total_tokens_over_limit,
            max_observed_token_count: truncation.max_observed_token_count,
            examples: truncation
                .examples
                .into_iter()
                .map(DocumentEmbeddingTruncationExampleReport::from_embedding_example)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentEmbeddingRecordTruncationCoverageReport {
    pub record_count: usize,
    pub records_with_child_units: usize,
    pub records_with_any_truncated_unit: usize,
    pub records_with_truncated_parent_unit: usize,
    pub records_with_truncated_child_unit: usize,
    pub records_with_truncated_parent_and_child_units: usize,
    pub records_with_truncated_parent_and_all_child_units_fit: usize,
    pub records_with_truncated_parent_without_child_units: usize,
}

impl DocumentEmbeddingRecordTruncationCoverageReport {
    fn from_embedding_coverage(coverage: DocumentEmbeddingRecordTruncationCoverage) -> Self {
        Self {
            record_count: coverage.record_count,
            records_with_child_units: coverage.records_with_child_units,
            records_with_any_truncated_unit: coverage.records_with_any_truncated_unit,
            records_with_truncated_parent_unit: coverage.records_with_truncated_parent_unit,
            records_with_truncated_child_unit: coverage.records_with_truncated_child_unit,
            records_with_truncated_parent_and_child_units: coverage
                .records_with_truncated_parent_and_child_units,
            records_with_truncated_parent_and_all_child_units_fit: coverage
                .records_with_truncated_parent_and_all_child_units_fit,
            records_with_truncated_parent_without_child_units: coverage
                .records_with_truncated_parent_without_child_units,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingSectionTruncationReport {
    pub section: String,
    pub document_count: usize,
    pub dropped_chunk_count: usize,
}

impl DocumentEmbeddingSectionTruncationReport {
    fn from_embedding_section(section: DocumentEmbeddingSectionTruncation) -> Self {
        Self {
            section: section.section,
            document_count: section.document_count,
            dropped_chunk_count: section.dropped_chunk_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceLoad {
    pub(crate) manifest_path: PathBuf,
    pub(crate) source_signature: String,
    pub(crate) source_record_count: usize,
    pub(crate) packs: Vec<LoadedPack>,
    pub(crate) records: Vec<LoadedSourceRecord>,
    pub(crate) references: Vec<ReferenceEdge>,
    pub(crate) aliases: Vec<RecordAlias>,
    pub(crate) remaster_links: Vec<RemasterLink>,
    pub(crate) pending_document_embeddings: Vec<PendingDocumentEmbedding>,
    pub(crate) document_embeddings: Vec<GeneratedDocumentEmbedding>,
    pub(crate) document_embedding_tokenization: DocumentEmbeddingTokenizationTelemetry,
    pub(crate) diagnostics: IngestDiagnostics,
    pub(crate) skipped_records: Vec<SkippedRecord>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedRecord {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoadedPack {
    pub(crate) name: PackName,
    pub(crate) label: String,
    pub(crate) document_type: String,
    pub(crate) declared_path: String,
    pub(crate) resolved_path: PathBuf,
    pub(crate) record_count: usize,
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
