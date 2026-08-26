use std::cell::RefCell;
use std::fs::File;
use std::path::{Path, PathBuf};

use diesel::connection::SimpleConnection;
use diesel::{Connection as DieselConnection, SqliteConnection};
use rusqlite::{Connection, OpenFlags};

use crate::IndexValidationError;
use crate::artifact::pair::{PairLock, VerifiedArtifactFile, adjacent_manifest_path};
use crate::read::search::vector::register_sqlite_vec_extension;

pub struct SqliteIndexReader {
    path: PathBuf,
    artifact_file: File,
    _verified_artifact_sha256: Option<String>,
    diesel_connection: RefCell<SqliteConnection>,
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
        after_verification();
        let database_url = immutable_read_only_sqlite_uri(&verified.file, &path)?;
        let mut diesel_connection = SqliteConnection::establish(&database_url)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        diesel_connection
            .batch_execute("PRAGMA query_only = ON")
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        drop(pair_lock);
        Ok(Self {
            path,
            artifact_file: verified.file,
            _verified_artifact_sha256: Some(verified.sha256),
            diesel_connection: RefCell::new(diesel_connection),
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
        let mut diesel_connection = SqliteConnection::establish(&database_url)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        diesel_connection
            .batch_execute("PRAGMA query_only = ON")
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self {
            path,
            artifact_file,
            _verified_artifact_sha256: None,
            diesel_connection: RefCell::new(diesel_connection),
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

    pub(crate) fn validation_connection(&self) -> Result<Connection, IndexValidationError> {
        let database_url = immutable_read_only_sqlite_uri(&self.artifact_file, &self.path)?;
        Connection::open_with_flags(
            database_url,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))
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

    #[cfg(unix)]
    #[test]
    fn opened_database_generation_is_the_one_verified_before_path_swap() {
        let root = unique_root("verified-generation");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        let replacement_artifact = root.join("replacement.sqlite");
        let replacement_manifest = root.join("replacement.json");
        sqlite_with_marker(&artifact, "verified-old");
        sqlite_with_marker(&replacement_artifact, "visible-new");
        write_manifest(&manifest, &artifact);
        write_manifest(&replacement_manifest, &replacement_artifact);
        let old_hash = sha256(&artifact);

        let reader = SqliteIndexReader::open_bound_read_only_with_hook(&artifact, || {
            std::fs::rename(&replacement_artifact, &artifact).unwrap();
            std::fs::rename(&replacement_manifest, &manifest).unwrap();
        })
        .unwrap();

        let marker: String = reader
            .validation_connection()
            .unwrap()
            .query_row("SELECT value FROM marker", [], |row| row.get(0))
            .unwrap();
        assert_eq!(marker, "verified-old");
        assert_eq!(reader.verified_artifact_sha256(), Some(old_hash.as_str()));
        assert_ne!(sha256(&artifact), old_hash);
        std::fs::remove_dir_all(root).unwrap();
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
