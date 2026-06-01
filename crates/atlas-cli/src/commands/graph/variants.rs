use std::process::ExitCode;

use crate::output::{write_json_data, write_json_error};

use super::super::record::{open_record_service, record_runtime};
use super::args::GraphVariantsOptions;
use super::data::graph_variants_data;
use super::render::print_graph_variants;
use super::resolve::{GraphCommandOutcome, resolve_graph_variant_group};

pub(crate) fn run_graph_variants(options: GraphVariantsOptions) -> Result<ExitCode, String> {
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
