use clap::Args;
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
        help = "CEL filter expression; discover supported fields with atlas filters fields"
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
}
