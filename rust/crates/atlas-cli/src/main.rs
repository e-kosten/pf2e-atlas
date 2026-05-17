#![deny(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use atlas_domain::DetailLevel;
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
    #[command(about = "Fetch and resolve Atlas records")]
    Record(RecordArgs),
    #[command(about = "Run Atlas search commands")]
    Search(SearchOptions),
}

#[derive(Debug, Args)]
struct IndexArgs {
    #[command(subcommand)]
    command: IndexCommand,
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

#[derive(Debug, Args)]
struct RecordArgs {
    #[command(subcommand)]
    command: RecordCommand,
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
enum RecordCommand {
    #[command(about = "Fetch one or more records by canonical record key")]
    Get(RecordGetOptions),
    #[command(about = "Resolve one or more strict record names or aliases")]
    Resolve(RecordResolveOptions),
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
struct RecordGetOptions {
    #[arg(required = true, num_args = 1..)]
    keys: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary")]
    detail: DetailLevel,
    #[arg(long)]
    include_raw: bool,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct RecordResolveOptions {
    #[arg(required = true, num_args = 1..)]
    queries: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary")]
    detail: DetailLevel,
    #[arg(long)]
    filter_json: Option<String>,
    #[arg(long, default_value_t = 0)]
    alternatives: u8,
    #[arg(long)]
    include_raw: bool,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct SearchOptions {
    #[arg()]
    query: Option<String>,
    #[arg(long = "query", alias = "query-text", value_name = "QUERY")]
    query_text: Option<String>,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: u32,
    #[arg(long, default_value_t = 0)]
    offset: u32,
    #[arg(long)]
    filter_json: Option<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary")]
    detail: DetailLevel,
    #[arg(long, default_value = "alphabetical")]
    sort: String,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long)]
    include_raw: bool,
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
        Command::Record(record) => match record.command {
            RecordCommand::Get(options) => commands::record::run_record_get(options),
            RecordCommand::Resolve(options) => commands::record::run_record_resolve(options),
        },
        Command::Search(options) => commands::search::run_search(options),
    }
}

fn parse_detail_level(value: &str) -> Result<DetailLevel, String> {
    value
        .parse::<DetailLevel>()
        .map_err(|error| error.to_string())
}
