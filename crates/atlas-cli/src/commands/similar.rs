use std::process::ExitCode;

use atlas_app_model::AppErrorCode;
use atlas_domain::DetailLevel;
use atlas_record::{RecordJsonOptions, record_json};
use atlas_search::{
    RecordResolutionResult, SimilarRecordRefResult, SimilarRecordResult, SimilarScoreWeights,
};
use serde::Serialize;
use serde_json::Value;

use crate::client::{
    AtlasClient, AtlasClientConfig, AtlasClientHandle, LocalAtlasClientOptions, connect,
};
use crate::commands::filters::build_filter;
use crate::commands::record::{detail_outputs_description, print_record_for_detail};
use crate::output::{write_json_data, write_json_error, write_json_error_data};

pub(crate) mod args;

use args::SimilarOptions;

#[derive(Debug, Serialize)]
struct SimilarData {
    detail: String,
    seed: atlas_record::RecordJson,
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
    unit_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    distance: f64,
    rank_distance: f64,
}

#[derive(Debug, Serialize)]
struct SimilarGraphJson {
    shared_references: Vec<SimilarSharedReferenceJson>,
    shared_traits: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SimilarSharedReferenceJson {
    key: String,
    name: String,
}

pub(crate) fn run_similar(options: SimilarOptions) -> Result<ExitCode, String> {
    let weights = SimilarScoreWeights {
        semantic: options.semantic_weight,
        reference: options.reference_weight,
        traits: options.trait_weight,
    };
    let (filter, filter_value) =
        match build_filter(options.filter_json.as_deref(), &options.filter_options) {
            Ok(filter) => filter,
            Err(error) if options.json => {
                write_json_error(error.code, error.message)?;
                return Ok(ExitCode::from(2));
            }
            Err(error) => return Err(error.message),
        };
    let client = match similar_client(options.path_mode.into(), options.index, options.json)? {
        SimilarCommandStep::Ready(client) => client,
        SimilarCommandStep::Exit(code) => return Ok(code),
    };
    let result = match client.similar_records_for_ref(
        options.record_ref.clone(),
        filter,
        options.limit,
        options.candidates,
        weights,
    ) {
        Ok(SimilarRecordRefResult::Found(result)) => *result,
        Ok(SimilarRecordRefResult::RecordNotFound(seed)) if options.json => {
            write_json_error("record_not_found", format!("record not found: {seed}"))?;
            return Ok(ExitCode::from(1));
        }
        Ok(SimilarRecordRefResult::RecordNotFound(seed)) => {
            eprintln!("record not found: {seed}");
            return Ok(ExitCode::from(1));
        }
        Ok(SimilarRecordRefResult::ResolutionMiss) => {
            if options.json {
                write_json_error(
                    "record_resolution_miss",
                    format!("record resolution miss: {}", options.record_ref),
                )?;
            } else {
                eprintln!("record resolution miss: {}", options.record_ref);
            }
            return Ok(ExitCode::from(1));
        }
        Ok(SimilarRecordRefResult::ResolutionAmbiguous(matches)) => {
            let ambiguity = ambiguous_seed_resolution(&options.record_ref, &matches);
            if options.json {
                write_json_error_data(
                    "record_resolution_ambiguous",
                    ambiguity.message(),
                    ambiguity,
                )?;
            } else {
                eprintln!("{ambiguity}");
            }
            return Ok(ExitCode::from(1));
        }
        Err(error) if options.json => {
            let exit_code = similar_error_exit_code(error.code);
            write_json_error(similar_error_code(error.code), error.message)?;
            return Ok(exit_code);
        }
        Err(error) => return Err(error.message),
    };
    let data = similar_data(&result, options.detail, options.include_raw, filter_value);
    if options.json {
        write_json_data(data)?;
    } else {
        print_similar(&data, options.detail, options.explain);
    }
    Ok(ExitCode::SUCCESS)
}

enum SimilarCommandStep<T> {
    Ready(T),
    Exit(ExitCode),
}

fn similar_client(
    path_mode: atlas_runtime::AtlasPathMode,
    index_path: Option<std::path::PathBuf>,
    json: bool,
) -> Result<SimilarCommandStep<AtlasClientHandle>, String> {
    match connect(AtlasClientConfig::Local(LocalAtlasClientOptions {
        path_mode,
        index_path,
        embedding_cache_root: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandStoredVectors,
    })) {
        Ok(client) => Ok(SimilarCommandStep::Ready(client)),
        Err(error) if json => {
            write_json_error(similar_error_code(error.code), error.message)?;
            Ok(SimilarCommandStep::Exit(similar_error_exit_code(
                error.code,
            )))
        }
        Err(error) => Err(error.message),
    }
}

fn similar_error_exit_code(code: AppErrorCode) -> ExitCode {
    match code {
        AppErrorCode::FilterInvalid | AppErrorCode::InvalidRequest => ExitCode::from(2),
        _ => ExitCode::from(3),
    }
}

fn similar_error_code(code: AppErrorCode) -> &'static str {
    match code {
        AppErrorCode::IndexUnavailable => "index_unavailable",
        AppErrorCode::ArtifactIncompatible => "index_unavailable",
        AppErrorCode::FilterInvalid => "invalid_filter",
        AppErrorCode::InvalidRequest => "invalid_option",
        AppErrorCode::VectorReadinessRequired => "vector_readiness_required",
        AppErrorCode::EmbeddingModelUnavailable | AppErrorCode::QueryFailed => "query_failed",
        AppErrorCode::ArtifactNotReady
        | AppErrorCode::SetupRequired
        | AppErrorCode::SetupInProgress => "runtime_error",
        _ => "query_failed",
    }
}

#[derive(Debug, Serialize)]
struct AmbiguousSeedResolution {
    result: AmbiguousSeedResult,
}

#[derive(Debug, Serialize)]
struct AmbiguousSeedResult {
    query: String,
    alternatives: Vec<ResolutionAlternativeJson>,
}

#[derive(Debug, Serialize)]
struct ResolutionAlternativeJson {
    record: atlas_record::RecordJson,
    resolution: ResolutionJson,
}

#[derive(Debug, Serialize)]
struct ResolutionJson {
    query: String,
    normalized_query: String,
    match_kind: &'static str,
    matched_text: String,
}

impl AmbiguousSeedResolution {
    fn message(&self) -> String {
        format!(
            "record resolution ambiguous: {}; candidates: {}",
            self.result.query,
            self.result
                .alternatives
                .iter()
                .map(|alternative| {
                    format!("{} ({})", alternative.record.name, alternative.record.key)
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn ambiguous_seed_resolution(
    record_ref: &str,
    matches: &[RecordResolutionResult],
) -> AmbiguousSeedResolution {
    let record_options = RecordJsonOptions {
        detail: DetailLevel::Summary,
        include_source_json: false,
    };
    AmbiguousSeedResolution {
        result: AmbiguousSeedResult {
            query: record_ref.to_string(),
            alternatives: matches
                .iter()
                .take(5)
                .map(|resolution| ResolutionAlternativeJson {
                    record: record_json(&resolution.record, record_options),
                    resolution: ResolutionJson {
                        query: resolution.query.clone(),
                        normalized_query: resolution.normalized_query.clone(),
                        match_kind: resolution.match_kind.as_str(),
                        matched_text: resolution.matched_text.clone(),
                    },
                })
                .collect(),
        },
    }
}

impl std::fmt::Display for AmbiguousSeedResolution {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message())
    }
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
        filter,
        results: result
            .records
            .iter()
            .map(|record| SimilarResultJson {
                record: record_json(&record.record, options),
                similarity: SimilarityJson {
                    score: record.score,
                    semantic: SimilarSemanticJson {
                        unit_kind: record.semantic.unit_kind.clone(),
                        label: record.semantic.label.clone(),
                        distance: record.semantic.distance,
                        rank_distance: record.semantic.rank_distance,
                    },
                    graph: SimilarGraphJson {
                        shared_references: record
                            .graph
                            .shared_references
                            .iter()
                            .map(|reference| SimilarSharedReferenceJson {
                                key: reference.key.to_string(),
                                name: reference.name.clone(),
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
        data.seed.key, data.seed.name, data.seed.kind
    );
    for result in &data.results {
        if detail_outputs_description(detail) {
            println!();
            print_record_for_detail(&result.record, detail);
            if explain {
                print_similarity_evidence(result);
            }
        } else if explain {
            print_similarity_evidence(result);
        } else {
            println!(
                "{}\t{}\t{}\tscore={:.4}",
                result.record.key, result.record.name, result.record.kind, result.similarity.score
            );
        }
    }
}

fn print_similarity_evidence(result: &SimilarResultJson) {
    println!(
        "{}\t{}\t{}\tscore={:.4}\tdistance={:.4}\tshared_references={}\tshared_traits={}",
        result.record.key,
        result.record.name,
        result.record.kind,
        result.similarity.score,
        result.similarity.semantic.rank_distance,
        result.similarity.graph.shared_references.len(),
        result.similarity.graph.shared_traits.len()
    );
}
