use atlas_artifact::schema::{
    Column, Table, actor_records, item_records, record_metrics, record_traits, records,
    spell_records,
};
use atlas_domain::MetricOperator;
#[cfg(test)]
use rusqlite::types::Value;

pub(super) const RECORDS_ALIAS: &str = "r";
pub(super) const REFERENCE_EDGES_ALIAS: &str = "re";

pub(super) fn side_table_for_column(column: Column) -> Option<(&'static str, Table)> {
    match column.table() {
        table if table == actor_records::TABLE => Some(("a", actor_records::TABLE)),
        table if table == item_records::TABLE => Some(("i", item_records::TABLE)),
        table if table == spell_records::TABLE => Some(("s", spell_records::TABLE)),
        _ => None,
    }
}

pub(super) fn record_key_column(table: Table) -> Column {
    if table == actor_records::TABLE {
        actor_records::columns::RECORD_KEY
    } else if table == item_records::TABLE {
        item_records::columns::RECORD_KEY
    } else if table == spell_records::TABLE {
        spell_records::columns::RECORD_KEY
    } else if table == record_traits::TABLE {
        record_traits::columns::RECORD_KEY
    } else if table == record_metrics::TABLE {
        record_metrics::columns::RECORD_KEY
    } else {
        records::columns::RECORD_KEY
    }
}

pub(super) fn record_column(column: Column) -> String {
    aliased_column(RECORDS_ALIAS, column)
}

pub(super) fn aliased_column(alias: &str, column: Column) -> String {
    format!("{alias}.{}", column.name())
}

pub(super) fn metric_operator_sql(op: MetricOperator) -> &'static str {
    match op {
        MetricOperator::Eq => "=",
        MetricOperator::NotEq => "<>",
        MetricOperator::Gt => ">",
        MetricOperator::Gte => ">=",
        MetricOperator::Lt => "<",
        MetricOperator::Lte => "<=",
    }
}

pub(super) fn json_array_contains_sql(column: &str, placeholder: &str) -> String {
    format!(
        "EXISTS (SELECT 1 FROM json_each(COALESCE({column}, '[]')) j WHERE j.value = {placeholder})"
    )
}

pub(super) fn json_array_empty_sql(column: &str) -> String {
    format!("NOT EXISTS (SELECT 1 FROM json_each(COALESCE({column}, '[]')))")
}

#[cfg(test)]
pub(super) fn push_integer_parameter(parameters: &mut Vec<Value>, value: u32) -> String {
    parameters.push(Value::Integer(i64::from(value)));
    format!("?{}", parameters.len())
}

pub(super) fn contains_like_pattern(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len() + 2);
    pattern.push('%');
    for character in value.chars() {
        match character {
            '%' | '_' | '\\' => {
                pattern.push('\\');
                pattern.push(character);
            }
            _ => pattern.push(character),
        }
    }
    pattern.push('%');
    pattern
}
