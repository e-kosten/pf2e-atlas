use std::path::PathBuf;

use atlas_cli_support::CliPathMode;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct IndexArgs {
    #[command(subcommand)]
    pub(crate) command: IndexCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum IndexCommand {
    #[command(about = "Inspect artifact table and field coverage")]
    Inspect(IndexPathOptions),
}

#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  atlas-dev index inspect\n  atlas-dev index inspect --json")]
pub(crate) struct IndexPathOptions {
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
