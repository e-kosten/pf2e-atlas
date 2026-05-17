use std::collections::BTreeMap;
use std::process::ExitCode;

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_record::{RecordJsonOptions, record_json};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{RecordResolutionResult, SearchError};
use serde::Serialize;

use crate::output::{CliError, write_json_data, write_json_error};
use crate::{RecordGetOptions, RecordResolveOptions};

const MAX_GET_KEYS: usize = 100;
const MAX_RESOLVE_QUERIES: usize = 25;

#[derive(Debug, Serialize)]
struct RecordGetData<T> {
    detail: String,
    #[serde(flatten)]
    body: T,
}

#[derive(Debug, Serialize)]
struct SingleRecordBody {
    record: atlas_record::RecordJson,
}

#[derive(Debug, Serialize)]
struct BatchRecordBody {
    results: Vec<RecordGetItem>,
    counts: BatchCounts,
}

#[derive(Debug, Serialize)]
struct RecordGetItem {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<atlas_record::RecordJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<CliError>,
}

#[derive(Debug, Serialize)]
struct RecordResolveData<T> {
    detail: String,
    #[serde(flatten)]
    body: T,
}

#[derive(Debug, Serialize)]
struct SingleResolveBody {
    result: RecordResolveItem,
}

#[derive(Debug, Serialize)]
struct BatchResolveBody {
    results: Vec<RecordResolveItem>,
    counts: BatchCounts,
}

#[derive(Debug, Serialize)]
struct RecordResolveItem {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<atlas_record::RecordJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<RecordResolutionJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alternatives: Option<Vec<RecordResolveAlternative>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<CliError>,
}

#[derive(Debug, Serialize)]
struct RecordResolveAlternative {
    record: atlas_record::RecordJson,
    resolution: RecordResolutionJson,
}

#[derive(Debug, Serialize)]
struct RecordResolutionJson {
    query: String,
    normalized_query: String,
    match_kind: &'static str,
    matched_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_source_ref: Option<String>,
}

#[derive(Debug, Serialize)]
struct BatchCounts {
    requested: usize,
    matched: usize,
    failed: usize,
}

pub(crate) fn run_record_get(options: RecordGetOptions) -> Result<ExitCode, String> {
    if options.keys.len() > MAX_GET_KEYS {
        return invalid_input(
            options.json,
            format!("record get accepts at most {MAX_GET_KEYS} keys"),
        );
    }
    let mut keys = Vec::new();
    for key in &options.keys {
        match RecordKey::parse(key) {
            Ok(parsed) => keys.push(parsed),
            Err(error) => {
                return invalid_record_key(options.json, key, error.to_string());
            }
        }
    }
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
    let records = match service.get_records(&keys) {
        Ok(records) => records,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let by_key = records
        .into_iter()
        .map(|record| (record.key.to_string(), record))
        .collect::<BTreeMap<_, _>>();
    let record_options = RecordJsonOptions {
        detail: options.detail,
        include_source_json: options.include_raw,
    };

    if keys.len() == 1 {
        let key = keys[0].to_string();
        if let Some(record) = by_key.get(&key) {
            let data = RecordGetData {
                detail: options.detail.to_string(),
                body: SingleRecordBody {
                    record: record_json(record, record_options),
                },
            };
            if options.json {
                write_json_data(data)?;
            } else {
                print_single_record(&data.body.record);
            }
            return Ok(ExitCode::SUCCESS);
        }
        if options.json {
            write_json_error("record_not_found", format!("record not found: {key}"))?;
        } else {
            eprintln!("record not found: {key}");
        }
        return Ok(ExitCode::from(1));
    }

    let mut failed = 0;
    let results = keys
        .iter()
        .map(|key| {
            let key_text = key.to_string();
            if let Some(record) = by_key.get(&key_text) {
                RecordGetItem {
                    key: key_text,
                    record: Some(record_json(record, record_options)),
                    error: None,
                }
            } else {
                failed += 1;
                RecordGetItem {
                    key: key_text.clone(),
                    record: None,
                    error: Some(CliError {
                        code: "record_not_found",
                        message: format!("record not found: {key_text}"),
                    }),
                }
            }
        })
        .collect::<Vec<_>>();
    let data = RecordGetData {
        detail: options.detail.to_string(),
        body: BatchRecordBody {
            counts: BatchCounts {
                requested: keys.len(),
                matched: keys.len() - failed,
                failed,
            },
            results,
        },
    };
    if options.json {
        write_json_data(&data)?;
    } else {
        print_record_get_batch(&data.body);
    }
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

pub(crate) fn run_record_resolve(options: RecordResolveOptions) -> Result<ExitCode, String> {
    if options.queries.len() > MAX_RESOLVE_QUERIES {
        return invalid_input(
            options.json,
            format!("record resolve accepts at most {MAX_RESOLVE_QUERIES} queries"),
        );
    }
    let filter = match parse_filter(options.filter_json.as_deref()) {
        Ok(filter) => filter,
        Err(error) if options.json => {
            write_json_error("invalid_filter_json", error)?;
            return Ok(ExitCode::from(2));
        }
        Err(error) => return Err(error),
    };
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
    let record_options = RecordJsonOptions {
        detail: options.detail,
        include_source_json: options.include_raw,
    };

    let mut failed = 0;
    let mut results = Vec::new();
    for query in &options.queries {
        let matches = match service.resolve_record(query, filter.as_ref()) {
            Ok(matches) => matches,
            Err(error) if options.json => {
                write_json_error(search_error_code(&error), error.to_string())?;
                return Ok(ExitCode::from(3));
            }
            Err(error) => return Err(search_error(error)),
        };
        let item = resolve_item(query, matches, record_options, options.alternatives);
        if item.error.is_some() {
            failed += 1;
        }
        results.push(item);
    }

    if results.len() == 1 {
        let result = results.into_iter().next().expect("single result");
        if let Some(error) = result.error.as_ref() {
            if result.alternatives.is_some() {
                let data = RecordResolveData {
                    detail: options.detail.to_string(),
                    body: SingleResolveBody { result },
                };
                if options.json {
                    write_json_data(&data)?;
                } else {
                    print_single_resolve(&data.body.result);
                }
                return Ok(ExitCode::from(1));
            }
            if options.json {
                write_json_error(error.code, error.message.clone())?;
            } else {
                eprintln!("{}", error.message);
            }
            return Ok(ExitCode::from(1));
        }
        let data = RecordResolveData {
            detail: options.detail.to_string(),
            body: SingleResolveBody { result },
        };
        if options.json {
            write_json_data(&data)?;
        } else {
            print_single_resolve(&data.body.result);
        }
        return Ok(ExitCode::SUCCESS);
    }

    let data = RecordResolveData {
        detail: options.detail.to_string(),
        body: BatchResolveBody {
            counts: BatchCounts {
                requested: results.len(),
                matched: results.len() - failed,
                failed,
            },
            results,
        },
    };
    if options.json {
        write_json_data(&data)?;
    } else {
        print_resolve_batch(&data.body);
    }
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn resolve_item(
    query: &str,
    matches: Vec<RecordResolutionResult>,
    record_options: RecordJsonOptions,
    alternatives: u8,
) -> RecordResolveItem {
    if matches.is_empty() {
        return RecordResolveItem {
            query: query.to_string(),
            record: None,
            resolution: None,
            alternatives: None,
            error: Some(CliError {
                code: "record_resolution_miss",
                message: format!("record resolution miss: {query}"),
            }),
        };
    }
    if matches.len() > 1 {
        return RecordResolveItem {
            query: query.to_string(),
            record: None,
            resolution: None,
            alternatives: Some(
                matches
                    .iter()
                    .take(alternatives as usize)
                    .map(|resolution| RecordResolveAlternative {
                        record: record_json(&resolution.record, record_options),
                        resolution: resolution_json(resolution, record_options),
                    })
                    .collect::<Vec<_>>(),
            )
            .filter(|alternatives| !alternatives.is_empty()),
            error: Some(CliError {
                code: "record_resolution_ambiguous",
                message: format!("record resolution ambiguous: {query}"),
            }),
        };
    }
    let resolution = matches.into_iter().next().expect("one match");
    RecordResolveItem {
        query: query.to_string(),
        record: Some(record_json(&resolution.record, record_options)),
        resolution: Some(resolution_json(&resolution, record_options)),
        alternatives: None,
        error: None,
    }
}

fn resolution_json(
    resolution: &RecordResolutionResult,
    record_options: RecordJsonOptions,
) -> RecordResolutionJson {
    let full = record_options.detail == atlas_domain::DetailLevel::Full;
    RecordResolutionJson {
        query: resolution.query.clone(),
        normalized_query: resolution.normalized_query.clone(),
        match_kind: resolution.match_kind.as_str(),
        matched_text: resolution.matched_text.clone(),
        alias_source: full.then(|| resolution.alias_source.clone()).flatten(),
        alias_source_ref: full.then(|| resolution.alias_source_ref.clone()).flatten(),
    }
}

fn parse_filter(filter_json: Option<&str>) -> Result<Option<SearchFilterNode>, String> {
    filter_json
        .map(|filter_json| {
            serde_json::from_str::<SearchFilterNode>(filter_json)
                .map_err(|error| format!("failed to parse --filter-json: {error}"))
        })
        .transpose()
}

fn print_single_record(record: &atlas_record::RecordJson) {
    println!("{}\t{}\t{}", record.key, record.name, record.record_family);
}

fn print_record_get_batch(batch: &BatchRecordBody) {
    println!(
        "matched {}/{} records",
        batch.counts.matched, batch.counts.requested
    );
    for result in &batch.results {
        if let Some(record) = &result.record {
            print_single_record(record);
        } else if let Some(error) = &result.error {
            eprintln!("{}\t{}", result.key, error.message);
        }
    }
}

fn print_single_resolve(result: &RecordResolveItem) {
    if let Some(record) = &result.record {
        let match_kind = result
            .resolution
            .as_ref()
            .map(|resolution| resolution.match_kind)
            .unwrap_or("unknown");
        println!("{}\t{}\t{}", record.key, record.name, match_kind);
        return;
    }
    if let Some(error) = &result.error {
        eprintln!("{}", error.message);
    }
    if let Some(alternatives) = &result.alternatives {
        for alternative in alternatives {
            println!(
                "{}\t{}\t{}",
                alternative.record.key, alternative.record.name, alternative.resolution.match_kind
            );
        }
    }
}

fn print_resolve_batch(batch: &BatchResolveBody) {
    println!(
        "matched {}/{} queries",
        batch.counts.matched, batch.counts.requested
    );
    for result in &batch.results {
        print_single_resolve(result);
    }
}

fn invalid_record_key(json: bool, key: &str, message: String) -> Result<ExitCode, String> {
    if json {
        write_json_error(
            "invalid_record_key",
            format!("invalid record key `{key}`: {message}"),
        )?;
        Ok(ExitCode::from(2))
    } else {
        Err(format!("invalid record key `{key}`: {message}"))
    }
}

fn invalid_input(json: bool, message: String) -> Result<ExitCode, String> {
    if json {
        write_json_error("invalid_input", message)?;
        Ok(ExitCode::from(2))
    } else {
        Err(message)
    }
}

fn record_runtime(
    path_mode: atlas_runtime::AtlasPathMode,
    index: Option<std::path::PathBuf>,
) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })
}

fn open_record_service(
    runtime: &AtlasRuntime,
) -> Result<atlas_search::AtlasRetrievalService, String> {
    runtime
        .open_record_retrieval_service()
        .map_err(|error| error.to_string())
}

fn search_error(error: SearchError) -> String {
    error.to_string()
}

fn search_error_code(error: &SearchError) -> &'static str {
    match error {
        SearchError::Index(atlas_index::IndexValidationError::Unavailable(_)) => {
            "index_unavailable"
        }
        SearchError::Index(atlas_index::IndexValidationError::InvalidArtifact(_)) => {
            "artifact_contract_violation"
        }
        SearchError::Filter(_) => "invalid_filter",
        _ => "query_failed",
    }
}
