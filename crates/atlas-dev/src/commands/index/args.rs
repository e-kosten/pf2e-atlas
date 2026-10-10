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
    #[command(
        about = "Inspect a checked source snapshot and provenance; optionally verify original JSON from a configured clone"
    )]
    Record(RecordInspectOptions),
}

#[derive(Debug, Args)]
pub(crate) struct RecordInspectOptions {
    pub(crate) key: String,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(
        long,
        help = "Explicitly read original JSON from the configured source clone after checking its path and hash"
    )]
    pub(crate) original: bool,
    #[arg(long, help = "Override the configured source clone path")]
    pub(crate) source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
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
