use std::path::PathBuf;

use atlas_domain::SearchProfileId;
use clap::{Args, Subcommand};

use atlas_cli_support::CliPathMode;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas setup\n  atlas setup --no-embeddings\n  atlas setup --check --offline --path-mode global\n  atlas setup clean --embeddings\n  atlas setup clean --all --yes"
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
        help = "Override the persistent SQLite workspace path used by setup"
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
        help = "Indexing locale; defaults to English for a new artifact and preserves an existing artifact locale"
    )]
    pub(crate) locale: Option<String>,
    #[arg(
        long,
        help = "Prepare records without requesting additional embedding models; existing models are preserved"
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
    #[arg(long="search-profile",value_parser=crate::cli::args::parse_search_profile,conflicts_with="no_embeddings",help="Add bge-base or embeddinggemma; repeat for both; omission preserves configured profiles (fresh default: bge-base)")]
    pub(crate) search_profiles: Vec<SearchProfileId>,
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
        help = "Remove source and embedding caches; preserve the workspace and saved data"
    )]
    pub(crate) all: bool,
    #[arg(
        long,
        help_heading = "Cleanup Behavior",
        help = "Skip confirmation when removing both caches; required non-interactively unless --check is used"
    )]
    pub(crate) yes: bool,
}
