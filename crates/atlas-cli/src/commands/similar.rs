use super::{filters::build_filter, product::app_result};
use crate::client::{ClientOptions, connect};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;
pub(crate) mod args;
use args::SimilarOptions;
pub(crate) fn run_similar(o: SimilarOptions) -> Result<ExitCode, String> {
    let (filter, _) = build_filter(&o.filter_options).map_err(|e| e.message)?;
    let s = connect(ClientOptions {
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandStoredVectors,
    })
    .map_err(|e| e.message)?;
    let Some(result) = app_result(
        s.similar_records(
            o.record_ref,
            filter,
            atlas_app_model::SearchPageRequest {
                number: 1,
                size: o.limit,
            },
        ),
        o.json,
    )?
    else {
        return Ok(ExitCode::from(3));
    };
    let Some(result) = result else {
        return Err("seed record not found".into());
    };
    if o.json {
        write_json_data(&result)?;
    } else {
        println!("Similar to {}", result.seed.title);
        for row in &result.results.rows {
            println!("{}\t{}", row.record.record_key, row.record.title);
            for witness in &row.matches {
                println!(
                    "  {:?}: {}",
                    witness.lane,
                    witness.label.as_deref().unwrap_or("identity")
                );
            }
        }
        println!(
            "count basis={} bounded window={:?}",
            result.results.coverage.count_basis, result.results.coverage.semantic_unit_window
        );
    }
    Ok(ExitCode::SUCCESS)
}
