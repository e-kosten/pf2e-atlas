use clap::Args;
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
