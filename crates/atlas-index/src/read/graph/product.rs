use atlas_domain::{RecordKey, RemasterLinkSource};
use diesel::OptionalExtension;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Text};
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::schema::{records, remaster_links};
use crate::{GraphReferenceEdge, RecordLoadError, ReferenceEdgeDirection, SqliteIndexReader};

pub trait ReferenceReadIndex {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError>;
}

pub trait VariantReadIndex {
    fn variant_group_for_record(
        &self,
        seed: &RecordKey,
    ) -> Result<Option<IndexVariantGroup>, RecordLoadError>;

    fn variant_groups_by_base_name(
        &self,
        normalized_base_name: &str,
    ) -> Result<Vec<IndexVariantGroup>, RecordLoadError>;
}

pub trait RemasterReadIndex {
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

impl ReferenceReadIndex for SqliteIndexReader {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
        SqliteIndexReader::reference_edges_for_seed(self, seed, direction)
    }
}

impl VariantReadIndex for SqliteIndexReader {
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
}

impl RemasterReadIndex for SqliteIndexReader {
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
    let variant_group_key = records::table
        .filter(records::record_key.eq(seed.to_string()))
        .select(records::variant_group_key)
        .get_result::<Option<String>>(connection)
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
    connection: &mut SqliteConnection,
    normalized_base_name: &str,
) -> Result<Vec<IndexVariantGroup>, RecordLoadError> {
    let group_keys = records::table
        .filter(records::variant_group_key.is_not_null())
        .filter(
            sql::<Bool>("LOWER(TRIM(variant_base_name)) = ").bind::<Text, _>(normalized_base_name),
        )
        .filter(records::is_default_visible.eq(true))
        .select(records::variant_group_key)
        .distinct()
        .order(records::variant_group_key.asc())
        .load::<Option<String>>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    group_keys
        .into_iter()
        .map(|group_key| variant_group_by_key(connection, &group_key, false))
        .collect()
}

fn variant_group_by_key(
    connection: &mut SqliteConnection,
    group_key: &str,
    include_hidden: bool,
) -> Result<IndexVariantGroup, RecordLoadError> {
    let mut query = records::table
        .filter(records::variant_group_key.eq(group_key))
        .select(records::record_key)
        .order((
            sql::<diesel::sql_types::BigInt>("COALESCE(level, 9223372036854775807)"),
            records::name.asc(),
            records::record_key.asc(),
        ))
        .into_boxed();
    if !include_hidden {
        query = query.filter(records::is_default_visible.eq(true));
    }
    let record_keys = query
        .load::<String>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        .into_iter()
        .map(|record_key| {
            RecordKey::parse(&record_key).map_err(|error| {
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
    let seed_key = seed.to_string();
    let exists = records::table
        .filter(records::record_key.eq(&seed_key))
        .count()
        .get_result::<i64>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
        > 0;
    if !exists {
        return Ok(None);
    }

    let link_rows = remaster_links::table
        .filter(
            remaster_links::remaster_record_key
                .eq(&seed_key)
                .or(remaster_links::legacy_record_key.eq(&seed_key)),
        )
        .select(RemasterLinkRow::as_select())
        .order((
            remaster_links::remaster_record_key.asc(),
            remaster_links::legacy_record_key.asc(),
            remaster_links::source_kind.asc(),
            remaster_links::source_ref.asc(),
        ))
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = remaster_links)]
#[diesel(check_for_backend(Sqlite))]
struct RemasterLinkRow {
    remaster_record_key: String,
    legacy_record_key: String,
    source_kind: String,
    source_ref: String,
}
