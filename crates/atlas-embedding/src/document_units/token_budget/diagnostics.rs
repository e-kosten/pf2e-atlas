use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use atlas_record::ContentSourceKind;
use tracing::info;

use crate::document_renderer::EmbeddingInputSection;
use crate::error::EmbeddingError;
use crate::tokenization::{BudgetedEmbeddingInput, EmbeddingChunkBudgetOutcome};
use crate::unit_kind::EmbeddingUnitKind;

use crate::document_units::model::PendingDocumentEmbedding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DocumentEmbeddingChunkBudgetDiagnostic {
    embedding_unit_key: String,
    record_key: String,
    unit_kind: EmbeddingUnitKind,
    label: Option<String>,
    source_kind: Option<ContentSourceKind>,
    original_token_count: usize,
    final_token_count: usize,
    max_token_count: usize,
    original_chunk_count: usize,
    final_chunk_count: usize,
    chunks: Vec<DocumentEmbeddingChunkBudgetDiagnosticChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentEmbeddingChunkBudgetDiagnosticChunk {
    section: EmbeddingInputSection,
    outcome: EmbeddingChunkBudgetOutcome,
    source_kind: Option<ContentSourceKind>,
    group_key: Option<String>,
    original_text: String,
    final_text: Option<String>,
}

pub(super) fn push_budget_diagnostic(
    diagnostics: &mut Vec<DocumentEmbeddingChunkBudgetDiagnostic>,
    unit: &PendingDocumentEmbedding,
    budgeted: &BudgetedEmbeddingInput,
) {
    diagnostics.push(DocumentEmbeddingChunkBudgetDiagnostic {
        embedding_unit_key: unit.embedding_unit_key.clone(),
        record_key: unit.record_key.clone(),
        unit_kind: unit.unit_kind,
        label: unit.label.clone(),
        source_kind: unit.source_kind,
        original_token_count: budgeted.tokenization.token_count,
        final_token_count: budgeted.final_token_count,
        max_token_count: budgeted.tokenization.max_token_count.unwrap_or(0),
        original_chunk_count: unit.input_chunks.len(),
        final_chunk_count: budgeted
            .chunk_diagnostics
            .iter()
            .filter(|chunk| chunk.final_text.is_some())
            .count(),
        chunks: budgeted
            .chunk_diagnostics
            .iter()
            .map(|chunk| DocumentEmbeddingChunkBudgetDiagnosticChunk {
                section: chunk.section,
                outcome: chunk.outcome,
                source_kind: chunk.source_kind,
                group_key: chunk.group_key.clone(),
                original_text: chunk.original_text.clone(),
                final_text: chunk.final_text.clone(),
            })
            .collect(),
    });
}

pub(super) fn write_embedding_chunk_diagnostics_jsonl(
    path: &Path,
    diagnostics: &[DocumentEmbeddingChunkBudgetDiagnostic],
) -> Result<(), EmbeddingError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| EmbeddingError::DiagnosticWriteFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
    }
    let file = File::create(path).map_err(|error| EmbeddingError::DiagnosticWriteFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    let mut writer = BufWriter::new(file);
    for diagnostic in diagnostics {
        serde_json::to_writer(&mut writer, &embedding_chunk_diagnostic_json(diagnostic)).map_err(
            |error| EmbeddingError::DiagnosticWriteFailed {
                path: path.display().to_string(),
                message: error.to_string(),
            },
        )?;
        writer
            .write_all(b"\n")
            .map_err(|error| EmbeddingError::DiagnosticWriteFailed {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;
    }
    writer
        .flush()
        .map_err(|error| EmbeddingError::DiagnosticWriteFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
    info!(
        path = %path.display(),
        over_limit_embedding_inputs = diagnostics.len(),
        "wrote embedding chunk diagnostics"
    );
    Ok(())
}

fn embedding_chunk_diagnostic_json(
    diagnostic: &DocumentEmbeddingChunkBudgetDiagnostic,
) -> serde_json::Value {
    serde_json::json!({
        "embedding_unit_key": diagnostic.embedding_unit_key,
        "record_key": diagnostic.record_key,
        "unit_kind": diagnostic.unit_kind.as_str(),
        "label": diagnostic.label,
        "source_kind": diagnostic.source_kind.map(|source_kind| source_kind.as_str()),
        "original_token_count": diagnostic.original_token_count,
        "final_token_count": diagnostic.final_token_count,
        "max_token_count": diagnostic.max_token_count,
        "original_chunk_count": diagnostic.original_chunk_count,
        "final_chunk_count": diagnostic.final_chunk_count,
        "chunks": diagnostic.chunks.iter().map(|chunk| {
            serde_json::json!({
                "section": chunk.section.as_str(),
                "outcome": chunk.outcome.as_str(),
                "source_kind": chunk.source_kind.map(|source_kind| source_kind.as_str()),
                "group_key": chunk.group_key,
                "original_text": chunk.original_text,
                "final_text": chunk.final_text,
            })
        }).collect::<Vec<_>>(),
    })
}
