use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliSearchContent {
    All,
    Core,
}

impl From<CliSearchContent> for atlas_domain::SearchContentScope {
    fn from(value: CliSearchContent) -> Self {
        match value {
            CliSearchContent::All => Self::All,
            CliSearchContent::Core => Self::Core,
        }
    }
}

impl From<CliSearchContent> for atlas_domain::SimilaritySourceSections {
    fn from(value: CliSearchContent) -> Self {
        match value {
            CliSearchContent::All => Self::All,
            CliSearchContent::Core => Self::Core,
        }
    }
}
pub(crate) fn parse_search_profile(value: &str) -> Result<atlas_domain::SearchProfileId, String> {
    let id = atlas_domain::SearchProfileId::new(value);
    atlas_embedding::search_profile(&id).map_err(|_| {
        format!("unsupported search profile `{value}`; expected bge-base or embeddinggemma")
    })?;
    Ok(id)
}
#[derive(Debug, Clone, Default, Args)]
pub(crate) struct FilterOptions {
    #[arg(
        long = "where",
        help = "CEL filter expression, including links_to/linked_from and AND/OR/NOT; discover fields with atlas filters fields"
    )]
    pub(crate) where_expression: Option<String>,
    #[arg(long = "kind", help = "Record kind; repeat to accept any listed kind")]
    pub(crate) kinds: Vec<String>,
    #[arg(long = "pack-name", help = "Foundry pack ID; repeat for alternatives")]
    pub(crate) pack_names: Vec<String>,
    #[arg(long = "rarity", help = "Rarity; repeat for alternatives")]
    pub(crate) rarities: Vec<String>,
    #[arg(
        long = "trait",
        help = "Require a trait identifier; repeat to require every listed trait"
    )]
    pub(crate) traits: Vec<String>,
    #[arg(
        long,
        help = "Require links to a record by strict name or key; check each result and its contents; repeat for AND"
    )]
    pub(crate) links_to: Vec<String>,
    #[arg(
        long,
        help = "Require records linked from a record by strict name or key, including its contents; repeat for AND"
    )]
    pub(crate) linked_from: Vec<String>,
    #[arg(
        long,
        help = "Require links to a record by strict name or key; check only each result's main entry"
    )]
    pub(crate) links_to_main_entry: Vec<String>,
    #[arg(
        long,
        help = "Require records linked from the selected main entry only; strict name or key"
    )]
    pub(crate) linked_from_main_entry: Vec<String>,
}
