pub(crate) mod agent_skills;
pub(crate) mod build_info;
pub(crate) mod characters;
pub(crate) mod completions;
pub(crate) mod encounters;
pub(crate) mod filter_discovery;
pub(crate) mod filters;
pub(crate) mod followup;
pub(crate) mod graph;
pub(crate) mod index;
pub(crate) mod lists;
pub(crate) mod parties;
pub(crate) mod product;
pub(crate) mod record;
mod remaster_presentation;
pub(crate) mod result_order;
pub(crate) mod roll;
mod saved_workspace;
pub(crate) mod search;
pub(crate) mod setup;
pub(crate) mod similar;
#[cfg(feature = "web")]
pub(crate) mod web;

#[cfg(any(target_os = "macos", all(windows, feature = "web")))]
pub(crate) mod desktop;
mod desktop_workspace;

pub(crate) mod workspace;
