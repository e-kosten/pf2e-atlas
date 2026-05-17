use atlas_artifact::schema::records;
use atlas_domain::SearchFilterNode;
use rusqlite::types::Value;

use super::FilterCompiler;
use super::error::FilterCompileError;
#[cfg(test)]
use super::sql_render::push_integer_parameter;
use super::sql_render::{RECORDS_ALIAS, record_column};

#[derive(Debug, Clone, PartialEq)]
pub struct EligibleRecordsQuery {
    pub sql: String,
    pub parameters: Vec<Value>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub struct FilteredRecordKeysQuery {
    pub sql: String,
    pub parameters: Vec<Value>,
}

#[cfg(test)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilteredRecordSort {
    RecordKeyAsc,
    NameAsc,
    LevelAsc,
    LevelDesc,
}

pub(crate) fn compile_eligible_records_query(
    filter: Option<&SearchFilterNode>,
) -> Result<EligibleRecordsQuery, FilterCompileError> {
    let mut compiler = FilterCompiler::default();
    let base = format!(
        "SELECT {record_key} FROM {records_table} {records_alias} WHERE {default_visible} = 1",
        record_key = record_column(records::columns::RECORD_KEY),
        records_table = records::TABLE.name(),
        records_alias = RECORDS_ALIAS,
        default_visible = record_column(records::columns::IS_DEFAULT_VISIBLE),
    );
    let sql = match filter {
        Some(filter) => format!("{base} AND ({})", compiler.compile_node(filter)?),
        None => base,
    };

    Ok(EligibleRecordsQuery {
        sql,
        parameters: compiler.parameters,
    })
}

#[cfg(test)]
pub(crate) fn compile_filtered_record_keys_query(
    filter: Option<&SearchFilterNode>,
    sort: FilteredRecordSort,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<FilteredRecordKeysQuery, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    Ok(eligible.into_record_keys_query(sort, limit, offset))
}

#[cfg(test)]
impl EligibleRecordsQuery {
    pub fn into_record_keys_query(
        self,
        sort: FilteredRecordSort,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> FilteredRecordKeysQuery {
        let mut parameters = self.parameters;
        let mut sql = format!(
            "WITH eligible(record_key) AS ({})
             SELECT {}
             FROM eligible e
             JOIN {} {} ON {} = e.record_key
             ORDER BY {}",
            self.sql,
            record_column(records::columns::RECORD_KEY),
            records::TABLE.name(),
            RECORDS_ALIAS,
            record_column(records::columns::RECORD_KEY),
            sort.sql()
        );

        match (limit, offset) {
            (Some(limit), Some(offset)) => {
                let limit_placeholder = push_integer_parameter(&mut parameters, limit);
                let offset_placeholder = push_integer_parameter(&mut parameters, offset);
                sql.push_str(&format!(
                    " LIMIT {limit_placeholder} OFFSET {offset_placeholder}"
                ));
            }
            (Some(limit), None) => {
                let limit_placeholder = push_integer_parameter(&mut parameters, limit);
                sql.push_str(&format!(" LIMIT {limit_placeholder}"));
            }
            (None, Some(offset)) => {
                let offset_placeholder = push_integer_parameter(&mut parameters, offset);
                sql.push_str(&format!(" LIMIT -1 OFFSET {offset_placeholder}"));
            }
            (None, None) => {}
        }

        FilteredRecordKeysQuery { sql, parameters }
    }
}

#[cfg(test)]
impl FilteredRecordSort {
    fn sql(self) -> String {
        match self {
            Self::RecordKeyAsc => format!("{} ASC", record_column(records::columns::RECORD_KEY)),
            Self::NameAsc => format!(
                "{} ASC, {} ASC",
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
            Self::LevelAsc => format!(
                "{} IS NULL ASC, {} ASC, {} ASC, {} ASC",
                record_column(records::columns::LEVEL),
                record_column(records::columns::LEVEL),
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
            Self::LevelDesc => format!(
                "{} IS NULL ASC, {} DESC, {} ASC, {} ASC",
                record_column(records::columns::LEVEL),
                record_column(records::columns::LEVEL),
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
        }
    }
}
