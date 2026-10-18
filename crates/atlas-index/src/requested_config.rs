//! Durable update intent, independent of the last successfully published reference.
use crate::IndexError;
use atlas_domain::SearchProfileId;
use atlas_storage::{ReadSession, WriteSession};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestedReferenceConfig {
    pub source_root: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub indexing_locale: String,
    pub search_profiles: Vec<SearchProfileId>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredConfig {
    version: u32,
    config: RequestedReferenceConfig,
}

pub fn read_requested_reference_config(
    session: &ReadSession,
) -> Result<Option<RequestedReferenceConfig>, IndexError> {
    let json: Option<String> = session
        .connection()
        .query_row(
            "SELECT config_json FROM requested_reference_config WHERE id=1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    json.map(|json| {
        let stored: StoredConfig = serde_json::from_str(&json)?;
        if stored.version != 1 {
            return Err(IndexError::Invalid(format!(
                "unsupported requested reference configuration version {}",
                stored.version
            )));
        }
        validate(&stored.config)?;
        Ok(stored.config)
    })
    .transpose()
}

/// The caller owns the transaction and commits intent before expensive preparation.
pub fn write_requested_reference_config(
    session: &WriteSession,
    config: &RequestedReferenceConfig,
) -> Result<(), IndexError> {
    validate(config)?;
    let json = serde_json::to_string(&StoredConfig {
        version: 1,
        config: config.clone(),
    })?;
    session.connection().execute(
        "INSERT INTO requested_reference_config(id,config_json) VALUES(1,?1)
         ON CONFLICT(id) DO UPDATE SET config_json=excluded.config_json",
        [json],
    )?;
    Ok(())
}

fn validate(config: &RequestedReferenceConfig) -> Result<(), IndexError> {
    let invalid = |message: &str| {
        IndexError::Invalid(format!("requested reference configuration: {message}"))
    };
    if config.source_root.as_os_str().is_empty()
        || config
            .manifest_path
            .as_ref()
            .is_some_and(|path| path.as_os_str().is_empty())
    {
        return Err(invalid("paths must not be empty"));
    }
    if config.indexing_locale.is_empty()
        || config.indexing_locale.trim() != config.indexing_locale
        || config.indexing_locale.chars().any(char::is_control)
    {
        return Err(invalid(
            "locale must be nonempty and have no surrounding whitespace or control characters",
        ));
    }
    let mut profiles = BTreeSet::new();
    for profile in &config.search_profiles {
        let value = profile.as_str();
        if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(invalid(
                "profile identifiers must be nonempty and contain no whitespace or control characters",
            ));
        }
        if !profiles.insert(profile) {
            return Err(invalid("profile identifiers must be unique"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_storage::{Migration, Workspace};

    fn config() -> RequestedReferenceConfig {
        RequestedReferenceConfig {
            source_root: PathBuf::from("/source/pf2e"),
            manifest_path: None,
            indexing_locale: "en".into(),
            search_profiles: vec![SearchProfileId::new("bge-base")],
        }
    }

    fn workspace() -> (tempfile::TempDir, Workspace) {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::at(directory.path().join("workspace.sqlite")).unwrap();
        workspace.migrate(&[Migration {
            version: 1, name: "requested config test",
            sql: "CREATE TABLE requested_reference_config(id INTEGER PRIMARY KEY CHECK(id=1), config_json TEXT NOT NULL CHECK(json_valid(config_json)));",
        }], |_| Ok(())).unwrap();
        (directory, workspace)
    }

    #[test]
    fn intent_survives_without_published_reference_and_requires_commit() {
        let (_directory, workspace) = workspace();
        assert_eq!(
            read_requested_reference_config(&workspace.read(1).unwrap()).unwrap(),
            None
        );
        let write = workspace.write(1).unwrap();
        write_requested_reference_config(&write, &config()).unwrap();
        drop(write);
        assert_eq!(
            read_requested_reference_config(&workspace.read(1).unwrap()).unwrap(),
            None
        );
        let write = workspace.write(1).unwrap();
        write_requested_reference_config(&write, &config()).unwrap();
        write.commit().unwrap();
        assert_eq!(
            read_requested_reference_config(&workspace.read(1).unwrap()).unwrap(),
            Some(config())
        );
    }

    #[test]
    fn invalid_replacement_preserves_prior_intent_even_if_caller_commits() {
        let (_directory, workspace) = workspace();
        let write = workspace.write(1).unwrap();
        write_requested_reference_config(&write, &config()).unwrap();
        let mut invalid = config();
        invalid
            .search_profiles
            .push(invalid.search_profiles[0].clone());
        assert!(write_requested_reference_config(&write, &invalid).is_err());
        write.commit().unwrap();
        assert_eq!(
            read_requested_reference_config(&workspace.read(1).unwrap()).unwrap(),
            Some(config())
        );
    }

    #[test]
    fn corrupted_or_unknown_storage_payload_is_not_treated_as_missing() {
        let (_directory, workspace) = workspace();
        for json in [
            r#"{"version":2,"config":{"source_root":"/source","manifest_path":null,"indexing_locale":"en","search_profiles":[]}}"#,
            r#"{"version":1,"config":{"source_root":"/source","manifest_path":null,"indexing_locale":"en","search_profiles":[""]}}"#,
            r#"{"version":1,"config":{"source_root":"/source","manifest_path":null,"indexing_locale":"en","search_profiles":[]},"extra":true}"#,
        ] {
            let write = workspace.write(1).unwrap();
            write
                .connection()
                .execute(
                    "INSERT OR REPLACE INTO requested_reference_config VALUES(1,?1)",
                    [json],
                )
                .unwrap();
            write.commit().unwrap();
            assert!(read_requested_reference_config(&workspace.read(1).unwrap()).is_err());
        }
    }
}
