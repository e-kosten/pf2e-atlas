use crate::schema_inventory::records;
use crate::sqlite::raw_sql::SqlBindValue;
use atlas_domain::SearchFilterNode;

use super::FilterCompiler;
use super::error::FilterCompileError;
use super::sql_render::{RECORDS_ALIAS, record_column};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EligibleRecordKeyset<'a> {
    filter: Option<&'a SearchFilterNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FilterSqlQuery {
    pub sql: String,
    pub parameters: Vec<SqlBindValue>,
}

pub(crate) struct FilterSqlBuilder<'a> {
    parameters: &'a mut Vec<SqlBindValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompiledEligibleRecordKeyset {
    select_sql: String,
    parameters: Vec<SqlBindValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilteredRecordSort {
    RecordKeyAsc,
    NameAsc,
    LevelAsc,
    LevelDesc,
    PriceAsc,
    PriceDesc,
}

impl<'a> EligibleRecordKeyset<'a> {
    pub(crate) fn new(filter: Option<&'a SearchFilterNode>) -> Self {
        Self { filter }
    }

    pub(crate) fn compile(self) -> Result<CompiledEligibleRecordKeyset, FilterCompileError> {
        let mut compiler = FilterCompiler::default();
        let base = format!(
            "SELECT {record_key} FROM {records_table} {records_alias} WHERE {default_visible} = 1",
            record_key = record_column(records::columns::RECORD_KEY),
            records_table = records::TABLE.name(),
            records_alias = RECORDS_ALIAS,
            default_visible = record_column(records::columns::IS_DEFAULT_VISIBLE),
        );
        let select_sql = match self.filter {
            Some(filter) => format!("{base} AND ({})", compiler.compile_node(filter)?),
            None => base,
        };

        Ok(CompiledEligibleRecordKeyset {
            select_sql,
            parameters: compiler.parameters,
        })
    }
}

impl CompiledEligibleRecordKeyset {
    #[cfg(test)]
    pub(crate) fn select_sql(&self) -> &str {
        &self.select_sql
    }

    #[cfg(test)]
    pub(crate) fn parameters(&self) -> &[SqlBindValue] {
        &self.parameters
    }

    pub(crate) fn eligible_cte_sql(&self) -> String {
        format!("eligible(record_key) AS ({})", self.select_sql)
    }

    pub(crate) fn with_eligible_cte(
        self,
        build_body: impl FnOnce(&mut FilterSqlBuilder<'_>) -> String,
    ) -> FilterSqlQuery {
        let cte = self.eligible_cte_sql();
        let mut parameters = self.parameters;
        let mut builder = FilterSqlBuilder {
            parameters: &mut parameters,
        };
        let body = build_body(&mut builder);
        FilterSqlQuery {
            sql: format!("WITH {cte} {body}"),
            parameters,
        }
    }

    pub(crate) fn count_query(self) -> FilterSqlQuery {
        self.with_eligible_cte(|_| "SELECT COUNT(*) AS count FROM eligible".to_string())
    }

    pub(crate) fn into_record_keys_query(
        self,
        sort: FilteredRecordSort,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> FilterSqlQuery {
        self.with_eligible_cte(|builder| {
            let mut sql = format!(
                "SELECT {}
                 FROM eligible e
                 JOIN {} {} ON {} = e.record_key
                 ORDER BY {}",
                record_column(records::columns::RECORD_KEY),
                records::TABLE.name(),
                RECORDS_ALIAS,
                record_column(records::columns::RECORD_KEY),
                sort.sql()
            );

            match (limit, offset) {
                (Some(limit), Some(offset)) => {
                    let limit_placeholder = builder.push_integer(i64::from(limit));
                    let offset_placeholder = builder.push_integer(i64::from(offset));
                    sql.push_str(&format!(
                        " LIMIT {limit_placeholder} OFFSET {offset_placeholder}"
                    ));
                }
                (Some(limit), None) => {
                    let limit_placeholder = builder.push_integer(i64::from(limit));
                    sql.push_str(&format!(" LIMIT {limit_placeholder}"));
                }
                (None, Some(offset)) => {
                    let offset_placeholder = builder.push_integer(i64::from(offset));
                    sql.push_str(&format!(" LIMIT -1 OFFSET {offset_placeholder}"));
                }
                (None, None) => {}
            }

            sql
        })
    }
}

impl FilterSqlBuilder<'_> {
    pub(crate) fn push(&mut self, value: SqlBindValue) -> String {
        self.parameters.push(value);
        format!("?{}", self.parameters.len())
    }

    pub(crate) fn push_integer(&mut self, value: i64) -> String {
        self.push(SqlBindValue::Integer(value))
    }

    pub(crate) fn push_text(&mut self, value: impl Into<String>) -> String {
        self.push(SqlBindValue::Text(value.into()))
    }

    pub(crate) fn push_blob(&mut self, value: Vec<u8>) -> String {
        self.push(SqlBindValue::Blob(value))
    }

    pub(crate) fn extend(&mut self, values: Vec<SqlBindValue>) {
        self.parameters.extend(values);
    }
}

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
            Self::PriceAsc => format!(
                "{} IS NULL ASC, {} ASC, {} ASC, {} ASC",
                record_column(records::columns::PRICE_CP),
                record_column(records::columns::PRICE_CP),
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
            Self::PriceDesc => format!(
                "{} IS NULL ASC, {} DESC, {} ASC, {} ASC",
                record_column(records::columns::PRICE_CP),
                record_column(records::columns::PRICE_CP),
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
        }
    }
}
