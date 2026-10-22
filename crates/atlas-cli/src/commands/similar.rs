use super::{
    filters::{resolve_filter, resolve_reference},
    product::{app_result, print_search_matches, report_app_error},
};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{SearchCriteria, SearchModeSummary, SearchPageRequest, SearchResultsRequest};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;
pub(crate) mod args;
use args::SimilarOptions;
pub(crate) fn run_similar(o: SimilarOptions) -> Result<ExitCode, String> {
    let sort = super::result_order::requested_sort(&o.sort, o.seed)?;
    let service = match connect(ClientOptions {
        search_profile: o.search_profile.clone(),
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandStoredVectors,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let filter = match resolve_filter(&o.filter_options, &service) {
        Ok(filter) => filter,
        Err(error) => return report_app_error(error, o.json),
    };
    let seed = match resolve_reference(&service, &o.record_ref) {
        Ok(key) => key,
        Err(error) => return report_app_error(error, o.json),
    };
    let result = match app_result(
        service.search_page(
            SearchResultsRequest {
                sort,
                ranking: None,
                search_profile: o.search_profile,
                criteria: SearchCriteria::SimilarRecords {
                    seed_record_key: seed.to_string(),
                    filter,
                },
                page: SearchPageRequest {
                    number: o.page,
                    size: o.limit,
                },
                expected_artifact_generation: None,
            },
            &atlas_app_service::ReadExecutionContext::uncancellable(),
        ),
        o.json,
    )? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    if o.json {
        write_json_data(&result)?;
    } else {
        if let SearchModeSummary::SimilarRecords { seed } = &result.mode {
            println!("Similar to {}", seed.title);
        }
        super::result_order::print_random_seed(&result.sort);
        print_search_matches(&result.rows);
        super::product::print_search_page_count(&result);
    }
    Ok(ExitCode::SUCCESS)
}
