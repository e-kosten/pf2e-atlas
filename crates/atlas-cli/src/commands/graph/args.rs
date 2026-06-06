use std::path::PathBuf;

use atlas_domain::DetailLevel;
use atlas_search::{
    DEFAULT_GRAPH_BACKLINK_LIMIT, DEFAULT_GRAPH_OUTGOING_LIMIT, DEFAULT_GRAPH_USES_LIMIT,
    MAX_GRAPH_CONTEXT_LIMIT,
};
use clap::{Args, Subcommand};

use crate::cli::args::CliPathMode;
use crate::cli::parse::{DETAIL_HELP, parse_detail_level};

#[derive(Debug, Args)]
pub(crate) struct GraphArgs {
    #[command(subcommand)]
    pub(crate) command: GraphCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum GraphCommand {
    #[command(about = "Fetch one-hop reference links for a record")]
    Links(GraphLinksOptions),
    #[command(about = "Show records that reference or use a record")]
    Uses(GraphUsesOptions),
    #[command(about = "Show variant siblings or progression for a record")]
    Variants(GraphVariantsOptions),
    #[command(about = "Show legacy/remaster links for a record")]
    Remaster(GraphRemasterOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas graph links actionspf2e:1kGNdIIhuglAjIp9\n  atlas graph links \"Demoralize\" --backlinks 4 --json\n  atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --outgoing 0 --backlinks 8 --json"
)]
pub(crate) struct GraphLinksOptions {
    #[arg(help = "Seed record key or strict resolvable record name")]
    pub(crate) record_ref: String,
    #[arg(long, default_value_t = DEFAULT_GRAPH_OUTGOING_LIMIT, value_parser = parse_graph_limit, help = "Maximum outgoing neighbor records to include, 0-50")]
    pub(crate) outgoing: usize,
    #[arg(long, default_value_t = DEFAULT_GRAPH_BACKLINK_LIMIT, value_parser = parse_graph_limit, help = "Maximum backlink neighbor records to include, 0-50; 0 disables backlinks")]
    pub(crate) backlinks: usize,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas graph uses \"Frightened\"\n  atlas graph uses conditionitems:AJh5ex99aV6VTggg --limit 25 --json"
)]
pub(crate) struct GraphUsesOptions {
    #[arg(help = "Seed record key or strict resolvable record name")]
    pub(crate) record_ref: String,
    #[arg(long, default_value_t = DEFAULT_GRAPH_USES_LIMIT, value_parser = parse_graph_limit, help = "Maximum records that use this record to include, 0-50")]
    pub(crate) limit: usize,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas graph variants \"Dread Ampoule\"\n  atlas graph variants equipment-srd:IvFEJqp2MUew65nQ --json"
)]
pub(crate) struct GraphVariantsOptions {
    #[arg(help = "Seed record key, strict resolvable record name, or variant base name")]
    pub(crate) record_ref: String,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas graph remaster \"Air Mephit\"\n  atlas graph remaster pathfinder-bestiary:KDRlxdIUADWHI6Vr --json"
)]
pub(crate) struct GraphRemasterOptions {
    #[arg(help = "Seed record key or strict resolvable record name")]
    pub(crate) record_ref: String,
    #[arg(long, value_parser = parse_detail_level, default_value = "summary", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

fn parse_graph_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|error| format!("invalid graph limit `{value}`: {error}"))?;
    if limit <= MAX_GRAPH_CONTEXT_LIMIT {
        Ok(limit)
    } else {
        Err(format!(
            "graph limit must be between 0 and {MAX_GRAPH_CONTEXT_LIMIT}, got {limit}"
        ))
    }
}
