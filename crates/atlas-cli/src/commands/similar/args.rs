use std::path::PathBuf;

use atlas_domain::DetailLevel;
use clap::{ArgAction, Args};

use crate::cli::args::{CliPathMode, FilterOptions};
use crate::cli::parse::{DETAIL_HELP, parse_detail_level};

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas similar \"Dirge of Doom\"\n  atlas similar \"Shield Block\" --limit 12 --explain --json\n  atlas similar spells-srd:RDXXE7wMrSPCLv5k --family spell --max-level 5"
)]
pub(crate) struct SimilarOptions {
    #[arg(help = "Seed record key or strict resolvable record name")]
    pub(crate) record_ref: String,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, default_value_t = 20, value_parser = parse_similar_limit, help = "Maximum similar records to return, 1-100")]
    pub(crate) limit: u32,
    #[arg(long, default_value_t = 100, value_parser = parse_similar_limit, help = "Vector candidate records to inspect before graph reranking, 1-100")]
    pub(crate) candidates: u32,
    #[arg(
        long,
        default_value_t = 0.80,
        help = "Semantic vector-distance score weight for similar ranking"
    )]
    pub(crate) semantic_weight: f64,
    #[arg(
        long,
        default_value_t = 0.15,
        help = "Shared reference graph score weight for similar ranking"
    )]
    pub(crate) reference_weight: f64,
    #[arg(
        long,
        default_value_t = 0.05,
        help = "Shared trait score weight for similar ranking"
    )]
    pub(crate) trait_weight: f64,
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON; do not combine with convenience filter flags"
    )]
    pub(crate) filter_json: Option<String>,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(long, help = "Include raw source JSON in JSON output")]
    pub(crate) include_raw: bool,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, action = ArgAction::SetTrue, help = "Include similarity evidence in human output")]
    pub(crate) explain: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

fn parse_similar_limit(value: &str) -> Result<u32, String> {
    let limit = value
        .parse::<u32>()
        .map_err(|error| format!("invalid similar limit `{value}`: {error}"))?;
    if (1..=100).contains(&limit) {
        Ok(limit)
    } else {
        Err(format!(
            "similar limit must be between 1 and 100, got {limit}"
        ))
    }
}
