use std::ffi::OsString;
use std::fs::{File, Metadata, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_MANIFEST_VERSION, ARTIFACT_SCHEMA_VERSION,
};
use crate::{IndexValidationError, IndexWriteError, ValidationStatus};

pub(crate) const ADJACENT_MANIFEST_FILE_NAME: &str = "manifest.json";
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

#[cfg(unix)]
fn timestamp_token(seconds: i64, nanos: i64) -> u128 {
    ((seconds as i128) << 64 | (nanos as i128 & i128::from(u64::MAX))) as u128
}

fn receipt_invalidated(message: impl Into<String>) -> IndexWriteError {
    IndexWriteError::ReceiptInvalidated(message.into())
}

fn write_error(error: std::io::Error) -> IndexWriteError {
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
    pub(crate) sha256: String,
    pub(crate) artifact_contract_version: String,
    pub(crate) schema_version: String,
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
    Ok((
        generation,
        GenerationLease {
            path: generation_path(target_artifact, &verified.sha256),
            manifest_path: manifest_path.to_path_buf(),
            sha256: verified.sha256.clone(),
        },
        materialization,
    ))
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
    let sha256 = receipt.artifact_sha256();
    let (generation, materialization) =
        ensure_generation_file(receipt.retained_file()?, target_artifact, sha256)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let source_identity =
        portable_identity(&receipt.retained_file()?.metadata().map_err(write_error)?)?;
    let generation_identity = portable_identity(&generation.file.metadata().map_err(write_error)?)?;
    if source_identity == generation_identity {
        let _ = std::fs::remove_file(&generation.path);
        return Err(receipt_invalidated(
            "generation snapshot aliases the receipt-bound artifact instead of isolating bytes",
        ));
    }
    Ok((
        generation.path,
        GenerationMaterialization {
            copied_bytes: materialization.copied_bytes,
            copy_count: materialization.copy_count,
            verify_sha_pass_count: materialization.verify_sha_pass_count,
            distinct_identity_check_count: 1,
        },
    ))
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
) -> Result<(VerifiedGenerationFile, GenerationMaterialization), std::io::Error> {
    let path = generation_path(target_artifact, sha256);
    if path.exists() {
        match File::open(&path) {
            Ok(file) => {
                return Ok((
                    VerifiedGenerationFile { file, path },
                    GenerationMaterialization {
                        copied_bytes: 0,
                        copy_count: 0,
                        verify_sha_pass_count: 0,
                        distinct_identity_check_count: 0,
                    },
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    let directory = generation_directory(target_artifact);
    std::fs::create_dir_all(&directory)?;
    let temporary = generation_temp_path(target_artifact, sha256);
    let result = copy_open_file(source, &temporary).and_then(|()| {
        let verified = open_file_with_sha256(&temporary, sha256)?;
        match std::fs::rename(&temporary, &path) {
            Ok(()) => Ok(verified),
            Err(_) if path.exists() => {
                drop(verified);
                std::fs::remove_file(&temporary)?;
                File::open(&path)
            }
            Err(error) => Err(error),
        }
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    let file = result?;
    let copied_bytes = file.metadata()?.len();
    Ok((
        VerifiedGenerationFile { file, path },
        GenerationMaterialization {
            copied_bytes,
            copy_count: 1,
            verify_sha_pass_count: 1,
            distinct_identity_check_count: 0,
        },
    ))
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
    artifact_contract_version: String,
    schema_version: String,
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
    let manifest = read_manifest(manifest_path)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let file = File::open(artifact_path).map_err(write_error)?;
    let sha256 = sha256_file(&file).map_err(write_error)?;
    if sha256 != manifest.build.artifact_sha256 {
        return Err(IndexWriteError::WriteFailed(pair_mismatch().to_string()));
    }
    Ok(sha256)
}

pub(crate) fn verify_manifest_digest(
    manifest_path: &Path,
    expected_sha256: &str,
) -> Result<(), IndexWriteError> {
    let manifest = read_manifest(manifest_path)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    if manifest.build.artifact_sha256 != expected_sha256 {
        return Err(IndexWriteError::ReceiptInvalidated(
            "adjacent manifest digest does not match the artifact publication receipt".to_string(),
        ));
    }
    Ok(())
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
            "adjacent manifest is not a supported pair manifest: {error}; rebuild the artifact and manifest together"
        ))
    })?;
    if manifest.manifest_version != ARTIFACT_MANIFEST_VERSION {
        return Err(IndexValidationError::Unavailable(format!(
            "adjacent manifest contract `{}` is unsupported; rebuild the artifact and manifest together",
            manifest.manifest_version
        )));
    }
    if manifest.artifact_contract_version != ARTIFACT_CONTRACT_VERSION {
        return Err(IndexValidationError::Unavailable(format!(
            "adjacent manifest artifact contract `{}` is unsupported; rebuild the artifact and manifest together",
            manifest.artifact_contract_version
        )));
    }
    if manifest.schema_version != ARTIFACT_SCHEMA_VERSION {
        return Err(IndexValidationError::Unavailable(format!(
            "adjacent manifest schema `{}` is unsupported; rebuild the artifact and manifest together",
            manifest.schema_version
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

#[cfg(test)]
mod receipt_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
