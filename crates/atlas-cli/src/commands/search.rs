use atlas_domain::{DetailLevel, SearchFilterNode};
use atlas_record::{RecordJsonOptions, record_json};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{
    BrowseRecordsRequest, DEFAULT_FTS_FUSION_POLICY_NAME, FusionOptions, RecordBrowseSort,
    RecordRetrieval, RetrievalMode, SearchError, SearchErrorKind, TextRetrieval, TextSearchExplain,
    TextSearchMatch, TextSearchRequest,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::ExitCode;
use tracing::info;

use crate::output::{write_json_data, write_json_error};
use crate::terminal::TerminalStyle;

pub(crate) mod args;

use args::{CliFusionMethod, CliSearchSort, SearchOptions};

use super::filters::build_filter;
use super::record::{detail_outputs_description, print_record_for_detail};

const MAX_LIMIT: u32 = 100;
const MAX_RANKED_CANDIDATE_WINDOW: u32 = 5_000;

#[derive(Debug, Serialize)]
struct SearchData {
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_analysis: Option<SearchQueryAnalysisJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retrieval: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fusion: Option<SearchFusionJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_windows: Option<SearchCandidateWindowsJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Value>,
    sort: SearchSortJson,
    pagination: SearchPagination,
    results: Vec<SearchResultItem>,
}

#[derive(Debug, Serialize)]
struct SearchFusionJson {
    method: &'static str,
    fts_weight: f64,
    vector_weight: f64,
    rank_constant: f64,
    fts_policy: &'static str,
}

#[derive(Debug, Serialize)]
struct SearchQueryAnalysisJson {
    normalized_query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fts_query: Option<String>,
    fts_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_query: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exclude_tokens: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SearchCandidateWindowsJson {
    fts_top_k: u32,
    vector_top_k: u32,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    retrieval: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity_match_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    explain: Option<SearchExplainJson>,
}

#[derive(Debug, Serialize)]
struct SearchExplainJson {
    rank: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    fusion: Option<SearchFusionExplainJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fts: Option<SearchFtsExplainJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector: Option<SearchVectorExplainJson>,
}

#[derive(Debug, Serialize)]
struct SearchFusionExplainJson {
    fused_score: Option<f64>,
}

#[derive(Debug, Serialize)]
struct SearchFtsExplainJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    fts_rank: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fts_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fts_lane: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fts_confidence: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct SearchVectorExplainJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    vector_rank: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector_distance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector_rank_distance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector_unit_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector_label: Option<String>,
}

pub(crate) fn run_search(options: SearchOptions) -> Result<ExitCode, String> {
    let limit = options.limit.clamp(1, MAX_LIMIT);
    let (filter, filter_value) =
        match build_filter(options.filter_json.as_deref(), &options.filter_options) {
            Ok(filter) => filter,
            Err(error) if options.json => {
                write_json_error(error.code, error.message)?;
                return Ok(ExitCode::from(2));
            }
            Err(error) => return Err(error.message),
        };
    if options.print_filter {
        let filter_value = filter_value.unwrap_or(Value::Null);
        if options.json {
            write_json_data(serde_json::json!({ "filter": filter_value }))?;
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&filter_value)
                    .map_err(|error| format!("failed to render filter JSON: {error}"))?
            );
        }
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(query) = options.query.clone() {
        return run_ranked_search_text(options, &query, filter.as_ref(), filter_value, limit);
    }

    let (sort, sort_json) = match parse_sort(options.sort, options.seed) {
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
            complete_search_progress();
            write_json_error("runtime_error", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => {
            complete_search_progress();
            return Err(error);
        }
    };
    let service = match runtime.open_record_retrieval_service() {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error("index_unavailable", error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let page = match service.browse_records(BrowseRecordsRequest {
        filter: filter.as_ref(),
        sort,
        limit,
        offset: options.offset,
    }) {
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
            r#match: SearchMatchJson {
                kind: "filter",
                retrieval: None,
                identity_match_kind: None,
                explain: None,
            },
        })
        .collect::<Vec<_>>();
    let next_offset = options.offset.saturating_add(results.len() as u32);
    let has_more = u64::from(next_offset) < page.total;

    let data = SearchData {
        detail: options.detail.to_string(),
        query: None,
        query_analysis: None,
        retrieval: None,
        fusion: None,
        candidate_windows: None,
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

fn run_ranked_search_text(
    options: SearchOptions,
    query: &str,
    filter: Option<&SearchFilterNode>,
    filter_value: Option<Value>,
    limit: u32,
) -> Result<ExitCode, String> {
    let retrieval = RetrievalMode::from(options.retrieval);
    let fusion = FusionOptions {
        method: options.fusion.into(),
        fts_weight: options.fts_weight,
        vector_weight: options.vector_weight,
        rank_constant: options.rank_constant,
    };
    if options.fusion == CliFusionMethod::Rrf
        && ((options.fts_weight - 1.0).abs() > f64::EPSILON
            || (options.vector_weight - 1.0).abs() > f64::EPSILON)
    {
        let message =
            "--fusion rrf does not accept lane weights; use --fusion weighted-rrf".to_string();
        if options.json {
            write_json_error("invalid_option", message)?;
            return Ok(ExitCode::from(2));
        }
        return Err(message);
    }
    if fusion.rank_constant <= 0.0 || fusion.fts_weight < 0.0 || fusion.vector_weight < 0.0 {
        let message =
            "fusion weights must be non-negative and --rank-constant must be greater than zero"
                .to_string();
        if options.json {
            write_json_error("invalid_option", message)?;
            return Ok(ExitCode::from(2));
        }
        return Err(message);
    }
    let ranked_window = options.offset.saturating_add(limit);
    if ranked_window > MAX_RANKED_CANDIDATE_WINDOW
        || options.fts_top_k > MAX_RANKED_CANDIDATE_WINDOW
        || options.vector_top_k > MAX_RANKED_CANDIDATE_WINDOW
    {
        let message = format!(
            "ranked search candidate windows must be at most {MAX_RANKED_CANDIDATE_WINDOW}; got --offset {}, --limit {limit}, --fts-top-k {}, --vector-top-k {}",
            options.offset, options.fts_top_k, options.vector_top_k
        );
        if options.json {
            write_json_error("invalid_option", message)?;
            return Ok(ExitCode::from(2));
        }
        return Err(message);
    }
    search_progress("Resolving Atlas paths", "resolve");
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
    let mut search = if retrieval == RetrievalMode::Fts {
        search_progress("Opening index", "open-index");
        match runtime.open_record_retrieval_service() {
            Ok(search) => search,
            Err(error) if options.json => {
                complete_search_progress();
                write_json_error("index_unavailable", error.to_string())?;
                return Ok(ExitCode::from(3));
            }
            Err(error) => {
                complete_search_progress();
                return Err(error.to_string());
            }
        }
    } else {
        search_progress("Loading embedding model", "load-embeddings");
        match runtime.open_retrieval_service_with_model(options.embedding_model.to_string()) {
            Ok(search) => search,
            Err(error) if options.json && vector_readiness_error(&error) => {
                complete_search_progress();
                write_json_error(
                    "vector_readiness_required",
                    vector_readiness_message(&error),
                )?;
                return Ok(ExitCode::from(3));
            }
            Err(error) if options.json => {
                complete_search_progress();
                write_json_error(search_error_code(&error), error.to_string())?;
                return Ok(ExitCode::from(3));
            }
            Err(error) => {
                complete_search_progress();
                if vector_readiness_error(&error) {
                    return Err(vector_readiness_message(&error));
                }
                return Err(error.to_string());
            }
        }
    };
    search_progress("Searching records", "search");
    let fts_top_k = options.fts_top_k.max(ranked_window);
    let vector_top_k = options.vector_top_k.max(ranked_window);
    let result = match search.search_text(TextSearchRequest {
        query,
        exclude: options.exclude.as_deref(),
        filter,
        limit,
        offset: options.offset,
        retrieval,
        fusion,
        fts_top_k,
        vector_top_k,
        explain: options.explain,
    }) {
        Ok(result) => result,
        Err(error) if options.json => {
            complete_search_progress();
            let message = if retrieval != RetrievalMode::Fts && vector_readiness_error(&error) {
                vector_readiness_message(&error)
            } else {
                error.to_string()
            };
            write_json_error(search_error_code_for_retrieval(&error, retrieval), message)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => {
            complete_search_progress();
            if retrieval != RetrievalMode::Fts && vector_readiness_error(&error) {
                return Err(vector_readiness_message(&error));
            }
            return Err(error.to_string());
        }
    };
    search_progress("Rendering results", "render");
    let record_options = RecordJsonOptions {
        detail: options.detail,
        include_source_json: options.include_raw,
    };
    let results = result
        .records
        .into_iter()
        .map(|item| SearchResultItem {
            record: record_json(&item.record, record_options),
            r#match: search_match_json(item.match_info),
        })
        .collect::<Vec<_>>();
    let next_offset = options.offset.saturating_add(results.len() as u32);
    let has_more = u64::from(next_offset) < result.total;
    let data = SearchData {
        detail: options.detail.to_string(),
        query: Some(query.to_string()),
        query_analysis: options.explain.then_some(SearchQueryAnalysisJson {
            normalized_query: result.query.normalized_query,
            fts_query: result.query.fts_query,
            fts_tokens: result.query.fts_tokens,
            exclude_query: result.query.exclude_query,
            exclude_tokens: result.query.exclude_tokens,
        }),
        retrieval: Some(result.retrieval.as_str()),
        fusion: Some(SearchFusionJson {
            method: result.fusion.method.as_str(),
            fts_weight: result.fusion.fts_weight,
            vector_weight: result.fusion.vector_weight,
            rank_constant: result.fusion.rank_constant,
            fts_policy: DEFAULT_FTS_FUSION_POLICY_NAME,
        }),
        candidate_windows: options.explain.then_some(SearchCandidateWindowsJson {
            fts_top_k,
            vector_top_k,
        }),
        filter: filter_value,
        sort: SearchSortJson {
            kind: "ranked",
            seed: None,
        },
        pagination: SearchPagination {
            offset: options.offset,
            limit,
            count: results.len(),
            total: result.total,
            has_more,
            next_offset: has_more.then_some(next_offset),
        },
        results,
    };
    if options.json {
        complete_search_progress();
        write_json_data(&data)?;
    } else {
        complete_search_progress();
        print_search_results(&data);
    }

    Ok(ExitCode::SUCCESS)
}

fn search_progress(message: &'static str, phase: &'static str) {
    info!(target: "atlas_progress", phase, "{message}");
}

fn complete_search_progress() {
    info!(target: "atlas_progress", complete = true, "search complete");
}

fn parse_sort(
    value: CliSearchSort,
    seed: Option<u64>,
) -> Result<(RecordBrowseSort, SearchSortJson), String> {
    match value {
        CliSearchSort::Alphabetical => Ok((
            RecordBrowseSort::Alphabetical,
            SearchSortJson {
                kind: "alphabetical",
                seed: None,
            },
        )),
        CliSearchSort::LevelAsc => Ok((
            RecordBrowseSort::LevelAsc,
            SearchSortJson {
                kind: "level_asc",
                seed: None,
            },
        )),
        CliSearchSort::LevelDesc => Ok((
            RecordBrowseSort::LevelDesc,
            SearchSortJson {
                kind: "level_desc",
                seed: None,
            },
        )),
        CliSearchSort::PriceAsc => Ok((
            RecordBrowseSort::PriceAsc,
            SearchSortJson {
                kind: "price_asc",
                seed: None,
            },
        )),
        CliSearchSort::PriceDesc => Ok((
            RecordBrowseSort::PriceDesc,
            SearchSortJson {
                kind: "price_desc",
                seed: None,
            },
        )),
        CliSearchSort::RecordKey => Ok((
            RecordBrowseSort::RecordKey,
            SearchSortJson {
                kind: "record_key",
                seed: None,
            },
        )),
        CliSearchSort::Random => {
            let seed = seed.unwrap_or_else(generated_seed);
            Ok((
                RecordBrowseSort::Random { seed },
                SearchSortJson {
                    kind: "random",
                    seed: Some(seed),
                },
            ))
        }
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
    let detail = data.detail.parse().unwrap_or(DetailLevel::Summary);
    if detail_outputs_description(detail) {
        let style = TerminalStyle::stdout();
        for (index, result) in data.results.iter().enumerate() {
            if index > 0 {
                println!();
                println!("{}", style.separator());
                println!();
            }
            print_record_for_detail(&result.record, detail);
            println!("{}: {}", style.label("Match"), result.r#match.kind);
        }
        return;
    }
    let key_width = data
        .results
        .iter()
        .map(|result| result.record.key.len())
        .max()
        .unwrap_or("key".len())
        .max("key".len());
    let family_width = data
        .results
        .iter()
        .map(|result| result.record.record_family.len())
        .max()
        .unwrap_or("family".len());
    println!(
        "{:<key_width$}  {:<family_width$}  name",
        "key",
        "family",
        key_width = key_width,
        family_width = family_width
    );
    println!(
        "{:-<key_width$}  {:-<family_width$}  {:-<4}",
        "",
        "",
        "",
        key_width = key_width,
        family_width = family_width
    );
    for result in &data.results {
        println!(
            "{:<key_width$}  {:<family_width$}  {}",
            result.record.key,
            result.record.record_family,
            result.record.name,
            key_width = key_width,
            family_width = family_width
        );
    }
}

fn search_match_json(match_info: TextSearchMatch) -> SearchMatchJson {
    match match_info {
        TextSearchMatch::Identity {
            retrieval,
            identity_match_kind,
            explain,
        } => SearchMatchJson {
            kind: "identity",
            retrieval: Some(retrieval.as_str()),
            identity_match_kind: Some(identity_match_kind.as_str()),
            explain: explain.map(search_explain_json),
        },
        TextSearchMatch::Ranked { retrieval, explain } => SearchMatchJson {
            kind: "ranked",
            retrieval: Some(retrieval.as_str()),
            identity_match_kind: None,
            explain: explain.map(search_explain_json),
        },
    }
}

fn search_explain_json(explain: TextSearchExplain) -> SearchExplainJson {
    let fusion = explain
        .fused_score
        .map(|fused_score| SearchFusionExplainJson {
            fused_score: Some(fused_score),
        });
    let fts = (explain.fts_rank.is_some()
        || explain.fts_score.is_some()
        || explain.fts_lane.is_some()
        || explain.fts_confidence.is_some())
    .then_some(SearchFtsExplainJson {
        fts_rank: explain.fts_rank,
        fts_score: explain.fts_score,
        fts_lane: explain.fts_lane.map(|lane| lane.as_str()),
        fts_confidence: explain.fts_confidence.map(|confidence| confidence.as_str()),
    });
    let vector = (explain.vector_rank.is_some()
        || explain.vector_distance.is_some()
        || explain.vector_rank_distance.is_some()
        || explain.vector_unit_kind.is_some()
        || explain.vector_label.is_some())
    .then_some(SearchVectorExplainJson {
        vector_rank: explain.vector_rank,
        vector_distance: explain.vector_distance,
        vector_rank_distance: explain.vector_rank_distance,
        vector_unit_kind: explain.vector_unit_kind,
        vector_label: explain.vector_label,
    });
    SearchExplainJson {
        rank: explain.rank,
        fusion,
        fts,
        vector,
    }
}

fn search_error_code(error: &atlas_search::SearchError) -> &'static str {
    match error.kind() {
        SearchErrorKind::IndexUnavailable => "index_unavailable",
        SearchErrorKind::ArtifactContractViolation => "artifact_contract_violation",
        SearchErrorKind::InvalidFilter => "invalid_filter",
        SearchErrorKind::InvalidOptions => "invalid_option",
        SearchErrorKind::VectorReadinessRequired => "vector_readiness_required",
        SearchErrorKind::EmbeddingUnavailable | SearchErrorKind::QueryFailed => "query_failed",
    }
}

fn search_error_code_for_retrieval(error: &SearchError, retrieval: RetrievalMode) -> &'static str {
    if retrieval != RetrievalMode::Fts && vector_readiness_error(error) {
        return "vector_readiness_required";
    }
    search_error_code(error)
}

fn vector_readiness_error(error: &SearchError) -> bool {
    error.is_vector_readiness_required()
}

fn vector_readiness_message(error: &SearchError) -> String {
    format!(
        "{error}. Rebuild the Atlas artifact with embeddings or rerun the search with --retrieval fts."
    )
}

#[cfg(test)]
mod tests {
    use atlas_search::SearchError;

    use super::vector_readiness_error;

    #[test]
    fn invalid_search_options_are_not_vector_readiness_errors() {
        let error = SearchError::query_failed("fixture");

        assert!(!vector_readiness_error(&error));
    }

    #[test]
    fn embedding_failures_are_vector_readiness_errors() {
        let error = SearchError::vector_readiness_required("missing model");

        assert!(vector_readiness_error(&error));
    }
}
