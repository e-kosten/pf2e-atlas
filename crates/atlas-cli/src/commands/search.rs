use super::{
    filters::{build_filter, resolve_filter},
    product::{app_result, report_app_error},
};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{RetrievalModeView, SearchCriteria, SearchPageRequest, SearchResultsRequest};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;
pub(crate) mod args;
mod presentation;
use args::SearchOptions;
pub(crate) fn run_search(o: SearchOptions) -> Result<ExitCode, String> {
    let sort = match super::result_order::requested_sort(&o.sort, o.seed) {
        Ok(sort) => sort,
        Err(message) => return super::product::command_error(o.json, "invalid_input", message, 2),
    };
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
    let mode: RetrievalModeView = o.retrieval.unwrap_or(args::CliRetrievalMode::Hybrid).into();
    let use_embeddings = o.query.is_some() && mode != RetrievalModeView::Lexical;
    #[cfg(feature = "web")]
    let runtime = use_embeddings
        .then(|| {
            atlas_runtime::AtlasRuntime::resolve(atlas_runtime::AtlasRuntimeOptions {
                default_search_profile: o.search_profile.clone(),
                path_mode: crate::cli::runtime_path_mode(o.path_mode),
                overrides: atlas_runtime::AtlasPathOverrides {
                    index_path: o.index.clone(),
                    embedding_cache_root: o.embedding_cache_path.clone(),
                    source_root: None,
                },
            })
        })
        .and_then(Result::ok);
    let service = match connect(ClientOptions {
        search_profile: o.search_profile.clone(),
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index.clone(),
        embedding_cache: o.embedding_cache_path.clone(),
        retrieval_mode: if use_embeddings {
            atlas_app_service::AppServiceRetrievalMode::OnDemandEmbeddings
        } else {
            atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference
        },
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let mode = if o.query.is_some() && o.retrieval.is_none() && o.search_profile.is_none() {
        let readiness = service.readiness();
        if readiness.status == atlas_app_model::AppReadinessStatus::Ready
            && readiness.search_profiles.is_empty()
        {
            RetrievalModeView::Lexical
        } else {
            mode
        }
    } else {
        mode
    };
    let filter = match resolve_filter(
        &o.filter_options,
        &service,
        o.artifact_generation.as_deref(),
    ) {
        Ok(filter) => filter,
        Err(error) => return report_app_error(error, o.json),
    };
    let criteria = match o.query.clone() {
        Some(query) => SearchCriteria::TextSearch {
            lexical: None,
            search_content: o.search_content.map(Into::into),
            query,
            filter,
            mode,
        },
        None => SearchCriteria::ListRecords { filter },
    };
    let request = SearchResultsRequest {
        result_detail: atlas_app_model::SearchResultDetail::Evidence,
        connection_scope: None,
        sort,
        ranking: Some(o.ranking),
        search_profile: o.search_profile.clone(),
        criteria,
        page: SearchPageRequest {
            number: o.page,
            size: o.limit,
        },
        expected_artifact_generation: o.artifact_generation.clone(),
    };
    #[cfg(feature = "web")]
    let hosted = runtime
        .as_ref()
        .and_then(|runtime| super::web::managed::search(runtime, &request));
    let direct = || {
        tracing::info!(target: "atlas_cli", "using direct Atlas search");
        service.search_page(
            request,
            &atlas_app_service::ReadExecutionContext::uncancellable(),
        )
    };
    #[cfg(feature = "web")]
    let searched = hosted.unwrap_or_else(direct);
    #[cfg(not(feature = "web"))]
    let searched = direct();
    let result = match app_result(searched, o.json)? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    presentation::write(&o, &result)?;
    Ok(ExitCode::SUCCESS)
}
