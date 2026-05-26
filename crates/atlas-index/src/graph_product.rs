use std::collections::BTreeMap;

use atlas_artifact::schema::remaster_links;
use atlas_domain::{RecordKey, RemasterLinkSource};
use atlas_record::PersistedRecord;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    GraphReferenceEdge, RecordLoadOptions, ReferenceEdgeDirection, SearchError, SqliteIndexReader,
    records,
};

pub trait GraphProductIndex {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, SearchError>;

    fn variant_group_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<VariantGroup>, SearchError>;

    fn variant_groups_by_base_name(
        &self,
        normalized_base_name: &str,
    ) -> Result<Vec<VariantGroup>, SearchError>;

    fn remaster_links_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<RemasterLinks>, SearchError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariantGroup {
    pub seed: Option<RecordKey>,
    pub variant_group_key: Option<String>,
    pub records: Vec<PersistedRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinks {
    pub seed: RecordKey,
    pub links: Vec<RemasterLinkRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinkRecord {
    pub remaster_record: PersistedRecord,
    pub legacy_record: PersistedRecord,
    pub source: RemasterLinkSource,
    pub source_ref: String,
}

impl GraphProductIndex for SqliteIndexReader {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, SearchError> {
        Ok(SqliteIndexReader::reference_edges_for_seed(
            self, seed, direction,
        )?)
    }

    fn variant_group_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<VariantGroup>, SearchError> {
        variant_group_for_record(self.connection(), seed)
    }

    fn variant_groups_by_base_name(
        &self,
        normalized_base_name: &str,
    ) -> Result<Vec<VariantGroup>, SearchError> {
        variant_groups_by_base_name(self.connection(), normalized_base_name)
    }

    fn remaster_links_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<RemasterLinks>, SearchError> {
        remaster_links_for_record(self.connection(), seed)
    }
}

fn variant_group_for_record(
    connection: &Connection,
    seed: &RecordKey,
) -> Result<Option<VariantGroup>, SearchError> {
    let seed_key = seed.to_string();
    let variant_group_key = connection
        .query_row(
            "SELECT variant_group_key FROM records WHERE record_key = ?1",
            params![seed_key],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?;
    let Some(variant_group_key) = variant_group_key else {
        return Ok(None);
    };
    let Some(group_key) = variant_group_key.clone() else {
        return Ok(Some(VariantGroup {
            seed: Some(seed.clone()),
            variant_group_key,
            records: Vec::new(),
        }));
    };

    variant_group_by_key(connection, &group_key, Some(seed.clone())).map(Some)
}

fn variant_groups_by_base_name(
    connection: &Connection,
    normalized_base_name: &str,
) -> Result<Vec<VariantGroup>, SearchError> {
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT variant_group_key
             FROM records
             WHERE variant_group_key IS NOT NULL
               AND LOWER(TRIM(variant_base_name)) = ?1
             ORDER BY variant_group_key",
        )
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?;
    let group_keys = statement
        .query_map(params![normalized_base_name], |row| row.get::<_, String>(0))
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?;
    group_keys
        .into_iter()
        .map(|group_key| variant_group_by_key(connection, &group_key, None))
        .collect()
}

fn variant_group_by_key(
    connection: &Connection,
    group_key: &str,
    seed: Option<RecordKey>,
) -> Result<VariantGroup, SearchError> {
    let mut statement = connection
        .prepare(
            "SELECT record_key
             FROM records
             WHERE variant_group_key = ?1
             ORDER BY COALESCE(level, 9223372036854775807), name, record_key",
        )
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?;
    let keys = statement
        .query_map(params![group_key], |row| row.get::<_, String>(0))
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))
                .and_then(|key| {
                    RecordKey::parse(&key).map_err(|error| {
                        crate::RecordLoadError::InvalidData(format!(
                            "records.record_key must be a valid record key: {error}"
                        ))
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let records = load_records_preserving_order(connection, &keys)?;
    Ok(VariantGroup {
        seed,
        variant_group_key: Some(group_key.to_string()),
        records,
    })
}

fn remaster_links_for_record(
    connection: &Connection,
    seed: &RecordKey,
) -> Result<Option<RemasterLinks>, SearchError> {
    let seed_key = seed.to_string();
    let exists = connection
        .query_row(
            "SELECT 1 FROM records WHERE record_key = ?1",
            params![seed_key],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?
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
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?;
    let link_rows = statement
        .query_map(params![seed.to_string()], remaster_link_row)
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| crate::RecordLoadError::QueryFailed(error.to_string()))?;
    let keys = link_rows
        .iter()
        .flat_map(|link| {
            [
                link.remaster_record_key.clone(),
                link.legacy_record_key.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let records_by_key = records::load_persisted_records_by_key_from_connection_with_options(
        connection,
        &keys,
        RecordLoadOptions::omit_raw_json(),
    )?
    .into_iter()
    .map(|record| (record.key.clone(), record))
    .collect::<BTreeMap<_, _>>();
    let links = link_rows
        .into_iter()
        .map(|link| {
            let remaster_record = records_by_key
                .get(&link.remaster_record_key)
                .cloned()
                .ok_or_else(|| {
                    crate::RecordLoadError::InvalidData(format!(
                        "remaster link target `{}` was not found",
                        link.remaster_record_key
                    ))
                })?;
            let legacy_record = records_by_key
                .get(&link.legacy_record_key)
                .cloned()
                .ok_or_else(|| {
                    crate::RecordLoadError::InvalidData(format!(
                        "remaster link target `{}` was not found",
                        link.legacy_record_key
                    ))
                })?;
            Ok(RemasterLinkRecord {
                remaster_record,
                legacy_record,
                source: link.source,
                source_ref: link.source_ref,
            })
        })
        .collect::<Result<Vec<_>, crate::RecordLoadError>>()?;
    Ok(Some(RemasterLinks {
        seed: seed.clone(),
        links,
    }))
}

fn load_records_preserving_order(
    connection: &Connection,
    keys: &[RecordKey],
) -> Result<Vec<PersistedRecord>, SearchError> {
    let by_key = records::load_persisted_records_by_key_from_connection_with_options(
        connection,
        keys,
        RecordLoadOptions::omit_raw_json(),
    )?
    .into_iter()
    .map(|record| (record.key.clone(), record))
    .collect::<BTreeMap<_, _>>();
    Ok(keys
        .iter()
        .filter_map(|key| by_key.get(key).cloned())
        .collect::<Vec<_>>())
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
