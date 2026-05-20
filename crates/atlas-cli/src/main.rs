#![deny(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use atlas_domain::DetailLevel;
use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingModelId};
use atlas_runtime::AtlasPathMode;
use atlas_search::{FusionMethod, RetrievalMode};
use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};

const DETAIL_HELP: &str = "Record detail level: summary, preview, description, standard, or full; preview includes compact scan facts and a truncated description";

mod agent_skills;
mod commands;
mod output;
mod progress;
mod terminal;

#[derive(Debug, Parser)]
#[command(name = "atlas")]
#[command(version)]
#[command(about = "PF2e Atlas local search and index tooling")]
#[command(
    after_help = "Examples:\n  atlas setup\n  atlas setup --no-embeddings\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9\n  atlas record resolve \"Treat Wounds\" --pack-name actionspf2e"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = CliProgressMode::Auto,
        help_heading = "Output",
        help = "Control progress rendering: auto shows human progress on terminals, never suppresses routine progress, always forces it"
    )]
    progress: CliProgressMode,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Install, repair, or check local Atlas runtime data")]
    Setup(SetupArgs),
    #[command(about = "Build, validate, inspect, and analyze Atlas indexes")]
    Index(IndexArgs),
    #[command(about = "Fetch and resolve Atlas records")]
    Record(RecordArgs),
    #[command(about = "Retrieve local record reference graph context")]
    Graph(GraphArgs),
    #[command(about = "Run Atlas search commands")]
    Search(Box<SearchOptions>),
    #[command(about = "Discover filter fields and values")]
    Filters(FiltersArgs),
    #[command(about = "Install and inspect Atlas agent integrations")]
    Agent(commands::agent_skills::AgentArgs),
    #[command(about = "Generate shell completion scripts")]
    Completions(CompletionsArgs),
}

#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  atlas completions zsh\n  atlas completions bash")]
struct CompletionsArgs {
    #[arg(value_enum, help = "Shell to generate completions for")]
    shell: Shell,
}

#[derive(Debug, Args)]
struct IndexArgs {
    #[command(subcommand)]
    command: IndexCommand,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas setup\n  atlas setup --no-embeddings\n  atlas setup --check --offline --path-mode global\n  atlas setup clean --artifact\n  atlas setup clean --all --yes"
)]
struct SetupArgs {
    #[command(flatten)]
    paths: SetupPathOptions,
    #[command(flatten)]
    run: SetupRunOptions,
    #[command(subcommand)]
    command: Option<SetupCommand>,
}

#[derive(Debug, Subcommand)]
enum SetupCommand {
    #[command(about = "Remove local Atlas runtime data without uninstalling the CLI")]
    Clean(SetupCleanOptions),
}

#[derive(Debug, Args)]
struct SetupPathOptions {
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = CliPathMode::Global,
        help_heading = "Path Overrides",
        help = "Use global install paths by default, or force checkout-local developer paths with repo"
    )]
    path_mode: CliPathMode,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the PF2E source checkout path used by setup and setup clean --source-checkout"
    )]
    source: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the embedding model cache root used by setup and setup clean --embeddings"
    )]
    embedding_cache_path: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the SQLite artifact path used by setup and setup clean --artifact"
    )]
    index: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Output",
        help = "Emit the standard JSON envelope"
    )]
    json: bool,
}

#[derive(Debug, Args)]
struct SetupRunOptions {
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
}

#[derive(Debug, Args)]
struct SetupCleanOptions {
    #[arg(
        long,
        help_heading = "Cleanup Behavior",
        help = "Report cleanup targets without removing files"
    )]
    check: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove the SQLite artifact and companion WAL/SHM files"
    )]
    artifact: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove the embedding model cache root"
    )]
    embeddings: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove the PF2E source checkout"
    )]
    source_checkout: bool,
    #[arg(
        long,
        help_heading = "Cleanup Targets",
        help = "Remove source, embedding cache, and SQLite artifact files"
    )]
    all: bool,
    #[arg(
        long,
        help_heading = "Cleanup Behavior",
        help = "Confirm cleanup when every target is selected"
    )]
    yes: bool,
}

#[derive(Debug, Args)]
struct RecordArgs {
    #[command(subcommand)]
    command: RecordCommand,
}

#[derive(Debug, Args)]
struct GraphArgs {
    #[command(subcommand)]
    command: GraphCommand,
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
    #[command(about = "Run a fast artifact readiness check")]
    Check(CheckIndexOptions),
    #[command(about = "Inspect artifact table and field coverage")]
    Inspect(IndexPathOptions),
    #[command(
        about = "Run deep artifact validation diagnostics; embeddings are required by default"
    )]
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
enum GraphCommand {
    #[command(about = "Fetch one-hop reference graph context for a known record key")]
    Get(GraphGetOptions),
}

#[derive(Debug, Subcommand)]
enum FiltersCommand {
    #[command(about = "List filterable fields available in a filter space")]
    Fields(Box<FiltersFieldsOptions>),
    #[command(about = "List values, samples, or stats for one filter field")]
    Values(Box<FiltersValuesOptions>),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas filters fields\n  atlas filters fields --family spell\n  atlas filters fields --family creature --json"
)]
struct FiltersFieldsOptions {
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON used to scope field discovery"
    )]
    filter_json: Option<String>,
    #[command(flatten)]
    filter_options: DiscoveryFilterOptions,
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas filters values --field traits --family spell\n  atlas filters values --field metric --family creature --metric-query armor\n  atlas filters values --field metric --family creature --metric ac.value --json"
)]
struct FiltersValuesOptions {
    #[arg(
        long,
        help = "Filter field id to inspect, such as traits, rarity, level, or metric"
    )]
    field: String,
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON used to scope value discovery"
    )]
    filter_json: Option<String>,
    #[command(flatten)]
    filter_options: DiscoveryFilterOptions,
    #[arg(
        long,
        value_enum,
        help = "Sort discovered values by count, alphabetically, or canonical order"
    )]
    sort: Option<CliFilterValueSort>,
    #[arg(
        long,
        visible_alias = "limit",
        help = "Limit sampled text examples; enumerable values are not limited by default"
    )]
    sample_limit: Option<usize>,
    #[arg(long, help = "Show values for one resolved metric key")]
    metric: Option<String>,
    #[arg(
        long,
        help = "Show metric keys below this namespace prefix, such as speed."
    )]
    metric_prefix: Option<String>,
    #[arg(long, help = "Show metric keys with this exact display label")]
    metric_label: Option<String>,
    #[arg(
        long,
        help = "Filter metric discovery by case-insensitive query across metric keys, labels, short labels, and groups"
    )]
    metric_query: Option<String>,
    #[arg(
        long,
        help = "Limit metric discovery to this metric domain, such as actor or item"
    )]
    metric_domain: Option<String>,
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas index analyze\n  atlas index analyze --source vendor/pf2e --manifest scratch/ingest-manifest.json --json"
)]
struct AnalyzeIndexOptions {
    #[arg(long, help = "Override the PF2E source checkout path")]
    source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Write the ingest manifest report to this path")]
    manifest: Option<PathBuf>,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Advanced manual artifact build. Standard users should run `atlas setup` instead.\n\nExamples:\n  atlas index build --no-embeddings\n  atlas index build --source vendor/pf2e --output .cache/pf2e-index.sqlite --json"
)]
struct BuildIndexOptions {
    #[arg(long, help = "Override the PF2E source checkout path")]
    source: Option<PathBuf>,
    #[arg(long, help = "SQLite artifact path to write")]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Write the ingest manifest report to this path")]
    manifest: Option<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL, help = "Embedding model to use for semantic search rows")]
    embedding_model: EmbeddingModelId,
    #[arg(long, help = "Override the embedding model cache root")]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long, default_value_t = 32, help = "Embedding generation batch size")]
    embedding_batch_size: usize,
    #[arg(
        long,
        help = "Regenerate embeddings instead of reusing compatible cached rows"
    )]
    no_reuse_embeddings: bool,
    #[arg(
        long,
        help = "Build a record/resolve-ready artifact without semantic embeddings"
    )]
    no_embeddings: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas index validate\n  atlas index validate --no-embeddings\n  atlas index validate --embeddings-only"
)]
struct ValidateIndexOptions {
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "embeddings_only", help = "Validate only the base record artifact and skip sqlite-vec/vector readiness")]
    no_embeddings: bool,
    #[arg(long, action = ArgAction::SetTrue, help = "Run the focused embedding/vector readiness diagnostics")]
    embeddings_only: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas index check\n  atlas index check --no-embeddings\n  atlas index check --json"
)]
struct CheckIndexOptions {
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, action = ArgAction::SetTrue, help = "Check only the base record artifact and skip sqlite-vec/vector readiness")]
    no_embeddings: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  atlas index inspect\n  atlas index inspect --json")]
struct IndexPathOptions {
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9\n  atlas record get equipment-srd:s1vB3HdXjMigYAnY\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail standard --json"
)]
struct RecordGetOptions {
    #[arg(required = true, num_args = 1.., help = "Canonical record keys in pack:id form; this command does not resolve names")]
    keys: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "standard", help = DETAIL_HELP)]
    detail: DetailLevel,
    #[arg(long, help = "Include raw source JSON in JSON output")]
    include_raw: bool,
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record resolve \"Treat Wounds\" --pack-name actionspf2e\n  atlas record resolve \"Treat Wounds\" --alternatives 3 --json\n\nFilter discovery:\n  atlas filters fields\n  atlas filters values --field traits --family rule"
)]
struct RecordResolveOptions {
    #[arg(required = true, num_args = 1.., help = "Strict record names or verified aliases to resolve")]
    queries: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "standard", help = DETAIL_HELP)]
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
        default_value_t = 5,
        help = "Return up to this many alternatives when a strict query is ambiguous"
    )]
    alternatives: u8,
    #[arg(long, help = "Include raw source JSON in JSON output")]
    include_raw: bool,
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas graph get actionspf2e:1kGNdIIhuglAjIp9\n  atlas graph get actionspf2e:1kGNdIIhuglAjIp9 --backlinks 4 --json\n  atlas graph get actionspf2e:1kGNdIIhuglAjIp9 --outgoing 0 --backlinks 8 --json"
)]
struct GraphGetOptions {
    #[arg(help = "Canonical seed record key in pack:id form; this command does not resolve names")]
    key: String,
    #[arg(long, default_value_t = 8, value_parser = parse_graph_limit, help = "Maximum outgoing neighbor records to include, 0-50")]
    outgoing: usize,
    #[arg(long, default_value_t = 0, value_parser = parse_graph_limit, help = "Maximum backlink neighbor records to include, 0-50; 0 disables backlinks")]
    backlinks: usize,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    detail: DetailLevel,
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas search --family spell --rarity uncommon --json\n  atlas search \"low level healing spell\" --json\n  atlas search --family creature --metric 'ac.value>=25' --detail preview --limit 8\n  atlas search --family creature --metric 'hp.value:40' --print-filter --json\n  atlas search \"low level healing spell\" --retrieval fts --json\n\nFilter discovery:\n  atlas filters fields\n  atlas filters values --field traits --family spell\n  atlas filters values --field metric --family creature --metric-query armor\n\nAdvanced retrieval controls:\n  --retrieval selects fts, vector, or hybrid retrieval.\n  --fusion selects rrf or weighted-rrf. weighted-rrf is the default with equal lane weights."
)]
struct SearchOptions {
    #[arg(help = "Plain-text query for ranked retrieval; omit for filter-only listing")]
    query: Option<String>,
    #[arg(long, help = "Override the SQLite artifact path")]
    index: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = 20,
        help = "Maximum records to return; capped at 100"
    )]
    limit: u32,
    #[arg(long, default_value_t = 0, help = "Number of matching records to skip")]
    offset: u32,
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON; do not combine with convenience filter flags"
    )]
    filter_json: Option<String>,
    #[command(flatten)]
    filter_options: FilterOptions,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    detail: DetailLevel,
    #[arg(long, value_enum, default_value_t = CliSearchSort::Alphabetical, help = "Sort order for filter-only searches; text queries are ranked")]
    sort: CliSearchSort,
    #[arg(long, help = "Seed for --sort random; generated when omitted")]
    seed: Option<u64>,
    #[arg(long, help = "Include raw source JSON in JSON output")]
    include_raw: bool,
    #[arg(long, help = "Override the embedding model cache root")]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL, help = "Embedding model for vector or hybrid retrieval")]
    embedding_model: EmbeddingModelId,
    #[arg(long, value_enum, default_value_t = CliRetrievalMode::Hybrid, help = "Rank text queries with fts, vector, or hybrid retrieval")]
    retrieval: CliRetrievalMode,
    #[arg(long, value_enum, default_value_t = CliFusionMethod::WeightedRrf, help = "Fusion algorithm for hybrid retrieval")]
    fusion: CliFusionMethod,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "FTS lane weight for weighted-rrf hybrid retrieval"
    )]
    fts_weight: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Vector lane weight for weighted-rrf hybrid retrieval"
    )]
    vector_weight: f64,
    #[arg(long, default_value_t = 60.0, help = "Reciprocal-rank fusion constant")]
    rank_constant: f64,
    #[arg(
        long,
        default_value_t = 200,
        help = "FTS candidate window for ranked text search"
    )]
    fts_top_k: u32,
    #[arg(
        long,
        default_value_t = 200,
        help = "Vector candidate window for ranked text search"
    )]
    vector_top_k: u32,
    #[arg(
        long,
        help = "Exclude records whose indexed search text matches this plain-text query"
    )]
    exclude: Option<String>,
    #[arg(long, action = ArgAction::SetTrue, help = "Include query analysis, rank scores, and retrieval lane diagnostics")]
    explain: bool,
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Print the lowered canonical filter and exit before opening the runtime"
    )]
    print_filter: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
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
        help = "Filter by metric predicate such as ac.value>=18, hp.value:40, or save.fort.mod>=12; repeat to require all"
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
enum CliSearchSort {
    #[value(name = "alphabetical")]
    Alphabetical,
    #[value(name = "level_asc")]
    LevelAsc,
    #[value(name = "level_desc")]
    LevelDesc,
    #[value(name = "price_asc")]
    PriceAsc,
    #[value(name = "price_desc")]
    PriceDesc,
    #[value(name = "record_key")]
    RecordKey,
    #[value(name = "random")]
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliPathMode {
    Repo,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliProgressMode {
    Auto,
    Always,
    Never,
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
            CliPathMode::Repo => Self::Repo,
            CliPathMode::Global => Self::Global,
        }
    }
}

impl From<CliProgressMode> for progress::ProgressMode {
    fn from(mode: CliProgressMode) -> Self {
        match mode {
            CliProgressMode::Auto => Self::Auto,
            CliProgressMode::Always => Self::Always,
            CliProgressMode::Never => Self::Never,
        }
    }
}

fn main() -> ExitCode {
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
    progress::init_tracing(progress::ProgressOptions {
        mode: cli.progress.into(),
        json: cli.command.uses_json(),
        setup_timing: cli.command.uses_setup_timing(),
    });
    match run(cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

impl Command {
    fn uses_json(&self) -> bool {
        match self {
            Self::Setup(args) => args.paths.json,
            Self::Index(args) => match &args.command {
                IndexCommand::Analyze(options) => options.json,
                IndexCommand::Build(options) => options.json,
                IndexCommand::Check(options) => options.json,
                IndexCommand::Inspect(options) => options.json,
                IndexCommand::Validate(options) => options.json,
            },
            Self::Record(args) => match &args.command {
                RecordCommand::Get(options) => options.json,
                RecordCommand::Resolve(options) => options.json,
            },
            Self::Graph(args) => match &args.command {
                GraphCommand::Get(options) => options.json,
            },
            Self::Search(options) => options.json,
            Self::Filters(filters) => match &filters.command {
                FiltersCommand::Fields(options) => options.json,
                FiltersCommand::Values(options) => options.json,
            },
            Self::Agent(args) => args.uses_json(),
            Self::Completions(_) => false,
        }
    }

    fn uses_setup_timing(&self) -> bool {
        matches!(self, Self::Setup(_))
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Setup(args) => commands::setup::run_setup(args),
        Command::Index(index) => match index.command {
            IndexCommand::Analyze(options) => commands::index::run_index_analyze(options),
            IndexCommand::Build(options) => commands::index::run_index_build(options),
            IndexCommand::Check(options) => commands::index::run_index_check(options),
            IndexCommand::Inspect(options) => commands::index::run_index_inspect(options),
            IndexCommand::Validate(options) => commands::index::run_index_validate(options),
        },
        Command::Record(record) => match record.command {
            RecordCommand::Get(options) => commands::record::run_record_get(options),
            RecordCommand::Resolve(options) => commands::record::run_record_resolve(*options),
        },
        Command::Graph(graph) => match graph.command {
            GraphCommand::Get(options) => commands::graph::run_graph_get(options),
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
        Command::Completions(args) => {
            let mut command = Cli::command();
            generate(args.shell, &mut command, "atlas", &mut std::io::stdout());
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn parse_detail_level(value: &str) -> Result<DetailLevel, String> {
    value
        .parse::<DetailLevel>()
        .map_err(|error| error.to_string())
}

fn parse_graph_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|error| format!("invalid graph limit `{value}`: {error}"))?;
    if limit <= 50 {
        Ok(limit)
    } else {
        Err(format!("graph limit must be between 0 and 50, got {limit}"))
    }
}
