use std::path::Path;

use tokenizers::{EncodeInput, Tokenizer, TruncationParams};
use tracing::info;

use crate::catalog::{EmbeddingModelSpec, EmbeddingRuntimeConfig};
use crate::error::EmbeddingError;
use crate::text::{normalize_embedding_text, prefixed_text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddingInputTokenization {
    pub token_count: usize,
    pub max_token_count: Option<usize>,
    pub truncated: bool,
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
        self.analyze_texts(texts, self.spec.document_prefix)
    }

    fn analyze_texts(
        &self,
        texts: &[&str],
        prefix: &str,
    ) -> Result<Vec<EmbeddingInputTokenization>, EmbeddingError> {
        let max_token_count = self.spec.max_input_tokens;
        let mut tokenizer = self.tokenizer.clone();
        tokenizer
            .with_truncation(None)
            .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;
        tokenizer.with_padding(None);

        texts
            .iter()
            .map(|text| {
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
            })
            .collect()
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
