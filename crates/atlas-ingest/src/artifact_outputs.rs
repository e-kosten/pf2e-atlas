use std::path::{Path, PathBuf};

use atlas_embedding::EmbeddingModelId;

use crate::artifact::writer;
use crate::error::IngestError;
use crate::ladybug;
use crate::source::SourceLoad;

pub(crate) trait ArtifactOutputWriter {
    fn label(&self) -> &'static str;
    fn output_path(&self) -> &Path;
    fn write(
        &self,
        source: &SourceLoad,
        embedding_model: EmbeddingModelId,
    ) -> Result<(), IngestError>;
}

pub(crate) struct SqliteArtifactOutput {
    path: PathBuf,
}

impl SqliteArtifactOutput {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ArtifactOutputWriter for SqliteArtifactOutput {
    fn label(&self) -> &'static str {
        "SQLite"
    }

    fn output_path(&self) -> &Path {
        &self.path
    }

    fn write(
        &self,
        source: &SourceLoad,
        embedding_model: EmbeddingModelId,
    ) -> Result<(), IngestError> {
        writer::write_artifact(&self.path, source, embedding_model)
    }
}

pub(crate) struct LadybugArtifactOutput {
    path: PathBuf,
}

impl LadybugArtifactOutput {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ArtifactOutputWriter for LadybugArtifactOutput {
    fn label(&self) -> &'static str {
        "LadybugDB"
    }

    fn output_path(&self) -> &Path {
        &self.path
    }

    fn write(
        &self,
        source: &SourceLoad,
        _embedding_model: EmbeddingModelId,
    ) -> Result<(), IngestError> {
        ladybug::write_artifact(&self.path, source)
    }
}
