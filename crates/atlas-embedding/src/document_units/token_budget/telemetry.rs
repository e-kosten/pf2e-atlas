use std::collections::{BTreeMap, BTreeSet};

use crate::tokenization::{EmbeddingInputTokenization, EmbeddingSectionTruncation};
use crate::unit_kind::EmbeddingUnitKind;

use crate::document_units::model::{
    DocumentEmbeddingRecordTruncationCoverage, DocumentEmbeddingSectionTruncation,
    DocumentEmbeddingTokenizationTelemetry, DocumentEmbeddingTruncationExample,
    DocumentEmbeddingUnitKindTruncation, PendingDocumentEmbedding,
};

pub(in crate::document_units) fn summarize_document_embedding_tokenization(
    pending: &[PendingDocumentEmbedding],
    tokenizations: &[EmbeddingInputTokenization],
    truncated_sections_by_unit: &BTreeMap<String, Vec<EmbeddingSectionTruncation>>,
) -> DocumentEmbeddingTokenizationTelemetry {
    debug_assert_eq!(pending.len(), tokenizations.len());
    let max_token_count = tokenizations
        .iter()
        .find_map(|tokenization| tokenization.max_token_count);
    let max_observed_token_count = tokenizations
        .iter()
        .map(|tokenization| tokenization.token_count)
        .max()
        .unwrap_or(0);
    let total_observed_token_count = tokenizations
        .iter()
        .map(|tokenization| tokenization.token_count)
        .sum();
    let total_tokens_over_limit = tokenizations
        .iter()
        .filter_map(|tokenization| {
            let max_token_count = tokenization.max_token_count?;
            Some(tokenization.token_count.saturating_sub(max_token_count))
        })
        .sum();
    let truncated_document_count = tokenizations
        .iter()
        .filter(|tokenization| tokenization.truncated)
        .count();
    let record_truncation_coverage = summarize_record_truncation_coverage(pending, tokenizations);
    let mut truncated_examples =
        truncated_examples(pending, tokenizations, truncated_sections_by_unit);
    let unit_kind_truncations = summarize_unit_kind_truncations(&truncated_examples);
    truncated_examples.truncate(10);

    let section_truncations = summarize_section_truncations(truncated_sections_by_unit);

    DocumentEmbeddingTokenizationTelemetry {
        document_count: pending.len(),
        truncated_document_count,
        max_token_count,
        max_observed_token_count,
        total_observed_token_count,
        total_tokens_over_limit,
        unit_kind_truncations,
        record_truncation_coverage,
        section_truncations,
        truncated_examples,
    }
}

fn truncated_examples(
    pending: &[PendingDocumentEmbedding],
    tokenizations: &[EmbeddingInputTokenization],
    truncated_sections_by_unit: &BTreeMap<String, Vec<EmbeddingSectionTruncation>>,
) -> Vec<DocumentEmbeddingTruncationExample> {
    let mut examples = pending
        .iter()
        .zip(tokenizations.iter())
        .filter_map(|(entry, tokenization)| {
            let max_token_count = tokenization.max_token_count?;
            tokenization
                .truncated
                .then(|| DocumentEmbeddingTruncationExample {
                    embedding_unit_key: entry.embedding_unit_key.clone(),
                    record_key: entry.record_key.clone(),
                    unit_kind: entry.unit_kind,
                    label: entry.label.clone(),
                    token_count: tokenization.token_count,
                    max_token_count,
                    truncated_sections: truncated_sections_by_unit
                        .get(&entry.embedding_unit_key)
                        .map(|sections| {
                            sections
                                .iter()
                                .map(|section| section.section.as_str().to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                })
        })
        .collect::<Vec<_>>();
    examples.sort_by(|left, right| {
        right
            .token_count
            .cmp(&left.token_count)
            .then_with(|| left.record_key.cmp(&right.record_key))
            .then_with(|| left.embedding_unit_key.cmp(&right.embedding_unit_key))
    });
    examples
}

fn summarize_unit_kind_truncations(
    truncated_examples: &[DocumentEmbeddingTruncationExample],
) -> Vec<DocumentEmbeddingUnitKindTruncation> {
    let mut by_kind = BTreeMap::<
        String,
        (
            usize,
            BTreeSet<String>,
            usize,
            usize,
            Vec<DocumentEmbeddingTruncationExample>,
        ),
    >::new();
    for example in truncated_examples {
        let tokens_over_limit = example.token_count.saturating_sub(example.max_token_count);
        let kind = example.unit_kind.as_str().to_string();
        let summary = by_kind.entry(kind).or_default();
        summary.0 += 1;
        summary.1.insert(example.record_key.clone());
        summary.2 += tokens_over_limit;
        summary.3 = summary.3.max(example.token_count);
        if summary.4.len() < 5 {
            summary.4.push(example.clone());
        }
    }
    by_kind
        .into_iter()
        .map(
            |(
                unit_kind,
                (
                    unit_count,
                    record_keys,
                    total_tokens_over_limit,
                    max_observed_token_count,
                    examples,
                ),
            )| {
                DocumentEmbeddingUnitKindTruncation {
                    unit_kind,
                    unit_count,
                    record_count: record_keys.len(),
                    total_tokens_over_limit,
                    max_observed_token_count,
                    examples,
                }
            },
        )
        .collect()
}

fn summarize_record_truncation_coverage(
    pending: &[PendingDocumentEmbedding],
    tokenizations: &[EmbeddingInputTokenization],
) -> DocumentEmbeddingRecordTruncationCoverage {
    #[derive(Default)]
    struct RecordState {
        has_child_unit: bool,
        parent_truncated: bool,
        child_truncated: bool,
    }

    let mut by_record = BTreeMap::<String, RecordState>::new();
    for (entry, tokenization) in pending.iter().zip(tokenizations.iter()) {
        let state = by_record.entry(entry.record_key.clone()).or_default();
        if entry.unit_kind == EmbeddingUnitKind::Parent {
            state.parent_truncated |= tokenization.truncated;
        } else {
            state.has_child_unit = true;
            state.child_truncated |= tokenization.truncated;
        }
    }

    let mut coverage = DocumentEmbeddingRecordTruncationCoverage {
        record_count: by_record.len(),
        ..Default::default()
    };
    for state in by_record.values() {
        coverage.records_with_child_units += usize::from(state.has_child_unit);
        coverage.records_with_any_truncated_unit +=
            usize::from(state.parent_truncated || state.child_truncated);
        coverage.records_with_truncated_parent_unit += usize::from(state.parent_truncated);
        coverage.records_with_truncated_child_unit += usize::from(state.child_truncated);
        coverage.records_with_truncated_parent_and_child_units +=
            usize::from(state.parent_truncated && state.has_child_unit);
        coverage.records_with_truncated_parent_and_all_child_units_fit +=
            usize::from(state.parent_truncated && state.has_child_unit && !state.child_truncated);
        coverage.records_with_truncated_parent_without_child_units +=
            usize::from(state.parent_truncated && !state.has_child_unit);
    }
    coverage
}

fn summarize_section_truncations(
    truncated_sections_by_unit: &BTreeMap<String, Vec<EmbeddingSectionTruncation>>,
) -> Vec<DocumentEmbeddingSectionTruncation> {
    let mut by_section = BTreeMap::<String, (usize, usize)>::new();
    for sections in truncated_sections_by_unit.values() {
        for section in sections {
            let entry = by_section
                .entry(section.section.as_str().to_string())
                .or_insert((0, 0));
            entry.0 += 1;
            entry.1 += section.dropped_chunk_count;
        }
    }
    by_section
        .into_iter()
        .map(|(section, (document_count, dropped_chunk_count))| {
            DocumentEmbeddingSectionTruncation {
                section,
                document_count,
                dropped_chunk_count,
            }
        })
        .collect()
}
