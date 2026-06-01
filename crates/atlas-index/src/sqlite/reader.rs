use std::cell::RefCell;
use std::path::{Path, PathBuf};

use diesel::connection::SimpleConnection;
use diesel::{Connection as DieselConnection, SqliteConnection};
use rusqlite::{Connection, OpenFlags};

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
}
