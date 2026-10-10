use super::{
    filters::build_filter,
    product::{app_result, print_search_matches, print_summary, search_json},
};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{RetrievalModeView, SearchPageRequest};
use atlas_cli_support::{write_json_data, write_json_error};
use std::process::ExitCode;
pub(crate) mod args;
use args::SearchOptions;
pub(crate) fn run_search(o: SearchOptions) -> Result<ExitCode, String> {
    let (filter, value) = match build_filter(&o.filter_options) {
        Ok(v) => v,
        Err(e) => {
            if o.json {
                write_json_error(e.code, e.message)?;
                return Ok(ExitCode::from(2));
            }
            return Err(e.message);
        }
    };
    if o.print_filter {
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
    let page = SearchPageRequest {
        number: o.page,
        size: o.limit,
    };
    let mode: RetrievalModeView = o.retrieval.into();
    let s = connect(ClientOptions {
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: o.embedding_cache_path,
        retrieval_mode: if o.query.is_some() && mode != RetrievalModeView::Lexical {
            atlas_app_service::AppServiceRetrievalMode::FullPool
        } else {
            atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings
        },
    })
    .map_err(|e| e.message)?;
    if let Some(query) = o.query {
        let Some(result) = app_result(s.search_text(query, filter, page, mode), o.json)? else {
            return Ok(ExitCode::from(3));
        };
        if o.json {
            write_json_data(search_json(&result))?;
        } else {
            print_search_matches(&result.rows)?;
            println!(
                "count={} basis={} exhaustive={} semantic unit window={:?}",
                result.page.total,
                result.coverage.count_basis,
                result.coverage.exhaustive,
                result.coverage.semantic_unit_window
            );
        }
    } else {
        let Some(result) = app_result(s.list_records(filter, page), o.json)? else {
            return Ok(ExitCode::from(3));
        };
        if o.json {
            write_json_data(
                serde_json::json!({"pagination":result.page,"results":result.records}),
            )?;
        } else {
            for r in result.records {
                print_summary(&r);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
