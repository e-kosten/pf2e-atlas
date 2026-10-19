use std::path::PathBuf;

use atlas_domain::SearchProfileId;
use clap::{ArgAction, Args, Subcommand};

use atlas_cli_support::CliPathMode;

#[derive(Debug, Args)]
pub(crate) struct IndexArgs {
    #[command(subcommand)]
    pub(crate) command: IndexCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum IndexCommand {
    #[command(about = "Update workspace reference data from Foundry source files")]
    Build(BuildIndexOptions),
    #[command(about = "Run a fast artifact readiness check")]
    Check(CheckIndexOptions),
    #[command(
        about = "Run deep artifact validation diagnostics; embeddings are required by default"
    )]
    Validate(ValidateIndexOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Advanced manual reference update. Standard users should run `atlas setup` instead. Saved lists and encounters are retained.\n\nExamples:\n  atlas index build --no-embeddings\n  atlas index build --source vendor/pf2e --output .atlas/workspace.sqlite --json"
)]
pub(crate) struct BuildIndexOptions {
    #[arg(long, help = "Override the PF2E source checkout path")]
    pub(crate) source: Option<PathBuf>,
    #[arg(long, help = "SQLite workspace path to initialize or update")]
    pub(crate) output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Override the Foundry source manifest path")]
    pub(crate) manifest: Option<PathBuf>,
    #[arg(
        long,
        help = "Indexing locale; omission preserves configured locale (fresh default: en)"
    )]
    pub(crate) locale: Option<String>,
    #[arg(long="search-profile",value_parser=crate::cli::args::parse_search_profile,conflicts_with="no_embeddings",help="Add a stored search profile; omission preserves configured models (fresh default: bge-base)")]
    pub(crate) search_profiles: Vec<SearchProfileId>,
    #[arg(long, help = "Override the embedding model cache root")]
    pub(crate) embedding_cache_path: Option<PathBuf>,
    #[arg(long, default_value_t = 32, help = "Embedding generation batch size")]
    pub(crate) embedding_batch_size: usize,
    #[arg(
        long,
        help = "Regenerate embeddings instead of reusing compatible cached rows"
    )]
    pub(crate) no_reuse_embeddings: bool,
    #[arg(
        long,
        help = "Build without requesting additional embedding models; existing models are preserved"
    )]
    pub(crate) no_embeddings: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  atlas index validate\n  atlas index validate --no-embeddings")]
pub(crate) struct ValidateIndexOptions {
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, action = ArgAction::SetTrue, help = "Validate the lexical artifact without requiring semantic readiness")]
    pub(crate) no_embeddings: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas index check\n  atlas index check --no-embeddings\n  atlas index check --json"
)]
pub(crate) struct CheckIndexOptions {
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, action = ArgAction::SetTrue, help = "Check only the base record artifact and skip sqlite-vec/vector readiness")]
    pub(crate) no_embeddings: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
