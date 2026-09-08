use std::ffi::OsString;
use std::fs::{File, Metadata, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::artifact::pair_manifest::{ArtifactSha256, read_manifest};
use crate::{IndexValidationError, IndexWriteError, ValidationStatus};

pub(crate) use super::pair_manifest::adjacent_manifest_path;

pub(crate) const PUBLICATION_LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
const GENERATION_DIRECTORY_SUFFIX: &str = ".atlas-generations";
static GENERATION_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PortableFileIdentity {
    platform: &'static str,
    primary: u64,
    secondary: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileState {
    identity: PortableFileIdentity,
    bytes: u64,
    modified: u128,
    changed: u128,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerationTrust {
    artifact_sha256: String,
    platform: String,
    primary: u64,
    secondary: u64,
    bytes: u64,
    modified: u128,
    changed: u128,
}

impl GenerationTrust {
    fn from_state(artifact_sha256: &ArtifactSha256, state: &FileState) -> Self {
        Self {
            artifact_sha256: artifact_sha256.as_str().to_string(),
            platform: state.identity.platform.to_string(),
            primary: state.identity.primary,
            secondary: state.identity.secondary,
            bytes: state.bytes,
            modified: state.modified,
            changed: state.changed,
        }
    }

    fn matches(&self, artifact_sha256: &ArtifactSha256, state: &FileState) -> bool {
        self.artifact_sha256 == artifact_sha256.as_str()
            && self.platform == state.identity.platform
            && self.primary == state.identity.primary
            && self.secondary == state.identity.secondary
            && self.bytes == state.bytes
            && self.modified == state.modified
            && self.changed == state.changed
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactReceiptTelemetry {
    pub write_ms: u128,
    pub compatibility_check_ms: u128,
    pub writer_digest_ms: u128,
    pub artifact_bytes: u64,
    pub compatibility_check_count: u64,
    pub writer_digest_pass_count: u64,
    pub writer_digest_bytes: u64,
    pub receipt_issue_count: u64,
    pub receipt_identity_check_count: u64,
}

/// Live, single-use authority produced by the canonical SQLite writer.
///
/// Its private retained file handle and identity state cannot be serialized or
/// reconstructed from evidence. Publication consumes it.
pub struct ArtifactPublicationReceipt {
    file: Option<File>,
    staged_path: PathBuf,
    publication_target: PathBuf,
    state: FileState,
    sha256: String,
    telemetry: ArtifactReceiptTelemetry,
}

impl std::fmt::Debug for ArtifactPublicationReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ArtifactPublicationReceipt")
            .field("staged_path", &self.staged_path)
            .field("publication_target", &self.publication_target)
            .field("sha256", &self.sha256)
            .field("telemetry", &self.telemetry)
            .finish_non_exhaustive()
    }
}

impl ArtifactPublicationReceipt {
    pub(crate) fn issue(
        staged_path: &Path,
        publication_target: &Path,
        write_ms: u128,
    ) -> Result<Self, IndexWriteError> {
        Self::issue_with_validator(
            staged_path,
            publication_target,
            write_ms,
            |file, path| {
                let connection = open_sqlite_from_retained_file(file, path)?;
                let report = crate::validate_index_metadata_connection(
                    path.display().to_string(),
                    &connection,
                )
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
                if report.status != ValidationStatus::Ok {
                    let details = report
                        .diagnostics
                        .iter()
                        .take(10)
                        .map(|diagnostic| diagnostic.message.as_str())
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Err(IndexWriteError::WriteFailed(format!(
                        "candidate artifact failed compatibility checks before publication: {details}"
                    )));
                }
                Ok(())
            },
            || Ok(()),
            || Ok(()),
        )
    }

    fn issue_with_validator(
        staged_path: &Path,
        publication_target: &Path,
        write_ms: u128,
        validator: impl FnOnce(&File, &Path) -> Result<(), IndexWriteError>,
        after_compatibility_check: impl FnOnce() -> Result<(), IndexWriteError>,
        after_digest: impl FnOnce() -> Result<(), IndexWriteError>,
    ) -> Result<Self, IndexWriteError> {
        reject_sqlite_companions(staged_path)?;
        let staged_path = canonical_existing_path(staged_path)?;
        let publication_target = canonical_future_path(publication_target)?;
        let file = open_receipt_candidate(&staged_path)?;
        let state = FileState::from_file(&file)?;
        if !file
            .metadata()
            .map_err(write_error)?
            .permissions()
            .readonly()
        {
            return Err(receipt_invalidated(
                "candidate artifact is not sealed read-only",
            ));
        }
        let mut identity_checks = 0;
        assert_path_state(&file, &staged_path, &state)?;
        identity_checks += 1;

        let compatibility_started = Instant::now();
        validator(&file, &staged_path)?;
        let compatibility_check_ms = compatibility_started.elapsed().as_millis();
        after_compatibility_check()?;
        assert_path_state(&file, &staged_path, &state)?;
        identity_checks += 1;

        let digest_started = Instant::now();
        let sha256 = sha256_file(&file).map_err(write_error)?;
        let writer_digest_ms = digest_started.elapsed().as_millis();
        after_digest()?;
        assert_path_state(&file, &staged_path, &state)?;
        identity_checks += 1;
        reject_sqlite_companions(&staged_path)?;
        assert_path_state(&file, &staged_path, &state)?;
        identity_checks += 1;

        Ok(Self {
            file: Some(file),
            staged_path,
            publication_target,
            sha256,
            telemetry: ArtifactReceiptTelemetry {
                write_ms,
                compatibility_check_ms,
                writer_digest_ms,
                artifact_bytes: state.bytes,
                compatibility_check_count: 1,
                writer_digest_pass_count: 1,
                writer_digest_bytes: state.bytes,
                receipt_issue_count: 1,
                receipt_identity_check_count: identity_checks,
            },
            state,
        })
    }

    pub fn artifact_sha256(&self) -> &str {
        &self.sha256
    }

    pub fn artifact_bytes(&self) -> u64 {
        self.state.bytes
    }

    pub fn telemetry(&self) -> &ArtifactReceiptTelemetry {
        &self.telemetry
    }

    pub(crate) fn staged_path(&self) -> &Path {
        &self.staged_path
    }

    pub(crate) fn retained_file(&self) -> Result<&File, IndexWriteError> {
        self.file
            .as_ref()
            .ok_or_else(|| receipt_invalidated("receipt no longer retains its checked file"))
    }

    pub(crate) fn assert_staged_current(&self) -> Result<(), IndexWriteError> {
        reject_sqlite_companions(&self.staged_path)?;
        assert_path_state(self.retained_file()?, &self.staged_path, &self.state)
    }

    pub(crate) fn assert_published_current(&self, target: &Path) -> Result<(), IndexWriteError> {
        let target = canonical_existing_path(target)?;
        if target != self.publication_target {
            return Err(receipt_invalidated(format!(
                "receipt target is {}, not {}",
                self.publication_target.display(),
                target.display()
            )));
        }
        reject_sqlite_companions(&target)?;
        assert_path_state(self.retained_file()?, &target, &self.state)
    }

    pub(crate) fn require_target(&self, target: &Path) -> Result<(), IndexWriteError> {
        let target = canonical_future_path(target)?;
        if target == self.publication_target {
            Ok(())
        } else {
            Err(receipt_invalidated(format!(
                "receipt is bound to publication target {}, not {}",
                self.publication_target.display(),
                target.display()
            )))
        }
    }

    #[cfg(test)]
    pub(crate) fn issue_test(
        staged_path: &Path,
        publication_target: &Path,
    ) -> Result<Self, IndexWriteError> {
        let mut permissions = std::fs::metadata(staged_path)
            .map_err(write_error)?
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(staged_path, permissions).map_err(write_error)?;
        Self::issue_with_validator(
            staged_path,
            publication_target,
            0,
            |_, _| Ok(()),
            || Ok(()),
            || Ok(()),
        )
    }

    #[cfg(test)]
    fn issue_test_with_hooks(
        staged_path: &Path,
        publication_target: &Path,
        after_compatibility_check: impl FnOnce() -> Result<(), IndexWriteError>,
        after_digest: impl FnOnce() -> Result<(), IndexWriteError>,
    ) -> Result<Self, IndexWriteError> {
        let mut permissions = std::fs::metadata(staged_path)
            .map_err(write_error)?
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(staged_path, permissions).map_err(write_error)?;
        Self::issue_with_validator(
            staged_path,
            publication_target,
            0,
            |_, _| Ok(()),
            after_compatibility_check,
            after_digest,
        )
    }
}

impl Drop for ArtifactPublicationReceipt {
    fn drop(&mut self) {
        let mut matched_path = None;
        for path in [&self.staged_path, &self.publication_target] {
            if path.exists()
                && self
                    .file
                    .as_ref()
                    .is_some_and(|file| assert_path_state(file, path, &self.state).is_ok())
            {
                matched_path = Some(path.clone());
                break;
            }
        }
        drop(self.file.take());
        if let Some(path) = matched_path
            && let Ok(metadata) = std::fs::metadata(&path)
        {
            let _ = make_owner_writable(&path, metadata);
        }
    }
}

impl FileState {
    fn from_file(file: &File) -> Result<Self, IndexWriteError> {
        let metadata = file.metadata().map_err(write_error)?;
        Ok(Self {
            identity: portable_identity(&metadata)?,
            bytes: metadata.len(),
            modified: modified_token(&metadata)?,
            changed: changed_token(&metadata)?,
        })
    }

    fn from_file_io(file: &File) -> Result<Self, std::io::Error> {
        Self::from_metadata_io(&file.metadata()?)
    }

    fn from_metadata_io(metadata: &Metadata) -> Result<Self, std::io::Error> {
        Ok(Self {
            identity: portable_identity_io(metadata)?,
            bytes: metadata.len(),
            modified: modified_token_io(metadata)?,
            changed: changed_token_io(metadata)?,
        })
    }
}

fn assert_path_state(
    file: &File,
    path: &Path,
    expected: &FileState,
) -> Result<(), IndexWriteError> {
    let handle_state = FileState::from_file(file)?;
    let path_metadata = std::fs::metadata(path).map_err(|error| {
        receipt_invalidated(format!(
            "unable to inspect bound path {}: {error}",
            path.display()
        ))
    })?;
    let path_state = FileState {
        identity: portable_identity(&path_metadata)?,
        bytes: path_metadata.len(),
        modified: modified_token(&path_metadata)?,
        changed: changed_token(&path_metadata)?,
    };
    if &handle_state != expected || &path_state != expected {
        return Err(receipt_invalidated(format!(
            "retained handle or bound path {} changed after receipt issue",
            path.display()
        )));
    }
    Ok(())
}

fn canonical_existing_path(path: &Path) -> Result<PathBuf, IndexWriteError> {
    std::fs::canonicalize(path).map_err(write_error)
}

fn canonical_future_path(path: &Path) -> Result<PathBuf, IndexWriteError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent = std::fs::canonicalize(parent).map_err(write_error)?;
    let name = path.file_name().ok_or_else(|| {
        IndexWriteError::WriteFailed("artifact path has no file name".to_string())
    })?;
    Ok(parent.join(name))
}

fn reject_sqlite_companions(path: &Path) -> Result<(), IndexWriteError> {
    for suffix in ["-wal", "-shm"] {
        let mut companion = OsString::from(path.as_os_str());
        companion.push(suffix);
        if PathBuf::from(companion).exists() {
            return Err(receipt_invalidated(format!(
                "sealed SQLite artifact has a {suffix} companion"
            )));
        }
    }
    Ok(())
}

fn open_receipt_candidate(path: &Path) -> Result<File, IndexWriteError> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        // Permit read-only SQLite duplicates and the publisher's later rename,
        // while denying handles that could mutate the retained bytes.
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_SHARE_DELETE: u32 = 0x0000_0004;
        return OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE)
            .open(path)
            .map_err(write_error);
    }
    #[cfg(unix)]
    {
        File::open(path).map_err(write_error)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Err(IndexWriteError::WriteFailed(
            "artifact publication receipts require Unix or Windows file identity support"
                .to_string(),
        ))
    }
}

fn open_sqlite_from_retained_file(
    file: &File,
    display_path: &Path,
) -> Result<rusqlite::Connection, IndexWriteError> {
    use rusqlite::OpenFlags;

    #[cfg(unix)]
    let database_url = {
        use std::os::fd::AsRawFd;
        let _ = display_path;
        format!("file:/dev/fd/{}?mode=ro&immutable=1", file.as_raw_fd())
    };
    #[cfg(windows)]
    let database_url = read_only_sqlite_uri(display_path)?;
    #[cfg(not(any(unix, windows)))]
    let database_url: String = {
        let _ = (file, display_path);
        return Err(IndexWriteError::WriteFailed(
            "artifact publication receipts require retained-handle SQLite support".to_string(),
        ));
    };

    rusqlite::Connection::open_with_flags(
        database_url,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

#[cfg(unix)]
fn make_owner_writable(path: &Path, metadata: Metadata) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o200);
    std::fs::set_permissions(path, permissions)
}

#[cfg(windows)]
fn make_owner_writable(path: &Path, metadata: Metadata) -> Result<(), std::io::Error> {
    let mut permissions = metadata.permissions();
    permissions.set_readonly(false);
    std::fs::set_permissions(path, permissions)
}

#[cfg(not(any(unix, windows)))]
fn make_owner_writable(_path: &Path, _metadata: Metadata) -> Result<(), std::io::Error> {
    Ok(())
}

#[cfg(windows)]
fn read_only_sqlite_uri(path: &Path) -> Result<String, IndexWriteError> {
    let path = path.to_str().ok_or_else(|| {
        IndexWriteError::WriteFailed(format!(
            "SQLite artifact path is not valid UTF-8: {}",
            path.display()
        ))
    })?;
    let mut escaped = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            '?' => escaped.push_str("%3f"),
            '#' => escaped.push_str("%23"),
            '%' => escaped.push_str("%25"),
            _ => escaped.push(ch),
        }
    }
    Ok(format!("file:{escaped}?mode=ro&immutable=1"))
}

#[cfg(unix)]
fn portable_identity(metadata: &Metadata) -> Result<PortableFileIdentity, IndexWriteError> {
    use std::os::unix::fs::MetadataExt;
    Ok(PortableFileIdentity {
        platform: "unix-device-inode",
        primary: metadata.dev(),
        secondary: metadata.ino(),
    })
}

fn portable_identity_io(metadata: &Metadata) -> Result<PortableFileIdentity, std::io::Error> {
    portable_identity(metadata).map_err(|error| std::io::Error::other(error.to_string()))
}

#[cfg(windows)]
fn portable_identity(metadata: &Metadata) -> Result<PortableFileIdentity, IndexWriteError> {
    use std::os::windows::fs::MetadataExt;
    Ok(PortableFileIdentity {
        platform: "windows-volume-file-index",
        primary: metadata
            .volume_serial_number()
            .ok_or_else(|| receipt_invalidated("Windows volume serial number is unavailable"))?
            as u64,
        secondary: metadata
            .file_index()
            .ok_or_else(|| receipt_invalidated("Windows file index is unavailable"))?,
    })
}

#[cfg(not(any(unix, windows)))]
fn portable_identity(_metadata: &Metadata) -> Result<PortableFileIdentity, IndexWriteError> {
    Err(IndexWriteError::WriteFailed(
        "portable file identity is unsupported on this target".to_string(),
    ))
}

#[cfg(unix)]
fn modified_token(metadata: &Metadata) -> Result<u128, IndexWriteError> {
    use std::os::unix::fs::MetadataExt;
    Ok(timestamp_token(metadata.mtime(), metadata.mtime_nsec()))
}

#[cfg(unix)]
fn changed_token(metadata: &Metadata) -> Result<u128, IndexWriteError> {
    use std::os::unix::fs::MetadataExt;
    // File-name changes update ctime on Unix even though the retained bytes are
    // unchanged. The receipt separately binds device/inode and canonical path;
    // use the content modification token here so the publisher's controlled
    // rename does not invalidate an otherwise unchanged open file.
    Ok(timestamp_token(metadata.mtime(), metadata.mtime_nsec()))
}

#[cfg(windows)]
fn modified_token(metadata: &Metadata) -> Result<u128, IndexWriteError> {
    use std::os::windows::fs::MetadataExt;
    Ok(metadata.last_write_time() as u128)
}

#[cfg(windows)]
fn changed_token(metadata: &Metadata) -> Result<u128, IndexWriteError> {
    use std::os::windows::fs::MetadataExt;
    Ok(((metadata.creation_time() as u128) << 64) | metadata.last_write_time() as u128)
}

#[cfg(not(any(unix, windows)))]
fn modified_token(_metadata: &Metadata) -> Result<u128, IndexWriteError> {
    Err(IndexWriteError::WriteFailed(
        "file modification token is unsupported on this target".to_string(),
    ))
}

#[cfg(not(any(unix, windows)))]
fn changed_token(_metadata: &Metadata) -> Result<u128, IndexWriteError> {
    Err(IndexWriteError::WriteFailed(
        "file change token is unsupported on this target".to_string(),
    ))
}

fn modified_token_io(metadata: &Metadata) -> Result<u128, std::io::Error> {
    modified_token(metadata).map_err(|error| std::io::Error::other(error.to_string()))
}

fn changed_token_io(metadata: &Metadata) -> Result<u128, std::io::Error> {
    changed_token(metadata).map_err(|error| std::io::Error::other(error.to_string()))
}

#[cfg(unix)]
fn timestamp_token(seconds: i64, nanos: i64) -> u128 {
    ((seconds as i128) << 64 | (nanos as i128 & i128::from(u64::MAX))) as u128
}

fn receipt_invalidated(message: impl Into<String>) -> IndexWriteError {
    IndexWriteError::ReceiptInvalidated(message.into())
}

pub(super) fn write_error(error: std::io::Error) -> IndexWriteError {
    IndexWriteError::WriteFailed(error.to_string())
}

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
    pub(crate) sha256: ArtifactSha256,
    pub(crate) artifact_contract_version: String,
    pub(crate) schema_version: String,
}

pub(crate) struct VerifiedGenerationFile {
    pub(crate) file: File,
    pub(crate) path: PathBuf,
    state: FileState,
    trust_path: PathBuf,
    trust_state: FileState,
}

pub(crate) struct GenerationLease {
    path: PathBuf,
    state: FileState,
    trust_path: PathBuf,
    trust_state: FileState,
    manifest_path: PathBuf,
    sha256: ArtifactSha256,
}

impl Drop for GenerationLease {
    fn drop(&mut self) {
        let generation_is_current = read_manifest(&self.manifest_path)
            .map(|manifest| manifest.build.artifact_sha256 == self.sha256)
            .unwrap_or(false);
        if !generation_is_current
            && remove_file_if_state_matches(&self.path, &self.state).unwrap_or(false)
        {
            let _ = remove_file_if_state_matches(&self.trust_path, &self.trust_state);
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
        Ok(Self {
            file,
            sha256: manifest.build.artifact_sha256,
            artifact_contract_version: manifest.artifact_contract_version,
            schema_version: manifest.schema_version,
        })
    }
}

pub(crate) fn open_verified_generation(
    target_artifact: &Path,
    manifest_path: &Path,
    verified: &VerifiedArtifactFile,
) -> Result<
    (
        VerifiedGenerationFile,
        GenerationLease,
        GenerationMaterialization,
    ),
    IndexValidationError,
> {
    let (generation, materialization) =
        ensure_generation_file(&verified.file, target_artifact, &verified.sha256)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    let lease = GenerationLease {
        path: generation.path.clone(),
        state: generation.state.clone(),
        trust_path: generation.trust_path.clone(),
        trust_state: generation.trust_state.clone(),
        manifest_path: manifest_path.to_path_buf(),
        sha256: verified.sha256.clone(),
    };
    Ok((generation, lease, materialization))
}

pub(crate) struct GenerationMaterialization {
    pub(crate) copied_bytes: u64,
    pub(crate) copy_count: u64,
    pub(crate) verify_sha_pass_count: u64,
    pub(crate) distinct_identity_check_count: u64,
}

pub(crate) fn prepare_generation_file(
    receipt: &ArtifactPublicationReceipt,
    target_artifact: &Path,
) -> Result<(PathBuf, GenerationMaterialization), IndexWriteError> {
    receipt.assert_staged_current()?;
    let sha256 = ArtifactSha256::try_from(receipt.artifact_sha256().to_string())
        .map_err(IndexWriteError::ReceiptInvalidated)?;
    let (generation, materialization) =
        ensure_generation_file(receipt.retained_file()?, target_artifact, &sha256).map_err(
            |error| {
                if error.kind() == std::io::ErrorKind::InvalidData
                    && error.to_string().contains("aliases")
                {
                    receipt_invalidated(error.to_string())
                } else {
                    IndexWriteError::WriteFailed(error.to_string())
                }
            },
        )?;
    Ok((
        generation.path,
        GenerationMaterialization {
            copied_bytes: materialization.copied_bytes,
            copy_count: materialization.copy_count,
            verify_sha_pass_count: materialization.verify_sha_pass_count,
            distinct_identity_check_count: materialization.distinct_identity_check_count,
        },
    ))
}

pub(crate) fn cleanup_generation_files(
    target_artifact: &Path,
    keep_sha256: Option<&str>,
) -> Result<(), IndexWriteError> {
    let keep_sha256 = keep_sha256
        .map(|value| ArtifactSha256::try_from(value.to_string()))
        .transpose()
        .map_err(IndexWriteError::WriteFailed)?;
    let directory = generation_directory(target_artifact);
    if !directory.exists() {
        return Ok(());
    }
    let directory = validated_generation_directory(&directory).map_err(write_error)?;
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => return Err(IndexWriteError::WriteFailed(error.to_string())),
    };
    let keep = keep_sha256
        .as_ref()
        .map(|digest| direct_generation_path(&directory, digest));
    let keep_trust = keep_sha256
        .as_ref()
        .map(|digest| generation_trust_path(&directory, digest));
    for entry in entries {
        let entry = entry.map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let path = entry.path();
        if keep.as_ref() == Some(&path)
            || keep_trust.as_ref() == Some(&path)
            || !owned_generation_cache_file(&path)
        {
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
    sha256: &ArtifactSha256,
) -> Result<(VerifiedGenerationFile, GenerationMaterialization), std::io::Error> {
    ensure_generation_file_with_reopen_hook(source, target_artifact, sha256, |_| Ok(()))
}

fn ensure_generation_file_with_reopen_hook(
    source: &File,
    target_artifact: &Path,
    sha256: &ArtifactSha256,
    mut before_reopen: impl FnMut(&Path) -> Result<(), std::io::Error>,
) -> Result<(VerifiedGenerationFile, GenerationMaterialization), std::io::Error> {
    let directory = generation_directory(target_artifact);
    std::fs::create_dir_all(&directory)?;
    let directory = validated_generation_directory(&directory)?;
    let path = direct_generation_path(&directory, sha256);
    let trust_path = generation_trust_path(&directory, sha256);
    let source_identity = portable_identity_io(&source.metadata()?)?;
    let mut materialization = GenerationMaterialization {
        copied_bytes: 0,
        copy_count: 0,
        verify_sha_pass_count: 0,
        distinct_identity_check_count: 0,
    };

    loop {
        match open_regular_file(&path) {
            Ok((file, state)) => {
                materialization.distinct_identity_check_count += 1;
                if state.identity == source_identity {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "artifact generation snapshot aliases the visible or staged artifact",
                    ));
                }
                if !file.metadata()?.permissions().readonly() {
                    remove_generation_if_state_matches(&path, &state, &trust_path)?;
                    continue;
                }
                if let Some(trust_state) =
                    read_matching_generation_trust(&trust_path, sha256, &state)?
                {
                    return Ok((
                        VerifiedGenerationFile {
                            file,
                            path,
                            state,
                            trust_path,
                            trust_state,
                        },
                        materialization,
                    ));
                }

                materialization.verify_sha_pass_count += 1;
                if sha256_file(&file)? == sha256.as_str() {
                    let trust_state = write_generation_trust(&trust_path, sha256, &state)?;
                    return Ok((
                        VerifiedGenerationFile {
                            file,
                            path,
                            state,
                            trust_path,
                            trust_state,
                        },
                        materialization,
                    ));
                }
                drop(file);
                remove_generation_if_state_matches(&path, &state, &trust_path)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let temporary = generation_temp_path(&directory, sha256);
                let result = copy_and_seal_generation(source, &temporary, sha256);
                materialization.copy_count += 1;
                materialization.verify_sha_pass_count += 1;
                let (file, state) = match result {
                    Ok(value) => value,
                    Err(error) => {
                        let _ = std::fs::remove_file(&temporary);
                        return Err(error);
                    }
                };
                materialization.copied_bytes = state.bytes;
                match std::fs::hard_link(&temporary, &path) {
                    Ok(()) => {
                        std::fs::remove_file(&temporary)?;
                        sync_directory(&directory)?;
                        let installed_state = FileState::from_file_io(&file)?;
                        before_reopen(&path)?;
                        // Reopen the installed name rather than retaining the
                        // now-unlinked temporary name. On Linux, /dev/fd for
                        // the temporary handle resolves through its deleted
                        // pathname, which SQLite cannot reopen.
                        let (installed_file, reopened_state) = open_regular_file(&path)?;
                        if reopened_state != installed_state {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "artifact generation changed while reopening its installed name",
                            ));
                        }
                        drop(file);
                        materialization.distinct_identity_check_count += 1;
                        if reopened_state.identity == source_identity {
                            drop(installed_file);
                            remove_generation_if_state_matches(
                                &path,
                                &reopened_state,
                                &trust_path,
                            )?;
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "artifact generation snapshot aliases the visible or staged artifact",
                            ));
                        }
                        if !installed_file.metadata()?.permissions().readonly() {
                            drop(installed_file);
                            remove_generation_if_state_matches(
                                &path,
                                &reopened_state,
                                &trust_path,
                            )?;
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "installed artifact generation is not sealed read-only",
                            ));
                        }
                        let trust_state =
                            write_generation_trust(&trust_path, sha256, &reopened_state)?;
                        return Ok((
                            VerifiedGenerationFile {
                                file: installed_file,
                                path,
                                state: reopened_state,
                                trust_path,
                                trust_state,
                            },
                            materialization,
                        ));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        drop(file);
                        std::fs::remove_file(&temporary)?;
                    }
                    Err(error) => {
                        drop(file);
                        let _ = std::fs::remove_file(&temporary);
                        return Err(error);
                    }
                }
            }
            Err(error) => return Err(error),
        }
    }
}

fn copy_and_seal_generation(
    source: &File,
    target_path: &Path,
    expected: &ArtifactSha256,
) -> Result<(File, FileState), std::io::Error> {
    let mut source = source.try_clone()?;
    source.seek(SeekFrom::Start(0))?;
    let mut target = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(target_path)?;
    std::io::copy(&mut source, &mut target)?;
    target.flush()?;
    target.sync_all()?;
    let actual = sha256_file(&target)?;
    if actual != expected.as_str() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "artifact generation snapshot {} has digest {actual}, expected {expected}",
                target_path.display()
            ),
        ));
    }
    seal_file_read_only(&target)?;
    let state = FileState::from_file_io(&target)?;
    Ok((target, state))
}

fn direct_generation_path(directory: &Path, sha256: &ArtifactSha256) -> PathBuf {
    directory.join(format!("{}.sqlite", sha256.as_str()))
}

#[cfg(test)]
pub(crate) fn generation_path(target_artifact: &Path, sha256: &str) -> PathBuf {
    let directory = generation_directory(target_artifact);
    let sha256 = ArtifactSha256::try_from(sha256.to_string())
        .expect("test generation digests must be canonical SHA-256 values");
    direct_generation_path(&directory, &sha256)
}

#[cfg(test)]
pub(crate) fn generation_trust_path_for_test(target_artifact: &Path, sha256: &str) -> PathBuf {
    let directory = generation_directory(target_artifact);
    let sha256 = ArtifactSha256::try_from(sha256.to_string())
        .expect("test generation digests must be canonical SHA-256 values");
    generation_trust_path(&directory, &sha256)
}

#[cfg(test)]
pub(crate) fn generation_directory_for_test(target_artifact: &Path) -> PathBuf {
    generation_directory(target_artifact)
}

fn generation_directory(target_artifact: &Path) -> PathBuf {
    let mut value = OsString::from(target_artifact.as_os_str());
    value.push(GENERATION_DIRECTORY_SUFFIX);
    PathBuf::from(value)
}

fn generation_temp_path(directory: &Path, sha256: &ArtifactSha256) -> PathBuf {
    let counter = GENERATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    directory.join(format!(
        ".{}.{}.{}.tmp",
        sha256.as_str(),
        std::process::id(),
        counter
    ))
}

fn generation_trust_path(directory: &Path, sha256: &ArtifactSha256) -> PathBuf {
    directory.join(format!("{}.sqlite.trusted", sha256.as_str()))
}

fn owned_generation_cache_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name.starts_with('.') && name.ends_with(".tmp") {
        return true;
    }
    let digest = name
        .strip_suffix(".sqlite")
        .or_else(|| name.strip_suffix(".sqlite.trusted"));
    digest.is_some_and(|digest| ArtifactSha256::try_from(digest.to_string()).is_ok())
}

fn validated_generation_directory(path: &Path) -> Result<PathBuf, std::io::Error> {
    let metadata = std::fs::symlink_metadata(path)?;
    if is_alias_metadata(&metadata) || !metadata.file_type().is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "artifact generation directory {} must be a real directory, not a link or reparse alias",
                path.display()
            ),
        ));
    }
    std::fs::canonicalize(path)
}

fn open_regular_file(path: &Path) -> Result<(File, FileState), std::io::Error> {
    let path_metadata = std::fs::symlink_metadata(path)?;
    if is_alias_metadata(&path_metadata) || !path_metadata.file_type().is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "artifact generation entry {} must be a direct regular file, not a link or reparse alias",
                path.display()
            ),
        ));
    }
    let file = File::open(path)?;
    let state = FileState::from_file_io(&file)?;
    let path_state = FileState::from_metadata_io(&path_metadata)?;
    if state != path_state {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "artifact generation entry {} changed while opening",
                path.display()
            ),
        ));
    }
    Ok((file, state))
}

fn read_matching_generation_trust(
    trust_path: &Path,
    artifact_sha256: &ArtifactSha256,
    generation_state: &FileState,
) -> Result<Option<FileState>, std::io::Error> {
    let (file, state) = match open_regular_file(trust_path) {
        Ok(value) => value,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.kind() == std::io::ErrorKind::InvalidData =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    if !file.metadata()?.permissions().readonly() || state.bytes > 4096 {
        return Ok(None);
    }
    let trust: GenerationTrust = match serde_json::from_reader(file) {
        Ok(trust) => trust,
        Err(_) => return Ok(None),
    };
    Ok(trust
        .matches(artifact_sha256, generation_state)
        .then_some(state))
}

fn write_generation_trust(
    trust_path: &Path,
    artifact_sha256: &ArtifactSha256,
    generation_state: &FileState,
) -> Result<FileState, std::io::Error> {
    let directory = trust_path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "generation trust path has no parent directory",
        )
    })?;
    let counter = GENERATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = directory.join(format!(
        ".{}.{}.{}.trust.tmp",
        artifact_sha256.as_str(),
        std::process::id(),
        counter
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        serde_json::to_writer(
            &mut file,
            &GenerationTrust::from_state(artifact_sha256, generation_state),
        )
        .map_err(std::io::Error::other)?;
        file.flush()?;
        file.sync_all()?;
        seal_file_read_only(&file)?;
        let state = FileState::from_file_io(&file)?;
        match std::fs::rename(&temporary, trust_path) {
            Ok(()) => {}
            Err(error) if cfg!(windows) && error.kind() == std::io::ErrorKind::AlreadyExists => {
                std::fs::remove_file(trust_path)?;
                std::fs::rename(&temporary, trust_path)?;
            }
            Err(error) => return Err(error),
        }
        sync_directory(directory)?;
        Ok(state)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn remove_generation_if_state_matches(
    generation_path: &Path,
    generation_state: &FileState,
    trust_path: &Path,
) -> Result<(), std::io::Error> {
    let trust_state = open_regular_file(trust_path).ok().map(|(_, state)| state);
    if remove_file_if_state_matches(generation_path, generation_state)?
        && let Some(trust_state) = trust_state
    {
        let _ = remove_file_if_state_matches(trust_path, &trust_state);
    }
    Ok(())
}

fn remove_file_if_state_matches(path: &Path, expected: &FileState) -> Result<bool, std::io::Error> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "cache path has no parent")
    })?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cache path has no file name",
            )
        })?;
    let counter = GENERATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tombstone = parent.join(format!(".{name}.{}.{}.delete", std::process::id(), counter));
    match std::fs::rename(path, &tombstone) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    }
    let matches = open_regular_file(&tombstone)
        .map(|(_, state)| &state == expected)
        .unwrap_or(false);
    if matches {
        std::fs::remove_file(&tombstone)?;
        sync_directory(parent)?;
        return Ok(true);
    }
    if !path.exists() {
        let _ = std::fs::rename(&tombstone, path);
    }
    Ok(false)
}

fn seal_file_read_only(file: &File) -> Result<(), std::io::Error> {
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    File::open(path)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

#[cfg(unix)]
fn is_alias_metadata(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn is_alias_metadata(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(any(unix, windows)))]
fn is_alias_metadata(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

pub(super) fn sha256_file(file: &File) -> Result<String, std::io::Error> {
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

#[cfg(test)]
mod receipt_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(target_os = "linux")]
    #[test]
    fn newly_materialized_generation_opens_sqlite_through_installed_handle() {
        let root = temp_root("installed-generation-handle");
        let source_path = root.join("source.sqlite");
        let target = root.join("index.sqlite");
        let connection = rusqlite::Connection::open(&source_path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE probe (value TEXT NOT NULL); INSERT INTO probe VALUES ('installed');",
            )
            .unwrap();
        drop(connection);
        let source = File::open(&source_path).unwrap();
        let digest = ArtifactSha256::try_from(sha256_file(&source).unwrap()).unwrap();

        let (generation, _) = ensure_generation_file(&source, &target, &digest).unwrap();
        let connection = open_sqlite_from_retained_file(&generation.file, &generation.path)
            .expect("SQLite must reopen the installed generation through the retained handle");
        let value: String = connection
            .query_row("SELECT value FROM probe", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "installed");

        drop((connection, generation));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn generation_replacement_during_final_reopen_fails_closed_without_trust() {
        let root = temp_root("generation-reopen-replacement");
        let source_path = root.join("source.sqlite");
        let target = root.join("index.sqlite");
        std::fs::write(&source_path, b"verified generation bytes").unwrap();
        let source = File::open(&source_path).unwrap();
        let digest = ArtifactSha256::try_from(sha256_file(&source).unwrap()).unwrap();
        let generation_path = generation_path(&target, digest.as_str());
        let trust_path = generation_trust_path_for_test(&target, digest.as_str());

        let result =
            ensure_generation_file_with_reopen_hook(&source, &target, &digest, |installed_path| {
                std::fs::remove_file(installed_path)?;
                std::fs::write(installed_path, b"replacement generation!!")?;
                let replacement = File::open(installed_path)?;
                seal_file_read_only(&replacement)
            });
        let error = match result {
            Ok(_) => panic!("a replacement generation must fail closed"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("changed while reopening"));
        assert_eq!(
            std::fs::read(generation_path).unwrap(),
            b"replacement generation!!"
        );
        assert!(!trust_path.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unchanged_retained_file_issues_one_live_receipt_and_one_digest() {
        let root = temp_root("unchanged");
        let staged = root.join("staged.sqlite");
        let target = root.join("index.sqlite");
        std::fs::write(&staged, b"receipt bytes").unwrap();

        let receipt = ArtifactPublicationReceipt::issue_test(&staged, &target).unwrap();

        assert_eq!(receipt.artifact_bytes(), 13);
        assert_eq!(receipt.telemetry().compatibility_check_count, 1);
        assert_eq!(receipt.telemetry().writer_digest_pass_count, 1);
        assert_eq!(receipt.telemetry().writer_digest_bytes, 13);
        assert_eq!(receipt.telemetry().receipt_issue_count, 1);
        assert_eq!(receipt.telemetry().receipt_identity_check_count, 4);
        receipt.assert_staged_current().unwrap();
        drop(receipt);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn path_replacement_after_compatibility_check_cannot_issue_a_receipt() {
        let root = temp_root("path-replacement");
        let staged = root.join("staged.sqlite");
        let displaced = root.join("displaced.sqlite");
        let target = root.join("index.sqlite");
        std::fs::write(&staged, b"original bytes").unwrap();
        let staged_for_hook = staged.clone();

        let error = ArtifactPublicationReceipt::issue_test_with_hooks(
            &staged,
            &target,
            move || {
                std::fs::rename(&staged_for_hook, &displaced).map_err(write_error)?;
                std::fs::write(&staged_for_hook, b"replacement!!!").map_err(write_error)
            },
            || Ok(()),
        )
        .unwrap_err();

        assert!(matches!(error, IndexWriteError::ReceiptInvalidated(_)));
        assert!(!target.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn same_size_mutation_after_compatibility_check_cannot_issue_a_receipt() {
        let root = temp_root("same-size-mutation");
        let staged = root.join("staged.sqlite");
        let target = root.join("index.sqlite");
        std::fs::write(&staged, b"abcdefgh").unwrap();
        let staged_for_hook = staged.clone();

        let error = ArtifactPublicationReceipt::issue_test_with_hooks(
            &staged,
            &target,
            move || {
                let metadata = std::fs::metadata(&staged_for_hook).map_err(write_error)?;
                make_owner_writable(&staged_for_hook, metadata).map_err(write_error)?;
                std::fs::write(&staged_for_hook, b"hgfedcba").map_err(write_error)
            },
            || Ok(()),
        )
        .unwrap_err();

        assert!(matches!(error, IndexWriteError::ReceiptInvalidated(_)));
        assert!(!target.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn path_replacement_after_digest_cannot_issue_a_receipt() {
        let root = temp_root("post-digest-replacement");
        let staged = root.join("staged.sqlite");
        let displaced = root.join("displaced.sqlite");
        let target = root.join("index.sqlite");
        std::fs::write(&staged, b"digest-bound").unwrap();
        let staged_for_hook = staged.clone();

        let error = ArtifactPublicationReceipt::issue_test_with_hooks(
            &staged,
            &target,
            || Ok(()),
            move || {
                std::fs::rename(&staged_for_hook, &displaced).map_err(write_error)?;
                std::fs::write(&staged_for_hook, b"digest-swap!").map_err(write_error)
            },
        )
        .unwrap_err();

        assert!(matches!(error, IndexWriteError::ReceiptInvalidated(_)));
        assert!(!target.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn receipt_rejects_cross_target_reuse() {
        let root = temp_root("cross-target");
        let staged = root.join("staged.sqlite");
        let target = root.join("index.sqlite");
        let wrong_target = root.join("other.sqlite");
        std::fs::write(&staged, b"receipt bytes").unwrap();
        let receipt = ArtifactPublicationReceipt::issue_test(&staged, &target).unwrap();

        let error = receipt.require_target(&wrong_target).unwrap_err();

        assert!(matches!(error, IndexWriteError::ReceiptInvalidated(_)));
        assert!(!target.exists());
        assert!(!wrong_target.exists());
        drop(receipt);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "atlas-receipt-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }
}
