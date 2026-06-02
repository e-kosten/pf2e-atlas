use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use atlas_record::ContentSourceKind;
use tracing::info;

use crate::document_input::hash_document_embedding_input;
use crate::document_renderer::render_embedding_chunks_for_embedding;
use crate::error::EmbeddingError;
use crate::tokenization::{
    EmbeddingChunkBudgetOutcome, EmbeddingInputTokenization, EmbeddingSectionTruncation,
    TextEmbeddingTokenizer,
};

use super::diagnostics::{DocumentEmbeddingChunkBudgetDiagnostic, push_budget_diagnostic};
use super::{DOCUMENT_EMBEDDING_BUDGET_PHASE, token_budget_progress_interval};
use crate::document_units::model::{PendingDocumentEmbedding, PendingDocumentEmbeddingCandidate};

pub(super) fn is_child_embedding_source(source_kind: ContentSourceKind) -> bool {
    matches!(
        source_kind,
        ContentSourceKind::Description
            | ContentSourceKind::DetailsFieldDescription
            | ContentSourceKind::PublicNotes
    )
}

pub(super) fn impacted_child_groups(
    diagnostics: &[crate::tokenization::EmbeddingChunkBudgetDiagnostic],
) -> BTreeSet<String> {
    diagnostics
        .iter()
        .filter(|chunk| {
            chunk.outcome != EmbeddingChunkBudgetOutcome::Accepted
                && chunk.source_kind.is_some_and(is_child_embedding_source)
        })
        .filter_map(|chunk| chunk.group_key.clone())
        .collect()
}

pub(super) fn materialize_child_units(
    candidates: Vec<PendingDocumentEmbeddingCandidate>,
    tokenizer: &TextEmbeddingTokenizer,
    mut diagnostics: Option<&mut Vec<DocumentEmbeddingChunkBudgetDiagnostic>>,
    truncated_sections_by_unit: &mut BTreeMap<String, Vec<EmbeddingSectionTruncation>>,
    final_tokenizations: &mut Vec<EmbeddingInputTokenization>,
) -> Result<Vec<PendingDocumentEmbedding>, EmbeddingError> {
    let candidate_count = candidates.len();
    let materialize_started_at = Instant::now();
    let mut units = Vec::with_capacity(candidate_count);
    for candidate in candidates {
        let input_text = render_embedding_chunks_for_embedding(&candidate.input_chunks);
        let input_hash = hash_document_embedding_input(&input_text);
        units.push(PendingDocumentEmbedding {
            embedding_unit_key: candidate.embedding_unit_key,
            record_key: candidate.record_key,
            unit_kind: candidate.unit_kind,
            label: candidate.label,
            source_kind: Some(candidate.source_kind),
            ordinal: candidate.ordinal,
            input_chunks: candidate.input_chunks,
            input_text,
            input_hash,
            child_candidates: Vec::new(),
        });
    }
    info!(
        child_embedding_inputs = units.len(),
        duration_ms = materialize_started_at.elapsed().as_millis(),
        "materialized child embedding inputs"
    );

    info!(target: "atlas_progress",
        phase = DOCUMENT_EMBEDDING_BUDGET_PHASE,
        current = 0_u64,
        total = units.len() as u64,
        "Fitting child embedding inputs to token limit"
    );
    let fit_started_at = Instant::now();
    let progress_interval = token_budget_progress_interval(units.len());
    let total_units = units.len();
    let mut processed_units = 0;
    let mut truncated_units = 0;
    let mut tokenizers = tokenizer.token_budget_tokenizers()?;
    for unit in &mut units {
        let full_tokenization =
            tokenizer.analyze_document_input_with(&mut tokenizers, &unit.input_text)?;
        if !full_tokenization.truncated {
            final_tokenizations.push(full_tokenization);
        } else {
            let budgeted = tokenizer.budget_over_limit_document_input_with(
                &mut tokenizers,
                &unit.input_chunks,
                full_tokenization,
            )?;
            truncated_units += 1;
            if !budgeted.truncated_sections.is_empty() {
                truncated_sections_by_unit.insert(
                    unit.embedding_unit_key.clone(),
                    budgeted.truncated_sections.clone(),
                );
            }
            if let Some(diagnostics) = diagnostics.as_deref_mut() {
                push_budget_diagnostic(diagnostics, unit, &budgeted);
            }
            unit.input_text = budgeted.text;
            unit.input_hash = hash_document_embedding_input(&unit.input_text);
            final_tokenizations.push(budgeted.tokenization);
        }
        processed_units += 1;
        if processed_units == total_units || processed_units % progress_interval == 0 {
            info!(
                fit_child_embedding_inputs = processed_units,
                child_embedding_inputs = total_units,
                truncated_child_embedding_inputs = truncated_units,
                "fit child embedding input batch"
            );
            info!(target: "atlas_progress",
                phase = DOCUMENT_EMBEDDING_BUDGET_PHASE,
                current = processed_units as u64,
                total = total_units as u64,
                "Fitting child embedding inputs to token limit"
            );
        }
    }
    info!(
        child_embedding_inputs = total_units,
        truncated_child_embedding_inputs = truncated_units,
        duration_ms = fit_started_at.elapsed().as_millis(),
        "fit child embedding inputs to token limit"
    );
    Ok(units)
}
