use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::IndexWriteError;
use crate::artifact::pair::{PairLock, verify_pair_files};

pub fn publish_artifact_pair(
    staged_artifact: &Path,
    staged_manifest: &Path,
    target_artifact: &Path,
    target_manifest: &Path,
) -> Result<(), IndexWriteError> {
    publish_artifact_pair_with_hook(
        staged_artifact,
        staged_manifest,
        target_artifact,
        target_manifest,
        || Ok(()),
        FailureDisposition::Restore,
    )
}

#[derive(Clone, Copy)]
enum FailureDisposition {
    Restore,
    #[cfg(test)]
    AbandonForCrashSimulation,
}

fn publish_artifact_pair_with_hook(
    staged_artifact: &Path,
    staged_manifest: &Path,
    target_artifact: &Path,
    target_manifest: &Path,
    after_artifact_publish: impl FnOnce() -> Result<(), IndexWriteError>,
    failure_disposition: FailureDisposition,
) -> Result<(), IndexWriteError> {
    ensure_same_parent(target_artifact, target_manifest)?;
    verify_pair_files(staged_artifact, staged_manifest)?;
    sync_file(staged_artifact)?;
    sync_file(staged_manifest)?;

    let _pair_lock = PairLock::exclusive(target_manifest)?;
    recover_interrupted_publication(target_artifact, target_manifest)?;
    let had_previous = snapshot_previous_pair(target_artifact, target_manifest)?;

    remove_sqlite_companions(target_artifact)?;
    if let Err(error) = replace_file(staged_artifact, target_artifact) {
        return handle_publication_failure(
            error,
            staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }
    if let Err(error) = sync_parent(target_artifact) {
        return handle_publication_failure(
            error,
            staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }

    if let Err(error) = after_artifact_publish() {
        return handle_publication_failure(
            error,
            staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }

    if let Err(error) = replace_file(staged_manifest, target_manifest) {
        return handle_publication_failure(
            error,
            staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }
    if let Err(error) = sync_parent(target_manifest) {
        return handle_publication_failure(
            error,
            staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }

    if let Err(error) = verify_pair_files(target_artifact, target_manifest) {
        return handle_publication_failure(
            error,
            staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }

    cleanup_backups(target_artifact, target_manifest)?;
    sync_parent(target_artifact)?;
    Ok(())
}

fn handle_publication_failure(
    error: IndexWriteError,
    staged_artifact: &Path,
    staged_manifest: &Path,
    target_artifact: &Path,
    target_manifest: &Path,
    had_previous: bool,
    disposition: FailureDisposition,
) -> Result<(), IndexWriteError> {
    #[cfg(test)]
    if matches!(disposition, FailureDisposition::AbandonForCrashSimulation) {
        return Err(error);
    }

    let _ = disposition;
    let restore = restore_previous_pair(target_artifact, target_manifest, had_previous);
    let cleanup = cleanup_staging(staged_artifact, staged_manifest);
    match (restore, cleanup) {
        (Ok(()), Ok(())) => Err(error),
        (Err(restore), Ok(())) => Err(IndexWriteError::WriteFailed(format!(
            "{error}; also failed to restore the prior artifact pair: {restore}"
        ))),
        (Ok(()), Err(cleanup)) => Err(IndexWriteError::WriteFailed(format!(
            "{error}; also failed to clean staged publication files: {cleanup}"
        ))),
        (Err(restore), Err(cleanup)) => Err(IndexWriteError::WriteFailed(format!(
            "{error}; also failed to restore the prior artifact pair: {restore}; also failed to clean staged publication files: {cleanup}"
        ))),
    }
}

fn recover_interrupted_publication(
    target_artifact: &Path,
    target_manifest: &Path,
) -> Result<(), IndexWriteError> {
    if pair_is_valid(target_artifact, target_manifest) {
        cleanup_backups(target_artifact, target_manifest)?;
        return Ok(());
    }

    let artifact_backup = backup_path(target_artifact);
    let manifest_backup = backup_path(target_manifest);
    if pair_is_valid(&artifact_backup, &manifest_backup) {
        remove_if_exists(target_artifact)?;
        remove_if_exists(target_manifest)?;
        remove_sqlite_companions(target_artifact)?;
        replace_file(&artifact_backup, target_artifact)?;
        replace_file(&manifest_backup, target_manifest)?;
        sync_parent(target_artifact)?;
        verify_pair_files(target_artifact, target_manifest)?;
        cleanup_backups(target_artifact, target_manifest)?;
        return Ok(());
    }

    remove_if_exists(target_artifact)?;
    remove_if_exists(target_manifest)?;
    remove_sqlite_companions(target_artifact)?;
    cleanup_backups(target_artifact, target_manifest)?;
    sync_parent(target_artifact)
}

fn snapshot_previous_pair(
    target_artifact: &Path,
    target_manifest: &Path,
) -> Result<bool, IndexWriteError> {
    if !target_artifact.exists() && !target_manifest.exists() {
        return Ok(false);
    }
    verify_pair_files(target_artifact, target_manifest)?;
    cleanup_backups(target_artifact, target_manifest)?;
    snapshot_file(target_artifact, &backup_path(target_artifact))?;
    snapshot_file(target_manifest, &backup_path(target_manifest))?;
    sync_parent(target_artifact)?;
    Ok(true)
}

fn restore_previous_pair(
    target_artifact: &Path,
    target_manifest: &Path,
    had_previous: bool,
) -> Result<(), IndexWriteError> {
    remove_if_exists(target_artifact)?;
    remove_if_exists(target_manifest)?;
    remove_sqlite_companions(target_artifact)?;
    if had_previous {
        replace_file(&backup_path(target_artifact), target_artifact)?;
        replace_file(&backup_path(target_manifest), target_manifest)?;
        verify_pair_files(target_artifact, target_manifest)?;
    }
    cleanup_backups(target_artifact, target_manifest)?;
    sync_parent(target_artifact)
}

fn pair_is_valid(artifact: &Path, manifest: &Path) -> bool {
    artifact.is_file() && manifest.is_file() && verify_pair_files(artifact, manifest).is_ok()
}

fn ensure_same_parent(artifact: &Path, manifest: &Path) -> Result<(), IndexWriteError> {
    let artifact_parent = artifact.parent().unwrap_or_else(|| Path::new("."));
    let manifest_parent = manifest.parent().unwrap_or_else(|| Path::new("."));
    if artifact_parent != manifest_parent {
        return Err(IndexWriteError::WriteFailed(
            "SQLite artifact and adjacent manifest must have the same parent directory".to_string(),
        ));
    }
    Ok(())
}

fn snapshot_file(source: &Path, backup: &Path) -> Result<(), IndexWriteError> {
    remove_if_exists(backup)?;
    fs::hard_link(source, backup)
        .or_else(|_| fs::copy(source, backup).map(|_| ()))
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn replace_file(source: &Path, target: &Path) -> Result<(), IndexWriteError> {
    remove_if_exists(target)?;
    fs::rename(source, target).map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn cleanup_staging(artifact: &Path, manifest: &Path) -> Result<(), IndexWriteError> {
    remove_if_exists(artifact)?;
    remove_if_exists(manifest)
}

fn cleanup_backups(artifact: &Path, manifest: &Path) -> Result<(), IndexWriteError> {
    remove_if_exists(&backup_path(artifact))?;
    remove_if_exists(&backup_path(manifest))
}

fn remove_sqlite_companions(path: &Path) -> Result<(), IndexWriteError> {
    for companion in sqlite_companions(path) {
        remove_if_exists(&companion)?;
    }
    Ok(())
}

fn remove_if_exists(path: &Path) -> Result<(), IndexWriteError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(IndexWriteError::WriteFailed(error.to_string())),
    }
}

fn sync_file(path: &Path) -> Result<(), IndexWriteError> {
    fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn sync_parent(path: &Path) -> Result<(), IndexWriteError> {
    fs::File::open(path.parent().unwrap_or_else(|| Path::new(".")))
        .and_then(|directory| directory.sync_all())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn sqlite_companions(path: &Path) -> [PathBuf; 2] {
    ["-wal", "-shm"].map(|suffix| {
        let mut value = path.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    })
}

fn backup_path(target: &Path) -> PathBuf {
    let mut name = OsString::from(target.as_os_str());
    name.push(".pair-backup");
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::sync::{Arc, Barrier, mpsc};

    #[test]
    fn manifest_failure_restores_prior_matching_pair() {
        let fixture = PairFixture::new("manifest-failure");
        fixture.publish_initial("old");
        let (staged_artifact, staged_manifest) = fixture.stage("new");

        let error = publish_artifact_pair_with_hook(
            &staged_artifact,
            &staged_manifest,
            &fixture.artifact,
            &fixture.manifest,
            || {
                Err(IndexWriteError::WriteFailed(
                    "injected manifest failure".to_string(),
                ))
            },
            FailureDisposition::Restore,
        )
        .unwrap_err();

        assert!(error.to_string().contains("injected manifest failure"));
        fixture.assert_published("old");
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn first_publication_failure_removes_unbound_artifact() {
        let fixture = PairFixture::new("first-failure");
        let (staged_artifact, staged_manifest) = fixture.stage("new");

        publish_artifact_pair_with_hook(
            &staged_artifact,
            &staged_manifest,
            &fixture.artifact,
            &fixture.manifest,
            || {
                Err(IndexWriteError::WriteFailed(
                    "injected crash window".to_string(),
                ))
            },
            FailureDisposition::Restore,
        )
        .unwrap_err();

        assert!(!fixture.artifact.exists());
        assert!(!fixture.manifest.exists());
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn crash_window_is_fail_closed_and_next_publication_recovers() {
        let fixture = PairFixture::new("crash-recovery");
        fixture.publish_initial("old");
        let (crash_artifact, crash_manifest) = fixture.stage("crash");

        publish_artifact_pair_with_hook(
            &crash_artifact,
            &crash_manifest,
            &fixture.artifact,
            &fixture.manifest,
            || {
                Err(IndexWriteError::WriteFailed(
                    "simulated process crash".to_string(),
                ))
            },
            FailureDisposition::AbandonForCrashSimulation,
        )
        .unwrap_err();

        assert!(crate::SqliteIndexReader::open_read_only(&fixture.artifact).is_err());
        let (recovery_artifact, recovery_manifest) = fixture.stage("recovered");
        publish_artifact_pair(
            &recovery_artifact,
            &recovery_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap();
        fixture.assert_published("recovered");
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn first_publication_crash_is_recovered_without_accepting_missing_manifest() {
        let fixture = PairFixture::new("first-crash-recovery");
        let (crash_artifact, crash_manifest) = fixture.stage("crash");
        publish_artifact_pair_with_hook(
            &crash_artifact,
            &crash_manifest,
            &fixture.artifact,
            &fixture.manifest,
            || {
                Err(IndexWriteError::WriteFailed(
                    "simulated process crash".to_string(),
                ))
            },
            FailureDisposition::AbandonForCrashSimulation,
        )
        .unwrap_err();

        let error = match crate::SqliteIndexReader::open_read_only(&fixture.artifact) {
            Ok(_) => panic!("missing-manifest crash window must fail closed"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("required adjacent manifest is missing")
        );

        let (recovery_artifact, recovery_manifest) = fixture.stage("recovered");
        publish_artifact_pair(
            &recovery_artifact,
            &recovery_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap();
        fixture.assert_published("recovered");
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn concurrent_publishers_are_serialized_and_leave_one_matching_pair() {
        let fixture = Arc::new(PairFixture::new("publisher-contention"));
        fixture.publish_initial("old");
        let (artifact_a, manifest_a) = fixture.stage("publisher-a");
        let (artifact_b, manifest_b) = fixture.stage("publisher-b");
        let (entered_tx, entered_rx) = mpsc::channel();
        let release = Arc::new(Barrier::new(2));

        let fixture_a = Arc::clone(&fixture);
        let release_a = Arc::clone(&release);
        let publisher_a = std::thread::spawn(move || {
            publish_artifact_pair_with_hook(
                &artifact_a,
                &manifest_a,
                &fixture_a.artifact,
                &fixture_a.manifest,
                || {
                    entered_tx.send("a").unwrap();
                    release_a.wait();
                    Ok(())
                },
                FailureDisposition::Restore,
            )
        });
        assert_eq!(entered_rx.recv().unwrap(), "a");

        let contention_probe = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(crate::artifact::pair::lock_path(&fixture.manifest))
            .unwrap();
        let contention_error = contention_probe
            .try_lock()
            .expect_err("the first publisher must hold the target pair lock");
        assert!(matches!(
            contention_error,
            std::fs::TryLockError::WouldBlock
        ));

        let fixture_b = Arc::clone(&fixture);
        let publisher_b = std::thread::spawn(move || {
            publish_artifact_pair_with_hook(
                &artifact_b,
                &manifest_b,
                &fixture_b.artifact,
                &fixture_b.manifest,
                || Ok(()),
                FailureDisposition::Restore,
            )
        });
        release.wait();
        publisher_a.join().unwrap().unwrap();
        publisher_b.join().unwrap().unwrap();

        fixture.assert_published("publisher-b");
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn failed_publisher_cannot_restore_over_later_success() {
        let fixture = Arc::new(PairFixture::new("publisher-failure-contention"));
        fixture.publish_initial("old");
        let (artifact_a, manifest_a) = fixture.stage("failing-a");
        let (artifact_b, manifest_b) = fixture.stage("successful-b");
        let barrier = Arc::new(Barrier::new(2));

        let fixture_a = Arc::clone(&fixture);
        let barrier_a = Arc::clone(&barrier);
        let publisher_a = std::thread::spawn(move || {
            publish_artifact_pair_with_hook(
                &artifact_a,
                &manifest_a,
                &fixture_a.artifact,
                &fixture_a.manifest,
                || {
                    barrier_a.wait();
                    Err(IndexWriteError::WriteFailed(
                        "publisher a failed".to_string(),
                    ))
                },
                FailureDisposition::Restore,
            )
        });
        barrier.wait();

        let fixture_b = Arc::clone(&fixture);
        let publisher_b = std::thread::spawn(move || {
            publish_artifact_pair(
                &artifact_b,
                &manifest_b,
                &fixture_b.artifact,
                &fixture_b.manifest,
            )
        });
        assert!(publisher_a.join().unwrap().is_err());
        publisher_b.join().unwrap().unwrap();

        fixture.assert_published("successful-b");
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn concurrent_publisher_stress_never_leaves_a_split_pair() {
        let fixture = Arc::new(PairFixture::new("publisher-stress"));
        fixture.publish_initial("old");
        let start = Arc::new(Barrier::new(9));
        let mut publishers = Vec::new();
        for index in 0..8 {
            let marker = format!("stress-{index}");
            let (artifact, manifest) = fixture.stage(&marker);
            let fixture = Arc::clone(&fixture);
            let start = Arc::clone(&start);
            publishers.push(std::thread::spawn(move || {
                start.wait();
                publish_artifact_pair(&artifact, &manifest, &fixture.artifact, &fixture.manifest)
            }));
        }
        start.wait();
        for publisher in publishers {
            publisher.join().unwrap().unwrap();
        }

        verify_pair_files(&fixture.artifact, &fixture.manifest).unwrap();
        let marker = fixture.marker();
        assert!(marker.starts_with("stress-"));
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn reader_connections_remain_bound_to_their_verified_generation() {
        let fixture = PairFixture::new("reader-generation");
        fixture.publish_initial("old");
        let old_hash = sha256(&fixture.artifact);
        let old_reader = crate::SqliteIndexReader::open_read_only(&fixture.artifact).unwrap();
        let (new_artifact, new_manifest) = fixture.stage("new");
        let new_hash = sha256(&new_artifact);

        publish_artifact_pair(
            &new_artifact,
            &new_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap();
        let new_reader = crate::SqliteIndexReader::open_read_only(&fixture.artifact).unwrap();

        assert_eq!(reader_marker(&old_reader), "old");
        assert_eq!(
            old_reader.verified_artifact_sha256(),
            Some(old_hash.as_str())
        );
        assert_eq!(reader_marker(&new_reader), "new");
        assert_eq!(
            new_reader.verified_artifact_sha256(),
            Some(new_hash.as_str())
        );
        assert_ne!(old_hash, new_hash);
    }

    #[test]
    fn completed_publication_with_stale_backups_keeps_committed_pair() {
        let fixture = PairFixture::new("completed-crash-recovery");
        fixture.publish_initial("committed");
        snapshot_file(&fixture.artifact, &backup_path(&fixture.artifact)).unwrap();
        snapshot_file(&fixture.manifest, &backup_path(&fixture.manifest)).unwrap();

        let (next_artifact, next_manifest) = fixture.stage("next");
        publish_artifact_pair(
            &next_artifact,
            &next_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap();

        fixture.assert_published("next");
        fixture.assert_no_transaction_residue();
    }

    #[test]
    fn unrelated_stale_staging_files_cannot_change_the_published_pair() {
        let fixture = PairFixture::new("stale-staging");
        fixture.publish_initial("old");
        let stale_artifact = fixture.root.join(".index.sqlite.artifact-stale.stage");
        let stale_manifest = fixture.root.join(".index.sqlite.manifest-stale.stage");
        fs::write(&stale_artifact, b"untrusted stale bytes").unwrap();
        fs::write(&stale_manifest, b"untrusted stale bytes").unwrap();
        let (new_artifact, new_manifest) = fixture.stage("new");

        publish_artifact_pair(
            &new_artifact,
            &new_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap();

        fixture.assert_published("new");
        assert_eq!(fs::read(stale_artifact).unwrap(), b"untrusted stale bytes");
        assert_eq!(fs::read(stale_manifest).unwrap(), b"untrusted stale bytes");
    }

    struct PairFixture {
        root: PathBuf,
        artifact: PathBuf,
        manifest: PathBuf,
    }

    impl PairFixture {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "atlas-pair-{name}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&root).unwrap();
            Self {
                artifact: root.join("index.sqlite"),
                manifest: root.join("manifest.json"),
                root,
            }
        }

        fn publish_initial(&self, marker: &str) {
            let (artifact, manifest) = self.stage(marker);
            publish_artifact_pair(&artifact, &manifest, &self.artifact, &self.manifest).unwrap();
        }

        fn stage(&self, marker: &str) -> (PathBuf, PathBuf) {
            let artifact = self.root.join(format!("{marker}.sqlite"));
            let manifest = self.root.join(format!("{marker}.json"));
            sqlite_with_marker(&artifact, marker);
            write_manifest(&manifest, &artifact);
            (artifact, manifest)
        }

        fn marker(&self) -> String {
            crate::SqliteIndexReader::open_read_only(&self.artifact)
                .unwrap()
                .validation_connection()
                .unwrap()
                .query_row("SELECT value FROM marker", [], |row| row.get(0))
                .unwrap()
        }

        fn assert_published(&self, marker: &str) {
            verify_pair_files(&self.artifact, &self.manifest).unwrap();
            assert_eq!(self.marker(), marker);
        }

        fn assert_no_transaction_residue(&self) {
            assert!(!backup_path(&self.artifact).exists());
            assert!(!backup_path(&self.manifest).exists());
        }
    }

    impl Drop for PairFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn sqlite_with_marker(path: &Path, value: &str) {
        let connection = rusqlite::Connection::open(path).unwrap();
        connection
            .execute_batch("CREATE TABLE marker(value TEXT NOT NULL);")
            .unwrap();
        connection
            .execute("INSERT INTO marker(value) VALUES (?1)", [value])
            .unwrap();
    }

    fn write_manifest(path: &Path, artifact: &Path) {
        let hash = sha256(artifact);
        fs::write(
            path,
            format!(
                r#"{{"manifest_version":"pf2e-atlas-artifact-manifest/v2","build":{{"artifact_sha256":"{hash}"}}}}"#
            ),
        )
        .unwrap();
    }

    fn sha256(path: &Path) -> String {
        format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
    }

    fn reader_marker(reader: &crate::SqliteIndexReader) -> String {
        reader
            .validation_connection()
            .unwrap()
            .query_row("SELECT value FROM marker", [], |row| row.get(0))
            .unwrap()
    }
}
