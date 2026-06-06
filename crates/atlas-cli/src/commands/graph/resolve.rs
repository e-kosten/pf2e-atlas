use std::process::ExitCode;

use atlas_domain::{DetailLevel, RecordKey};
use atlas_record::{RecordJsonOptions, record_json};
use atlas_search::{
    RecordRefResolutionResult, RecordResolutionResult, RecordRetrieval, ResolveRecordRefRequest,
    ResolveVariantGroupRefRequest, SearchError, VariantGroupRefResolutionResult,
    VariantGroupResult, VariantRetrieval,
};
use serde::Serialize;

use crate::output::{write_json_error, write_json_error_data};

use super::super::record::{search_error, search_error_code};

pub(super) enum GraphCommandOutcome<T> {
    Value(T),
    Exit(ExitCode),
}

pub(super) fn resolve_graph_record_ref(
    service: &impl RecordRetrieval,
    record_ref: &str,
    json: bool,
) -> Result<GraphCommandOutcome<RecordKey>, String> {
    let resolution = match service.resolve_record_ref(ResolveRecordRefRequest {
        record_ref,
        filter: None,
    }) {
        Ok(resolution) => resolution,
        Err(error) => return graph_search_error(error, json),
    };
    match resolution {
        RecordRefResolutionResult::Key(key) => Ok(GraphCommandOutcome::Value(key)),
        RecordRefResolutionResult::Miss => {
            if json {
                write_json_error(
                    "record_resolution_miss",
                    format!("record resolution miss: {record_ref}"),
                )?;
            } else {
                eprintln!("record resolution miss: {record_ref}");
            }
            Ok(GraphCommandOutcome::Exit(ExitCode::from(1)))
        }
        RecordRefResolutionResult::Ambiguous(matches) => {
            write_record_resolution_ambiguity(record_ref, &matches, json)?;
            Ok(GraphCommandOutcome::Exit(ExitCode::from(1)))
        }
    }
}

pub(super) fn resolve_graph_variant_group(
    service: &impl VariantRetrieval,
    record_ref: &str,
    json: bool,
) -> Result<GraphCommandOutcome<VariantGroupResult>, String> {
    let resolution = match service.resolve_variant_group_ref(ResolveVariantGroupRefRequest {
        variant_group_ref: record_ref,
    }) {
        Ok(resolution) => resolution,
        Err(error) => return graph_search_error(error, json),
    };

    match resolution {
        VariantGroupRefResolutionResult::Group { result, .. } => {
            Ok(GraphCommandOutcome::Value(*result))
        }
        VariantGroupRefResolutionResult::RecordNotFound { record_key } => {
            record_not_found(&record_key, json).map(GraphCommandOutcome::Exit)
        }
        VariantGroupRefResolutionResult::ResolutionMiss => {
            if json {
                write_json_error(
                    "record_resolution_miss",
                    format!("record resolution miss: {record_ref}"),
                )?;
            } else {
                eprintln!("record resolution miss: {record_ref}");
            }
            Ok(GraphCommandOutcome::Exit(ExitCode::from(1)))
        }
        VariantGroupRefResolutionResult::RecordResolutionAmbiguous(matches) => {
            write_record_resolution_ambiguity(record_ref, &matches, json)?;
            Ok(GraphCommandOutcome::Exit(ExitCode::from(1)))
        }
        VariantGroupRefResolutionResult::VariantGroupAmbiguous(groups) => {
            write_variant_group_ambiguity(record_ref, &groups, json)?;
            Ok(GraphCommandOutcome::Exit(ExitCode::from(1)))
        }
    }
}

pub(super) fn graph_search_error<T>(
    error: SearchError,
    json: bool,
) -> Result<GraphCommandOutcome<T>, String> {
    if json {
        write_json_error(search_error_code(&error), error.to_string())?;
        Ok(GraphCommandOutcome::Exit(ExitCode::from(3)))
    } else {
        Err(search_error(error))
    }
}

fn write_record_resolution_ambiguity(
    record_ref: &str,
    matches: &[RecordResolutionResult],
    json: bool,
) -> Result<(), String> {
    let ambiguity = ambiguous_record_resolution(record_ref, matches);
    let message = ambiguity.message();
    if json {
        write_json_error_data("record_resolution_ambiguous", message, ambiguity)?;
    } else {
        eprintln!("{message}");
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct AmbiguousGraphResolution {
    result: AmbiguousGraphResult,
}

#[derive(Debug, Serialize)]
struct AmbiguousGraphResult {
    query: String,
    alternatives: Vec<GraphResolutionAlternativeJson>,
}

#[derive(Debug, Serialize)]
struct GraphResolutionAlternativeJson {
    record: atlas_record::RecordJson,
    resolution: GraphResolutionJson,
}

#[derive(Debug, Serialize)]
struct GraphResolutionJson {
    query: String,
    normalized_query: String,
    match_kind: &'static str,
    matched_text: String,
}

impl AmbiguousGraphResolution {
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

fn ambiguous_record_resolution(
    record_ref: &str,
    matches: &[RecordResolutionResult],
) -> AmbiguousGraphResolution {
    let record_options = RecordJsonOptions {
        detail: DetailLevel::Summary,
        include_source_json: false,
    };
    AmbiguousGraphResolution {
        result: AmbiguousGraphResult {
            query: record_ref.to_string(),
            alternatives: matches
                .iter()
                .take(5)
                .map(|resolution| GraphResolutionAlternativeJson {
                    record: record_json(&resolution.record, record_options),
                    resolution: GraphResolutionJson {
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

fn write_variant_group_ambiguity(
    record_ref: &str,
    groups: &[VariantGroupResult],
    json: bool,
) -> Result<(), String> {
    let alternatives = groups
        .iter()
        .take(5)
        .map(|group| {
            let group_key = group.variant_group_key.as_deref().unwrap_or("<unknown>");
            let first_name = group
                .variants
                .first()
                .map(|record| record.identity.name.as_str())
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

pub(super) fn record_not_found(key: &RecordKey, json: bool) -> Result<ExitCode, String> {
    if json {
        write_json_error("record_not_found", format!("record not found: {key}"))?;
    } else {
        eprintln!("record not found: {key}");
    }
    Ok(ExitCode::from(1))
}
