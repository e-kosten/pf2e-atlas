//! Reference maintenance admitted by the bundled desktop command.
use atlas_domain::SearchProfileId;
use atlas_foundry_model::generated::{REFERENCE_SOURCE_COMMIT, REFERENCE_SOURCE_VERSION};
use atlas_index::{RequestedReferenceConfig, SourceSearchProfileIdentity, WorkspaceModelChange};
use atlas_storage::{StorageError, UpdateOwner, Workspace};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

use crate::{ResolvedAtlasPaths, ResolvedPathMode, WORKSPACE_SCHEMA_VERSION, workspace};

/// The packaged debug smoke uses a disposable, offline managed-source fixture.
pub(crate) fn fixture(paths: &ResolvedAtlasPaths) -> bool {
    #[cfg(debug_assertions)]
    {
        let Some(home) = std::env::var_os("ATLAS_PILOT_HOME").map(PathBuf::from) else {
            return false;
        };
        if std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "reference")
            && std::env::var_os("HOME").as_deref() == Some(home.as_os_str())
            && home
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("atlas-native-smoke-"))
            && home.canonicalize().is_ok_and(|home| {
                std::env::temp_dir()
                    .canonicalize()
                    .is_ok_and(|temp| home.parent() == Some(&temp))
                    && paths.index_path.starts_with(&home)
                    && paths.managed_source_root.starts_with(&home)
            })
            && paths.managed_source_root.join("module.json").is_file()
        {
            return true;
        }
    }
    let _ = paths;
    false
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DesktopModelSelection {
    pub profiles: Vec<SearchProfileId>,
    pub preferred: Option<SearchProfileId>,
}

impl DesktopModelSelection {
    fn validate(&self) -> Result<(), String> {
        if self.profiles.windows(2).any(|pair| pair[0] >= pair[1])
            || self
                .profiles
                .iter()
                .any(|id| atlas_embedding::search_profile(id).is_err())
            || self
                .preferred
                .as_ref()
                .is_some_and(|id| !self.profiles.contains(id))
        {
            return Err("Select supported search models and an installed default.".into());
        }
        Ok(())
    }

    fn retry_offer(&self, offer: &str) -> String {
        let encoded = serde_json::to_vec(&(offer, self))
            .expect("search model selection contains only strings");
        format!("{:x}", Sha256::digest(encoded))
    }
}

pub struct DesktopReferenceStatus {
    pub workspace: PathBuf,
    pub generation: String,
    pub revision: Option<String>,
    pub target: &'static str,
    pub target_version: &'static str,
    pub managed: bool,
    pub offer: Option<String>,
    pub model_offer: Option<String>,
    pub can_change_models: bool,
    pub preferred: Option<SearchProfileId>,
    pub published_profiles: Vec<SourceSearchProfileIdentity>,
}

pub enum DesktopReferenceOutcome {
    Ready {
        generation: String,
        updated: bool,
        model_repaired: bool,
    },
    SeedUnavailable {
        retry_offer: String,
    },
    Busy,
}

#[derive(Debug)]
pub enum DesktopRefreshError {
    Busy,
    Other(String),
}

impl std::fmt::Display for DesktopRefreshError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Busy => {
                formatter.write_str("workspace is busy; retry after the current operation finishes")
            }
            Self::Other(message) => formatter.write_str(message),
        }
    }
}

impl From<String> for DesktopRefreshError {
    fn from(message: String) -> Self {
        Self::Other(message)
    }
}

impl From<StorageError> for DesktopRefreshError {
    fn from(error: StorageError) -> Self {
        if error.is_contention() {
            Self::Busy
        } else {
            Self::Other(error.to_string())
        }
    }
}

impl From<atlas_index::IndexError> for DesktopRefreshError {
    fn from(error: atlas_index::IndexError) -> Self {
        match error {
            atlas_index::IndexError::Storage(error) => error.into(),
            error => Self::Other(error.to_string()),
        }
    }
}

struct Snapshot {
    status: DesktopReferenceStatus,
    configuration: Option<RequestedReferenceConfig>,
    file_identity: Option<(u64, u64)>,
}

pub(crate) struct Refresh {
    pub workspace: Workspace,
    pub generation: String,
    pub configuration: RequestedReferenceConfig,
    file_identity: Option<(u64, u64)>,
    pub models: Option<WorkspaceModelChange>,
    model_offer: Option<String>,
}

pub(crate) fn inspect(paths: &ResolvedAtlasPaths) -> Result<DesktopReferenceStatus, String> {
    let workspace = Workspace::existing(&paths.index_path).map_err(|e| e.to_string())?;
    Ok(snapshot(paths, &workspace)?.status)
}

pub(crate) fn managed_pin(paths: &ResolvedAtlasPaths, config: &RequestedReferenceConfig) -> bool {
    config.source_root.parent() == paths.managed_source_root.parent()
        && config
            .source_root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(commit_id)
}

fn commit_id(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn snapshot(paths: &ResolvedAtlasPaths, workspace: &Workspace) -> Result<Snapshot, String> {
    let file_identity = workspace_file_identity(paths, workspace)?;
    workspace::register_extensions().map_err(|e| e.to_string())?;
    workspace::check_workspace_history(workspace).map_err(|e| e.to_string())?;
    let read = workspace
        .read(WORKSPACE_SCHEMA_VERSION)
        .map_err(|e| e.to_string())?;
    let config = atlas_index::read_requested_reference_config(&read).map_err(|e| e.to_string())?;
    let context = atlas_index::read_stored_reference_context(read.connection())
        .map_err(|e| e.to_string())?
        .ok_or("Reference data is unavailable.")?;
    let upgrade =
        atlas_index::classify_reference_upgrade(read.connection()).map_err(|e| e.to_string())?;
    let generation = format!(
        "{}:{}",
        read.workspace_key().map_err(|e| e.to_string())?,
        read.reference_revision().map_err(|e| e.to_string())?,
    );
    let revision = context
        .source_revision
        .clone()
        .filter(|revision| commit_id(revision));
    let managed = paths.mode == ResolvedPathMode::Global
        && !paths.source_root_explicit
        && config
            .as_ref()
            .is_some_and(|config| managed_pin(paths, config) && config.manifest_path.is_none());
    let offer = config
        .as_ref()
        .filter(|config| {
            managed
                && revision.is_some()
                && upgrade != atlas_index::ReferenceUpgrade::SourceReplacement
                && !crate::setup::can_resume_desktop_stage(paths, config, &context)
        })
        .map(|config| {
            serde_json::to_vec(&(
                workspace.path(),
                file_identity,
                &generation,
                config,
                REFERENCE_SOURCE_COMMIT,
            ))
            .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
        })
        .transpose()
        .map_err(|e| e.to_string())?;
    let preferred =
        atlas_index::preferred_search_profile(read.connection()).map_err(|e| e.to_string())?;
    let model_offer = (paths.mode == ResolvedPathMode::Global
        && upgrade == atlas_index::ReferenceUpgrade::Current
        && !config
            .as_ref()
            .is_some_and(|config| crate::setup::can_resume_desktop_stage(paths, config, &context)))
    .then(|| {
        serde_json::to_vec(&(
            workspace.path(),
            file_identity,
            &generation,
            &config,
            &preferred,
        ))
        .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
    })
    .transpose()
    .map_err(|e| e.to_string())?;
    let can_change_models = model_offer.is_some()
        && managed
        && revision.as_deref() == Some(REFERENCE_SOURCE_COMMIT)
        && crate::setup_freshness::has_current_reference_preparation(&context)
        && config.as_ref().is_some_and(|config| {
            config.source_root == paths.managed_source_root
                && config.indexing_locale == context.indexing_locale
        });
    if workspace_file_identity(paths, workspace)? != file_identity {
        return Err(workspace_changed());
    }
    Ok(Snapshot {
        status: DesktopReferenceStatus {
            workspace: workspace.path().to_owned(),
            generation,
            revision,
            target: REFERENCE_SOURCE_COMMIT,
            target_version: REFERENCE_SOURCE_VERSION,
            managed,
            offer,
            model_offer,
            can_change_models,
            preferred,
            published_profiles: context.search_profiles,
        },
        configuration: config,
        file_identity,
    })
}

impl Refresh {
    pub(crate) fn admit(
        paths: &ResolvedAtlasPaths,
        workspace: &Workspace,
        offer: &str,
    ) -> Result<Self, DesktopRefreshError> {
        let check = || -> Result<Snapshot, String> {
            let current = snapshot(paths, workspace)?;
            if current.status.offer.as_deref() != Some(offer) {
                return Err(
                    "The workspace changed. Check its reference status and try again.".into(),
                );
            }
            Ok(current)
        };
        check()?;
        let owner = workspace.update_owner()?;
        let admitted = check()?;
        let mut configuration = admitted
            .configuration
            .ok_or_else(|| "Reference configuration is unavailable.".to_owned())?;
        if configuration.source_root != paths.managed_source_root {
            configuration
                .source_root
                .clone_from(&paths.managed_source_root);
            let write = workspace.write(WORKSPACE_SCHEMA_VERSION)?;
            if workspace_file_identity(paths, workspace)? != admitted.file_identity {
                return Err(workspace_changed().into());
            }
            atlas_index::write_requested_reference_config(&write, &configuration)?;
            write.commit()?;
        }
        drop(owner);
        Ok(Self {
            workspace: workspace.clone(),
            generation: admitted.status.generation,
            configuration,
            file_identity: admitted.file_identity,
            models: None,
            model_offer: None,
        })
    }

    pub(crate) fn admit_models(
        paths: &ResolvedAtlasPaths,
        workspace: &Workspace,
        offer: &str,
        selection: &DesktopModelSelection,
        local: bool,
    ) -> Result<Self, DesktopRefreshError> {
        selection.validate()?;
        let _owner = workspace.update_owner()?;
        let admitted = snapshot(paths, workspace)?;
        let base = admitted
            .status
            .model_offer
            .filter(|base| {
                if local {
                    selection.retry_offer(base) == offer
                } else {
                    base == offer
                }
            })
            .ok_or_else(workspace_changed)?;
        if !admitted.status.can_change_models {
            return Err(
                "Update reference data before changing the installed search models."
                    .to_owned()
                    .into(),
            );
        }
        if installed_ids(&admitted.status.published_profiles) == selection.profiles {
            return Err("The selected search models are already installed."
                .to_owned()
                .into());
        }
        let previous = admitted.configuration.ok_or_else(workspace_changed)?;
        let mut configuration = previous.clone();
        configuration
            .search_profiles
            .clone_from(&selection.profiles);
        Ok(Self {
            workspace: workspace.clone(),
            generation: admitted.status.generation,
            configuration: configuration.clone(),
            file_identity: admitted.file_identity,
            models: Some(WorkspaceModelChange {
                previous_configuration: previous,
                previous_preference: admitted.status.preferred,
                configuration,
                preference: selection.preferred.clone(),
            }),
            model_offer: Some(base),
        })
    }

    pub(crate) fn publication_owner(
        &self,
        paths: &ResolvedAtlasPaths,
    ) -> Result<UpdateOwner, DesktopRefreshError> {
        self.check_location(paths)?;
        let owner = self.workspace.update_owner()?;
        let current = snapshot(paths, &self.workspace)?;
        let expected_config = self
            .models
            .as_ref()
            .map_or(&self.configuration, |models| &models.previous_configuration);
        if current.status.generation != self.generation
            || current.configuration.as_ref() != Some(expected_config)
            || current.file_identity != self.file_identity
            || self
                .model_offer
                .as_ref()
                .is_some_and(|offer| current.status.model_offer.as_ref() != Some(offer))
        {
            return Err(workspace_changed().into());
        }
        Ok(owner)
    }

    pub(crate) fn check_location(&self, paths: &ResolvedAtlasPaths) -> Result<(), String> {
        if workspace_file_identity(paths, &self.workspace)? != self.file_identity {
            return Err(workspace_changed());
        }
        Ok(())
    }

    pub(crate) fn retry_offer(&self, paths: &ResolvedAtlasPaths) -> Option<String> {
        let current = snapshot(paths, &self.workspace).ok()?;
        if let Some(models) = &self.models {
            let base = self.model_offer.as_ref()?;
            if current.status.model_offer.as_ref() != Some(base) {
                return None;
            }
            return Some(
                DesktopModelSelection {
                    profiles: models.configuration.search_profiles.clone(),
                    preferred: models.preference.clone(),
                }
                .retry_offer(base),
            );
        }
        if current.status.generation == self.generation
            && current.configuration.as_ref() == Some(&self.configuration)
            && current.file_identity == self.file_identity
        {
            return current.status.offer;
        }
        None
    }
}

fn installed_ids(profiles: &[SourceSearchProfileIdentity]) -> Vec<SearchProfileId> {
    let mut ids: Vec<_> = profiles
        .iter()
        .map(|profile| profile.profile_id.clone())
        .collect();
    ids.sort();
    ids
}

pub(crate) fn set_default(
    runtime: &crate::AtlasRuntime,
    paths: &ResolvedAtlasPaths,
    offer: &str,
    selection: &DesktopModelSelection,
) -> Result<(), DesktopRefreshError> {
    selection.validate()?;
    let workspace = Workspace::existing(&paths.index_path)?;
    let _setup = workspace.setup_guard()?;
    let _owner = workspace.update_owner()?;
    let admitted = snapshot(paths, &workspace)?;
    if admitted.status.model_offer.as_deref() != Some(offer)
        || installed_ids(&admitted.status.published_profiles) != selection.profiles
    {
        return Err(workspace_changed().into());
    }
    if let Some(preference) = &selection.preferred
        && !runtime
            .search_profile_readiness(&admitted.status.published_profiles)
            .iter()
            .any(|profile| profile.usable && &profile.id == preference)
    {
        return Err("The selected default needs a usable search model."
            .to_owned()
            .into());
    }
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION)?;
    let c = write.connection();
    let key: String = c
        .query_row(
            "SELECT workspace_key FROM workspace WHERE id=1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let revision = write.reference_revision().map_err(|e| e.to_string())?;
    let generation = format!("{key}:{revision}");
    if workspace_file_identity(paths, &workspace)? != admitted.file_identity
        || generation != admitted.status.generation
        || atlas_index::preferred_search_profile(c).map_err(|e| e.to_string())?
            != admitted.status.preferred
        || atlas_index::read_requested_reference_config_for_write(&write)
            .map_err(|e| e.to_string())?
            != admitted.configuration
    {
        return Err(workspace_changed().into());
    }
    atlas_index::set_preferred_search_profile(c, selection.preferred.as_ref())
        .map_err(|e| e.to_string())?;
    Ok(write.commit()?)
}

fn workspace_changed() -> String {
    "The workspace changed during setup. Check its reference status and try again.".into()
}

fn workspace_file_identity(
    paths: &ResolvedAtlasPaths,
    workspace: &Workspace,
) -> Result<Option<(u64, u64)>, String> {
    let current = Workspace::existing(&paths.index_path).map_err(|e| e.to_string())?;
    if current.path() != workspace.path() {
        return Err(workspace_changed());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::metadata(current.path()).map_err(|e| e.to_string())?;
        Ok(Some((metadata.dev(), metadata.ino())))
    }
    #[cfg(not(unix))]
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(revision: &str) -> (tempfile::TempDir, ResolvedAtlasPaths, Workspace) {
        let home = tempfile::tempdir().unwrap();
        let managed = home.path().join("pf2e").join(REFERENCE_SOURCE_COMMIT);
        let paths = ResolvedAtlasPaths {
            mode: ResolvedPathMode::Global,
            repo_root: None,
            source_root: managed.clone(),
            managed_source_root: managed.clone(),
            source_root_explicit: false,
            embedding_cache_root: home.path().join("models"),
            index_path: home.path().join("workspace.sqlite"),
        };
        let workspace = crate::initialize_workspace(&paths.index_path).unwrap();
        let mut input = atlas_search::test_support::input(
            vec![atlas_search::test_support::record(
                "actions",
                "Item",
                serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Recall Knowledge","type":"action"}),
            )],
            false,
        );
        input.context.source_revision = Some(revision.into());
        atlas_index::test_support::publish_input_in(&workspace, WORKSPACE_SCHEMA_VERSION, &input)
            .unwrap();
        write_config(&workspace, managed);
        (home, paths, workspace)
    }

    fn write_config(workspace: &Workspace, source_root: PathBuf) {
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        atlas_index::write_requested_reference_config(
            &write,
            &RequestedReferenceConfig {
                source_root,
                manifest_path: None,
                indexing_locale: "en".into(),
                search_profiles: Vec::new(),
            },
        )
        .unwrap();
        write.commit().unwrap();
    }

    #[test]
    fn applying_the_bundled_pin_preserves_keyword_only_intent() {
        let old = "a".repeat(40);
        let (_home, paths, workspace) = fixture(&old);
        write_config(&workspace, paths.managed_source_root.with_file_name(&old));
        let offer = inspect(&paths).unwrap();
        assert_eq!(offer.revision.as_deref(), Some(old.as_str()));
        let _setup = workspace.setup_guard().unwrap();
        let refresh = Refresh::admit(&paths, &workspace, offer.offer.as_deref().unwrap()).unwrap();
        assert_eq!(refresh.configuration.source_root, paths.managed_source_root);
        assert!(refresh.configuration.search_profiles.is_empty());
        assert_eq!(inspect(&paths).unwrap().generation, offer.generation);
        assert!(refresh.publication_owner(&paths).is_ok());
        assert_ne!(refresh.retry_offer(&paths), offer.offer);
        assert!(refresh.retry_offer(&paths).is_some());
        assert!(Refresh::admit(&paths, &workspace, offer.offer.as_deref().unwrap()).is_err());
    }

    #[test]
    fn default_only_requires_current_offer_and_reuses_setup_contention() {
        let (home, paths, workspace) = fixture(REFERENCE_SOURCE_COMMIT);
        let runtime = crate::AtlasRuntime::resolve(crate::AtlasRuntimeOptions {
            overrides: crate::AtlasPathOverrides {
                index_path: Some(workspace.path().into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
        let before = inspect(&paths).unwrap();
        let offer = before.model_offer.as_deref().unwrap();
        let automatic = DesktopModelSelection {
            profiles: vec![],
            preferred: None,
        };
        {
            let _setup = workspace.setup_guard().unwrap();
            assert!(matches!(
                set_default(&runtime, &paths, offer, &automatic),
                Err(DesktopRefreshError::Busy)
            ));
        }
        {
            let _pause = workspace.pause_saved_writes().unwrap();
            set_default(&runtime, &paths, offer, &automatic).unwrap();
        }
        assert_eq!(inspect(&paths).unwrap().generation, before.generation);
        write_config(&workspace, home.path().join("changed-source"));
        assert!(set_default(&runtime, &paths, offer, &automatic).is_err());
    }

    #[test]
    fn rejects_an_offer_after_requested_source_changes() {
        let (home, paths, workspace) = fixture(REFERENCE_SOURCE_COMMIT);
        let before = inspect(&paths).unwrap();
        let _setup = workspace.setup_guard().unwrap();
        let refresh = Refresh::admit(&paths, &workspace, before.offer.as_deref().unwrap()).unwrap();
        write_config(&workspace, home.path().join("custom"));
        let after = inspect(&paths).unwrap();
        assert_eq!(after.revision, before.revision);
        assert!(!after.managed);
        assert!(after.offer.is_none());
        assert!(refresh.publication_owner(&paths).is_err());
        assert!(refresh.retry_offer(&paths).is_none());
        assert!(Refresh::admit(&paths, &workspace, before.offer.as_deref().unwrap()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_retargeted_or_replaced_workspace() {
        use std::{fs, os::unix::fs::symlink};

        let (home, mut paths, workspace) = fixture(REFERENCE_SOURCE_COMMIT);
        let original = paths.index_path.clone();
        let other = home.path().join("other.sqlite");
        fs::copy(&original, &other).unwrap();
        paths.index_path = home.path().join("active.sqlite");
        symlink(&original, &paths.index_path).unwrap();
        let before = inspect(&paths).unwrap();
        let _setup = workspace.setup_guard().unwrap();
        let refresh = Refresh::admit(&paths, &workspace, before.offer.as_deref().unwrap()).unwrap();
        fs::remove_file(&paths.index_path).unwrap();
        symlink(&other, &paths.index_path).unwrap();
        assert!(refresh.publication_owner(&paths).is_err());
        fs::remove_file(&paths.index_path).unwrap();
        symlink(&original, &paths.index_path).unwrap();
        fs::rename(other, original).unwrap();
        assert!(refresh.publication_owner(&paths).is_err());
        assert!(Refresh::admit(&paths, &workspace, before.offer.as_deref().unwrap()).is_err());
    }

    #[test]
    fn desktop_refresh_reports_setup_and_publisher_contention() {
        let (_home, paths, workspace) = fixture(REFERENCE_SOURCE_COMMIT);
        let before = inspect(&paths).unwrap();
        let verify_busy = || {
            let mut outcome = None;
            let report = crate::setup::refresh_desktop_reference(
                &paths,
                Default::default(),
                before.offer.as_deref().unwrap(),
                None,
                |event| outcome = Some(event),
            );
            assert!(!report.ready);
            assert!(matches!(outcome, Some(DesktopReferenceOutcome::Busy)));
        };
        {
            let _setup = workspace.setup_guard().unwrap();
            verify_busy();
        }
        {
            let _publisher = workspace.update_owner().unwrap();
            verify_busy();
        }
    }
}
