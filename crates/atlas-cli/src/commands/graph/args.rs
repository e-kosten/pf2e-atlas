use std::path::PathBuf;

use clap::{Args, Subcommand};

use atlas_cli_support::CliPathMode;

#[derive(Debug, Args)]
pub(crate) struct GraphArgs {
    #[command(subcommand)]
    pub(crate) command: GraphCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum GraphCommand {
    #[command(about = "Fetch one-hop reference links for a record")]
    Links(GraphLinksOptions),
    #[command(
        about = "Page exact link occurrences for a search result and positive link condition"
    )]
    Occurrences(Box<GraphOccurrencesOptions>),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Repeat the search's complete filter and artifact generation. Use --clause-id from its relationship summaries when it has multiple positive link conditions.\nExample:\n  atlas graph occurrences PACK:ID --links-to 'Leather Armor' --artifact-generation GENERATION --page 2 --json"
)]
pub(crate) struct GraphOccurrencesOptions {
    #[arg(help = "Canonical key of a returned search result")]
    pub(crate) result_record_key: String,
    #[command(flatten)]
    pub(crate) filter_options: crate::cli::args::FilterOptions,
    #[arg(
        long,
        help = "Positive link clause ID; optional when exactly one exists"
    )]
    pub(crate) clause_id: Option<String>,
    #[arg(long, help = "Exact artifact generation returned by search")]
    pub(crate) artifact_generation: String,
    #[arg(
        long,
        default_value_t = 1,
        help = "Occurrence page number, starting at 1"
    )]
    pub(crate) page: u32,
    #[arg(long, default_value_t = 20, help = "Occurrences per page (1-100)")]
    pub(crate) limit: u32,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas graph links actionspf2e:1kGNdIIhuglAjIp9\n  atlas graph links \"Demoralize\" --backlinks 4 --json\n  atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --outgoing 0 --backlinks 8 --json"
)]
pub(crate) struct GraphLinksOptions {
    #[arg(help = "Seed record key or strict resolvable record name")]
    pub(crate) record_ref: String,
    #[arg(long, default_value_t = 64, value_parser = parse_graph_limit, help = "Maximum outgoing neighbor records to include, 0-1024")]
    pub(crate) outgoing: usize,
    #[arg(long, default_value_t = 0, value_parser = parse_graph_limit, help = "Maximum backlink neighbor records to include, 0-1024; 0 disables backlinks")]
    pub(crate) backlinks: usize,
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
    if limit <= 1024 {
        Ok(limit)
    } else {
        Err(format!(
            "graph limit must be between 0 and 1024, got {limit}"
        ))
    }
}
