#![deny(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingModelId};
use atlas_runtime::AtlasPathMode;
use atlas_search::SemanticSearchMode;
use clap::{Args, Parser, Subcommand, ValueEnum};

mod commands;
mod output;
mod progress;

#[derive(Debug, Parser)]
#[command(name = "atlas")]
#[command(about = "PF2e Atlas local search and index tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Check or prepare local Atlas data and model paths")]
    Setup(SetupOptions),
    #[command(about = "Build, validate, inspect, and analyze Atlas indexes")]
    Index(IndexArgs),
    #[command(about = "Run Atlas search commands")]
    Search(SearchArgs),
}

#[derive(Debug, Args)]
struct IndexArgs {
    #[command(subcommand)]
    command: IndexCommand,
}

#[derive(Debug, Args)]
struct SearchArgs {
    #[command(subcommand)]
    command: SearchCommand,
}

#[derive(Debug, Args)]
struct SetupOptions {
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long)]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long)]
    fetch_source: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum IndexCommand {
    #[command(about = "Analyze Foundry source ingest without writing SQLite")]
    Analyze(AnalyzeIndexOptions),
    #[command(about = "Build a Rust SQLite artifact from Foundry source files")]
    Build(BuildIndexOptions),
    #[command(about = "Inspect artifact table and field coverage")]
    Inspect(IndexPathOptions),
    #[command(about = "Open an index read-only and validate Rust artifact metadata")]
    Validate(IndexPathOptions),
    #[command(about = "Validate sqlite-vec availability and record_vector_index coherence")]
    ValidateVectors(IndexPathOptions),
}

#[derive(Debug, Subcommand)]
enum SearchCommand {
    #[command(about = "Run semantic vector search and print raw record-key hits")]
    Semantic(SemanticSearchOptions),
}

#[derive(Debug, Args)]
struct AnalyzeIndexOptions {
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    manifest: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct BuildIndexOptions {
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    manifest: Option<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL)]
    embedding_model: EmbeddingModelId,
    #[arg(long)]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long, default_value_t = 32)]
    embedding_batch_size: usize,
    #[arg(long)]
    no_reuse_embeddings: bool,
    #[arg(long)]
    no_embeddings: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct IndexPathOptions {
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct SemanticSearchOptions {
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long)]
    query: String,
    #[arg(long, default_value_t = 10)]
    limit: u32,
    #[arg(long)]
    filter_json: Option<String>,
    #[arg(long)]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL)]
    embedding_model: EmbeddingModelId,
    #[arg(long, value_enum, default_value_t = CliSemanticMode::WeightedChunks)]
    semantic_mode: CliSemanticMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliSemanticMode {
    ParentOnly,
    Chunks,
    WeightedChunks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliPathMode {
    Auto,
    Repo,
    User,
}

impl From<CliSemanticMode> for SemanticSearchMode {
    fn from(mode: CliSemanticMode) -> Self {
        match mode {
            CliSemanticMode::ParentOnly => Self::ParentOnly,
            CliSemanticMode::Chunks => Self::Chunks,
            CliSemanticMode::WeightedChunks => Self::WeightedChunks,
        }
    }
}

impl From<CliPathMode> for AtlasPathMode {
    fn from(mode: CliPathMode) -> Self {
        match mode {
            CliPathMode::Auto => Self::Auto,
            CliPathMode::Repo => Self::Repo,
            CliPathMode::User => Self::User,
        }
    }
}

fn main() -> ExitCode {
    progress::init_tracing();
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Setup(options) => commands::setup::run_setup(options),
        Command::Index(index) => match index.command {
            IndexCommand::Analyze(options) => commands::index::run_index_analyze(options),
            IndexCommand::Build(options) => commands::index::run_index_build(options),
            IndexCommand::Inspect(options) => commands::index::run_index_inspect(options),
            IndexCommand::Validate(options) => commands::index::run_index_validate(options),
            IndexCommand::ValidateVectors(options) => {
                commands::index::run_index_validate_vectors(options)
            }
        },
        Command::Search(search) => match search.command {
            SearchCommand::Semantic(options) => commands::search::run_search_semantic(options),
        },
    }
}
