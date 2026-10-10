use crate::model_cache::read_verified_asset;
use crate::{
    EmbeddingError, EmbeddingModelSpec, EmbeddingRuntimeConfig, default_embedding_model_spec,
};
use std::path::Path;
use tokenizers::Tokenizer;

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
        if spec != default_embedding_model_spec() {
            return Err(EmbeddingError::UnsupportedModelContract);
        }
        let path = model_dir.as_ref().join("tokenizer.json");
        let bytes = read_verified_asset(model_dir.as_ref(), "tokenizer.json")?;
        let tokenizer =
            Tokenizer::from_bytes(bytes).map_err(|e| EmbeddingError::TokenizerLoadFailed {
                path: path.display().to_string(),
                message: e.to_string(),
            })?;
        Self::from_tokenizer(spec, tokenizer)
    }
    pub(crate) fn from_tokenizer(
        spec: EmbeddingModelSpec,
        mut tokenizer: Tokenizer,
    ) -> Result<Self, EmbeddingError> {
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(None)
            .map_err(|e| EmbeddingError::TokenizationFailed(e.to_string()))?;
        Ok(Self { spec, tokenizer })
    }
    pub fn spec(&self) -> EmbeddingModelSpec {
        self.spec
    }
    pub fn count_tokens(&self, text: &str, special_tokens: bool) -> Result<usize, EmbeddingError> {
        self.tokenizer
            .encode(text, special_tokens)
            .map(|e| e.len())
            .map_err(|e| EmbeddingError::TokenizationFailed(e.to_string()))
    }
    pub fn validate_document_input(&self, text: &str) -> Result<usize, EmbeddingError> {
        self.validate_input(text, self.spec.document_prefix)
    }
    pub fn validate_query_input(&self, text: &str) -> Result<usize, EmbeddingError> {
        self.validate_input(text, self.spec.query_prefix)
    }
    fn validate_input(&self, text: &str, prefix: &str) -> Result<usize, EmbeddingError> {
        if text.trim().is_empty() || self.count_tokens(text, false)? == 0 {
            return Err(EmbeddingError::EmptyInput);
        }
        let actual = self.count_tokens(&format!("{prefix}{text}"), true)?;
        let max = self.spec.max_input_tokens.unwrap_or(512);
        if actual > max {
            return Err(EmbeddingError::TokenBudgetExceeded { actual, max });
        }
        Ok(actual)
    }
    pub(crate) fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }
    pub(crate) fn bounded_context<'a>(
        &self,
        text: &'a str,
    ) -> Result<(&'a str, bool), EmbeddingError> {
        let encoded = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| EmbeddingError::TokenizationFailed(e.to_string()))?;
        if encoded.len() <= crate::CONTEXT_TOKEN_BUDGET {
            return Ok((text, false));
        }
        let mut end = encoded.get_offsets()[crate::CONTEXT_TOKEN_BUDGET - 1].1;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let result = &text[..end];
        if self.count_tokens(result, false)? > crate::CONTEXT_TOKEN_BUDGET {
            return Err(EmbeddingError::SplitterFailed(
                "shortened context still exceeds its budget".into(),
            ));
        }
        Ok((result, true))
    }
}
