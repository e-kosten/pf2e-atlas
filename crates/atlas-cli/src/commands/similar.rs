use std::process::ExitCode;

use atlas_domain::{DetailLevel, RecordKey, SearchFilterNode};
use atlas_record::{RecordJsonOptions, record_json};
use atlas_runtime::AtlasRuntime;
use atlas_search::{
    RecordResolutionResult, SearchError, SimilarRecordRequest, SimilarRecordResult,
    SimilarScoreWeights,
};
use serde::Serialize;
use serde_json::Value;

use crate::commands::filters::build_filter;
use crate::commands::record::{
    detail_outputs_description, print_record_for_detail, record_runtime, search_error,
    search_error_code,
};
use crate::output::{write_json_data, write_json_error};
use crate::{CliIndexBackend, SimilarOptions};

#[derive(Debug, Serialize)]
struct SimilarData {
    detail: String,
    seed: atlas_record::RecordJson,
    seed_embedding_unit_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Value>,
    results: Vec<SimilarResultJson>,
}

#[derive(Debug, Serialize)]
struct SimilarResultJson {
    record: atlas_record::RecordJson,
    similarity: SimilarityJson,
}

#[derive(Debug, Serialize)]
struct SimilarityJson {
    score: f64,
    semantic: SimilarSemanticJson,
    graph: SimilarGraphJson,
}

#[derive(Debug, Serialize)]
struct SimilarSemanticJson {
    embedding_unit_key: String,
    unit_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    distance: f64,
    rank_distance: f64,
}

#[derive(Debug, Serialize)]
struct SimilarGraphJson {
    shared_mechanics: Vec<SimilarSharedMechanicJson>,
    shared_traits: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SimilarSharedMechanicJson {
    key: String,
    name: String,
}

pub(crate) fn run_similar(options: SimilarOptions) -> Result<ExitCode, String> {
    let (filter, filter_value) =
        match build_filter(options.filter_json.as_deref(), &options.filter_options) {
            Ok(filter) => filter,
            Err(error) if options.json => {
                write_json_error(error.code, error.message)?;
                return Ok(ExitCode::from(2));
            }
            Err(error) => return Err(error.message),
        };
    let runtime = match record_runtime(
        options.path_mode.into(),
        options.index,
        options.ladybug_index,
    ) {
        Ok(runtime) => runtime,
        Err(error) if options.json => {
            write_json_error("runtime_error", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let service = match open_similar_service(&runtime, options.index_backend) {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error("index_unavailable", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let seed = match resolve_seed(&service, &options.record_ref, filter.as_ref()) {
        Ok(Some(seed)) => seed,
        Ok(None) if options.json => {
            write_json_error(
                "record_resolution_miss",
                format!("record resolution miss: {}", options.record_ref),
            )?;
            return Ok(ExitCode::from(1));
        }
        Ok(None) => {
            eprintln!("record resolution miss: {}", options.record_ref);
            return Ok(ExitCode::from(1));
        }
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let result = match service.similar_records(SimilarRecordRequest {
        seed: &seed,
        filter: filter.as_ref(),
        limit: options.limit,
        candidate_limit: options.candidates,
        load_options: if options.include_raw {
            atlas_index::RecordLoadOptions::include_raw_json()
        } else {
            atlas_index::RecordLoadOptions::omit_raw_json()
        },
        weights: SimilarScoreWeights {
            semantic: options.semantic_weight,
            mechanic: options.mechanic_weight,
            traits: options.trait_weight,
        },
    }) {
        Ok(Some(result)) => result,
        Ok(None) if options.json => {
            write_json_error("record_not_found", format!("record not found: {seed}"))?;
            return Ok(ExitCode::from(1));
        }
        Ok(None) => {
            eprintln!("record not found: {seed}");
            return Ok(ExitCode::from(1));
        }
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let data = similar_data(&result, options.detail, options.include_raw, filter_value);
    if options.json {
        write_json_data(data)?;
    } else {
        print_similar(&data, options.detail, options.explain);
    }
    Ok(ExitCode::SUCCESS)
}

fn open_similar_service(
    runtime: &AtlasRuntime,
    backend: CliIndexBackend,
) -> Result<atlas_search::AtlasRetrievalService, String> {
    match backend {
        CliIndexBackend::Sqlite => runtime.open_vector_record_retrieval_service(),
        CliIndexBackend::Ladybug => runtime.open_ladybug_record_retrieval_service(),
    }
    .map_err(|error| error.to_string())
}

fn resolve_seed(
    service: &atlas_search::AtlasRetrievalService,
    record_ref: &str,
    filter: Option<&SearchFilterNode>,
) -> Result<Option<RecordKey>, SearchError> {
    if let Ok(key) = RecordKey::parse(record_ref) {
        return Ok(Some(key));
    }
    let matches = service.resolve_record_with_options(
        record_ref,
        filter,
        atlas_index::RecordLoadOptions::omit_raw_json(),
    )?;
    match matches.as_slice() {
        [] => Ok(None),
        [resolution] => Ok(Some(resolution.record.key.clone())),
        alternatives => Err(SearchError::InvalidSearchOptions(format!(
            "record resolution ambiguous: {}; candidates: {}",
            record_ref,
            resolution_candidates(alternatives)
        ))),
    }
}

fn resolution_candidates(matches: &[RecordResolutionResult]) -> String {
    matches
        .iter()
        .take(5)
        .map(|resolution| format!("{} ({})", resolution.record.name, resolution.record.key))
        .collect::<Vec<_>>()
        .join(", ")
}

fn similar_data(
    result: &SimilarRecordResult,
    detail: DetailLevel,
    include_raw: bool,
    filter: Option<Value>,
) -> SimilarData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: include_raw,
    };
    SimilarData {
        detail: detail.to_string(),
        seed: record_json(&result.seed, options),
        seed_embedding_unit_key: result.seed_embedding_unit_key.clone(),
        filter,
        results: result
            .records
            .iter()
            .map(|record| SimilarResultJson {
                record: record_json(&record.record, options),
                similarity: SimilarityJson {
                    score: record.score,
                    semantic: SimilarSemanticJson {
                        embedding_unit_key: record.semantic.embedding_unit_key.clone(),
                        unit_kind: record.semantic.unit_kind.clone(),
                        label: record.semantic.label.clone(),
                        distance: record.semantic.distance,
                        rank_distance: record.semantic.rank_distance,
                    },
                    graph: SimilarGraphJson {
                        shared_mechanics: record
                            .graph
                            .shared_mechanics
                            .iter()
                            .map(|mechanic| SimilarSharedMechanicJson {
                                key: mechanic.key.to_string(),
                                name: mechanic.name.clone(),
                            })
                            .collect(),
                        shared_traits: record.graph.shared_traits.clone(),
                    },
                },
            })
            .collect(),
    }
}

fn print_similar(data: &SimilarData, detail: DetailLevel, explain: bool) {
    println!(
        "Seed: {}\t{}\t{}",
        data.seed.key, data.seed.name, data.seed.record_family
    );
    for result in &data.results {
        if detail_outputs_description(detail) {
            println!();
            print_record_for_detail(&result.record, detail);
        } else if explain {
            println!(
                "{}\t{}\t{}\tscore={:.4}\tdistance={:.4}\tshared_mechanics={}\tshared_traits={}",
                result.record.key,
                result.record.name,
                result.record.record_family,
                result.similarity.score,
                result.similarity.semantic.rank_distance,
                result.similarity.graph.shared_mechanics.len(),
                result.similarity.graph.shared_traits.len()
            );
        } else {
            println!(
                "{}\t{}\t{}\tscore={:.4}",
                result.record.key,
                result.record.name,
                result.record.record_family,
                result.similarity.score
            );
        }
    }
}
