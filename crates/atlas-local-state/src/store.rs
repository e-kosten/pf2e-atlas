use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::schema;
use crate::{LocalStateResult, SavedLists};

#[derive(Debug, Clone)]
pub struct LocalStateStore {
    path: PathBuf,
}

impl LocalStateStore {
    pub fn open(path: impl Into<PathBuf>) -> LocalStateResult<Self> {
        let path = path.into();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        let store = Self { path };
        store.initialize()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn saved_lists(&self) -> SavedLists<'_> {
        SavedLists::new(self)
    }

    pub(crate) fn connection(&self) -> LocalStateResult<Connection> {
        let connection = Connection::open(&self.path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(connection)
    }

    fn initialize(&self) -> LocalStateResult<()> {
        let connection = self.connection()?;
        schema::initialize(&connection)
    }
}
