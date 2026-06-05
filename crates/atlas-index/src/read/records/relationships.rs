use atlas_record::{RecordAlias, ReferenceEdge, RemasterLink};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::schema::{record_aliases, reference_edges, remaster_links};

use super::RecordLoadError;
use super::parse::{
    parse_alias_source, parse_content_source_kind, parse_content_visibility, parse_record_key,
    parse_reference_relation_kind, parse_remaster_link_source,
};

pub(super) fn read_reference_edges(
    connection: &mut SqliteConnection,
) -> Result<Vec<ReferenceEdge>, RecordLoadError> {
    let rows = reference_edges::table
        .select(ReferenceEdgeRow::as_select())
        .order((
            reference_edges::from_record_key.asc(),
            reference_edges::to_record_key.asc(),
            reference_edges::source_kind.asc(),
            reference_edges::reference_text.asc(),
        ))
        .load::<ReferenceEdgeRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter()
        .map(|row| {
            Ok(ReferenceEdge {
                from_record_key: parse_record_key(&row.from_record_key)?,
                to_record_key: parse_record_key(&row.to_record_key)?,
                display_text: row.display_text,
                reference_text: row.reference_text,
                relation_kind: parse_reference_relation_kind(&row.relation_kind)?,
                source_kind: parse_content_source_kind(&row.source_kind)?,
                visibility: parse_content_visibility(&row.visibility)?,
            })
        })
        .collect()
}

pub(super) fn read_aliases(
    connection: &mut SqliteConnection,
) -> Result<Vec<RecordAlias>, RecordLoadError> {
    let rows = record_aliases::table
        .select(RecordAliasRow::as_select())
        .order((
            record_aliases::canonical_record_key.asc(),
            record_aliases::normalized_alias.asc(),
            record_aliases::source_kind.asc(),
            record_aliases::source_ref.asc(),
        ))
        .load::<RecordAliasRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter()
        .map(|row| {
            Ok(RecordAlias {
                canonical_record_key: parse_record_key(&row.canonical_record_key)?,
                alias_text: row.alias_text,
                normalized_alias: row.normalized_alias,
                source: parse_alias_source(&row.source_kind)?,
                source_ref: row.source_ref,
            })
        })
        .collect()
}

pub(super) fn read_remaster_links(
    connection: &mut SqliteConnection,
) -> Result<Vec<RemasterLink>, RecordLoadError> {
    let rows = remaster_links::table
        .select(RemasterLinkRow::as_select())
        .order((
            remaster_links::remaster_record_key.asc(),
            remaster_links::legacy_record_key.asc(),
            remaster_links::source_kind.asc(),
            remaster_links::source_ref.asc(),
        ))
        .load::<RemasterLinkRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter()
        .map(|row| {
            Ok(RemasterLink {
                remaster_record_key: parse_record_key(&row.remaster_record_key)?,
                legacy_record_key: parse_record_key(&row.legacy_record_key)?,
                source: parse_remaster_link_source(&row.source_kind)?,
                source_ref: row.source_ref,
            })
        })
        .collect()
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = reference_edges)]
#[diesel(check_for_backend(Sqlite))]
struct ReferenceEdgeRow {
    from_record_key: String,
    to_record_key: String,
    display_text: Option<String>,
    reference_text: String,
    relation_kind: String,
    source_kind: String,
    visibility: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = record_aliases)]
#[diesel(check_for_backend(Sqlite))]
struct RecordAliasRow {
    canonical_record_key: String,
    alias_text: String,
    normalized_alias: String,
    source_kind: String,
    source_ref: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = remaster_links)]
#[diesel(check_for_backend(Sqlite))]
struct RemasterLinkRow {
    remaster_record_key: String,
    legacy_record_key: String,
    source_kind: String,
    source_ref: String,
}
