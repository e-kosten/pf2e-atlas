use atlas_cli_support::CliPathMode;
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, WorkspaceArchive};
use clap::{Args, Subcommand};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Args)]
pub(crate) struct WorkspaceArgs {
    #[arg(long, global = true, help = "Override the SQLite workspace")]
    index: Option<PathBuf>,
    #[arg(long, global = true, value_enum, default_value_t = CliPathMode::Global)]
    path_mode: CliPathMode,
    #[command(subcommand)]
    command: WorkspaceCommand,
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommand {
    /// Export saved state and workspace settings to a new archive file.
    Export { file: PathBuf },
    /// Restore an archive into an initialized workspace with empty saved state.
    Restore {
        file: PathBuf,
        #[arg(
            long,
            help = "Prompt before replacing conflicting requested reference configuration"
        )]
        replace_reference_config: bool,
    },
}

pub(crate) fn run(args: WorkspaceArgs) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: crate::cli::runtime_path_mode(args.path_mode),
        overrides: AtlasPathOverrides {
            index_path: args.index,
            ..Default::default()
        },
        default_search_profile: None,
    })
    .map_err(|error| error.to_string())?;
    match args.command {
        WorkspaceCommand::Export { file } => {
            let archive = runtime.capture_workspace_archive()?;
            publish(&file, &archive)?;
            println!("Exported workspace archive to {}", file.display());
        }
        WorkspaceCommand::Restore {
            file,
            replace_reference_config,
        } => {
            let input = File::open(&file)
                .map_err(|error| format!("Cannot open {}: {error}", file.display()))?;
            let archive = WorkspaceArchive::read(BufReader::new(input))?;
            let preflight = runtime.preflight_workspace_restore()?;
            let mut confirmed = false;
            if replace_reference_config && preflight.conflicts_with(&archive) {
                if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
                    return Err("Replacing requested reference configuration requires interactive confirmation.".into());
                }
                eprintln!("Workspace: {}", runtime.index_path().display());
                eprintln!(
                    "Current requested configuration: {:?}",
                    preflight.current_configuration()
                );
                if let Some(config) = archive.requested_configuration() {
                    eprintln!("Archived requested configuration: {config:?}");
                } else {
                    eprintln!(
                        "The archive has no requested configuration. Restoring it will clear the current settings."
                    );
                }
                eprint!("Replace the requested reference configuration? [y/N] ");
                io::stderr().flush().map_err(|error| error.to_string())?;
                let mut response = String::new();
                io::stdin()
                    .read_line(&mut response)
                    .map_err(|error| error.to_string())?;
                confirmed = matches!(response.trim().to_ascii_lowercase().as_str(), "y" | "yes");
                if !confirmed {
                    return Err("Restore cancelled.".into());
                }
            }
            runtime.restore_workspace_archive(&archive, &preflight, confirmed)?;
            println!("Restored workspace from {}", file.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn publish(path: &Path, archive: &WorkspaceArchive) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| format!("Cannot stage {}: {error}", path.display()))?;
    {
        let mut output = BufWriter::new(staged.as_file_mut());
        archive.write(&mut output)?;
        output.flush().map_err(|error| error.to_string())?;
    }
    staged
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    staged
        .persist_noclobber(path)
        .map_err(|error| format!("Cannot publish {}: {}", path.display(), error.error))?;
    #[cfg(unix)]
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "Archive {} was published but directory sync failed: {error}",
                path.display()
            )
        })?;
    Ok(())
}
