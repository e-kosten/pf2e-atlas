use super::product::{app_result, emit};
use super::saved_workspace::{file_error, respond, write_new_file};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    BatchAddSavedListItemsRequest, BatchSavedListItemInput, BatchSavedListItemMutationView,
    CreateSavedListRequest, FilterSavedListRequest, ImportSavedListRequest,
    RemoveSavedListItemRequest, SavedListDetailView,
};
use atlas_app_service::AtlasAppService;
use atlas_cli_support::{JsonStatus, write_json_data, write_json_response};
use serde::Serialize;
use std::{
    fs,
    io::{self, Read},
    process::ExitCode,
};
pub(crate) mod args;
use args::*;
fn client(paths: ListsPathOptions) -> Result<AtlasAppService, atlas_app_model::AppError> {
    connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(paths.path_mode),
        index: paths.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    })
}
pub(crate) fn run_lists_create(o: ListCreateOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    let request = CreateSavedListRequest {
        slug: o.slug,
        name: o.name,
        description: o.description,
        tags: o.tags,
    };
    if o.records.is_empty() {
        let result = s.create_saved_list(request);
        let view = match app_result(result, o.json)? {
            std::ops::ControlFlow::Continue(value) => value,
            std::ops::ControlFlow::Break(exit) => return Ok(exit),
        };
        if o.json {
            write_json_data(&view)?;
        } else {
            println!("Created {} ({})", view.list.name, view.list.slug);
        }
        Ok(ExitCode::SUCCESS)
    } else {
        let items = o
            .records
            .into_iter()
            .map(|record_ref| BatchSavedListItemInput {
                record_ref,
                note: None,
            })
            .collect();
        emit_batch(s.create_saved_list_with_items(request, items), true, o.json)
    }
}

fn emit_batch(
    result: atlas_app_service::AppServiceResult<BatchSavedListItemMutationView>,
    created: bool,
    json: bool,
) -> Result<ExitCode, String> {
    let view = match app_result(result, json)? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    let exit = view
        .items
        .iter()
        .filter_map(|item| item.error.as_ref())
        .map(|error| super::product::exit_class(error.code))
        .max()
        .unwrap_or(0);
    let status = if view.failed_count == 0 {
        JsonStatus::Ok
    } else if view.failed_count == view.requested_count {
        JsonStatus::Error
    } else {
        JsonStatus::Partial
    };
    if json {
        #[derive(Serialize)]
        struct BatchResponse<'a> {
            created: bool,
            #[serde(flatten)]
            batch: &'a BatchSavedListItemMutationView,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            retry_inputs: Vec<&'a str>,
        }
        let retry_inputs = view
            .items
            .iter()
            .filter(|item| item.error.is_some())
            .map(|item| item.input.as_str())
            .collect();
        write_json_response(
            status,
            BatchResponse {
                created,
                batch: &view,
                retry_inputs,
            },
        )?;
    } else {
        if created {
            println!("Created {} ({})", view.slug, view.list_key);
        }
        println!(
            "{}: {} added, {} already present, {} failed",
            view.slug, view.added_count, view.already_present_count, view.failed_count
        );
        for item in &view.items {
            if let Some(error) = &item.error {
                eprintln!("{}: {}", item.input, error.message);
                super::product::print_error_hint(error);
                if let Some(matches) = error
                    .details
                    .as_ref()
                    .and_then(|details| details.get("matches"))
                    .and_then(serde_json::Value::as_array)
                {
                    for candidate in matches {
                        if let Some(record) = candidate.get("record") {
                            eprintln!(
                                "  {} — {} ({})",
                                record["title"].as_str().unwrap_or(""),
                                record["pack"].as_str().unwrap_or(""),
                                record["record_key"].as_str().unwrap_or("")
                            );
                        }
                    }
                }
            } else {
                println!(
                    "  {}: {} ({})",
                    match item.outcome {
                        atlas_app_model::BatchSavedListItemOutcomeView::Added => "Added",
                        atlas_app_model::BatchSavedListItemOutcomeView::AlreadyPresent =>
                            "Already present",
                        atlas_app_model::BatchSavedListItemOutcomeView::Failed => "Failed",
                    },
                    item.record_name.as_deref().unwrap_or(&item.input),
                    item.record_key.as_deref().unwrap_or("")
                );
            }
        }
    }
    if !json && view.failed_count > 0 {
        eprintln!(
            "Retry the failed items with atlas lists add {} --record <RECORD> using the same workspace.",
            view.slug
        );
    }
    Ok(ExitCode::from(exit))
}

pub(crate) fn run_lists_ls(o: ListLsOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    respond(s.saved_lists(o.include_archived), o.json, |view| {
        if view.lists.is_empty() {
            return "No saved lists.\n".into();
        }
        view.lists.iter().map(list_summary).collect()
    })
}
pub(crate) fn run_lists_show(o: ListShowOptions) -> Result<ExitCode, String> {
    let sort = match super::result_order::requested_sort(&o.sort, o.seed) {
        Ok(sort) => sort,
        Err(message) => return super::product::command_error(o.json, "invalid_input", message, 2),
    };
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    let mut view = match app_result(
        if (o.keys_only || o.no_records)
            && matches!(sort, None | Some(atlas_domain::ResultSort::SavedOrder {}))
        {
            s.saved_list_snapshot(&o.slug)
        } else if sort.is_some() {
            s.filter_saved_list(
                &o.slug,
                FilterSavedListRequest {
                    sort,
                    query: None,
                    filter: None,
                },
            )
        } else {
            s.saved_list(&o.slug)
        },
        o.json,
    )? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    if o.keys_only {
        if o.json {
            write_json_data(
                serde_json::json!({"sort":view.sort,"list":view.list,"record_keys":view.items.iter().map(|i|&i.record_key).collect::<Vec<_>>()}),
            )?;
        } else {
            if let atlas_domain::ResultSort::Random { seed } = view.sort {
                eprintln!("Random seed: {seed}");
            }
            for i in view.items {
                println!("{}", i.record_key);
            }
        }
    } else {
        if o.no_records {
            for i in &mut view.items {
                i.record = None;
            }
        }
        if o.json {
            write_json_data(&view)?;
        } else {
            print_list(&view);
        }
    }
    Ok(ExitCode::SUCCESS)
}
fn list_summary(list: &atlas_app_model::SavedListSummaryView) -> String {
    let archived = if list.archived { " · archived" } else { "" };
    let mut text = format!(
        "{} ({}) · {} items{archived}\n  {}\n",
        list.name, list.slug, list.item_count, list.list_key
    );
    if let Some(description) = &list.description {
        text.push_str(&format!("  {description}\n"));
    }
    if !list.tags.is_empty() {
        text.push_str(&format!("  Tags: {}\n", list.tags.join(", ")));
    }
    text
}
fn print_list(v: &SavedListDetailView) {
    print!("{}", list_summary(&v.list));
    if let Some(error) = &v.reference_error {
        eprintln!("Reference records could not be checked. {}", error.message);
    }
    super::result_order::print_random_seed(&v.sort);
    for i in &v.items {
        let name = i
            .record
            .as_ref()
            .map(|r| r.title.as_str())
            .unwrap_or(&i.snapshot.title);
        println!(
            "{}\t{}\t{}\t{:?}{}",
            i.position,
            i.record_key,
            name,
            i.status,
            i.record
                .as_ref()
                .and_then(|r| r.unit_price_label.as_ref())
                .map(|p| format!("\t{p}"))
                .unwrap_or_default()
        );
        if let Some(note) = &i.note {
            println!("  {note}");
        }
    }
}
pub(crate) fn run_lists_add(mut o: ListAddOptions) -> Result<ExitCode, String> {
    o.record_refs.extend(o.records);
    if o.stdin {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| e.to_string())?;
        o.record_refs.extend(
            text.lines()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
        );
    }
    if o.record_refs.is_empty() {
        return super::product::command_error(
            o.json,
            "invalid_input",
            "Provide record references or --stdin.".into(),
            2,
        );
    }
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    let items = o
        .record_refs
        .into_iter()
        .map(|record_ref| BatchSavedListItemInput {
            record_ref,
            note: o.note.clone(),
        })
        .collect();
    emit_batch(
        s.add_saved_list_items(BatchAddSavedListItemsRequest {
            list_ref: o.slug,
            items,
        }),
        false,
        o.json,
    )
}

pub(crate) fn run_lists_remove(o: ListRemoveOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    respond(
        s.remove_saved_list_item(
            &o.slug,
            RemoveSavedListItemRequest {
                record_ref: o.record_ref,
            },
        ),
        o.json,
        |view| {
            let action =
                if view.outcome == atlas_app_model::SavedListItemMutationOutcomeView::NotPresent {
                    "Already absent"
                } else {
                    "Removed"
                };
            format!(
                "{action} · {} ({}) · {} ({})\n",
                view.record_name.as_deref().unwrap_or("Record"),
                view.record_key,
                view.slug,
                view.list_key
            )
        },
    )
}
pub(crate) fn run_lists_delete(o: ListDeleteOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    respond(s.delete_saved_list(&o.slug), o.json, |view| {
        format!("Deleted {} ({})\n", view.slug, view.list_key)
    })
}
pub(crate) fn run_lists_edit(o: ListEditOptions) -> Result<ExitCode, String> {
    if o.clear_description && o.description.is_some() {
        return super::product::command_error(
            o.json,
            "invalid_input",
            "--description conflicts with --clear-description".into(),
            2,
        );
    }
    if o.clear_tags && !o.tags.is_empty() {
        return super::product::command_error(
            o.json,
            "invalid_input",
            "--tag conflicts with --clear-tags".into(),
            2,
        );
    }
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    let description = if o.clear_description {
        Some(None)
    } else {
        o.description.map(Some)
    };
    let tags = if o.clear_tags || !o.tags.is_empty() {
        Some(o.tags)
    } else {
        None
    };
    let archived = if o.archive || o.restore {
        Some(o.archive)
    } else {
        None
    };
    respond(
        s.edit_saved_list_metadata(&o.list_ref, o.id, o.name, description, tags, archived),
        o.json,
        |view| format!("Updated {}", list_summary(&view.list)),
    )
}
pub(crate) fn run_lists_export(o: ListExportOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, false),
    };
    let document = match s.export_saved_list(&o.slug) {
        Ok(value) => value,
        Err(error) => return super::product::report_app_error(error.into_app_error(), false),
    };
    let json = serde_json::to_string_pretty(&document).map_err(|e| e.to_string())?;
    if let Some(p) = o.output {
        if let Err(error) = write_new_file(&p, json.as_bytes()) {
            return file_error(false, error);
        }
    } else {
        println!("{json}");
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_lists_import(o: ListImportOptions) -> Result<ExitCode, String> {
    let bytes = match fs::read(&o.input) {
        Ok(bytes) => bytes,
        Err(error) => {
            return file_error(
                o.json,
                io::Error::new(error.kind(), format!("{}: {error}", o.input.display())),
            );
        }
    };
    let document: atlas_app_model::SavedListExportDocumentView =
        match serde_json::from_slice(&bytes) {
            Ok(document) => document,
            Err(error) => {
                return super::product::command_error(
                    o.json,
                    "invalid_input",
                    format!("{}: {error}", o.input.display()),
                    2,
                );
            }
        };
    if let Err(error) = document.validate_format() {
        return super::product::report_app_error(error, o.json);
    }
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    emit(
        app_result(
            s.import_saved_list(ImportSavedListRequest {
                document,
                id: o.id,
                replace: o.replace,
            }),
            o.json,
        )?,
        o.json,
    )
}

pub(crate) fn run_lists_note(o: ListNoteOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    respond(
        s.set_saved_list_item_note(
            &o.slug,
            &o.record_key,
            o.text.filter(|text| !text.trim().is_empty()),
        ),
        o.json,
        |view| {
            let note = view.note.as_deref().unwrap_or("Note cleared");
            format!(
                "{} ({}) · {} ({})\n  {note}\n",
                view.title, view.record_key, view.slug, view.list_key
            )
        },
    )
}
