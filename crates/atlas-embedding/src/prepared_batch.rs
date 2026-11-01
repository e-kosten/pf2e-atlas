//! Bounded profile-native inputs checked before registration or actual inference.
use crate::{
    EmbeddingError, EmbeddingModelSpec, TextEmbeddingTokenizer, hash_document_embedding_input,
};

/// Address-free model input; source attribution belongs to the occurrence stream.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PreparedNativeInput {
    pub input: String,
    pub input_sha256: String,
    pub reuse_key: String,
    pub token_count: usize,
}

/// Exact raw input and native budget identity requested from a numerical value source.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeInputDescriptor {
    pub input_sha256: String,
    pub token_count: usize,
}
impl PreparedNativeInput {
    pub fn descriptor(&self) -> NativeInputDescriptor {
        NativeInputDescriptor {
            input_sha256: self.input_sha256.clone(),
            token_count: self.token_count,
        }
    }
}
impl From<crate::PreparedEmbeddingInput> for PreparedNativeInput {
    fn from(input: crate::PreparedEmbeddingInput) -> Self {
        Self {
            input: input.input,
            input_sha256: input.input_sha256,
            reuse_key: input.reuse_key,
            token_count: input.token_count,
        }
    }
}

/// Immutable native counts and execution identities for one verified tokenizer.
/// Keeping fields private prevents callers from changing text after its guard.
pub struct PreparedEmbeddingBatch {
    spec: EmbeddingModelSpec,
    inputs: Vec<PreparedNativeInput>,
}

impl PreparedEmbeddingBatch {
    pub fn prepare(
        tokenizer: &TextEmbeddingTokenizer,
        mut inputs: Vec<PreparedNativeInput>,
    ) -> Result<Self, EmbeddingError> {
        if inputs.len() > crate::NATIVE_INFERENCE_BATCH_LIMIT {
            return Err(EmbeddingError::PreparedInputMismatch);
        }
        let spec = tokenizer.spec();
        for input in &mut inputs {
            if input.input_sha256 != hash_document_embedding_input(&input.input) {
                return Err(EmbeddingError::PreparedInputMismatch);
            }
            input.token_count = tokenizer.validate_document_input(&input.input)?;
            input.reuse_key = spec.document_reuse_key(&input.input);
        }
        Ok(Self { spec, inputs })
    }

    pub fn inputs(&self) -> &[PreparedNativeInput] {
        &self.inputs
    }

    pub(crate) fn spec(&self) -> EmbeddingModelSpec {
        self.spec
    }
}
