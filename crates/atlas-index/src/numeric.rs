//! SQLite STRICT ANY keeps integer and real source arms distinct.
use diesel::{
    deserialize::{self, FromSql},
    sqlite::{Sqlite, SqliteType, SqliteValue},
};
use serde_json::Number;
pub(crate) fn numeric_value(n: &Number) -> Result<rusqlite::types::Value, crate::IndexError> {
    if let Some(n) = n.as_i64() {
        return Ok(rusqlite::types::Value::Integer(n));
    }
    if n.is_u64() {
        return Err(crate::IndexError::Invalid(
            "unsigned numeric projection exceeds SQLite integer domain".into(),
        ));
    }
    n.as_f64()
        .filter(|n| n.is_finite())
        .map(rusqlite::types::Value::Real)
        .ok_or_else(|| crate::IndexError::Invalid("numeric projection is not finite".into()))
}
#[derive(diesel::sql_types::SqlType)]
#[diesel(sqlite_type(name = "Double"))]
pub struct SourceNumberSql;
#[derive(Debug, diesel::deserialize::FromSqlRow)]
pub(crate) struct SourceNumber(pub Number);
impl FromSql<SourceNumberSql, Sqlite> for SourceNumber {
    fn from_sql(mut raw: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        match raw.value_type() {
            Some(SqliteType::Long) => Ok(Self(Number::from(raw.read_long()))),
            Some(SqliteType::Double) => Number::from_f64(raw.read_double())
                .map(Self)
                .ok_or_else(|| "nonfinite source projection".into()),
            _ => Err("source numeric projection is not integer/real".into()),
        }
    }
}
