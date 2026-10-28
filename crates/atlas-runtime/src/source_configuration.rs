//! Requested preparation state survives failed or incompatible reference builds.
use crate::{ResolvedAtlasPaths, RuntimeSetupOptions, WORKSPACE_SCHEMA_VERSION};
use atlas_domain::SearchProfileId;
use atlas_index::RequestedReferenceConfig;

pub(crate) fn resolve_configuration(
    paths: &ResolvedAtlasPaths,
    options: &RuntimeSetupOptions,
    manifest_override: Option<&std::path::Path>,
) -> Result<RequestedReferenceConfig, String> {
    let mut requested = std::collections::BTreeSet::new();
    for profile in &options.search_profiles {
        atlas_embedding::search_profile(profile).map_err(|e| e.to_string())?;
        if !requested.insert(profile) {
            return Err(format!("duplicate requested search profile: {profile}"));
        }
    }
    if options.locale.as_ref().is_some_and(|locale| {
        locale.is_empty() || locale.trim() != locale || locale.chars().any(char::is_control)
    }) {
        return Err("indexing locale must be nonempty and contain no surrounding whitespace or control characters".into());
    }
    crate::workspace::register_extensions().map_err(|e| e.to_string())?;
    let workspace = if options.check {
        if paths.index_path.exists() {
            Some(atlas_storage::Workspace::at(&paths.index_path).map_err(|e| e.to_string())?)
        } else {
            None
        }
    } else {
        let workspace =
            crate::initialize_workspace(&paths.index_path).map_err(|e| e.to_string())?;
        atlas_index::ensure_reference_projections(&workspace, WORKSPACE_SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
        Some(workspace)
    };
    let owner = if !options.check {
        Some(
            workspace
                .as_ref()
                .ok_or("workspace initialization did not complete")?
                .update_owner()
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    if let Some(owner) = &owner {
        atlas_index::recover_reference_publication(owner, WORKSPACE_SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
    }
    let existing = if let Some(workspace) = &workspace {
        let read = workspace
            .read(WORKSPACE_SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
        atlas_index::read_requested_reference_config(&read).map_err(|e| e.to_string())?
    } else {
        None
    };
    let mut config = existing.unwrap_or_else(|| RequestedReferenceConfig {
        source_root: paths.source_root.clone(),
        manifest_path: None,
        indexing_locale: "en".into(),
        search_profiles: vec![],
    });
    if paths.source_root_explicit {
        config.source_root = paths.source_root.clone();
    }
    if config.source_root.is_relative() {
        config.source_root = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(&config.source_root);
    }
    if let Some(locale) = &options.locale {
        config.indexing_locale.clone_from(locale);
    }
    if let Some(manifest) = manifest_override {
        config.manifest_path = Some(if manifest.is_absolute() {
            manifest.to_owned()
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(manifest)
        });
    }
    for profile in &options.search_profiles {
        if !config.search_profiles.contains(profile) {
            config.search_profiles.push(profile.clone());
        }
    }
    if config.search_profiles.is_empty() && options.target.requires_embeddings() {
        config.search_profiles.push(SearchProfileId::new(
            atlas_embedding::DEFAULT_SEARCH_PROFILE,
        ));
    }
    for profile in &config.search_profiles {
        atlas_embedding::search_profile(profile).map_err(|e| e.to_string())?;
    }
    config.search_profiles.sort();
    if !options.check {
        let write = workspace
            .as_ref()
            .ok_or("workspace initialization did not complete")?
            .write(WORKSPACE_SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
        atlas_index::write_requested_reference_config(&write, &config)
            .map_err(|e| e.to_string())?;
        write.commit().map_err(|e| e.to_string())?;
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_configuration(
        paths: &ResolvedAtlasPaths,
        options: &RuntimeSetupOptions,
    ) -> Result<RequestedReferenceConfig, String> {
        super::resolve_configuration(paths, options, None)
    }

    fn paths(root: &std::path::Path) -> ResolvedAtlasPaths {
        ResolvedAtlasPaths {
            mode: crate::ResolvedPathMode::Global,
            repo_root: None,
            source_root: root.join("source"),
            managed_source_root: root.join("managed-source"),
            source_root_explicit: false,
            embedding_cache_root: root.join("models"),
            index_path: root.join("workspace.sqlite"),
        }
    }

    #[test]
    fn requested_models_and_locale_survive_without_a_reference_build() {
        let directory = tempfile::tempdir().unwrap();
        let paths = paths(directory.path());
        let initial = resolve_configuration(
            &paths,
            &RuntimeSetupOptions {
                locale: Some("fr".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            initial.search_profiles,
            vec![SearchProfileId::new("bge-base")]
        );
        assert!(crate::workspace::open_workspace_index(&paths.index_path).is_err());
        let updated = resolve_configuration(
            &paths,
            &RuntimeSetupOptions {
                search_profiles: vec![SearchProfileId::new("embeddinggemma")],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(updated.indexing_locale, "fr");
        assert_eq!(
            updated.search_profiles,
            vec![
                SearchProfileId::new("bge-base"),
                SearchProfileId::new("embeddinggemma")
            ]
        );
        let lexical = resolve_configuration(
            &paths,
            &RuntimeSetupOptions {
                target: crate::SetupTarget::Records,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(lexical.search_profiles, updated.search_profiles);
    }

    #[test]
    fn stored_source_path_wins_over_defaults_but_explicit_override_wins() {
        let directory = tempfile::tempdir().unwrap();
        let mut paths = paths(directory.path());
        let initial = resolve_configuration(&paths, &RuntimeSetupOptions::default()).unwrap();
        paths.source_root = directory.path().join("another-default");
        let unchanged = resolve_configuration(&paths, &RuntimeSetupOptions::default()).unwrap();
        assert_eq!(unchanged.source_root, initial.source_root);
        paths.source_root_explicit = true;
        let updated = resolve_configuration(&paths, &RuntimeSetupOptions::default()).unwrap();
        assert_eq!(updated.source_root, paths.source_root);
    }

    #[test]
    fn check_does_not_create_workspace_and_incompatible_files_do_not_fall_back() {
        let directory = tempfile::tempdir().unwrap();
        let paths = paths(directory.path());
        resolve_configuration(
            &paths,
            &RuntimeSetupOptions {
                check: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!paths.index_path.exists());
        std::fs::write(&paths.index_path, b"invalid database").unwrap();
        assert!(resolve_configuration(&paths, &RuntimeSetupOptions::default()).is_err());
        assert_eq!(
            std::fs::read(&paths.index_path).unwrap(),
            b"invalid database"
        );
    }
}
