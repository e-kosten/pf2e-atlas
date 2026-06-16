use std::process::ExitCode;

use atlas_search::GraphContextRequest;

use crate::client::AtlasClient;
use crate::output::write_json_data;

use super::args::GraphUsesOptions;
use super::data::graph_uses_data;
use super::open_graph_service;
use super::render::print_graph_uses;
use super::resolve::{
    GraphCommandOutcome, graph_search_error, record_not_found, resolve_graph_record_ref,
};

pub(crate) fn run_graph_uses(options: GraphUsesOptions) -> Result<ExitCode, String> {
    let service = match open_graph_service(options.path_mode, options.index, options.json)? {
        GraphCommandOutcome::Value(service) => service,
        GraphCommandOutcome::Exit(code) => return Ok(code),
    };
    let key = match resolve_graph_record_ref(&service, &options.record_ref, options.json)? {
        GraphCommandOutcome::Value(key) => key,
        GraphCommandOutcome::Exit(code) => return Ok(code),
    };
    let result = match service
        .graph_context(GraphContextRequest::uses(key.clone()).with_backlink_limit(options.limit))
    {
        Ok(Some(result)) => result,
        Ok(None) => return record_not_found(&key, options.json),
        Err(error) => match graph_search_error(error, options.json)? {
            GraphCommandOutcome::Value(result) => result,
            GraphCommandOutcome::Exit(code) => return Ok(code),
        },
    };
    let data = graph_uses_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_uses(&data, options.limit);
    }
    Ok(ExitCode::SUCCESS)
}
