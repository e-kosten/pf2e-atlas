use super::product::report_app_error;
use crate::client::{ClientOptions, connect};
use atlas_app_model::GraphContextViewRequest;
use atlas_app_service::AtlasAppService;
use atlas_cli_support::{CliPathMode, write_json_data};
use std::{path::PathBuf, process::ExitCode};
pub(crate) mod args;
pub(crate) mod occurrences;
use args::*;
fn client(
    mode: CliPathMode,
    index: Option<PathBuf>,
) -> Result<AtlasAppService, atlas_app_model::AppError> {
    connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(mode),
        index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    })
}
fn show_graph(
    s: &AtlasAppService,
    request: GraphContextViewRequest,
    json: bool,
) -> Result<ExitCode, String> {
    let result = match s.graph_context(request) {
        Ok(value) => value,
        Err(error) => return report_app_error(error.into_app_error(), json),
    };
    let Some(result) = result else {
        return report_app_error(
            atlas_app_model::AppError::new(
                atlas_app_model::AppErrorCode::RecordNotFound,
                "record not found",
            ),
            json,
        );
    };
    let data = serde_json::to_value(result).map_err(|e| e.to_string())?;
    if json {
        write_json_data(data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_graph_links(o: GraphLinksOptions) -> Result<ExitCode, String> {
    let s = match client(o.path_mode, o.index) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let key = match super::record::resolve_seed(&s, o.record_ref, o.json)? {
        Ok(key) => key,
        Err(exit) => return Ok(exit),
    };
    show_graph(
        &s,
        GraphContextViewRequest {
            record_key: key.to_string(),
            owners: None,
            source_fingerprint: None,
            outgoing_limit: o.outgoing,
            backlink_limit: o.backlinks,
        },
        o.json,
    )
}
