use std::{
    path::PathBuf,
    process::{Command, ExitCode},
};

use atlas_app_model::{AppErrorCode, AppReadinessStatus};
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_desktop_protocol::{
    BUNDLE_ID, ReferenceStatus, VERSION, WorkspaceProbe, WorkspaceStatus, parse_record_link,
};
use atlas_runtime::{
    AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, RuntimeErrorKind, Workspace,
};
use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct OpenArgs {
    /// An optional pf2e-atlas v1 reference link.
    link: Option<String>,
}

pub(crate) fn open(args: OpenArgs) -> Result<ExitCode, String> {
    let mut opener = Command::new("/usr/bin/open");
    opener.args(["-b", BUNDLE_ID]);
    if let Some(link) = args.link {
        parse_record_link(&link).map_err(str::to_owned)?;
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
}

pub(crate) fn probe(args: ProbeArgs) -> Result<ExitCode, String> {
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
