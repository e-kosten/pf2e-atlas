use atlas_domain::SearchFilterNode;
use atlas_index::FilteredRecordSort;
use atlas_record::{RecordJsonOptions, record_json};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{AtlasSearchRequest, AtlasSearchResult, SemanticSearchMode};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Instant;

use crate::SearchOptions;
use crate::output::{write_json_data, write_json_error};

const MAX_LIMIT: u32 = 100;

#[derive(Debug, Serialize)]
struct SearchData {
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Value>,
    sort: SearchSortJson,
    pagination: SearchPagination,
    results: Vec<SearchResultItem>,
}

#[derive(Debug, Serialize)]
struct SearchSortJson {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Debug, Serialize)]
struct SearchPagination {
    offset: u32,
    limit: u32,
    count: usize,
    total: u64,
    has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_offset: Option<u32>,
}

#[derive(Debug, Serialize)]
struct SearchResultItem {
    record: atlas_record::RecordJson,
    r#match: SearchMatchJson,
}

#[derive(Debug, Serialize)]
struct SearchMatchJson {
    kind: &'static str,
}

pub(crate) fn run_search(options: SearchOptions) -> Result<ExitCode, String> {
    if options.query.as_deref() == Some("semantic") {
        return run_search_semantic_diagnostic(options);
    }
    if options.query_text.is_some() {
        if options.json {
            write_json_error(
                "invalid_input",
                "--query is only supported with `atlas search semantic` in this phase".to_string(),
            )?;
            return Ok(ExitCode::from(2));
        }
        return Err(
            "--query is only supported with `atlas search semantic` in this phase".to_string(),
        );
    }

    let filter_value = options
        .filter_json
        .as_deref()
        .map(|filter_json| {
            serde_json::from_str::<Value>(filter_json).map_err(|error| {
                if options.json {
                    let _ = write_json_error(
                        "invalid_filter_json",
                        format!("failed to parse --filter-json: {error}"),
                    );
                }
                format!("failed to parse --filter-json: {error}")
            })
        })
        .transpose();
    let filter_value = match filter_value {
        Ok(filter_value) => filter_value,
        Err(_) if options.json => return Ok(ExitCode::from(2)),
        Err(error) => return Err(error),
    };
    if options.query.is_some() {
        if options.json {
            write_json_error(
                "unsupported_search_mode",
                "ranked text search is not implemented in the Rust CLI yet".to_string(),
            )?;
            return Ok(ExitCode::from(2));
        }
        return Err("ranked text search is not implemented in the Rust CLI yet".to_string());
    }

    let limit = options.limit.clamp(1, MAX_LIMIT);
    let filter = options
        .filter_json
        .as_deref()
        .map(|filter_json| {
            serde_json::from_str::<SearchFilterNode>(filter_json).map_err(|error| {
                if options.json {
                    let _ = write_json_error(
                        "invalid_filter_json",
                        format!("failed to parse --filter-json: {error}"),
                    );
                }
                format!("failed to parse --filter-json: {error}")
            })
        })
        .transpose();
    let filter = match filter {
        Ok(filter) => filter,
        Err(_) if options.json => return Ok(ExitCode::from(2)),
        Err(error) => return Err(error),
    };
    let (sort, sort_json) = match parse_sort(&options.sort, options.seed) {
        Ok(sort) => sort,
        Err(error) if options.json => {
            write_json_error("invalid_option", error)?;
            return Ok(ExitCode::from(2));
        }
        Err(error) => return Err(error),
    };
    let runtime = match AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: options.index,
        },
    }) {
        Ok(runtime) => runtime,
        Err(error) if options.json => {
            write_json_error("runtime_error", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let service = match runtime.open_record_retrieval_service() {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error("index_unavailable", error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let page = match service.filter_only_records(filter.as_ref(), sort, limit, options.offset) {
        Ok(page) => page,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let by_key = page
        .records
        .into_iter()
        .map(|record| (record.key.to_string(), record))
        .collect::<BTreeMap<_, _>>();
    let record_options = RecordJsonOptions {
        detail: options.detail,
        include_source_json: options.include_raw,
    };
    let results = page
        .record_keys
        .iter()
        .filter_map(|key| by_key.get(&key.to_string()))
        .map(|record| SearchResultItem {
            record: record_json(record, record_options),
            r#match: SearchMatchJson { kind: "filter" },
        })
        .collect::<Vec<_>>();
    let next_offset = options.offset.saturating_add(results.len() as u32);
    let has_more = u64::from(next_offset) < page.total;

    let data = SearchData {
        detail: options.detail.to_string(),
        filter: filter_value,
        sort: sort_json,
        pagination: SearchPagination {
            offset: options.offset,
            limit,
            count: results.len(),
            total: page.total,
            has_more,
            next_offset: has_more.then_some(next_offset),
        },
        results,
    };
    if options.json {
        write_json_data(&data)?;
    } else {
        print_search_results(&data);
    }
    Ok(ExitCode::SUCCESS)
}

fn run_search_semantic_diagnostic(options: SearchOptions) -> Result<ExitCode, String> {
    let query = match options.query_text.as_deref() {
        Some(query) => query,
        None if options.json => {
            write_json_error(
                "invalid_input",
                "missing --query for semantic search".to_string(),
            )?;
            return Ok(ExitCode::from(2));
        }
        None => return Err("missing --query for semantic search".to_string()),
    };
    let total_started_at = Instant::now();
    let filter = options
        .filter_json
        .as_deref()
        .map(|filter_json| {
            serde_json::from_str::<SearchFilterNode>(filter_json).map_err(|error| {
                if options.json {
                    let _ = write_json_error(
                        "invalid_filter_json",
                        format!("failed to parse --filter-json: {error}"),
                    );
                }
                format!("failed to parse --filter-json: {error}")
            })
        })
        .transpose();
    let filter = match filter {
        Ok(filter) => filter,
        Err(_) if options.json => return Ok(ExitCode::from(2)),
        Err(error) => return Err(error),
    };

    let runtime = match AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    }) {
        Ok(runtime) => runtime,
        Err(error) if options.json => {
            write_json_error("runtime_error", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let service_open_started_at = Instant::now();
    let mut search =
        match runtime.open_retrieval_service_with_model(options.embedding_model.to_string()) {
            Ok(search) => search,
            Err(error) if options.json => {
                write_json_error("index_unavailable", error.to_string())?;
                return Ok(ExitCode::from(3));
            }
            Err(error) => return Err(error.to_string()),
        };
    let service_open_duration_ms = service_open_started_at.elapsed().as_millis();
    let semantic_mode = SemanticSearchMode::from(options.semantic_mode);
    let result = match search.search(AtlasSearchRequest::Semantic {
        query,
        filter: filter.as_ref(),
        limit: options.limit,
        mode: semantic_mode,
    }) {
        Ok(AtlasSearchResult::Semantic(result)) => result,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let hits = result.hits;

    if options.json {
        write_json_data(serde_json::json!({
            "query": query,
            "limit": options.limit,
            "semantic_mode": semantic_mode.as_str(),
            "timing": {
                "service_open_duration_ms": service_open_duration_ms,
                "query_embedding_duration_ms": result.timing.query_embedding_duration_ms,
                "vector_search_duration_ms": result.timing.vector_search_duration_ms,
                "semantic_duration_ms": result.timing.total_duration_ms,
                "total_duration_ms": total_started_at.elapsed().as_millis(),
            },
            "hits": hits.iter().map(|hit| {
                serde_json::json!({
                    "record_key": hit.record_key,
                    "embedding_unit_key": hit.embedding_unit_key,
                    "unit_kind": hit.unit_kind,
                    "label": hit.label,
                    "distance": hit.distance,
                    "rank_distance": hit.rank_distance,
                })
            }).collect::<Vec<_>>()
        }))?;
    } else {
        println!("ok: {} semantic hits", hits.len());
        for hit in hits {
            println!("{}\t{}", hit.record_key, hit.distance);
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn parse_sort(
    value: &str,
    seed: Option<u64>,
) -> Result<(FilteredRecordSort, SearchSortJson), String> {
    match value {
        "alphabetical" => Ok((
            FilteredRecordSort::Alphabetical,
            SearchSortJson {
                kind: "alphabetical",
                seed: None,
            },
        )),
        "level_asc" => Ok((
            FilteredRecordSort::LevelAsc,
            SearchSortJson {
                kind: "level_asc",
                seed: None,
            },
        )),
        "level_desc" => Ok((
            FilteredRecordSort::LevelDesc,
            SearchSortJson {
                kind: "level_desc",
                seed: None,
            },
        )),
        "record_key" => Ok((
            FilteredRecordSort::RecordKey,
            SearchSortJson {
                kind: "record_key",
                seed: None,
            },
        )),
        "random" => {
            let seed = seed.unwrap_or_else(generated_seed);
            Ok((
                FilteredRecordSort::Random { seed },
                SearchSortJson {
                    kind: "random",
                    seed: Some(seed),
                },
            ))
        }
        _ => Err(format!("unsupported --sort value `{value}`")),
    }
}

fn generated_seed() -> u64 {
    rand::random()
}

fn print_search_results(data: &SearchData) {
    println!(
        "showing {} of {} records",
        data.pagination.count, data.pagination.total
    );
    for result in &data.results {
        println!(
            "{}\t{}\t{}",
            result.record.key, result.record.name, result.record.record_family
        );
    }
}

fn search_error_code(error: &atlas_search::SearchError) -> &'static str {
    match error {
        atlas_search::SearchError::Index(atlas_index::IndexValidationError::Unavailable(_)) => {
            "index_unavailable"
        }
        atlas_search::SearchError::Index(atlas_index::IndexValidationError::InvalidArtifact(_)) => {
            "artifact_contract_violation"
        }
        atlas_search::SearchError::Filter(_) => "invalid_filter",
        _ => "query_failed",
    }
}
