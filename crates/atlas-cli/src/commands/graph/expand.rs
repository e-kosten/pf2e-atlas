use std::process::ExitCode;

use atlas_domain::DetailLevel;
use atlas_record::{RecordJsonOptions, record_json};
use atlas_search::{GraphExpandEvidence, GraphExpandRequest, GraphExpandResult};
use serde::Serialize;
use serde_json::Value;

use crate::GraphExpandOptions;
use crate::output::{write_json_data, write_json_error};

use super::{record_runtime, search_error, search_error_code};
use crate::commands::filters::build_filter;

#[derive(Debug, Serialize)]
struct GraphExpandData {
    detail: String,
    query: String,
    min_support: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Value>,
    semantic_seeds: Vec<GraphExpandSeedJson>,
    mechanics: Vec<GraphExpandMechanicJson>,
    expanded_records: Vec<GraphExpandRecordJson>,
}

#[derive(Debug, Serialize)]
struct GraphExpandSeedJson {
    record: atlas_record::RecordJson,
    distance: f64,
    rank_distance: f64,
    unit_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

#[derive(Debug, Serialize)]
struct GraphExpandMechanicJson {
    record: atlas_record::RecordJson,
    support_count: usize,
    supported_by: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GraphExpandRecordJson {
    record: atlas_record::RecordJson,
    is_semantic_seed: bool,
    evidence: Vec<GraphExpandEvidenceJson>,
}

#[derive(Debug, Serialize)]
struct GraphExpandEvidenceJson {
    mechanic_key: String,
    mechanic_name: String,
    support_count: usize,
    edge_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_text: Option<String>,
    reference_text: String,
}

pub(crate) fn run_graph_expand(options: GraphExpandOptions) -> Result<ExitCode, String> {
    let (filter, filter_value) =
        match build_filter(options.filter_json.as_deref(), &options.filter_options) {
            Ok(filter) => filter,
            Err(error) if options.json => {
                write_json_error(error.code, error.message)?;
                return Ok(ExitCode::from(2));
            }
            Err(error) => return Err(error.message),
        };
    let runtime = match record_runtime(options.path_mode.into(), options.index, None) {
        Ok(runtime) => runtime,
        Err(error) if options.json => {
            write_json_error("runtime_error", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let mut service = match runtime.open_retrieval_service() {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let result = match service.graph_expand(GraphExpandRequest {
        query: &options.query,
        filter: filter.as_ref(),
        semantic_limit: options.semantic_limit as u32,
        mechanic_limit: options.mechanics,
        min_support: options.min_support,
        expansion_limit: options.limit,
    }) {
        Ok(result) => result,
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let data = graph_expand_data(&result, options.detail, options.min_support, filter_value);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_expand(&data);
    }
    Ok(ExitCode::SUCCESS)
}

fn graph_expand_data(
    result: &GraphExpandResult,
    detail: DetailLevel,
    min_support: usize,
    filter: Option<Value>,
) -> GraphExpandData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphExpandData {
        detail: detail.to_string(),
        query: result.query.clone(),
        min_support,
        filter,
        semantic_seeds: result
            .semantic_seeds
            .iter()
            .map(|seed| GraphExpandSeedJson {
                record: record_json(&seed.record, options),
                distance: seed.hit.distance,
                rank_distance: seed.hit.rank_distance,
                unit_kind: seed.hit.unit_kind.clone(),
                label: seed.hit.label.clone(),
            })
            .collect(),
        mechanics: result
            .mechanics
            .iter()
            .map(|mechanic| GraphExpandMechanicJson {
                record: record_json(&mechanic.record, options),
                support_count: mechanic.support_count,
                supported_by: mechanic
                    .supported_by
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            })
            .collect(),
        expanded_records: result
            .expanded_records
            .iter()
            .map(|expanded| GraphExpandRecordJson {
                record: record_json(&expanded.record, options),
                is_semantic_seed: expanded.is_semantic_seed,
                evidence: expanded
                    .evidence
                    .iter()
                    .map(graph_expand_evidence_json)
                    .collect(),
            })
            .collect(),
    }
}

fn graph_expand_evidence_json(evidence: &GraphExpandEvidence) -> GraphExpandEvidenceJson {
    GraphExpandEvidenceJson {
        mechanic_key: evidence.mechanic_key.to_string(),
        mechanic_name: evidence.mechanic_name.clone(),
        support_count: evidence.support_count,
        edge_count: evidence.edge_count,
        display_text: evidence.display_text.clone(),
        reference_text: evidence.reference_text.clone(),
    }
}

fn print_graph_expand(data: &GraphExpandData) {
    println!("Query: {}", data.query);
    println!("Semantic seeds: {}", data.semantic_seeds.len());
    for seed in &data.semantic_seeds {
        println!(
            "- {}\t{}\t{}\tdistance={:.4}",
            seed.record.key, seed.record.name, seed.record.record_family, seed.rank_distance
        );
    }
    println!("Mechanics: {}", data.mechanics.len());
    for mechanic in &data.mechanics {
        println!(
            "- {}\t{}\t{}\tsupported_by={}",
            mechanic.record.key,
            mechanic.record.name,
            mechanic.record.record_family,
            mechanic.support_count
        );
    }
    println!("Expanded records: {}", data.expanded_records.len());
    for expanded in &data.expanded_records {
        let marker = if expanded.is_semantic_seed {
            " semantic_seed=true"
        } else {
            ""
        };
        let evidence = expanded
            .evidence
            .first()
            .map(|evidence| evidence.mechanic_name.as_str())
            .unwrap_or("-");
        println!(
            "- {}\t{}\t{}\tvia={evidence}{marker}",
            expanded.record.key, expanded.record.name, expanded.record.record_family
        );
    }
}
