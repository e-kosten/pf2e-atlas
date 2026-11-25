//! Roll a selected published table once using its checked source generation.
use super::{
    args::{RollTableOptions, TerminalDetail},
    json,
    navigation::{self, Locator},
    render_error, unique_match,
};
use crate::{
    client::{ClientOptions, connect},
    commands::product::report_app_error,
};
use atlas_app_model::{AppError, AppErrorCode, RollTableRequest};
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService};
use atlas_domain::RecordKey;
use std::process::ExitCode;

pub(crate) fn run(o: RollTableOptions) -> Result<ExitCode, String> {
    let service = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
    }) {
        Ok(s) => s,
        Err(error) => return report_app_error(error, o.json),
    };
    let result = (|| {
        let locator = select(&service, o.record_ref.as_deref(), o.navigation.as_deref())?;
        let rolled = match service.roll_table_full(RollTableRequest {
            selection: locator.request().into_selection(),
            artifact_generation: locator.artifact_generation.clone(),
        }) {
            Ok(roll) => roll,
            Err(error) => {
                // A rebuild between resolution and the roll may have removed the selected row.
                // Check before reporting this as a missing or invalid document.
                let error = error.into_app_error();
                if matches!(
                    error.code,
                    AppErrorCode::InvalidRequest | AppErrorCode::RecordNotFound
                ) {
                    service
                        .record_summaries_at(vec![], Some(locator.artifact_generation.clone()))
                        .map_err(|e| e.into_app_error())?;
                }
                return Err(error);
            }
        };
        if o.json {
            let matches = rolled
                .matches
                .iter()
                .map(|row| json::selected(row, TerminalDetail::Standard, &[], &[], None))
                .collect::<Result<Vec<_>, _>>()
                .map_err(render_error)?;
            let invalid: Vec<_> = rolled
                .invalid_ranges
                .iter()
                .map(|a| Locator::new(&locator.artifact_generation, a))
                .collect();
            let table = json::header(
                &rolled.table,
                &locator.artifact_generation,
                &locator.address,
            );
            atlas_cli_support::write_json_data(serde_json::json!({
                "table": table,
                "outcome": rolled.outcome,
                "matches": matches,
                "invalid_ranges": invalid,
            }))
            .map_err(render_error)?;
        } else {
            super::print_header(&rolled.table);
            println!("{}", crate::commands::roll::render_roll(&rolled.outcome));
            if rolled.matches.is_empty() {
                println!("No range matched the roll.");
            }
            for row in rolled.matches {
                let mut output =
                    super::render_selected_record(&service, &row, TerminalDetail::Standard)?;
                super::links::append(&mut output, &row).map_err(render_error)?;
                print!("{}", super::terminal::wrap_facts(&output, super::width()));
                println!(
                    "  Navigation: {}",
                    serde_json::json!(Locator::new(&row.artifact_generation, &row.selected))
                );
            }
            for address in rolled.invalid_ranges {
                println!(
                    "Unavailable range: {}",
                    serde_json::json!(Locator::new(&locator.artifact_generation, &address))
                );
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(ExitCode::SUCCESS),
        Err(error) => report_app_error(error, o.json),
    }
}

fn select(
    service: &AtlasAppService,
    reference: Option<&str>,
    navigation: Option<&Locator>,
) -> Result<Locator, AppError> {
    if let Some(locator) = navigation {
        if locator.address.field.is_some()
            || locator.address.passage.is_some()
            || locator.address.spell_rank.is_some()
        {
            return Err(AppError::new(
                AppErrorCode::InvalidRequest,
                "Choose a table document to roll.",
            ));
        }
        return Ok(locator.clone());
    }
    let reference =
        reference.ok_or_else(|| AppError::new(AppErrorCode::InvalidRequest, "Choose a table."))?;
    let (generation, record) = if let Ok(key) = RecordKey::parse(reference) {
        let (generation, rows) = service
            .record_summaries_at(vec![key], None)
            .map_err(|e| e.into_app_error())?;
        (
            generation,
            rows.into_iter().next().ok_or_else(|| {
                AppError::new(
                    AppErrorCode::RecordNotFound,
                    format!("record `{reference}` was not found"),
                )
            })?,
        )
    } else {
        let (generation, rows) = service
            .resolve_record_with_generation(reference.to_string(), None)
            .map_err(|e| e.into_app_error())?;
        (generation, unique_match(rows, 5)?.record)
    };
    let navigation = Locator::new(&generation, &navigation::root(&record.record_key));
    Ok(navigation)
}
