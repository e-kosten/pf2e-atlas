use std::process::ExitCode;

use crate::output::write_json_data;

use super::args::GraphVariantsOptions;
use super::data::graph_variants_data;
use super::open_graph_service;
use super::render::print_graph_variants;
use super::resolve::{GraphCommandOutcome, resolve_graph_variant_group};

pub(crate) fn run_graph_variants(options: GraphVariantsOptions) -> Result<ExitCode, String> {
    let service = match open_graph_service(options.path_mode, options.index, options.json)? {
        GraphCommandOutcome::Value(service) => service,
        GraphCommandOutcome::Exit(code) => return Ok(code),
    };
    let result = match resolve_graph_variant_group(&service, &options.record_ref, options.json)? {
        GraphCommandOutcome::Value(result) => result,
        GraphCommandOutcome::Exit(code) => return Ok(code),
    };
    let data = graph_variants_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_variants(&data);
    }
    Ok(ExitCode::SUCCESS)
}
