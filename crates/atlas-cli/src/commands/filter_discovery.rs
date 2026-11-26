use super::{filters::resolve_filter, product::app_result};
use crate::client::{ClientOptions, connect};
use atlas_app_service::{CliFilterValues, CliFilterValuesRequest};
use atlas_cli_support::write_json_data;
use atlas_domain::{QueryFieldDefinition, QueryFieldType};
use std::process::ExitCode;
pub(crate) mod args;
use args::{FiltersFieldsOptions, FiltersValuesOptions};
pub(crate) fn run_filters_fields(o: FiltersFieldsOptions) -> Result<ExitCode, String> {
    if o.field.is_none() && !o.all && o.text.is_none() {
        if o.detail.is_none() && o.group.is_none() {
            let groups = atlas_app_service::filter_group_names();
            if o.json {
                let groups: Vec<_> = groups
                    .map(|(id, label)| serde_json::json!({"id": id, "label": label}))
                    .collect();
                write_json_data(serde_json::json!({"groups": groups}))?;
            } else {
                for (id, label) in groups {
                    println!("{id}\t{label}");
                }
                print_group_help();
            }
            return Ok(ExitCode::SUCCESS);
        }
        if let Some(id) = &o.group
            && !o.json
        {
            let group = match app_result(atlas_app_service::filter_group_fields(id), false)? {
                std::ops::ControlFlow::Continue(group) => group,
                std::ops::ControlFlow::Break(exit) => return Ok(exit),
            };
            let Some((label, fields)) = group else {
                return unknown_group(id, false);
            };
            println!("{id}\t{label}");
            for field in fields.iter().filter(|f| f.scope.is_none()) {
                println!(
                    "  {}\t{}\t{}",
                    field.id,
                    type_label(&field.field_type),
                    field.label
                );
            }
            print_group_help();
            return Ok(ExitCode::SUCCESS);
        }
        let s = match connect(ClientOptions {
            search_profile: None,
            path_mode: crate::cli::runtime_path_mode(o.path_mode),
            index: o.index,
            embedding_cache: None,
            retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
        }) {
            Ok(service) => service,
            Err(error) => return crate::commands::product::report_app_error(error, o.json),
        };
        let result = if let Some(id) = &o.group {
            s.discover_cli_filter_group(id.clone())
        } else {
            s.discover_filter_editor(atlas_app_model::DiscoverFilterEditorRequest {
                filter: None,
                selected_field_ids: vec![],
                expected_artifact_generation: None,
            })
            .map(Some)
        };
        match app_result(result, o.json)? {
            std::ops::ControlFlow::Continue(Some(editor)) => write_json_data(&editor)?,
            std::ops::ControlFlow::Continue(None) => {
                return unknown_group(o.group.as_deref().unwrap_or_default(), o.json);
            }
            std::ops::ControlFlow::Break(exit) => return Ok(exit),
        }
        return Ok(ExitCode::SUCCESS);
    }
    let mut c = match app_result(atlas_app_service::discover_filter_fields(), o.json)? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    if let Some(id) = &o.field
        && !c.fields.iter().any(|field| &field.id == id)
    {
        if let Some(collection) = id.strip_suffix("[]")
            && c.fields.iter().any(|field| {
                field.id == collection && field.field_type == QueryFieldType::Collection
            })
        {
            return super::product::command_error(
                o.json,
                "invalid_input",
                format!(
                    "`{id}` identifies collection entries.\nUse atlas filters fields --field {collection} to inspect the collection and its fields."
                ),
                2,
            );
        }
        return super::product::command_error(
            o.json,
            "invalid_input",
            format!("Unknown field `{id}`; use atlas filters fields --text QUERY"),
            2,
        );
    }
    if o.json {
        c.fields
            .retain(|field| matches_field(field, o.field.as_deref(), o.text.as_deref()));
        write_json_data(&c)?;
    } else {
        for f in c
            .fields
            .iter()
            .filter(|field| matches_field(field, o.field.as_deref(), o.text.as_deref()))
        {
            println!("{}\t{}\t{}", f.id, type_label(&f.field_type), f.label);
            if o.field.is_none() {
                continue;
            }
            println!("  Operators: {}", f.operators.join(", "));
            if !f.family_types.is_empty() {
                println!("  Source families: {}", f.family_types.join(", "));
            }
            if let Some(units) = &f.units {
                println!("  Units: {units}");
            }
            if let Some(scope) = &f.scope {
                println!("  Collection: {scope}; field within each entry: {}", f.path);
                println!("  Combine conditions inside one exists(...) to match the same entry.");
            } else if f.field_type == QueryFieldType::Collection {
                println!("  Use exists(...) to match an entry in this collection.");
            }
            if !f.choice_groups.is_empty() {
                println!("  Kind selectors:");
                for id in &f.choices {
                    if f.choice_groups.iter().any(|g| g.children.contains(id)) {
                        continue;
                    }
                    println!("    {id}");
                    if let Some(group) = f.choice_groups.iter().find(|g| g.id == *id) {
                        for child in &group.children {
                            println!("      {child}");
                        }
                        if let Some(prefix) = &group.dynamic_child_prefix {
                            println!("      {prefix}<pack-id>");
                        }
                    }
                }
            } else if !f.choices.is_empty() {
                println!("  Allowed choices: {}", f.choices.join(", "));
            }
            println!("  Basis: {}", f.basis);
            println!("  CEL examples:");
            for example in &f.examples {
                println!("  {example}");
            }
            if f.field_type == QueryFieldType::Collection {
                println!("  Entry fields:");
                for child in c
                    .fields
                    .iter()
                    .filter(|child| child.scope.as_deref() == Some(f.path.as_str()))
                {
                    println!(
                        "    {}\t{}\t{}",
                        child.id,
                        type_label(&child.field_type),
                        child.label
                    );
                }
                println!(
                    "  Inspect an entry field with --field ID; discover its values with filters values --field ID."
                );
                println!(
                    "  Combine entry conditions inside one exists(...) to match the same entry."
                );
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
fn unknown_group(id: &str, json: bool) -> Result<ExitCode, String> {
    super::product::command_error(
        json,
        "invalid_input",
        format!("Unknown group `{id}`; use atlas filters fields to list groups"),
        2,
    )
}
fn print_group_help() {
    println!(
        "\nUse --group ID to browse, --field ID to inspect, or --text QUERY to search all fields."
    );
}

pub(crate) fn run_filters_values(o: FiltersValuesOptions) -> Result<ExitCode, String> {
    let s = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    }) {
        Ok(service) => service,
        Err(error) => return crate::commands::product::report_app_error(error, o.json),
    };
    let filter = match resolve_filter(&o.filter_options, &s, o.artifact_generation.as_deref()) {
        Ok(filter) => filter,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    let context = match o.list_ref {
        Some(list_ref) => atlas_app_model::FilterDiscoveryContext::SavedList {
            connection_scope: None,
            search_content: None,
            list_ref,
            filter,
            text: None,
            search_profile: None,
            expected_artifact_generation: o.artifact_generation,
        },
        None => atlas_app_model::FilterDiscoveryContext::Filtered {
            connection_scope: None,
            search_content: None,
            filter,
            text: None,
            search_profile: None,
            expected_artifact_generation: o.artifact_generation,
        },
    };
    let definition = match app_result(atlas_app_service::filter_field_definition(&o.field), o.json)?
    {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    let statistics_only = matches!(
        definition.field_type,
        QueryFieldType::Number | QueryFieldType::Collection
    );
    if statistics_only && (o.text.is_some() || o.offset != 0 || o.limit != 50) {
        return super::product::command_error(
            o.json,
            "invalid_input",
            "--text, --offset and --limit apply only to sampled values.".into(),
            2,
        );
    }
    let CliFilterValues { counts, values } = match app_result(
        s.discover_cli_filter_values(CliFilterValuesRequest {
            field: definition,
            context,
            clause_id: o.clause_id,
            text: o.text,
            offset: o.offset,
            limit: o.limit,
        }),
        o.json,
    )? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    if o.json {
        write_json_data(serde_json::json!({"values":values,"counts":counts}))?;
    } else {
        println!("{} ({})", definition.label, definition.id);
        if let Some(values) = values {
            for v in &values.options {
                println!(
                    "{}\t{}\t{} records{}",
                    serde_json::to_string(&v.value).map_err(|e| e.to_string())?,
                    v.label,
                    v.distinct_roots,
                    v.help
                        .as_ref()
                        .and_then(|help| help.definition_key.as_ref())
                        .map(|key| format!("\tRead with: atlas record definition {key}"))
                        .unwrap_or_default()
                );
            }
            println!(
                "{} of {} values shown",
                values.options.len(),
                values.total_values
            );
            let next_offset = o.offset.saturating_add(values.options.len());
            if (next_offset as u64) < values.total_values {
                println!("Next page: repeat with --offset {next_offset}");
            }
            println!("Counts are distinct parent records for each suggested value.");
            if definition.scope.is_some() {
                println!(
                    "An existing collection filter may match a different entry; use one exists(...) for same-entry constraints."
                );
            }
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
        println!("Source availability ({}):", counts.counting_scope);
        for state in counts.states {
            println!(
                "{}\t{} records\t{} occurrences",
                state.state.as_str(),
                state.distinct_roots,
                state.occurrences
            );
        }
        if definition.scope.is_some() {
            println!(
                "Occurrences count collection entries; a parent record can contribute more than one."
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
            .find(|f| f.id == "defenses.hp.maximum")
            .unwrap();
        assert_eq!(hp.label, "Maximum HP");
        assert!(matches_field(hp, None, Some("maximum hp")));
        assert!(matches_field(hp, None, Some("hazard")));
        assert!(!matches_field(hp, Some("shield.hp.maximum"), None));
        assert_eq!(
            catalog
                .fields
                .iter()
                .filter(|f| matches_field(f, Some("defenses.hp.maximum"), None))
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
