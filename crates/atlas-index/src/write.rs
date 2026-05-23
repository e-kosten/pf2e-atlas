use std::path::Path;

use atlas_embedding::EmbeddingModelId;
use thiserror::Error;

use crate::IndexBuildInput;

#[derive(Debug, Error)]
pub enum IndexWriteError {
    #[error("index write failed: {0}")]
    WriteFailed(String),
}

pub trait IndexArtifactWriter {
    fn label(&self) -> &'static str;
    fn output_path(&self) -> &Path;
    fn write(
        &self,
        input: &IndexBuildInput<'_>,
        embedding_model: EmbeddingModelId,
    ) -> Result<(), IndexWriteError>;
}
