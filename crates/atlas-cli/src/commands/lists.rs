use std::collections::BTreeMap;
use std::process::ExitCode;

use atlas_domain::{DetailLevel, RecordKey};
use atlas_local_state::{
    AddSavedListItemOutcome, HydratedSavedListItem, LocalStateError, LocalStateStore, NewSavedList,
    NewSavedListItem, SavedList, SavedListItem, SavedListItemStatus, hydrate_saved_list_item,
};
use atlas_record::{RecordJson, RecordJsonOptions, record_json};
use atlas_search::{
    GetRecordsRequest, RecordRefResolutionResult, RecordRetrieval, ResolveRecordRefRequest,
};
use serde::Serialize;

use crate::output::{write_json_data, write_json_error, write_json_error_data};

use super::record::{open_record_service, record_runtime, search_error, search_error_code};

pub(crate) mod args;

use args::{
    ListAddOptions, ListCreateOptions, ListDeleteOptions, ListLsOptions, ListRemoveOptions,
    ListShowOptions, ListsPathOptions,
};

#[derive(Debug, Serialize)]
struct ListsData {
    local_state_path: String,
    lists: Vec<SavedList>,
}

#[derive(Debug, Serialize)]
struct ListData {
    local_state_path: String,
    list: SavedList,
}

#[derive(Debug, Serialize)]
struct ListDeleteData {
    local_state_path: String,
    slug: String,
    deleted: bool,
}

#[derive(Debug, Serialize)]
struct ListItemMutationData {
    local_state_path: String,
    slug: String,
    record_key: String,
    outcome: &'static str,
}

#[derive(Debug, Serialize)]
struct ListShowData {
    local_state_path: String,
    list: SavedList,
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
    record: Option<RecordJson>,
}

#[derive(Debug, Serialize)]
struct ListItemSnapshot {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
}

enum ListCommandStep<T> {
    Ready(T),
    Exit(ExitCode),
}

pub(crate) fn run_lists_create(options: ListCreateOptions) -> Result<ExitCode, String> {
    let (store, local_state_path) = match local_state_store(&options.paths, options.json)? {
        ListCommandStep::Ready(store) => store,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let list = match store.create_list(NewSavedList {
        slug: options.slug,
        name: options.name,
        description: options.description,
    }) {
        Ok(list) => list,
        Err(error) => return local_state_error(error, options.json),
    };
    let data = ListData {
        local_state_path,
        list,
    };
    if options.json {
        write_json_data(data)?;
    } else {
        println!("created saved list {} ({})", data.list.slug, data.list.name);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_ls(options: ListLsOptions) -> Result<ExitCode, String> {
    let (store, local_state_path) = match local_state_store(&options.paths, options.json)? {
        ListCommandStep::Ready(store) => store,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let lists = match store.lists() {
        Ok(lists) => lists,
        Err(error) => return local_state_error(error, options.json),
    };
    let data = ListsData {
        local_state_path,
        lists,
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
    let (store, local_state_path) = match local_state_store(&options.paths, options.json)? {
        ListCommandStep::Ready(store) => store,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let Some(list) = (match store.get_list_with_items(&options.slug) {
        Ok(list) => list,
        Err(error) => return local_state_error(error, options.json),
    }) else {
        return list_not_found(&options.slug, options.json);
    };
    let records_by_key = match hydrate_records(&list.items, &options.paths, options.json)? {
        ListCommandStep::Ready(records) => records,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let data = ListShowData {
        local_state_path,
        list: list.list,
        items: list
            .items
            .into_iter()
            .map(|item| show_item(item, &records_by_key))
            .collect(),
    };
    if options.json {
        write_json_data(data)?;
    } else {
        print_saved_list(&data);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_add(options: ListAddOptions) -> Result<ExitCode, String> {
    let runtime = match runtime_for_lists(&options.paths, options.json)? {
        ListCommandStep::Ready(runtime) => runtime,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let service = match record_service_for_lists(&runtime, options.json)? {
        ListCommandStep::Ready(service) => service,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let record = match resolve_record_ref(&service, &options.record_ref, options.json)? {
        RecordRefOutcome::Record(record) => *record,
        RecordRefOutcome::Exit(code) => return Ok(code),
    };
    let (store, local_state_path) = match open_store(runtime.local_state_path(), options.json)? {
        ListCommandStep::Ready(store) => store,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let record_key = record.identity.key.to_string();
    let outcome = match store.add_item(
        &options.slug,
        NewSavedListItem {
            record_key: record_key.clone(),
            note: options.note,
            record_title_snapshot: record.identity.name.clone(),
            record_kind_snapshot: Some(record.classification.kind.as_str().to_string()),
        },
    ) {
        Ok(outcome) => outcome,
        Err(error) => return local_state_error(error, options.json),
    };
    let outcome_text = match outcome {
        AddSavedListItemOutcome::Added => "added",
        AddSavedListItemOutcome::AlreadyPresent => "already_present",
    };
    let data = ListItemMutationData {
        local_state_path,
        slug: options.slug,
        record_key,
        outcome: outcome_text,
    };
    if options.json {
        write_json_data(data)?;
    } else {
        println!("{}\t{}\t{}", data.outcome, data.slug, data.record_key);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_remove(options: ListRemoveOptions) -> Result<ExitCode, String> {
    let runtime = match runtime_for_lists(&options.paths, options.json)? {
        ListCommandStep::Ready(runtime) => runtime,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let record_key = if let Ok(key) = RecordKey::parse(&options.record_ref) {
        key
    } else {
        let service = match record_service_for_lists(&runtime, options.json)? {
            ListCommandStep::Ready(service) => service,
            ListCommandStep::Exit(code) => return Ok(code),
        };
        match resolve_record_ref(&service, &options.record_ref, options.json)? {
            RecordRefOutcome::Record(record) => record.identity.key,
            RecordRefOutcome::Exit(code) => return Ok(code),
        }
    };
    let (store, local_state_path) = match open_store(runtime.local_state_path(), options.json)? {
        ListCommandStep::Ready(store) => store,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let removed = match store.remove_item(&options.slug, &record_key.to_string()) {
        Ok(removed) => removed,
        Err(error) => return local_state_error(error, options.json),
    };
    let data = ListItemMutationData {
        local_state_path,
        slug: options.slug,
        record_key: record_key.to_string(),
        outcome: if removed { "removed" } else { "not_present" },
    };
    if options.json {
        write_json_data(data)?;
    } else {
        println!("{}\t{}\t{}", data.outcome, data.slug, data.record_key);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_delete(options: ListDeleteOptions) -> Result<ExitCode, String> {
    let (store, local_state_path) = match local_state_store(&options.paths, options.json)? {
        ListCommandStep::Ready(store) => store,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let deleted = match store.delete_list(&options.slug) {
        Ok(deleted) => deleted,
        Err(error) => return local_state_error(error, options.json),
    };
    let data = ListDeleteData {
        local_state_path,
        slug: options.slug,
        deleted,
    };
    if options.json {
        write_json_data(data)?;
    } else if data.deleted {
        println!("deleted saved list {}", data.slug);
    } else {
        println!("saved list not found {}", data.slug);
    }
    Ok(ExitCode::SUCCESS)
}

fn local_state_store(
    paths: &ListsPathOptions,
    json: bool,
) -> Result<ListCommandStep<(LocalStateStore, String)>, String> {
    let runtime = match runtime_for_lists(paths, json)? {
        ListCommandStep::Ready(runtime) => runtime,
        ListCommandStep::Exit(code) => return Ok(ListCommandStep::Exit(code)),
    };
    open_store(runtime.local_state_path(), json)
}

fn runtime_for_lists(
    paths: &ListsPathOptions,
    json: bool,
) -> Result<ListCommandStep<atlas_runtime::AtlasRuntime>, String> {
    match record_runtime(paths.path_mode.into(), paths.index.clone()) {
        Ok(runtime) => Ok(ListCommandStep::Ready(runtime)),
        Err(error) if json => {
            write_json_error("runtime_error", error)?;
            Ok(ListCommandStep::Exit(ExitCode::from(3)))
        }
        Err(error) => Err(error),
    }
}

fn open_store(
    path: &std::path::Path,
    json: bool,
) -> Result<ListCommandStep<(LocalStateStore, String)>, String> {
    match LocalStateStore::open(path.to_path_buf()) {
        Ok(store) => Ok(ListCommandStep::Ready((store, path.display().to_string()))),
        Err(error) if json => {
            write_json_error("local_state_error", error.to_string())?;
            Ok(ListCommandStep::Exit(ExitCode::from(3)))
        }
        Err(error) => Err(error.to_string()),
    }
}

fn record_service_for_lists(
    runtime: &atlas_runtime::AtlasRuntime,
    json: bool,
) -> Result<ListCommandStep<atlas_search::AtlasRetrievalService>, String> {
    match open_record_service(runtime) {
        Ok(service) => Ok(ListCommandStep::Ready(service)),
        Err(error) if json => {
            write_json_error("index_unavailable", error)?;
            Ok(ListCommandStep::Exit(ExitCode::from(3)))
        }
        Err(error) => Err(error),
    }
}

enum RecordRefOutcome {
    Record(Box<atlas_record::AtlasRecord>),
    Exit(ExitCode),
}

fn resolve_record_ref(
    service: &impl RecordRetrieval,
    record_ref: &str,
    json: bool,
) -> Result<RecordRefOutcome, String> {
    let resolution = match service.resolve_record_ref(ResolveRecordRefRequest {
        record_ref,
        filter: None,
    }) {
        Ok(resolution) => resolution,
        Err(error) if json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(RecordRefOutcome::Exit(ExitCode::from(3)));
        }
        Err(error) => return Err(search_error(error)),
    };
    let key = match resolution {
        RecordRefResolutionResult::Key(key) => key,
        RecordRefResolutionResult::Miss => {
            if json {
                write_json_error(
                    "record_resolution_miss",
                    format!("record resolution miss: {record_ref}"),
                )?;
            } else {
                eprintln!("record resolution miss: {record_ref}");
            }
            return Ok(RecordRefOutcome::Exit(ExitCode::from(1)));
        }
        RecordRefResolutionResult::Ambiguous(matches) => {
            let message = format!("record resolution ambiguous: {record_ref}");
            if json {
                write_json_error_data(
                    "record_resolution_ambiguous",
                    message,
                    AmbiguousRecordRefs {
                        record_ref,
                        matches: matches
                            .iter()
                            .map(|resolution| {
                                record_json(&resolution.record, standard_record_json_options())
                            })
                            .collect(),
                    },
                )?;
            } else {
                eprintln!("{message}");
            }
            return Ok(RecordRefOutcome::Exit(ExitCode::from(1)));
        }
    };
    let records = match service.get_records(GetRecordsRequest {
        record_keys: std::slice::from_ref(&key),
    }) {
        Ok(records) => records,
        Err(error) if json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(RecordRefOutcome::Exit(ExitCode::from(3)));
        }
        Err(error) => return Err(search_error(error)),
    };
    let Some(record) = records.into_iter().next() else {
        if json {
            write_json_error("record_not_found", format!("record not found: {key}"))?;
        } else {
            eprintln!("record not found: {key}");
        }
        return Ok(RecordRefOutcome::Exit(ExitCode::from(1)));
    };
    Ok(RecordRefOutcome::Record(Box::new(record)))
}

#[derive(Debug, Serialize)]
struct AmbiguousRecordRefs<'a> {
    record_ref: &'a str,
    matches: Vec<RecordJson>,
}

fn hydrate_records(
    items: &[SavedListItem],
    paths: &ListsPathOptions,
    json: bool,
) -> Result<ListCommandStep<BTreeMap<String, atlas_record::AtlasRecord>>, String> {
    if items.is_empty() {
        return Ok(ListCommandStep::Ready(BTreeMap::new()));
    }
    let runtime = match runtime_for_lists(paths, json)? {
        ListCommandStep::Ready(runtime) => runtime,
        ListCommandStep::Exit(code) => return Ok(ListCommandStep::Exit(code)),
    };
    let service = match record_service_for_lists(&runtime, json)? {
        ListCommandStep::Ready(service) => service,
        ListCommandStep::Exit(code) => return Ok(ListCommandStep::Exit(code)),
    };
    let keys = items
        .iter()
        .filter_map(|item| RecordKey::parse(&item.record_key).ok())
        .collect::<Vec<_>>();
    let records = match service.get_records(GetRecordsRequest { record_keys: &keys }) {
        Ok(records) => records,
        Err(error) if json => {
            write_json_error(search_error_code(&error), error.to_string())?;
            return Ok(ListCommandStep::Exit(ExitCode::from(3)));
        }
        Err(error) => return Err(search_error(error)),
    };
    Ok(ListCommandStep::Ready(
        records
            .into_iter()
            .map(|record| (record.identity.key.to_string(), record))
            .collect(),
    ))
}

fn show_item(
    item: SavedListItem,
    records_by_key: &BTreeMap<String, atlas_record::AtlasRecord>,
) -> ListShowItem {
    let record = records_by_key
        .get(&item.record_key)
        .map(|record| record_json(record, standard_record_json_options()));
    let hydrated = hydrate_saved_list_item(item, record);
    list_show_item_from_hydrated(hydrated)
}

fn list_show_item_from_hydrated(item: HydratedSavedListItem<RecordJson>) -> ListShowItem {
    ListShowItem {
        record_key: item.record_key,
        position: item.position,
        note: item.note,
        status: list_item_status_text(item.status),
        snapshot: ListItemSnapshot {
            name: item.snapshot.title,
            kind: item.snapshot.kind,
        },
        record: item.record,
    }
}

fn list_item_status_text(status: SavedListItemStatus) -> &'static str {
    match status {
        SavedListItemStatus::Active => "active",
        SavedListItemStatus::Unresolved => "unresolved",
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
            .map(|record| record.kind)
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

fn list_not_found(slug: &str, json: bool) -> Result<ExitCode, String> {
    if json {
        write_json_error(
            "saved_list_not_found",
            format!("saved list not found: {slug}"),
        )?;
    } else {
        eprintln!("saved list not found: {slug}");
    }
    Ok(ExitCode::from(1))
}

fn local_state_error(error: LocalStateError, json: bool) -> Result<ExitCode, String> {
    let code = match &error {
        LocalStateError::InvalidSlug { .. } => "invalid_saved_list_slug",
        LocalStateError::InvalidRecordKey { .. } => "invalid_record_key",
        LocalStateError::ListAlreadyExists(_) => "saved_list_already_exists",
        LocalStateError::ListNotFound(_) => "saved_list_not_found",
        LocalStateError::UnsupportedMetadata { .. }
        | LocalStateError::IncompatibleSchema(_)
        | LocalStateError::Database(_)
        | LocalStateError::Filesystem(_)
        | LocalStateError::Timestamp(_)
        | LocalStateError::NonUtf8Path(_) => "local_state_error",
    };
    let exit = match &error {
        LocalStateError::InvalidSlug { .. } | LocalStateError::InvalidRecordKey { .. } => {
            ExitCode::from(2)
        }
        LocalStateError::ListAlreadyExists(_) | LocalStateError::ListNotFound(_) => {
            ExitCode::from(1)
        }
        LocalStateError::UnsupportedMetadata { .. }
        | LocalStateError::IncompatibleSchema(_)
        | LocalStateError::Database(_)
        | LocalStateError::Filesystem(_)
        | LocalStateError::Timestamp(_)
        | LocalStateError::NonUtf8Path(_) => ExitCode::from(3),
    };
    if json {
        write_json_error(code, error.to_string())?;
        Ok(exit)
    } else {
        match &error {
            LocalStateError::InvalidSlug { .. }
            | LocalStateError::InvalidRecordKey { .. }
            | LocalStateError::UnsupportedMetadata { .. }
            | LocalStateError::IncompatibleSchema(_)
            | LocalStateError::Database(_)
            | LocalStateError::Filesystem(_)
            | LocalStateError::Timestamp(_)
            | LocalStateError::NonUtf8Path(_) => Err(error.to_string()),
            LocalStateError::ListAlreadyExists(_) | LocalStateError::ListNotFound(_) => {
                eprintln!("{error}");
                Ok(exit)
            }
        }
    }
}

fn standard_record_json_options() -> RecordJsonOptions {
    RecordJsonOptions {
        detail: DetailLevel::Standard,
        include_source_json: false,
    }
}
