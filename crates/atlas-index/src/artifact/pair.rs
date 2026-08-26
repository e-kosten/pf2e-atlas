use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{IndexValidationError, IndexWriteError};

pub(crate) const ADJACENT_MANIFEST_FILE_NAME: &str = "manifest.json";
const ARTIFACT_MANIFEST_VERSION: &str = "pf2e-atlas-artifact-manifest/v2";
pub(crate) const PUBLICATION_LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
const GENERATION_DIRECTORY_SUFFIX: &str = ".atlas-generations";
static GENERATION_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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
        Self::exclusive_with_timeout(manifest_path, PUBLICATION_LOCK_TIMEOUT)
    }

    pub(crate) fn exclusive_with_timeout(
        manifest_path: &Path,
        timeout: Duration,
    ) -> Result<Self, IndexWriteError> {
        let file = open_lock_file(manifest_path)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let started = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { _file: file }),
                Err(std::fs::TryLockError::WouldBlock) if started.elapsed() < timeout => {
                    std::thread::sleep(
                        LOCK_RETRY_INTERVAL.min(timeout.saturating_sub(started.elapsed())),
                    );
                }
                Err(std::fs::TryLockError::WouldBlock) => {
                    return Err(IndexWriteError::WriteFailed(format!(
                        "timed out after {:.3}s waiting for the artifact publication lock at {}; another Atlas reader or publisher is using this target; retry after the concurrent operation completes",
                        timeout.as_secs_f64(),
                        lock_path(manifest_path).display()
                    )));
                }
                Err(std::fs::TryLockError::Error(error)) => {
                    return Err(IndexWriteError::WriteFailed(error.to_string()));
                }
            }
        }
    }
}

pub(crate) struct VerifiedArtifactFile {
    pub(crate) file: File,
    pub(crate) sha256: String,
}

pub(crate) struct VerifiedGenerationFile {
    pub(crate) file: File,
    pub(crate) path: PathBuf,
}

pub(crate) struct GenerationLease {
    path: PathBuf,
    manifest_path: PathBuf,
    sha256: String,
}

impl Drop for GenerationLease {
    fn drop(&mut self) {
        let generation_is_current = read_manifest(&self.manifest_path)
            .map(|manifest| manifest.build.artifact_sha256 == self.sha256)
            .unwrap_or(false);
        if !generation_is_current {
            let _ = std::fs::remove_file(&self.path);
        }
    }
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

pub(crate) fn open_verified_generation(
    target_artifact: &Path,
    manifest_path: &Path,
    verified: &VerifiedArtifactFile,
) -> Result<(VerifiedGenerationFile, GenerationLease), IndexValidationError> {
    let path = ensure_generation_file(&verified.file, target_artifact, &verified.sha256)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    let file = open_file_with_sha256(&path, &verified.sha256)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    Ok((
        VerifiedGenerationFile {
            file,
            path: path.clone(),
        },
        GenerationLease {
            path,
            manifest_path: manifest_path.to_path_buf(),
            sha256: verified.sha256.clone(),
        },
    ))
}

pub(crate) fn prepare_generation_file(
    staged_artifact: &Path,
    target_artifact: &Path,
    sha256: &str,
) -> Result<PathBuf, IndexWriteError> {
    let file = File::open(staged_artifact)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    ensure_generation_file(&file, target_artifact, sha256)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

pub(crate) fn cleanup_generation_files(
    target_artifact: &Path,
    keep_sha256: &str,
) -> Result<(), IndexWriteError> {
    let directory = generation_directory(target_artifact);
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(IndexWriteError::WriteFailed(error.to_string())),
    };
    let keep = generation_path(target_artifact, keep_sha256);
    for entry in entries {
        let entry = entry.map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let path = entry.path();
        if path == keep || !owned_generation_file(&path) {
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    || error.kind() == std::io::ErrorKind::PermissionDenied => {}
            Err(error) => return Err(IndexWriteError::WriteFailed(error.to_string())),
        }
    }
    Ok(())
}

fn ensure_generation_file(
    source: &File,
    target_artifact: &Path,
    sha256: &str,
) -> Result<PathBuf, std::io::Error> {
    let path = generation_path(target_artifact, sha256);
    if path.exists() {
        match open_file_with_sha256(&path, sha256) {
            Ok(_) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
                match std::fs::remove_file(&path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    let directory = generation_directory(target_artifact);
    std::fs::create_dir_all(&directory)?;
    let temporary = generation_temp_path(target_artifact, sha256);
    let result = copy_open_file(source, &temporary).and_then(|()| {
        open_file_with_sha256(&temporary, sha256)?;
        match std::fs::rename(&temporary, &path) {
            Ok(()) => Ok(()),
            Err(_) if path.exists() => {
                open_file_with_sha256(&path, sha256)?;
                std::fs::remove_file(&temporary)
            }
            Err(error) => Err(error),
        }
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result?;
    Ok(path)
}

fn copy_open_file(source: &File, target: &Path) -> Result<(), std::io::Error> {
    let mut source = source.try_clone()?;
    source.seek(SeekFrom::Start(0))?;
    let mut target = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    std::io::copy(&mut source, &mut target)?;
    target.flush()?;
    target.sync_all()
}

fn open_file_with_sha256(path: &Path, expected: &str) -> Result<File, std::io::Error> {
    let file = File::open(path)?;
    let actual = sha256_file(&file)?;
    if actual != expected {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "artifact generation snapshot {} has digest {actual}, expected {expected}",
                path.display()
            ),
        ));
    }
    Ok(file)
}

pub(crate) fn generation_path(target_artifact: &Path, sha256: &str) -> PathBuf {
    generation_directory(target_artifact).join(format!("{sha256}.sqlite"))
}

fn generation_directory(target_artifact: &Path) -> PathBuf {
    let mut value = OsString::from(target_artifact.as_os_str());
    value.push(GENERATION_DIRECTORY_SUFFIX);
    PathBuf::from(value)
}

fn generation_temp_path(target_artifact: &Path, sha256: &str) -> PathBuf {
    let counter = GENERATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    generation_directory(target_artifact).join(format!(
        ".{sha256}.{}.{}.tmp",
        std::process::id(),
        counter
    ))
}

fn owned_generation_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name.starts_with('.') && name.ends_with(".tmp") {
        return true;
    }
    name.strip_suffix(".sqlite").is_some_and(|digest| {
        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
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
