use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::IndexWriteError;
use crate::artifact::pair::{
    ArtifactPublicationReceipt, PairLock, cleanup_generation_files, prepare_generation_file,
    verify_manifest_digest, verify_pair_files,
};

#[derive(Debug, Clone, Default)]
pub struct ArtifactPublicationTelemetry {
    pub lock_wait_ms: u128,
    pub recovery_ms: u128,
    pub generation_materialization_ms: u128,
    pub prior_pair_snapshot_ms: u128,
    pub pair_install_ms: u128,
    pub visible_pair_verification_ms: u128,
    pub cleanup_ms: u128,
    pub publication_sha_pass_count: u64,
    pub publication_sha_bytes: u64,
    pub generation_copy_count: u64,
    pub generation_copy_bytes: u64,
    pub generation_copy_verify_sha_pass_count: u64,
    pub generation_distinct_identity_check_count: u64,
    pub generation_alias_rejection_count: u64,
    pub receipt_reuse_count: u64,
    pub receipt_invalidation_count: u64,
    pub recovery_sha_pass_count: u64,
    pub unclassified_sha_pass_count: u64,
    pub unclassified_copy_count: u64,
}

pub fn publish_artifact_pair(
    receipt: ArtifactPublicationReceipt,
    staged_manifest: &Path,
    target_artifact: &Path,
    target_manifest: &Path,
) -> Result<ArtifactPublicationTelemetry, IndexWriteError> {
    publish_artifact_pair_with_hook(
        receipt,
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
    receipt: ArtifactPublicationReceipt,
    staged_manifest: &Path,
    target_artifact: &Path,
    target_manifest: &Path,
    after_artifact_publish: impl FnOnce() -> Result<(), IndexWriteError>,
    failure_disposition: FailureDisposition,
) -> Result<ArtifactPublicationTelemetry, IndexWriteError> {
    let staged_artifact = receipt.staged_path().to_path_buf();
    ensure_same_parent(target_artifact, target_manifest)?;
    receipt.require_target(target_artifact)?;
    receipt.assert_staged_current()?;
    let staged_sha256 = receipt.artifact_sha256().to_string();
    verify_manifest_digest(staged_manifest, &staged_sha256)?;
    receipt
        .retained_file()?
        .sync_all()
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    sync_file(staged_manifest)?;

    let mut telemetry = ArtifactPublicationTelemetry::default();
    let phase = Instant::now();
    let _pair_lock = PairLock::exclusive(target_manifest)?;
    telemetry.lock_wait_ms = phase.elapsed().as_millis();
    let phase = Instant::now();
    let recovery = recover_interrupted_publication(target_artifact, target_manifest)?;
    telemetry.recovery_ms = phase.elapsed().as_millis();
    telemetry.recovery_sha_pass_count = recovery.sha_pass_count;
    let phase = Instant::now();
    let (_, generation) = prepare_generation_file(&receipt, target_artifact)?;
    telemetry.generation_materialization_ms = phase.elapsed().as_millis();
    telemetry.generation_copy_count = generation.copy_count;
    telemetry.generation_copy_bytes = generation.copied_bytes;
    telemetry.generation_copy_verify_sha_pass_count = generation.verify_sha_pass_count;
    telemetry.generation_distinct_identity_check_count = generation.distinct_identity_check_count;
    let phase = Instant::now();
    let had_previous = snapshot_previous_pair(
        target_artifact,
        target_manifest,
        recovery.prior_pair_available,
    )?;
    telemetry.prior_pair_snapshot_ms = phase.elapsed().as_millis();

    let phase = Instant::now();
    remove_sqlite_companions(target_artifact)?;
    if let Err(error) = replace_file(&staged_artifact, target_artifact) {
        return handle_publication_failure(
            error,
            &staged_artifact,
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
            &staged_artifact,
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
            &staged_artifact,
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
            &staged_artifact,
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
            &staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }

    telemetry.pair_install_ms = phase.elapsed().as_millis();

    let phase = Instant::now();
    if let Err(error) = receipt
        .assert_published_current(target_artifact)
        .and_then(|()| verify_manifest_digest(target_manifest, &staged_sha256))
    {
        return handle_publication_failure(
            error,
            &staged_artifact,
            staged_manifest,
            target_artifact,
            target_manifest,
            had_previous,
            failure_disposition,
        );
    }
    telemetry.visible_pair_verification_ms = phase.elapsed().as_millis();
    telemetry.receipt_reuse_count = 1;

    let phase = Instant::now();
    cleanup_backups(target_artifact, target_manifest)?;
    cleanup_generation_files(target_artifact, Some(&staged_sha256))?;
    sync_parent(target_artifact)?;
    telemetry.cleanup_ms = phase.elapsed().as_millis();
    Ok(telemetry)
}

fn handle_publication_failure<T>(
    error: IndexWriteError,
    staged_artifact: &Path,
    staged_manifest: &Path,
    target_artifact: &Path,
    target_manifest: &Path,
    had_previous: bool,
    disposition: FailureDisposition,
) -> Result<T, IndexWriteError> {
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

#[derive(Debug, Clone, Copy)]
struct RecoveryResult {
    prior_pair_available: bool,
    sha_pass_count: u64,
}

fn recover_interrupted_publication(
    target_artifact: &Path,
    target_manifest: &Path,
) -> Result<RecoveryResult, IndexWriteError> {
    let mut sha_pass_count = 0;
    if pair_is_valid(target_artifact, target_manifest, &mut sha_pass_count) {
        cleanup_backups(target_artifact, target_manifest)?;
        return Ok(RecoveryResult {
            prior_pair_available: true,
            sha_pass_count,
        });
    }

    let artifact_backup = backup_path(target_artifact);
    let manifest_backup = backup_path(target_manifest);
    if pair_is_valid(&artifact_backup, &manifest_backup, &mut sha_pass_count) {
        remove_if_exists(target_artifact)?;
        remove_if_exists(target_manifest)?;
        remove_sqlite_companions(target_artifact)?;
        replace_file(&artifact_backup, target_artifact)?;
        replace_file(&manifest_backup, target_manifest)?;
        sync_parent(target_artifact)?;
        cleanup_backups(target_artifact, target_manifest)?;
        return Ok(RecoveryResult {
            prior_pair_available: true,
            sha_pass_count,
        });
    }

    remove_if_exists(target_artifact)?;
    remove_if_exists(target_manifest)?;
    remove_sqlite_companions(target_artifact)?;
    cleanup_backups(target_artifact, target_manifest)?;
    sync_parent(target_artifact)?;
    Ok(RecoveryResult {
        prior_pair_available: false,
        sha_pass_count,
    })
}

fn snapshot_previous_pair(
    target_artifact: &Path,
    target_manifest: &Path,
    prior_pair_available: bool,
) -> Result<bool, IndexWriteError> {
    if !prior_pair_available {
        return Ok(false);
    }
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
        let sha256 = verify_pair_files(target_artifact, target_manifest)?;
        cleanup_generation_files(target_artifact, Some(&sha256))?;
    } else {
        cleanup_generation_files(target_artifact, None)?;
    }
    cleanup_backups(target_artifact, target_manifest)?;
    sync_parent(target_artifact)
}

fn pair_is_valid(artifact: &Path, manifest: &Path, sha_pass_count: &mut u64) -> bool {
    if !artifact.is_file() || !manifest.is_file() {
        return false;
    }
    *sha_pass_count += 1;
    verify_pair_files(artifact, manifest).is_ok()
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
    use std::time::{Duration, Instant};

    fn publish_artifact_pair(
        staged_artifact: &Path,
        staged_manifest: &Path,
        target_artifact: &Path,
        target_manifest: &Path,
    ) -> Result<(), IndexWriteError> {
        let receipt = ArtifactPublicationReceipt::issue_test(staged_artifact, target_artifact)?;
        super::publish_artifact_pair(receipt, staged_manifest, target_artifact, target_manifest)
            .map(|_| ())
    }

    fn publish_artifact_pair_with_hook(
        staged_artifact: &Path,
        staged_manifest: &Path,
        target_artifact: &Path,
        target_manifest: &Path,
        after_artifact_publish: impl FnOnce() -> Result<(), IndexWriteError>,
        failure_disposition: FailureDisposition,
    ) -> Result<(), IndexWriteError> {
        let receipt = ArtifactPublicationReceipt::issue_test(staged_artifact, target_artifact)?;
        super::publish_artifact_pair_with_hook(
            receipt,
            staged_manifest,
            target_artifact,
            target_manifest,
            after_artifact_publish,
            failure_disposition,
        )
        .map(|_| ())
    }

    #[test]
    fn live_receipt_reuses_writer_digest_and_materializes_one_distinct_copy() {
        let fixture = PairFixture::new("receipt-telemetry");
        let (staged_artifact, staged_manifest) = fixture.stage("new");
        let bytes = fs::metadata(&staged_artifact).unwrap().len();
        let receipt =
            ArtifactPublicationReceipt::issue_test(&staged_artifact, &fixture.artifact).unwrap();

        let telemetry = super::publish_artifact_pair(
            receipt,
            &staged_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap();

        assert_eq!(telemetry.publication_sha_pass_count, 0);
        assert_eq!(telemetry.publication_sha_bytes, 0);
        assert_eq!(telemetry.generation_copy_count, 1);
        assert_eq!(telemetry.generation_copy_bytes, bytes);
        assert_eq!(telemetry.generation_copy_verify_sha_pass_count, 1);
        assert_eq!(telemetry.generation_distinct_identity_check_count, 1);
        assert_eq!(telemetry.receipt_reuse_count, 1);
        assert_eq!(telemetry.unclassified_sha_pass_count, 0);
        assert_eq!(telemetry.unclassified_copy_count, 0);
        fixture.assert_published("new");
    }

    #[test]
    fn manifest_digest_mismatch_invalidates_receipt_before_visible_mutation() {
        let fixture = PairFixture::new("receipt-manifest-mismatch");
        let (staged_artifact, staged_manifest) = fixture.stage("new");
        let receipt =
            ArtifactPublicationReceipt::issue_test(&staged_artifact, &fixture.artifact).unwrap();
        fs::write(
            &staged_manifest,
            format!(
                r#"{{"manifest_version":"{}","artifact_contract_version":"{}","schema_version":"{}","build":{{"artifact_sha256":"{}"}}}}"#,
                crate::ARTIFACT_MANIFEST_VERSION,
                crate::ARTIFACT_CONTRACT_VERSION,
                crate::ARTIFACT_SCHEMA_VERSION,
                "0".repeat(64),
            ),
        )
        .unwrap();

        let error = super::publish_artifact_pair(
            receipt,
            &staged_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap_err();

        assert!(matches!(error, IndexWriteError::ReceiptInvalidated(_)));
        assert!(!fixture.artifact.exists());
        assert!(!fixture.manifest.exists());
    }

    #[test]
    fn hard_link_generation_alias_is_rejected_before_visible_install() {
        let fixture = PairFixture::new("receipt-generation-alias");
        let (staged_artifact, staged_manifest) = fixture.stage("new");
        let receipt =
            ArtifactPublicationReceipt::issue_test(&staged_artifact, &fixture.artifact).unwrap();
        let generation =
            crate::artifact::pair::generation_path(&fixture.artifact, receipt.artifact_sha256());
        fs::create_dir_all(generation.parent().unwrap()).unwrap();
        fs::hard_link(&staged_artifact, &generation).unwrap();

        let error = super::publish_artifact_pair(
            receipt,
            &staged_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap_err();

        assert!(matches!(error, IndexWriteError::ReceiptInvalidated(_)));
        assert!(!fixture.artifact.exists());
        assert!(!fixture.manifest.exists());
        assert!(generation.exists());
    }

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
    fn crash_window_serves_old_generation_and_next_publication_recovers() {
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

        let crash_reader = crate::SqliteIndexReader::open_read_only(&fixture.artifact).unwrap();
        assert_eq!(reader_marker(&crash_reader), "old");
        drop(crash_reader);
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
    fn generation_binding_publishes_with_multiple_long_lived_readers() {
        let fixture = PairFixture::new("reader-generation");
        fixture.publish_initial("old");
        let old_hash = sha256(&fixture.artifact);
        let old_generation = crate::artifact::pair::generation_path(&fixture.artifact, &old_hash);
        let old_reader_a = crate::SqliteIndexReader::open_read_only(&fixture.artifact).unwrap();
        let old_reader_b = crate::SqliteIndexReader::open_read_only(&fixture.artifact).unwrap();
        let (new_artifact, new_manifest) = fixture.stage("new");
        let new_hash = sha256(&new_artifact);
        let new_generation = crate::artifact::pair::generation_path(&fixture.artifact, &new_hash);
        let target_artifact = fixture.artifact.clone();
        let target_manifest = fixture.manifest.clone();
        let (published_tx, published_rx) = mpsc::channel();

        let publisher = std::thread::spawn(move || {
            published_tx
                .send(publish_artifact_pair(
                    &new_artifact,
                    &new_manifest,
                    &target_artifact,
                    &target_manifest,
                ))
                .unwrap();
        });

        published_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("publisher must finish while long-lived readers remain active")
            .unwrap();
        publisher.join().unwrap();

        assert_eq!(reader_marker(&old_reader_a), "old");
        assert_eq!(reader_marker(&old_reader_b), "old");
        assert_eq!(
            old_reader_a.verified_artifact_sha256(),
            Some(old_hash.as_str())
        );

        let new_reader = crate::SqliteIndexReader::open_read_only(&fixture.artifact).unwrap();
        assert_eq!(reader_marker(&new_reader), "new");
        assert_eq!(
            new_reader.verified_artifact_sha256(),
            Some(new_hash.as_str())
        );
        assert_ne!(old_hash, new_hash);
        assert!(new_generation.exists());
        drop(old_reader_a);
        drop(old_reader_b);
        assert!(!old_generation.exists());
    }

    #[test]
    fn generation_binding_writer_lock_has_an_actionable_deadline() {
        let fixture = PairFixture::new("writer-lock-deadline");
        fixture.publish_initial("old");
        let _reader_acquisition = PairLock::shared(&fixture.manifest).unwrap();
        let timeout = Duration::from_millis(75);
        let started = Instant::now();
        let error = match PairLock::exclusive_with_timeout(&fixture.manifest, timeout) {
            Ok(_) => panic!("exclusive publication lock must not bypass an active reader lock"),
            Err(error) => error,
        };

        assert!(started.elapsed() >= timeout);
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(error.to_string().contains("timed out"));
        assert!(error.to_string().contains("retry"));
    }

    #[test]
    fn existing_owner_writable_corrupted_generation_is_recreated_at_copy_boundary() {
        let fixture = PairFixture::new("corrupted-generation-snapshot");
        fixture.publish_initial("old");
        let (new_artifact, new_manifest) = fixture.stage("new");
        let new_sha256 = sha256(&new_artifact);
        let corrupted_generation =
            crate::artifact::pair::generation_path(&fixture.artifact, &new_sha256);
        fs::write(&corrupted_generation, b"corrupted generation cache").unwrap();

        let receipt =
            ArtifactPublicationReceipt::issue_test(&new_artifact, &fixture.artifact).unwrap();
        let telemetry = super::publish_artifact_pair(
            receipt,
            &new_manifest,
            &fixture.artifact,
            &fixture.manifest,
        )
        .unwrap();

        assert_eq!(telemetry.generation_copy_count, 1);
        assert_eq!(telemetry.generation_copy_verify_sha_pass_count, 1);
        verify_pair_files(&fixture.artifact, &fixture.manifest).unwrap();
        assert_eq!(sha256(&corrupted_generation), new_sha256);
        assert_eq!(fixture.marker(), "new");
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
        let stale_generation =
            crate::artifact::pair::generation_path(&fixture.artifact, &"f".repeat(64));
        fs::create_dir_all(stale_generation.parent().unwrap()).unwrap();
        fs::write(&stale_generation, b"untrusted stale generation").unwrap();
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
        assert!(!stale_generation.exists());
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
            let generation_directory =
                crate::artifact::pair::generation_directory_for_test(&self.artifact);
            let mut actual = if generation_directory.exists() {
                fs::read_dir(&generation_directory)
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            actual.sort();
            let expected = verify_pair_files(&self.artifact, &self.manifest)
                .ok()
                .map(|sha256| {
                    vec![
                        crate::artifact::pair::generation_path(&self.artifact, &sha256),
                        crate::artifact::pair::generation_trust_path_for_test(
                            &self.artifact,
                            &sha256,
                        ),
                    ]
                })
                .unwrap_or_default();
            let mut expected = expected;
            expected.sort();
            assert_eq!(actual, expected);
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
            .execute_batch(
                "CREATE TABLE marker(value TEXT NOT NULL);
                 CREATE TABLE artifact_metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);",
            )
            .unwrap();
        connection
            .execute("INSERT INTO marker(value) VALUES (?1)", [value])
            .unwrap();
        connection
            .execute(
                "INSERT INTO artifact_metadata(key, value) VALUES ('artifact_contract_version', ?1)",
                [crate::ARTIFACT_CONTRACT_VERSION],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO artifact_metadata(key, value) VALUES ('schema_version', ?1)",
                [crate::ARTIFACT_SCHEMA_VERSION],
            )
            .unwrap();
    }

    fn write_manifest(path: &Path, artifact: &Path) {
        let hash = sha256(artifact);
        fs::write(
            path,
            format!(
                r#"{{"manifest_version":"{}","artifact_contract_version":"{}","schema_version":"{}","build":{{"artifact_sha256":"{hash}"}}}}"#,
                crate::ARTIFACT_MANIFEST_VERSION,
                crate::ARTIFACT_CONTRACT_VERSION,
                crate::ARTIFACT_SCHEMA_VERSION,
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
