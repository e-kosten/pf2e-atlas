use std::path::Path;

use ort::{inputs, session::Session, value::Tensor};
use tokenizers::{EncodeInput, PaddingDirection, PaddingParams, PaddingStrategy, Tokenizer};
use tracing::info;

use crate::catalog::{EmbeddingModelSpec, EmbeddingRuntimeConfig};
use crate::error::EmbeddingError;
use crate::text::{normalize_embedding_text, prefixed_text};
use crate::tokenization::{apply_model_truncation, load_tokenizer};
use crate::vector_math::mean_pool_normalized;

pub struct TextEmbedder {
    spec: EmbeddingModelSpec,
    tokenizer: Tokenizer,
    session: Session,
}

impl TextEmbedder {
    pub fn load(config: &EmbeddingRuntimeConfig) -> Result<Self, EmbeddingError> {
        Self::load_from_model_dir(config.model_spec(), config.model_dir())
    }

    pub fn load_from_model_dir(
        spec: EmbeddingModelSpec,
        model_dir: impl AsRef<Path>,
    ) -> Result<Self, EmbeddingError> {
        let model_dir = model_dir.as_ref();
        let tokenizer_path = model_dir.join("tokenizer.json");
        let onnx_path = model_dir.join("onnx").join("model.onnx");

        let mut tokenizer = load_tokenizer(&tokenizer_path)?;
        apply_model_truncation(&mut tokenizer, spec)?;
        tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            direction: PaddingDirection::Right,
            pad_to_multiple_of: None,
            pad_id: 0,
            pad_type_id: 0,
            pad_token: "[PAD]".to_string(),
        }));

        info!(path = %onnx_path.display(), "loading embedding ONNX model");
        let session = Session::builder()
            .map_err(|error| EmbeddingError::ModelLoadFailed {
                path: onnx_path.display().to_string(),
                message: error.to_string(),
            })?
            .commit_from_file(&onnx_path)
            .map_err(|error| EmbeddingError::ModelLoadFailed {
                path: onnx_path.display().to_string(),
                message: error.to_string(),
            })?;
        info!(path = %onnx_path.display(), "loaded embedding ONNX model");

        Ok(Self {
            spec,
            tokenizer,
            session,
        })
    }

    pub fn spec(&self) -> EmbeddingModelSpec {
        self.spec
    }

    pub fn embed_query(&mut self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed_text(text, self.spec.query_prefix)
    }

    pub fn embed_document(&mut self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed_text(text, self.spec.document_prefix)
    }

    pub fn embed_documents(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.embed_texts(texts, self.spec.document_prefix)
    }

    fn embed_text(&mut self, text: &str, prefix: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut vectors = self.embed_texts(&[text], prefix)?;
        Ok(vectors.pop().expect("single text returns one vector"))
    }

    fn embed_texts(
        &mut self,
        texts: &[&str],
        prefix: &str,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut vectors = vec![vec![0.0; self.spec.dimensions]; texts.len()];
        let mut non_empty_indices = Vec::new();
        let mut normalized_inputs = Vec::new();
        for (index, text) in texts.iter().enumerate() {
            let normalized = normalize_embedding_text(&prefixed_text(text, prefix));
            if normalized.is_empty() {
                continue;
            }
            non_empty_indices.push(index);
            normalized_inputs.push(EncodeInput::Single(normalized.into()));
        }

        if non_empty_indices.is_empty() {
            return Ok(vectors);
        }

        let encodings = self
            .tokenizer
            .encode_batch(normalized_inputs, true)
            .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;
        let token_count = encodings
            .first()
            .map(|encoding| encoding.get_ids().len())
            .unwrap_or(0);
        let shape = [encodings.len(), token_count];
        let input_ids = Tensor::from_array((
            shape,
            to_i64_vec_batch(&encodings, |encoding| encoding.get_ids()),
        ))
        .map_err(|error| EmbeddingError::TensorPrepareFailed(error.to_string()))?;
        let attention_mask = Tensor::from_array((
            shape,
            to_i64_vec_batch(&encodings, |encoding| encoding.get_attention_mask()),
        ))
        .map_err(|error| EmbeddingError::TensorPrepareFailed(error.to_string()))?;
        let mut session_inputs = inputs![
            "input_ids" => input_ids,
            "attention_mask" => attention_mask,
        ];
        if self
            .session
            .inputs()
            .iter()
            .any(|input| input.name() == "token_type_ids")
        {
            let token_type_ids = Tensor::from_array((
                shape,
                to_i64_vec_batch(&encodings, |encoding| encoding.get_type_ids()),
            ))
            .map_err(|error| EmbeddingError::TensorPrepareFailed(error.to_string()))?;
            session_inputs.push(("token_type_ids".into(), token_type_ids.into()));
        }

        let outputs = self
            .session
            .run(session_inputs)
            .map_err(|error| EmbeddingError::ModelRunFailed(error.to_string()))?;

        let output = if let Some(output) = outputs
            .get("last_hidden_state")
            .or_else(|| outputs.get("output_0"))
        {
            output
        } else if outputs.len() > 0 {
            &outputs[0]
        } else {
            return Err(EmbeddingError::MissingHiddenState);
        };
        let (shape, data) = output
            .try_extract_tensor::<f32>()
            .map_err(|error| EmbeddingError::ModelRunFailed(error.to_string()))?;
        let shape = shape
            .iter()
            .map(|dimension| *dimension as usize)
            .collect::<Vec<_>>();
        if shape.len() != 3 {
            return Err(EmbeddingError::UnexpectedHiddenStateShape(shape));
        }
        let batch = shape[0];
        let tokens = shape[1];
        let dimensions = shape[2];
        if dimensions < self.spec.dimensions {
            return Err(EmbeddingError::DimensionMismatch {
                expected: self.spec.dimensions,
                actual: dimensions,
            });
        }
        for batch_index in 0..batch {
            let output_index = non_empty_indices[batch_index];
            let offset = batch_index * tokens * dimensions;
            vectors[output_index] = mean_pool_normalized(
                &data[offset..offset + (tokens * dimensions)],
                tokens,
                self.spec.dimensions,
                dimensions,
                encodings[batch_index].get_attention_mask(),
            )?;
        }
        Ok(vectors)
    }
}

fn to_i64_vec_batch(
    encodings: &[tokenizers::Encoding],
    values: impl Fn(&tokenizers::Encoding) -> &[u32],
) -> Vec<i64> {
    encodings
        .iter()
        .flat_map(|encoding| values(encoding).iter().copied().map(i64::from))
        .collect()
}
