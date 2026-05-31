use crate::sqlite::raw_sql::SqlBindValue;
use atlas_domain::{RecordKey, RemasterLinkSource};
use diesel::OptionalExtension;
use diesel::sql_types::{Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::sqlite::raw_sql::{CountRow, bind_sql_query};
use crate::{GraphReferenceEdge, RecordLoadError, ReferenceEdgeDirection, SqliteIndexReader};

pub trait GraphReadIndex {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError>;

    fn variant_group_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<IndexVariantGroup>, RecordLoadError>;

    fn variant_groups_by_base_name(
        &self,
        normalized_base_name: &str,
    ) -> Result<Vec<IndexVariantGroup>, RecordLoadError>;

    fn remaster_links_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<IndexRemasterLinks>, RecordLoadError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexVariantGroup {
    pub variant_group_key: Option<String>,
    pub record_keys: Vec<RecordKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexRemasterLinks {
    pub links: Vec<IndexRemasterLinkRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexRemasterLinkRecord {
    pub remaster_record_key: RecordKey,
    pub legacy_record_key: RecordKey,
    pub source: RemasterLinkSource,
    pub source_ref: String,
}

impl GraphReadIndex for SqliteIndexReader {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
        SqliteIndexReader::reference_edges_for_seed(self, seed, direction)
    }

    fn variant_group_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<IndexVariantGroup>, RecordLoadError> {
        self.with_diesel_connection(|connection| variant_group_for_record(connection, seed))
    }

    fn variant_groups_by_base_name(
        &self,
        normalized_base_name: &str,
    ) -> Result<Vec<IndexVariantGroup>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            variant_groups_by_base_name(connection, normalized_base_name)
        })
    }

    fn remaster_links_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<IndexRemasterLinks>, RecordLoadError> {
        self.with_diesel_connection(|connection| remaster_links_for_record(connection, seed))
    }
}

fn variant_group_for_record(
    connection: &mut SqliteConnection,
    seed: &RecordKey,
) -> Result<Option<IndexVariantGroup>, RecordLoadError> {
    let variant_group_key = bind_sql_query(
        "SELECT variant_group_key FROM records WHERE record_key = ?1".to_string(),
        &[SqlBindValue::Text(seed.to_string())],
    )
    .get_result::<VariantGroupKeyRow>(connection)
    .optional()
    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    .map(|row| row.variant_group_key);
    let Some(variant_group_key) = variant_group_key else {
        return Ok(None);
    };
    let Some(group_key) = variant_group_key.clone() else {
        return Ok(Some(IndexVariantGroup {
            variant_group_key,
            record_keys: Vec::new(),
        }));
    };

    variant_group_by_key(connection, &group_key, true).map(Some)
}

fn variant_groups_by_base_name(
    connection: &mut SqliteConnection,
    normalized_base_name: &str,
) -> Result<Vec<IndexVariantGroup>, RecordLoadError> {
    let group_keys = bind_sql_query(
        "SELECT DISTINCT variant_group_key
         FROM records
         WHERE variant_group_key IS NOT NULL
           AND LOWER(TRIM(variant_base_name)) = ?1
           AND is_default_visible = 1
         ORDER BY variant_group_key"
            .to_string(),
        &[SqlBindValue::Text(normalized_base_name.to_string())],
    )
    .load::<VariantGroupKeyRequiredRow>(connection)
    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    group_keys
        .into_iter()
        .map(|row| variant_group_by_key(connection, &row.variant_group_key, false))
        .collect()
}

fn variant_group_by_key(
    connection: &mut SqliteConnection,
    group_key: &str,
    include_hidden: bool,
) -> Result<IndexVariantGroup, RecordLoadError> {
    let visibility_clause = if include_hidden {
        ""
    } else {
        " AND is_default_visible = 1"
    };
    let record_keys = bind_sql_query(
        format!(
            "SELECT record_key
             FROM records
             WHERE variant_group_key = ?1{visibility_clause}
             ORDER BY COALESCE(level, 9223372036854775807), name, record_key"
        ),
        &[SqlBindValue::Text(group_key.to_string())],
    )
    .load::<RecordKeyRow>(connection)
    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    .into_iter()
    .map(|row| {
        RecordKey::parse(&row.record_key).map_err(|error| {
            RecordLoadError::InvalidData(format!(
                "records.record_key must be a valid record key: {error}"
            ))
        })
    })
    .collect::<Result<Vec<_>, _>>()?;
    Ok(IndexVariantGroup {
        variant_group_key: Some(group_key.to_string()),
        record_keys,
    })
}

fn remaster_links_for_record(
    connection: &mut SqliteConnection,
    seed: &RecordKey,
) -> Result<Option<IndexRemasterLinks>, RecordLoadError> {
    let exists = bind_sql_query(
        "SELECT COUNT(*) AS count FROM records WHERE record_key = ?1".to_string(),
        &[SqlBindValue::Text(seed.to_string())],
    )
    .get_result::<CountRow>(connection)
    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    .count
        > 0;
    if !exists {
        return Ok(None);
    }

    let link_rows = bind_sql_query(
        "SELECT remaster_record_key, legacy_record_key, source_kind, source_ref
         FROM remaster_links
         WHERE remaster_record_key = ?1 OR legacy_record_key = ?1
         ORDER BY remaster_record_key, legacy_record_key, source_kind, source_ref"
            .to_string(),
        &[SqlBindValue::Text(seed.to_string())],
    )
    .load::<RemasterLinkRow>(connection)
    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let links = link_rows
        .into_iter()
        .map(|link| {
            Ok(IndexRemasterLinkRecord {
                remaster_record_key: RecordKey::parse(&link.remaster_record_key).map_err(
                    |error| {
                        RecordLoadError::InvalidData(format!(
                            "remaster_links.remaster_record_key must be a valid record key: {error}"
                        ))
                    },
                )?,
                legacy_record_key: RecordKey::parse(&link.legacy_record_key).map_err(|error| {
                    RecordLoadError::InvalidData(format!(
                        "remaster_links.legacy_record_key must be a valid record key: {error}"
                    ))
                })?,
                source: RemasterLinkSource::from_canonical(&link.source_kind).ok_or_else(|| {
                    RecordLoadError::InvalidData(format!(
                        "unknown remaster link source `{}`",
                        link.source_kind
                    ))
                })?,
                source_ref: link.source_ref,
            })
        })
        .collect::<Result<Vec<_>, RecordLoadError>>()?;
    Ok(Some(IndexRemasterLinks { links }))
}

#[derive(QueryableByName)]
struct VariantGroupKeyRow {
    #[diesel(sql_type = Nullable<Text>)]
    variant_group_key: Option<String>,
}

#[derive(QueryableByName)]
struct VariantGroupKeyRequiredRow {
    #[diesel(sql_type = Text)]
    variant_group_key: String,
}

#[derive(QueryableByName)]
struct RecordKeyRow {
    #[diesel(sql_type = Text)]
    record_key: String,
}

#[derive(QueryableByName)]
struct RemasterLinkRow {
    #[diesel(sql_type = Text)]
    remaster_record_key: String,
    #[diesel(sql_type = Text)]
    legacy_record_key: String,
    #[diesel(sql_type = Text)]
    source_kind: String,
    #[diesel(sql_type = Text)]
    source_ref: String,
}
