use std::collections::{BTreeMap, BTreeSet};

use atlas_app_model::{
    AddSavedListItemRequest, AppErrorCode, BatchAddSavedListItemsRequest, BatchSavedListItemInput,
    BatchSavedListItemMutationView, BatchSavedListItemOutcomeView, BatchSavedListItemResultView,
    CreateSavedListRequest, DeleteSavedListView, FilterSavedListRequest, ImportSavedListRequest,
    ImportSavedListView, RecordResolutionAmbiguousView, RecordResolutionCandidateView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView,
    SavedListExportDocumentView, SavedListExportItemView, SavedListExportListView,
    SavedListIndexView, SavedListItemMutationOutcomeView, SavedListItemMutationView,
    SavedListItemNoteView, SavedListItemSnapshotView, SavedListItemStatusView, SavedListItemView,
    SavedListSummaryView, SavedListUpdateView, UpdateSavedListItemNoteRequest,
    UpdateSavedListRequest,
};
use atlas_domain::RecordKey;
use atlas_local_state::{
    AddSavedListItemOutcome, HydratedSavedListItem, ImportSavedList, ImportSavedListItem,
    NewSavedList, ResolvedSavedListItem, SavedList, SavedListItem, SavedListItemStatus,
    UpdateSavedList, hydrate_saved_list_item,
};
use atlas_search::{
    GetRecordsRequest, ListRecordsRequest, RecordRefResolutionResult, RecordScope,
    ResolveRecordRefRequest, RetrievalMode, SearchPage, TextSearchRequest,
};
use serde_json::json;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_summary_view;
use crate::service::AtlasAppService;

impl AtlasAppService {
    pub fn saved_lists(&self) -> AppServiceResult<SavedListIndexView> {
        self.with_state(move |session| {
            let store = session.local_state();
            let lists = store.saved_lists().list()?;
            Ok(SavedListIndexView {
                lists: lists.into_iter().map(saved_list_summary).collect(),
            })
        })
    }

    pub fn saved_list(&self, list_ref: &str) -> AppServiceResult<SavedListDetailView> {
        self.saved_list_with_filter(list_ref, None, None, false)
    }

    pub fn filter_saved_list(
        &self,
        request: FilterSavedListRequest,
    ) -> AppServiceResult<SavedListDetailView> {
        let filter = request.filter;
        let query = request
            .query
            .as_deref()
            .map(str::trim)
            .filter(|query| !query.is_empty());
        let has_match_scope = filter.is_some() || query.is_some();
        self.saved_list_with_filter(&request.list_ref, filter.as_ref(), query, has_match_scope)
    }

    pub fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> AppServiceResult<SavedListCreateView> {
        self.with_write(move |session| {
            let store = session.local_state();
            let list = store.saved_lists().create(NewSavedList {
                slug: request.slug,
                name: request.name,
                description: request.description,
                tags: request.tags,
            })?;
            Ok(SavedListCreateView {
                list: saved_list_summary(list),
            })
        })
    }

    pub fn saved_list_item_note(
        &self,
        list_ref: &str,
        record_key: &str,
    ) -> AppServiceResult<SavedListItemNoteView> {
        self.with_state(|session| {
            let saved = session
                .local_state()
                .saved_lists()
                .get_with_items(list_ref)?
                .ok_or_else(|| saved_list_not_found(list_ref))?;
            let item = saved
                .items
                .into_iter()
                .find(|item| item.record_key == record_key)
                .ok_or_else(|| AppServiceError::invalid_request("record is not in this list"))?;
            Ok(SavedListItemNoteView {
                list_key: saved.list.list_key,
                slug: saved.list.slug,
                record_key: item.record_key,
                title: item.record_title_snapshot,
                note: item.note,
            })
        })
    }

    pub fn update_saved_list_item_note(
        &self,
        request: UpdateSavedListItemNoteRequest,
    ) -> AppServiceResult<SavedListItemNoteView> {
        self.with_write(|session| {
            let store = session.local_state();
            let saved = store
                .saved_lists()
                .get_with_items(&request.list_ref)?
                .ok_or_else(|| saved_list_not_found(&request.list_ref))?;
            let item = saved
                .items
                .into_iter()
                .find(|item| item.record_key == request.record_key)
                .ok_or_else(|| AppServiceError::invalid_request("record is not in this list"))?;
            store.saved_lists().update_item_note(
                &saved.list.list_key,
                &request.record_key,
                request.note.as_deref(),
            )?;
            Ok(SavedListItemNoteView {
                list_key: saved.list.list_key,
                slug: saved.list.slug,
                record_key: request.record_key,
                title: item.record_title_snapshot,
                note: request.note,
            })
        })
    }

    pub fn update_saved_list(
        &self,
        request: UpdateSavedListRequest,
    ) -> AppServiceResult<SavedListUpdateView> {
        self.with_write(move |session| {
            let store = session.local_state();
            let list_key = request.list_key;
            let list = store
                .saved_lists()
                .update(UpdateSavedList {
                    list_key: list_key.clone(),
                    slug: request.slug,
                    name: request.name,
                    description: request.description,
                    tags: request.tags,
                })?
                .ok_or_else(|| saved_list_not_found(&list_key))?;
            Ok(SavedListUpdateView {
                list: saved_list_summary(list),
            })
        })
    }

    pub fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> AppServiceResult<SavedListItemMutationView> {
        let record_ref = request.record_ref.clone();
        let list_ref = request.list_ref.clone();
        let (record, list, revision) = self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let list = session
                    .local_state()
                    .saved_lists()
                    .get(&list_ref)?
                    .ok_or_else(|| saved_list_not_found(&list_ref))?;
                let record = resolve_record_ref(session.retrieval_mut(), &record_ref)?;
                Ok((record, list, session.reference_revision()?))
            },
        )?;
        self.with_write(move |session| {
            session.require_reference_revision(revision)?;
            let record_key = record.key.to_string();
            let record_name = record
                .name
                .as_value()
                .cloned()
                .unwrap_or_else(|| record.key.to_string());
            let store = session.local_state();
            let list = store
                .saved_lists()
                .get(&list.list_key)?
                .ok_or_else(|| saved_list_not_found(&list.list_key))?;
            let outcome = store.saved_lists().add_resolved_item(
                &list.list_key,
                ResolvedSavedListItem {
                    record_key: record.key,
                    title_snapshot: record_name.clone(),
                    kind_snapshot: record
                        .record_kind
                        .as_value()
                        .map(|kind| kind.as_str().to_string()),
                    note: request.note,
                },
            )?;
            Ok(SavedListItemMutationView {
                list_key: list.list_key,
                slug: list.slug,
                record_key,
                record_name: Some(record_name),
                outcome: match outcome {
                    AddSavedListItemOutcome::Added => SavedListItemMutationOutcomeView::Added,
                    AddSavedListItemOutcome::AlreadyPresent => {
                        SavedListItemMutationOutcomeView::AlreadyPresent
                    }
                },
            })
        })
    }

    pub fn add_saved_list_items(
        &self,
        request: BatchAddSavedListItemsRequest,
    ) -> AppServiceResult<BatchSavedListItemMutationView> {
        self.write_saved_list_items(request, None)
    }

    /// Create a new list and commit all successfully resolved inputs in one write.
    pub fn create_saved_list_with_items(
        &self,
        request: CreateSavedListRequest,
        items: Vec<BatchSavedListItemInput>,
    ) -> AppServiceResult<BatchSavedListItemMutationView> {
        self.write_saved_list_items(
            BatchAddSavedListItemsRequest {
                list_ref: request.slug.clone(),
                items,
            },
            Some(request),
        )
    }

    fn write_saved_list_items(
        &self,
        request: BatchAddSavedListItemsRequest,
        create: Option<CreateSavedListRequest>,
    ) -> AppServiceResult<BatchSavedListItemMutationView> {
        let creating = create.is_some();
        let (list_key, resolved, revision) = self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let existing = session.local_state().saved_lists().get(&request.list_ref)?;
                let list_key = match (creating, existing) {
                    (true, Some(_)) => {
                        return Err(atlas_local_state::LocalStateError::ListAlreadyExists(
                            request.list_ref,
                        )
                        .into());
                    }
                    (true, None) => None,
                    (false, Some(list)) => Some(list.list_key),
                    (false, None) => return Err(saved_list_not_found(&request.list_ref)),
                };
                let resolved = request
                    .items
                    .into_iter()
                    .map(|item| {
                        let record = resolve_record_ref(session.retrieval_mut(), &item.record_ref);
                        (item.record_ref, item.note, record)
                    })
                    .collect::<Vec<_>>();
                Ok((list_key, resolved, session.reference_revision()?))
            },
        )?;
        self.with_write(move |session| {
            session.require_reference_revision(revision)?;
            let store = session.local_state();
            let list = if let Some(request) = create {
                store.saved_lists().create(NewSavedList {
                    slug: request.slug,
                    name: request.name,
                    description: request.description,
                    tags: request.tags,
                })?
            } else {
                let list_key = list_key.ok_or_else(|| {
                    AppServiceError::invalid_request("saved-list key is required")
                })?;
                store
                    .saved_lists()
                    .get(&list_key)?
                    .ok_or_else(|| saved_list_not_found(&list_key))?
            };
            let mut items = Vec::new();
            let mut added_count = 0;
            let mut already_present_count = 0;
            let mut failed_count = 0;
            for (input, note, resolved) in resolved {
                let record = match resolved {
                    Ok(record) => record,
                    Err(error) => {
                        let error = error.into_app_error();
                        if !is_batch_item_error(error.code) {
                            return Err(error.into());
                        }
                        failed_count += 1;
                        items.push(BatchSavedListItemResultView {
                            input,
                            outcome: BatchSavedListItemOutcomeView::Failed,
                            record_key: None,
                            record_name: None,
                            error: Some(error),
                        });
                        continue;
                    }
                };
                let record_key = record.key.to_string();
                let record_name = record
                    .name
                    .as_value()
                    .cloned()
                    .unwrap_or_else(|| record.key.to_string());
                let outcome = store.saved_lists().add_resolved_item(
                    &list.list_key,
                    ResolvedSavedListItem {
                        record_key: record.key,
                        title_snapshot: record_name.clone(),
                        kind_snapshot: record
                            .record_kind
                            .as_value()
                            .map(|kind| kind.as_str().to_string()),
                        note,
                    },
                )?;
                let outcome = match outcome {
                    AddSavedListItemOutcome::Added => {
                        added_count += 1;
                        BatchSavedListItemOutcomeView::Added
                    }
                    AddSavedListItemOutcome::AlreadyPresent => {
                        already_present_count += 1;
                        BatchSavedListItemOutcomeView::AlreadyPresent
                    }
                };
                items.push(BatchSavedListItemResultView {
                    input,
                    outcome,
                    record_key: Some(record_key),
                    record_name: Some(record_name),
                    error: None,
                });
            }
            Ok(BatchSavedListItemMutationView {
                list_key: list.list_key,
                slug: list.slug,
                requested_count: items.len() as u64,
                added_count,
                already_present_count,
                failed_count,
                items,
            })
        })
    }

    pub fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> AppServiceResult<SavedListItemMutationView> {
        let (record_key, record_name, revision, canonical_list) =
            if let Ok(key) = RecordKey::parse(&request.record_ref) {
                (key, None, None, None)
            } else {
                let record_ref = request.record_ref.clone();
                let list_ref = request.list_ref.clone();
                self.with_workspace(
                    &crate::ReadExecutionContext::uncancellable(),
                    move |session| {
                        let list = session
                            .local_state()
                            .saved_lists()
                            .get(&list_ref)?
                            .ok_or_else(|| saved_list_not_found(&list_ref))?;
                        let record = resolve_record_ref(session.retrieval_mut(), &record_ref)?;
                        Ok((
                            record.key,
                            record.name.as_value().cloned(),
                            Some(session.reference_revision()?),
                            Some(list.list_key),
                        ))
                    },
                )?
            };
        self.with_write(move |session| {
            if let Some(revision) = revision {
                session.require_reference_revision(revision)?;
            }
            let lists = session.local_state().saved_lists();
            let list_ref = canonical_list.as_deref().unwrap_or(&request.list_ref);
            let list = lists
                .get(list_ref)?
                .ok_or_else(|| saved_list_not_found(list_ref))?;
            let removed = lists.remove_item(&list.list_key, &record_key.to_string())?;
            Ok(SavedListItemMutationView {
                list_key: list.list_key,
                slug: list.slug,
                record_key: record_key.to_string(),
                record_name,
                outcome: if removed {
                    SavedListItemMutationOutcomeView::Removed
                } else {
                    SavedListItemMutationOutcomeView::NotPresent
                },
            })
        })
    }

    pub fn delete_saved_list(&self, list_ref: &str) -> AppServiceResult<DeleteSavedListView> {
        let list_ref = list_ref.to_string();
        self.with_write(move |session| {
            let store = session.local_state();
            let list = store
                .saved_lists()
                .get(&list_ref)?
                .ok_or_else(|| saved_list_not_found(&list_ref))?;
            let deleted = store.saved_lists().delete(&list.list_key)?;
            Ok(DeleteSavedListView {
                list_key: list.list_key,
                slug: list.slug,
                deleted,
            })
        })
    }

    pub fn export_saved_list(
        &self,
        list_ref: &str,
    ) -> AppServiceResult<SavedListExportDocumentView> {
        let view = self.saved_list(list_ref)?;
        Ok(SavedListExportDocumentView {
            format: "pf2e-atlas.saved-list".to_string(),
            version: 1,
            exported_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .map_err(|error| {
                    AppServiceError::new(AppErrorCode::InternalError, error.to_string())
                })?,
            list: SavedListExportListView {
                id: view.list.slug,
                name: view.list.name,
                description: view.list.description,
                tags: view.list.tags,
            },
            items: view.items.into_iter().map(saved_list_export_item).collect(),
        })
    }

    pub fn import_saved_list(
        &self,
        request: ImportSavedListRequest,
    ) -> AppServiceResult<ImportSavedListView> {
        if request.document.format != "pf2e-atlas.saved-list" {
            return Err(AppServiceError::invalid_request(format!(
                "unsupported saved-list import format `{}`",
                request.document.format
            )));
        }
        if request.document.version != 1 {
            return Err(AppServiceError::invalid_request(format!(
                "unsupported saved-list import version `{}`",
                request.document.version
            )));
        }
        let id = request.id.unwrap_or(request.document.list.id);
        let export_items = request.document.items.clone();
        let (records_by_key, revision) = if export_items.is_empty() {
            (BTreeMap::new(), None)
        } else {
            self.with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |session| {
                    let records = hydrate_export_records(session.retrieval_mut(), &export_items)?;
                    Ok((records, Some(session.reference_revision()?)))
                },
            )?
        };
        let imported = request
            .document
            .items
            .into_iter()
            .map(|item| {
                let record = records_by_key.get(&item.record_key);
                ImportSavedListItem {
                    record_key: item.record_key,
                    note: item.note,
                    record_title_snapshot: record
                        .map(|record| {
                            record
                                .name
                                .as_value()
                                .cloned()
                                .unwrap_or_else(|| record.key.to_string())
                        })
                        .unwrap_or_else(|| item.snapshot.title),
                    record_kind_snapshot: record
                        .map(|record| {
                            record
                                .record_kind
                                .as_value()
                                .map(|kind| kind.as_str().to_string())
                                .unwrap_or_default()
                        })
                        .or(item.snapshot.kind),
                }
            })
            .collect::<Vec<_>>();
        self.with_write(move |session| {
            if let Some(revision) = revision {
                session.require_reference_revision(revision)?;
            }
            let store = session.local_state();
            let replaced = store.saved_lists().get(&id)?.is_some() && request.replace;
            let imported = store.saved_lists().import(ImportSavedList {
                slug: id,
                name: request.document.list.name,
                description: request.document.list.description,
                tags: request.document.list.tags,
                items: imported,
                replace: request.replace,
            })?;
            let mut active_count = 0;
            let mut unresolved_count = 0;
            for item in &imported.items {
                if records_by_key.contains_key(&item.record_key) {
                    active_count += 1;
                } else {
                    unresolved_count += 1;
                }
            }
            Ok(ImportSavedListView {
                list: saved_list_summary(imported.list),
                replaced,
                active_count,
                unresolved_count,
            })
        })
    }

    fn saved_list_with_filter(
        &self,
        list_ref: &str,
        filter: Option<&atlas_domain::QueryPredicate>,
        query: Option<&str>,
        has_match_scope: bool,
    ) -> AppServiceResult<SavedListDetailView> {
        let initial = self.with_state(|session| {
            session
                .local_state()
                .saved_lists()
                .get_with_items(list_ref)?
                .ok_or_else(|| saved_list_not_found(list_ref))
        })?;
        if !initial
            .items
            .iter()
            .any(|item| RecordKey::parse(&item.record_key).is_ok())
        {
            return Ok(SavedListDetailView {
                list: saved_list_summary(initial.list),
                items: initial
                    .items
                    .into_iter()
                    .filter(|_| !has_match_scope)
                    .map(|item| saved_list_item_view(item, &BTreeMap::new()))
                    .collect(),
            });
        }

        let list_ref = list_ref.to_string();
        let filter = filter.cloned();
        let query = query.map(str::to_owned);
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let list = session
                    .local_state()
                    .saved_lists()
                    .get_with_items(&list_ref)?
                    .ok_or_else(|| saved_list_not_found(&list_ref))?;
                let active_keys = list
                    .items
                    .iter()
                    .filter_map(|item| RecordKey::parse(&item.record_key).ok())
                    .collect::<Vec<_>>();
                let records_by_key = if let Some(query) = query.as_deref() {
                    searched_saved_list_records(
                        session.retrieval_mut(),
                        &active_keys,
                        filter.as_ref(),
                        query,
                    )?
                } else if has_match_scope {
                    filtered_saved_list_records(
                        session.retrieval_mut(),
                        &active_keys,
                        filter.as_ref(),
                    )?
                } else {
                    hydrate_saved_list_records(session.retrieval_mut(), &list.items)?
                };
                Ok(SavedListDetailView {
                    list: saved_list_summary(list.list),
                    items: list
                        .items
                        .into_iter()
                        .filter(|item| {
                            !has_match_scope || records_by_key.contains_key(&item.record_key)
                        })
                        .map(|item| saved_list_item_view(item, &records_by_key))
                        .collect(),
                })
            },
        )
    }
}

fn hydrate_saved_list_records(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    items: &[SavedListItem],
) -> AppServiceResult<BTreeMap<String, HydratedListRecord>> {
    let record_keys = items
        .iter()
        .filter_map(|item| RecordKey::parse(&item.record_key).ok())
        .collect::<Vec<_>>();
    if record_keys.is_empty() {
        return Ok(BTreeMap::new());
    }

    Ok(retrieval
        .get_records(GetRecordsRequest {
            record_keys: &record_keys,
        })?
        .records
        .into_iter()
        .map(|record| {
            (
                record.key.to_string(),
                HydratedListRecord {
                    view: crate::projection::localized_summary_view(&record, retrieval),
                    summary: record,
                },
            )
        })
        .collect())
}

fn hydrate_export_records(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    items: &[SavedListExportItemView],
) -> AppServiceResult<BTreeMap<String, atlas_domain::SourceRecordSummary>> {
    let record_keys = items
        .iter()
        .filter_map(|item| RecordKey::parse(&item.record_key).ok())
        .collect::<Vec<_>>();
    if record_keys.is_empty() {
        return Ok(BTreeMap::new());
    }

    Ok(retrieval
        .get_records(GetRecordsRequest {
            record_keys: &record_keys,
        })?
        .records
        .into_iter()
        .map(|record| (record.key.to_string(), record))
        .collect())
}

fn saved_list_export_item(item: SavedListItemView) -> SavedListExportItemView {
    let status = item.status;
    let (record_name, kind) = match item.record {
        Some(record) => (record.title, Some(record.kind)),
        None => (item.snapshot.title.clone(), item.snapshot.kind.clone()),
    };
    SavedListExportItemView {
        position: item.position,
        record_key: item.record_key,
        record_name,
        kind,
        status,
        note: item.note,
        snapshot: item.snapshot,
    }
}

fn is_batch_item_error(code: AppErrorCode) -> bool {
    matches!(
        code,
        AppErrorCode::InvalidRecordKey
            | AppErrorCode::RecordResolutionMiss
            | AppErrorCode::RecordResolutionAmbiguous
            | AppErrorCode::RecordNotFound
    )
}

fn filtered_saved_list_records(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    record_keys: &[RecordKey],
    filter: Option<&atlas_domain::QueryPredicate>,
) -> AppServiceResult<BTreeMap<String, HydratedListRecord>> {
    if record_keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let record_keys = record_keys.to_vec();
    let filter = filter.cloned();

    let mut records = BTreeMap::new();
    let mut page_number = 1;
    loop {
        let page = SearchPage::new(page_number, atlas_search::MAX_SEARCH_PAGE_SIZE)?;
        let result = retrieval.list_records(
            ListRecordsRequest::new(filter.as_ref(), page)
                .with_scope(RecordScope::Keys(&record_keys)),
        )?;
        for record in result.records {
            records.insert(
                record.key.to_string(),
                HydratedListRecord {
                    view: crate::projection::localized_summary_view(&record, retrieval),
                    summary: record,
                },
            );
        }
        if !result.page.has_more {
            break;
        }
        page_number += 1;
    }
    Ok(records)
}

fn searched_saved_list_records(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    record_keys: &[RecordKey],
    filter: Option<&atlas_domain::QueryPredicate>,
    query: &str,
) -> AppServiceResult<BTreeMap<String, HydratedListRecord>> {
    if record_keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let record_keys = record_keys.to_vec();
    let scoped_keys = record_keys.iter().cloned().collect::<BTreeSet<_>>();
    let filter = filter.cloned();
    let query = query.to_string();

    let mut records = BTreeMap::new();
    let mut page_number = 1;
    loop {
        let page = SearchPage::new(page_number, atlas_search::MAX_SEARCH_PAGE_SIZE)?;
        let result = retrieval.search_text(TextSearchRequest {
            ranking: Default::default(),
            search_profile: None,
            query: &query,
            filter: filter.as_ref(),
            scope: RecordScope::Keys(&record_keys),
            page,
            mode: RetrievalMode::Fts,
            prefer_remaster: true,
        })?;
        for record in result.records {
            if scoped_keys.contains(&record.record.key) {
                records.insert(
                    record.record.key.to_string(),
                    HydratedListRecord {
                        view: crate::projection::localized_summary_view(&record.record, retrieval),
                        summary: record.record,
                    },
                );
            }
        }
        if !result.page.has_more {
            break;
        }
        page_number += 1;
    }
    Ok(records)
}

fn resolve_record_ref(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    record_ref: &str,
) -> AppServiceResult<atlas_domain::SourceRecordSummary> {
    let record_ref = record_ref.to_string();

    let resolution = retrieval.resolve_record_ref(ResolveRecordRefRequest {
        record_ref: &record_ref,
        filter: None,
    })?;
    let key = match resolution {
        RecordRefResolutionResult::Key(key) => key,
        RecordRefResolutionResult::Miss => {
            return Err(AppServiceError::new(
                AppErrorCode::RecordResolutionMiss,
                format!("record resolution miss: {record_ref}"),
            ));
        }
        RecordRefResolutionResult::Ambiguous(matches) => {
            return Err(record_resolution_ambiguous_error(&record_ref, matches));
        }
    };
    let records = retrieval.get_records(GetRecordsRequest {
        record_keys: std::slice::from_ref(&key),
    })?;
    records.records.into_iter().next().ok_or_else(|| {
        AppServiceError::new(
            AppErrorCode::RecordNotFound,
            format!("record not found: {key}"),
        )
    })
}

fn record_resolution_ambiguous_error(
    record_ref: &str,
    matches: Vec<atlas_search::RecordResolutionResult>,
) -> AppServiceError {
    let details = RecordResolutionAmbiguousView {
        record_ref: record_ref.to_string(),
        matches: matches
            .into_iter()
            .map(record_resolution_candidate_view)
            .collect(),
    };
    let details = serde_json::to_value(details).unwrap_or_else(|_| {
        json!({
            "record_ref": record_ref,
            "matches": [],
        })
    });
    AppServiceError::from(
        atlas_app_model::AppError::new(
            AppErrorCode::RecordResolutionAmbiguous,
            format!("record resolution ambiguous: {record_ref}"),
        )
        .with_details(details),
    )
}

fn record_resolution_candidate_view(
    resolution: atlas_search::RecordResolutionResult,
) -> RecordResolutionCandidateView {
    RecordResolutionCandidateView {
        record: record_summary_view(&resolution.record),
        query: resolution.query,
        normalized_query: resolution.normalized_query,
        match_kind: match resolution.match_kind {
            atlas_search::RecordResolutionMatchKind::Name => "name",
            atlas_search::RecordResolutionMatchKind::NormalizedName => "normalized_name",
            atlas_search::RecordResolutionMatchKind::Alias => "alias",
        }
        .into(),
        matched_text: resolution.matched_text,
        evidence: resolution.evidence,
    }
}

fn saved_list_not_found(list_ref: &str) -> AppServiceError {
    AppServiceError::new(
        AppErrorCode::SavedListNotFound,
        format!("saved list `{list_ref}` was not found"),
    )
}

fn saved_list_summary(list: SavedList) -> SavedListSummaryView {
    SavedListSummaryView {
        list_key: list.list_key,
        slug: list.slug,
        name: list.name,
        description: list.description,
        tags: list.tags,
        item_count: list.item_count,
        created_at: list.created_at,
        updated_at: list.updated_at,
    }
}

fn saved_list_item_view(
    item: SavedListItem,
    records_by_key: &BTreeMap<String, HydratedListRecord>,
) -> SavedListItemView {
    let record = records_by_key.get(&item.record_key);
    let hydrated = hydrate_saved_list_item(item, record.map(|r| &r.summary));
    saved_list_item_from_hydrated(hydrated, record.map(|r| r.view.clone()))
}

fn saved_list_item_from_hydrated(
    item: HydratedSavedListItem<&atlas_domain::SourceRecordSummary>,
    record: Option<atlas_app_model::RecordSummaryView>,
) -> SavedListItemView {
    SavedListItemView {
        record_key: item.record_key,
        position: item.position,
        note: item.note,
        status: saved_list_item_status(item.status),
        snapshot: SavedListItemSnapshotView {
            title: item.snapshot.title,
            kind: item.snapshot.kind,
        },
        record,
    }
}

struct HydratedListRecord {
    summary: atlas_domain::SourceRecordSummary,
    view: atlas_app_model::RecordSummaryView,
}

fn saved_list_item_status(status: SavedListItemStatus) -> SavedListItemStatusView {
    match status {
        SavedListItemStatus::Active => SavedListItemStatusView::Active,
        SavedListItemStatus::Unresolved => SavedListItemStatusView::Unresolved,
    }
}
