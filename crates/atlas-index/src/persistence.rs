//! Parameterized writes on the caller's SQLite connection.
use crate::IndexError;
use rusqlite::Connection;
use rusqlite::types::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ProjectionId {
    Root(i64),
    Local(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProjectionRow {
    pub table: &'static str,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
    ids: Vec<(usize, ProjectionId)>,
}
impl ProjectionRow {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            columns: Vec::new(),
            values: Vec::new(),
            ids: Vec::new(),
        }
    }
    pub fn push(&mut self, column: impl Into<String>, value: Value) {
        self.columns.push(column.into());
        self.values.push(value);
    }
    pub fn id(&mut self, column: &'static str, id: i64) {
        self.push(column, Value::Integer(id));
    }
    pub fn projection_id(&mut self, column: &'static str, id: ProjectionId) {
        self.ids.push((self.values.len(), id));
        let value = match id {
            ProjectionId::Root(value) | ProjectionId::Local(value) => value,
        };
        self.id(column, value);
    }
    /// Bind a row extracted with root zero and a zero-based allocation seed.
    pub fn bind_projection_ids(&mut self, root: i64, seed: i64) -> Result<(), IndexError> {
        for (cell, id) in self.ids.drain(..) {
            let value = match id {
                ProjectionId::Root(_) => root,
                ProjectionId::Local(offset) => seed
                    .checked_add(offset)
                    .ok_or_else(|| IndexError::Invalid("projection ID overflow".into()))?,
            };
            self.values[cell] = Value::Integer(value);
        }
        Ok(())
    }
    pub fn text(&mut self, column: &'static str, value: impl Into<String>) {
        self.push(column, Value::Text(value.into()));
    }
    pub fn insert(&self, connection: &Connection) -> Result<(), IndexError> {
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table,
            self.columns.join(","),
            vec!["?"; self.values.len()].join(",")
        );
        execute(connection, &sql, &self.values)?;
        Ok(())
    }
}
pub(crate) fn execute(
    connection: &Connection,
    sql: &str,
    values: &[Value],
) -> Result<usize, IndexError> {
    Ok(connection.execute(sql, rusqlite::params_from_iter(values))?)
}
