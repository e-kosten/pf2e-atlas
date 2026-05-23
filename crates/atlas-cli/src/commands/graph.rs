use std::collections::BTreeMap;
use std::process::ExitCode;

use atlas_domain::{DetailLevel, RecordKey};
use atlas_record::{RecordJsonOptions, record_json};
use atlas_search::{
    GraphContextEdge, GraphContextRequest, GraphContextResult, GraphContextSection,
};
use serde::Serialize;

use crate::output::{write_json_data, write_json_error};
use crate::{CliIndexBackend, GraphGetOptions};

use super::record::{open_record_service, record_runtime, search_error, search_error_code};

#[derive(Debug, Serialize)]
struct GraphGetData {
    detail: String,
    seed: GraphSeedJson,
    outgoing: GraphSectionJson,
    backlinks: GraphSectionJson,
}

#[derive(Debug, Serialize)]
struct GraphSeedJson {
    record: atlas_record::RecordJson,
}

#[derive(Debug, Serialize)]
struct GraphSectionJson {
    records: Vec<atlas_record::RecordJson>,
    edges: Vec<GraphEdgeJson>,
    truncated: bool,
    total_records: usize,
    total_edges: usize,
}

#[derive(Debug, Serialize)]
struct GraphEdgeJson {
    from: String,
    to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_text: Option<String>,
    reference_text: String,
    source: GraphEdgeSourceJson,
}

#[derive(Debug, Serialize)]
struct GraphEdgeSourceJson {
    kind: String,
    visibility: String,
}

pub(crate) fn run_graph_get(options: GraphGetOptions) -> Result<ExitCode, String> {
    let key = match RecordKey::parse(&options.key) {
        Ok(parsed) => parsed,
        Err(error) => {
            if options.json {
                write_json_error(
                    "invalid_input",
                    format!("invalid record key `{}`: {error}", options.key),
                )?;
                return Ok(ExitCode::from(2));
            }
            return Err(format!("invalid record key `{}`: {error}", options.key));
        }
    };
    let runtime = match record_runtime(options.path_mode.into(), options.index, None) {
        Ok(runtime) => runtime,
        Err(error) if options.json => {
            write_json_error("runtime_error", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let service = match open_record_service(&runtime, CliIndexBackend::Sqlite) {
        Ok(service) => service,
        Err(error) if options.json => {
            write_json_error("index_unavailable", error)?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(error),
    };
    let result = match service.graph_context(GraphContextRequest {
        seed: key.clone(),
        outgoing_limit: options.outgoing,
        backlink_limit: options.backlinks,
    }) {
        Ok(Some(result)) => result,
        Ok(None) => {
            if options.json {
                write_json_error("record_not_found", format!("record not found: {key}"))?;
            } else {
                eprintln!("record not found: {key}");
            }
            return Ok(ExitCode::from(1));
        }
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let data = graph_get_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_get(&data, options.outgoing, options.backlinks);
    }
    Ok(ExitCode::SUCCESS)
}

fn graph_get_data(result: &GraphContextResult, detail: DetailLevel) -> GraphGetData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphGetData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: record_json(&result.seed, options),
        },
        outgoing: graph_section_json(&result.outgoing, options),
        backlinks: graph_section_json(&result.backlinks, options),
    }
}

fn graph_section_json(
    section: &GraphContextSection,
    options: RecordJsonOptions,
) -> GraphSectionJson {
    GraphSectionJson {
        records: section
            .records
            .iter()
            .map(|record| record_json(record, options))
            .collect(),
        edges: section.edges.iter().map(graph_edge_json).collect(),
        truncated: section.truncated,
        total_records: section.total_records,
        total_edges: section.total_edges,
    }
}

fn graph_edge_json(edge: &GraphContextEdge) -> GraphEdgeJson {
    GraphEdgeJson {
        from: edge.from.to_string(),
        to: edge.to.to_string(),
        display_text: edge.display_text.clone(),
        reference_text: edge.reference_text.clone(),
        source: GraphEdgeSourceJson {
            kind: edge.source.kind.clone(),
            visibility: edge.source.visibility.clone(),
        },
    }
}

fn print_graph_get(data: &GraphGetData, outgoing_limit: usize, backlink_limit: usize) {
    println!(
        "{}\t{}\t{}",
        data.seed.record.key, data.seed.record.name, data.seed.record.record_family
    );
    print_section("Outgoing", &data.outgoing, outgoing_limit, true);
    print_section("Backlinks", &data.backlinks, backlink_limit, false);
}

fn print_section(label: &str, section: &GraphSectionJson, limit: usize, outgoing: bool) {
    if limit == 0 {
        println!("{label}: disabled");
        return;
    }
    println!(
        "{label}: {} records, {} edges (of {} records, {} edges)",
        section.records.len(),
        section.edges.len(),
        section.total_records,
        section.total_edges
    );
    let names_by_key = section
        .records
        .iter()
        .map(|record| (record.key.clone(), record.name.clone()))
        .collect::<BTreeMap<_, _>>();
    for edge in &section.edges {
        let neighbor_key = if outgoing { &edge.to } else { &edge.from };
        let label = edge
            .display_text
            .as_deref()
            .or_else(|| names_by_key.get(neighbor_key).map(String::as_str))
            .unwrap_or(neighbor_key);
        if outgoing {
            println!("- {label} -> {neighbor_key}");
        } else {
            println!("- {neighbor_key} -> {label}");
        }
    }
}
