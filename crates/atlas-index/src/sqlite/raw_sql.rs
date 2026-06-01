use diesel::query_builder::{BoxedSqlQuery, SqlQuery};
use diesel::sql_types::{BigInt, Binary, Double, Text};
use diesel::sqlite::Sqlite;
use diesel::{QueryableByName, sql_query};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SqlBindValue {
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(QueryableByName)]
pub(crate) struct CountRow {
    #[diesel(sql_type = BigInt)]
    pub(crate) count: i64,
}

#[derive(QueryableByName)]
pub(crate) struct RecordKeyRow {
    #[diesel(sql_type = Text)]
    pub(crate) record_key: String,
}

pub(crate) fn bind_sql_query(
    sql: String,
    parameters: &[SqlBindValue],
) -> BoxedSqlQuery<'static, Sqlite, SqlQuery> {
    let mut query = sql_query(sql).into_boxed::<Sqlite>();
    for parameter in parameters {
        query = match parameter {
            SqlBindValue::Integer(value) => query.bind::<BigInt, _>(*value),
            SqlBindValue::Real(value) => query.bind::<Double, _>(*value),
            SqlBindValue::Text(value) => query.bind::<Text, _>(value.clone()),
            SqlBindValue::Blob(value) => query.bind::<Binary, _>(value.clone()),
        };
    }
    query
}
