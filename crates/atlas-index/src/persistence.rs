//! Parameterized Diesel writes. Relation names and columns are executable-owned.
use crate::IndexError;
use diesel::{
    Connection, RunQueryDsl, SqliteConnection,
    connection::SimpleConnection,
    sql_types::{BigInt, Binary, Double, Nullable, Text},
};
use rusqlite::types::Value;
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProjectionRow {
    pub table: &'static str,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}
impl ProjectionRow {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            columns: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn push(&mut self, column: impl Into<String>, value: Value) {
        self.columns.push(column.into());
        self.values.push(value);
    }
    pub fn id(&mut self, column: &'static str, id: i64) {
        self.push(column, Value::Integer(id));
    }
    pub fn text(&mut self, column: &'static str, value: impl Into<String>) {
        self.push(column, Value::Text(value.into()));
    }
    pub fn insert(&self, connection: &mut SqliteConnection) -> Result<(), IndexError> {
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
    connection: &mut SqliteConnection,
    sql: &str,
    values: &[Value],
) -> Result<usize, IndexError> {
    let mut query = diesel::sql_query(sql).into_boxed::<diesel::sqlite::Sqlite>();
    for value in values {
        query = match value {
            Value::Null => query.bind::<Nullable<Binary>, _>(None::<Vec<u8>>),
            Value::Integer(n) => query.bind::<BigInt, _>(*n),
            Value::Real(n) => query.bind::<Double, _>(*n),
            Value::Text(s) => query.bind::<Text, _>(s),
            Value::Blob(b) => query.bind::<Binary, _>(b),
        };
    }
    Ok(query.execute(connection)?)
}
pub(crate) fn writable(path: &std::path::Path) -> Result<SqliteConnection, IndexError> {
    let path = path
        .to_str()
        .ok_or_else(|| IndexError::Unavailable("artifact path must be UTF-8".into()))?;
    let mut c =
        SqliteConnection::establish(path).map_err(|e| IndexError::Unavailable(e.to_string()))?;
    c.batch_execute(
        "PRAGMA foreign_keys=ON; PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;",
    )?;
    Ok(c)
}
