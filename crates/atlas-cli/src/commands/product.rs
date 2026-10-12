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
        "{} — {}{}",
        view.title,
        view.kind_label,
        view.level_label
            .as_ref()
            .map(|v| format!(" · {v}"))
            .unwrap_or_default()
    );
    println!("  {}", view.record_key);
}
pub(crate) fn print_search_matches(records: &[atlas_app_model::ResultWindowRow]) {
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
        println!();
    }
}
pub(crate) fn search_json(result: &atlas_app_model::SearchResultsView) -> serde_json::Value {
    serde_json::json!({"pagination":result.page,"coverage":result.coverage,"results":result.rows})
}
