pub(crate) mod agent_skills;
pub(crate) mod build_info;
pub(crate) mod completions;
pub(crate) mod encounters;
pub(crate) mod filter_discovery;
pub(crate) mod filters;
pub(crate) mod graph;
pub(crate) mod index;
pub(crate) mod lists;
pub(crate) mod product;
pub(crate) mod record;
mod remaster_presentation;
pub(crate) mod result_order;
pub(crate) mod roll;
pub(crate) mod search;
pub(crate) mod setup;
pub(crate) mod similar;
#[cfg(feature = "web")]
pub(crate) mod web;

#[cfg(target_os = "macos")]
pub(crate) mod desktop;

pub(crate) mod workspace;
