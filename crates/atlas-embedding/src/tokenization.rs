use std::path::Path;

use tokenizers::{EncodeInput, Tokenizer, TruncationParams};
use tracing::info;

use crate::catalog::{EmbeddingModelSpec, EmbeddingRuntimeConfig};
use crate::document_renderer::{
    EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
};
use crate::error::EmbeddingError;
use crate::text::{normalize_embedding_text, prefixed_text};

#[cfg(test)]
const DOCUMENT_EMBEDDING_TOKENIZATION_PHASE: &str = "document_embedding_tokenization";
#[cfg(test)]
const DOCUMENT_EMBEDDING_TOKENIZATION_MESSAGE: &str = "Analyzing document embedding tokenization";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddingInputTokenization {
    pub token_count: usize,
    pub max_token_count: Option<usize>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BudgetedEmbeddingInput {
    pub text: String,
    pub tokenization: EmbeddingInputTokenization,
    pub final_token_count: usize,
    pub truncated_sections: Vec<EmbeddingSectionTruncation>,
    pub chunk_diagnostics: Vec<EmbeddingChunkBudgetDiagnostic>,
}

pub(crate) struct TokenBudgetTokenizers {
    unbounded: Tokenizer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingSectionTruncation {
    pub section: EmbeddingInputSection,
    pub dropped_chunk_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddingChunkBudgetDiagnostic {
    pub section: EmbeddingInputSection,
    pub outcome: EmbeddingChunkBudgetOutcome,
    pub source_kind: Option<atlas_record::ContentSourceKind>,
    pub group_key: Option<String>,
    pub original_text: String,
    pub final_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmbeddingChunkBudgetOutcome {
    Accepted,
    Trimmed,
    Dropped,
}

impl EmbeddingChunkBudgetOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Trimmed => "trimmed",
            Self::Dropped => "dropped",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextEmbeddingTokenizer {
    spec: EmbeddingModelSpec,
    tokenizer: Tokenizer,
}

impl TextEmbeddingTokenizer {
    pub fn load(config: &EmbeddingRuntimeConfig) -> Result<Self, EmbeddingError> {
        Self::load_from_model_dir(config.model_spec(), config.model_dir())
    }

    pub fn load_from_model_dir(
        spec: EmbeddingModelSpec,
        model_dir: impl AsRef<Path>,
    ) -> Result<Self, EmbeddingError> {
        let tokenizer_path = model_dir.as_ref().join("tokenizer.json");
        let tokenizer = load_tokenizer(&tokenizer_path)?;
        Ok(Self { spec, tokenizer })
    }

    #[cfg(test)]
    pub(crate) fn analyze_document_inputs(
        &self,
        texts: &[&str],
    ) -> Result<Vec<EmbeddingInputTokenization>, EmbeddingError> {
        let max_token_count = self.spec.max_input_tokens;
        let mut tokenizer = self.unbounded_tokenizer()?;
        let total = texts.len();
        let progress_interval = tokenization_progress_interval(total);
        let mut results = Vec::with_capacity(total);
        info!(
            document_embeddings = total,
            "analyzing document embedding tokenization"
        );
        info!(target: "atlas_progress",
            phase = DOCUMENT_EMBEDDING_TOKENIZATION_PHASE,
            current = 0_u64,
            total = total as u64,
            "{DOCUMENT_EMBEDDING_TOKENIZATION_MESSAGE}"
        );
        for (index, text) in texts.iter().enumerate() {
            let current = index + 1;
            results.push(analyze_text(
                &mut tokenizer,
                text,
                self.spec.document_prefix,
                max_token_count,
            )?);
            if current == total || current % progress_interval == 0 {
                info!(
                    analyzed_document_embeddings = current,
                    document_embeddings = total,
                    "analyzed document embedding tokenization batch"
                );
                info!(target: "atlas_progress",
                    phase = DOCUMENT_EMBEDDING_TOKENIZATION_PHASE,
                    current = current as u64,
                    total = total as u64,
                    "{DOCUMENT_EMBEDDING_TOKENIZATION_MESSAGE}"
                );
            }
        }
        info!(
            document_embeddings = total,
            "document embedding tokenization analysis complete"
        );
        Ok(results)
    }

    #[cfg(test)]
    pub(crate) fn budget_document_input(
        &self,
        chunks: &[EmbeddingInputChunk],
    ) -> Result<BudgetedEmbeddingInput, EmbeddingError> {
        let full_text = render_embedding_chunks_for_embedding(chunks);
        let mut tokenizers = self.token_budget_tokenizers()?;
        let full_tokenization = self.analyze_document_input_with(&mut tokenizers, &full_text)?;
        if !full_tokenization.truncated {
            return Ok(BudgetedEmbeddingInput {
                text: full_text,
                tokenization: full_tokenization,
                final_token_count: full_tokenization.token_count,
                truncated_sections: Vec::new(),
                chunk_diagnostics: Vec::new(),
            });
        }
        self.budget_over_limit_document_input_with(&mut tokenizers, chunks, full_tokenization)
    }

    pub(crate) fn analyze_document_input_with(
        &self,
        tokenizers: &mut TokenBudgetTokenizers,
        text: &str,
    ) -> Result<EmbeddingInputTokenization, EmbeddingError> {
        self.single_text_tokenization_with(
            &mut tokenizers.unbounded,
            text,
            self.spec.document_prefix,
        )
    }

    pub(crate) fn budget_over_limit_document_input_with(
        &self,
        tokenizers: &mut TokenBudgetTokenizers,
        chunks: &[EmbeddingInputChunk],
        full_tokenization: EmbeddingInputTokenization,
    ) -> Result<BudgetedEmbeddingInput, EmbeddingError> {
        let max_token_count = full_tokenization.max_token_count.ok_or_else(|| {
            EmbeddingError::ModelRunFailed(
                "cannot budget an over-limit document without a token limit".to_string(),
            )
        })?;
        debug_assert!(full_tokenization.truncated);
        let special_token_overhead =
            self.special_token_overhead_for_chunks(&mut tokenizers.unbounded, chunks)?;
        let tokenized_chunks = self.tokenize_chunks(&mut tokenizers.unbounded, chunks)?;
        let mut accepted = Vec::<(usize, EmbeddingInputChunk)>::new();
        let mut truncated_sections = Vec::<EmbeddingSectionTruncation>::new();
        let mut chunk_diagnostics = chunks
            .iter()
            .map(|chunk| EmbeddingChunkBudgetDiagnostic {
                section: chunk.section,
                outcome: EmbeddingChunkBudgetOutcome::Dropped,
                source_kind: chunk.source_kind,
                group_key: chunk.group_key.clone(),
                original_text: chunk.text.clone(),
                final_text: None,
            })
            .collect::<Vec<_>>();
        let mut estimated_token_count = special_token_overhead;
        for index in prioritized_chunk_indexes(chunks) {
            let chunk = &chunks[index];
            let tokenized = &tokenized_chunks[index];
            if estimated_token_count + tokenized.token_count <= max_token_count {
                accepted.push((index, chunk.clone()));
                estimated_token_count += tokenized.token_count;
                chunk_diagnostics[index].outcome = EmbeddingChunkBudgetOutcome::Accepted;
                chunk_diagnostics[index].final_text = Some(chunk.text.clone());
                continue;
            }
            let remaining = max_token_count.saturating_sub(estimated_token_count);
            if chunk.truncatable
                && let Some(trimmed_chunk) = self.trim_chunk_to_token_budget(
                    &mut tokenizers.unbounded,
                    index,
                    chunk,
                    remaining,
                )?
            {
                estimated_token_count += self.positioned_chunk_token_count(
                    &mut tokenizers.unbounded,
                    index,
                    &trimmed_chunk,
                )?;
                chunk_diagnostics[index].outcome = EmbeddingChunkBudgetOutcome::Trimmed;
                chunk_diagnostics[index].final_text = Some(trimmed_chunk.text.clone());
                accepted.push((index, trimmed_chunk));
            } else {
                chunk_diagnostics[index].outcome = EmbeddingChunkBudgetOutcome::Dropped;
                chunk_diagnostics[index].final_text = None;
            }
            increment_section_truncation(&mut truncated_sections, chunk.section);
        }

        accepted.sort_by_key(|(index, _)| *index);
        let accepted_chunks = accepted
            .iter()
            .map(|(_, chunk)| chunk.clone())
            .collect::<Vec<_>>();
        let text = render_embedding_chunks_for_embedding(&accepted_chunks);
        let token_count = self.document_token_count(&mut tokenizers.unbounded, &text)?;
        if token_count > max_token_count {
            return Err(EmbeddingError::TokenBudgetExceeded {
                estimated: estimated_token_count,
                actual: token_count,
                max: max_token_count,
            });
        }
        Ok(BudgetedEmbeddingInput {
            text,
            tokenization: EmbeddingInputTokenization {
                token_count: full_tokenization.token_count,
                max_token_count: Some(max_token_count),
                truncated: true,
            },
            final_token_count: token_count,
            truncated_sections,
            chunk_diagnostics,
        })
    }

    fn tokenize_chunks(
        &self,
        tokenizer: &mut Tokenizer,
        chunks: &[EmbeddingInputChunk],
    ) -> Result<Vec<TokenizedEmbeddingInputChunk>, EmbeddingError> {
        chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| {
                let token_count = self.positioned_chunk_token_count(tokenizer, index, chunk)?;
                Ok(TokenizedEmbeddingInputChunk { token_count })
            })
            .collect()
    }

    fn special_token_overhead_for_chunks(
        &self,
        tokenizer: &mut Tokenizer,
        chunks: &[EmbeddingInputChunk],
    ) -> Result<usize, EmbeddingError> {
        let Some(chunk) = chunks.first() else {
            return Ok(0);
        };
        let (prefix, text) = self.positioned_chunk_text(0, chunk);
        let without_special_tokens =
            analyze_text_with_special_tokens(tokenizer, &text, prefix, None, false)?.token_count;
        let with_special_tokens =
            analyze_text_with_special_tokens(tokenizer, &text, prefix, None, true)?.token_count;
        Ok(with_special_tokens.saturating_sub(without_special_tokens))
    }

    fn positioned_chunk_token_count(
        &self,
        tokenizer: &mut Tokenizer,
        index: usize,
        chunk: &EmbeddingInputChunk,
    ) -> Result<usize, EmbeddingError> {
        let (prefix, text) = self.positioned_chunk_text(index, chunk);
        Ok(analyze_text_with_special_tokens(tokenizer, &text, prefix, None, false)?.token_count)
    }

    fn positioned_chunk_text<'a>(
        &'a self,
        index: usize,
        chunk: &'a EmbeddingInputChunk,
    ) -> (&'static str, String) {
        if index == 0 {
            (self.spec.document_prefix, chunk.text.clone())
        } else {
            ("", format!("\n{}", chunk.text))
        }
    }

    pub(crate) fn token_budget_tokenizers(&self) -> Result<TokenBudgetTokenizers, EmbeddingError> {
        Ok(TokenBudgetTokenizers {
            unbounded: self.unbounded_tokenizer()?,
        })
    }

    fn unbounded_tokenizer(&self) -> Result<Tokenizer, EmbeddingError> {
        let mut tokenizer = self.tokenizer.clone();
        tokenizer
            .with_truncation(None)
            .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;
        tokenizer.with_padding(None);
        Ok(tokenizer)
    }

    fn document_token_count(
        &self,
        tokenizer: &mut Tokenizer,
        text: &str,
    ) -> Result<usize, EmbeddingError> {
        Ok(analyze_text(
            tokenizer,
            text,
            self.spec.document_prefix,
            self.spec.max_input_tokens,
        )?
        .token_count)
    }

    fn single_text_tokenization_with(
        &self,
        tokenizer: &mut Tokenizer,
        text: &str,
        prefix: &str,
    ) -> Result<EmbeddingInputTokenization, EmbeddingError> {
        analyze_text(tokenizer, text, prefix, self.spec.max_input_tokens)
    }

    fn trim_chunk_to_token_budget(
        &self,
        tokenizer: &mut Tokenizer,
        index: usize,
        chunk: &EmbeddingInputChunk,
        token_budget: usize,
    ) -> Result<Option<EmbeddingInputChunk>, EmbeddingError> {
        if token_budget == 0 {
            return Ok(None);
        }
        let Some((prefix, body)) = chunk.text.split_once(": ") else {
            return Ok(None);
        };
        let words = body.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            return Ok(None);
        }
        let mut low = 0;
        let mut high = words.len();
        while low < high {
            let mid = (low + high).div_ceil(2);
            let candidate_chunk = EmbeddingInputChunk {
                section: chunk.section,
                text: format!("{prefix}: {}", words[..mid].join(" ")),
                truncatable: chunk.truncatable,
                source_kind: chunk.source_kind,
                group_key: chunk.group_key.clone(),
            };
            if self.positioned_chunk_token_count(tokenizer, index, &candidate_chunk)?
                <= token_budget
            {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        if low == 0 {
            return Ok(None);
        }
        Ok(Some(EmbeddingInputChunk {
            section: chunk.section,
            text: format!("{prefix}: {}", words[..low].join(" ")),
            truncatable: chunk.truncatable,
            source_kind: chunk.source_kind,
            group_key: chunk.group_key.clone(),
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenizedEmbeddingInputChunk {
    token_count: usize,
}

fn analyze_text(
    tokenizer: &mut Tokenizer,
    text: &str,
    prefix: &str,
    max_token_count: Option<usize>,
) -> Result<EmbeddingInputTokenization, EmbeddingError> {
    analyze_text_with_special_tokens(tokenizer, text, prefix, max_token_count, true)
}

fn analyze_text_with_special_tokens(
    tokenizer: &mut Tokenizer,
    text: &str,
    prefix: &str,
    max_token_count: Option<usize>,
    add_special_tokens: bool,
) -> Result<EmbeddingInputTokenization, EmbeddingError> {
    let normalized = normalize_embedding_text(&prefixed_text(text, prefix));
    if normalized.is_empty() {
        return Ok(EmbeddingInputTokenization {
            token_count: 0,
            max_token_count,
            truncated: false,
        });
    }
    let encoding = tokenizer
        .encode(EncodeInput::Single(normalized.into()), add_special_tokens)
        .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;
    let token_count = encoding.get_ids().len();
    Ok(EmbeddingInputTokenization {
        token_count,
        max_token_count,
        truncated: max_token_count.is_some_and(|max| token_count > max),
    })
}

#[cfg(test)]
impl TextEmbeddingTokenizer {
    pub(crate) fn whitespace_wordlevel_for_tests(max_input_tokens: usize) -> Self {
        Self::whitespace_wordlevel_with_prefix_for_tests(max_input_tokens, "")
    }

    pub(crate) fn whitespace_wordlevel_with_prefix_for_tests(
        max_input_tokens: usize,
        document_prefix: &'static str,
    ) -> Self {
        let tokenizer = <Tokenizer as std::str::FromStr>::from_str(
            r#"{
                "version": "1.0",
                "truncation": null,
                "padding": null,
                "added_tokens": [],
                "normalizer": null,
                "pre_tokenizer": { "type": "Whitespace" },
                "post_processor": {
                    "type": "TemplateProcessing",
                    "single": [
                        { "SpecialToken": { "id": "[CLS]", "type_id": 0 } },
                        { "Sequence": { "id": "A", "type_id": 0 } },
                        { "SpecialToken": { "id": "[SEP]", "type_id": 0 } }
                    ],
                    "pair": [
                        { "SpecialToken": { "id": "[CLS]", "type_id": 0 } },
                        { "Sequence": { "id": "A", "type_id": 0 } },
                        { "SpecialToken": { "id": "[SEP]", "type_id": 0 } },
                        { "Sequence": { "id": "B", "type_id": 1 } },
                        { "SpecialToken": { "id": "[SEP]", "type_id": 1 } }
                    ],
                    "special_tokens": {
                        "[CLS]": { "id": "[CLS]", "ids": [1], "tokens": ["[CLS]"] },
                        "[SEP]": { "id": "[SEP]", "ids": [2], "tokens": ["[SEP]"] }
                    }
                },
                "decoder": null,
                "model": {
                    "type": "WordLevel",
                    "vocab": {
                        "[UNK]": 0,
                        "[CLS]": 1,
                        "[SEP]": 2
                    },
                    "unk_token": "[UNK]"
                }
            }"#,
        )
        .expect("test tokenizer JSON is valid");
        Self {
            spec: EmbeddingModelSpec {
                provider_family: "test",
                model_id: "test-wordlevel",
                model_revision: "test",
                tokenizer_id: "test-wordlevel",
                max_input_tokens: Some(max_input_tokens),
                pooling: crate::catalog::PoolingStrategy::Mean,
                normalization: crate::catalog::Normalization::L2,
                dimensions: 1,
                dtype: crate::catalog::VectorDType::F32,
                distance_metric: crate::catalog::DistanceMetric::Cosine,
                document_prefix,
                query_prefix: "",
            },
            tokenizer,
        }
    }
}

#[cfg(test)]
fn tokenization_progress_interval(total: usize) -> usize {
    (total / 100).clamp(500, 5_000)
}

fn increment_section_truncation(
    truncated_sections: &mut Vec<EmbeddingSectionTruncation>,
    section: EmbeddingInputSection,
) {
    if let Some(existing) = truncated_sections
        .iter_mut()
        .find(|existing| existing.section == section)
    {
        existing.dropped_chunk_count += 1;
    } else {
        truncated_sections.push(EmbeddingSectionTruncation {
            section,
            dropped_chunk_count: 1,
        });
    }
}

fn prioritized_chunk_indexes(chunks: &[EmbeddingInputChunk]) -> Vec<usize> {
    let mut indexes = (0..chunks.len()).collect::<Vec<_>>();
    indexes.sort_by_key(|index| (chunk_budget_priority(&chunks[*index]), *index));
    indexes
}

fn chunk_budget_priority(chunk: &EmbeddingInputChunk) -> u8 {
    match chunk.section {
        EmbeddingInputSection::Identity => 0,
        EmbeddingInputSection::Traits => 1,
        EmbeddingInputSection::Classification | EmbeddingInputSection::Summary => 2,
        EmbeddingInputSection::Defense
        | EmbeddingInputSection::Movement
        | EmbeddingInputSection::Offense
        | EmbeddingInputSection::Routine
        | EmbeddingInputSection::References => 3,
        EmbeddingInputSection::Description => 4,
        EmbeddingInputSection::Aliases => 5,
        EmbeddingInputSection::Details => 6,
    }
}

pub(crate) fn load_tokenizer(tokenizer_path: &Path) -> Result<Tokenizer, EmbeddingError> {
    info!(path = %tokenizer_path.display(), "loading embedding tokenizer");
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|error| {
        EmbeddingError::TokenizerLoadFailed {
            path: tokenizer_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    info!(path = %tokenizer_path.display(), "loaded embedding tokenizer");
    Ok(tokenizer)
}

pub(crate) fn apply_model_truncation(
    tokenizer: &mut Tokenizer,
    spec: EmbeddingModelSpec,
) -> Result<(), EmbeddingError> {
    if let Some(max_length) = spec.max_input_tokens {
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length,
                ..Default::default()
            }))
            .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::document_renderer::{EmbeddingInputChunk, EmbeddingInputSection};

    use super::*;

    #[test]
    fn chunk_budget_counts_special_tokens_once() {
        let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(6);
        let budgeted = tokenizer
            .budget_document_input(&[
                EmbeddingInputChunk::line(EmbeddingInputSection::Identity, "Name: one"),
                EmbeddingInputChunk::line(EmbeddingInputSection::Description, "Description: two"),
            ])
            .expect("input should fit exactly");

        assert!(!budgeted.tokenization.truncated);
        assert_eq!(budgeted.tokenization.token_count, 6);
        assert_eq!(budgeted.final_token_count, 6);
        assert!(budgeted.chunk_diagnostics.is_empty());
    }

    #[test]
    fn chunk_budget_counts_document_prefix_once() {
        let tokenizer =
            TextEmbeddingTokenizer::whitespace_wordlevel_with_prefix_for_tests(9, "prefix words");
        let budgeted = tokenizer
            .budget_document_input(&[
                EmbeddingInputChunk::line(EmbeddingInputSection::Identity, "Name: one"),
                EmbeddingInputChunk::truncatable_line(
                    EmbeddingInputSection::Description,
                    "Description: two three four five",
                ),
            ])
            .expect("input should budget without double-counting prefix");

        assert!(budgeted.tokenization.truncated);
        assert_eq!(budgeted.final_token_count, 9);
        assert!(budgeted.text.contains("two three"));
    }

    #[test]
    fn chunk_budget_prioritizes_mechanics_and_references_over_description() {
        let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(15);
        let budgeted = tokenizer
            .budget_document_input(&[
                EmbeddingInputChunk::line(EmbeddingInputSection::Identity, "Name: Guardian"),
                EmbeddingInputChunk::line(EmbeddingInputSection::Defense, "AC: 25"),
                EmbeddingInputChunk::line(EmbeddingInputSection::Offense, "Attack: claw +15"),
                EmbeddingInputChunk::truncatable_line(
                    EmbeddingInputSection::Description,
                    "Description: soft flavor text should be truncated before mechanics",
                ),
                EmbeddingInputChunk::truncatable_line(
                    EmbeddingInputSection::References,
                    "References: frightened, fear",
                ),
            ])
            .expect("input should budget mechanics before description");

        assert!(budgeted.tokenization.truncated);
        assert!(budgeted.text.contains("AC: 25"));
        assert!(budgeted.text.contains("Attack: claw +15"));
        assert!(budgeted.text.contains("References: frightened, fear"));
        assert!(!budgeted.text.contains("soft flavor text"));
        assert!(
            budgeted
                .truncated_sections
                .iter()
                .any(|section| section.section == EmbeddingInputSection::Description)
        );
    }
}
