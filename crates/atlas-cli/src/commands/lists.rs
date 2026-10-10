use super::product::{app_result, emit};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    BatchAddSavedListItemsRequest, BatchSavedListItemInput, CreateSavedListRequest,
    ImportSavedListRequest, RemoveSavedListItemRequest, SavedListDetailView,
    UpdateSavedListRequest,
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
fn client(paths: ListsPathOptions) -> Result<AtlasAppService, String> {
    connect(ClientOptions {
        path_mode: paths.path_mode.into(),
        index: paths.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .map_err(|e| e.message)
}
pub(crate) fn run_lists_create(o: ListCreateOptions) -> Result<ExitCode, String> {
    let s = client(o.paths)?;
    emit(
        app_result(
            s.create_saved_list(CreateSavedListRequest {
                slug: o.slug,
                name: o.name,
                description: o.description,
                tags: o.tags,
            }),
            o.json,
        )?,
        o.json,
    )
}
pub(crate) fn run_lists_ls(o: ListLsOptions) -> Result<ExitCode, String> {
    let s = client(o.paths)?;
    emit(app_result(s.saved_lists(), o.json)?, o.json)
}
pub(crate) fn run_lists_show(o: ListShowOptions) -> Result<ExitCode, String> {
    let s = client(o.paths)?;
    let Some(mut view) = app_result(s.saved_list(&o.slug), o.json)? else {
        return Ok(ExitCode::from(3));
    };
    if o.keys_only {
        if o.json {
            write_json_data(
                serde_json::json!({"list":view.list,"record_keys":view.items.iter().map(|i|&i.record_key).collect::<Vec<_>>()}),
            )?;
        } else {
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
    for i in &v.items {
        let name = i
            .record
            .as_ref()
            .map(|r| r.title.as_str())
            .unwrap_or(&i.snapshot.title);
        println!("{}\t{}\t{}\t{:?}", i.position, i.record_key, name, i.status);
        if let Some(note) = &i.note {
            println!("  {note}");
        }
    }
}
pub(crate) fn run_lists_add(mut o: ListAddOptions) -> Result<ExitCode, String> {
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
    let s = client(o.paths)?;
    let items = o
        .record_refs
        .into_iter()
        .map(|record_ref| BatchSavedListItemInput {
            record_ref,
            note: o.note.clone(),
        })
        .collect();
    emit(
        app_result(
            s.add_saved_list_items(BatchAddSavedListItemsRequest {
                list_ref: o.slug,
                items,
            }),
            o.json,
        )?,
        o.json,
    )
}
pub(crate) fn run_lists_remove(o: ListRemoveOptions) -> Result<ExitCode, String> {
    let s = client(o.paths)?;
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
    let s = client(o.paths)?;
    emit(app_result(s.delete_saved_list(&o.slug), o.json)?, o.json)
}
pub(crate) fn run_lists_edit(o: ListEditOptions) -> Result<ExitCode, String> {
    if o.clear_description && o.description.is_some() {
        return Err("--description conflicts with --clear-description".into());
    }
    if o.clear_tags && !o.tags.is_empty() {
        return Err("--tag conflicts with --clear-tags".into());
    }
    let s = client(o.paths)?;
    let Some(v) = app_result(s.saved_list(&o.list_ref), o.json)? else {
        return Ok(ExitCode::from(3));
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
    let s = client(o.paths)?;
    let document = s.export_saved_list(&o.slug).map_err(|e| e.to_string())?;
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
    let s = client(o.paths)?;
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
