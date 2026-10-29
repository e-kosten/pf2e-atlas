use super::{
    filters::{build_filter, resolve_filter},
    product::{app_result, print_search_matches, report_app_error},
};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{RetrievalModeView, SearchCriteria, SearchPageRequest, SearchResultsRequest};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;
pub(crate) mod args;
use args::SearchOptions;
pub(crate) fn run_search(o: SearchOptions) -> Result<ExitCode, String> {
    let sort = super::result_order::requested_sort(&o.sort, o.seed)?;
    if o.print_filter {
        let (_, value) = match build_filter(&o.filter_options) {
            Ok(value) => value,
            Err(error) => return report_app_error(error, o.json),
        };
        if o.json {
            write_json_data(serde_json::json!({"predicate":value}))?;
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
            );
        }
        return Ok(ExitCode::SUCCESS);
    }
    let mode: RetrievalModeView = o.retrieval.into();
    let service = match connect(ClientOptions {
        search_profile: o.search_profile.clone(),
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index,
        embedding_cache: o.embedding_cache_path,
        retrieval_mode: if o.query.is_some() && mode != RetrievalModeView::Lexical {
            atlas_app_service::AppServiceRetrievalMode::OnDemandEmbeddings
        } else {
            atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings
        },
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let filter = match resolve_filter(&o.filter_options, &service) {
        Ok(filter) => filter,
        Err(error) => return report_app_error(error, o.json),
    };
    let criteria = match o.query {
        Some(query) => SearchCriteria::TextSearch {
            search_content: o.search_content.map(Into::into),
            query,
            filter,
            mode,
        },
        None => SearchCriteria::ListRecords { filter },
    };
    let result = match app_result(
        service.search_page(
            SearchResultsRequest {
                sort,
                ranking: Some(o.ranking),
                search_profile: o.search_profile,
                criteria,
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
        super::result_order::print_random_seed(&result.sort);
        print_search_matches(&result.rows);
        super::product::print_search_page_count(&result);
    }
    Ok(ExitCode::SUCCESS)
}
