//! CLI transport for exact, generation-bound reference occurrence pages.
use super::args::GraphOccurrencesOptions;
use crate::commands::{
    filters::resolve_filter,
    product::{app_result, print_relationship_occurrence, report_app_error},
};
use atlas_app_model::{AppError, AppErrorCode, RelationshipOccurrencesRequest, SearchPageRequest};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;

pub(crate) fn run(options: GraphOccurrencesOptions) -> Result<ExitCode, String> {
    let service = match super::client(options.path_mode, options.index) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, options.json),
    };
    let filter = match resolve_filter(&options.filter_options, &service) {
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
    if options.json {
        write_json_data(&result)?;
    } else {
        for occurrence in &result.occurrences {
            print_relationship_occurrence(occurrence);
        }
        println!(
            "{} of {} references{}",
            result.page.count,
            result.page.total,
            if result.page.has_more {
                "; more on the next page"
            } else {
                ""
            }
        );
    }
    Ok(ExitCode::SUCCESS)
}
