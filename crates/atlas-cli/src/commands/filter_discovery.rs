use super::{filters::build_filter, product::app_result};
use crate::client::{ClientOptions, connect};
use atlas_app_service::{RawFilterCountsRequest, RawFilterValuesRequest};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;
pub(crate) mod args;
use args::{FiltersFieldsOptions, FiltersValuesOptions};
pub(crate) fn run_filters_fields(o: FiltersFieldsOptions) -> Result<ExitCode, String> {
    let s = connect(ClientOptions {
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .map_err(|e| e.message)?;
    let Some(c) = app_result(s.discover_filter_fields(), o.json)? else {
        return Ok(ExitCode::from(3));
    };
    if o.json {
        write_json_data(&c)?;
    } else {
        for f in c.fields {
            println!("{}\t{:?}\t{}", f.path, f.field_type, f.label);
            println!(
                "  operators={} families={} units={}",
                f.operators.join(", "),
                f.family_types.join(", "),
                f.units.as_deref().unwrap_or("none")
            );
            if !f.choices.is_empty() {
                println!("  choices={}", f.choices.join(", "));
            }
            println!("  discovery={:?}; {}", f.value_discovery, f.basis);
            for example in f.examples {
                println!("  {example}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_filters_values(o: FiltersValuesOptions) -> Result<ExitCode, String> {
    let (filter, _) = build_filter(&o.filter_options).map_err(|e| e.message)?;
    let s = connect(ClientOptions {
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .map_err(|e| e.message)?;
    let Some(counts) = app_result(
        s.discover_raw_filter_counts(RawFilterCountsRequest {
            field: o.field.clone(),
            filter: filter.clone(),
            clause_id: None,
        }),
        o.json,
    )?
    else {
        return Ok(ExitCode::from(3));
    };
    let Some(values) = app_result(
        s.discover_raw_filter_values(RawFilterValuesRequest {
            field: o.field,
            filter,
            clause_id: None,
            text: o.text,
            offset: o.offset,
            limit: o.limit,
        }),
        o.json,
    )?
    else {
        return Ok(ExitCode::from(3));
    };
    if o.json {
        write_json_data(serde_json::json!({"values":values,"counts":counts}))?;
    } else {
        println!("{} ({})", values.field, values.count_basis);
        for v in values.options {
            println!(
                "{}\t{} roots",
                serde_json::to_string(&v.value).map_err(|e| e.to_string())?,
                v.distinct_roots
            );
        }
        println!("minimum={:?} maximum={:?}", counts.minimum, counts.maximum);
        for state in counts.states {
            println!(
                "{}\t{} roots\t{} occurrences",
                state.state.as_str(),
                state.distinct_roots,
                state.occurrences
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}
