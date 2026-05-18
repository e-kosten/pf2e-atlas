use std::process::ExitCode;

use atlas_index::{DiscoveryError, DiscoveryValueSort, FilterValueRequest};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

use crate::output::{write_json_data, write_json_error};
use crate::{CliFilterValueSort, FilterOptions, FiltersFieldsOptions, FiltersValuesOptions};

use super::filters::build_filter;

pub(crate) fn run_filters_fields(options: FiltersFieldsOptions) -> Result<ExitCode, String> {
    let (filter, filter_value) = match build_filter(
        options.filter_json.as_deref(),
        &FilterOptions::from(options.filter_options),
    ) {
        Ok(filter) => filter,
        Err(error) => {
            write_json_error(error.code, error.message)?;
            return Ok(ExitCode::from(2));
        }
    };
    let index = match open_index(options.index, options.path_mode.into()) {
        Ok(index) => index,
        Err(error) => {
            write_json_error("index_unavailable", error)?;
            return Ok(ExitCode::from(3));
        }
    };
    match index.list_filter_fields(filter.as_ref(), filter_value) {
        Ok(data) => {
            write_json_data(data)?;
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => write_discovery_error(error),
    }
}

pub(crate) fn run_filters_values(options: FiltersValuesOptions) -> Result<ExitCode, String> {
    let (filter, filter_value) = match build_filter(
        options.filter_json.as_deref(),
        &FilterOptions::from(options.filter_options),
    ) {
        Ok(filter) => filter,
        Err(error) => {
            write_json_error(error.code, error.message)?;
            return Ok(ExitCode::from(2));
        }
    };
    let request = FilterValueRequest {
        field: options.field,
        filter_json: filter_value,
        sort: options.sort.map(discovery_sort),
        sample_limit: options.sample_limit,
        metric: options.metric,
        metric_prefix: options.metric_prefix,
        metric_label: options.metric_label,
        metric_domain: options.metric_domain,
    };
    let index = match open_index(options.index, options.path_mode.into()) {
        Ok(index) => index,
        Err(error) => {
            write_json_error("index_unavailable", error)?;
            return Ok(ExitCode::from(3));
        }
    };
    match index.list_filter_values(filter.as_ref(), request) {
        Ok(data) => {
            write_json_data(data)?;
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => write_discovery_error(error),
    }
}

fn open_index(
    index: Option<std::path::PathBuf>,
    path_mode: atlas_runtime::AtlasPathMode,
) -> Result<atlas_index::AtlasIndex, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })?;
    runtime.open_index().map_err(|error| error.to_string())
}

fn discovery_sort(sort: CliFilterValueSort) -> DiscoveryValueSort {
    match sort {
        CliFilterValueSort::Count => DiscoveryValueSort::Count,
        CliFilterValueSort::Alpha => DiscoveryValueSort::Alpha,
        CliFilterValueSort::Canonical => DiscoveryValueSort::Canonical,
    }
}

fn write_discovery_error(error: DiscoveryError) -> Result<ExitCode, String> {
    let code = match error {
        DiscoveryError::InvalidField(_) => "invalid_field",
        DiscoveryError::InvalidOption(_) => "invalid_option",
        DiscoveryError::FieldNotApplicable(_) => "field_not_applicable",
        DiscoveryError::AmbiguousMetric(_) => "ambiguous_metric",
        DiscoveryError::Filter(_) => "invalid_filter",
        DiscoveryError::QueryFailed(_) => "query_failed",
    };
    write_json_error(code, error.to_string())?;
    Ok(ExitCode::from(2))
}
