use atlas_runtime::AtlasPathMode;
use clap::{Args, ValueEnum};

use crate::progress;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliPathMode {
    Repo,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliProgressMode {
    Auto,
    Always,
    Never,
}

impl From<CliPathMode> for AtlasPathMode {
    fn from(mode: CliPathMode) -> Self {
        match mode {
            CliPathMode::Repo => Self::Repo,
            CliPathMode::Global => Self::Global,
        }
    }
}

impl From<CliProgressMode> for progress::ProgressMode {
    fn from(mode: CliProgressMode) -> Self {
        match mode {
            CliProgressMode::Auto => Self::Auto,
            CliProgressMode::Always => Self::Always,
            CliProgressMode::Never => Self::Never,
        }
    }
}

#[derive(Debug, Clone, Default, Args)]
pub(crate) struct FilterOptions {
    #[arg(
        long = "kind",
        help = "Filter to records in this kind; repeat for any of several kinds"
    )]
    pub(crate) kinds: Vec<String>,
    #[arg(
        long = "pack-name",
        help = "Filter to this stable source pack name; repeat for any of several packs"
    )]
    pub(crate) pack_names: Vec<String>,
    #[arg(
        long = "pack-label",
        help = "Filter to this display pack label; repeat for any of several labels"
    )]
    pub(crate) pack_labels: Vec<String>,
    #[arg(
        long = "rarity",
        help = "Filter to this rarity; repeat for any of several rarities"
    )]
    pub(crate) rarities: Vec<String>,
    #[arg(
        long = "publication-title",
        help = "Filter to this exact publication title; repeat for any of several titles"
    )]
    pub(crate) publication_titles: Vec<String>,
    #[arg(
        long = "level",
        help = "Filter to an exact level or inclusive range such as 1..5"
    )]
    pub(crate) level: Option<String>,
    #[arg(long = "min-level", help = "Filter to records at or above this level")]
    pub(crate) min_level: Option<f64>,
    #[arg(long = "max-level", help = "Filter to records at or below this level")]
    pub(crate) max_level: Option<f64>,
    #[arg(
        long = "price",
        help = "Filter to an exact price in copper pieces or inclusive range such as 100..500"
    )]
    pub(crate) price: Option<String>,
    #[arg(
        long = "min-price",
        help = "Filter to records priced at or above this many copper pieces"
    )]
    pub(crate) min_price: Option<f64>,
    #[arg(
        long = "max-price",
        help = "Filter to records priced at or below this many copper pieces"
    )]
    pub(crate) max_price: Option<f64>,
    #[arg(
        long = "trait",
        help = "Require this trait; repeat to require all listed traits"
    )]
    pub(crate) traits: Vec<String>,
    #[arg(
        long = "any-trait",
        help = "Require at least one of these traits; repeat for alternatives"
    )]
    pub(crate) any_traits: Vec<String>,
    #[arg(
        long = "references",
        help = "Filter to records that link to/reference this canonical record key; repeat to require all targets"
    )]
    pub(crate) references: Vec<String>,
    #[arg(
        long = "referenced-by",
        help = "Filter to records linked from/referenced by this canonical record key; repeat to require all sources"
    )]
    pub(crate) referenced_by: Vec<String>,
    #[arg(
        long = "metric",
        help = "Filter by metric predicate such as ac.value>=18, hp.value:40, or save.fort.mod>=12; repeat to require all"
    )]
    pub(crate) metrics: Vec<String>,
}
