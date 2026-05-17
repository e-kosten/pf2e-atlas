use atlas_artifact::schema::{
    record_alias_select_sql, reference_edge_select_sql, remaster_link_select_sql,
};
use atlas_record::{RecordAlias, ReferenceEdge, RemasterLink};
use rusqlite::Connection;

use super::RecordLoadError;
use super::parse::{
    optional_string, parse_alias_source, parse_content_source_kind, parse_content_visibility,
    parse_record_key, parse_remaster_link_source, required_string,
};

pub(super) fn read_reference_edges(
    connection: &Connection,
) -> Result<Vec<ReferenceEdge>, RecordLoadError> {
    let mut statement = connection
        .prepare(&reference_edge_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut edges = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        edges.push(ReferenceEdge {
            from_record_key: parse_record_key(&required_string(row, "from_record_key")?)?,
            to_record_key: parse_record_key(&required_string(row, "to_record_key")?)?,
            display_text: optional_string(row, "display_text")?,
            reference_text: required_string(row, "reference_text")?,
            source_kind: parse_content_source_kind(&required_string(row, "source_kind")?)?,
            visibility: parse_content_visibility(&required_string(row, "visibility")?)?,
        });
    }
    Ok(edges)
}

pub(super) fn read_aliases(connection: &Connection) -> Result<Vec<RecordAlias>, RecordLoadError> {
    let mut statement = connection
        .prepare(&record_alias_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut aliases = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        aliases.push(RecordAlias {
            canonical_record_key: parse_record_key(&required_string(row, "canonical_record_key")?)?,
            alias_text: required_string(row, "alias_text")?,
            normalized_alias: required_string(row, "normalized_alias")?,
            source: parse_alias_source(&required_string(row, "source_kind")?)?,
            source_ref: required_string(row, "source_ref")?,
        });
    }
    Ok(aliases)
}

pub(super) fn read_remaster_links(
    connection: &Connection,
) -> Result<Vec<RemasterLink>, RecordLoadError> {
    let mut statement = connection
        .prepare(&remaster_link_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut links = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        links.push(RemasterLink {
            remaster_record_key: parse_record_key(&required_string(row, "remaster_record_key")?)?,
            legacy_record_key: parse_record_key(&required_string(row, "legacy_record_key")?)?,
            source: parse_remaster_link_source(&required_string(row, "source_kind")?)?,
            source_ref: required_string(row, "source_ref")?,
        });
    }
    Ok(links)
}
