use std::process::ExitCode;

use atlas_domain::RecordKey;
use atlas_search::{
    AtlasRetrievalService, GraphVariantGroupResult, RecordResolutionResult, SearchError,
};

use crate::output::write_json_error;

use super::super::record::{search_error, search_error_code};

pub(super) enum GraphCommandOutcome<T> {
    Value(T),
    Exit(ExitCode),
}

pub(super) fn resolve_graph_record_ref(
    service: &AtlasRetrievalService,
    record_ref: &str,
    json: bool,
) -> Result<GraphCommandOutcome<RecordKey>, String> {
    if let Ok(key) = RecordKey::parse(record_ref) {
        return Ok(GraphCommandOutcome::Value(key));
    }
    let matches = match service.resolve_record(record_ref, None) {
        Ok(matches) => matches,
        Err(error) => return graph_search_error(error, json),
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
        return Ok(GraphCommandOutcome::Exit(ExitCode::from(1)));
    }
    if matches.len() > 1 {
        write_record_resolution_ambiguity(record_ref, &matches, json)?;
        return Ok(GraphCommandOutcome::Exit(ExitCode::from(1)));
    }
    Ok(GraphCommandOutcome::Value(matches[0].record.key.clone()))
}

pub(super) fn resolve_graph_variant_group(
    service: &AtlasRetrievalService,
    record_ref: &str,
    json: bool,
) -> Result<GraphCommandOutcome<GraphVariantGroupResult>, String> {
    if let Ok(key) = RecordKey::parse(record_ref) {
        return match service.variant_group(&key) {
            Ok(Some(result)) => Ok(GraphCommandOutcome::Value(result)),
            Ok(None) => record_not_found(&key, json).map(GraphCommandOutcome::Exit),
            Err(error) => graph_search_error(error, json),
        };
    }
    let matches = match service.resolve_record(record_ref, None) {
        Ok(matches) => matches,
        Err(error) => return graph_search_error(error, json),
    };
    if matches.len() == 1 {
        let key = &matches[0].record.key;
        return match service.variant_group(key) {
            Ok(Some(result)) => Ok(GraphCommandOutcome::Value(result)),
            Ok(None) => record_not_found(key, json).map(GraphCommandOutcome::Exit),
            Err(error) => graph_search_error(error, json),
        };
    }
    if matches.len() > 1 {
        write_record_resolution_ambiguity(record_ref, &matches, json)?;
        return Ok(GraphCommandOutcome::Exit(ExitCode::from(1)));
    }

    let variant_groups = match service.variant_groups_by_base_name(record_ref) {
        Ok(groups) => groups,
        Err(error) => return graph_search_error(error, json),
    };
    match variant_groups.len() {
        1 => {
            if let Some(result) = variant_groups.into_iter().next() {
                return Ok(GraphCommandOutcome::Value(result));
            }
        }
        count if count > 1 => {
            write_variant_group_ambiguity(record_ref, &variant_groups, json)?;
            return Ok(GraphCommandOutcome::Exit(ExitCode::from(1)));
        }
        _ => {}
    }

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
            let group_key = group.variant_group_key.as_deref().unwrap_or("<unknown>");
            let first_name = group
                .variants
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

pub(super) fn record_not_found(key: &RecordKey, json: bool) -> Result<ExitCode, String> {
    if json {
        write_json_error("record_not_found", format!("record not found: {key}"))?;
    } else {
        eprintln!("record not found: {key}");
    }
    Ok(ExitCode::from(1))
}
