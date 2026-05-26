use std::collections::BTreeMap;
use std::process::ExitCode;

use atlas_domain::{DetailLevel, RecordKey};
use atlas_index::RecordLoadOptions;
use atlas_record::{RecordJsonOptions, record_json};
use atlas_search::{
    AtlasRetrievalService, GraphContextEdge, GraphContextRequest, GraphContextResult,
    GraphContextSection, GraphRemasterLinksResult, GraphVariantGroupResult,
};
use serde::Serialize;

use crate::output::{write_json_data, write_json_error};
use crate::{
    CliIndexBackend, GraphLinksOptions, GraphRemasterOptions, GraphUsesOptions,
    GraphVariantsOptions,
};

use super::record::{open_record_service, record_runtime, search_error, search_error_code};

#[derive(Debug, Serialize)]
struct GraphLinksData {
    detail: String,
    seed: GraphSeedJson,
    outgoing: GraphSectionJson,
    backlinks: GraphSectionJson,
}

#[derive(Debug, Serialize)]
struct GraphUsesData {
    detail: String,
    seed: GraphSeedJson,
    uses: GraphSectionJson,
}

#[derive(Debug, Serialize)]
struct GraphVariantsData {
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<GraphSeedJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant_group_key: Option<String>,
    variants: Vec<GraphVariantJson>,
}

#[derive(Debug, Serialize)]
struct GraphVariantJson {
    record: atlas_record::RecordJson,
    is_seed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant_label: Option<String>,
    variant_axes: Vec<String>,
    variant_source: String,
}

#[derive(Debug, Serialize)]
struct GraphRemasterData {
    detail: String,
    seed: GraphSeedJson,
    links: Vec<GraphRemasterLinkJson>,
}

#[derive(Debug, Serialize)]
struct GraphRemasterLinkJson {
    direction: &'static str,
    remaster: atlas_record::RecordJson,
    legacy: atlas_record::RecordJson,
    source: GraphRemasterSourceJson,
}

#[derive(Debug, Serialize)]
struct GraphRemasterSourceJson {
    kind: String,
    reference: String,
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

pub(crate) fn run_graph_links(options: GraphLinksOptions) -> Result<ExitCode, String> {
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
    let key = match resolve_graph_record_ref(&service, &options.record_ref, options.json)? {
        Some(key) => key,
        None => return Ok(ExitCode::from(1)),
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
    let data = graph_links_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_links(&data, options.outgoing, options.backlinks);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_graph_uses(options: GraphUsesOptions) -> Result<ExitCode, String> {
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
    let key = match resolve_graph_record_ref(&service, &options.record_ref, options.json)? {
        Some(key) => key,
        None => return Ok(ExitCode::from(1)),
    };
    let result = match service.graph_context(GraphContextRequest {
        seed: key.clone(),
        outgoing_limit: 0,
        backlink_limit: options.limit,
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
    let data = graph_uses_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_uses(&data, options.limit);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_graph_variants(options: GraphVariantsOptions) -> Result<ExitCode, String> {
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
    let result = match resolve_graph_variant_group(&service, &options.record_ref, options.json)? {
        Ok(Some(result)) => result,
        Ok(None) => return Ok(ExitCode::from(1)),
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let data = graph_variants_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_variants(&data);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_graph_remaster(options: GraphRemasterOptions) -> Result<ExitCode, String> {
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
    let key = match resolve_graph_record_ref(&service, &options.record_ref, options.json)? {
        Some(key) => key,
        None => return Ok(ExitCode::from(1)),
    };
    let result = match service.remaster_links(&key) {
        Ok(Some(result)) => result,
        Ok(None) => return record_not_found(&key, options.json),
        Err(error) if options.json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ExitCode::from(3));
        }
        Err(error) => return Err(search_error(error)),
    };
    let data = graph_remaster_data(&result, options.detail);
    if options.json {
        write_json_data(data)?;
    } else {
        print_graph_remaster(&data);
    }
    Ok(ExitCode::SUCCESS)
}

fn graph_links_data(result: &GraphContextResult, detail: DetailLevel) -> GraphLinksData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphLinksData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: record_json(&result.seed, options),
        },
        outgoing: graph_section_json(&result.outgoing, options),
        backlinks: graph_section_json(&result.backlinks, options),
    }
}

fn graph_uses_data(result: &GraphContextResult, detail: DetailLevel) -> GraphUsesData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphUsesData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: record_json(&result.seed, options),
        },
        uses: graph_section_json(&result.backlinks, options),
    }
}

fn graph_variants_data(result: &GraphVariantGroupResult, detail: DetailLevel) -> GraphVariantsData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphVariantsData {
        detail: detail.to_string(),
        seed: result.seed.as_ref().map(|seed| GraphSeedJson {
            record: record_json(seed, options),
        }),
        variant_group_key: result.group.variant_group_key.clone(),
        variants: result
            .group
            .records
            .iter()
            .map(|record| GraphVariantJson {
                record: record_json(record, options),
                is_seed: result
                    .seed
                    .as_ref()
                    .is_some_and(|seed| record.key == seed.key),
                variant_label: record.variant_label.clone(),
                variant_axes: record.variant_axes.clone(),
                variant_source: record.variant_source.clone(),
            })
            .collect(),
    }
}

fn graph_remaster_data(
    result: &GraphRemasterLinksResult,
    detail: DetailLevel,
) -> GraphRemasterData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphRemasterData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: record_json(&result.seed, options),
        },
        links: result
            .links
            .links
            .iter()
            .map(|link| {
                let direction = if result.seed.key == link.legacy_record.key {
                    "legacy_to_remaster"
                } else {
                    "remaster_to_legacy"
                };
                GraphRemasterLinkJson {
                    direction,
                    remaster: record_json(&link.remaster_record, options),
                    legacy: record_json(&link.legacy_record, options),
                    source: GraphRemasterSourceJson {
                        kind: link.source.as_str().to_string(),
                        reference: link.source_ref.clone(),
                    },
                }
            })
            .collect(),
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

fn print_graph_links(data: &GraphLinksData, outgoing_limit: usize, backlink_limit: usize) {
    println!(
        "{}\t{}\t{}",
        data.seed.record.key, data.seed.record.name, data.seed.record.record_family
    );
    print_section("Outgoing", &data.outgoing, outgoing_limit, true);
    print_section("Backlinks", &data.backlinks, backlink_limit, false);
}

fn print_graph_uses(data: &GraphUsesData, limit: usize) {
    println!(
        "{}\t{}\t{}",
        data.seed.record.key, data.seed.record.name, data.seed.record.record_family
    );
    print_section("Uses", &data.uses, limit, false);
}

fn print_graph_variants(data: &GraphVariantsData) {
    if let Some(seed) = &data.seed {
        println!(
            "{}\t{}\t{}",
            seed.record.key, seed.record.name, seed.record.record_family
        );
    }
    let Some(group_key) = &data.variant_group_key else {
        println!("Variants: none");
        return;
    };
    println!("Variant group: {group_key}");
    if data.variants.is_empty() {
        println!("Variants: none");
        return;
    }
    for variant in &data.variants {
        let current = if variant.is_seed { " *" } else { "" };
        let label = variant
            .variant_label
            .as_deref()
            .filter(|label| !label.is_empty())
            .unwrap_or("-");
        let axes = if variant.variant_axes.is_empty() {
            "-".to_string()
        } else {
            variant.variant_axes.join(", ")
        };
        println!(
            "-{} {}\t{}\t{}\tlabel={label}\taxes={axes}",
            current, variant.record.key, variant.record.name, variant.record.record_family
        );
    }
}

fn print_graph_remaster(data: &GraphRemasterData) {
    println!(
        "{}\t{}\t{}",
        data.seed.record.key, data.seed.record.name, data.seed.record.record_family
    );
    if data.links.is_empty() {
        println!("Remaster links: none");
        return;
    }
    for link in &data.links {
        println!(
            "- {}\t{} -> {}\t{} -> {}",
            link.direction,
            link.legacy.key,
            link.remaster.key,
            link.legacy.name,
            link.remaster.name
        );
    }
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

fn resolve_graph_record_ref(
    service: &AtlasRetrievalService,
    record_ref: &str,
    json: bool,
) -> Result<Option<RecordKey>, String> {
    if let Ok(key) = RecordKey::parse(record_ref) {
        return Ok(Some(key));
    }
    let matches = match service.resolve_record_with_options(
        record_ref,
        None,
        RecordLoadOptions::omit_raw_json(),
    ) {
        Ok(matches) => matches,
        Err(error) if json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(None);
        }
        Err(error) => return Err(search_error(error)),
    };
    if matches.is_empty() {
        if json {
            write_json_error(
                "record_resolution_miss",
                format!("record resolution miss: {record_ref}"),
            )?;
        } else {
            eprintln!("record resolution miss: {record_ref}");
        }
        return Ok(None);
    }
    if matches.len() > 1 {
        write_record_resolution_ambiguity(record_ref, &matches, json)?;
        return Ok(None);
    }
    Ok(matches.into_iter().next().map(|match_| match_.record.key))
}

fn resolve_graph_variant_group(
    service: &AtlasRetrievalService,
    record_ref: &str,
    json: bool,
) -> Result<Result<Option<GraphVariantGroupResult>, atlas_search::SearchError>, String> {
    if let Ok(key) = RecordKey::parse(record_ref) {
        return Ok(service.variant_group(&key));
    }
    let matches = match service.resolve_record_with_options(
        record_ref,
        None,
        RecordLoadOptions::omit_raw_json(),
    ) {
        Ok(matches) => matches,
        Err(error) if json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(Ok(None));
        }
        Err(error) => return Err(search_error(error)),
    };
    if matches.len() == 1 {
        return Ok(service.variant_group(&matches[0].record.key));
    }
    if matches.len() > 1 {
        write_record_resolution_ambiguity(record_ref, &matches, json)?;
        return Ok(Ok(None));
    }

    let variant_groups = match service.variant_groups_by_base_name(record_ref) {
        Ok(groups) => groups,
        Err(error) => return Ok(Err(error)),
    };
    if variant_groups.len() == 1 {
        return Ok(Ok(variant_groups.into_iter().next()));
    }
    if variant_groups.len() > 1 {
        write_variant_group_ambiguity(record_ref, &variant_groups, json)?;
        return Ok(Ok(None));
    }

    if json {
        write_json_error(
            "record_resolution_miss",
            format!("record resolution miss: {record_ref}"),
        )?;
    } else {
        eprintln!("record resolution miss: {record_ref}");
    }
    Ok(Ok(None))
}

fn write_record_resolution_ambiguity(
    record_ref: &str,
    matches: &[atlas_index::RecordResolutionResult],
    json: bool,
) -> Result<(), String> {
    let alternatives = matches
        .iter()
        .take(5)
        .map(|hit| format!("{} ({})", hit.record.name, hit.record.key))
        .collect::<Vec<_>>()
        .join(", ");
    let message = format!("record resolution ambiguous: {record_ref}; candidates: {alternatives}");
    if json {
        write_json_error("record_resolution_ambiguous", message)?;
    } else {
        eprintln!("{message}");
    }
    Ok(())
}

fn write_variant_group_ambiguity(
    record_ref: &str,
    groups: &[GraphVariantGroupResult],
    json: bool,
) -> Result<(), String> {
    let alternatives = groups
        .iter()
        .take(5)
        .map(|group| {
            let group_key = group
                .group
                .variant_group_key
                .as_deref()
                .unwrap_or("<unknown>");
            let first_name = group
                .group
                .records
                .first()
                .map(|record| record.name.as_str())
                .unwrap_or("<empty>");
            format!("{group_key} ({first_name})")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let message =
        format!("variant group resolution ambiguous: {record_ref}; candidates: {alternatives}");
    if json {
        write_json_error("variant_group_resolution_ambiguous", message)?;
    } else {
        eprintln!("{message}");
    }
    Ok(())
}

fn record_not_found(key: &RecordKey, json: bool) -> Result<ExitCode, String> {
    if json {
        write_json_error("record_not_found", format!("record not found: {key}"))?;
    } else {
        eprintln!("record not found: {key}");
    }
    Ok(ExitCode::from(1))
}
