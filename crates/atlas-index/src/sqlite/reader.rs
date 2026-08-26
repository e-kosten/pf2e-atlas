use std::cell::{RefCell, RefMut};
use std::fs::File;
use std::path::{Path, PathBuf};

use diesel::connection::SimpleConnection;
use diesel::{Connection as DieselConnection, SqliteConnection};
use rusqlite::{Connection, OpenFlags};

use crate::IndexValidationError;
use crate::artifact::pair::{
    GenerationLease, PairLock, VerifiedArtifactFile, adjacent_manifest_path,
    open_verified_generation,
};
use crate::read::search::vector::register_sqlite_vec_extension;

pub struct SqliteIndexReader {
    path: PathBuf,
    _verified_artifact_sha256: Option<String>,
    diesel_connection: RefCell<SqliteConnection>,
    validation_connection: RefCell<Connection>,
    _artifact_file: File,
    // This field must remain after the SQLite handles and retained file so an
    // obsolete generation is removed only after this reader releases it.
    _generation_lease: Option<GenerationLease>,
}

impl SqliteIndexReader {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        Self::open_bound_read_only_with_hook(path.as_ref(), || {})
    }

    fn open_bound_read_only_with_hook(
        path: &Path,
        after_verification: impl FnOnce(),
    ) -> Result<Self, IndexValidationError> {
        let path = path.to_path_buf();
        let manifest_path = adjacent_manifest_path(&path);
        let pair_lock = PairLock::shared(&manifest_path)?;
        let verified = VerifiedArtifactFile::open(&path, &manifest_path)?;
        let (generation, generation_lease) =
            open_verified_generation(&path, &manifest_path, &verified)?;
        after_verification();
        let database_url = immutable_read_only_sqlite_uri(&generation.file, &generation.path)?;
        let (diesel_connection, validation_connection) = open_connections(&database_url)?;
        drop(pair_lock);
        Ok(Self {
            path,
            _verified_artifact_sha256: Some(verified.sha256),
            diesel_connection: RefCell::new(diesel_connection),
            validation_connection: RefCell::new(validation_connection),
            _artifact_file: generation.file,
            _generation_lease: Some(generation_lease),
        })
    }

    #[cfg(test)]
    pub(crate) fn open_unpublished_read_only(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        let path = path.as_ref().to_path_buf();
        let artifact_file = File::open(&path)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        let database_url = immutable_read_only_sqlite_uri(&artifact_file, &path)?;
        let (diesel_connection, validation_connection) = open_connections(&database_url)?;
        Ok(Self {
            path,
            _verified_artifact_sha256: None,
            diesel_connection: RefCell::new(diesel_connection),
            validation_connection: RefCell::new(validation_connection),
            _artifact_file: artifact_file,
            _generation_lease: None,
        })
    }

    #[cfg(test)]
    pub(crate) fn open_unpublished_read_only_with_vectors(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension().map_err(IndexValidationError::Unavailable)?;
        Self::open_unpublished_read_only(path)
    }

    pub fn open_read_only_with_vectors(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension().map_err(IndexValidationError::Unavailable)?;
        Self::open_read_only(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn validation_connection(
        &self,
    ) -> Result<RefMut<'_, Connection>, IndexValidationError> {
        Ok(self.validation_connection.borrow_mut())
    }

    pub(crate) fn with_diesel_connection<T>(
        &self,
        f: impl FnOnce(&mut SqliteConnection) -> T,
    ) -> T {
        f(&mut self.diesel_connection.borrow_mut())
    }

    #[cfg(test)]
    pub(crate) fn verified_artifact_sha256(&self) -> Option<&str> {
        self._verified_artifact_sha256.as_deref()
    }
}

fn open_connections(
    database_url: &str,
) -> Result<(SqliteConnection, Connection), IndexValidationError> {
    let mut diesel_connection = SqliteConnection::establish(database_url)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    diesel_connection
        .batch_execute("PRAGMA query_only = ON")
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    let validation_connection = Connection::open_with_flags(
        database_url,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    Ok((diesel_connection, validation_connection))
}

fn immutable_read_only_sqlite_uri(
    artifact_file: &File,
    display_path: &Path,
) -> Result<String, IndexValidationError> {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;

        let _ = display_path;
        Ok(format!(
            "file:/dev/fd/{}?mode=ro&immutable=1",
            artifact_file.as_raw_fd()
        ))
    }

    #[cfg(not(unix))]
    read_only_sqlite_uri(display_path)
}

#[cfg(not(unix))]
fn read_only_sqlite_uri(path: &Path) -> Result<String, IndexValidationError> {
    let path = path.to_str().ok_or_else(|| {
        IndexValidationError::Unavailable(format!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn old_unbound_manifest_is_rejected_with_rebuild_guidance() {
        let root = std::env::temp_dir().join(format!("atlas-old-pair-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("index.sqlite");
        rusqlite::Connection::open(&path).unwrap();
        std::fs::write(root.join("manifest.json"), r#"{"build":{}}"#).unwrap();
        let error = match SqliteIndexReader::open_read_only(&path) {
            Ok(_) => panic!("old unbound manifest must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("rebuild"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_manifest_hard_link_fails_closed() {
        let root = unique_root("missing-manifest-hard-link");
        let published = root.join("published");
        let unbound = root.join("unbound");
        std::fs::create_dir_all(&published).unwrap();
        std::fs::create_dir_all(&unbound).unwrap();
        let source = published.join("index.sqlite");
        sqlite_with_marker(&source, "bound");
        write_manifest(&published.join("manifest.json"), &source);
        let hard_link = unbound.join("index.sqlite");
        std::fs::hard_link(&source, &hard_link).unwrap();

        let error = match SqliteIndexReader::open_read_only(&hard_link) {
            Ok(_) => panic!("an otherwise valid hard link without its manifest must fail"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("required adjacent manifest is missing")
        );
        assert!(error.to_string().contains("rebuild"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generation_binding_allows_publish_while_old_reader_stays_coherent() {
        let root = unique_root("verified-generation");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        let replacement_artifact = root.join("replacement.sqlite");
        let replacement_manifest = root.join("replacement.json");
        create_valid_generation(&artifact, "Verified Old");
        create_valid_generation(&replacement_artifact, "Visible New");
        write_manifest(&manifest, &artifact);
        write_manifest(&replacement_manifest, &replacement_artifact);
        let old_hash = sha256(&artifact);
        let new_hash = sha256(&replacement_artifact);
        let old_generation = crate::artifact::pair::generation_path(&artifact, &old_hash);
        let new_generation = crate::artifact::pair::generation_path(&artifact, &new_hash);
        let (attempted_tx, attempted_rx) = std::sync::mpsc::channel();
        let (published_tx, published_rx) = std::sync::mpsc::channel();

        let reader = SqliteIndexReader::open_bound_read_only_with_hook(&artifact, || {
            let target_artifact = artifact.clone();
            let target_manifest = manifest.clone();
            std::thread::spawn(move || {
                let contention_probe = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(crate::artifact::pair::lock_path(&target_manifest))
                    .unwrap();
                let contention_error = contention_probe
                    .try_lock()
                    .expect_err("reader acquisition must hold the shared target lock");
                assert!(matches!(
                    contention_error,
                    std::fs::TryLockError::WouldBlock
                ));
                attempted_tx.send(()).unwrap();
                published_tx
                    .send(crate::publish_artifact_pair(
                        &replacement_artifact,
                        &replacement_manifest,
                        &target_artifact,
                        &target_manifest,
                    ))
                    .unwrap();
            });
            attempted_rx.recv().unwrap();
        })
        .unwrap();

        published_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap()
            .unwrap();
        assert_reader_generation(&reader, "Verified Old", &old_hash);
        drop(reader);
        assert!(!old_generation.exists());
        assert!(new_generation.exists());

        let reader = SqliteIndexReader::open_read_only(&artifact).unwrap();
        assert_reader_generation(&reader, "Visible New", &new_hash);
        drop(reader);
        assert_eq!(sha256(&artifact), new_hash);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn assert_reader_generation(reader: &SqliteIndexReader, name: &str, hash: &str) {
        assert_eq!(reader.verified_artifact_sha256(), Some(hash));
        assert_eq!(reader.check().unwrap().status, crate::ValidationStatus::Ok);
        assert_eq!(
            reader.validate().unwrap().status,
            crate::ValidationStatus::Ok
        );
        assert_eq!(reader.inspect().unwrap().records.total_records, 3);
        let hydrated = reader.load_hydrated_records().unwrap();
        assert_eq!(hydrated.len(), 3);
        assert_eq!(hydrated[0].record.identity.name, format!("{name} 1"));
        let records = reader.load_records().unwrap();
        assert_eq!(records[0].identity.name, format!("{name} 1"));
        let validation_name: String = reader
            .validation_connection()
            .unwrap()
            .query_row(
                "SELECT name FROM records WHERE record_key = 'actions:testAction1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(validation_name, format!("{name} 1"));
    }

    fn create_valid_generation(path: &Path, name: &str) {
        crate::tests::create_valid_artifact_database(&path.to_path_buf()).unwrap();
        let connection = rusqlite::Connection::open(path).unwrap();
        for index in 1..=3 {
            let record_key = format!("actions:testAction{index}");
            let record_name = format!("{name} {index}");
            connection
                .execute(
                    "UPDATE records SET name = ?1, normalized_name = ?2 WHERE record_key = ?3",
                    (&record_name, record_name.to_lowercase(), record_key),
                )
                .unwrap();
        }
    }

    fn unique_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "atlas-reader-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn sqlite_with_marker(path: &Path, marker: &str) {
        let connection = rusqlite::Connection::open(path).unwrap();
        connection
            .execute_batch("CREATE TABLE marker(value TEXT NOT NULL);")
            .unwrap();
        connection
            .execute("INSERT INTO marker(value) VALUES (?1)", [marker])
            .unwrap();
    }

    fn write_manifest(path: &Path, artifact: &Path) {
        std::fs::write(
            path,
            format!(
                r#"{{"manifest_version":"pf2e-atlas-artifact-manifest/v2","build":{{"artifact_sha256":"{}"}}}}"#,
                sha256(artifact)
            ),
        )
        .unwrap();
    }

    fn sha256(path: &Path) -> String {
        format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
    }
}
