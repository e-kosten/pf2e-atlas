use atlas_domain::{RecordKey, SearchFilterNode};
use diesel::sql_types::{Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::filters::{FilterCompileError, SqliteEligibleRecordKeyset};
use crate::schema_inventory::{
    record_aliases as artifact_record_aliases, records as artifact_records,
};
use crate::sqlite::raw_sql::{SqlBindValue, bind_sql_query};
use crate::{RecordIdentityMatch, RecordIdentityMatchKind};

pub fn resolve_record_identity_matches(
    connection: &mut SqliteConnection,
    query: &str,
    normalized_query: &str,
    filter: Option<&SearchFilterNode>,
) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
    for query in [
        identity_name_query(query),
        identity_normalized_name_query(normalized_query),
        identity_alias_query(normalized_query),
        identity_variant_name_query(normalized_query),
    ] {
        let matches = read_identity_matches(connection, filter, query)?;
        if !matches.is_empty() {
            return Ok(matches);
        }
    }
    Ok(Vec::new())
}

struct IdentityQuery {
    match_kind: RecordIdentityMatchKind,
    from_sql: String,
    where_sql: String,
    parameters: Vec<SqlBindValue>,
    order_sql: String,
}

fn identity_name_query(query: &str) -> IdentityQuery {
    IdentityQuery {
        match_kind: RecordIdentityMatchKind::Name,
        from_sql: format!("{} r", artifact_records::TABLE.name()),
        where_sql: "r.name = ?".to_string(),
        parameters: vec![SqlBindValue::Text(query.to_string())],
        order_sql: "r.record_key ASC".to_string(),
    }
}

fn identity_normalized_name_query(normalized_query: &str) -> IdentityQuery {
    IdentityQuery {
        match_kind: RecordIdentityMatchKind::NormalizedName,
        from_sql: format!("{} r", artifact_records::TABLE.name()),
        where_sql: "r.variant_label IS NULL AND r.normalized_name = ?".to_string(),
        parameters: vec![SqlBindValue::Text(normalized_query.to_string())],
        order_sql: "r.record_key ASC".to_string(),
    }
}

fn identity_alias_query(normalized_query: &str) -> IdentityQuery {
    IdentityQuery {
        match_kind: RecordIdentityMatchKind::Alias,
        from_sql: format!(
            "{} r JOIN {} a ON a.canonical_record_key = r.record_key",
            artifact_records::TABLE.name(),
            artifact_record_aliases::TABLE.name()
        ),
        where_sql: "a.normalized_alias = ?".to_string(),
        parameters: vec![SqlBindValue::Text(normalized_query.to_string())],
        order_sql: "r.record_key ASC, a.source_kind ASC, a.source_ref ASC".to_string(),
    }
}

fn identity_variant_name_query(normalized_query: &str) -> IdentityQuery {
    IdentityQuery {
        match_kind: RecordIdentityMatchKind::VariantName,
        from_sql: format!("{} r", artifact_records::TABLE.name()),
        where_sql: "r.variant_label IS NOT NULL AND r.normalized_name = ?".to_string(),
        parameters: vec![SqlBindValue::Text(normalized_query.to_string())],
        order_sql: "r.record_key ASC".to_string(),
    }
}

fn read_identity_matches(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    identity_query: IdentityQuery,
) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
            builder.extend(identity_query.parameters);
            format!(
                "SELECT
           r.record_key,
           {matched_text} AS matched_text,
           {alias_source} AS alias_source,
           {alias_source_ref} AS alias_source_ref
         FROM {from_sql}
         JOIN eligible e ON e.record_key = r.record_key
         WHERE {where_sql}
         ORDER BY {order_sql}",
                matched_text = match identity_query.match_kind {
                    RecordIdentityMatchKind::Name => "r.name",
                    RecordIdentityMatchKind::NormalizedName
                    | RecordIdentityMatchKind::VariantName => "r.normalized_name",
                    RecordIdentityMatchKind::Alias => "a.alias_text",
                },
                alias_source = match identity_query.match_kind {
                    RecordIdentityMatchKind::Alias => "a.source_kind",
                    RecordIdentityMatchKind::Name
                    | RecordIdentityMatchKind::NormalizedName
                    | RecordIdentityMatchKind::VariantName => "NULL",
                },
                alias_source_ref = match identity_query.match_kind {
                    RecordIdentityMatchKind::Alias => "a.source_ref",
                    RecordIdentityMatchKind::Name
                    | RecordIdentityMatchKind::NormalizedName
                    | RecordIdentityMatchKind::VariantName => "NULL",
                },
                from_sql = identity_query.from_sql,
                where_sql = identity_query.where_sql,
                order_sql = identity_query.order_sql,
            )
        });
    let mut matches = bind_sql_query(query.sql, &query.parameters)
        .load::<IdentityMatchRow>(connection)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .into_iter()
        .map(|row| identity_match_from_row(row, identity_query.match_kind))
        .collect::<Result<Vec<_>, _>>()?;
    matches.dedup_by(|left, right| left.record_key == right.record_key);
    Ok(matches)
}

fn identity_match_from_row(
    row: IdentityMatchRow,
    match_kind: RecordIdentityMatchKind,
) -> Result<RecordIdentityMatch, FilterCompileError> {
    let record_key = RecordKey::parse(&row.record_key).map_err(|error| {
        FilterCompileError::InvalidValue(format!(
            "identity result record_key `{}` is invalid: {error}",
            row.record_key
        ))
    })?;

    Ok(RecordIdentityMatch {
        record_key,
        match_kind,
        matched_text: row.matched_text,
        alias_source: row.alias_source,
        alias_source_ref: row.alias_source_ref,
    })
}

#[derive(QueryableByName)]
struct IdentityMatchRow {
    #[diesel(sql_type = Text)]
    record_key: String,
    #[diesel(sql_type = Text)]
    matched_text: String,
    #[diesel(sql_type = Nullable<Text>)]
    alias_source: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    alias_source_ref: Option<String>,
}
