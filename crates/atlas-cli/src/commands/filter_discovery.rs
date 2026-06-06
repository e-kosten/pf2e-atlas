use std::process::ExitCode;

use atlas_domain::{
    BooleanFieldCounts, FilterFieldDiscovery, FilterFieldGroup, FilterFieldInfo, FilterFieldType,
    FilterSample, FilterValueDiscovery, FilterValuePayload, FilterValuePolicy, MetricKeyDiscovery,
    MetricValuePayload, NumericFieldStats,
};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{
    DiscoverFilterFieldsRequest, DiscoverFilterValuesRequest, FilterDiscoveryError,
    FilterDiscoveryRetrieval, MetricDiscoverySelector,
};
use serde_json::Value;

use crate::cli::args::FilterOptions;
use crate::output::{write_json_data, write_json_error};

pub(crate) mod args;

use args::{CliFilterValueSort, FiltersFieldsOptions, FiltersValuesOptions};

use super::filters::build_filter;

pub(crate) fn run_filters_fields(options: FiltersFieldsOptions) -> Result<ExitCode, String> {
    let (filter, filter_value) = match build_filter(
        options.filter_json.as_deref(),
        &FilterOptions::from(options.filter_options),
    ) {
        Ok(filter) => filter,
        Err(error) => {
            return discovery_cli_error(options.json, error.code, error.message, ExitCode::from(2));
        }
    };
    let service = match open_filter_discovery_service(options.index, options.path_mode.into()) {
        Ok(service) => service,
        Err(error) => {
            return discovery_cli_error(
                options.json,
                "index_unavailable",
                error,
                ExitCode::from(3),
            );
        }
    };
    match service.discover_filter_fields(DiscoverFilterFieldsRequest {
        filter: filter.as_ref(),
        filter_json: filter_value,
    }) {
        Ok(data) => {
            if options.json {
                write_json_data(data)?;
            } else {
                print_filter_fields(&data);
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => write_discovery_error(error, options.json),
    }
}

pub(crate) fn run_filters_values(options: FiltersValuesOptions) -> Result<ExitCode, String> {
    let metric_selector = match metric_selector_from_options(&options) {
        Ok(selector) => selector,
        Err(message) => {
            return discovery_cli_error(options.json, "invalid_option", message, ExitCode::from(2));
        }
    };
    let (filter, filter_value) = match build_filter(
        options.filter_json.as_deref(),
        &FilterOptions::from(options.filter_options),
    ) {
        Ok(filter) => filter,
        Err(error) => {
            return discovery_cli_error(options.json, error.code, error.message, ExitCode::from(2));
        }
    };
    let request = DiscoverFilterValuesRequest {
        field: options.field,
        filter: filter.as_ref(),
        filter_json: filter_value,
        sort: options.sort.map(discovery_sort),
        sample_limit: options.sample_limit,
        metric_selector,
        metric_domain: options.metric_domain,
    };
    let service = match open_filter_discovery_service(options.index, options.path_mode.into()) {
        Ok(service) => service,
        Err(error) => {
            return discovery_cli_error(
                options.json,
                "index_unavailable",
                error,
                ExitCode::from(3),
            );
        }
    };
    match service.discover_filter_values(request) {
        Ok(data) => {
            if options.json {
                write_json_data(data)?;
            } else {
                print_filter_values(&data);
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => write_discovery_error(error, options.json),
    }
}

fn open_filter_discovery_service(
    index: Option<std::path::PathBuf>,
    path_mode: atlas_runtime::AtlasPathMode,
) -> Result<atlas_search::AtlasRetrievalService, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })
    .map_err(|error| error.to_string())?;
    runtime
        .open_retrieval_service_no_embeddings()
        .map_err(|error| error.to_string())
}

fn discovery_sort(sort: CliFilterValueSort) -> atlas_domain::FilterValueSort {
    match sort {
        CliFilterValueSort::Count => atlas_domain::FilterValueSort::Count,
        CliFilterValueSort::Alpha => atlas_domain::FilterValueSort::Alpha,
        CliFilterValueSort::Canonical => atlas_domain::FilterValueSort::Canonical,
    }
}

fn metric_selector_from_options(
    options: &FiltersValuesOptions,
) -> Result<Option<MetricDiscoverySelector>, String> {
    let mut selectors = Vec::new();
    if let Some(metric) = options.metric.clone() {
        selectors.push(("--metric", MetricDiscoverySelector::ExactKey(metric)));
    }
    if let Some(prefix) = options.metric_prefix.clone() {
        selectors.push(("--metric-prefix", MetricDiscoverySelector::Prefix(prefix)));
    }
    if let Some(label) = options.metric_label.clone() {
        selectors.push(("--metric-label", MetricDiscoverySelector::Label(label)));
    }
    if let Some(query) = options.metric_query.clone() {
        selectors.push(("--metric-query", MetricDiscoverySelector::Query(query)));
    }

    match selectors.len() {
        0 => Ok(None),
        1 => Ok(selectors.pop().map(|(_, selector)| selector)),
        _ => Err(format!(
            "metric selector flags are mutually exclusive: {}",
            selectors
                .into_iter()
                .map(|(flag, _)| flag)
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn write_discovery_error(error: FilterDiscoveryError, json: bool) -> Result<ExitCode, String> {
    let code = match error {
        FilterDiscoveryError::InvalidField(_) => "invalid_field",
        FilterDiscoveryError::InvalidOption(_) => "invalid_option",
        FilterDiscoveryError::FieldNotApplicable(_) => "field_not_applicable",
        FilterDiscoveryError::AmbiguousMetric(_) => "ambiguous_metric",
        FilterDiscoveryError::InvalidFilter(_) => "invalid_filter",
        FilterDiscoveryError::QueryFailed(_) => "query_failed",
    };
    discovery_cli_error(json, code, error.to_string(), ExitCode::from(2))
}

fn discovery_cli_error(
    json: bool,
    code: &'static str,
    message: String,
    exit_code: ExitCode,
) -> Result<ExitCode, String> {
    if json {
        write_json_error(code, message)?;
    } else {
        eprintln!("{message}");
    }
    Ok(exit_code)
}

fn print_filter_fields(data: &FilterFieldDiscovery) {
    println!("Filter fields");
    print_filter_context(data.filter.as_ref(), data.matching_record_count);
    println!();

    let groups = [
        FilterFieldGroup::Record,
        FilterFieldGroup::Spell,
        FilterFieldGroup::Actor,
        FilterFieldGroup::Item,
        FilterFieldGroup::Variant,
        FilterFieldGroup::Metric,
    ];
    for group in groups {
        let fields = data
            .fields
            .iter()
            .filter(|field| field.group == group)
            .collect::<Vec<_>>();
        if fields.is_empty() {
            continue;
        }
        println!("{}", group_label(group));
        for field in fields {
            print_field_row(field);
        }
        println!();
    }
}

fn print_field_row(field: &FilterFieldInfo) {
    let flags = if field.cli_flags.is_empty() {
        String::new()
    } else {
        format!("  flags: {}", field.cli_flags.join(", "))
    };
    println!(
        "  {:<22} {:<12} {}{}",
        field.field,
        field_type_label(field.field_type),
        value_policy_label(field.value_policy),
        flags
    );
}

fn print_filter_values(data: &FilterValueDiscovery) {
    println!("Values for field: {}", data.field);
    print_filter_context(data.filter.as_ref(), data.matching_record_count);
    println!();

    match &data.payload {
        FilterValuePayload::Enumerable {
            values, null_count, ..
        } => {
            for value in values {
                println!("{:<28} {}", value.value, value.count);
            }
            println!();
            println!("Null values: {null_count}");
        }
        FilterValuePayload::Sample {
            sample,
            field_stats,
            null_count,
        } => print_sample_values(sample, field_stats.distinct_count, *null_count),
        FilterValuePayload::NumericStats { stats } => print_numeric_stats(stats),
        FilterValuePayload::BooleanCounts { counts } => print_boolean_counts(counts),
        FilterValuePayload::MetricKeys { metrics } => print_metric_keys(metrics),
        FilterValuePayload::MetricValues { metric, values } => {
            print_metric_header(metric);
            println!();
            match values {
                MetricValuePayload::NumericStats { stats } => print_numeric_stats(stats),
                MetricValuePayload::TextValues { values } => {
                    for value in values {
                        println!("{:<28} {}", value.value, value.count);
                    }
                }
                MetricValuePayload::BooleanCounts { counts } => print_boolean_counts(counts),
            }
        }
    }
}

fn print_filter_context(filter: Option<&Value>, matching_record_count: u64) {
    if let Some(filter) = filter {
        println!("Filter: {}", filter_label(filter));
    }
    println!("Matching records: {matching_record_count}");
}

fn print_sample_values(sample: &FilterSample, distinct_count: u64, null_count: u64) {
    println!("Distinct values: {distinct_count}");
    println!("Null values: {null_count}");
    println!();
    for example in &sample.examples {
        let suffix = if example.truncated { "..." } else { "" };
        println!("{:<28} {}{}", example.text, example.count, suffix);
    }
    if sample.omitted_distinct_count > 0 {
        println!();
        println!("{} distinct values omitted", sample.omitted_distinct_count);
    }
}

fn print_metric_keys(metrics: &[MetricKeyDiscovery]) {
    println!("Metrics");
    println!();
    for metric in metrics {
        let label = metric.label.as_deref().unwrap_or("");
        if let Some(stats) = &metric.numeric_stats {
            println!(
                "{:<18} {:<7} {:<22} count: {}  min: {}  p50: {}  max: {}",
                metric.metric_key,
                metric.value_type,
                label,
                metric.count,
                number_or_dash(stats.min),
                number_or_dash(stats.p50),
                number_or_dash(stats.max)
            );
        } else {
            println!(
                "{:<18} {:<7} {:<22} count: {}",
                metric.metric_key, metric.value_type, label, metric.count
            );
        }
    }
}

fn print_metric_header(metric: &MetricKeyDiscovery) {
    let label = metric.label.as_deref().unwrap_or("");
    println!(
        "Metric: {}  {}  {}  count: {}",
        metric.metric_key, metric.value_type, label, metric.count
    );
}

fn print_numeric_stats(stats: &NumericFieldStats) {
    println!("Numeric stats");
    println!("Count: {}", stats.count);
    println!("Null values: {}", stats.null_count);
    println!(
        "Min: {}  p05: {}  p25: {}  p50: {}  mean: {}  p75: {}  p95: {}  max: {}",
        number_or_dash(stats.min),
        number_or_dash(stats.p05),
        number_or_dash(stats.p25),
        number_or_dash(stats.p50),
        number_or_dash(stats.mean),
        number_or_dash(stats.p75),
        number_or_dash(stats.p95),
        number_or_dash(stats.max)
    );
}

fn print_boolean_counts(counts: &BooleanFieldCounts) {
    println!("true   {}", counts.r#true);
    println!("false  {}", counts.r#false);
    println!("null   {}", counts.null);
}

fn filter_label(filter: &Value) -> String {
    match (
        filter.get("kind").and_then(Value::as_str),
        filter.get("value").and_then(Value::as_str),
    ) {
        (Some("record_kind"), Some(value)) => format!("kind = {value}"),
        _ => serde_json::to_string(filter).unwrap_or_else(|_| "<filter>".to_string()),
    }
}

fn group_label(group: FilterFieldGroup) -> &'static str {
    match group {
        FilterFieldGroup::Record => "record",
        FilterFieldGroup::Spell => "spell",
        FilterFieldGroup::Actor => "actor",
        FilterFieldGroup::Item => "item",
        FilterFieldGroup::Variant => "variant",
        FilterFieldGroup::Metric => "metric",
    }
}

fn field_type_label(field_type: FilterFieldType) -> &'static str {
    match field_type {
        FilterFieldType::Set => "set",
        FilterFieldType::EnumString => "enum_string",
        FilterFieldType::Text => "text",
        FilterFieldType::Number => "number",
        FilterFieldType::Boolean => "boolean",
        FilterFieldType::Metric => "metric",
    }
}

fn value_policy_label(policy: FilterValuePolicy) -> &'static str {
    match policy {
        FilterValuePolicy::Enumerable => "enumerable",
        FilterValuePolicy::Sample => "sample",
        FilterValuePolicy::NumericStats => "numeric_stats",
        FilterValuePolicy::BooleanCounts => "boolean_counts",
        FilterValuePolicy::MetricKeys => "metric_keys",
        FilterValuePolicy::MetricValues => "metric_values",
    }
}

fn number_or_dash(value: Option<f64>) -> String {
    value
        .map(|value| {
            if value.fract() == 0.0 {
                format!("{}", value as i64)
            } else {
                format!("{value:.2}")
            }
        })
        .unwrap_or_else(|| "-".to_string())
}
