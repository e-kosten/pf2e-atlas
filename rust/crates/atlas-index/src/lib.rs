#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use atlas_domain::{
    ARTIFACT_METADATA_TABLE, ArtifactMetadataSummary, ArtifactValidationReport,
    LEGACY_METADATA_TABLE, REQUIRED_ARTIFACT_METADATA_KEYS,
};
use rusqlite::{Connection, OpenFlags};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexValidationError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("index query failed: {0}")]
    QueryFailed(String),
}

pub fn validate_index(
    path: impl AsRef<Path>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let path = path.as_ref();
    let index = path.display().to_string();
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    if !table_exists(&connection, ARTIFACT_METADATA_TABLE)? {
        let legacy_schema_version = if table_exists(&connection, LEGACY_METADATA_TABLE)? {
            metadata_value(&connection, LEGACY_METADATA_TABLE, "schema_version")?
        } else {
            None
        };
        return Ok(ArtifactValidationReport::missing_artifact_metadata(
            index,
            legacy_schema_version,
        ));
    }

    let metadata = read_metadata(&connection, ARTIFACT_METADATA_TABLE)?;
    let summary = summarize_metadata(&metadata);
    let missing_keys = REQUIRED_ARTIFACT_METADATA_KEYS
        .iter()
        .filter(|key| {
            metadata
                .get(**key)
                .is_none_or(|value| value.trim().is_empty())
        })
        .map(|key| (*key).to_string())
        .collect::<Vec<_>>();

    if missing_keys.is_empty() {
        Ok(ArtifactValidationReport::ok(index, summary))
    } else {
        Ok(ArtifactValidationReport::missing_required_metadata(
            index,
            summary,
            missing_keys,
        ))
    }
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool, IndexValidationError> {
    let mut statement = connection
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([table])
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    rows.next()
        .map(|row| row.is_some())
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
}

fn metadata_value(
    connection: &Connection,
    table: &str,
    key: &str,
) -> Result<Option<String>, IndexValidationError> {
    let sql = format!("SELECT value FROM {table} WHERE key = ?1 LIMIT 1");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    match statement.query_row([key], |row| row.get::<_, String>(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(IndexValidationError::QueryFailed(error.to_string())),
    }
}

fn read_metadata(
    connection: &Connection,
    table: &str,
) -> Result<BTreeMap<String, String>, IndexValidationError> {
    let sql = format!("SELECT key, value FROM {table}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;

    let mut metadata = BTreeMap::new();
    for row in rows {
        let (key, value) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        metadata.insert(key, value);
    }
    Ok(metadata)
}

fn summarize_metadata(metadata: &BTreeMap<String, String>) -> ArtifactMetadataSummary {
    ArtifactMetadataSummary {
        artifact_contract_version: metadata.get("artifact_contract_version").cloned(),
        schema_version: metadata.get("schema_version").cloned(),
        source_signature: metadata.get("source_signature").cloned(),
        embedding_model_id: metadata.get("embedding_model_id").cloned(),
        embedding_dimensions: metadata.get("embedding_dimensions").cloned(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use atlas_domain::{ValidationCode, ValidationStatus};
    use rusqlite::Connection;

    use super::validate_index;

    #[test]
    fn reports_valid_artifact_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("valid");
        create_contract_database(&path)?;

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Ok);
        assert_eq!(report.code, ValidationCode::Ok);
        assert_eq!(report.artifact_contract_version.as_deref(), Some("0.1.0"));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn reports_legacy_metadata_without_accepting_it_as_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("legacy");
        let connection = Connection::open(&path)?;
        connection.execute(
            "CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        connection.execute(
            "INSERT INTO metadata (key, value) VALUES ('schema_version', '25')",
            [],
        )?;
        drop(connection);

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::MissingArtifactMetadata);
        assert_eq!(report.legacy_schema_version.as_deref(), Some("25"));
        fs::remove_file(path)?;
        Ok(())
    }

    fn create_contract_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::open(path)?;
        connection.execute(
            "CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        for (key, value) in [
            ("artifact_contract_version", "0.1.0"),
            ("schema_version", "1"),
            ("source_signature", "fixture:test"),
            ("embedding_model_id", "Xenova/all-MiniLM-L12-v2"),
            ("embedding_dimensions", "384"),
        ] {
            connection.execute(
                "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                [key, value],
            )?;
        }
        Ok(())
    }

    fn temp_db_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "atlas-index-{name}-{}-{}.sqlite",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = fs::remove_file(&path);
        path
    }
}
