use std::process::ExitCode;
use std::time::Instant;

use atlas_domain::SearchFilterNode;
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{AtlasSearchRequest, AtlasSearchResult, SemanticSearchMode};
use serde_json::json;

use crate::SemanticSearchOptions;

pub(crate) fn run_search_semantic(options: SemanticSearchOptions) -> Result<ExitCode, String> {
    let total_started_at = Instant::now();
    let filter = options
        .filter_json
        .as_deref()
        .map(|filter_json| {
            serde_json::from_str::<SearchFilterNode>(filter_json)
                .map_err(|error| format!("failed to parse --filter-json: {error}"))
        })
        .transpose()?;

    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })?;
    let service_open_started_at = Instant::now();
    let mut search = runtime
        .open_retrieval_service_with_model(options.embedding_model.to_string())
        .map_err(|error| error.to_string())?;
    let service_open_duration_ms = service_open_started_at.elapsed().as_millis();
    let semantic_mode = SemanticSearchMode::from(options.semantic_mode);
    let AtlasSearchResult::Semantic(result) = search
        .search(AtlasSearchRequest::Semantic {
            query: &options.query,
            filter: filter.as_ref(),
            limit: options.limit,
            mode: semantic_mode,
        })
        .map_err(|error| error.to_string())?;
    let hits = result.hits;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ok",
                "query": options.query,
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
                    json!({
                        "record_key": hit.record_key,
                        "embedding_unit_key": hit.embedding_unit_key,
                        "unit_kind": hit.unit_kind,
                        "label": hit.label,
                        "distance": hit.distance,
                        "rank_distance": hit.rank_distance,
                    })
                }).collect::<Vec<_>>()
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("ok: {} semantic hits", hits.len());
        for hit in hits {
            println!("{}\t{}", hit.record_key, hit.distance);
        }
    }

    Ok(ExitCode::SUCCESS)
}
