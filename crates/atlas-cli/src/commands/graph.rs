use std::path::PathBuf;
use std::process::ExitCode;

use crate::cli::args::CliPathMode;
use crate::client::{AtlasClientConfig, AtlasClientHandle, LocalAtlasClientOptions, connect};
use crate::output::write_json_error;

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
) -> Result<resolve::GraphCommandOutcome<AtlasClientHandle>, String> {
    match connect(AtlasClientConfig::Local(LocalAtlasClientOptions {
        path_mode: path_mode.into(),
        index_path: index,
        embedding_cache_root: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })) {
        Ok(client) => Ok(resolve::GraphCommandOutcome::Value(client)),
        Err(error) if json => {
            write_json_error(resolve::graph_error_code(error.code), error.message)?;
            Ok(resolve::GraphCommandOutcome::Exit(ExitCode::from(3)))
        }
        Err(error) => Err(error.message),
    }
}
