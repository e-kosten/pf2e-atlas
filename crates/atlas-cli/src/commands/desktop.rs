#[cfg(target_os = "macos")]
use std::process::Command;
use std::{
    io::{self, BufRead, Read, Write},
    path::PathBuf,
    process::ExitCode,
};

use atlas_app_model::{AppErrorCode, AppReadinessStatus};
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_desktop_protocol::workspace_selection::{Config, Role as TransitionWorkspace};
#[cfg(target_os = "macos")]
use atlas_desktop_protocol::{BUNDLE_ID, EntityLink};
use atlas_desktop_protocol::{ReferenceStatus, VERSION, WorkspaceProbe, WorkspaceStatus};
use atlas_runtime::{
    AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, RuntimeErrorKind, Workspace,
};
use clap::Args;

#[cfg(target_os = "macos")]
#[derive(Debug, Args)]
pub(crate) struct OpenArgs {
    /// An optional pf2e-atlas v1 entity link.
    link: Option<String>,
}

#[cfg(target_os = "macos")]
pub(crate) fn open(args: OpenArgs) -> Result<ExitCode, String> {
    let mut opener = Command::new("/usr/bin/open");
    opener.args(["-b", BUNDLE_ID]);
    if let Some(link) = args.link {
        EntityLink::parse(&link).map_err(str::to_owned)?;
        opener.arg("-u").arg(link);
    }
    if !opener
        .output()
        .map_err(|error| error.to_string())?
        .status
        .success()
    {
        return Err(
            "Could not open the PF2e Atlas app. Install it and open it from Finder first.".into(),
        );
    }
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug, Args)]
pub(crate) struct ProbeArgs {
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long)]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long)]
    wait_for_service: bool,
    #[arg(long, hide = true, conflicts_with = "wait_for_service")]
    reference_status: bool,
    #[arg(long, hide = true, conflicts_with_all = ["wait_for_service", "reference_status"])]
    update_compatibility: bool,
    #[arg(long, hide = true, value_parser = parse_managed_role)]
    transition_workspace: Option<TransitionWorkspace>,
    #[arg(long, hide = true, requires = "transition_workspace")]
    reserve_transition: bool,
    #[arg(long, hide = true, conflicts_with_all = ["transition_workspace", "update_compatibility", "reference_status", "wait_for_service", "index"])]
    workspace_selection: Option<SelectionAction>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum SelectionAction {
    Get,
    Save,
    RecoverOriginal,
}

fn parse_managed_role(value: &str) -> Result<TransitionWorkspace, &'static str> {
    match value {
        "original" => Ok(TransitionWorkspace::Original),
        "separate" => Ok(TransitionWorkspace::Separate),
        _ => Err("Expected original or separate workspace."),
    }
}

fn selection(action: SelectionAction) -> Result<ExitCode, String> {
    let config = match action {
        SelectionAction::Get => {
            use atlas_desktop_protocol::workspace_selection::{
                ManagedWorkspaceInfo, SelectionSnapshot,
            };
            let config = atlas_runtime::load_workspace_selection()?;
            let workspaces = [TransitionWorkspace::Original, TransitionWorkspace::Separate]
                .into_iter()
                .map(|role| {
                    let path = atlas_runtime::managed_workspace_path(role)
                        .map_err(|error| error.to_string())?;
                    let path = Workspace::at(&path)
                        .map_err(|error| error.to_string())?
                        .path()
                        .to_path_buf();
                    let exists = match std::fs::symlink_metadata(&path) {
                        Ok(_) => true,
                        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
                        Err(error) => return Err(error.to_string()),
                    };
                    Ok(ManagedWorkspaceInfo { role, path, exists })
                })
                .collect::<Result<Vec<_>, String>>()?;
            atlas_cli_support::write_json_data(SelectionSnapshot { config, workspaces })?;
            return Ok(ExitCode::SUCCESS);
        }
        SelectionAction::RecoverOriginal => atlas_runtime::recover_original_workspace_selection()?,
        SelectionAction::Save => {
            let mut line = String::new();
            io::stdin()
                .lock()
                .take(65_537)
                .read_line(&mut line)
                .map_err(|error| error.to_string())?;
            if line.len() > 65_536 || !line.ends_with('\n') {
                return Err("Invalid workspace selection.".into());
            }
            let mut config: Config =
                serde_json::from_str(&line).map_err(|error| error.to_string())?;
            atlas_runtime::save_workspace_selection(&mut config)?;
            config
        }
    };
    atlas_cli_support::write_json_data(config)?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(unix)]
fn file_identity(path: &std::path::Path) -> io::Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = path.metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn file_identity(path: &std::path::Path) -> io::Result<(u64, u64)> {
    let information = winapi_util::file::information(std::fs::File::open(path)?)?;
    Ok((information.volume_serial_number(), information.file_index()))
}

pub(crate) fn verify_expected_workspace(workspace: &Workspace) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct Expected {
        workspace: PathBuf,
        key: Option<String>,
        file_id: Option<(u64, u64)>,
    }
    let Ok(expected) = std::env::var("ATLAS_DESKTOP_EXPECTED_WORKSPACE") else {
        return Ok(());
    };
    let expected: Expected = serde_json::from_str(&expected).map_err(|error| error.to_string())?;
    let current = workspace.path();
    let changed = || {
        "The selected workspace changed after the switch. Choose which workspace to open."
            .to_owned()
    };
    if current != expected.workspace {
        return Err(changed());
    }
    match (expected.key, expected.file_id) {
        (Some(key), Some(identity)) => {
            let ledger = atlas_runtime::desktop_migration_ledger();
            let (actual_key, _) = workspace
                .check_update_history(&ledger)
                .map_err(|error| error.to_string())?;
            if actual_key != key
                || file_identity(current).map_err(|error| error.to_string())? != identity
            {
                return Err(changed());
            }
        }
        (None, None) => match std::fs::symlink_metadata(current) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            _ => return Err(changed()),
        },
        _ => return Err(changed()),
    }
    Ok(())
}

pub(crate) fn is_managed_index(index: &std::path::Path) -> bool {
    [TransitionWorkspace::Original, TransitionWorkspace::Separate]
        .iter()
        .filter_map(|role| atlas_runtime::managed_workspace_path(*role).ok())
        .any(|expected| {
            expected == index || expected.canonicalize().is_ok_and(|path| path == index)
        })
}

fn check_transition(args: &ProbeArgs, role: TransitionWorkspace) -> Result<ExitCode, String> {
    let selected = atlas_runtime::managed_workspace_path(role).map_err(|err| err.to_string())?;
    let other =
        atlas_runtime::managed_workspace_path(role.other()).map_err(|err| err.to_string())?;
    let path = &selected;
    let mut input = io::stdin().lock();
    let mut line = String::new();
    (&mut input)
        .take(65_537)
        .read_line(&mut line)
        .map_err(|err| err.to_string())?;
    if line.len() > 65_536 || !line.ends_with('\n') {
        return Err("Invalid update migration history.".into());
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SeedSource {
        workspace: PathBuf,
        key: String,
        schema: u32,
        file_id: (u64, u64),
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Initialization {
        workspace: PathBuf,
        key: String,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Request {
        ledger: atlas_runtime::MigrationLedger,
        seed: Option<SeedSource>,
        initialize: Option<Initialization>,
    }
    let Request {
        ledger,
        seed,
        initialize,
    } = serde_json::from_str(&line).map_err(|err| err.to_string())?;
    if let Some(initialization) = initialize {
        if seed.is_some() || args.reserve_transition {
            return Err("Invalid workspace initialization request.".into());
        }
        let destination = Workspace::at(path).map_err(|err| err.to_string())?;
        if destination.path() != initialization.workspace {
            return Err("The selected workspace moved before initialization.".into());
        }
        atlas_runtime::claim_desktop_workspace(destination.path(), &initialization.key)
            .map_err(|err| err.to_string())?;
    }
    if let Some(seed) = seed {
        if args.reserve_transition {
            return Err("Cannot create a copy while reserving a workspace.".into());
        }
        let verify_source = || {
            if other.canonicalize()? != seed.workspace || file_identity(&other)? != seed.file_id {
                return Err(atlas_runtime::StorageError::Migration(
                    "The source workspace changed. Review the switch again.".into(),
                ));
            }
            Ok(())
        };
        verify_source().map_err(|err| err.to_string())?;
        atlas_runtime::seed_desktop_copy(&other, path, &seed.key, seed.schema, verify_source)
            .map_err(|err| err.to_string())?;
    }
    let workspace = Workspace::at(path).map_err(|err| err.to_string())?;
    let mut setup = None;
    let mut update = None;
    let mut initialization = None;
    let state = match std::fs::symlink_metadata(path) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            if args.reserve_transition {
                initialization = Some(
                    workspace
                        .begin_initialization()
                        .map_err(|err| err.to_string())?
                        .ok_or("Workspace appeared before the switch. Try again.")?,
                );
            }
            serde_json::json!({"workspace": workspace.path(), "role": role})
        }
        Err(err) => return Err(err.to_string()),
        Ok(_) => {
            if args.reserve_transition {
                setup = Some(workspace.setup_guard().map_err(|err| err.to_string())?);
                update = Some(workspace.update_owner().map_err(|err| err.to_string())?);
            }
            let identity = file_identity(path).map_err(|err| err.to_string())?;
            match file_identity(&other) {
                Ok(other) if other == identity => {
                    return Err("The two workspaces point to the same file.".into());
                }
                Err(err) if err.kind() != io::ErrorKind::NotFound => {
                    return Err(err.to_string());
                }
                _ => {}
            }
            let (key, schema) = workspace
                .check_update_history(&ledger)
                .map_err(|err| err.to_string())?;
            if file_identity(path).map_err(|err| err.to_string())? != identity {
                return Err("The workspace file changed during the check.".into());
            }
            serde_json::json!({
                "workspace": workspace.path(), "role": role, "key": key,
                "schema": schema, "file_id": identity
            })
        }
    };
    atlas_cli_support::write_json_data(state)?;
    if args.reserve_transition {
        io::stdout().flush().map_err(|err| err.to_string())?;
        io::copy(&mut input, &mut io::sink()).map_err(|err| err.to_string())?;
    }
    drop((initialization, update, setup));
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn probe(args: ProbeArgs) -> Result<ExitCode, String> {
    if let Some(action) = args.workspace_selection {
        return selection(action);
    }
    if let Some(role) = args.transition_workspace {
        return check_transition(&args, role);
    }
    if args.update_compatibility {
        let (commit, version) = atlas_runtime::desktop_reference_baseline();
        atlas_cli_support::write_json_data(serde_json::json!({
            "workspaceSchema": atlas_runtime::WORKSPACE_SCHEMA_VERSION,
            "migrationLedger": atlas_runtime::desktop_migration_ledger(),
            "foundryCommit": commit,
            "foundryVersion": version,
        }))?;
        return Ok(ExitCode::SUCCESS);
    }
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            index_path: args.index.clone(),
            embedding_cache_root: args.embedding_cache_path.clone(),
            ..Default::default()
        },
        ..Default::default()
    })
    .map_err(|error| error.to_string())?;
    if args.reference_status {
        let status = runtime.desktop_reference_status()?;
        let readiness = runtime.search_profile_readiness(&status.published_profiles);
        let models = atlas_embedding::SEARCH_PROFILES
            .iter()
            .map(|profile| {
                let id = profile.profile_id();
                let ready = readiness.iter().find(|ready| ready.id == id);
                atlas_desktop_protocol::SearchModelStatus {
                    id,
                    label: profile.label.into(),
                    installed: ready.is_some(),
                    usable: ready.is_some_and(|ready| ready.usable),
                }
            })
            .collect();
        atlas_cli_support::write_json_data(ReferenceStatus {
            version: VERSION,
            workspace: status.workspace,
            generation: status.generation,
            revision: status.revision,
            target: status.target.into(),
            target_version: status.target_version.into(),
            managed: status.managed,
            offer: status.offer,
            model_offer: status.model_offer,
            can_change_models: status.can_change_models,
            preferred: status.preferred,
            models,
        })?;
        return Ok(ExitCode::SUCCESS);
    }
    let workspace = Workspace::existing(runtime.index_path())
        .map(|workspace| workspace.path().to_owned())
        .unwrap_or_else(|_| runtime.index_path().to_owned());
    let (mut status, mut message) = if let Err(error) = runtime.check_workspace_if_present() {
        let status = match error.kind() {
            RuntimeErrorKind::WorkspaceUpgradeRequired => WorkspaceStatus::UpgradeRequired,
            RuntimeErrorKind::WorkspaceIncompatible => WorkspaceStatus::Incompatible,
            RuntimeErrorKind::WorkspaceBusy => WorkspaceStatus::Busy,
            RuntimeErrorKind::WorkspaceMaintenance => WorkspaceStatus::RecoveryRequired,
            _ => WorkspaceStatus::Unavailable,
        };
        (status, error.to_string())
    } else {
        let service = AtlasAppService::start(AtlasAppServiceOptions {
            index_path: args.index,
            embedding_cache_root: args.embedding_cache_path,
            retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
            ..Default::default()
        })
        .map_err(|error| error.into_app_error().message)?;
        let readiness = service.readiness();
        let status = if let Some(error) = &readiness.workspace_error {
            match error.code {
                AppErrorCode::SetupRequired => WorkspaceStatus::NeedsSetup,
                AppErrorCode::ServiceBusy => WorkspaceStatus::Busy,
                AppErrorCode::WorkspaceMaintenanceRequired => WorkspaceStatus::RecoveryRequired,
                _ => WorkspaceStatus::Unavailable,
            }
        } else if readiness.status == AppReadinessStatus::Ready {
            if readiness
                .search_profiles
                .iter()
                .any(|profile| profile.usable)
            {
                WorkspaceStatus::Ready
            } else {
                WorkspaceStatus::Browsable
            }
        } else {
            WorkspaceStatus::NeedsSetup
        };
        (status, readiness.message)
    };
    if args.wait_for_service && status == WorkspaceStatus::UpgradeRequired {
        let workspace = Workspace::existing(&workspace).map_err(|error| error.to_string())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
        loop {
            match workspace.service_guard() {
                Ok(_) => break,
                Err(atlas_runtime::StorageError::Busy) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(atlas_runtime::StorageError::Busy) => {
                    status = WorkspaceStatus::Busy;
                    message = "Another Atlas session is still using this workspace. Close it and retry the upgrade.".into();
                    break;
                }
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    atlas_cli_support::write_json_data(WorkspaceProbe {
        version: VERSION,
        workspace,
        status,
        message,
        stage: runtime.desktop_setup_stage().and_then(|stage| match stage {
            atlas_runtime::DesktopSetupStage::FirstRun => {
                Some(atlas_desktop_protocol::SetupStage::FirstRun)
            }
            atlas_runtime::DesktopSetupStage::Resume => {
                Some(atlas_desktop_protocol::SetupStage::Resume)
            }
            atlas_runtime::DesktopSetupStage::Fixture => None,
        }),
    })?;
    Ok(ExitCode::SUCCESS)
}
