use crate::catalog::BGE_TOKENIZER_SHA256;
use crate::document_input::hash_bytes;
use crate::{EmbeddingError, EmbeddingModelSpec, EmbeddingRuntimeConfig};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tracing::info;

/// Immutable execution assets, including the configuration required by the
/// local-file FastEmbed API. Cached files are verified before every load.
pub(crate) const BASE_ASSETS: &[(&str, &str)] = &[
    ("tokenizer.json", BGE_TOKENIZER_SHA256),
    (
        "onnx/model.onnx",
        "9bc579acdba21c253c62a9bf866891355a63ffa3442b52c8a37d75b2ccb91848",
    ),
    (
        "config.json",
        "bc00af31a4a31b74040d73370aa83b62da34c90b75eb77bfa7db039d90abd591",
    ),
    (
        "tokenizer_config.json",
        "9261e7d79b44c8195c1cada2b453e55b00aeb81e907a6664974b4d7776172ab3",
    ),
    (
        "special_tokens_map.json",
        "b6d346be366a7d1d48332dbc9fdf3bf8960b5d879522b7799ddba59e76237ee3",
    ),
];
pub(crate) const GEMMA_ASSETS: &[(&str, &str)] = &[
    (
        "tokenizer.json",
        "4dda02faaf32bc91031dc8c88457ac272b00c1016cc679757d1c441b248b9c47",
    ),
    (
        "onnx/model.onnx",
        "ea91fd315a7c152d427d231746f0f811a1ac93beaba656abfdf2b24e091265e4",
    ),
    (
        "onnx/model.onnx_data",
        "ef835ae565d8695236652475903078e8ed794c7c35faf1164d78ec3238e8a88d",
    ),
    (
        "config.json",
        "6e1f06404b7163e0325ed2ea3e6781cde50f4a50b31780a95ad0d30e8404d77b",
    ),
    (
        "tokenizer_config.json",
        "3ca953eea6c3c9fcda9cf3df22949ff18b216f7c74bd6459230f3f1013953f3a",
    ),
    (
        "special_tokens_map.json",
        "2f7b0adf4fb469770bb1490e3e35df87b1dc578246c5e7e6fc76ecf33213a397",
    ),
];

pub(crate) fn model_assets(
    spec: EmbeddingModelSpec,
) -> Result<&'static [(&'static str, &'static str)], EmbeddingError> {
    crate::catalog::validate_model_spec(spec)?;
    Ok(spec.required_assets)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingModelCacheFile {
    pub source_repo: &'static str,
    pub source_revision: &'static str,
    pub source_path: &'static str,
    pub local_path: PathBuf,
    pub sha256: &'static str,
}
/// The shared recipe requires only this BGE tokenizer, including Gemma-only builds.
pub fn baseline_tokenizer_cache_file(cache_root: impl AsRef<Path>) -> EmbeddingModelCacheFile {
    let config =
        EmbeddingRuntimeConfig::new(crate::EmbeddingModelId::BgeBaseEnV15, cache_root.as_ref());
    EmbeddingModelCacheFile {
        source_repo: config.model_spec().model_id,
        source_revision: config.model_spec().model_revision,
        source_path: "tokenizer.json",
        local_path: config.model_dir().join("tokenizer.json"),
        sha256: BGE_TOKENIZER_SHA256,
    }
}
pub fn prepare_baseline_tokenizer_cache(
    cache_root: impl AsRef<Path>,
) -> Result<bool, EmbeddingError> {
    let file = baseline_tokenizer_cache_file(cache_root);
    prepare_cache_file(&file, download)
}
pub fn required_embedding_model_cache_files(
    config: &EmbeddingRuntimeConfig,
) -> Vec<EmbeddingModelCacheFile> {
    let spec = config.model_spec();
    spec.required_assets
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
        verify_asset_file(&file.local_path, file.sha256)?;
    }
    Ok(())
}
pub(crate) fn read_verified_asset(
    spec: EmbeddingModelSpec,
    model_dir: &Path,
    source_path: &str,
) -> Result<Vec<u8>, EmbeddingError> {
    let expected = model_assets(spec)?
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

// Readiness and download verification stream large external weights, so these
// operations do not allocate a second model-sized buffer without loading it.
fn verify_asset_file(path: &Path, expected: &str) -> Result<(), EmbeddingError> {
    let mut input = fs::File::open(path).map_err(|e| EmbeddingError::ModelLoadFailed {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    verify_asset_reader(path, &mut input, expected)
}
fn verify_asset_reader(
    path: &Path,
    input: &mut dyn io::Read,
    expected: &str,
) -> Result<(), EmbeddingError> {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    io::copy(input, &mut digest).map_err(|e| EmbeddingError::ModelLoadFailed {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    let actual = format!("{:x}", digest.finalize());
    if actual != expected {
        return Err(EmbeddingError::AssetChecksumMismatch {
            path: path.display().to_string(),
            expected: expected.into(),
            actual,
        });
    }
    Ok(())
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
        if prepare_cache_file(&file, download)? {
            downloaded.push(file.local_path);
        }
    }
    validate_embedding_model_cache(config)?;
    Ok(downloaded)
}
fn prepare_cache_file(
    file: &EmbeddingModelCacheFile,
    fetch: impl FnOnce(&EmbeddingModelCacheFile) -> Result<(), EmbeddingError>,
) -> Result<bool, EmbeddingError> {
    if file.local_path.is_file() {
        match verify_asset_file(&file.local_path, file.sha256) {
            Ok(()) => return Ok(false),
            Err(EmbeddingError::AssetChecksumMismatch { .. }) => {}
            Err(error) => return Err(error),
        }
    }
    info!(target: "atlas_progress", phase = "embedding_model_cache", asset = file.source_path,
        "Preparing pinned asset {}/{} at revision {}; download duration unknown",file.source_repo,file.source_path,file.source_revision);
    // The existing verified temporary-file/rename path preserves any previous
    // file until its replacement has passed the exact checksum contract.
    fetch(file)?;
    verify_asset_file(&file.local_path, file.sha256)?;
    Ok(true)
}
fn download(file: &EmbeddingModelCacheFile) -> Result<(), EmbeddingError> {
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
    store_verified_download(file, &mut response.body_mut().as_reader(), &url)
}
fn store_verified_download(
    file: &EmbeddingModelCacheFile,
    input: &mut dyn io::Read,
    url: &str,
) -> Result<(), EmbeddingError> {
    use io::Seek;
    let parent = file
        .local_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| EmbeddingError::ModelCachePrepareFailed {
        path: parent.display().to_string(),
        message: e.to_string(),
    })?;
    // Each operation owns its file, including across concurrent model.onnx and
    // model.onnx_data repairs. Drop cleans only this operation's unfinished file.
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| {
        EmbeddingError::ModelCachePrepareFailed {
            path: parent.display().to_string(),
            message: e.to_string(),
        }
    })?;
    let temporary_path = temporary.path().to_path_buf();
    io::copy(input, temporary.as_file_mut()).map_err(|e| {
        EmbeddingError::ModelCacheDownloadFailed {
            url: url.into(),
            path: temporary_path.display().to_string(),
            message: e.to_string(),
        }
    })?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|e| EmbeddingError::ModelCachePrepareFailed {
            path: temporary_path.display().to_string(),
            message: e.to_string(),
        })?;
    temporary
        .as_file_mut()
        .rewind()
        .map_err(|e| EmbeddingError::ModelCachePrepareFailed {
            path: temporary_path.display().to_string(),
            message: e.to_string(),
        })?;
    // Verify through the owned descriptor, then publish the same owned file.
    verify_asset_reader(&temporary_path, temporary.as_file_mut(), file.sha256)?;
    temporary
        .persist(&file.local_path)
        .map_err(|e| EmbeddingError::ModelCachePrepareFailed {
            path: file.local_path.display().to_string(),
            message: e.to_string(),
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_and_in_memory_hashes_match_standard_sha256_vectors() {
        for (bytes, expected) in [
            (
                "",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                "abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ] {
            assert_eq!(hash_bytes(bytes.as_bytes()), expected);
            verify_asset_reader(
                Path::new("known-sha256-vector"),
                &mut io::Cursor::new(bytes.as_bytes()),
                expected,
            )
            .unwrap();
        }
    }
    #[test]
    fn revision_paths_are_isolated_and_gemma_baseline_requires_only_base_tokenizer() {
        let directory = tempfile::tempdir().unwrap();
        let base = crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15);
        let mut next = base;
        next.model_revision = "next-revision";
        assert_ne!(
            base.model_cache_path(directory.path()),
            next.model_cache_path(directory.path())
        );
        assert!(
            base.model_cache_path(directory.path())
                .ends_with(base.model_revision)
        );
        let baseline = baseline_tokenizer_cache_file(directory.path());
        assert_eq!(
            baseline.local_path,
            base.model_cache_path(directory.path())
                .join("tokenizer.json")
        );
        assert_eq!(baseline.source_revision, base.model_revision);
        assert_eq!(
            baseline.sha256,
            crate::BASELINE_SEGMENTATION_TOKENIZER_SHA256
        );
    }
    #[test]
    fn selected_checksum_repair_is_verified_atomic_and_failed_partial_download_preserves_previous_bytes()
     {
        let directory = tempfile::tempdir().unwrap();
        let bytes = b"verified replacement";
        let expected: &'static str = Box::leak(hash_bytes(bytes).into_boxed_str());
        let file = EmbeddingModelCacheFile {
            source_repo: "test/model",
            source_revision: "pinned-revision",
            source_path: "weights",
            local_path: directory.path().join("pinned-revision/weights"),
            sha256: expected,
        };
        fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
        fs::write(&file.local_path, b"previous mismatched bytes").unwrap();
        assert!(
            prepare_cache_file(&file, |f| store_verified_download(
                f,
                &mut io::Cursor::new(bytes),
                "test://verified"
            ))
            .unwrap()
        );
        assert_eq!(fs::read(&file.local_path).unwrap(), bytes);
        assert!(!prepare_cache_file(&file, |_| panic!("verified file must not download")).unwrap());
        fs::write(&file.local_path, b"preserve on repair failure").unwrap();
        let error = prepare_cache_file(&file, |f| {
            store_verified_download(
                f,
                &mut io::Cursor::new(b"partial corrupt bytes"),
                "test://partial",
            )
        })
        .unwrap_err();
        assert!(matches!(
            error,
            EmbeddingError::AssetChecksumMismatch { .. }
        ));
        assert_eq!(
            fs::read(&file.local_path).unwrap(),
            b"preserve on repair failure"
        );
        assert_eq!(
            fs::read_dir(file.local_path.parent().unwrap())
                .unwrap()
                .count(),
            1
        );
        struct Broken;
        impl io::Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("interrupted fixture response"))
            }
        }
        let mut interrupted = io::Read::chain(io::Cursor::new(b"partial bytes"), Broken);
        assert!(matches!(
            store_verified_download(&file, &mut interrupted, "test://interrupted"),
            Err(EmbeddingError::ModelCacheDownloadFailed { .. })
        ));
        assert_eq!(
            fs::read(&file.local_path).unwrap(),
            b"preserve on repair failure"
        );
        assert_eq!(
            fs::read_dir(file.local_path.parent().unwrap())
                .unwrap()
                .count(),
            1
        );
        assert!(
            prepare_cache_file(&file, |f| store_verified_download(
                f,
                &mut io::Cursor::new(bytes),
                "test://retry"
            ))
            .unwrap()
        );
        assert_eq!(fs::read(&file.local_path).unwrap(), bytes);
    }

    #[test]
    fn overlapping_repairs_isolate_same_target_and_external_weight_temporaries() {
        use std::sync::mpsc::{self, Receiver, SyncSender};
        use std::time::Duration;

        struct PausedEndReader {
            bytes: io::Cursor<Vec<u8>>,
            reached_end: SyncSender<()>,
            resume: Receiver<()>,
            fail: bool,
        }
        impl io::Read for PausedEndReader {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                let read = self.bytes.read(buffer)?;
                if read != 0 {
                    return Ok(read);
                }
                // io::copy has written all bytes before requesting this read.
                self.reached_end.send(()).unwrap();
                self.resume.recv_timeout(Duration::from_secs(10)).unwrap();
                if self.fail {
                    Err(io::Error::other("interrupted overlapping repair"))
                } else {
                    Ok(0)
                }
            }
        }

        for second_name in ["model.onnx", "model.onnx_data"] {
            let directory = tempfile::tempdir().unwrap();
            let verified = b"complete verified model";
            let file = EmbeddingModelCacheFile {
                source_repo: "test/model",
                source_revision: "pinned-revision",
                source_path: "onnx/model.onnx",
                local_path: directory.path().join("model.onnx"),
                sha256: Box::leak(hash_bytes(verified).into_boxed_str()),
            };
            let mut second = file.clone();
            second.local_path = directory.path().join(second_name);
            second.source_path = if second_name == "model.onnx" {
                "onnx/model.onnx"
            } else {
                "onnx/model.onnx_data"
            };
            fs::write(&file.local_path, b"previous model").unwrap();
            if second.local_path != file.local_path {
                fs::write(&second.local_path, b"previous external weights").unwrap();
            }

            let (first_ready, first_wait) = mpsc::sync_channel(0);
            let (finish_first, first_resume) = mpsc::sync_channel(0);
            let first_file = file.clone();
            let first = std::thread::spawn(move || {
                store_verified_download(
                    &first_file,
                    &mut PausedEndReader {
                        bytes: io::Cursor::new(verified.to_vec()),
                        reached_end: first_ready,
                        resume: first_resume,
                        fail: false,
                    },
                    "test://complete",
                )
            });
            first_wait.recv_timeout(Duration::from_secs(10)).unwrap();

            let (second_ready, second_wait) = mpsc::sync_channel(0);
            let (fail_second, second_resume) = mpsc::sync_channel(0);
            let second_file = second.clone();
            let failed = std::thread::spawn(move || {
                store_verified_download(
                    &second_file,
                    &mut PausedEndReader {
                        bytes: io::Cursor::new(b"partial unverified response".to_vec()),
                        reached_end: second_ready,
                        resume: second_resume,
                        fail: true,
                    },
                    "test://partial",
                )
            });
            second_wait.recv_timeout(Duration::from_secs(10)).unwrap();
            let before_publication = fs::read(&file.local_path).unwrap();
            // Publish A while B still owns its open, partially written file.
            finish_first.send(()).unwrap();
            let first_result = first.join().unwrap();
            let after_publication = fs::read(&file.local_path).unwrap();
            // B fails only after A has verified and attempted publication.
            fail_second.send(()).unwrap();
            let failed_result = failed.join().unwrap();

            assert_eq!(before_publication, b"previous model", "{second_name}");
            assert!(first_result.is_ok(), "{second_name}: {first_result:?}");
            assert_eq!(after_publication, verified, "{second_name}");
            assert!(matches!(
                failed_result,
                Err(EmbeddingError::ModelCacheDownloadFailed { .. })
            ));
            assert_eq!(
                fs::read(&file.local_path).unwrap(),
                verified,
                "{second_name}"
            );
            if second.local_path != file.local_path {
                assert_eq!(
                    fs::read(&second.local_path).unwrap(),
                    b"previous external weights"
                );
            }
            assert_eq!(
                fs::read_dir(directory.path()).unwrap().count(),
                if second.local_path == file.local_path {
                    1
                } else {
                    2
                },
                "failed cleanup must leave only the destination assets: {second_name}"
            );
        }
    }
}
