use atlas_artifact::schema::remaster_links;
use atlas_domain::{RecordKey, RemasterLinkSource};
use rusqlite::{Connection, OptionalExtension, params};

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
        variant_group_for_record(self.connection(), seed)
    }

    fn variant_groups_by_base_name(
        &self,
        normalized_base_name: &str,
    ) -> Result<Vec<IndexVariantGroup>, RecordLoadError> {
        variant_groups_by_base_name(self.connection(), normalized_base_name)
    }

    fn remaster_links_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<IndexRemasterLinks>, RecordLoadError> {
        remaster_links_for_record(self.connection(), seed)
    }
}

fn variant_group_for_record(
    connection: &Connection,
    seed: &RecordKey,
) -> Result<Option<IndexVariantGroup>, RecordLoadError> {
    let variant_group_key = connection
        .query_row(
            "SELECT variant_group_key FROM records WHERE record_key = ?1",
            params![seed.to_string()],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
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
    connection: &Connection,
    normalized_base_name: &str,
) -> Result<Vec<IndexVariantGroup>, RecordLoadError> {
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT variant_group_key
             FROM records
             WHERE variant_group_key IS NOT NULL
               AND LOWER(TRIM(variant_base_name)) = ?1
               AND is_default_visible = 1
             ORDER BY variant_group_key",
        )
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let group_keys = statement
        .query_map(params![normalized_base_name], |row| row.get::<_, String>(0))
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    group_keys
        .into_iter()
        .map(|group_key| variant_group_by_key(connection, &group_key, false))
        .collect()
}

fn variant_group_by_key(
    connection: &Connection,
    group_key: &str,
    include_hidden: bool,
) -> Result<IndexVariantGroup, RecordLoadError> {
    let visibility_clause = if include_hidden {
        ""
    } else {
        " AND is_default_visible = 1"
    };
    let mut statement = connection
        .prepare(&format!(
            "SELECT record_key
             FROM records
             WHERE variant_group_key = ?1{visibility_clause}
             ORDER BY COALESCE(level, 9223372036854775807), name, record_key"
        ))
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let record_keys = statement
        .query_map(params![group_key], |row| row.get::<_, String>(0))
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
                .and_then(|key| {
                    RecordKey::parse(&key).map_err(|error| {
                        RecordLoadError::InvalidData(format!(
                            "records.record_key must be a valid record key: {error}"
                        ))
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(IndexVariantGroup {
        variant_group_key: Some(group_key.to_string()),
        record_keys,
    })
}

fn remaster_links_for_record(
    connection: &Connection,
    seed: &RecordKey,
) -> Result<Option<IndexRemasterLinks>, RecordLoadError> {
    let exists = connection
        .query_row(
            "SELECT 1 FROM records WHERE record_key = ?1",
            params![seed.to_string()],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .is_some();
    if !exists {
        return Ok(None);
    }

    let mut statement = connection
        .prepare(
            "SELECT remaster_record_key, legacy_record_key, source_kind, source_ref
             FROM remaster_links
             WHERE remaster_record_key = ?1 OR legacy_record_key = ?1
             ORDER BY remaster_record_key, legacy_record_key, source_kind, source_ref",
        )
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let link_rows = statement
        .query_map(params![seed.to_string()], remaster_link_row)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let links = link_rows
        .into_iter()
        .map(|link| {
            Ok(IndexRemasterLinkRecord {
                remaster_record_key: link.remaster_record_key,
                legacy_record_key: link.legacy_record_key,
                source: link.source,
                source_ref: link.source_ref,
            })
        })
        .collect::<Result<Vec<_>, RecordLoadError>>()?;
    Ok(Some(IndexRemasterLinks { links }))
}

struct RemasterLinkRow {
    remaster_record_key: RecordKey,
    legacy_record_key: RecordKey,
    source: RemasterLinkSource,
    source_ref: String,
}

fn remaster_link_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RemasterLinkRow> {
    let remaster_record_key =
        row.get::<_, String>(remaster_links::columns::REMASTER_RECORD_KEY.name())?;
    let legacy_record_key =
        row.get::<_, String>(remaster_links::columns::LEGACY_RECORD_KEY.name())?;
    let source_kind = row.get::<_, String>(remaster_links::columns::SOURCE_KIND.name())?;
    Ok(RemasterLinkRow {
        remaster_record_key: RecordKey::parse(&remaster_record_key).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        legacy_record_key: RecordKey::parse(&legacy_record_key).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        source: RemasterLinkSource::from_canonical(&source_kind).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                format!("unknown remaster link source `{source_kind}`").into(),
            )
        })?,
        source_ref: row.get(remaster_links::columns::SOURCE_REF.name())?,
    })
}
