//! CLI transport for exact, generation-bound reference occurrence pages.
use super::args::GraphOccurrencesOptions;
use crate::commands::{
    filters::resolve_filter,
    followup::{EvidenceDetail, OccurrenceArgs, filter_args, shell_command, workspace_args},
    product::{app_result, report_app_error},
};
use atlas_app_model::{AppError, AppErrorCode, RelationshipOccurrencesRequest, SearchPageRequest};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;

pub(crate) fn run(options: GraphOccurrencesOptions) -> Result<ExitCode, String> {
    let workspace = workspace_args(options.path_mode, options.index.as_deref());
    let service = match super::client(options.path_mode, options.index) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, options.json),
    };
    let filter = match resolve_filter(
        &options.filter_options,
        &service,
        Some(&options.artifact_generation),
    ) {
        Ok(Some(filter)) => filter,
        Ok(None) => {
            return report_app_error(
                AppError::new(
                    AppErrorCode::FilterInvalid,
                    "Choose a positive link condition with --where, --links-to or --linked-from.",
                ),
                options.json,
            );
        }
        Err(error) => return report_app_error(error, options.json),
    };
    let clause_id =
        match atlas_domain::select_relationship_clause_id(&filter, options.clause_id.as_deref()) {
            Ok(id) => id,
            Err(error) => {
                return report_app_error(
                    AppError::new(AppErrorCode::FilterInvalid, error.to_string()),
                    options.json,
                );
            }
        };
    let result = match app_result(
        service.relationship_occurrences(
            RelationshipOccurrencesRequest {
                filter,
                clause_id,
                result_record_key: options.result_record_key,
                page: SearchPageRequest {
                    number: options.page,
                    size: options.limit,
                },
                expected_artifact_generation: options.artifact_generation,
            },
            &atlas_app_service::ReadExecutionContext::uncancellable(),
        ),
        options.json,
    )? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    let filters = filter_args(&options.filter_options);
    let next = OccurrenceArgs {
        workspace: workspace.as_deref().ok(),
        filter: &filters,
        generation: &result.artifact_generation,
        json: options.json,
        detail: options.detail,
    }
    .next(
        &result.result_record.record_key,
        &result.clause.clause_id,
        &result.page,
    );
    let owner = |edge: &atlas_app_model::RecordRelationshipView| {
        [&result.result_record, &result.clause.record]
            .into_iter()
            .find(|r| r.record_key == edge.source_navigation.record_key)
    };
    if options.json {
        use serde_json::json;
        let mut data = if options.detail == EvidenceDetail::Full {
            json!(result)
        } else {
            json!({"artifact_generation":result.artifact_generation,
            "clause":{"clause_id":result.clause.clause_id,"title":result.clause.title,"relationship":result.clause.relationship},
            "result_record":super::evidence::identity(&result.result_record),"page":result.page,
            "occurrences":result.occurrences.iter().map(|edge| super::evidence::occurrence(edge,owner(edge),&result.artifact_generation)).collect::<Vec<_>>()})
        };
        if let Some(args) = next {
            data["next_args"] = json!(args);
        } else if result.page.next_page.is_some()
            && let Err(reason) = workspace
        {
            data["followup_unavailable"] = json!(reason);
        }
        write_json_data(data)?;
    } else {
        println!("{} · {}", result.result_record.title, result.clause.title);
        for edge in &result.occurrences {
            println!("  {}", super::evidence::line(edge, owner(edge)));
            if options.detail == EvidenceDetail::Full
                && let Ok(workspace) = &workspace
            {
                println!(
                    "    {}",
                    shell_command(&super::open_record(
                        &edge.source_navigation,
                        &result.artifact_generation,
                        workspace,
                        false
                    ))
                );
            }
        }
        println!("{} of {} references", result.page.count, result.page.total);
        if let Some(args) = next {
            println!("Next page: {}", shell_command(&args));
        } else if let Err(reason) = workspace {
            println!("{reason}");
        }
    }
    Ok(ExitCode::SUCCESS)
}
