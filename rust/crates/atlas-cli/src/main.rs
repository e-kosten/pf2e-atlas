#![deny(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use atlas_domain::DetailLevel;
use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingModelId};
use atlas_runtime::AtlasPathMode;
use atlas_search::{FusionMethod, RetrievalMode};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

mod agent_skills;
mod commands;
mod output;
mod progress;
mod terminal;

#[derive(Debug, Parser)]
#[command(name = "atlas")]
#[command(about = "PF2e Atlas local search and index tooling")]
#[command(
    after_help = "Examples:\n  atlas setup\n  atlas setup --no-embeddings\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9\n  atlas record resolve \"Treat Wounds\" --pack-name actionspf2e"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Install, repair, or check local Atlas runtime data")]
    Setup(SetupOptions),
    #[command(about = "Build, validate, inspect, and analyze Atlas indexes")]
    Index(IndexArgs),
    #[command(about = "Fetch and resolve Atlas records")]
    Record(RecordArgs),
    #[command(about = "Run Atlas search commands")]
    Search(Box<SearchOptions>),
    #[command(about = "Discover filter fields and values")]
    Filters(FiltersArgs),
    #[command(about = "Install and inspect Atlas agent integrations")]
    Agent(commands::agent_skills::AgentArgs),
}

#[derive(Debug, Args)]
struct IndexArgs {
    #[command(subcommand)]
    command: IndexCommand,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas setup\n  atlas setup --no-embeddings\n  atlas setup --check --offline --path-mode user"
)]
struct SetupOptions {
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto, help = "Use repo-local paths inside a checkout, user cache paths outside, or force one mode")]
    path_mode: CliPathMode,
    #[arg(long, help = "Override the PF2E source checkout path")]
    source: Option<PathBuf>,
    #[arg(long, help = "Override the embedding model cache root")]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(
        long,
        help = "Prepare a record/resolve-ready artifact without semantic embeddings"
    )]
    no_embeddings: bool,
    #[arg(
        long,
        help = "Report readiness and planned actions without writing files or using the network"
    )]
    check: bool,
    #[arg(
        long,
        help = "Do not fetch source or prepare embedding model files from the network"
    )]
    offline: bool,
    #[arg(
        long,
        help = "Rebuild the artifact even if it already satisfies the selected setup target"
    )]
    force_rebuild: bool,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL, help = "Embedding model to use for full setup")]
    embedding_model: EmbeddingModelId,
    #[arg(
        long,
        default_value_t = 32,
        help = "Embedding generation batch size for full setup"
    )]
    embedding_batch_size: usize,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
struct RecordArgs {
    #[command(subcommand)]
    command: RecordCommand,
}

#[derive(Debug, Args)]
struct FiltersArgs {
    #[command(subcommand)]
    command: FiltersCommand,
}

#[derive(Debug, Subcommand)]
enum IndexCommand {
    #[command(about = "Analyze Foundry source ingest without writing SQLite")]
    Analyze(AnalyzeIndexOptions),
    #[command(about = "Manually build a Rust SQLite artifact from Foundry source files")]
    Build(BuildIndexOptions),
    #[command(about = "Inspect artifact table and field coverage")]
    Inspect(IndexPathOptions),
    #[command(about = "Validate an Atlas artifact; embeddings are required by default")]
    Validate(ValidateIndexOptions),
}

#[derive(Debug, Subcommand)]
enum RecordCommand {
    #[command(about = "Fetch one or more records by canonical record key")]
    Get(RecordGetOptions),
    #[command(about = "Resolve one or more strict record names or aliases")]
    Resolve(Box<RecordResolveOptions>),
}

#[derive(Debug, Subcommand)]
enum FiltersCommand {
    #[command(about = "List filterable fields available in a filter space")]
    Fields(Box<FiltersFieldsOptions>),
    #[command(about = "List values, samples, or stats for one filter field")]
    Values(Box<FiltersValuesOptions>),
}

#[derive(Debug, Args)]
struct FiltersFieldsOptions {
    #[arg(long)]
    filter_json: Option<String>,
    #[command(flatten)]
    filter_options: DiscoveryFilterOptions,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
}

#[derive(Debug, Args)]
struct FiltersValuesOptions {
    #[arg(long)]
    field: String,
    #[arg(long)]
    filter_json: Option<String>,
    #[command(flatten)]
    filter_options: DiscoveryFilterOptions,
    #[arg(long, value_enum)]
    sort: Option<CliFilterValueSort>,
    #[arg(long)]
    sample_limit: Option<usize>,
    #[arg(long)]
    metric: Option<String>,
    #[arg(long)]
    metric_prefix: Option<String>,
    #[arg(long)]
    metric_label: Option<String>,
    #[arg(long)]
    metric_domain: Option<String>,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
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
#[command(
    after_help = "Advanced manual artifact build. Standard users should run `atlas setup` instead.\n\nExamples:\n  atlas index build --no-embeddings\n  atlas index build --source ../vendor/pf2e --output ../.cache/pf2e-rust-index.sqlite --json"
)]
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
#[command(
    after_help = "Examples:\n  atlas index validate\n  atlas index validate --no-embeddings\n  atlas index validate --embeddings-only"
)]
struct ValidateIndexOptions {
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "embeddings_only", help = "Validate only the base record artifact and skip sqlite-vec/vector readiness")]
    no_embeddings: bool,
    #[arg(long, action = ArgAction::SetTrue, help = "Run the focused embedding/vector readiness diagnostics")]
    embeddings_only: bool,
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
#[command(
    after_help = "Examples:\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9\n  atlas record get equipment-srd:s1vB3HdXjMigYAnY\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail standard --json"
)]
struct RecordGetOptions {
    #[arg(required = true, num_args = 1.., help = "Canonical record keys in pack:id form; this command does not resolve names")]
    keys: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "standard", help = "Record detail level: summary, preview, description, standard, or full; preview is a truncated description")]
    detail: DetailLevel,
    #[arg(long, help = "Include raw source JSON with full detail output")]
    include_raw: bool,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record resolve \"Treat Wounds\" --pack-name actionspf2e\n  atlas record resolve \"Treat Wounds\" --alternatives 3 --json\n\nFilter discovery:\n  atlas filters fields\n  atlas filters values --field traits --family rule"
)]
struct RecordResolveOptions {
    #[arg(required = true, num_args = 1.., help = "Strict record names or verified aliases to resolve")]
    queries: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "standard", help = "Record detail level: summary, preview, description, standard, or full; preview is a truncated description")]
    detail: DetailLevel,
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON used to narrow strict resolution"
    )]
    filter_json: Option<String>,
    #[command(flatten)]
    filter_options: FilterOptions,
    #[arg(
        long,
        default_value_t = 0,
        help = "Return up to this many alternatives when a strict query is ambiguous"
    )]
    alternatives: u8,
    #[arg(long, help = "Include raw source JSON with full detail output")]
    include_raw: bool,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas search --family spell --rarity uncommon --json\n  atlas search \"low level healing spell\" --json\n  atlas search \"low level healing spell\" --retrieval fts --json\n\nFilter discovery:\n  atlas filters fields\n  atlas filters values --field traits --family spell\n\nAdvanced retrieval controls:\n  --retrieval selects fts, vector, or hybrid retrieval.\n  --fusion selects rrf or weighted-rrf. weighted-rrf is the default with equal lane weights."
)]
struct SearchOptions {
    #[arg()]
    query: Option<String>,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: u32,
    #[arg(long, default_value_t = 0)]
    offset: u32,
    #[arg(long)]
    filter_json: Option<String>,
    #[command(flatten)]
    filter_options: FilterOptions,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = "Record detail level: summary, preview, description, standard, or full; preview is a truncated description")]
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
    #[arg(long, value_enum, default_value_t = CliRetrievalMode::Hybrid)]
    retrieval: CliRetrievalMode,
    #[arg(long, value_enum, default_value_t = CliFusionMethod::WeightedRrf)]
    fusion: CliFusionMethod,
    #[arg(long, default_value_t = 1.0)]
    fts_weight: f64,
    #[arg(long, default_value_t = 1.0)]
    vector_weight: f64,
    #[arg(long, default_value_t = 60.0)]
    rank_constant: f64,
    #[arg(long, default_value_t = 200)]
    fts_top_k: u32,
    #[arg(long, default_value_t = 200)]
    vector_top_k: u32,
    #[arg(
        long,
        help = "Exclude records whose indexed search text matches this plain-text query"
    )]
    exclude: Option<String>,
    #[arg(long, action = ArgAction::SetTrue)]
    explain: bool,
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Print the lowered canonical filter and exit before opening the runtime"
    )]
    print_filter: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Default, Args)]
struct FilterOptions {
    #[arg(
        long = "family",
        help = "Filter to records in this family; repeat for any of several families"
    )]
    families: Vec<String>,
    #[arg(
        long = "pack-name",
        help = "Filter to this stable source pack name; repeat for any of several packs"
    )]
    pack_names: Vec<String>,
    #[arg(
        long = "pack-label",
        help = "Filter to this display pack label; repeat for any of several labels"
    )]
    pack_labels: Vec<String>,
    #[arg(
        long = "rarity",
        help = "Filter to this rarity; repeat for any of several rarities"
    )]
    rarities: Vec<String>,
    #[arg(
        long = "publication-title",
        help = "Filter to this exact publication title; repeat for any of several titles"
    )]
    publication_titles: Vec<String>,
    #[arg(
        long = "level",
        help = "Filter to an exact level or inclusive range such as 1..5"
    )]
    level: Option<String>,
    #[arg(long = "min-level", help = "Filter to records at or above this level")]
    min_level: Option<f64>,
    #[arg(long = "max-level", help = "Filter to records at or below this level")]
    max_level: Option<f64>,
    #[arg(
        long = "price",
        help = "Filter to an exact price in copper pieces or inclusive range such as 100..500"
    )]
    price: Option<String>,
    #[arg(
        long = "min-price",
        help = "Filter to records priced at or above this many copper pieces"
    )]
    min_price: Option<f64>,
    #[arg(
        long = "max-price",
        help = "Filter to records priced at or below this many copper pieces"
    )]
    max_price: Option<f64>,
    #[arg(
        long = "trait",
        help = "Require this trait; repeat to require all listed traits"
    )]
    traits: Vec<String>,
    #[arg(
        long = "any-trait",
        help = "Require at least one of these traits; repeat for alternatives"
    )]
    any_traits: Vec<String>,
    #[arg(
        long = "references",
        help = "Filter to records that reference this canonical record key; repeat to require all targets"
    )]
    references: Vec<String>,
    #[arg(
        long = "referenced-by",
        help = "Filter to records referenced by this canonical record key; repeat to require all sources"
    )]
    referenced_by: Vec<String>,
    #[arg(
        long = "metric",
        help = "Filter by metric predicate such as defense.ac>=18, hp.value:40, or name=text; repeat to require all"
    )]
    metrics: Vec<String>,
}

#[derive(Debug, Clone, Default, Args)]
struct DiscoveryFilterOptions {
    #[arg(
        long = "family",
        help = "Filter to records in this family; repeat for any of several families"
    )]
    families: Vec<String>,
    #[arg(
        long = "pack-name",
        help = "Filter to this stable source pack name; repeat for any of several packs"
    )]
    pack_names: Vec<String>,
    #[arg(
        long = "pack-label",
        help = "Filter to this display pack label; repeat for any of several labels"
    )]
    pack_labels: Vec<String>,
    #[arg(
        long = "rarity",
        help = "Filter to this rarity; repeat for any of several rarities"
    )]
    rarities: Vec<String>,
    #[arg(
        long = "publication-title",
        help = "Filter to this exact publication title; repeat for any of several titles"
    )]
    publication_titles: Vec<String>,
    #[arg(
        long = "level",
        help = "Filter to an exact level or inclusive range such as 1..5"
    )]
    level: Option<String>,
    #[arg(long = "min-level", help = "Filter to records at or above this level")]
    min_level: Option<f64>,
    #[arg(long = "max-level", help = "Filter to records at or below this level")]
    max_level: Option<f64>,
    #[arg(
        long = "price",
        help = "Filter to an exact price in copper pieces or inclusive range such as 100..500"
    )]
    price: Option<String>,
    #[arg(
        long = "min-price",
        help = "Filter to records priced at or above this many copper pieces"
    )]
    min_price: Option<f64>,
    #[arg(
        long = "max-price",
        help = "Filter to records priced at or below this many copper pieces"
    )]
    max_price: Option<f64>,
    #[arg(
        long = "trait",
        help = "Require this trait; repeat to require all listed traits"
    )]
    traits: Vec<String>,
    #[arg(
        long = "any-trait",
        help = "Require at least one of these traits; repeat for alternatives"
    )]
    any_traits: Vec<String>,
    #[arg(
        long = "references",
        help = "Filter to records that reference this canonical record key; repeat to require all targets"
    )]
    references: Vec<String>,
    #[arg(
        long = "referenced-by",
        help = "Filter to records referenced by this canonical record key; repeat to require all sources"
    )]
    referenced_by: Vec<String>,
}

impl From<DiscoveryFilterOptions> for FilterOptions {
    fn from(options: DiscoveryFilterOptions) -> Self {
        Self {
            families: options.families,
            pack_names: options.pack_names,
            pack_labels: options.pack_labels,
            rarities: options.rarities,
            publication_titles: options.publication_titles,
            level: options.level,
            min_level: options.min_level,
            max_level: options.max_level,
            price: options.price,
            min_price: options.min_price,
            max_price: options.max_price,
            traits: options.traits,
            any_traits: options.any_traits,
            references: options.references,
            referenced_by: options.referenced_by,
            metrics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliRetrievalMode {
    Fts,
    Vector,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliFusionMethod {
    Rrf,
    WeightedRrf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliFilterValueSort {
    Count,
    Alpha,
    Canonical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliPathMode {
    Auto,
    Repo,
    User,
}

impl From<CliRetrievalMode> for RetrievalMode {
    fn from(mode: CliRetrievalMode) -> Self {
        match mode {
            CliRetrievalMode::Fts => Self::Fts,
            CliRetrievalMode::Vector => Self::Vector,
            CliRetrievalMode::Hybrid => Self::Hybrid,
        }
    }
}

impl From<CliFusionMethod> for FusionMethod {
    fn from(method: CliFusionMethod) -> Self {
        match method {
            CliFusionMethod::Rrf => Self::Rrf,
            CliFusionMethod::WeightedRrf => Self::WeightedRrf,
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
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if std::env::args().any(|arg| arg == "--json") {
                let _ = output::write_json_error("invalid_input", error.to_string());
                return ExitCode::from(2);
            }
            let _ = error.print();
            return ExitCode::from(error.exit_code() as u8);
        }
    };
    match run(cli) {
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
        },
        Command::Record(record) => match record.command {
            RecordCommand::Get(options) => commands::record::run_record_get(options),
            RecordCommand::Resolve(options) => commands::record::run_record_resolve(*options),
        },
        Command::Search(options) => commands::search::run_search(*options),
        Command::Filters(filters) => match filters.command {
            FiltersCommand::Fields(options) => {
                commands::filter_discovery::run_filters_fields(*options)
            }
            FiltersCommand::Values(options) => {
                commands::filter_discovery::run_filters_values(*options)
            }
        },
        Command::Agent(agent) => commands::agent_skills::run_agent(agent),
    }
}

fn parse_detail_level(value: &str) -> Result<DetailLevel, String> {
    value
        .parse::<DetailLevel>()
        .map_err(|error| error.to_string())
}
