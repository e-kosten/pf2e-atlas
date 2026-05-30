use atlas_artifact::schema::Column;
use atlas_domain::RecordKey;
use rusqlite::types::Value;

pub(super) fn key_parameters(keys: &[RecordKey]) -> Vec<Value> {
    keys.iter()
        .map(|key| Value::Text(key.to_string()))
        .collect()
}

pub(super) fn select_by_keys_sql(
    table: &str,
    columns: &[Column],
    key_column: &str,
    order_by: &[&str],
    key_count: usize,
) -> String {
    let placeholders = (1..=key_count)
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let columns = columns
        .iter()
        .map(|column| column.name())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "SELECT {columns} FROM {table} WHERE {key_column} IN ({placeholders}) ORDER BY {order_by}",
        order_by = order_by.join(", ")
    )
}
