use super::product::{app_result, emit};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    BatchAddSavedListItemsRequest, BatchSavedListItemInput, BatchSavedListItemMutationView,
    CreateSavedListRequest, ImportSavedListRequest, RemoveSavedListItemRequest,
    SavedListDetailView, UpdateSavedListRequest,
};
use atlas_app_service::AtlasAppService;
use atlas_cli_support::write_json_data;
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
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
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
        emit(app_result(s.create_saved_list(request), o.json)?, o.json)
    } else {
        let items = o
            .records
            .into_iter()
            .map(|record_ref| BatchSavedListItemInput {
                record_ref,
                note: None,
            })
            .collect();
        emit_batch(s.create_saved_list_with_items(request, items), o.json)
    }
}

fn emit_batch(
    result: atlas_app_service::AppServiceResult<BatchSavedListItemMutationView>,
    json: bool,
) -> Result<ExitCode, String> {
    let view = match app_result(result, json)? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    let failed = view.failed_count > 0;
    if json {
        write_json_data(&view)?;
    } else {
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
    Ok(if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

pub(crate) fn run_lists_ls(o: ListLsOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    emit(app_result(s.saved_lists(), o.json)?, o.json)
}
pub(crate) fn run_lists_show(o: ListShowOptions) -> Result<ExitCode, String> {
    let sort = super::result_order::requested_sort(&o.sort, o.seed)?;
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    let mut view = match app_result(
        if sort.is_some() {
            s.filter_saved_list(atlas_app_model::FilterSavedListRequest {
                sort,
                list_ref: o.slug.clone(),
                query: None,
                filter: None,
            })
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
fn print_list(v: &SavedListDetailView) {
    println!("{} — {}", v.list.slug, v.list.name);
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
        return Err("provide record references or --stdin".into());
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
        o.json,
    )
}

pub(crate) fn run_lists_remove(o: ListRemoveOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    emit(
        app_result(
            s.remove_saved_list_item(RemoveSavedListItemRequest {
                list_ref: o.slug,
                record_ref: o.record_ref,
            }),
            o.json,
        )?,
        o.json,
    )
}
pub(crate) fn run_lists_delete(o: ListDeleteOptions) -> Result<ExitCode, String> {
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    emit(app_result(s.delete_saved_list(&o.slug), o.json)?, o.json)
}
pub(crate) fn run_lists_edit(o: ListEditOptions) -> Result<ExitCode, String> {
    if o.clear_description && o.description.is_some() {
        return Err("--description conflicts with --clear-description".into());
    }
    if o.clear_tags && !o.tags.is_empty() {
        return Err("--tag conflicts with --clear-tags".into());
    }
    let s = match client(o.paths) {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    let v = match app_result(s.saved_list(&o.list_ref), o.json)? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    let request = UpdateSavedListRequest {
        list_key: v.list.list_key,
        slug: o.id.unwrap_or(v.list.slug),
        name: o.name.unwrap_or(v.list.name),
        description: if o.clear_description {
            None
        } else {
            o.description.or(v.list.description)
        },
        tags: if o.clear_tags {
            Vec::new()
        } else if o.tags.is_empty() {
            v.list.tags
        } else {
            o.tags
        },
    };
    emit(app_result(s.update_saved_list(request), o.json)?, o.json)
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
        fs::write(p, json).map_err(|e| e.to_string())?;
    } else {
        println!("{json}");
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_lists_import(o: ListImportOptions) -> Result<ExitCode, String> {
    let document = serde_json::from_slice(&fs::read(o.input).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
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
    emit(
        app_result(
            s.update_saved_list_item_note(atlas_app_model::UpdateSavedListItemNoteRequest {
                list_ref: o.slug,
                record_key: o.record_key,
                note: o.text.filter(|text| !text.trim().is_empty()),
            }),
            o.json,
        )?,
        o.json,
    )
}
