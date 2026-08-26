use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{IndexValidationError, IndexWriteError};

pub(crate) const ADJACENT_MANIFEST_FILE_NAME: &str = "manifest.json";
const ARTIFACT_MANIFEST_VERSION: &str = "pf2e-atlas-artifact-manifest/v2";

pub(crate) struct PairLock {
    _file: File,
}

impl PairLock {
    pub(crate) fn shared(manifest_path: &Path) -> Result<Self, IndexValidationError> {
        let file = open_lock_file(manifest_path)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        file.lock_shared()
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self { _file: file })
    }

    pub(crate) fn exclusive(manifest_path: &Path) -> Result<Self, IndexWriteError> {
        let file = open_lock_file(manifest_path)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        file.lock()
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        Ok(Self { _file: file })
    }
}

pub(crate) struct VerifiedArtifactFile {
    pub(crate) file: File,
    pub(crate) sha256: String,
}

impl VerifiedArtifactFile {
    pub(crate) fn open(
        artifact_path: &Path,
        manifest_path: &Path,
    ) -> Result<Self, IndexValidationError> {
        let file = File::open(artifact_path).map_err(|error| {
            IndexValidationError::Unavailable(format!(
                "unable to open database file {}: {error}",
                artifact_path.display()
            ))
        })?;
        let manifest = read_manifest(manifest_path)?;
        let sha256 = sha256_file(&file)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        if sha256 != manifest.build.artifact_sha256 {
            return Err(pair_mismatch());
        }
        Ok(Self { file, sha256 })
    }
}

#[derive(Deserialize)]
struct PairManifest {
    manifest_version: String,
    build: PairManifestBuild,
}

#[derive(Deserialize)]
struct PairManifestBuild {
    artifact_sha256: String,
}

pub(crate) fn adjacent_manifest_path(artifact_path: &Path) -> PathBuf {
    artifact_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(ADJACENT_MANIFEST_FILE_NAME)
}

pub(crate) fn verify_pair_files(
    artifact_path: &Path,
    manifest_path: &Path,
) -> Result<String, IndexWriteError> {
    VerifiedArtifactFile::open(artifact_path, manifest_path)
        .map(|verified| verified.sha256)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn read_manifest(path: &Path) -> Result<PairManifest, IndexValidationError> {
    let bytes = std::fs::read(path).map_err(|error| {
        let message = if error.kind() == std::io::ErrorKind::NotFound {
            format!(
                "required adjacent manifest is missing at {}; rebuild the artifact and manifest together with `atlas index build`",
                path.display()
            )
        } else {
            format!("unable to read adjacent manifest {}: {error}", path.display())
        };
        IndexValidationError::Unavailable(message)
    })?;
    let manifest: PairManifest = serde_json::from_slice(&bytes).map_err(|error| {
        IndexValidationError::Unavailable(format!(
            "adjacent manifest is not a v2 pair manifest: {error}; rebuild the artifact and manifest together"
        ))
    })?;
    if manifest.manifest_version != ARTIFACT_MANIFEST_VERSION {
        return Err(IndexValidationError::Unavailable(format!(
            "adjacent manifest contract `{}` is unsupported; rebuild the artifact and manifest together",
            manifest.manifest_version
        )));
    }
    Ok(manifest)
}

fn sha256_file(file: &File) -> Result<String, std::io::Error> {
    let mut file = file.try_clone()?;
    file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn pair_mismatch() -> IndexValidationError {
    IndexValidationError::Unavailable(
        "SQLite artifact and adjacent manifest are not a matching published pair; rebuild them together with `atlas index build`".to_string(),
    )
}

fn open_lock_file(manifest_path: &Path) -> Result<File, std::io::Error> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path(manifest_path))
}

pub(crate) fn lock_path(manifest_path: &Path) -> PathBuf {
    let mut value = OsString::from(manifest_path.as_os_str());
    value.push(".pair.lock");
    PathBuf::from(value)
}
