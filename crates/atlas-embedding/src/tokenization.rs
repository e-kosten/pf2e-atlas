use std::path::Path;

use tokenizers::{EncodeInput, Tokenizer, TruncationParams};
use tracing::info;

use crate::catalog::{EmbeddingModelSpec, EmbeddingRuntimeConfig};
use crate::document_renderer::{
    EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
};
use crate::error::EmbeddingError;
use crate::text::{normalize_embedding_text, prefixed_text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddingInputTokenization {
    pub token_count: usize,
    pub max_token_count: Option<usize>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetedEmbeddingInput {
    pub text: String,
    pub tokenization: EmbeddingInputTokenization,
    pub truncated_sections: Vec<EmbeddingSectionTruncation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingSectionTruncation {
    pub section: EmbeddingInputSection,
    pub dropped_chunk_count: usize,
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

    pub fn analyze_document_inputs(
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
            phase = "document_embedding_tokenization",
            current = 0_u64,
            total = total as u64,
            "Analyzing document embedding tokenization"
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
                    phase = "document_embedding_tokenization",
                    current = current as u64,
                    total = total as u64,
                    "Analyzed document embedding tokenization"
                );
            }
        }
        info!(
            document_embeddings = total,
            "document embedding tokenization analysis complete"
        );
        Ok(results)
    }

    pub fn budget_document_input(
        &self,
        chunks: &[EmbeddingInputChunk],
    ) -> Result<BudgetedEmbeddingInput, EmbeddingError> {
        let max_token_count = self.spec.max_input_tokens;
        let full_text = render_embedding_chunks_for_embedding(chunks);
        let full_tokenization = self.analyze_single_text(&full_text, self.spec.document_prefix)?;
        let Some(max_token_count) = max_token_count else {
            return Ok(BudgetedEmbeddingInput {
                text: full_text,
                tokenization: full_tokenization,
                truncated_sections: Vec::new(),
            });
        };
        if full_tokenization.token_count <= max_token_count {
            return Ok(BudgetedEmbeddingInput {
                text: full_text,
                tokenization: full_tokenization,
                truncated_sections: Vec::new(),
            });
        }

        let mut accepted = Vec::new();
        let mut truncated_sections = Vec::<EmbeddingSectionTruncation>::new();
        for chunk in chunks {
            let candidate = candidate_text(&accepted, chunk);
            if self.document_token_count(&candidate)? <= max_token_count {
                accepted.push(chunk.clone());
                continue;
            }
            if chunk.truncatable
                && let Some(trimmed_chunk) =
                    self.trim_chunk_to_fit(&accepted, chunk, max_token_count)?
            {
                accepted.push(trimmed_chunk);
            }
            increment_section_truncation(&mut truncated_sections, chunk.section);
        }

        let text = render_embedding_chunks_for_embedding(&accepted);
        let token_count = self.document_token_count(&text)?;
        debug_assert!(token_count <= max_token_count);
        Ok(BudgetedEmbeddingInput {
            text,
            tokenization: EmbeddingInputTokenization {
                token_count: full_tokenization.token_count,
                max_token_count: Some(max_token_count),
                truncated: true,
            },
            truncated_sections,
        })
    }

    fn analyze_single_text(
        &self,
        text: &str,
        prefix: &str,
    ) -> Result<EmbeddingInputTokenization, EmbeddingError> {
        let max_token_count = self.spec.max_input_tokens;
        let mut tokenizer = self.unbounded_tokenizer()?;
        analyze_text(&mut tokenizer, text, prefix, max_token_count)
    }

    fn unbounded_tokenizer(&self) -> Result<Tokenizer, EmbeddingError> {
        let mut tokenizer = self.tokenizer.clone();
        tokenizer
            .with_truncation(None)
            .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;
        tokenizer.with_padding(None);
        Ok(tokenizer)
    }

    fn document_token_count(&self, text: &str) -> Result<usize, EmbeddingError> {
        Ok(self
            .analyze_single_text(text, self.spec.document_prefix)?
            .token_count)
    }

    fn trim_chunk_to_fit(
        &self,
        accepted: &[EmbeddingInputChunk],
        chunk: &EmbeddingInputChunk,
        max_token_count: usize,
    ) -> Result<Option<EmbeddingInputChunk>, EmbeddingError> {
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
            };
            let candidate = candidate_text(accepted, &candidate_chunk);
            if self.document_token_count(&candidate)? <= max_token_count {
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
        }))
    }
}

fn analyze_text(
    tokenizer: &mut Tokenizer,
    text: &str,
    prefix: &str,
    max_token_count: Option<usize>,
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
        .encode(EncodeInput::Single(normalized.into()), true)
        .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;
    let token_count = encoding.get_ids().len();
    Ok(EmbeddingInputTokenization {
        token_count,
        max_token_count,
        truncated: max_token_count.is_some_and(|max| token_count > max),
    })
}

fn tokenization_progress_interval(total: usize) -> usize {
    (total / 100).clamp(500, 5_000)
}

fn candidate_text(accepted: &[EmbeddingInputChunk], chunk: &EmbeddingInputChunk) -> String {
    let mut candidate = render_embedding_chunks_for_embedding(accepted);
    if !candidate.is_empty() {
        candidate.push('\n');
    }
    candidate.push_str(&chunk.text);
    candidate
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
