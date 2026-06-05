use atlas_domain::{DetailLevel, SearchFilterNode};
use atlas_record::{RecordJsonOptions, record_json};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{
    ListRecordsRequest, RecordListSort, RecordRetrieval, RetrievalMode, SearchError,
    SearchErrorKind, SearchPage, SearchPageInfo, TextRetrieval, TextSearchMatch, TextSearchRequest,
    TextSearchTuning, expert::DEFAULT_FTS_FUSION_POLICY_NAME,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::ExitCode;
use tracing::info;

use crate::output::{write_json_data, write_json_error};
use crate::terminal::TerminalStyle;

pub(crate) mod args;

use args::{CliRetrievalMode, CliSearchSort, SearchOptions};

use super::filters::build_filter;
use super::record::{detail_outputs_description, print_record_for_detail};

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
    page: u32,
    limit: u32,
    count: usize,
    total: u64,
    has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_page: Option<u32>,
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

    let page = match SearchPage::new(options.page, options.limit) {
        Ok(page) => page,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(2));
        }
        Err(error) => return Err(error.to_string()),
    };

    if let Some(query) = options.query.clone() {
        return run_ranked_search_text(options, &query, filter.as_ref(), filter_value, page);
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
            write_json_error("runtime_error", error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => {
            complete_search_progress();
            return Err(error.to_string());
        }
    };
    let service = match runtime.open_retrieval_service_no_embeddings() {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error("index_unavailable", error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let list_result = match service.list_records(ListRecordsRequest {
        filter: filter.as_ref(),
        sort,
        page,
    }) {
        Ok(list_result) => list_result,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let by_key = list_result
        .records
        .into_iter()
        .map(|record| (record.identity.key.to_string(), record))
        .collect::<BTreeMap<_, _>>();
    let record_options = RecordJsonOptions {
        detail: options.detail,
        include_source_json: options.include_raw,
    };
    let results = list_result
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

    let data = SearchData {
        detail: options.detail.to_string(),
        query: None,
        query_analysis: None,
        retrieval: None,
        fusion: None,
        candidate_windows: None,
        filter: filter_value,
        sort: sort_json,
        pagination: search_pagination_json(list_result.page),
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
    page: SearchPage,
) -> Result<ExitCode, String> {
    let request_tuning = text_search_tuning_from_options(&options);
    let resolved_tuning = match TextSearchTuning::resolve_for_page(request_tuning, page) {
        Ok(tuning) => tuning,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(2));
        }
        Err(error) => return Err(error.to_string()),
    };
    let explicit_fts = matches!(options.retrieval, Some(CliRetrievalMode::Fts));
    let retrieval = resolved_tuning.retrieval();
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
            write_json_error("runtime_error", error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let mut search = if explicit_fts {
        search_progress("Opening index", "open-index");
        match runtime.open_retrieval_service_no_embeddings() {
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
        match runtime.open_retrieval_service() {
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
    let result = match search.search_text(TextSearchRequest {
        query,
        exclude: options.exclude.as_deref(),
        filter,
        page,
        tuning: request_tuning,
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
    let data = SearchData {
        detail: options.detail.to_string(),
        query: Some(query.to_string()),
        query_analysis: result
            .diagnostics
            .map(|diagnostics| SearchQueryAnalysisJson {
                normalized_query: diagnostics.query.normalized_query,
                fts_query: diagnostics.query.fts_query,
                fts_tokens: diagnostics.query.fts_tokens,
                exclude_query: diagnostics.query.exclude_query,
                exclude_tokens: diagnostics.query.exclude_tokens,
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
            fts_top_k: resolved_tuning.fts_top_k(),
            vector_top_k: resolved_tuning.vector_top_k(),
        }),
        filter: filter_value,
        sort: SearchSortJson {
            kind: "ranked",
            seed: None,
        },
        pagination: search_pagination_json(result.page),
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

fn text_search_tuning_from_options(options: &SearchOptions) -> Option<TextSearchTuning> {
    if options.retrieval.is_none()
        && options.fusion.is_none()
        && options.fts_weight.is_none()
        && options.vector_weight.is_none()
        && options.rank_constant.is_none()
        && options.fts_top_k.is_none()
        && options.vector_top_k.is_none()
    {
        return None;
    }

    let mut tuning = TextSearchTuning::default();
    if let Some(retrieval) = options.retrieval {
        tuning = tuning.with_retrieval(retrieval.into());
    }
    if let Some(method) = options.fusion {
        tuning = tuning.with_fusion_method(method.into());
    }
    if let Some(weight) = options.fts_weight {
        tuning = tuning.with_fts_weight(weight);
    }
    if let Some(weight) = options.vector_weight {
        tuning = tuning.with_vector_weight(weight);
    }
    if let Some(rank_constant) = options.rank_constant {
        tuning = tuning.with_rank_constant(rank_constant);
    }
    if let Some(fts_top_k) = options.fts_top_k {
        tuning = tuning.with_fts_top_k(fts_top_k);
    }
    if let Some(vector_top_k) = options.vector_top_k {
        tuning = tuning.with_vector_top_k(vector_top_k);
    }

    Some(tuning)
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
) -> Result<(RecordListSort, SearchSortJson), String> {
    match value {
        CliSearchSort::Alphabetical => Ok((
            RecordListSort::Alphabetical,
            SearchSortJson {
                kind: "alphabetical",
                seed: None,
            },
        )),
        CliSearchSort::LevelAsc => Ok((
            RecordListSort::LevelAsc,
            SearchSortJson {
                kind: "level_asc",
                seed: None,
            },
        )),
        CliSearchSort::LevelDesc => Ok((
            RecordListSort::LevelDesc,
            SearchSortJson {
                kind: "level_desc",
                seed: None,
            },
        )),
        CliSearchSort::PriceAsc => Ok((
            RecordListSort::PriceAsc,
            SearchSortJson {
                kind: "price_asc",
                seed: None,
            },
        )),
        CliSearchSort::PriceDesc => Ok((
            RecordListSort::PriceDesc,
            SearchSortJson {
                kind: "price_desc",
                seed: None,
            },
        )),
        CliSearchSort::RecordKey => Ok((
            RecordListSort::RecordKey,
            SearchSortJson {
                kind: "record_key",
                seed: None,
            },
        )),
        CliSearchSort::Random => {
            let seed = seed.unwrap_or_else(generated_seed);
            Ok((
                RecordListSort::Random { seed },
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

fn search_pagination_json(page: SearchPageInfo) -> SearchPagination {
    SearchPagination {
        page: page.number,
        limit: page.size,
        count: page.count,
        total: page.total,
        has_more: page.has_more,
        next_page: page.next_page,
    }
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
    let kind_width = data
        .results
        .iter()
        .map(|result| result.record.kind.len())
        .max()
        .unwrap_or("kind".len());
    println!(
        "{:<key_width$}  {:<kind_width$}  name",
        "key",
        "kind",
        key_width = key_width,
        kind_width = kind_width
    );
    println!(
        "{:-<key_width$}  {:-<kind_width$}  {:-<4}",
        "",
        "",
        "",
        key_width = key_width,
        kind_width = kind_width
    );
    for result in &data.results {
        println!(
            "{:<key_width$}  {:<kind_width$}  {}",
            result.record.key,
            result.record.kind,
            result.record.name,
            key_width = key_width,
            kind_width = kind_width
        );
    }
}

fn search_match_json(match_info: TextSearchMatch) -> SearchMatchJson {
    match match_info {
        TextSearchMatch::Identity {
            retrieval,
            identity_match_kind,
            diagnostics,
        } => SearchMatchJson {
            kind: "identity",
            retrieval: Some(retrieval.as_str()),
            identity_match_kind: Some(identity_match_kind.as_str()),
            explain: diagnostics.map(search_explain_json),
        },
        TextSearchMatch::Ranked {
            retrieval,
            diagnostics,
        } => SearchMatchJson {
            kind: "ranked",
            retrieval: Some(retrieval.as_str()),
            identity_match_kind: None,
            explain: diagnostics.map(search_explain_json),
        },
    }
}

fn search_explain_json(explain: atlas_search::TextSearchMatchDiagnostics) -> SearchExplainJson {
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
        fts_lane: explain.fts_lane,
        fts_confidence: explain.fts_confidence,
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
