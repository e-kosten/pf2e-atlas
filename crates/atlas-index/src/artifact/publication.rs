use std::fs;
use std::path::{Path, PathBuf};

use crate::IndexWriteError;

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
    )
}

fn publish_artifact_pair_with_hook(
    staged_artifact: &Path,
    staged_manifest: &Path,
    target_artifact: &Path,
    target_manifest: &Path,
    before_manifest_publish: impl FnOnce() -> Result<(), IndexWriteError>,
) -> Result<(), IndexWriteError> {
    let backup = backup_path(target_artifact);
    let target_companions = sqlite_companions(target_artifact);
    let backup_companions = sqlite_companions(&backup);
    let had_previous = target_artifact.exists();
    if had_previous {
        let _ = fs::remove_file(&backup);
        fs::hard_link(target_artifact, &backup)
            .or_else(|_| fs::copy(target_artifact, &backup).map(|_| ()))
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        for (target, companion_backup) in target_companions.iter().zip(&backup_companions) {
            if target.exists() {
                fs::rename(target, companion_backup)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
            }
        }
    }
    if let Err(error) = fs::rename(staged_artifact, target_artifact) {
        let _ = fs::remove_file(&backup);
        return Err(IndexWriteError::WriteFailed(error.to_string()));
    }
    let manifest_result = before_manifest_publish().and_then(|()| {
        fs::rename(staged_manifest, target_manifest)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
    });
    if let Err(error) = manifest_result {
        if had_previous {
            fs::rename(&backup, target_artifact).map_err(|restore| {
                IndexWriteError::WriteFailed(format!(
                    "{error}; also failed to restore prior artifact: {restore}"
                ))
            })?;
            for (target, companion_backup) in target_companions.iter().zip(&backup_companions) {
                if companion_backup.exists() {
                    fs::rename(companion_backup, target)
                        .map_err(|restore| IndexWriteError::WriteFailed(restore.to_string()))?;
                }
            }
        } else {
            let _ = fs::remove_file(target_artifact);
        }
        return Err(error);
    }
    let _ = fs::remove_file(backup);
    for backup in backup_companions {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

fn sqlite_companions(path: &Path) -> [PathBuf; 2] {
    ["-wal", "-shm"].map(|suffix| {
        let mut value = path.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    })
}

fn backup_path(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_os_string();
    name.push(format!(".pair-backup-{}", std::process::id()));
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn manifest_failure_restores_prior_matching_pair() {
        let root = std::env::temp_dir().join(format!(
            "atlas-pair-publication-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        let staged_artifact = root.join("staged.sqlite");
        let staged_manifest = root.join("staged.json");
        fs::write(&artifact, b"old artifact").unwrap();
        fs::write(&manifest, b"old manifest").unwrap();
        fs::write(&staged_artifact, b"new artifact").unwrap();
        fs::write(&staged_manifest, b"new manifest").unwrap();
        let error = publish_artifact_pair_with_hook(
            &staged_artifact,
            &staged_manifest,
            &artifact,
            &manifest,
            || {
                Err(IndexWriteError::WriteFailed(
                    "injected manifest failure".to_string(),
                ))
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("injected manifest failure"));
        assert_eq!(fs::read(&artifact).unwrap(), b"old artifact");
        assert_eq!(fs::read(&manifest).unwrap(), b"old manifest");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn concurrent_reader_observes_only_matching_old_or_new_pair() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let root =
            std::env::temp_dir().join(format!("atlas-pair-concurrent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        let staged_artifact = root.join("staged.sqlite");
        let staged_manifest = root.join("staged.json");
        sqlite_with_marker(&artifact, "old");
        sqlite_with_marker(&staged_artifact, "new");
        write_manifest(&manifest, &artifact);
        write_manifest(&staged_manifest, &staged_artifact);

        let running = Arc::new(AtomicBool::new(true));
        let reader_running = Arc::clone(&running);
        let reader_artifact = artifact.clone();
        let reader = std::thread::spawn(move || {
            let mut observed = Vec::new();
            while reader_running.load(Ordering::Acquire) {
                let index = crate::SqliteIndexReader::open_read_only(&reader_artifact).unwrap();
                let connection = index.validation_connection().unwrap();
                let marker: String = connection
                    .query_row("SELECT value FROM marker", [], |row| row.get(0))
                    .unwrap();
                assert!(marker == "old" || marker == "new");
                observed.push(marker);
            }
            observed
        });
        publish_artifact_pair_with_hook(
            &staged_artifact,
            &staged_manifest,
            &artifact,
            &manifest,
            || {
                std::thread::sleep(std::time::Duration::from_millis(20));
                Ok(())
            },
        )
        .unwrap();
        running.store(false, Ordering::Release);
        let observed = reader.join().unwrap();
        assert!(!observed.is_empty());
        fs::remove_dir_all(root).unwrap();
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
        let hash = format!("{:x}", Sha256::digest(fs::read(artifact).unwrap()));
        fs::write(
            path,
            format!(r#"{{"build":{{"artifact_sha256":"{hash}"}}}}"#),
        )
        .unwrap();
    }
}
