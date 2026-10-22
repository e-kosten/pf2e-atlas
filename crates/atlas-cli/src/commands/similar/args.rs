use crate::cli::args::FilterOptions;
use atlas_cli_support::CliPathMode;
use clap::Args;
use std::path::PathBuf;
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas similar spells-srd:1xLVcA8Y1onw7toT --kind spell\n  atlas similar feats-srd:jM72TjJ965jocBV8 --limit 12 --json\nSimilarity uses the seed's stored root identity vector. Filters constrain candidates, not seed resolution. Candidate coverage is bounded."
)]
pub(crate) struct SimilarOptions {
    #[arg(
        long,
        help = "Result order: relevance, random or repeated attribute tokens"
    )]
    pub(crate) sort: Vec<String>,
    #[arg(
        long,
        help = "Random order seed (requires only --sort random); generated when omitted"
    )]
    pub(crate) seed: Option<u32>,
    #[arg(long, default_value_t = 1, help = "Page number, starting at 1")]
    pub(crate) page: u32,
    #[arg(long,value_parser=crate::cli::args::parse_search_profile,help="Stored search profile: bge-base or embeddinggemma; needs stored vectors, not model weights")]
    pub(crate) search_profile: Option<atlas_domain::SearchProfileId>,
    #[arg(help = "Foundry record key or strict name/verified alias")]
    pub(crate) record_ref: String,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, default_value_t = 20, help = "Results per page (1-100)")]
    pub(crate) limit: u32,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
