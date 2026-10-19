//! Profile-native inputs checked once before checkpoint lookup or inference.
use crate::{
    EmbeddingError, EmbeddingModelSpec, PreparedEmbeddingInput, TextEmbeddingTokenizer,
    hash_document_embedding_input,
};

/// Immutable native counts and execution identities for one verified tokenizer.
/// Keeping fields private prevents callers from changing text after its guard.
pub struct PreparedEmbeddingBatch {
    spec: EmbeddingModelSpec,
    inputs: Vec<PreparedEmbeddingInput>,
}

impl PreparedEmbeddingBatch {
    pub fn prepare(
        tokenizer: &TextEmbeddingTokenizer,
        mut inputs: Vec<PreparedEmbeddingInput>,
    ) -> Result<Self, EmbeddingError> {
        let spec = tokenizer.spec();
        let total = inputs.len();
        crate::inference::embedding_progress(
            spec,
            "native_validation",
            0,
            total,
            "occurrences",
            None,
            total,
        );
        for (ordinal, input) in inputs.iter_mut().enumerate() {
            if input.input_sha256 != hash_document_embedding_input(&input.input) {
                return Err(EmbeddingError::PreparedInputMismatch);
            }
            input.token_count = tokenizer.validate_document_input(&input.input)?;
            input.reuse_key = spec.document_reuse_key(&input.input);
            if (ordinal + 1).is_multiple_of(1000) || ordinal + 1 == total {
                crate::inference::embedding_progress(
                    spec,
                    "native_validation",
                    ordinal + 1,
                    total,
                    "occurrences",
                    None,
                    total,
                );
            }
        }
        Ok(Self { spec, inputs })
    }

    pub fn inputs(&self) -> &[PreparedEmbeddingInput] {
        &self.inputs
    }

    pub(crate) fn spec(&self) -> EmbeddingModelSpec {
        self.spec
    }
}
