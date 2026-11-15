//! Workspace-owned settings shared by search and reference publication.
use crate::IndexError;
use atlas_domain::SearchProfileId;
use rusqlite::Connection;

pub const WORKSPACE_SETTINGS_SCHEMA_VERSION: u32 = 43;

/// Exact configuration change attached to a reference publication.
pub struct WorkspaceModelChange {
    pub previous_configuration: crate::RequestedReferenceConfig,
    pub previous_preference: Option<SearchProfileId>,
    pub configuration: crate::RequestedReferenceConfig,
    pub preference: Option<SearchProfileId>,
}

impl WorkspaceModelChange {
    pub(crate) fn check(&self, connection: &Connection) -> Result<(), IndexError> {
        if crate::requested_config::read_configuration(connection)?.as_ref()
            != Some(&self.previous_configuration)
            || preferred_search_profile(connection)? != self.previous_preference
        {
            return Err(IndexError::Operation(
                "Search model configuration changed during setup. Check the current settings and retry.".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn commit(&self, connection: &Connection) -> Result<(), IndexError> {
        crate::requested_config::write_configuration(connection, &self.configuration)?;
        set_preferred_search_profile(connection, self.preference.as_ref())
    }
}

pub fn preferred_search_profile(
    connection: &Connection,
) -> Result<Option<SearchProfileId>, IndexError> {
    let id: Option<String> = connection.query_row(
        "SELECT preferred_search_profile_id FROM workspace_settings WHERE id=1",
        [],
        |row| row.get(0),
    )?;
    Ok(id.map(SearchProfileId::new))
}

pub fn set_preferred_search_profile(
    connection: &Connection,
    id: Option<&SearchProfileId>,
) -> Result<(), IndexError> {
    if let Some(id) = id {
        atlas_embedding::search_profile(id)
            .map_err(|error| IndexError::Invalid(error.to_string()))?;
        let published: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM search_profiles WHERE profile_id=?)",
            [id.as_str()],
            |row| row.get(0),
        )?;
        if !published {
            return Err(IndexError::Invalid(format!(
                "search profile {id} is not published"
            )));
        }
    }
    connection.execute(
        "UPDATE workspace_settings SET preferred_search_profile_id=? WHERE id=1",
        [id.map(SearchProfileId::as_str)],
    )?;
    Ok(())
}

pub(crate) fn retain_published_preference(connection: &Connection) -> Result<(), IndexError> {
    connection.execute(
        "UPDATE workspace_settings SET preferred_search_profile_id=NULL
         WHERE preferred_search_profile_id IS NOT NULL
         AND NOT EXISTS(SELECT 1 FROM search_profiles WHERE profile_id=preferred_search_profile_id)",
        [],
    )?;
    Ok(())
}
