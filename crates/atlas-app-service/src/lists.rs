use std::collections::BTreeMap;

use atlas_app_model::{
    AddSavedListItemRequest, AppErrorCode, CreateSavedListRequest, DeleteSavedListView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView, SavedListIndexView,
    SavedListItemMutationOutcomeView, SavedListItemMutationView, SavedListItemSnapshotView,
    SavedListItemStatusView, SavedListItemView, SavedListSummaryView,
};
use atlas_domain::RecordKey;
use atlas_local_state::{
    AddSavedListItemOutcome, HydratedSavedListItem, LocalStateStore, NewSavedList,
    ResolvedSavedListItem, SavedList, SavedListItem, SavedListItemStatus, hydrate_saved_list_item,
};
use atlas_search::{
    GetRecordsRequest, RecordRefResolutionResult, RecordRetrieval, ResolveRecordRefRequest,
};
use serde_json::json;

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_summary;
use crate::service::AtlasAppService;

impl AtlasAppService {
    pub fn saved_lists(&self) -> AppServiceResult<SavedListIndexView> {
        let store = self.local_state_store()?;
        let lists = store.saved_lists().list()?;
        Ok(SavedListIndexView {
            lists: lists.into_iter().map(saved_list_summary).collect(),
        })
    }

    pub fn saved_list(&self, slug: &str) -> AppServiceResult<SavedListDetailView> {
        let list = self
            .local_state_store()?
            .saved_lists()
            .get_with_items(slug)?
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::SavedListNotFound,
                    format!("saved list `{slug}` was not found"),
                )
            })?;
        let records_by_key = hydrate_saved_list_records(self, &list.items)?;
        Ok(SavedListDetailView {
            list: saved_list_summary(list.list),
            items: list
                .items
                .into_iter()
                .map(|item| saved_list_item_view(item, &records_by_key))
                .collect(),
        })
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
        })?;
        Ok(SavedListCreateView {
            list: saved_list_summary(list),
        })
    }

    pub fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> AppServiceResult<SavedListItemMutationView> {
        let record = resolve_record_ref(self, &request.record_ref)?;
        let record_key = record.identity.key.to_string();
        let store = self.local_state_store()?;
        let outcome = store.saved_lists().add_resolved_item(
            &request.slug,
            ResolvedSavedListItem {
                record_key: record.identity.key,
                title_snapshot: record.identity.name,
                kind_snapshot: Some(record.classification.kind.as_str().to_string()),
                note: request.note,
            },
        )?;
        Ok(SavedListItemMutationView {
            slug: request.slug,
            record_key,
            outcome: match outcome {
                AddSavedListItemOutcome::Added => SavedListItemMutationOutcomeView::Added,
                AddSavedListItemOutcome::AlreadyPresent => {
                    SavedListItemMutationOutcomeView::AlreadyPresent
                }
            },
        })
    }

    pub fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> AppServiceResult<SavedListItemMutationView> {
        let record_key = if let Ok(key) = RecordKey::parse(&request.record_ref) {
            key
        } else {
            resolve_record_ref(self, &request.record_ref)?.identity.key
        };
        let store = self.local_state_store()?;
        let removed = store
            .saved_lists()
            .remove_item(&request.slug, &record_key.to_string())?;
        Ok(SavedListItemMutationView {
            slug: request.slug,
            record_key: record_key.to_string(),
            outcome: if removed {
                SavedListItemMutationOutcomeView::Removed
            } else {
                SavedListItemMutationOutcomeView::NotPresent
            },
        })
    }

    pub fn delete_saved_list(&self, slug: &str) -> AppServiceResult<DeleteSavedListView> {
        let store = self.local_state_store()?;
        let deleted = store.saved_lists().delete(slug)?;
        Ok(DeleteSavedListView {
            slug: slug.to_string(),
            deleted,
        })
    }

    fn local_state_store(&self) -> AppServiceResult<LocalStateStore> {
        LocalStateStore::open(self.local_state_path.clone()).map_err(Into::into)
    }
}

fn hydrate_saved_list_records(
    service: &AtlasAppService,
    items: &[SavedListItem],
) -> AppServiceResult<BTreeMap<String, atlas_record::AtlasRecord>> {
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
            .map(|record| (record.identity.key.to_string(), record))
            .collect())
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
                return Err(record_resolution_ambiguous_error(&record_ref, matches));
            }
        };
        let records = retrieval.get_records(GetRecordsRequest {
            record_keys: std::slice::from_ref(&key),
        })?;
        records.into_iter().next().ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::RecordNotFound,
                format!("record not found: {key}"),
            )
        })
    })
}

fn record_resolution_ambiguous_error(
    record_ref: &str,
    matches: Vec<atlas_search::RecordResolutionResult>,
) -> AppServiceError {
    let matches = matches
        .into_iter()
        .map(|resolution| {
            json!({
                "record_key": resolution.record.identity.key.to_string(),
                "title": resolution.record.identity.name,
                "kind": resolution.record.classification.kind.as_str(),
            })
        })
        .collect::<Vec<_>>();
    AppServiceError::from(
        atlas_app_model::AppError::new(
            AppErrorCode::RecordResolutionAmbiguous,
            format!("record resolution ambiguous: {record_ref}"),
        )
        .with_details(json!({
            "record_ref": record_ref,
            "matches": matches,
        })),
    )
}

fn saved_list_summary(list: SavedList) -> SavedListSummaryView {
    SavedListSummaryView {
        slug: list.slug,
        name: list.name,
        description: list.description,
        created_at: list.created_at,
        updated_at: list.updated_at,
    }
}

fn saved_list_item_view(
    item: SavedListItem,
    records_by_key: &BTreeMap<String, atlas_record::AtlasRecord>,
) -> SavedListItemView {
    let record = records_by_key.get(&item.record_key);
    let hydrated = hydrate_saved_list_item(item, record);
    saved_list_item_from_hydrated(hydrated)
}

fn saved_list_item_from_hydrated(
    item: HydratedSavedListItem<&atlas_record::AtlasRecord>,
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
        record: item.record.map(record_summary),
    }
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
        AddSavedListItemRequest, AppErrorCode, CreateSavedListRequest, RemoveSavedListItemRequest,
        SavedListItemMutationOutcomeView, SavedListItemStatusView,
    };
    use atlas_domain::RecordKey;
    use atlas_local_state::{NewSavedList, ResolvedSavedListItem};

    use crate::test_support::fixture_worker;

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
            })
            .expect("list should create");
        store
            .saved_lists()
            .create(NewSavedList {
                slug: "a-first".to_string(),
                name: "A First".to_string(),
                description: Some("Campaign prep".to_string()),
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
        assert_eq!(
            view.items[0]
                .record
                .as_ref()
                .expect("active item should hydrate")
                .title,
            "Test Action 1"
        );
        assert_eq!(view.items[1].status, SavedListItemStatusView::Unresolved);
        assert!(view.items[1].record.is_none());
        assert_eq!(view.items[1].snapshot.title, "Missing Action");
    }

    #[test]
    fn saved_list_reports_missing_slug() {
        let fixture = fixture_worker();

        let error = fixture
            .worker
            .saved_list("missing")
            .expect_err("missing list should fail")
            .into_app_error();

        assert_eq!(error.code, AppErrorCode::SavedListNotFound);
    }

    #[test]
    fn saved_list_reports_invalid_slug() {
        let fixture = fixture_worker();

        let error = fixture
            .worker
            .saved_list("bad slug")
            .expect_err("invalid slug should fail")
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
            })
            .expect("list should create");
        assert_eq!(created.list.slug, "research");
        assert_eq!(created.list.description.as_deref(), Some("Campaign prep"));

        let added = fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                slug: "research".to_string(),
                record_ref: "Test Action 1".to_string(),
                note: Some("important".to_string()),
            })
            .expect("item should add");
        assert_eq!(added.record_key, "actions:testAction1");
        assert_eq!(added.outcome, SavedListItemMutationOutcomeView::Added);

        let duplicate = fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                slug: "research".to_string(),
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
                slug: "research".to_string(),
                record_ref: "Test Action 1".to_string(),
            })
            .expect("item should remove by resolvable name");
        assert_eq!(removed.outcome, SavedListItemMutationOutcomeView::Removed);

        let absent = fixture
            .worker
            .remove_saved_list_item(RemoveSavedListItemRequest {
                slug: "research".to_string(),
                record_ref: "actions:testAction1".to_string(),
            })
            .expect("absent item remove should be a no-op");
        assert_eq!(absent.outcome, SavedListItemMutationOutcomeView::NotPresent);

        let deleted = fixture
            .worker
            .delete_saved_list("research")
            .expect("list should delete");
        assert!(deleted.deleted);
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
            })
            .expect("list should create");

        let error = fixture
            .worker
            .add_saved_list_item(AddSavedListItemRequest {
                slug: "research".to_string(),
                record_ref: "No Such Record".to_string(),
                note: None,
            })
            .expect_err("missing record ref should fail")
            .into_app_error();

        assert_eq!(error.code, AppErrorCode::RecordResolutionMiss);
    }
}
