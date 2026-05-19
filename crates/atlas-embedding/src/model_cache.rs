use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{EmbeddingError, EmbeddingRuntimeConfig};
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingModelCacheFile {
    pub source_repo: &'static str,
    pub source_revision: &'static str,
    pub source_path: &'static str,
    pub local_path: PathBuf,
}

pub fn required_embedding_model_cache_files(
    config: &EmbeddingRuntimeConfig,
) -> Vec<EmbeddingModelCacheFile> {
    let spec = config.model_spec();
    let model_dir = config.model_dir();
    vec![
        EmbeddingModelCacheFile {
            source_repo: spec.tokenizer_id,
            source_revision: spec.model_revision,
            source_path: "tokenizer.json",
            local_path: model_dir.join("tokenizer.json"),
        },
        EmbeddingModelCacheFile {
            source_repo: spec.model_id,
            source_revision: spec.model_revision,
            source_path: "onnx/model.onnx",
            local_path: model_dir.join("onnx").join("model.onnx"),
        },
    ]
}

pub fn prepare_embedding_model_cache(
    config: &EmbeddingRuntimeConfig,
) -> Result<Vec<PathBuf>, EmbeddingError> {
    let mut downloaded = Vec::new();
    for file in required_embedding_model_cache_files(config) {
        if file.local_path.is_file() {
            model_cache_progress(
                "embedding_model_cache",
                format!("Embedding model cache already has {}", file.source_path),
            );
            continue;
        }
        model_cache_progress(
            "embedding_model_cache",
            format!("Downloading embedding model file {}", file.source_path),
        );
        download_model_cache_file(&file)?;
        model_cache_progress(
            "embedding_model_cache",
            format!("Cached embedding model file {}", file.source_path),
        );
        downloaded.push(file.local_path);
    }
    Ok(downloaded)
}

fn download_model_cache_file(file: &EmbeddingModelCacheFile) -> Result<(), EmbeddingError> {
    if let Some(parent) = file.local_path.parent() {
        fs::create_dir_all(parent).map_err(|error| EmbeddingError::ModelCachePrepareFailed {
            path: parent.display().to_string(),
            message: error.to_string(),
        })?;
    }
    let temp_path = temp_download_path(&file.local_path);
    let url = hugging_face_resolve_url(file);
    let mut response =
        ureq::get(&url)
            .call()
            .map_err(|error| EmbeddingError::ModelCacheDownloadFailed {
                url: url.clone(),
                path: file.local_path.display().to_string(),
                message: error.to_string(),
            })?;
    let mut output =
        fs::File::create(&temp_path).map_err(|error| EmbeddingError::ModelCachePrepareFailed {
            path: temp_path.display().to_string(),
            message: error.to_string(),
        })?;
    io::copy(&mut response.body_mut().as_reader(), &mut output).map_err(|error| {
        EmbeddingError::ModelCacheDownloadFailed {
            url,
            path: file.local_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    fs::rename(&temp_path, &file.local_path).map_err(|error| {
        EmbeddingError::ModelCachePrepareFailed {
            path: file.local_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    Ok(())
}

fn hugging_face_resolve_url(file: &EmbeddingModelCacheFile) -> String {
    format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        file.source_repo, file.source_revision, file.source_path
    )
}

fn temp_download_path(path: &Path) -> PathBuf {
    path.with_extension(
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) => format!("{extension}.tmp"),
            None => "tmp".to_string(),
        },
    )
}

fn model_cache_progress(phase: &'static str, message: impl AsRef<str>) {
    let message = message.as_ref();
    info!(target: "atlas_progress", phase, "{message}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_EMBEDDING_MODEL;

    #[test]
    fn required_files_use_catalog_model_and_tokenizer_sources() {
        let config = EmbeddingRuntimeConfig::new(DEFAULT_EMBEDDING_MODEL, "/tmp/atlas-models");
        let files = required_embedding_model_cache_files(&config);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].source_repo, "BAAI/bge-small-en-v1.5");
        assert_eq!(files[0].source_path, "tokenizer.json");
        assert_eq!(files[1].source_repo, "BAAI/bge-small-en-v1.5");
        assert_eq!(files[1].source_path, "onnx/model.onnx");
        assert!(files[0].local_path.ends_with("tokenizer.json"));
        assert!(files[1].local_path.ends_with("onnx/model.onnx"));
    }
}
