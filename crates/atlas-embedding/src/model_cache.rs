use crate::catalog::{BGE_MODEL_SHA256, BGE_TOKENIZER_SHA256};
use crate::document_input::hash_bytes;
use crate::{EmbeddingError, EmbeddingRuntimeConfig};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tracing::info;

/// Immutable execution assets, including the configuration required by the
/// local-file FastEmbed API. Cached files are verified before every load.
const ASSETS: &[(&str, &str)] = &[
    ("tokenizer.json", BGE_TOKENIZER_SHA256),
    ("onnx/model.onnx", BGE_MODEL_SHA256),
    (
        "config.json",
        "094f8e891b932f2000c92cfc663bac4c62069f5d8af5b5278c4306aef3084750",
    ),
    (
        "tokenizer_config.json",
        "9261e7d79b44c8195c1cada2b453e55b00aeb81e907a6664974b4d7776172ab3",
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingModelCacheFile {
    pub source_repo: &'static str,
    pub source_revision: &'static str,
    pub source_path: &'static str,
    pub local_path: PathBuf,
    pub sha256: &'static str,
}
pub fn required_embedding_model_cache_files(
    config: &EmbeddingRuntimeConfig,
) -> Vec<EmbeddingModelCacheFile> {
    let spec = config.model_spec();
    ASSETS
        .iter()
        .map(|(source_path, sha256)| EmbeddingModelCacheFile {
            source_repo: spec.model_id,
            source_revision: spec.model_revision,
            source_path,
            local_path: config.model_dir().join(source_path),
            sha256,
        })
        .collect()
}
/// Verify readiness with exactly the same pinned asset policy as inference,
/// without loading a native session or downloading missing files.
pub fn validate_embedding_model_cache(
    config: &EmbeddingRuntimeConfig,
) -> Result<(), EmbeddingError> {
    for file in required_embedding_model_cache_files(config) {
        read_verified_asset(&config.model_dir(), file.source_path)?;
    }
    Ok(())
}
pub(crate) fn read_verified_asset(
    model_dir: &Path,
    source_path: &str,
) -> Result<Vec<u8>, EmbeddingError> {
    let expected = ASSETS
        .iter()
        .find(|(name, _)| *name == source_path)
        .map(|(_, hash)| *hash)
        .ok_or(EmbeddingError::UnsupportedModelContract)?;
    let path = model_dir.join(source_path);
    let bytes = fs::read(&path).map_err(|e| EmbeddingError::ModelLoadFailed {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    verify(&path, &bytes, expected)?;
    Ok(bytes)
}
fn verify(path: &Path, bytes: &[u8], expected: &str) -> Result<(), EmbeddingError> {
    let actual = hash_bytes(bytes);
    if actual != expected {
        return Err(EmbeddingError::AssetChecksumMismatch {
            path: path.display().to_string(),
            expected: expected.into(),
            actual,
        });
    }
    Ok(())
}
pub fn prepare_embedding_model_cache(
    config: &EmbeddingRuntimeConfig,
) -> Result<Vec<PathBuf>, EmbeddingError> {
    let mut downloaded = Vec::new();
    for file in required_embedding_model_cache_files(config) {
        if file.local_path.is_file() {
            read_verified_asset(&config.model_dir(), file.source_path)?;
            continue;
        }
        info!(target: "atlas_progress", phase = "embedding_model_cache", asset = file.source_path, "Downloading pinned embedding asset");
        download(&file)?;
        downloaded.push(file.local_path);
    }
    Ok(downloaded)
}
fn download(file: &EmbeddingModelCacheFile) -> Result<(), EmbeddingError> {
    if let Some(parent) = file.local_path.parent() {
        fs::create_dir_all(parent).map_err(|e| EmbeddingError::ModelCachePrepareFailed {
            path: parent.display().to_string(),
            message: e.to_string(),
        })?;
    }
    let temporary = file.local_path.with_extension("download");
    let result = (|| {
        let url = format!(
            "https://huggingface.co/{}/resolve/{}/{}",
            file.source_repo, file.source_revision, file.source_path
        );
        let mut response =
            ureq::get(&url)
                .call()
                .map_err(|e| EmbeddingError::ModelCacheDownloadFailed {
                    url: url.clone(),
                    path: file.local_path.display().to_string(),
                    message: e.to_string(),
                })?;
        let mut output =
            fs::File::create(&temporary).map_err(|e| EmbeddingError::ModelCachePrepareFailed {
                path: temporary.display().to_string(),
                message: e.to_string(),
            })?;
        io::copy(&mut response.body_mut().as_reader(), &mut output).map_err(|e| {
            EmbeddingError::ModelCacheDownloadFailed {
                url,
                path: temporary.display().to_string(),
                message: e.to_string(),
            }
        })?;
        output
            .sync_all()
            .map_err(|e| EmbeddingError::ModelCachePrepareFailed {
                path: temporary.display().to_string(),
                message: e.to_string(),
            })?;
        verify(
            &temporary,
            &fs::read(&temporary).map_err(|e| EmbeddingError::ModelCachePrepareFailed {
                path: temporary.display().to_string(),
                message: e.to_string(),
            })?,
            file.sha256,
        )?;
        fs::rename(&temporary, &file.local_path).map_err(|e| {
            EmbeddingError::ModelCachePrepareFailed {
                path: file.local_path.display().to_string(),
                message: e.to_string(),
            }
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
