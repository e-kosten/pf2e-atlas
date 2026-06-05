use std::path::PathBuf;

use atlas_domain::DetailLevel;
use atlas_search::{DEFAULT_SEARCH_PAGE_SIZE, RetrievalMode, expert::FusionMethod};
use clap::{ArgAction, Args, ValueEnum};

use crate::cli::args::{CliPathMode, FilterOptions};
use crate::cli::parse::{DETAIL_HELP, parse_detail_level};

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas search --kind spell --rarity uncommon --json\n  atlas search \"low level healing spell\" --json\n  atlas search --kind creature --metric 'ac.value>=25' --detail preview --limit 8\n  atlas search --kind creature --metric 'hp.value:40' --print-filter --json\n  atlas search \"low level healing spell\" --retrieval fts --json\n  atlas search \"healing\" --page 2 --limit 10 --json\n\nFilter discovery:\n  atlas filters fields\n  atlas filters values --field traits --kind spell\n  atlas filters values --field metric --kind creature --metric-query armor\n\nAdvanced retrieval controls:\n  --retrieval selects fts, vector, or hybrid retrieval.\n  --fusion selects rrf or weighted-rrf. weighted-rrf is the default with equal lane weights."
)]
pub(crate) struct SearchOptions {
    #[arg(help = "Plain-text query for ranked retrieval; omit for filter-only listing")]
    pub(crate) query: Option<String>,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = DEFAULT_SEARCH_PAGE_SIZE,
        help = "Records per page; must be between 1 and 250"
    )]
    pub(crate) limit: u32,
    #[arg(long, default_value_t = 1, help = "1-based result page number")]
    pub(crate) page: u32,
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON; do not combine with convenience filter flags"
    )]
    pub(crate) filter_json: Option<String>,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(long, value_enum, default_value_t = CliSearchSort::Alphabetical, help = "Sort order for filter-only searches; text queries are ranked")]
    pub(crate) sort: CliSearchSort,
    #[arg(long, help = "Seed for --sort random; generated when omitted")]
    pub(crate) seed: Option<u64>,
    #[arg(long, help = "Include raw source JSON in JSON output")]
    pub(crate) include_raw: bool,
    #[arg(long, help = "Override the embedding model cache root")]
    pub(crate) embedding_cache_path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        value_enum,
        help = "Rank text queries with fts, vector, or hybrid retrieval"
    )]
    pub(crate) retrieval: Option<CliRetrievalMode>,
    #[arg(long, value_enum, help = "Fusion algorithm for hybrid retrieval")]
    pub(crate) fusion: Option<CliFusionMethod>,
    #[arg(long, help = "FTS lane weight for weighted-rrf hybrid retrieval")]
    pub(crate) fts_weight: Option<f64>,
    #[arg(long, help = "Vector lane weight for weighted-rrf hybrid retrieval")]
    pub(crate) vector_weight: Option<f64>,
    #[arg(long, help = "Reciprocal-rank fusion constant")]
    pub(crate) rank_constant: Option<f64>,
    #[arg(
        long,
        help = "FTS candidate window for ranked text search; capped at 5000"
    )]
    pub(crate) fts_top_k: Option<u32>,
    #[arg(
        long,
        help = "Vector candidate window for ranked text search; capped at 5000"
    )]
    pub(crate) vector_top_k: Option<u32>,
    #[arg(
        long,
        help = "Exclude records whose indexed search text matches this plain-text query"
    )]
    pub(crate) exclude: Option<String>,
    #[arg(long, action = ArgAction::SetTrue, help = "Include query analysis, rank scores, and retrieval lane diagnostics")]
    pub(crate) explain: bool,
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Print the lowered canonical filter and exit before opening the runtime"
    )]
    pub(crate) print_filter: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliRetrievalMode {
    Fts,
    Vector,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliFusionMethod {
    Rrf,
    WeightedRrf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliSearchSort {
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
