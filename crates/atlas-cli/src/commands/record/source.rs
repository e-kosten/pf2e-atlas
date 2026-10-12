use super::args::RecordSourceOptions;
use crate::client::{ClientOptions, connect};
use crate::commands::product::app_result;
use atlas_app_service::AppServiceRetrievalMode;
use atlas_cli_support::write_json_data;
use std::process::ExitCode;

pub(crate) fn run_record_source(options: RecordSourceOptions) -> Result<ExitCode, String> {
    let service = connect(ClientOptions {
        path_mode: options.path_mode.into(),
        index: options.index,
        embedding_cache: None,
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .map_err(|error| error.message)?;
    let Some(source) = app_result(service.record_source(&options.key), options.json)? else {
        return Ok(ExitCode::from(3));
    };
    if options.json {
        write_json_data(source)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&source).map_err(|error| error.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
