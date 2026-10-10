use crate::cli::args::FilterOptions;
use atlas_cli_support::CliPathMode;
use clap::Args;
use std::path::PathBuf;
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas similar 'Dirge of Doom' --kind spell\n  atlas similar feats-srd:jM72TjJ965jocBV8 --limit 12 --json\nSimilarity uses the seed's stored root identity vector. Candidate coverage is bounded."
)]
pub(crate) struct SimilarOptions {
    #[arg(help = "Foundry record key or strict name/verified alias")]
    pub(crate) record_ref: String,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    pub(crate) limit: u32,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
}
