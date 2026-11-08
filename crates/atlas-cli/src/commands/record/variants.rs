use super::{args::VariantsOptions, report_app_error};
use crate::client::{ClientOptions, connect};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;

pub(crate) fn run(o: VariantsOptions) -> Result<ExitCode, String> {
    let s = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let key = match super::resolve_seed(&s, o.record_ref, o.json)? {
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
