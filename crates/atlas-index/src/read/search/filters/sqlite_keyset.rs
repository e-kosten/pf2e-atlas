use crate::artifact::inventory::records;
use crate::read::sql::{CountRow, RecordKeyRow, SqlBindValue, bind_sql_query};
use crate::sqlite::SqliteIndexReader;
use atlas_domain::{RecordKey, SearchFilterNode};
use diesel::{RunQueryDsl, SqliteConnection};

use super::FilterCompiler;
use super::error::FilterCompileError;
use super::sql_render::{RECORDS_ALIAS, record_column};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SqliteEligibleRecordKeyset<'a> {
    filter: Option<&'a SearchFilterNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SqliteFilterSqlQuery {
    pub sql: String,
    pub parameters: Vec<SqlBindValue>,
}

pub(crate) struct SqliteFilterSqlBuilder<'a> {
    parameters: &'a mut Vec<SqlBindValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompiledSqliteEligibleRecordKeyset {
    select_sql: String,
    parameters: Vec<SqlBindValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SqliteFilteredRecordSort {
    RecordKeyAsc,
    NameAsc,
    LevelAsc,
    LevelDesc,
    PriceAsc,
    PriceDesc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilteredRecordSort {
    RecordKey,
    Alphabetical,
    LevelAsc,
    LevelDesc,
    PriceAsc,
    PriceDesc,
    Random { seed: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredRecordKeyPage {
    pub record_keys: Vec<RecordKey>,
    pub total: u64,
}

impl<'a> SqliteEligibleRecordKeyset<'a> {
    pub(crate) fn new(filter: Option<&'a SearchFilterNode>) -> Self {
        Self { filter }
    }

    pub(crate) fn compile(self) -> Result<CompiledSqliteEligibleRecordKeyset, FilterCompileError> {
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

        Ok(CompiledSqliteEligibleRecordKeyset {
            select_sql,
            parameters: compiler.parameters,
        })
    }
}

impl CompiledSqliteEligibleRecordKeyset {
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
        build_body: impl FnOnce(&mut SqliteFilterSqlBuilder<'_>) -> String,
    ) -> SqliteFilterSqlQuery {
        let cte = self.eligible_cte_sql();
        let mut parameters = self.parameters;
        let mut builder = SqliteFilterSqlBuilder {
            parameters: &mut parameters,
        };
        let body = build_body(&mut builder);
        SqliteFilterSqlQuery {
            sql: format!("WITH {cte} {body}"),
            parameters,
        }
    }

    pub(crate) fn count_query(self) -> SqliteFilterSqlQuery {
        self.with_eligible_cte(|_| "SELECT COUNT(*) AS count FROM eligible".to_string())
    }

    pub(crate) fn into_record_keys_query(
        self,
        sort: SqliteFilteredRecordSort,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> SqliteFilterSqlQuery {
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

impl SqliteFilterSqlBuilder<'_> {
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

impl SqliteFilteredRecordSort {
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

impl SqliteIndexReader {
    pub fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, FilterCompileError> {
        self.with_diesel_connection(|connection| {
            list_filtered_record_keys(connection, filter, sort, limit, offset)
        })
    }
}

fn list_filtered_record_keys(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    sort: FilteredRecordSort,
    limit: u32,
    offset: u32,
) -> Result<FilteredRecordKeyPage, FilterCompileError> {
    match sort {
        FilteredRecordSort::Random { seed } => {
            let query = SqliteEligibleRecordKeyset::new(filter)
                .compile()?
                .into_record_keys_query(SqliteFilteredRecordSort::RecordKeyAsc, None, None);
            let mut record_keys = read_record_keys(connection, &query)?;
            record_keys.sort_by_key(|key| seeded_key_hash(seed, &key.to_string()));
            let total = record_keys.len() as u64;
            let record_keys = record_keys
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect();
            Ok(FilteredRecordKeyPage { record_keys, total })
        }
        sort => {
            let total = count_filtered_records(connection, filter)?;
            let query = SqliteEligibleRecordKeyset::new(filter)
                .compile()?
                .into_record_keys_query(sql_sort(sort), Some(limit), Some(offset));
            Ok(FilteredRecordKeyPage {
                record_keys: read_record_keys(connection, &query)?,
                total,
            })
        }
    }
}

fn count_filtered_records(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, FilterCompileError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .count_query();
    bind_sql_query(query.sql, &query.parameters)
        .get_result::<CountRow>(connection)
        .map(|row| row.count as u64)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
}

fn read_record_keys(
    connection: &mut SqliteConnection,
    query: &SqliteFilterSqlQuery,
) -> Result<Vec<RecordKey>, FilterCompileError> {
    bind_sql_query(query.sql.clone(), &query.parameters)
        .load::<RecordKeyRow>(connection)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .into_iter()
        .map(|row| {
            RecordKey::parse(&row.record_key)
                .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
        })
        .collect()
}

fn sql_sort(sort: FilteredRecordSort) -> SqliteFilteredRecordSort {
    match sort {
        FilteredRecordSort::RecordKey => SqliteFilteredRecordSort::RecordKeyAsc,
        FilteredRecordSort::Alphabetical => SqliteFilteredRecordSort::NameAsc,
        FilteredRecordSort::LevelAsc => SqliteFilteredRecordSort::LevelAsc,
        FilteredRecordSort::LevelDesc => SqliteFilteredRecordSort::LevelDesc,
        FilteredRecordSort::PriceAsc => SqliteFilteredRecordSort::PriceAsc,
        FilteredRecordSort::PriceDesc => SqliteFilteredRecordSort::PriceDesc,
        FilteredRecordSort::Random { .. } => SqliteFilteredRecordSort::RecordKeyAsc,
    }
}

fn seeded_key_hash(seed: u64, key: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64 ^ seed;
    for byte in key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
