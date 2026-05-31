mod children;
mod diagnostics;
mod telemetry;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use tracing::info;

use crate::document_input::hash_document_embedding_input;
use crate::error::EmbeddingError;
use crate::tokenization::{
    EmbeddingInputTokenization, EmbeddingSectionTruncation, TextEmbeddingTokenizer,
};
use crate::unit_kind::EmbeddingUnitKind;

use self::children::{impacted_child_groups, materialize_child_units};
use self::diagnostics::{
    DocumentEmbeddingChunkBudgetDiagnostic, push_budget_diagnostic,
    write_embedding_chunk_diagnostics_jsonl,
};
pub(super) use self::telemetry::summarize_document_embedding_tokenization;
use super::model::{
    DocumentEmbeddingTokenizationTelemetry, PendingDocumentEmbedding,
    PendingDocumentEmbeddingCandidate,
};

pub(super) const DOCUMENT_EMBEDDING_BUDGET_PHASE: &str = "document_embedding_budget";

pub fn apply_document_embedding_token_budget(
    pending: &mut Vec<PendingDocumentEmbedding>,
    tokenizer: &TextEmbeddingTokenizer,
) -> Result<DocumentEmbeddingTokenizationTelemetry, EmbeddingError> {
    apply_document_embedding_token_budget_inner(pending, tokenizer, None)
}

pub fn apply_document_embedding_token_budget_with_diagnostic_jsonl(
    pending: &mut Vec<PendingDocumentEmbedding>,
    tokenizer: &TextEmbeddingTokenizer,
    diagnostics_path: impl AsRef<Path>,
) -> Result<DocumentEmbeddingTokenizationTelemetry, EmbeddingError> {
    let mut diagnostics = Vec::new();
    let telemetry =
        apply_document_embedding_token_budget_inner(pending, tokenizer, Some(&mut diagnostics))?;
    write_embedding_chunk_diagnostics_jsonl(diagnostics_path.as_ref(), &diagnostics)?;
    Ok(telemetry)
}

fn apply_document_embedding_token_budget_inner(
    pending: &mut Vec<PendingDocumentEmbedding>,
    tokenizer: &TextEmbeddingTokenizer,
    mut diagnostics: Option<&mut Vec<DocumentEmbeddingChunkBudgetDiagnostic>>,
) -> Result<DocumentEmbeddingTokenizationTelemetry, EmbeddingError> {
    let parent_count = pending
        .iter()
        .filter(|entry| entry.unit_kind == EmbeddingUnitKind::Parent)
        .count();
    let overall_started_at = Instant::now();
    let mut truncated_sections_by_unit = BTreeMap::<String, Vec<EmbeddingSectionTruncation>>::new();
    info!(
        document_embeddings = parent_count,
        "applying document embedding token budget"
    );
    info!(target: "atlas_progress",
        phase = DOCUMENT_EMBEDDING_BUDGET_PHASE,
        current = 0_u64,
        total = parent_count as u64,
        "Fitting parent embedding inputs to token limit"
    );
    let mut processed_parent = 0;
    let mut truncated_count = 0;
    let progress_interval = token_budget_progress_interval(parent_count);

    let mut child_candidates = Vec::<PendingDocumentEmbeddingCandidate>::new();
    let mut final_tokenizations = Vec::<EmbeddingInputTokenization>::new();
    let parent_started_at = Instant::now();
    let mut tokenizers = tokenizer.token_budget_tokenizers()?;
    for entry in pending
        .iter_mut()
        .filter(|entry| entry.unit_kind == EmbeddingUnitKind::Parent)
    {
        let full_tokenization =
            tokenizer.analyze_document_input_with(&mut tokenizers, &entry.input_text)?;
        processed_parent += 1;
        if !full_tokenization.truncated {
            final_tokenizations.push(full_tokenization);
            if processed_parent == parent_count || processed_parent % progress_interval == 0 {
                info!(
                    budgeted_document_embeddings = processed_parent,
                    document_embeddings = parent_count,
                    "budgeted document embedding input batch"
                );
                info!(target: "atlas_progress",
                    phase = DOCUMENT_EMBEDDING_BUDGET_PHASE,
                    current = processed_parent as u64,
                    total = parent_count as u64,
                    "Fitting parent embedding inputs to token limit"
                );
            }
            continue;
        }
        let budgeted = tokenizer.budget_over_limit_document_input_with(
            &mut tokenizers,
            &entry.input_chunks,
            full_tokenization,
        )?;
        truncated_count += 1;
        let impacted_groups = impacted_child_groups(&budgeted.chunk_diagnostics);
        if !budgeted.truncated_sections.is_empty() {
            truncated_sections_by_unit.insert(
                entry.embedding_unit_key.clone(),
                budgeted.truncated_sections.clone(),
            );
        }
        if let Some(diagnostics) = diagnostics.as_deref_mut() {
            push_budget_diagnostic(diagnostics, entry, &budgeted);
        }
        entry.input_text = budgeted.text;
        entry.input_hash = hash_document_embedding_input(&entry.input_text);
        child_candidates.extend(entry.child_candidates.iter().filter_map(|candidate| {
            impacted_groups
                .contains(&candidate.group_key)
                .then_some(candidate.clone())
        }));
        final_tokenizations.push(budgeted.tokenization);
        if processed_parent == parent_count || processed_parent % progress_interval == 0 {
            info!(
                budgeted_document_embeddings = processed_parent,
                document_embeddings = parent_count,
                truncated_document_embeddings = truncated_count,
                "budgeted document embedding input batch"
            );
            info!(target: "atlas_progress",
                phase = DOCUMENT_EMBEDDING_BUDGET_PHASE,
                current = processed_parent as u64,
                total = parent_count as u64,
                "Fitting parent embedding inputs to token limit"
            );
        }
    }
    let parent_duration = parent_started_at.elapsed();
    info!(
        document_embeddings = parent_count,
        truncated_document_embeddings = truncated_count,
        impacted_child_embedding_candidates = child_candidates.len(),
        duration_ms = parent_duration.as_millis(),
        "fit parent embedding inputs to token limit"
    );
    info!(target: "atlas_progress",
        phase = DOCUMENT_EMBEDDING_BUDGET_PHASE,
        "Materializing child embedding inputs"
    );
    let child_units = materialize_child_units(
        child_candidates,
        tokenizer,
        diagnostics,
        &mut truncated_sections_by_unit,
        &mut final_tokenizations,
    )?;
    let summary_started_at = Instant::now();
    info!(target: "atlas_progress",
        phase = DOCUMENT_EMBEDDING_BUDGET_PHASE,
        "Summarizing document embedding tokenization"
    );
    pending.extend(child_units);
    for entry in pending.iter_mut() {
        entry.child_candidates.clear();
    }
    let telemetry = summarize_document_embedding_tokenization(
        pending,
        &final_tokenizations,
        &truncated_sections_by_unit,
    );
    info!(
        truncated_document_embeddings = truncated_count,
        summary_duration_ms = summary_started_at.elapsed().as_millis(),
        total_duration_ms = overall_started_at.elapsed().as_millis(),
        "document embedding token budget complete"
    );
    Ok(telemetry)
}

pub(super) fn token_budget_progress_interval(total: usize) -> usize {
    (total / 100).clamp(25, 500)
}
