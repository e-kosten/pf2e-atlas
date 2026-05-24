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
use crate::unit_kind::EmbeddingUnitKind;

use super::model::{
    DocumentEmbeddingChunkBudgetDiagnostic, DocumentEmbeddingChunkBudgetDiagnosticChunk,
    DocumentEmbeddingRecordTruncationCoverage, DocumentEmbeddingSectionTruncation,
    DocumentEmbeddingTokenizationTelemetry, DocumentEmbeddingTruncationExample,
    DocumentEmbeddingUnitKindTruncation, PendingDocumentEmbedding,
    PendingDocumentEmbeddingCandidate,
};

pub fn apply_document_embedding_token_budget(
    pending: &mut Vec<PendingDocumentEmbedding>,
    tokenizer: &TextEmbeddingTokenizer,
) -> Result<DocumentEmbeddingTokenizationTelemetry, EmbeddingError> {
    apply_document_embedding_token_budget_inner(pending, tokenizer, None)
}

pub fn apply_document_embedding_token_budget_with_diagnostics(
    pending: &mut Vec<PendingDocumentEmbedding>,
    tokenizer: &TextEmbeddingTokenizer,
    diagnostics: &mut Vec<DocumentEmbeddingChunkBudgetDiagnostic>,
) -> Result<DocumentEmbeddingTokenizationTelemetry, EmbeddingError> {
    apply_document_embedding_token_budget_inner(pending, tokenizer, Some(diagnostics))
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
        phase = "document_embedding_analysis",
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
                    phase = "document_embedding_analysis",
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
                budgeted.truncated_sections,
            );
        }
        if let Some(diagnostics) = diagnostics.as_deref_mut() {
            diagnostics.push(DocumentEmbeddingChunkBudgetDiagnostic {
                embedding_unit_key: entry.embedding_unit_key.clone(),
                record_key: entry.record_key.clone(),
                unit_kind: entry.unit_kind,
                label: entry.label.clone(),
                source_kind: entry.source_kind,
                original_token_count: budgeted.tokenization.token_count,
                final_token_count: budgeted.final_token_count,
                max_token_count: budgeted.tokenization.max_token_count.unwrap_or(0),
                original_chunk_count: entry.input_chunks.len(),
                final_chunk_count: budgeted
                    .chunk_diagnostics
                    .iter()
                    .filter(|chunk| chunk.final_text.is_some())
                    .count(),
                chunks: budgeted
                    .chunk_diagnostics
                    .into_iter()
                    .map(|chunk| DocumentEmbeddingChunkBudgetDiagnosticChunk {
                        section: chunk.section,
                        outcome: chunk.outcome,
                        source_kind: chunk.source_kind,
                        group_key: chunk.group_key,
                        original_text: chunk.original_text,
                        final_text: chunk.final_text,
                    })
                    .collect(),
            });
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
                phase = "document_embedding_analysis",
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
        phase = "document_embedding_analysis",
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
        phase = "document_embedding_analysis",
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

fn is_child_embedding_source(source_kind: ContentSourceKind) -> bool {
    matches!(
        source_kind,
        ContentSourceKind::Description
            | ContentSourceKind::DetailsDescription
            | ContentSourceKind::PublicNotes
    )
}

fn impacted_child_groups(
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

fn materialize_child_units(
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
        phase = "document_embedding_analysis",
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
                truncated_sections_by_unit
                    .insert(unit.embedding_unit_key.clone(), budgeted.truncated_sections);
            }
            if let Some(diagnostics) = diagnostics.as_deref_mut() {
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
                        .into_iter()
                        .map(|chunk| DocumentEmbeddingChunkBudgetDiagnosticChunk {
                            section: chunk.section,
                            outcome: chunk.outcome,
                            source_kind: chunk.source_kind,
                            group_key: chunk.group_key,
                            original_text: chunk.original_text,
                            final_text: chunk.final_text,
                        })
                        .collect(),
                });
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
                phase = "document_embedding_analysis",
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

fn token_budget_progress_interval(total: usize) -> usize {
    (total / 100).clamp(25, 500)
}

pub(super) fn summarize_document_embedding_tokenization(
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

#[cfg(test)]
mod tests {
    use atlas_record::ContentSourceKind;

    use crate::document_input::hash_document_embedding_input;
    use crate::document_renderer::{
        EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
    };
    use crate::tokenization::TextEmbeddingTokenizer;

    use super::*;

    #[test]
    fn child_embedding_sources_are_limited_to_rich_content() {
        assert!(is_child_embedding_source(ContentSourceKind::Description));
        assert!(is_child_embedding_source(
            ContentSourceKind::DetailsDescription
        ));
        assert!(is_child_embedding_source(ContentSourceKind::PublicNotes));
        assert!(!is_child_embedding_source(ContentSourceKind::Blurb));
        assert!(!is_child_embedding_source(ContentSourceKind::Routine));
    }

    #[test]
    fn token_budget_materializes_only_impacted_rich_child_groups() {
        let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(180);
        let input_chunks = vec![
            identity_chunk(),
            rich_chunk(
                ContentSourceKind::Description,
                "description:0:description",
                "Description accepted rich content",
                70,
            ),
            rich_chunk(
                ContentSourceKind::PublicNotes,
                "public_notes:1:public_notes",
                "Public notes impacted rich content",
                130,
            ),
            rich_chunk(
                ContentSourceKind::Blurb,
                "blurb:2:summary",
                "Summary non rich content",
                130,
            ),
        ];
        let input_text = render_embedding_chunks_for_embedding(&input_chunks);
        let mut pending = vec![PendingDocumentEmbedding {
            embedding_unit_key: "packs:test#parent".to_string(),
            record_key: "packs:test".to_string(),
            unit_kind: EmbeddingUnitKind::Parent,
            label: None,
            source_kind: None,
            ordinal: 0,
            input_chunks,
            input_hash: hash_document_embedding_input(&input_text),
            input_text,
            child_candidates: vec![
                child_candidate(
                    "packs:test#heading_section:1",
                    ContentSourceKind::Description,
                    "description:0:description",
                    "Description",
                    70,
                ),
                child_candidate(
                    "packs:test#heading_section:2",
                    ContentSourceKind::PublicNotes,
                    "public_notes:1:public_notes",
                    "Public Notes",
                    130,
                ),
                child_candidate(
                    "packs:test#heading_section:3",
                    ContentSourceKind::Blurb,
                    "blurb:2:summary",
                    "Summary",
                    130,
                ),
            ],
        }];

        let telemetry = apply_document_embedding_token_budget(&mut pending, &tokenizer)
            .expect("budgeting should succeed");

        assert_eq!(
            pending
                .iter()
                .map(|entry| entry.embedding_unit_key.as_str())
                .collect::<Vec<_>>(),
            vec!["packs:test#parent", "packs:test#heading_section:2"]
        );
        assert!(
            pending
                .iter()
                .all(|entry| entry.child_candidates.is_empty())
        );
        let child = pending
            .iter()
            .find(|entry| entry.embedding_unit_key == "packs:test#heading_section:2")
            .expect("impacted public notes child materializes");
        assert_eq!(child.source_kind, Some(ContentSourceKind::PublicNotes));
        assert!(child.input_text.contains("Public Notes"));
        assert!(child.input_text.contains("token0"));
        assert!(child.input_text.contains("token129"));
        assert_eq!(telemetry.document_count, 2);
        assert_eq!(
            telemetry
                .record_truncation_coverage
                .records_with_child_units,
            1
        );
    }

    #[test]
    fn token_budget_materializes_short_impacted_rich_child_groups() {
        let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(40);
        let input_chunks = vec![
            identity_chunk(),
            rich_chunk(
                ContentSourceKind::Description,
                "description:0:description",
                "Description",
                20,
            ),
            rich_chunk(
                ContentSourceKind::PublicNotes,
                "public_notes:1:public_notes",
                "Short Public Notes",
                20,
            ),
        ];
        let input_text = render_embedding_chunks_for_embedding(&input_chunks);
        let mut pending = vec![PendingDocumentEmbedding {
            embedding_unit_key: "packs:test#parent".to_string(),
            record_key: "packs:test".to_string(),
            unit_kind: EmbeddingUnitKind::Parent,
            label: None,
            source_kind: None,
            ordinal: 0,
            input_chunks,
            input_hash: hash_document_embedding_input(&input_text),
            input_text,
            child_candidates: vec![child_candidate(
                "packs:test#heading_section:1",
                ContentSourceKind::PublicNotes,
                "public_notes:1:public_notes",
                "Short Public Notes",
                20,
            )],
        }];

        apply_document_embedding_token_budget(&mut pending, &tokenizer)
            .expect("budgeting should succeed");

        assert_eq!(pending.len(), 2);
        assert!(
            pending
                .iter()
                .any(|entry| entry.embedding_unit_key == "packs:test#heading_section:1")
        );
    }

    fn identity_chunk() -> EmbeddingInputChunk {
        EmbeddingInputChunk::line(EmbeddingInputSection::Identity, "Name: Test Record")
    }

    fn child_candidate(
        embedding_unit_key: &str,
        source_kind: ContentSourceKind,
        group_key: &str,
        label: &str,
        repeated_words: usize,
    ) -> PendingDocumentEmbeddingCandidate {
        PendingDocumentEmbeddingCandidate {
            embedding_unit_key: embedding_unit_key.to_string(),
            record_key: "packs:test".to_string(),
            unit_kind: EmbeddingUnitKind::HeadingSection,
            label: Some(label.to_string()),
            source_kind,
            group_key: group_key.to_string(),
            ordinal: 1,
            input_chunks: vec![
                identity_chunk(),
                rich_chunk(source_kind, group_key, label, repeated_words),
            ],
        }
    }

    fn rich_chunk(
        source_kind: ContentSourceKind,
        group_key: &str,
        label: &str,
        repeated_words: usize,
    ) -> EmbeddingInputChunk {
        EmbeddingInputChunk::truncatable_line(
            EmbeddingInputSection::Description,
            format!("{label}: {}", repeated_tokens(repeated_words)),
        )
        .with_source_kind(source_kind)
        .with_group_key(group_key)
    }

    fn repeated_tokens(count: usize) -> String {
        (0..count)
            .map(|index| format!("token{index}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
