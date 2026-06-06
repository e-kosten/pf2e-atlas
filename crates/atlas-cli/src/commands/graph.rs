use std::path::PathBuf;
use std::process::ExitCode;

use atlas_search::AtlasRetrievalService;

use crate::cli::args::CliPathMode;
use crate::output::write_json_error;

use super::record::{open_record_service, record_runtime};

pub(crate) mod args;
mod data;
mod links;
mod remaster;
mod render;
mod resolve;
mod uses;
mod variants;

pub(crate) use links::run_graph_links;
pub(crate) use remaster::run_graph_remaster;
pub(crate) use uses::run_graph_uses;
pub(crate) use variants::run_graph_variants;

fn open_graph_service(
    path_mode: CliPathMode,
    index: Option<PathBuf>,
    json: bool,
) -> Result<resolve::GraphCommandOutcome<AtlasRetrievalService>, String> {
    let runtime = match record_runtime(path_mode.into(), index) {
        Ok(runtime) => runtime,
        Err(error) if json => {
            write_json_error("runtime_error", error)?;
            return Ok(resolve::GraphCommandOutcome::Exit(ExitCode::from(3)));
        }
        Err(error) => return Err(error),
    };
    match open_record_service(&runtime) {
        Ok(service) => Ok(resolve::GraphCommandOutcome::Value(service)),
        Err(error) if json => {
            write_json_error("index_unavailable", error)?;
            Ok(resolve::GraphCommandOutcome::Exit(ExitCode::from(3)))
        }
        Err(error) => Err(error),
    }
}
