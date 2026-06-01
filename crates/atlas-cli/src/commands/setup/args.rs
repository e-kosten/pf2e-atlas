use std::path::PathBuf;

use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingModelId};
use clap::{Args, Subcommand};

use crate::cli::args::CliPathMode;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas setup\n  atlas setup --no-embeddings\n  atlas setup --check --offline --path-mode global\n  atlas setup clean --artifact\n  atlas setup clean --all --yes"
)]
pub(crate) struct SetupArgs {
    #[command(flatten)]
    pub(crate) paths: SetupPathOptions,
    #[command(flatten)]
    pub(crate) run: SetupRunOptions,
    #[command(subcommand)]
    pub(crate) command: Option<SetupCommand>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SetupCommand {
    #[command(about = "Remove local Atlas runtime data without uninstalling the CLI")]
    Clean(SetupCleanOptions),
}

#[derive(Debug, Args)]
pub(crate) struct SetupPathOptions {
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = CliPathMode::Global,
        help_heading = "Path Overrides",
        help = "Use global install paths by default, or force checkout-local developer paths with repo"
    )]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the PF2E source checkout path used by setup and setup clean --source-checkout"
    )]
    pub(crate) source: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the embedding model cache root used by setup and setup clean --embeddings"
    )]
    pub(crate) embedding_cache_path: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the SQLite artifact path used by setup and setup clean --artifact"
    )]
    pub(crate) index: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Output",
        help = "Emit the standard JSON envelope"
    )]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct SetupRunOptions {
    #[arg(
        long,
        help = "Prepare a record/resolve-ready artifact without semantic embeddings"
    )]
    pub(crate) no_embeddings: bool,
    #[arg(
        long,
        help = "Report readiness and planned actions without writing files or using the network"
    )]
    pub(crate) check: bool,
    #[arg(
        long,
        help = "Do not fetch source or prepare embedding model files from the network"
    )]
    pub(crate) offline: bool,
    #[arg(
        long = "force",
        help = "Rebuild the artifact even if it already satisfies the selected setup target"
    )]
    pub(crate) force_rebuild: bool,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL, help = "Embedding model to use for full setup")]
    pub(crate) embedding_model: EmbeddingModelId,
    #[arg(
        long,
        default_value_t = 32,
        help = "Embedding generation batch size for full setup"
    )]
    pub(crate) embedding_batch_size: usize,
}

#[derive(Debug, Args)]
pub(crate) struct SetupCleanOptions {
    #[arg(
        long,
        help_heading = "Cleanup Behavior",
        help = "Report cleanup targets without removing files"
    )]
    pub(crate) check: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove the SQLite artifact and companion WAL/SHM files"
    )]
    pub(crate) artifact: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove the embedding model cache root"
    )]
    pub(crate) embeddings: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove the PF2E source checkout"
    )]
    pub(crate) source_checkout: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove source, embedding cache, and SQLite artifact files"
    )]
    pub(crate) all: bool,
    #[arg(
        long,
        help_heading = "Cleanup Behavior",
        help = "Confirm cleanup when every target is selected"
    )]
    pub(crate) yes: bool,
}
