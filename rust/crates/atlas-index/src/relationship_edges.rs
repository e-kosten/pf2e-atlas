use atlas_artifact::schema::{Column, reference_edges};
use atlas_domain::RecordKey;
use atlas_record::{ContentSourceKind, ContentVisibility};
use rusqlite::{Connection, params};

use crate::database::ReferenceEdgeDirection;
use crate::filters::default_reference_edge_sql_predicate;
use crate::records::RecordLoadError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphReferenceEdge {
    pub from_record_key: RecordKey,
    pub to_record_key: RecordKey,
    pub display_text: Option<String>,
    pub reference_text: String,
    pub source_kind: ContentSourceKind,
    pub visibility: ContentVisibility,
}

pub(crate) fn read_reference_edges_for_seed(
    connection: &Connection,
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
        source_kind = aliased_reference_column(alias, reference_edges::columns::SOURCE_KIND),
        visibility = aliased_reference_column(alias, reference_edges::columns::VISIBILITY),
        key_column = aliased_reference_column(alias, key_column),
        order_column = aliased_reference_column(alias, order_column),
        default_predicate = default_reference_edge_sql_predicate(alias),
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    statement
        .query_map(params![seed.to_string()], |row| {
            let from = row.get::<_, String>(0)?;
            let to = row.get::<_, String>(1)?;
            let source_kind = row.get::<_, String>(4)?;
            let visibility = row.get::<_, String>(5)?;
            Ok((from, to, row.get(2)?, row.get(3)?, source_kind, visibility))
        })
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
                .and_then(
                    |(from, to, display_text, reference_text, source_kind, visibility)| {
                        Ok(GraphReferenceEdge {
                            from_record_key: RecordKey::parse(&from)
                                .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?,
                            to_record_key: RecordKey::parse(&to)
                                .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?,
                            display_text,
                            reference_text,
                            source_kind: ContentSourceKind::from_canonical(&source_kind)
                                .ok_or_else(|| {
                                    RecordLoadError::InvalidData(format!(
                                        "unknown content source kind `{source_kind}`"
                                    ))
                                })?,
                            visibility: ContentVisibility::from_canonical(&visibility).ok_or_else(
                                || {
                                    RecordLoadError::InvalidData(format!(
                                        "unknown content visibility `{visibility}`"
                                    ))
                                },
                            )?,
                        })
                    },
                )
        })
        .collect()
}

fn aliased_reference_column(alias: &str, column: Column) -> String {
    format!("{}.{}", alias, column.name())
}
