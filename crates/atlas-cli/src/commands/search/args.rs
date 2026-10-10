use crate::cli::args::FilterOptions;
use atlas_app_model::RetrievalModeView;
use atlas_cli_support::CliPathMode;
use atlas_search::DEFAULT_SEARCH_PAGE_SIZE;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas search --kind spell --trait healing\n  atlas search 'Ghoul Fever' --retrieval fts --json\n  atlas search --where 'actor.hp.maximum >= 80 && \"undead\" in traits'\n  atlas filters fields"
)]
pub(crate) struct SearchOptions {
    #[arg(help = "Text query; omit to browse eligible records alphabetically")]
    pub(crate) query: Option<String>,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long,default_value_t=DEFAULT_SEARCH_PAGE_SIZE)]
    pub(crate) limit: u32,
    #[arg(long, default_value_t = 1)]
    pub(crate) page: u32,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long,value_enum,default_value_t=CliRetrievalMode::Hybrid)]
    pub(crate) retrieval: CliRetrievalMode,
    #[arg(long)]
    pub(crate) embedding_cache_path: Option<PathBuf>,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        help = "Print the validated shared predicate without opening an artifact"
    )]
    pub(crate) print_filter: bool,
    #[arg(long)]
    pub(crate) json: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliRetrievalMode {
    Fts,
    Vector,
    Hybrid,
}
impl From<CliRetrievalMode> for RetrievalModeView {
    fn from(value: CliRetrievalMode) -> Self {
        match value {
            CliRetrievalMode::Fts => Self::Lexical,
            CliRetrievalMode::Vector => Self::Semantic,
            CliRetrievalMode::Hybrid => Self::Hybrid,
        }
    }
}
