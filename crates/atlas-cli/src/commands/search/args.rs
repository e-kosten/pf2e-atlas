use crate::cli::args::FilterOptions;
use atlas_app_model::RetrievalModeView;
use atlas_cli_support::CliPathMode;
use atlas_search::DEFAULT_SEARCH_PAGE_SIZE;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas search --kind spell --trait healing\n  atlas search 'Ghoul Fever' --retrieval fts --json\n  atlas search --where 'defenses.hp.maximum >= 80 && \"undead\" in traits'\n  atlas filters fields"
)]
pub(crate) struct SearchOptions {
    #[arg(
        long,
        help = "Result order: relevance, name_asc/name_desc, level_asc/level_desc, unit_price_asc/unit_price_desc"
    )]
    pub(crate) sort: Option<atlas_domain::ResultSort>,
    #[arg(
        long,
        default_value = "score",
        help = "Hybrid text ranking: score fusion or reciprocal rank fusion (rrf)"
    )]
    pub(crate) ranking: atlas_domain::SearchRanking,
    #[arg(long,value_parser=crate::cli::args::parse_search_profile,help="Stored search profile: bge-base or embeddinggemma; default: sole stored profile, or bge-base when both exist")]
    pub(crate) search_profile: Option<atlas_domain::SearchProfileId>,
    #[arg(help = "Text query; omit to browse eligible records alphabetically")]
    pub(crate) query: Option<String>,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_SEARCH_PAGE_SIZE, help = "Results per page (1-100)")]
    pub(crate) limit: u32,
    #[arg(long, default_value_t = 1, help = "Page number, starting at 1")]
    pub(crate) page: u32,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long, value_enum, default_value_t = CliRetrievalMode::Hybrid, help = "Text retrieval; fts needs no model weights; ignored when browsing without a query")]
    pub(crate) retrieval: CliRetrievalMode,
    #[arg(long, help = "Override the embedding model cache root")]
    pub(crate) embedding_cache_path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        help = "Print the validated shared predicate without opening an artifact"
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
impl From<CliRetrievalMode> for RetrievalModeView {
    fn from(value: CliRetrievalMode) -> Self {
        match value {
            CliRetrievalMode::Fts => Self::Lexical,
            CliRetrievalMode::Vector => Self::Semantic,
            CliRetrievalMode::Hybrid => Self::Hybrid,
        }
    }
}
