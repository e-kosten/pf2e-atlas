use std::collections::BTreeMap;
use std::process::ExitCode;

use atlas_app_model::{
    AddSavedListItemRequest, AppError, AppErrorCode, CreateSavedListRequest, DeleteSavedListView,
    RecordResolutionAmbiguousView, RemoveSavedListItemRequest, SavedListDetailView,
    SavedListItemMutationView, SavedListItemStatusView, SavedListItemView, SavedListSummaryView,
};
use atlas_domain::{DetailLevel, RecordKey};
use atlas_record::{RecordJson, RecordJsonOptions, record_json};
use serde::Serialize;

use crate::client::{
    AtlasClient, AtlasClientConfig, AtlasClientHandle, LocalAtlasClientOptions, connect,
};
use crate::output::{write_json_data, write_json_error, write_json_error_data};

pub(crate) mod args;

use args::{
    ListAddOptions, ListCreateOptions, ListDeleteOptions, ListLsOptions, ListRemoveOptions,
    ListShowDetail, ListShowOptions, ListsPathOptions,
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
    list_key: String,
    slug: String,
    deleted: bool,
}

#[derive(Debug, Serialize)]
struct ListItemMutationData {
    local_state_path: Option<String>,
    list_key: String,
    slug: String,
    record_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    record_name: Option<String>,
    outcome: &'static str,
}

#[derive(Debug, Serialize)]
struct ListShowData {
    local_state_path: Option<String>,
    list: SavedListSummaryView,
    item_count: u64,
    items: Vec<ListShowItem>,
}

#[derive(Debug, Serialize)]
struct ListShowNoRecordsData {
    local_state_path: Option<String>,
    list: SavedListSummaryView,
    item_count: u64,
    items: Vec<ListShowItemNoRecord>,
}

#[derive(Debug, Serialize)]
struct ListShowSummaryData {
    local_state_path: Option<String>,
    list: SavedListSummaryView,
    item_count: u64,
    items: Vec<ListShowSummaryItem>,
}

#[derive(Debug, Serialize)]
struct ListShowKeysData {
    local_state_path: Option<String>,
    list: SavedListSummaryView,
    item_count: u64,
    record_keys: Vec<String>,
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
struct ListShowItemNoRecord {
    record_key: String,
    position: i64,
    note: Option<String>,
    status: &'static str,
    snapshot: ListItemSnapshot,
}

#[derive(Debug, Serialize)]
struct ListShowSummaryItem {
    position: i64,
    record_key: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<String>,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ListItemSnapshot {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
}

#[derive(Debug, Serialize)]
struct LegacyAmbiguousRecordRefs {
    record_ref: String,
    matches: Vec<serde_json::Value>,
}

enum ListCommandStep<T> {
    Ready(T),
    Exit(ExitCode),
}

enum ListShowMode {
    Records(DetailLevel),
    Summary,
    KeysOnly,
    NoRecords,
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
        Err(error) => return app_error_with_client(&client, error, options.json),
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
    let mode = list_show_mode(&options)?;
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.saved_list(&options.slug) {
        Ok(view) => view,
        Err(error) => return app_error(error, options.json),
    };
    match mode {
        ListShowMode::Records(detail) => {
            let data = match list_show_data(&client, view, detail) {
                Ok(data) => data,
                Err(error) => return app_error(error, options.json),
            };
            if options.json {
                write_json_data(data)?;
            } else {
                print_saved_list(&data);
            }
        }
        ListShowMode::Summary => {
            let data = list_show_summary_data(&client, view);
            if options.json {
                write_json_data(data)?;
            } else {
                print_saved_list_summary(&data);
            }
        }
        ListShowMode::KeysOnly => {
            let data = list_show_keys_data(&client, view);
            if options.json {
                write_json_data(data)?;
            } else {
                for key in data.record_keys {
                    println!("{key}");
                }
            }
        }
        ListShowMode::NoRecords => {
            let data = list_show_no_records_data(&client, view);
            if options.json {
                write_json_data(data)?;
            } else {
                print_saved_list_no_records(&data);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_lists_add(options: ListAddOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.add_saved_list_item(AddSavedListItemRequest {
        list_ref: options.slug,
        record_ref: options.record_ref,
        note: options.note,
    }) {
        Ok(view) => view,
        Err(error) => return app_error_with_client(&client, error, options.json),
    };
    write_mutation_result(&client, view, options.json)
}

pub(crate) fn run_lists_remove(options: ListRemoveOptions) -> Result<ExitCode, String> {
    let client = match lists_client(&options.paths, options.json)? {
        ListCommandStep::Ready(client) => client,
        ListCommandStep::Exit(code) => return Ok(code),
    };
    let view = match client.remove_saved_list_item(RemoveSavedListItemRequest {
        list_ref: options.slug,
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

fn list_show_data(
    client: &impl AtlasClient,
    view: SavedListDetailView,
    detail: DetailLevel,
) -> Result<ListShowData, AppError> {
    let records_by_key = hydrate_records(client, &view.items)?;
    Ok(ListShowData {
        local_state_path: local_state_path(client),
        item_count: view.list.item_count,
        list: view.list,
        items: view
            .items
            .into_iter()
            .map(|item| list_show_item(item, &records_by_key, detail))
            .collect(),
    })
}

fn list_show_summary_data(
    client: &impl AtlasClient,
    view: SavedListDetailView,
) -> ListShowSummaryData {
    ListShowSummaryData {
        local_state_path: local_state_path(client),
        item_count: view.list.item_count,
        list: view.list,
        items: view.items.into_iter().map(list_show_summary_item).collect(),
    }
}

fn list_show_keys_data(client: &impl AtlasClient, view: SavedListDetailView) -> ListShowKeysData {
    ListShowKeysData {
        local_state_path: local_state_path(client),
        item_count: view.list.item_count,
        list: view.list,
        record_keys: view.items.into_iter().map(|item| item.record_key).collect(),
    }
}

fn list_show_no_records_data(
    client: &impl AtlasClient,
    view: SavedListDetailView,
) -> ListShowNoRecordsData {
    ListShowNoRecordsData {
        local_state_path: local_state_path(client),
        item_count: view.list.item_count,
        list: view.list,
        items: view
            .items
            .into_iter()
            .map(list_show_item_no_record)
            .collect(),
    }
}

fn hydrate_records(
    client: &impl AtlasClient,
    items: &[SavedListItemView],
) -> Result<BTreeMap<String, atlas_record::AtlasRecord>, AppError> {
    let keys = items
        .iter()
        .filter(|item| item.record.is_some())
        .filter_map(|item| RecordKey::parse(&item.record_key).ok())
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    Ok(client
        .get_records(keys)?
        .into_iter()
        .map(|record| (record.identity.key.to_string(), record))
        .collect())
}

fn list_show_item(
    item: SavedListItemView,
    records_by_key: &BTreeMap<String, atlas_record::AtlasRecord>,
    detail: DetailLevel,
) -> ListShowItem {
    let record = records_by_key
        .get(&item.record_key)
        .map(|record| record_json(record, record_json_options(detail)));
    ListShowItem {
        record_key: item.record_key,
        position: item.position,
        note: item.note,
        status: list_item_status_text(item.status),
        snapshot: ListItemSnapshot {
            name: item.snapshot.title,
            kind: item.snapshot.kind,
        },
        record,
    }
}

fn list_show_item_no_record(item: SavedListItemView) -> ListShowItemNoRecord {
    ListShowItemNoRecord {
        record_key: item.record_key,
        position: item.position,
        note: item.note,
        status: list_item_status_text(item.status),
        snapshot: ListItemSnapshot {
            name: item.snapshot.title,
            kind: item.snapshot.kind,
        },
    }
}

fn list_show_summary_item(item: SavedListItemView) -> ListShowSummaryItem {
    let status = list_item_status_text(item.status);
    let (name, level, kind) = match item.record {
        Some(record) => (record.title, record.level_label, record.kind),
        None => (
            item.snapshot.title,
            None,
            item.snapshot.kind.unwrap_or_else(|| "unknown".to_string()),
        ),
    };
    ListShowSummaryItem {
        position: item.position,
        record_key: item.record_key,
        name,
        level,
        kind,
        note: item.note,
        status,
    }
}

fn write_mutation_result(
    client: &impl AtlasClient,
    view: SavedListItemMutationView,
    json: bool,
) -> Result<ExitCode, String> {
    let data = ListItemMutationData {
        local_state_path: local_state_path(client),
        list_key: view.list_key,
        slug: view.slug,
        record_key: view.record_key,
        record_name: view.record_name,
        outcome: mutation_outcome_text(view.outcome),
    };
    if json {
        write_json_data(data)?;
    } else {
        if let Some(record_name) = &data.record_name {
            println!(
                "{}\t{}\t{}\t{}",
                data.outcome, data.slug, data.record_key, record_name
            );
        } else {
            println!("{}\t{}\t{}", data.outcome, data.slug, data.record_key);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn delete_data(client: &impl AtlasClient, view: DeleteSavedListView) -> ListDeleteData {
    ListDeleteData {
        local_state_path: local_state_path(client),
        list_key: view.list_key,
        slug: view.slug,
        deleted: view.deleted,
    }
}

fn local_state_path(client: &impl AtlasClient) -> Option<String> {
    client.local_state_path().map(str::to_string)
}

fn list_show_mode(options: &ListShowOptions) -> Result<ListShowMode, String> {
    let selected = [options.summary, options.keys_only, options.no_records]
        .into_iter()
        .filter(|selected| *selected)
        .count()
        + usize::from(options.detail.is_some());
    if selected > 1 {
        return Err(
            "choose only one of --summary, --keys-only, --no-records, or --detail".to_string(),
        );
    }
    if options.summary {
        return Ok(ListShowMode::Summary);
    }
    if options.keys_only {
        return Ok(ListShowMode::KeysOnly);
    }
    if options.no_records {
        return Ok(ListShowMode::NoRecords);
    }
    match options.detail {
        Some(ListShowDetail::Preview) => Ok(ListShowMode::Records(DetailLevel::Preview)),
        Some(ListShowDetail::Standard) => Ok(ListShowMode::Records(DetailLevel::Standard)),
        Some(ListShowDetail::None) => Ok(ListShowMode::NoRecords),
        None => Ok(ListShowMode::Records(DetailLevel::Standard)),
    }
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

fn print_saved_list_summary(data: &ListShowSummaryData) {
    println!(
        "{}\t{}\t{} items",
        data.list.slug, data.list.name, data.item_count
    );
    if let Some(description) = &data.list.description {
        println!("{description}");
    }
    for item in &data.items {
        let level = item.level.as_deref().unwrap_or("-");
        println!(
            "{}.\t{}\t{}\t{}\t{}\t{}",
            item.position, item.record_key, item.name, level, item.kind, item.status
        );
        if let Some(note) = &item.note {
            println!("\tnote: {note}");
        }
    }
}

fn print_saved_list_no_records(data: &ListShowNoRecordsData) {
    println!(
        "{}\t{}\t{} items",
        data.list.slug, data.list.name, data.item_count
    );
    if let Some(description) = &data.list.description {
        println!("{description}");
    }
    for item in &data.items {
        let kind = item.snapshot.kind.as_deref().unwrap_or("unknown");
        println!(
            "{}.\t{}\t{}\t{}\t{}",
            item.position, item.record_key, item.snapshot.name, kind, item.status
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

fn app_error_with_client(
    client: &impl AtlasClient,
    error: AppError,
    json: bool,
) -> Result<ExitCode, String> {
    let (_, exit) = cli_error_code(error.code);
    if json {
        write_app_json_error_with_client(client, error)?;
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

fn write_app_json_error_with_client(
    client: &impl AtlasClient,
    error: AppError,
) -> Result<(), String> {
    let (code, _) = cli_error_code(error.code);
    if error.code == AppErrorCode::RecordResolutionAmbiguous
        && let Some(details) = error.details.as_ref()
        && let Some(data) = legacy_ambiguous_record_refs(client, details)
    {
        return write_json_error_data(code, error.message, data);
    }
    write_app_json_error(error)
}

fn legacy_ambiguous_record_refs(
    client: &impl AtlasClient,
    details: &serde_json::Value,
) -> Option<LegacyAmbiguousRecordRefs> {
    let ambiguity = match serde_json::from_value::<RecordResolutionAmbiguousView>(details.clone()) {
        Ok(ambiguity) => ambiguity,
        Err(_) => return legacy_ambiguous_record_refs_from_value(details),
    };
    let keys = ambiguity
        .matches
        .iter()
        .filter_map(|candidate| RecordKey::parse(&candidate.record.record_key).ok())
        .collect::<Vec<_>>();
    if keys.len() != ambiguity.matches.len() {
        return None;
    }
    let records_by_key = client
        .get_records(keys)
        .ok()
        .unwrap_or_default()
        .into_iter()
        .map(|record| (record.identity.key.to_string(), record))
        .collect::<BTreeMap<_, _>>();
    let matches = ambiguity
        .matches
        .iter()
        .map(|candidate| {
            records_by_key
                .get(&candidate.record.record_key)
                .map(|record| {
                    serde_json::json!(record_json(
                        record,
                        record_json_options(DetailLevel::Standard)
                    ))
                })
                .unwrap_or_else(|| {
                    serde_json::json!({
                        "key": candidate.record.record_key,
                        "name": candidate.record.title,
                        "kind": candidate.record.kind,
                    })
                })
        })
        .collect();
    Some(LegacyAmbiguousRecordRefs {
        record_ref: ambiguity.record_ref,
        matches,
    })
}

fn legacy_ambiguous_record_refs_from_value(
    details: &serde_json::Value,
) -> Option<LegacyAmbiguousRecordRefs> {
    let record_ref = details.get("record_ref")?.as_str()?.to_string();
    let matches = details
        .get("matches")?
        .as_array()?
        .iter()
        .map(|candidate| {
            if candidate.get("key").is_some() {
                return Some(candidate.clone());
            }
            let record = candidate.get("record")?;
            Some(serde_json::json!({
                "key": record.get("record_key")?,
                "name": record.get("title")?,
                "kind": record.get("kind")?,
            }))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(LegacyAmbiguousRecordRefs {
        record_ref,
        matches,
    })
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
        AppErrorCode::EncounterNotFound
        | AppErrorCode::EncounterAlreadyExists
        | AppErrorCode::EncounterParticipantNotFound => ("local_state_error", ExitCode::from(3)),
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

fn record_json_options(detail: DetailLevel) -> RecordJsonOptions {
    RecordJsonOptions {
        detail,
        include_source_json: false,
    }
}
