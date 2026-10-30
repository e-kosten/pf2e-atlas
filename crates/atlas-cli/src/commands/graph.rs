use super::product::report_app_error;
use crate::client::{ClientOptions, connect};
use atlas_app_model::{GraphContextViewRequest, RecordRefResolutionView};
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
fn seed(
    s: &AtlasAppService,
    name: String,
    json: bool,
) -> Result<Result<atlas_domain::RecordKey, ExitCode>, String> {
    let result = match s.resolve_record_ref(name, None) {
        Ok(value) => value,
        Err(error) => return Ok(Err(report_app_error(error.into_app_error(), json)?)),
    };
    let (code, message, candidates) = match result {
        RecordRefResolutionView::Key(k) => return Ok(Ok(k)),
        RecordRefResolutionView::Miss => (
            atlas_app_model::AppErrorCode::RecordResolutionMiss,
            "no strict name or verified alias matched",
            vec![],
        ),
        RecordRefResolutionView::Ambiguous(matches) => (
            atlas_app_model::AppErrorCode::RecordResolutionAmbiguous,
            "ambiguous reference; specify a Foundry key",
            matches.iter().map(|m| m.record.clone()).collect(),
        ),
    };
    Ok(Err(report_app_error(
        atlas_app_model::AppError::new(code, message)
            .with_details(serde_json::json!({"candidates":candidates})),
        json,
    )?))
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
    let key = match seed(&s, o.record_ref, o.json)? {
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
pub(crate) fn run_graph_uses(o: GraphUsesOptions) -> Result<ExitCode, String> {
    let s = match client(o.path_mode, o.index) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let key = match seed(&s, o.record_ref, o.json)? {
        Ok(key) => key,
        Err(exit) => return Ok(exit),
    };
    show_graph(
        &s,
        GraphContextViewRequest {
            record_key: key.to_string(),
            owners: None,
            source_fingerprint: None,
            outgoing_limit: 0,
            backlink_limit: o.limit,
        },
        o.json,
    )
}
pub(crate) fn run_graph_remaster(o: GraphRemasterOptions) -> Result<ExitCode, String> {
    let s = match client(o.path_mode, o.index) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let key = match seed(&s, o.record_ref, o.json)? {
        Ok(key) => key,
        Err(exit) => return Ok(exit),
    };
    let r = match s.remaster_links(key) {
        Ok(value) => value,
        Err(error) => return report_app_error(error.into_app_error(), o.json),
    };
    let Some(r) = r else {
        return report_app_error(
            atlas_app_model::AppError::new(
                atlas_app_model::AppErrorCode::RecordNotFound,
                "record not found",
            ),
            o.json,
        );
    };
    let data = serde_json::to_value(r).map_err(|e| e.to_string())?;
    if o.json {
        write_json_data(data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_graph_variants(o: GraphVariantsOptions) -> Result<ExitCode, String> {
    let s = match client(o.path_mode, o.index) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let key = match seed(&s, o.record_ref, o.json)? {
        Ok(key) => key,
        Err(exit) => return Ok(exit),
    };
    let r = match s.variant_group(key) {
        Ok(value) => value,
        Err(error) => return report_app_error(error.into_app_error(), o.json),
    };
    let data = serde_json::json!({"label":"Suggested variants","availability":r});
    if o.json {
        write_json_data(data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
