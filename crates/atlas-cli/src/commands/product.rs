//! Product presentation and error handling; app-service owns product projections.
use atlas_app_service::AppServiceResult;
use atlas_cli_support::{write_json_data, write_json_failure};
use serde::Serialize;
use std::process::ExitCode;
/// Present a service failure once and propagate its classified exit to the command.
pub(crate) fn app_result<T>(
    result: AppServiceResult<T>,
    json: bool,
) -> Result<std::ops::ControlFlow<ExitCode, T>, String> {
    match result {
        Ok(value) => Ok(std::ops::ControlFlow::Continue(value)),
        Err(error) => Ok(std::ops::ControlFlow::Break(report_app_error(
            error.into_app_error(),
            json,
        )?)),
    }
}
pub(crate) fn emit<T: Serialize>(
    value: std::ops::ControlFlow<ExitCode, T>,
    json: bool,
) -> Result<ExitCode, String> {
    let value = match value {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    if json {
        write_json_data(&value)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn print_summary(view: &atlas_app_model::RecordSummaryView) {
    println!(
        "{} — {}{}{}",
        view.title,
        view.kind_label,
        view.level_label
            .as_ref()
            .map(|v| format!(" · {v}"))
            .unwrap_or_default(),
        view.unit_price_label
            .as_ref()
            .map(|v| format!(" · {v}"))
            .unwrap_or_default()
    );
    println!("  {}", view.record_key);
}
pub(crate) fn print_search_matches(records: &[atlas_app_model::SearchResultRow]) {
    for r in records {
        print_summary(&r.record);
        if let Some(w) = r
            .matches
            .iter()
            .find(|w| w.snippet.as_ref().is_some_and(|s| !s.trim().is_empty()))
        {
            let label = w
                .label
                .as_deref()
                .filter(|label| *label != r.record.title && !label.trim().is_empty());
            let prefix = label.map(|label| format!("{label}: ")).unwrap_or_default();
            println!("  {prefix}{}", w.snippet.as_deref().unwrap_or_default());
        } else if !r
            .matches
            .iter()
            .any(|w| w.navigation.owners.is_empty() && w.navigation.field.is_none())
            && let Some(label) = r
                .matches
                .iter()
                .find_map(|w| w.label.as_deref())
                .filter(|label| *label != r.record.title && !label.trim().is_empty())
        {
            println!("  Matched: {label}");
        }
        for evidence in &r.relationship_evidence {
            for occurrence in &evidence.occurrences {
                print_relationship_occurrence(occurrence);
            }
            if evidence.page.has_more {
                println!(
                    "  {} references; more available with `atlas graph occurrences`.",
                    evidence.page.total
                );
            }
        }
        println!();
    }
}
pub(crate) fn print_search_page_count(result: &atlas_app_model::SearchResultsPage) {
    let exhaustive = result
        .coverage
        .as_ref()
        .is_none_or(|coverage| coverage.exhaustive);
    if exhaustive {
        println!(
            "{} result{}{}",
            result.page.total,
            if result.page.total == 1 { "" } else { "s" },
            if result.page.has_more {
                "; more on the next page"
            } else {
                ""
            }
        );
    } else {
        println!(
            "Showing {} results{}",
            result.page.count,
            if result.page.has_more {
                "; more on the next page"
            } else {
                ""
            }
        );
    }
}

pub(crate) fn print_relationship_occurrence(occurrence: &atlas_app_model::RecordRelationshipView) {
    println!(
        "  {} → {}",
        occurrence
            .source_title
            .as_deref()
            .unwrap_or("Unnamed entry"),
        occurrence
            .target_title
            .as_deref()
            .unwrap_or("Unnamed entry")
    );
}

/// Emit a whole-command failure with the same exit policy as batch items.
pub(crate) fn report_app_error(
    error: atlas_app_model::AppError,
    json: bool,
) -> Result<ExitCode, String> {
    let exit = exit_class(error.code);
    if json {
        write_json_failure(&error)?;
    } else {
        eprintln!("{}", error.message);
        print_error_hint(&error);
        if let Some(details) = &error.details {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(details).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(ExitCode::from(exit))
}

pub(crate) fn exit_class(code: atlas_app_model::AppErrorCode) -> u8 {
    use atlas_app_model::AppErrorCode::*;
    match code {
        PartyNotFound
        | RecordNotFound
        | RecordResolutionMiss
        | RecordResolutionAmbiguous
        | SavedListNotFound
        | EncounterNotFound
        | EncounterParticipantNotFound => 1,
        PartyConflict
        | InvalidRequest
        | InvalidRecordKey
        | FilterInvalid
        | FilterFieldInvalid
        | FilterOptionInvalid
        | FilterFieldNotApplicable
        | SavedListAlreadyExists
        | EncounterAlreadyExists
        | EncounterParticipantConflict
        | EncounterTurnConflict
        | FilterEditorConflict => 2,
        ArtifactGenerationChanged
        | QueryFailed
        | IndexUnavailable
        | ArtifactNotReady
        | ArtifactIncompatible
        | VectorReadinessRequired
        | EmbeddingModelUnavailable
        | SetupRequired
        | SetupInProgress
        | ServiceBusy
        | OperationCancelled
        | OperationTimeout
        | InternalError => 3,
    }
}

pub(crate) fn print_error_hint(error: &atlas_app_model::AppError) {
    use atlas_app_model::AppErrorCode;
    let hint = match error.code {
        AppErrorCode::SavedListAlreadyExists => {
            "Choose another list id, or use `atlas lists add <ID> --record <RECORD>` to add to the existing list."
        }
        AppErrorCode::SavedListNotFound => {
            "Run `atlas lists ls` to see existing list ids, or `atlas lists create <ID> --name <NAME>` to create one. Use the same --index and --path-mode options."
        }
        AppErrorCode::RecordResolutionMiss | AppErrorCode::RecordResolutionAmbiguous => {
            "Run `atlas record resolve <NAME>` and use a returned canonical key. Use the same --index and --path-mode options."
        }
        AppErrorCode::InvalidRecordKey => {
            "Use a canonical pack:id key returned by `atlas record resolve <NAME>`."
        }
        _ => return,
    };
    eprintln!("{hint}");
}
