//! SQLite STRICT ANY keeps integer and real source arms distinct.
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
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
#[derive(Debug)]
pub(crate) struct SourceNumber(pub Number);
impl FromSql for SourceNumber {
    fn column_result(raw: ValueRef<'_>) -> FromSqlResult<Self> {
        match raw {
            ValueRef::Integer(value) => Ok(Self(Number::from(value))),
            ValueRef::Real(value) => Number::from_f64(value)
                .map(Self)
                .ok_or(FromSqlError::InvalidType),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}
