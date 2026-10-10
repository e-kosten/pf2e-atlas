//! Product presentation and error handling; app-service owns product projections.
use atlas_app_service::AppServiceResult;
use atlas_cli_support::{write_json_data, write_json_error_data};
use serde::Serialize;
use std::process::ExitCode;
pub(crate) fn app_result<T>(result: AppServiceResult<T>, json: bool) -> Result<Option<T>, String> {
    match result {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            let e = e.into_app_error();
            if json {
                let code = serde_json::to_value(e.code).map_err(|e| e.to_string())?;
                write_json_error_data(
                    code.as_str().ok_or("app error code was not a string")?,
                    e.message,
                    e.details.unwrap_or(serde_json::Value::Null),
                )?;
                Ok(None)
            } else {
                Err(e.message)
            }
        }
    }
}
pub(crate) fn emit<T: Serialize>(value: Option<T>, json: bool) -> Result<ExitCode, String> {
    let Some(value) = value else {
        return Ok(ExitCode::from(3));
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
        "{}\t{}\t{}\t{}",
        view.record_key,
        view.title,
        view.level_label.as_deref().unwrap_or("-"),
        view.kind_label
    );
}
pub(crate) fn print_search_matches(
    records: &[atlas_app_model::ResultWindowRow],
) -> Result<(), String> {
    for r in records {
        print_summary(&r.record);
        for w in r.matches.iter().take(3) {
            println!(
                "  {:?}: {} owners={} field={} label={}",
                w.lane,
                w.navigation.record_key,
                serde_json::to_string(&w.navigation.owners).map_err(|e| e.to_string())?,
                w.navigation.field.as_deref().unwrap_or("identity"),
                w.label.as_deref().unwrap_or("")
            );
            if let Some(s) = &w.snippet {
                println!("    {s}");
            }
        }
    }
    Ok(())
}
pub(crate) fn search_json(result: &atlas_app_model::SearchResultsView) -> serde_json::Value {
    serde_json::json!({"pagination":result.page,"coverage":result.coverage,"results":result.rows})
}
