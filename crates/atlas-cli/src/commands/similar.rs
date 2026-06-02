use std::process::ExitCode;

use atlas_domain::{DetailLevel, RecordKey};
use atlas_record::{RecordJsonOptions, record_json};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{
    RecordResolutionResult, RecordRetrieval, ResolveRecordRequest, SearchError,
    SimilarRecordRequest, SimilarRecordResult, SimilarRetrieval, SimilarScoreWeights,
};
use serde::Serialize;
use serde_json::Value;

use crate::commands::filters::build_filter;
use crate::commands::record::{
    detail_outputs_description, print_record_for_detail, search_error, search_error_code,
};
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
    if let Err(error) = weights.validate() {
        if options.json {
            write_json_error("invalid_option", error)?;
            return Ok(ExitCode::from(2));
        }
        return Err(error);
    }
    let (filter, filter_value) =
        match build_filter(options.filter_json.as_deref(), &options.filter_options) {
            Ok(filter) => filter,
            Err(error) if options.json => {
                write_json_error(error.code, error.message)?;
                return Ok(ExitCode::from(2));
            }
            Err(error) => return Err(error.message),
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
    let service = match runtime.open_vector_record_retrieval_service() {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error("index_unavailable", error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error.to_string()),
    };
    let seed = match resolve_seed(&service, &options.record_ref) {
        Ok(SeedResolution::Resolved(seed)) => seed,
        Ok(SeedResolution::Missing) => {
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
        Ok(SeedResolution::Ambiguous(ambiguity)) => {
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
        weights,
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

enum SeedResolution {
    Resolved(RecordKey),
    Missing,
    Ambiguous(AmbiguousSeedResolution),
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

fn resolve_seed(
    service: &atlas_search::AtlasRetrievalService,
    record_ref: &str,
) -> Result<SeedResolution, SearchError> {
    if let Ok(key) = RecordKey::parse(record_ref) {
        return Ok(SeedResolution::Resolved(key));
    }
    let matches = service.resolve_record(ResolveRecordRequest {
        query: record_ref,
        filter: None,
    })?;
    match matches.as_slice() {
        [] => Ok(SeedResolution::Missing),
        [resolution] => Ok(SeedResolution::Resolved(
            resolution.record.identity.key.clone(),
        )),
        alternatives => Ok(SeedResolution::Ambiguous(ambiguous_seed_resolution(
            record_ref,
            alternatives,
        ))),
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
