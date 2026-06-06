use std::collections::{BTreeMap, BTreeSet};

use crate::artifact::inventory::{Column, reference_edges};
use crate::read::sql::SqlBindValue;
use atlas_domain::RecordKey;
use atlas_record::{ContentSourceKind, ContentVisibility, ReferenceRelationKind};
use diesel::sql_types::{Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::read::records::RecordLoadError;
use crate::read::search::filters::default_reference_edge_sql_predicate;
use crate::read::sql::bind_sql_query;
use crate::sqlite::SqliteIndexReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceEdgeDirection {
    Outgoing,
    Backlink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphReferenceEdge {
    pub from_record_key: RecordKey,
    pub to_record_key: RecordKey,
    pub display_text: Option<String>,
    pub reference_text: String,
    pub relation_kind: ReferenceRelationKind,
    pub source_kind: ContentSourceKind,
    pub visibility: ContentVisibility,
}

impl SqliteIndexReader {
    pub fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            read_reference_edges_for_seed(connection, seed, direction)
        })
    }

    pub fn outgoing_reference_targets_for_records(
        &self,
        records: &[RecordKey],
    ) -> Result<BTreeMap<RecordKey, BTreeSet<RecordKey>>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            read_outgoing_reference_targets_for_records(connection, records)
        })
    }
}

pub(crate) fn read_reference_edges_for_seed(
    connection: &mut SqliteConnection,
    seed: &RecordKey,
    direction: ReferenceEdgeDirection,
) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
    let (key_column, order_column) = match direction {
        ReferenceEdgeDirection::Outgoing => (
            reference_edges::columns::FROM_RECORD_KEY,
            reference_edges::columns::TO_RECORD_KEY,
        ),
        ReferenceEdgeDirection::Backlink => (
            reference_edges::columns::TO_RECORD_KEY,
            reference_edges::columns::FROM_RECORD_KEY,
        ),
    };
    let alias = "re";
    let sql = format!(
        "SELECT
           {from_record_key},
           {to_record_key},
           {display_text},
           {reference_text},
           {relation_kind},
           {source_kind},
           {visibility}
         FROM {table} {alias}
         WHERE {key_column} = ?1
           AND {default_predicate}
         ORDER BY COALESCE({display_text}, ''), {order_column}, {reference_text}",
        table = reference_edges::TABLE.name(),
        from_record_key =
            aliased_reference_column(alias, reference_edges::columns::FROM_RECORD_KEY),
        to_record_key = aliased_reference_column(alias, reference_edges::columns::TO_RECORD_KEY),
        display_text = aliased_reference_column(alias, reference_edges::columns::DISPLAY_TEXT),
        reference_text = aliased_reference_column(alias, reference_edges::columns::REFERENCE_TEXT),
        relation_kind = aliased_reference_column(alias, reference_edges::columns::RELATION_KIND),
        source_kind = aliased_reference_column(alias, reference_edges::columns::SOURCE_KIND),
        visibility = aliased_reference_column(alias, reference_edges::columns::VISIBILITY),
        key_column = aliased_reference_column(alias, key_column),
        order_column = aliased_reference_column(alias, order_column),
        default_predicate = default_reference_edge_sql_predicate(alias),
    );
    bind_sql_query(sql, &[SqlBindValue::Text(seed.to_string())])
        .load::<ReferenceEdgeRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .into_iter()
        .map(|row| {
            Ok(GraphReferenceEdge {
                from_record_key: RecordKey::parse(&row.from_record_key)
                    .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?,
                to_record_key: RecordKey::parse(&row.to_record_key)
                    .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?,
                display_text: row.display_text,
                reference_text: row.reference_text,
                relation_kind: ReferenceRelationKind::from_canonical(&row.relation_kind)
                    .ok_or_else(|| {
                        RecordLoadError::InvalidData(format!(
                            "unknown reference relation kind `{}`",
                            row.relation_kind
                        ))
                    })?,
                source_kind: ContentSourceKind::from_canonical(&row.source_kind).ok_or_else(
                    || {
                        RecordLoadError::InvalidData(format!(
                            "unknown content source kind `{}`",
                            row.source_kind
                        ))
                    },
                )?,
                visibility: ContentVisibility::from_canonical(&row.visibility).ok_or_else(
                    || {
                        RecordLoadError::InvalidData(format!(
                            "unknown content visibility `{}`",
                            row.visibility
                        ))
                    },
                )?,
            })
        })
        .collect()
}

fn aliased_reference_column(alias: &str, column: Column) -> String {
    format!("{}.{}", alias, column.name())
}

pub(crate) fn read_outgoing_reference_targets_for_records(
    connection: &mut SqliteConnection,
    records: &[RecordKey],
) -> Result<BTreeMap<RecordKey, BTreeSet<RecordKey>>, RecordLoadError> {
    let mut targets_by_record = records
        .iter()
        .cloned()
        .map(|record| (record, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    if records.is_empty() {
        return Ok(targets_by_record);
    }

    let alias = "re";
    let placeholders = vec!["?"; records.len()].join(", ");
    let sql = format!(
        "SELECT
           {from_record_key},
           {to_record_key}
         FROM {table} {alias}
         WHERE {from_record_key} IN ({placeholders})
           AND {default_predicate}
         ORDER BY {from_record_key}, {to_record_key}",
        table = reference_edges::TABLE.name(),
        from_record_key =
            aliased_reference_column(alias, reference_edges::columns::FROM_RECORD_KEY),
        to_record_key = aliased_reference_column(alias, reference_edges::columns::TO_RECORD_KEY),
        default_predicate = default_reference_edge_sql_predicate(alias),
    );
    let parameters = records
        .iter()
        .map(|record| SqlBindValue::Text(record.to_string()))
        .collect::<Vec<_>>();
    for row in bind_sql_query(sql, &parameters)
        .load::<ReferenceTargetRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let from_record_key = RecordKey::parse(&row.from_record_key)
            .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?;
        let to_record_key = RecordKey::parse(&row.to_record_key)
            .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?;
        targets_by_record
            .entry(from_record_key)
            .or_default()
            .insert(to_record_key);
    }
    Ok(targets_by_record)
}

#[derive(QueryableByName)]
struct ReferenceEdgeRow {
    #[diesel(sql_type = Text)]
    from_record_key: String,
    #[diesel(sql_type = Text)]
    to_record_key: String,
    #[diesel(sql_type = Nullable<Text>)]
    display_text: Option<String>,
    #[diesel(sql_type = Text)]
    reference_text: String,
    #[diesel(sql_type = Text)]
    relation_kind: String,
    #[diesel(sql_type = Text)]
    source_kind: String,
    #[diesel(sql_type = Text)]
    visibility: String,
}

#[derive(QueryableByName)]
struct ReferenceTargetRow {
    #[diesel(sql_type = Text)]
    from_record_key: String,
    #[diesel(sql_type = Text)]
    to_record_key: String,
}
