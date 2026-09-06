use std::collections::{BTreeMap, BTreeSet};

use atlas_app_model::{
    AddSavedListItemRequest, AppErrorCode, BatchAddSavedListItemsRequest,
    BatchSavedListItemMutationView, BatchSavedListItemOutcomeView, BatchSavedListItemResultView,
    CreateSavedListRequest, DeleteSavedListView, FilterSavedListRequest, ImportSavedListRequest,
    ImportSavedListView, RecordResolutionAmbiguousView, RecordResolutionCandidateView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView,
    SavedListExportDocumentView, SavedListExportItemView, SavedListExportListView,
    SavedListIndexView, SavedListItemMutationOutcomeView, SavedListItemMutationView,
    SavedListItemSnapshotView, SavedListItemStatusView, SavedListItemView, SavedListSummaryView,
    SavedListUpdateView, UpdateSavedListRequest,
};
use atlas_domain::RecordKey;
use atlas_local_state::{
    AddSavedListItemOutcome, HydratedSavedListItem, ImportSavedList, ImportSavedListItem,
    LocalStateStore, NewSavedList, ResolvedSavedListItem, SavedList, SavedListItem,
    SavedListItemStatus, UpdateSavedList, hydrate_saved_list_item,
};
use atlas_search::{
    AtlasRetrievalService, GetRecordsRequest, ListRecordsRequest, RecordRefResolutionResult,
    RecordRetrieval, RecordScope, ResolveRecordRefRequest, RetrievalMode, SearchPage,
    TextRetrieval, TextSearchRequest, TextSearchTuning,
};
use serde_json::json;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::{AppServiceError, AppServiceResult};
use crate::filter::lower_basic_filter;
use crate::projection::record_summary;
use crate::retrieval::{
    VerifiedRemasterLookup, verified_remaster_lookup, verified_remaster_lookups_for_records,
};
use crate::service::AtlasAppService;

struct HydratedSavedListRecords {
    records_by_key: BTreeMap<String, atlas_record::RetrievedRecord>,
    remaster_lookups_by_key: BTreeMap<String, VerifiedRemasterLookup>,
}

impl AtlasAppService {
    pub fn saved_lists(&self) -> AppServiceResult<SavedListIndexView> {
        let store = self.local_state_store()?;
        let lists = store.saved_lists().list()?;
        Ok(SavedListIndexView {
            lists: lists.into_iter().map(saved_list_summary).collect(),
        })
    }

    pub fn saved_list(&self, list_ref: &str) -> AppServiceResult<SavedListDetailView> {
        let list = self
            .local_state_store()?
            .saved_lists()
            .get_with_items(list_ref)?
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::SavedListNotFound,
                    format!("saved list `{list_ref}` was not found"),
                )
            })?;
        let hydrated_records = hydrate_saved_list_records(self, &list.items)?;
        Ok(SavedListDetailView {
            list: saved_list_summary(list.list),
            items: list
                .items
                .into_iter()
                .map(|item| saved_list_item_view(item, &hydrated_records))
                .collect::<AppServiceResult<Vec<_>>>()?,
        })
    }

    pub fn filter_saved_list(
        &self,
        request: FilterSavedListRequest,
    ) -> AppServiceResult<SavedListDetailView> {
        let filter = lower_basic_filter(request.filter.as_ref())?;
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
        let store = self.local_state_store()?;
        let list = store.saved_lists().create(NewSavedList {
            slug: request.slug,
            name: request.name,
            description: request.description,
            tags: request.tags,
        })?;
        Ok(SavedListCreateView {
            list: saved_list_summary(list),
        })
    }

    pub fn update_saved_list(
        &self,
        request: UpdateSavedListRequest,
    ) -> AppServiceResult<SavedListUpdateView> {
        let store = self.local_state_store()?;
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
    }

    pub fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> AppServiceResult<SavedListItemMutationView> {
        let record = resolve_record_ref(self, &request.record_ref)?;
        let record_key = record.identity.key.to_string();
        let record_name = record.identity.name.clone();
        let store = self.local_state_store()?;
        let outcome = store.saved_lists().add_resolved_item(
            &request.list_ref,
            ResolvedSavedListItem {
                record_key: record.identity.key,
                title_snapshot: record_name.clone(),
                kind_snapshot: Some(record.classification.kind.as_str().to_string()),
                note: request.note,
            },
        )?;
        let list = store
            .saved_lists()
            .get(&request.list_ref)?
            .ok_or_else(|| saved_list_not_found(&request.list_ref))?;
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
    }

    pub fn add_saved_list_items(
        &self,
        request: BatchAddSavedListItemsRequest,
    ) -> AppServiceResult<BatchSavedListItemMutationView> {
        let store = self.local_state_store()?;
        let list = store
            .saved_lists()
            .get(&request.list_ref)?
            .ok_or_else(|| saved_list_not_found(&request.list_ref))?;
        let mut items = Vec::new();
        let mut added_count = 0;
        let mut already_present_count = 0;
        let mut failed_count = 0;
        for item in request.items {
            let input = item.record_ref;
            let record = match resolve_record_ref(self, &input) {
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
            let record_key = record.identity.key.to_string();
            let record_name = record.identity.name.clone();
            let outcome = store.saved_lists().add_resolved_item(
                &list.list_key,
                ResolvedSavedListItem {
                    record_key: record.identity.key,
                    title_snapshot: record_name.clone(),
                    kind_snapshot: Some(record.classification.kind.as_str().to_string()),
                    note: item.note,
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
    }

    pub fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> AppServiceResult<SavedListItemMutationView> {
        let (record_key, record_name) = if let Ok(key) = RecordKey::parse(&request.record_ref) {
            (key, None)
        } else {
            let record = resolve_record_ref(self, &request.record_ref)?;
            (record.identity.key, Some(record.identity.name))
        };
        let store = self.local_state_store()?;
        let removed = store
            .saved_lists()
            .remove_item(&request.list_ref, &record_key.to_string())?;
        let list = store
            .saved_lists()
            .get(&request.list_ref)?
            .ok_or_else(|| saved_list_not_found(&request.list_ref))?;
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
    }

    pub fn delete_saved_list(&self, list_ref: &str) -> AppServiceResult<DeleteSavedListView> {
        let store = self.local_state_store()?;
        let list = store
            .saved_lists()
            .get(list_ref)?
            .ok_or_else(|| saved_list_not_found(list_ref))?;
        let deleted = store.saved_lists().delete(&list.list_key)?;
        Ok(DeleteSavedListView {
            list_key: list.list_key,
            slug: list.slug,
            deleted,
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
        let records_by_key = hydrate_export_records(self, &request.document.items)?;
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
                        .map(|record| record.record.identity.name.clone())
                        .unwrap_or_else(|| item.snapshot.title),
                    record_kind_snapshot: record
                        .map(|record| record.record.classification.kind.as_str().to_string())
                        .or(item.snapshot.kind),
                }
            })
            .collect::<Vec<_>>();
        let store = self.local_state_store()?;
        let replaced = store.saved_lists().get(&id)?.is_some() && request.replace;
        let imported = store.saved_lists().import(ImportSavedList {
            slug: id,
            name: request.document.list.name,
            description: request.document.list.description,
            tags: request.document.list.tags,
            items: imported,
            replace: request.replace,
        })?;
        let hydrated_records = hydrate_saved_list_records(self, &imported.items)?;
        let mut active_count = 0;
        let mut unresolved_count = 0;
        for item in &imported.items {
            if hydrated_records
                .records_by_key
                .contains_key(&item.record_key)
            {
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
    }

    pub(crate) fn saved_list_record_keys(
        &self,
        list_ref: &str,
    ) -> AppServiceResult<Vec<RecordKey>> {
        Ok(self
            .local_state_store()?
            .saved_lists()
            .get_with_items(list_ref)?
            .ok_or_else(|| saved_list_not_found(list_ref))?
            .items
            .into_iter()
            .filter_map(|item| RecordKey::parse(&item.record_key).ok())
            .collect())
    }

    fn saved_list_with_filter(
        &self,
        list_ref: &str,
        filter: Option<&atlas_domain::SearchFilterNode>,
        query: Option<&str>,
        has_match_scope: bool,
    ) -> AppServiceResult<SavedListDetailView> {
        let list = self
            .local_state_store()?
            .saved_lists()
            .get_with_items(list_ref)?
            .ok_or_else(|| saved_list_not_found(list_ref))?;
        let active_keys = list
            .items
            .iter()
            .filter_map(|item| RecordKey::parse(&item.record_key).ok())
            .collect::<Vec<_>>();
        let hydrated_records = if let Some(query) = query {
            searched_saved_list_records(self, &active_keys, filter, query)?
        } else if has_match_scope {
            filtered_saved_list_records(self, &active_keys, filter)?
        } else {
            hydrate_saved_list_records(self, &list.items)?
        };
        Ok(SavedListDetailView {
            list: saved_list_summary(list.list),
            items: list
                .items
                .into_iter()
                .filter(|item| {
                    !has_match_scope
                        || hydrated_records
                            .records_by_key
                            .contains_key(&item.record_key)
                })
                .map(|item| saved_list_item_view(item, &hydrated_records))
                .collect::<AppServiceResult<Vec<_>>>()?,
        })
    }

    pub(crate) fn local_state_store(&self) -> AppServiceResult<LocalStateStore> {
        LocalStateStore::open(self.local_state_path.clone()).map_err(Into::into)
    }
}

fn hydrate_saved_list_records(
    service: &AtlasAppService,
    items: &[SavedListItem],
) -> AppServiceResult<HydratedSavedListRecords> {
    let record_keys = items
        .iter()
        .filter_map(|item| RecordKey::parse(&item.record_key).ok())
        .collect::<Vec<_>>();
    if record_keys.is_empty() {
        return Ok(HydratedSavedListRecords {
            records_by_key: BTreeMap::new(),
            remaster_lookups_by_key: BTreeMap::new(),
        });
    }
    service.submit_retrieval(move |retrieval| {
        let records_by_key = retrieval
            .get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?
            .into_iter()
            .map(|retrieved| (retrieved.record.identity.key.to_string(), retrieved))
            .collect::<BTreeMap<_, _>>();
        let remaster_lookups_by_key =
            verified_remaster_lookups_for_records(retrieval, records_by_key.values())?;
        Ok(HydratedSavedListRecords {
            records_by_key,
            remaster_lookups_by_key,
        })
    })
}

fn hydrate_export_records(
    service: &AtlasAppService,
    items: &[SavedListExportItemView],
) -> AppServiceResult<BTreeMap<String, atlas_record::RetrievedRecord>> {
    let record_keys = items
        .iter()
        .filter_map(|item| RecordKey::parse(&item.record_key).ok())
        .collect::<Vec<_>>();
    if record_keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    service.submit_retrieval(move |retrieval| {
        Ok(retrieval
            .get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?
            .into_iter()
            .map(|retrieved| (retrieved.record.identity.key.to_string(), retrieved))
            .collect())
    })
}

fn saved_list_export_item(item: SavedListItemView) -> SavedListExportItemView {
    let status = item.status;
    let (record_name, kind) = match item.record {
        Some(record) => (
            record.surface.metadata.title,
            Some(record.surface.metadata.kind),
        ),
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
    service: &AtlasAppService,
    record_keys: &[RecordKey],
    filter: Option<&atlas_domain::SearchFilterNode>,
) -> AppServiceResult<HydratedSavedListRecords> {
    if record_keys.is_empty() {
        return Ok(HydratedSavedListRecords {
            records_by_key: BTreeMap::new(),
            remaster_lookups_by_key: BTreeMap::new(),
        });
    }
    let record_keys = record_keys.to_vec();
    let filter = filter.cloned();
    service.submit_retrieval(move |retrieval| {
        let mut records = BTreeMap::new();
        let mut page_number = 1;
        loop {
            let page = SearchPage::new(page_number, atlas_search::MAX_SEARCH_PAGE_SIZE)?;
            let result = retrieval.list_records(
                ListRecordsRequest::new(filter.as_ref(), page)
                    .with_scope(RecordScope::Keys(&record_keys))
                    .with_sort(atlas_search::RecordListSort::RecordKey),
            )?;
            for retrieved in result.records {
                records.insert(retrieved.record.identity.key.to_string(), retrieved);
            }
            if !result.page.has_more {
                break;
            }
            page_number += 1;
        }
        let remaster_lookups_by_key =
            verified_remaster_lookups_for_records(retrieval, records.values())?;
        Ok(HydratedSavedListRecords {
            records_by_key: records,
            remaster_lookups_by_key,
        })
    })
}

fn searched_saved_list_records(
    service: &AtlasAppService,
    record_keys: &[RecordKey],
    filter: Option<&atlas_domain::SearchFilterNode>,
    query: &str,
) -> AppServiceResult<HydratedSavedListRecords> {
    if record_keys.is_empty() {
        return Ok(HydratedSavedListRecords {
            records_by_key: BTreeMap::new(),
            remaster_lookups_by_key: BTreeMap::new(),
        });
    }
    let record_keys = record_keys.to_vec();
    let scoped_keys = record_keys.iter().cloned().collect::<BTreeSet<_>>();
    let filter = filter.cloned();
    let query = query.to_string();
    service.submit_retrieval(move |retrieval| {
        let mut records = BTreeMap::new();
        let mut page_number = 1;
        let tuning = TextSearchTuning::default()
            .with_retrieval(RetrievalMode::Fts)
            .with_candidate_windows(
                atlas_search::MAX_RANKED_CANDIDATE_WINDOW,
                atlas_search::MAX_RANKED_CANDIDATE_WINDOW,
            );
        loop {
            let page = SearchPage::new(page_number, atlas_search::MAX_SEARCH_PAGE_SIZE)?;
            let result = retrieval.search_text(TextSearchRequest {
                query: &query,
                exclude: None,
                filter: filter.as_ref(),
                scope: RecordScope::Keys(&record_keys),
                page,
                tuning: Some(tuning),
                explain: false,
            })?;
            for result_record in result.records {
                if scoped_keys.contains(&result_record.record.record.identity.key) {
                    records.insert(
                        result_record.record.record.identity.key.to_string(),
                        result_record.record,
                    );
                }
            }
            if !result.page.has_more {
                break;
            }
            page_number += 1;
        }
        let remaster_lookups_by_key =
            verified_remaster_lookups_for_records(retrieval, records.values())?;
        Ok(HydratedSavedListRecords {
            records_by_key: records,
            remaster_lookups_by_key,
        })
    })
}

fn resolve_record_ref(
    service: &AtlasAppService,
    record_ref: &str,
) -> AppServiceResult<atlas_record::AtlasRecord> {
    let record_ref = record_ref.to_string();
    service.submit_retrieval(move |retrieval| {
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
                return Err(record_resolution_ambiguous_error(
                    retrieval,
                    &record_ref,
                    matches,
                )?);
            }
        };
        let records = retrieval.get_records(GetRecordsRequest {
            record_keys: std::slice::from_ref(&key),
        })?;
        records
            .into_iter()
            .next()
            .map(|record| record.record)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::RecordNotFound,
                    format!("record not found: {key}"),
                )
            })
    })
}

fn record_resolution_ambiguous_error(
    retrieval: &AtlasRetrievalService,
    record_ref: &str,
    matches: Vec<atlas_search::RecordResolutionResult>,
) -> AppServiceResult<AppServiceError> {
    let details = RecordResolutionAmbiguousView {
        record_ref: record_ref.to_string(),
        matches: matches
            .into_iter()
            .map(|resolution| {
                let remaster_lookup = verified_remaster_lookup(retrieval, &resolution.record)?;
                Ok(record_resolution_candidate_view(
                    resolution,
                    &remaster_lookup,
                ))
            })
            .collect::<AppServiceResult<Vec<_>>>()?,
    };
    let details = serde_json::to_value(details).unwrap_or_else(|_| {
        json!({
            "record_ref": record_ref,
            "matches": [],
        })
    });
    Ok(AppServiceError::from(
        atlas_app_model::AppError::new(
            AppErrorCode::RecordResolutionAmbiguous,
            format!("record resolution ambiguous: {record_ref}"),
        )
        .with_details(details),
    ))
}

fn record_resolution_candidate_view(
    resolution: atlas_search::RecordResolutionResult,
    remaster_lookup: &VerifiedRemasterLookup,
) -> RecordResolutionCandidateView {
    RecordResolutionCandidateView {
        record: record_summary(&resolution.record, remaster_lookup),
        query: resolution.query,
        normalized_query: resolution.normalized_query,
        match_kind: resolution.match_kind.as_str().to_string(),
        matched_text: resolution.matched_text,
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
    hydrated_records: &HydratedSavedListRecords,
) -> AppServiceResult<SavedListItemView> {
    let record = hydrated_records.records_by_key.get(&item.record_key);
    let remaster_lookup = hydrated_records
        .remaster_lookups_by_key
        .get(&item.record_key);
    let hydrated = hydrate_saved_list_item(item, record);
    saved_list_item_from_hydrated(hydrated, remaster_lookup)
}

fn saved_list_item_from_hydrated(
    item: HydratedSavedListItem<&atlas_record::RetrievedRecord>,
    remaster_lookup: Option<&VerifiedRemasterLookup>,
) -> AppServiceResult<SavedListItemView> {
    let record = match item.record {
        Some(record) => {
            let remaster_lookup = remaster_lookup.ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::InternalError,
                    format!(
                        "canonical saved-list record `{}` is missing its authenticated remaster lookup",
                        record.record.identity.key
                    ),
                )
            })?;
            Some(record_summary(record, remaster_lookup))
        }
        None => None,
    };
    Ok(SavedListItemView {
        record_key: item.record_key,
        position: item.position,
        note: item.note,
        status: saved_list_item_status(item.status),
        snapshot: SavedListItemSnapshotView {
            title: item.snapshot.title,
            kind: item.snapshot.kind,
        },
        record,
    })
}

fn saved_list_item_status(status: SavedListItemStatus) -> SavedListItemStatusView {
    match status {
        SavedListItemStatus::Active => SavedListItemStatusView::Active,
        SavedListItemStatus::Unresolved => SavedListItemStatusView::Unresolved,
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        AddSavedListItemRequest, AppErrorCode, BasicSearchFilter, BatchAddSavedListItemsRequest,
        BatchSavedListItemInput, BatchSavedListItemOutcomeView, CreateSavedListRequest,
        FilterClause, FilterClauseOperator, FilterSavedListRequest, ImportSavedListRequest,
        RecordSurfaceEditionCounterpartRoleView, RecordSurfaceEditionStatusView,
        RemoveSavedListItemRequest, SavedListItemMutationOutcomeView, SavedListItemStatusView,
        UpdateSavedListRequest,
    };
    use atlas_domain::{RecordKey, RecordKind, RemasterLinkSource};
    use atlas_local_state::{NewSavedList, ResolvedSavedListItem};
    use atlas_search::{
        RecordResolutionMatchKind, RecordResolutionResult, RemasterLinkResult, RemasterLinksResult,
    };

    use crate::test_support::fixture_worker;

    const AIR_MEPHIT_KEY: &str = "pathfinder-bestiary:KDRlxdIUADWHI6Vr";
    const AIR_SCAMP_KEY: &str = "pathfinder-monster-core:MSm1im7lZA5i82rz";

    #[test]
    fn linked_air_pair_ambiguous_candidates_include_verified_counterparts() {
        let air_mephit = edition_record(AIR_MEPHIT_KEY, "Air Mephit", false);
        let air_scamp = edition_record(AIR_SCAMP_KEY, "Air Scamp", true);
        let link = RemasterLinkResult {
            remaster_record: air_scamp.clone(),
            legacy_record: air_mephit.clone(),
            source: RemasterLinkSource::RemasterJournal,
            source_ref: "journal:Bestiaries".to_string(),
        };
        let legacy_candidate = ambiguous_candidate(
            air_mephit.clone(),
            RemasterLinksResult {
                seed: air_mephit,
                links: vec![link.clone()],
            },
        );
        let remaster_candidate = ambiguous_candidate(
            air_scamp.clone(),
            RemasterLinksResult {
                seed: air_scamp,
                links: vec![link],
            },
        );

        let legacy_edition = legacy_candidate
            .record
            .surface
            .metadata
            .edition
            .expect("legacy ambiguity candidate should expose edition metadata");
        assert_eq!(
            legacy_edition.status,
            RecordSurfaceEditionStatusView::Legacy
        );
        assert_eq!(legacy_edition.counterparts.len(), 1);
        assert_eq!(
            legacy_edition.counterparts[0].role,
            RecordSurfaceEditionCounterpartRoleView::RemasteredCounterpart
        );
        assert_eq!(legacy_edition.counterparts[0].record_key, AIR_SCAMP_KEY);
        assert_eq!(legacy_edition.counterparts[0].title, "Air Scamp");

        let remaster_edition = remaster_candidate
            .record
            .surface
            .metadata
            .edition
            .expect("remaster ambiguity candidate should expose edition metadata");
        assert_eq!(
            remaster_edition.status,
            RecordSurfaceEditionStatusView::Remaster
        );
        assert_eq!(remaster_edition.counterparts.len(), 1);
        assert_eq!(
            remaster_edition.counterparts[0].role,
            RecordSurfaceEditionCounterpartRoleView::LegacyCounterpart
        );
        assert_eq!(remaster_edition.counterparts[0].record_key, AIR_MEPHIT_KEY);
        assert_eq!(remaster_edition.counterparts[0].title, "Air Mephit");
    }

    fn ambiguous_candidate(
        record: atlas_record::RetrievedRecord,
        links: RemasterLinksResult,
    ) -> atlas_app_model::RecordResolutionCandidateView {
        let resolution = RecordResolutionResult {
            query: "Air".to_string(),
            normalized_query: "air".to_string(),
            match_kind: RecordResolutionMatchKind::Name,
            matched_text: record.record.identity.name.clone(),
            alias_source: None,
            alias_source_ref: None,
            record,
        };
        let lookup = crate::retrieval::VerifiedRemasterLookup::from_test_result(links);
        super::record_resolution_candidate_view(resolution, &lookup)
    }

    fn edition_record(
        record_key: &str,
        title: &str,
        remaster: bool,
    ) -> atlas_record::RetrievedRecord {
        let mut record = atlas_record::AtlasRecord::new(
            atlas_record::RecordIdentity::new(
                RecordKey::parse(record_key).expect("edition fixture key"),
                title,
            ),
            atlas_record::RecordClassification::new(RecordKind::Creature),
            atlas_record::FoundryRecordInfo::new(
                "Edition Fixture",
                atlas_record::FoundryDocumentType::Actor,
                atlas_record::FoundryRecordType::Npc,
            ),
            atlas_record::RecordProvenance::new(format!("fixtures/{record_key}.json")),
        );
        record.publication.remaster = remaster;
        atlas_record::RetrievedRecord {
            record,
            body: None,
            spell_children: Vec::new(),
        }
    }

    #[test]
    fn saved_lists_returns_local_state_summaries() {
        let fixture = fixture_worker();
        let store = fixture
            .worker
            .local_state_store()
            .expect("fixture local state should open");
        store
            .saved_lists()
            .create(NewSavedList {
                slug: "z-last".to_string(),
                name: "Z Last".to_string(),
                description: None,
                tags: vec![],
            })
            .expect("list should create");
        store
            .saved_lists()
            .create(NewSavedList {
                slug: "a-first".to_string(),
                name: "A First".to_string(),
                description: Some("Campaign prep".to_string()),
                tags: vec!["story-beat".to_string()],
            })
            .expect("list should create");

        let view = fixture
            .worker
            .saved_lists()
            .expect("saved lists should load");

        assert_eq!(
            view.lists
                .iter()
                .map(|list| list.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["a-first", "z-last"]
        );
        assert_eq!(view.lists[0].description.as_deref(), Some("Campaign prep"));
        assert_eq!(view.lists[0].tags, vec!["story-beat"]);
    }

    #[test]
    fn saved_list_hydrates_active_records_and_preserves_unresolved_items() {
        let fixture = fixture_worker();
        let store = fixture
            .worker
            .local_state_store()
            .expect("fixture local state should open");
        store
            .saved_lists()
            .create(NewSavedList {
                slug: "research".to_string(),
                name: "Research".to_string(),
                description: None,
                tags: vec![],
            })
            .expect("list should create");
        store
            .saved_lists()
            .add_resolved_item(
                "research",
                ResolvedSavedListItem {
                    record_key: RecordKey::parse("actions:testAction1")
                        .expect("fixture key should parse"),
                    note: Some("active note".to_string()),
                    title_snapshot: "Old Action Name".to_string(),
                    kind_snapshot: Some("old-kind".to_string()),
                },
            )
            .expect("active item should insert");
        store
            .saved_lists()
            .add_resolved_item(
                "research",
                ResolvedSavedListItem {
                    record_key: RecordKey::parse("actions:missing")
                        .expect("fixture key should parse"),
                    note: None,
                    title_snapshot: "Missing Action".to_string(),
                    kind_snapshot: Some("rule".to_string()),
                },
            )
            .expect("unresolved item should insert");

        let view = fixture
            .worker
            .saved_list("research")
            .expect("saved list should load");

        assert_eq!(view.list.slug, "research");
        assert_eq!(view.items.len(), 2);
        assert_eq!(view.items[0].status, SavedListItemStatusView::Active);
        let active_record = view.items[0]
            .record
            .as_ref()
            .expect("active item should hydrate");
        assert_eq!(active_record.surface.metadata.title, "Test Action 1");
        let edition =
            active_record.surface.metadata.edition.as_ref().expect(
                "canonical saved-list summary should expose authenticated edition metadata",
            );
        assert_eq!(edition.status, RecordSurfaceEditionStatusView::Legacy);
        assert!(edition.counterparts.is_empty());
        assert_eq!(view.items[1].status, SavedListItemStatusView::Unresolved);
        assert!(view.items[1].record.is_none());
        assert_eq!(view.items[1].snapshot.title, "Missing Action");
    }

    #[test]
    fn filtered_saved_list_uses_record_scope_and_hides_unresolved_items() {
        let fixture = fixture_worker();
        let store = fixture
            .worker
            .local_state_store()
            .expect("fixture local state should open");
        store
            .saved_lists()
            .create(NewSavedList {
                slug: "research".to_string(),
                name: "Research".to_string(),
                description: None,
                tags: vec![],
            })
            .expect("list should create");
        for record_key in ["actions:testAction1", "actions:missing"] {
            store
                .saved_lists()
                .add_resolved_item(
                    "research",
                    ResolvedSavedListItem {
                        record_key: RecordKey::parse(record_key).expect("fixture key should parse"),
                        note: None,
                        title_snapshot: record_key.to_string(),
                        kind_snapshot: Some("rule".to_string()),
                    },
                )
                .expect("item should insert");
        }

        let view = fixture
            .worker
            .filter_saved_list(FilterSavedListRequest {
                list_ref: "research".to_string(),
                query: None,
                filter: Some(BasicSearchFilter {
                    clauses: vec![FilterClause {
                        id: "kind-include_any".to_string(),
                        field: "kind".to_string(),
                        operator: FilterClauseOperator::IncludeAny,
                        values: vec!["rule".to_string()],
                        range: None,
                        metric: None,
                    }],
                }),
            })
            .expect("filtered saved list should load");

        assert_eq!(
            view.items
                .iter()
                .map(|item| item.record_key.as_str())
                .collect::<Vec<_>>(),
            vec!["actions:testAction1"]
        );
        assert_eq!(view.items[0].status, SavedListItemStatusView::Active);
    }

    #[test]
    fn searched_saved_list_matches_only_records_in_the_list() {
        let fixture = fixture_worker();
        let store = fixture
            .worker
            .local_state_store()
            .expect("fixture local state should open");
        store
            .saved_lists()
            .create(NewSavedList {
                slug: "research".to_string(),
                name: "Research".to_string(),
                description: None,
                tags: vec![],
            })
            .expect("list should create");
        store
            .saved_lists()
            .add_resolved_item(
                "research",
                ResolvedSavedListItem {
                    record_key: RecordKey::parse("actions:testAction2")
                        .expect("fixture key should parse"),
                    note: None,
                    title_snapshot: "Test Action 2".to_string(),
                    kind_snapshot: Some("rule".to_string()),
                },
            )
            .expect("item should insert");

        let missing = fixture
            .worker
            .filter_saved_list(FilterSavedListRequest {
                list_ref: "research".to_string(),
                query: Some("No Such Search Needle".to_string()),
                filter: None,
            })
            .expect("searched saved list should load");
        let matched = fixture
            .worker
            .filter_saved_list(FilterSavedListRequest {
                list_ref: "research".to_string(),
                query: Some("Test Action 2".to_string()),
                filter: None,
            })
            .expect("searched saved list should load");

        assert!(missing.items.is_empty());
        assert_eq!(
            matched
                .items
                .iter()
                .map(|item| item.record_key.as_str())
                .collect::<Vec<_>>(),
            vec!["actions:testAction2"]
        );
    }

    #[test]
    fn saved_list_reports_missing_ref() {
        let fixture = fixture_worker();

        let error = fixture
            .worker
            .saved_list("missing")
            .expect_err("missing list should fail")
            .into_app_error();

        assert_eq!(error.code, AppErrorCode::SavedListNotFound);
    }

    #[test]
    fn saved_list_reports_invalid_ref() {
        let fixture = fixture_worker();

        let error = fixture
            .worker
            .saved_list("")
            .expect_err("invalid ref should fail")
            .into_app_error();

        assert_eq!(error.code, AppErrorCode::InvalidRequest);
    }

    #[test]
    fn saved_list_mutations_create_add_remove_and_delete() {
        let fixture = fixture_worker();

        let created = fixture
            .worker
            .create_saved_list(CreateSavedListRequest {
                slug: "research".to_string(),
                name: "Research".to_string(),
                description: Some("Campaign prep".to_string()),
                tags: vec!["story-beat".to_string(), " boss ".to_string()],
            })
            .expect("list should create");
        assert_eq!(created.list.slug, "research");
        assert!(created.list.list_key.starts_with("list_"));
        assert_eq!(created.list.description.as_deref(), Some("Campaign prep"));
        assert_eq!(created.list.tags, vec!["boss", "story-beat"]);

        let updated = fixture
            .worker
            .update_saved_list(UpdateSavedListRequest {
                list_key: created.list.list_key.clone(),
                slug: "renamed-research".to_string(),
                name: "Renamed Research".to_string(),
                description: Some("Updated prep".to_string()),
                tags: vec!["renamed".to_string()],
            })
            .expect("list should update");
        assert_eq!(updated.list.list_key, created.list.list_key);
        assert_eq!(updated.list.slug, "renamed-research");
        assert_eq!(updated.list.name, "Renamed Research");
        assert_eq!(updated.list.description.as_deref(), Some("Updated prep"));
        assert_eq!(updated.list.tags, vec!["renamed"]);

        let added = fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                list_ref: "renamed-research".to_string(),
                record_ref: "Test Action 1".to_string(),
                note: Some("important".to_string()),
            })
            .expect("item should add");
        assert_eq!(added.list_key, created.list.list_key);
        assert_eq!(added.slug, "renamed-research");
        assert_eq!(added.record_key, "actions:testAction1");
        assert_eq!(added.outcome, SavedListItemMutationOutcomeView::Added);

        let duplicate = fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                list_ref: created.list.list_key.clone(),
                record_ref: "actions:testAction1".to_string(),
                note: None,
            })
            .expect("duplicate add should be a no-op");
        assert_eq!(
            duplicate.outcome,
            SavedListItemMutationOutcomeView::AlreadyPresent
        );

        let removed = fixture
            .worker
            .remove_saved_list_item(RemoveSavedListItemRequest {
                list_ref: created.list.list_key.clone(),
                record_ref: "Test Action 1".to_string(),
            })
            .expect("item should remove by resolvable name");
        assert_eq!(removed.outcome, SavedListItemMutationOutcomeView::Removed);

        let absent = fixture
            .worker
            .remove_saved_list_item(RemoveSavedListItemRequest {
                list_ref: "renamed-research".to_string(),
                record_ref: "actions:testAction1".to_string(),
            })
            .expect("absent item remove should be a no-op");
        assert_eq!(absent.outcome, SavedListItemMutationOutcomeView::NotPresent);

        let deleted = fixture
            .worker
            .delete_saved_list(&created.list.list_key)
            .expect("list should delete");
        assert_eq!(deleted.list_key, created.list.list_key);
        assert_eq!(deleted.slug, "renamed-research");
        assert!(deleted.deleted);
    }

    #[test]
    fn saved_list_batch_add_reports_partial_success() {
        let fixture = fixture_worker();
        fixture
            .worker
            .create_saved_list(CreateSavedListRequest {
                slug: "research".to_string(),
                name: "Research".to_string(),
                description: None,
                tags: vec![],
            })
            .expect("list should create");

        let result = fixture
            .worker
            .add_saved_list_items(BatchAddSavedListItemsRequest {
                list_ref: "research".to_string(),
                items: vec![
                    BatchSavedListItemInput {
                        record_ref: "Test Action 1".to_string(),
                        note: Some("shared".to_string()),
                    },
                    BatchSavedListItemInput {
                        record_ref: "No Such Record".to_string(),
                        note: Some("shared".to_string()),
                    },
                    BatchSavedListItemInput {
                        record_ref: "actions:testAction2".to_string(),
                        note: Some("shared".to_string()),
                    },
                ],
            })
            .expect("batch add should report item failures without aborting");

        assert_eq!(result.requested_count, 3);
        assert_eq!(result.added_count, 2);
        assert_eq!(result.failed_count, 1);
        assert_eq!(
            result.items[0].outcome,
            BatchSavedListItemOutcomeView::Added
        );
        assert_eq!(
            result.items[0].record_key.as_deref(),
            Some("actions:testAction1")
        );
        assert_eq!(
            result.items[0].record_name.as_deref(),
            Some("Test Action 1")
        );
        assert_eq!(
            result.items[1].outcome,
            BatchSavedListItemOutcomeView::Failed
        );
        assert_eq!(
            result.items[1]
                .error
                .as_ref()
                .expect("failure should include error")
                .code,
            AppErrorCode::RecordResolutionMiss
        );
        assert_eq!(
            result.items[2].outcome,
            BatchSavedListItemOutcomeView::Added
        );

        let list = fixture
            .worker
            .saved_list("research")
            .expect("saved list should load");
        assert_eq!(
            list.items
                .iter()
                .map(|item| item.record_key.as_str())
                .collect::<Vec<_>>(),
            vec!["actions:testAction1", "actions:testAction2"]
        );
        assert_eq!(list.items[0].note.as_deref(), Some("shared"));
        assert_eq!(list.items[1].note.as_deref(), Some("shared"));
    }

    #[test]
    fn saved_list_export_import_handles_conflict_id_override_and_replace() {
        let fixture = fixture_worker();
        fixture
            .worker
            .create_saved_list(CreateSavedListRequest {
                slug: "session-prep".to_string(),
                name: "Session Prep".to_string(),
                description: Some("Original".to_string()),
                tags: vec!["story-beat".to_string()],
            })
            .expect("list should create");
        fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                list_ref: "session-prep".to_string(),
                record_ref: "Test Action 1".to_string(),
                note: Some("Bring up early".to_string()),
            })
            .expect("item should add");

        let document = fixture
            .worker
            .export_saved_list("session-prep")
            .expect("list should export");
        assert_eq!(document.format, "pf2e-atlas.saved-list");
        assert_eq!(document.version, 1);
        assert_eq!(document.list.id, "session-prep");
        assert_eq!(document.list.tags, vec!["story-beat"]);
        assert_eq!(document.items.len(), 1);
        assert_eq!(document.items[0].record_key, "actions:testAction1");
        assert_eq!(document.items[0].record_name, "Test Action 1");
        assert_eq!(document.items[0].note.as_deref(), Some("Bring up early"));

        let conflict = fixture
            .worker
            .import_saved_list(ImportSavedListRequest {
                document: document.clone(),
                id: None,
                replace: false,
            })
            .expect_err("existing import id should conflict by default")
            .into_app_error();
        assert_eq!(conflict.code, AppErrorCode::SavedListAlreadyExists);

        let imported = fixture
            .worker
            .import_saved_list(ImportSavedListRequest {
                document: document.clone(),
                id: Some("session-copy".to_string()),
                replace: false,
            })
            .expect("id override should import a new list");
        assert_eq!(imported.list.slug, "session-copy");
        assert_eq!(imported.list.tags, vec!["story-beat"]);
        assert!(!imported.replaced);
        assert_eq!(imported.active_count, 1);
        assert_eq!(imported.unresolved_count, 0);

        fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                list_ref: "session-copy".to_string(),
                record_ref: "Test Action 2".to_string(),
                note: None,
            })
            .expect("extra item should add");
        let replaced = fixture
            .worker
            .import_saved_list(ImportSavedListRequest {
                document,
                id: Some("session-copy".to_string()),
                replace: true,
            })
            .expect("replace should target final id override");
        assert!(replaced.replaced);
        assert_eq!(replaced.list.slug, "session-copy");

        let copied = fixture
            .worker
            .saved_list("session-copy")
            .expect("copy should load");
        assert_eq!(copied.list.name, "Session Prep");
        assert_eq!(copied.list.description.as_deref(), Some("Original"));
        assert_eq!(copied.list.tags, vec!["story-beat"]);
        assert_eq!(copied.items.len(), 1);
        assert_eq!(copied.items[0].record_key, "actions:testAction1");
    }

    #[test]
    fn saved_list_add_reports_resolution_miss() {
        let fixture = fixture_worker();
        fixture
            .worker
            .create_saved_list(CreateSavedListRequest {
                slug: "research".to_string(),
                name: "Research".to_string(),
                description: None,
                tags: vec![],
            })
            .expect("list should create");

        let error = fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                list_ref: "research".to_string(),
                record_ref: "No Such Record".to_string(),
                note: None,
            })
            .expect_err("missing record ref should fail")
            .into_app_error();

        assert_eq!(error.code, AppErrorCode::RecordResolutionMiss);
    }
}
