use crate::cli::args::FilterOptions;
use atlas_cli_support::CliPathMode;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas similar 'Brown Mold' --list-parts\n  atlas similar 'Brown Mold' --method passages --decay 0.5 --explain\n  atlas similar 'Brown Mold' --method content-average --kind hazard\n  atlas similar trait:fire --method baseline --json\n\nSimilarity uses stored vectors and does not require model weights. Filters constrain candidates, not seed resolution. Candidate coverage is bounded. Copy exact --part selectors and --artifact-generation from --list-parts."
)]
pub(crate) struct SimilarOptions {
    #[arg(
        long,
        value_enum,
        conflicts_with = "list_parts",
        help = "Comparison method: passages, content-average or current baseline"
    )]
    pub(crate) method: Option<CliSimilarityMethod>,
    #[arg(long, value_parser = parse_decay, conflicts_with = "list_parts", help = "Passage match decay, 0 for strongest part through 1 for equal parts (default 0.5)")]
    pub(crate) decay: Option<f64>,
    #[arg(
        long = "part",
        requires = "artifact_generation",
        conflicts_with = "list_parts",
        help = "Exact part selector from --list-parts; repeat to select several parts"
    )]
    pub(crate) parts: Vec<String>,
    #[arg(
        long,
        value_enum,
        conflicts_with = "list_parts",
        help = "Candidate content: any or corresponding comparison roles (default any)"
    )]
    pub(crate) candidate_parts: Option<CliCandidateParts>,
    #[arg(long, conflicts_with_all = ["sort", "seed", "page", "limit", "explain", "artifact_generation"], help = "List selectable seed content and exact selectors without ranking")]
    pub(crate) list_parts: bool,
    #[arg(
        long,
        help = "Expected artifact generation; required with explicit --part selectors"
    )]
    pub(crate) artifact_generation: Option<String>,
    #[arg(
        long,
        conflicts_with = "list_parts",
        help = "Show part contributions and paired match evidence"
    )]
    pub(crate) explain: bool,
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
    #[arg(help = "Foundry record key, strict name/verified alias, or exact trait:<identifier>")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliSimilarityMethod {
    Passages,
    ContentAverage,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliCandidateParts {
    Any,
    Corresponding,
}

fn parse_decay(value: &str) -> Result<f64, String> {
    let decay = value
        .parse::<f64>()
        .map_err(|_| "decay must be a number from 0 through 1".to_owned())?;
    if decay.is_finite() && (0.0..=1.0).contains(&decay) {
        Ok(decay)
    } else {
        Err("decay must be a finite number from 0 through 1".into())
    }
}
