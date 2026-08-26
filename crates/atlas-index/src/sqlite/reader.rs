use std::cell::RefCell;
use std::io::Read;
use std::path::{Path, PathBuf};

use diesel::connection::SimpleConnection;
use diesel::{Connection as DieselConnection, SqliteConnection};
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::IndexValidationError;
use crate::read::search::vector::register_sqlite_vec_extension;

pub struct SqliteIndexReader {
    path: PathBuf,
    diesel_connection: RefCell<SqliteConnection>,
}

impl SqliteIndexReader {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(IndexValidationError::Unavailable(format!(
                "unable to open database file: {}",
                path.display()
            )));
        }
        verify_adjacent_manifest_pair(&path)?;
        let database_url = read_only_sqlite_uri(&path)?;
        let mut diesel_connection = SqliteConnection::establish(&database_url)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        diesel_connection
            .batch_execute("PRAGMA query_only = ON")
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self {
            path,
            diesel_connection: RefCell::new(diesel_connection),
        })
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
        Connection::open_with_flags(&self.path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))
    }

    pub(crate) fn with_diesel_connection<T>(
        &self,
        f: impl FnOnce(&mut SqliteConnection) -> T,
    ) -> T {
        f(&mut self.diesel_connection.borrow_mut())
    }
}

#[derive(Deserialize)]
struct PairManifest {
    build: PairManifestBuild,
}

#[derive(Deserialize)]
struct PairManifestBuild {
    artifact_sha256: String,
}

fn verify_adjacent_manifest_pair(path: &Path) -> Result<(), IndexValidationError> {
    let manifest_path = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("manifest.json");
    if !manifest_path.exists() {
        return Ok(());
    }
    for _ in 0..100 {
        let before = std::fs::read(&manifest_path)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        let actual = sha256_file(path)?;
        let after = std::fs::read(&manifest_path)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        if before != after {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }
        let manifest: PairManifest = serde_json::from_slice(&after).map_err(|error| {
            IndexValidationError::Unavailable(format!(
                "adjacent manifest is not a v2 pair manifest: {error}; rebuild the artifact and manifest together"
            ))
        })?;
        if actual == manifest.build.artifact_sha256 {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    Err(IndexValidationError::Unavailable(
        "SQLite artifact and adjacent manifest are not a matching published pair; rebuild them together with `atlas index build`".to_string(),
    ))
}

fn sha256_file(path: &Path) -> Result<String, IndexValidationError> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

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
    Ok(format!("file:{escaped}?mode=ro"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn read_only_uri_rejects_non_utf8_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(b"atlas-index-\xff.sqlite".to_vec()));
        let error = read_only_sqlite_uri(&path).expect_err("non-UTF-8 path should be rejected");

        assert!(matches!(error, IndexValidationError::Unavailable(_)));
        assert!(error.to_string().contains("not valid UTF-8"));
    }

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
}
