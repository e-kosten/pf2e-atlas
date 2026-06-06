use std::process::ExitCode;

use atlas_search::{RemasterLinksRequest, RemasterRetrieval};

use crate::output::{write_json_data, write_json_error};

use super::super::record::{search_error, search_error_code};
use super::args::GraphRemasterOptions;
use super::data::graph_remaster_data;
use super::open_graph_service;
use super::render::print_graph_remaster;
use super::resolve::{GraphCommandOutcome, record_not_found, resolve_graph_record_ref};

pub(crate) fn run_graph_remaster(options: GraphRemasterOptions) -> Result<ExitCode, String> {
    let service = match open_graph_service(options.path_mode, options.index, options.json)? {
        GraphCommandOutcome::Value(service) => service,
        GraphCommandOutcome::Exit(code) => return Ok(code),
    };
    let key = match resolve_graph_record_ref(&service, &options.record_ref, options.json)? {
        GraphCommandOutcome::Value(key) => key,
        GraphCommandOutcome::Exit(code) => return Ok(code),
    };
    let result = match service.remaster_links(RemasterLinksRequest { record_key: &key }) {
        Ok(Some(result)) => result,
        Ok(None) => return record_not_found(&key, options.json),
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let data = graph_remaster_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_remaster(&data);
    }
    Ok(ExitCode::SUCCESS)
}
