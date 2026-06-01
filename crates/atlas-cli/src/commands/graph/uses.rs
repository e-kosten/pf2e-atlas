use std::process::ExitCode;

use atlas_search::{GraphContextRequest, GraphRetrieval};

use crate::output::{write_json_data, write_json_error};

use super::super::record::{open_record_service, record_runtime, search_error, search_error_code};
use super::args::GraphUsesOptions;
use super::data::graph_uses_data;
use super::render::print_graph_uses;
use super::resolve::{GraphCommandOutcome, record_not_found, resolve_graph_record_ref};

pub(crate) fn run_graph_uses(options: GraphUsesOptions) -> Result<ExitCode, String> {
    let runtime = match record_runtime(options.path_mode.into(), options.index) {
        Ok(runtime) => runtime,
        Err(error) if options.json => {
            write_json_error("runtime_error", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let service = match open_record_service(&runtime) {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error("index_unavailable", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let key = match resolve_graph_record_ref(&service, &options.record_ref, options.json)? {
        GraphCommandOutcome::Value(key) => key,
        GraphCommandOutcome::Exit(code) => return Ok(code),
    };
    let result = match service.graph_context(GraphContextRequest {
        seed: key.clone(),
        outgoing_limit: 0,
        backlink_limit: options.limit,
    }) {
        Ok(Some(result)) => result,
        Ok(None) => return record_not_found(&key, options.json),
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let data = graph_uses_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_uses(&data, options.limit);
    }
    Ok(ExitCode::SUCCESS)
}
