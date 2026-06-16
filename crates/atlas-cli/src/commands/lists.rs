use std::process::ExitCode;

use atlas_app_model::{
    AddSavedListItemRequest, AppError, AppErrorCode, CreateSavedListRequest, DeleteSavedListView,
    RecordSummaryView, RemoveSavedListItemRequest, SavedListDetailView, SavedListItemMutationView,
    SavedListItemStatusView, SavedListItemView, SavedListSummaryView,
};
use serde::Serialize;

use crate::client::{
    AtlasClient, AtlasClientConfig, AtlasClientHandle, LocalAtlasClientOptions, connect,
};
use crate::output::{write_json_data, write_json_error, write_json_error_data};

pub(crate) mod args;

use args::{
    ListAddOptions, ListCreateOptions, ListDeleteOptions, ListLsOptions, ListRemoveOptions,
    ListShowOptions, ListsPathOptions,
};

#[derive(Debug, Serialize)]
struct ListsData {
    local_state_path: Option<String>,
    lists: Vec<SavedListSummaryView>,
}

#[derive(Debug, Serialize)]
struct ListData {
    local_state_path: Option<String>,
    list: SavedListSummaryView,
}

#[derive(Debug, Serialize)]
struct ListDeleteData {
    local_state_path: Option<String>,
    slug: String,
    deleted: bool,
}

#[derive(Debug, Serialize)]
struct ListItemMutationData {
    local_state_path: Option<String>,
    slug: String,
    record_key: String,
    outcome: &'static str,
}

#[derive(Debug, Serialize)]
struct ListShowData {
    local_state_path: Option<String>,
    list: SavedListSummaryView,
    items: Vec<ListShowItem>,
}

#[derive(Debug, Serialize)]
struct ListShowItem {
    record_key: String,
    position: i64,
    note: Option<String>,
    status: &'static str,
    snapshot: ListItemSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<ListItemRecord>,
}

#[derive(Debug, Serialize)]
struct ListItemSnapshot {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListItemRecord {
    record_key: String,
    name: String,
    kind: String,
}

enum ListCommandStep<T> {
    Ready(T),
    Exit(ExitCode),
}

pub(crate) fn run_lists_create(options: ListCreateOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.create_saved_list(CreateSavedListRequest {
        slug: options.slug,
        name: options.name,
        description: options.description,
    }) {
        Ok(view) => view,
        Err(error) => return app_error(error, options.json),
    };
    let data = ListData {
        local_state_path: local_state_path(&client),
        list: view.list,
    };
    if options.json {
        write_json_data(data)?;
    } else {
        println!("created saved list {} ({})", data.list.slug, data.list.name);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_ls(options: ListLsOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.saved_lists() {
        Ok(view) => view,
        Err(error) => return app_error(error, options.json),
    };
    let data = ListsData {
        local_state_path: local_state_path(&client),
        lists: view.lists,
    };
    if options.json {
        write_json_data(data)?;
    } else if data.lists.is_empty() {
        println!("no saved lists");
    } else {
        for list in &data.lists {
            println!("{}\t{}", list.slug, list.name);
        }
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_show(options: ListShowOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.saved_list(&options.slug) {
        Ok(view) => view,
        Err(error) => return app_error(error, options.json),
    };
    let data = list_show_data(&client, view);
    if options.json {
        write_json_data(data)?;
    } else {
        print_saved_list(&data);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_add(options: ListAddOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.add_saved_list_item(AddSavedListItemRequest {
        slug: options.slug,
        record_ref: options.record_ref,
        note: options.note,
    }) {
        Ok(view) => view,
        Err(error) => return app_error(error, options.json),
    };
    write_mutation_result(&client, view, options.json)
}

pub(crate) fn run_lists_remove(options: ListRemoveOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.remove_saved_list_item(RemoveSavedListItemRequest {
        slug: options.slug,
        record_ref: options.record_ref,
    }) {
        Ok(view) => view,
        Err(error) => return app_error(error, options.json),
    };
    write_mutation_result(&client, view, options.json)
}

pub(crate) fn run_lists_delete(options: ListDeleteOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.delete_saved_list(&options.slug) {
        Ok(view) => view,
        Err(error) => return app_error(error, options.json),
    };
    let data = delete_data(&client, view);
    if options.json {
        write_json_data(data)?;
    } else if data.deleted {
        println!("deleted saved list {}", data.slug);
    } else {
        println!("saved list not found {}", data.slug);
    }
    Ok(ExitCode::SUCCESS)
}

fn lists_client(
    paths: &ListsPathOptions,
    json: bool,
) -> Result<ListCommandStep<AtlasClientHandle>, String> {
    let config = AtlasClientConfig::Local(LocalAtlasClientOptions {
        path_mode: paths.path_mode.into(),
        index_path: paths.index.clone(),
        embedding_cache_root: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    });
    match connect(config) {
        Ok(client) => Ok(ListCommandStep::Ready(client)),
        Err(error) if json => {
            write_app_json_error(error)?;
            Ok(ListCommandStep::Exit(ExitCode::from(3)))
        }
        Err(error) => Err(error.message),
    }
}

fn list_show_data(client: &impl AtlasClient, view: SavedListDetailView) -> ListShowData {
    ListShowData {
        local_state_path: local_state_path(client),
        list: view.list,
        items: view.items.into_iter().map(list_show_item).collect(),
    }
}

fn list_show_item(item: SavedListItemView) -> ListShowItem {
    ListShowItem {
        record_key: item.record_key,
        position: item.position,
        note: item.note,
        status: list_item_status_text(item.status),
        snapshot: ListItemSnapshot {
            name: item.snapshot.title,
            kind: item.snapshot.kind,
        },
        record: item.record.map(list_item_record),
    }
}

fn list_item_record(record: RecordSummaryView) -> ListItemRecord {
    ListItemRecord {
        record_key: record.record_key,
        name: record.title,
        kind: record.kind,
    }
}

fn write_mutation_result(
    client: &impl AtlasClient,
    view: SavedListItemMutationView,
    json: bool,
) -> Result<ExitCode, String> {
    let data = ListItemMutationData {
        local_state_path: local_state_path(client),
        slug: view.slug,
        record_key: view.record_key,
        outcome: mutation_outcome_text(view.outcome),
    };
    if json {
        write_json_data(data)?;
    } else {
        println!("{}\t{}\t{}", data.outcome, data.slug, data.record_key);
    }
    Ok(ExitCode::SUCCESS)
}

fn delete_data(client: &impl AtlasClient, view: DeleteSavedListView) -> ListDeleteData {
    ListDeleteData {
        local_state_path: local_state_path(client),
        slug: view.slug,
        deleted: view.deleted,
    }
}

fn local_state_path(client: &impl AtlasClient) -> Option<String> {
    client.local_state_path().map(str::to_string)
}

fn mutation_outcome_text(
    outcome: atlas_app_model::SavedListItemMutationOutcomeView,
) -> &'static str {
    match outcome {
        atlas_app_model::SavedListItemMutationOutcomeView::Added => "added",
        atlas_app_model::SavedListItemMutationOutcomeView::AlreadyPresent => "already_present",
        atlas_app_model::SavedListItemMutationOutcomeView::Removed => "removed",
        atlas_app_model::SavedListItemMutationOutcomeView::NotPresent => "not_present",
    }
}

fn list_item_status_text(status: SavedListItemStatusView) -> &'static str {
    match status {
        SavedListItemStatusView::Active => "active",
        SavedListItemStatusView::Unresolved => "unresolved",
    }
}

fn print_saved_list(data: &ListShowData) {
    println!("{}\t{}", data.list.slug, data.list.name);
    if let Some(description) = &data.list.description {
        println!("{description}");
    }
    for item in &data.items {
        let name = item
            .record
            .as_ref()
            .map(|record| record.name.as_str())
            .unwrap_or(item.snapshot.name.as_str());
        let kind = item
            .record
            .as_ref()
            .map(|record| record.kind.as_str())
            .or(item.snapshot.kind.as_deref())
            .unwrap_or("unknown");
        println!(
            "{}.\t{}\t{}\t{}\t{}",
            item.position, item.record_key, name, kind, item.status
        );
        if let Some(note) = &item.note {
            println!("\tnote: {note}");
        }
    }
}

fn app_error(error: AppError, json: bool) -> Result<ExitCode, String> {
    let (_, exit) = cli_error_code(error.code);
    if json {
        write_app_json_error(error)?;
        return Ok(exit);
    }
    match exit {
        code if code == ExitCode::from(1) => {
            eprintln!("{}", error.message);
            Ok(exit)
        }
        _ => Err(error.message),
    }
}

fn write_app_json_error(error: AppError) -> Result<(), String> {
    let (code, _) = cli_error_code(error.code);
    if let Some(details) = error.details {
        write_json_error_data(code, error.message, details)
    } else {
        write_json_error(code, error.message)
    }
}

fn cli_error_code(code: AppErrorCode) -> (&'static str, ExitCode) {
    match code {
        AppErrorCode::InvalidRequest => ("invalid_saved_list_slug", ExitCode::from(2)),
        AppErrorCode::InvalidRecordKey => ("invalid_record_key", ExitCode::from(2)),
        AppErrorCode::RecordResolutionMiss => ("record_resolution_miss", ExitCode::from(1)),
        AppErrorCode::RecordResolutionAmbiguous => {
            ("record_resolution_ambiguous", ExitCode::from(1))
        }
        AppErrorCode::RecordNotFound => ("record_not_found", ExitCode::from(1)),
        AppErrorCode::SavedListNotFound => ("saved_list_not_found", ExitCode::from(1)),
        AppErrorCode::SavedListAlreadyExists => ("saved_list_already_exists", ExitCode::from(1)),
        AppErrorCode::IndexUnavailable => ("index_unavailable", ExitCode::from(3)),
        AppErrorCode::QueryFailed => ("query_failed", ExitCode::from(3)),
        AppErrorCode::ArtifactNotReady
        | AppErrorCode::ArtifactIncompatible
        | AppErrorCode::VectorReadinessRequired
        | AppErrorCode::EmbeddingModelUnavailable
        | AppErrorCode::SetupRequired
        | AppErrorCode::SetupInProgress => ("index_unavailable", ExitCode::from(3)),
        AppErrorCode::ServiceBusy
        | AppErrorCode::OperationCancelled
        | AppErrorCode::OperationTimeout => ("service_unavailable", ExitCode::from(3)),
        AppErrorCode::FilterInvalid
        | AppErrorCode::FilterFieldInvalid
        | AppErrorCode::FilterOptionInvalid
        | AppErrorCode::FilterFieldNotApplicable
        | AppErrorCode::FilterMetricAmbiguous
        | AppErrorCode::FilterEditorConflict
        | AppErrorCode::WindowNotFound
        | AppErrorCode::WindowExpired
        | AppErrorCode::InternalError => ("local_state_error", ExitCode::from(3)),
    }
}
