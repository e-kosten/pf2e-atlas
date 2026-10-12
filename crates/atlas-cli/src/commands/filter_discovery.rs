use super::{filters::build_filter, product::app_result};
use crate::client::{ClientOptions, connect};
use atlas_app_service::{RawFilterCountsRequest, RawFilterValuesRequest};
use atlas_cli_support::write_json_data;
use atlas_domain::{QueryFieldDefinition, QueryFieldType};
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
    let Some(mut c) = app_result(s.discover_filter_fields(), o.json)? else {
        return Ok(ExitCode::from(3));
    };
    if let Some(id) = &o.field
        && !c.fields.iter().any(|field| &field.id == id)
    {
        return Err(format!(
            "Unknown field `{id}`; use atlas filters fields --text QUERY"
        ));
    }
    c.fields
        .retain(|field| matches_field(field, o.field.as_deref(), o.text.as_deref()));
    if o.json {
        write_json_data(&c)?;
    } else {
        for f in c.fields {
            println!("{}\t{}\t{}", f.id, type_label(&f.field_type), f.label);
            if o.field.is_none() {
                continue;
            }
            println!(
                "  operators={} source families={} units={}",
                f.operators.join(", "),
                f.family_types.join(", "),
                f.units.as_deref().unwrap_or("none")
            );
            if let Some(scope) = &f.scope {
                println!("  scope={scope}; path within entry={}", f.path);
            }
            if !f.choices.is_empty() {
                println!("  choices={}", f.choices.join(", "));
            }
            println!("  discovery={:?}; {}", f.value_discovery, f.basis);
            for example in f.examples {
                println!("  {example}");
            }
        }
        if o.field.is_none() {
            println!("\nUse --field ID for operators, applicability, choices and CEL examples.");
            println!(
                "Use --text QUERY to find fields, or filters values --field ID for values and statistics."
            );
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
    let Some(catalog) = app_result(s.discover_filter_fields(), o.json)? else {
        return Ok(ExitCode::from(3));
    };
    let definition = catalog
        .fields
        .iter()
        .find(|field| field.id == o.field)
        .ok_or_else(|| format!("Unknown field `{}`", o.field))?;
    let statistics_only = matches!(
        definition.field_type,
        QueryFieldType::Number | QueryFieldType::Collection
    );
    if statistics_only && (o.text.is_some() || o.offset != 0 || o.limit != 50) {
        return Err("--text, --offset and --limit apply to sampled values, not numeric or collection statistics".into());
    }
    let values = if statistics_only {
        None
    } else {
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
        Some(values)
    };
    if o.json {
        write_json_data(serde_json::json!({"values":values,"counts":counts}))?;
    } else {
        println!("{} ({})", definition.label, definition.id);
        if let Some(values) = values {
            for v in &values.options {
                println!(
                    "{}\t{} roots",
                    serde_json::to_string(&v.value).map_err(|e| e.to_string())?,
                    v.distinct_roots
                );
            }
            println!(
                "{} of {} values shown{}",
                values.options.len(),
                values.total_values,
                if values.exhaustive {
                    ""
                } else {
                    "; additional values are available"
                }
            );
        }
        if definition.field_type == QueryFieldType::Number {
            match (&counts.minimum, &counts.maximum) {
                (Some(lo), Some(hi)) => println!(
                    "Range: {lo}–{hi}{}",
                    definition
                        .units
                        .as_ref()
                        .map(|u| format!(" {u}"))
                        .unwrap_or_default()
                ),
                _ => println!("No known numeric values in this selection."),
            }
        }
        if definition.field_type == QueryFieldType::Collection {
            println!(
                "Use filters fields --text '{}' to find fields within this collection.",
                definition.id
            );
        }
        println!("Availability ({}):", counts.counting_scope);
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

fn matches_field(field: &QueryFieldDefinition, id: Option<&str>, text: Option<&str>) -> bool {
    id.is_none_or(|id| field.id == id)
        && text.is_none_or(|query| {
            let query = query.to_lowercase();
            [&field.id, &field.label]
                .into_iter()
                .chain(field.scope.iter())
                .chain(field.family_types.iter())
                .any(|value| value.to_lowercase().contains(&query))
        })
}
fn type_label(field_type: &QueryFieldType) -> &'static str {
    match field_type {
        QueryFieldType::String => "text",
        QueryFieldType::Number => "number",
        QueryFieldType::Boolean => "yes/no",
        QueryFieldType::Set => "set",
        QueryFieldType::Collection => "collection",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn field_discovery_uses_labels_and_stable_ids_with_explicit_scope() {
        let catalog = atlas_index::query_capabilities().unwrap();
        let hp = catalog
            .fields
            .iter()
            .find(|f| f.id == "actor.hp.maximum")
            .unwrap();
        assert_eq!(hp.label, "Maximum HP");
        assert!(matches_field(hp, None, Some("maximum hp")));
        assert!(matches_field(hp, None, Some("hazard")));
        assert!(!matches_field(hp, Some("shield.hp.maximum"), None));
        assert_eq!(
            catalog
                .fields
                .iter()
                .filter(|f| matches_field(f, Some("actor.hp.maximum"), None))
                .count(),
            1
        );
        assert!(
            catalog
                .fields
                .iter()
                .all(|f| !f.label.is_empty() && !f.label.contains('.'))
        );
    }
}
