use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::cli::args::{CliPathMode, FilterOptions};

#[derive(Debug, Args)]
pub(crate) struct FiltersArgs {
    #[command(subcommand)]
    pub(crate) command: FiltersCommand,
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum FiltersCommand {
    #[command(about = "List filterable fields available in a filter space")]
    Fields(Box<FiltersFieldsOptions>),
    #[command(about = "List values, samples, or stats for one filter field")]
    Values(Box<FiltersValuesOptions>),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas filters fields\n  atlas filters fields --kind spell\n  atlas filters fields --kind creature --json"
)]
pub(crate) struct FiltersFieldsOptions {
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON used to scope field discovery"
    )]
    pub(crate) filter_json: Option<String>,
    #[command(flatten)]
    pub(crate) filter_options: DiscoveryFilterOptions,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas filters values --field traits --kind spell\n  atlas filters values --field metric --kind creature --metric-query armor\n  atlas filters values --field metric --kind creature --metric ac.value --json"
)]
pub(crate) struct FiltersValuesOptions {
    #[arg(
        long,
        help = "Filter field id to inspect, such as traits, rarity, level, or metric"
    )]
    pub(crate) field: String,
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON used to scope value discovery"
    )]
    pub(crate) filter_json: Option<String>,
    #[command(flatten)]
    pub(crate) filter_options: DiscoveryFilterOptions,
    #[arg(
        long,
        value_enum,
        help = "Sort discovered values by count, alphabetically, or canonical order"
    )]
    pub(crate) sort: Option<CliFilterValueSort>,
    #[arg(
        long,
        visible_alias = "limit",
        help = "Limit sampled text examples; enumerable values are not limited by default"
    )]
    pub(crate) sample_limit: Option<usize>,
    #[arg(long, help = "Show values for one resolved metric key")]
    pub(crate) metric: Option<String>,
    #[arg(
        long,
        help = "Show metric keys below this namespace prefix, such as speed."
    )]
    pub(crate) metric_prefix: Option<String>,
    #[arg(long, help = "Show metric keys with this exact display label")]
    pub(crate) metric_label: Option<String>,
    #[arg(
        long,
        help = "Filter metric discovery by case-insensitive query across metric keys, labels, short labels, and groups"
    )]
    pub(crate) metric_query: Option<String>,
    #[arg(
        long,
        help = "Limit metric discovery to this metric domain, such as actor or item"
    )]
    pub(crate) metric_domain: Option<String>,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliFilterValueSort {
    Count,
    Alpha,
    Canonical,
}

#[derive(Debug, Clone, Default, Args)]
pub(crate) struct DiscoveryFilterOptions {
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
        help = "Filter to records that reference this canonical record key; repeat to require all targets"
    )]
    pub(crate) references: Vec<String>,
    #[arg(
        long = "referenced-by",
        help = "Filter to records referenced by this canonical record key; repeat to require all sources"
    )]
    pub(crate) referenced_by: Vec<String>,
}

impl From<DiscoveryFilterOptions> for FilterOptions {
    fn from(options: DiscoveryFilterOptions) -> Self {
        Self {
            kinds: options.kinds,
            pack_names: options.pack_names,
            pack_labels: options.pack_labels,
            rarities: options.rarities,
            publication_titles: options.publication_titles,
            level: options.level,
            min_level: options.min_level,
            max_level: options.max_level,
            price: options.price,
            min_price: options.min_price,
            max_price: options.max_price,
            traits: options.traits,
            any_traits: options.any_traits,
            references: options.references,
            referenced_by: options.referenced_by,
            metrics: Vec::new(),
        }
    }
}
