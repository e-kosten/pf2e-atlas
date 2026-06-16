use std::collections::BTreeMap;

use atlas_app_model::{
    AppErrorCode, SavedListDetailView, SavedListIndexView, SavedListItemSnapshotView,
    SavedListItemStatusView, SavedListItemView, SavedListSummaryView,
};
use atlas_domain::RecordKey;
use atlas_local_state::{
    HydratedSavedListItem, LocalStateStore, SavedList, SavedListItem, SavedListItemStatus,
    hydrate_saved_list_item,
};
use atlas_search::{GetRecordsRequest, RecordRetrieval};

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_summary;
use crate::service::AtlasAppService;

impl AtlasAppService {
    pub fn saved_lists(&self) -> AppServiceResult<SavedListIndexView> {
        let lists = self.local_state_store()?.lists()?;
        Ok(SavedListIndexView {
            lists: lists.into_iter().map(saved_list_summary).collect(),
        })
    }

    pub fn saved_list(&self, slug: &str) -> AppServiceResult<SavedListDetailView> {
        let list = self
            .local_state_store()?
            .get_list_with_items(slug)?
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
    service.retrieval.submit(move |retrieval| {
        Ok(retrieval
            .get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?
            .into_iter()
            .map(|record| (record.identity.key.to_string(), record))
            .collect())
    })
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
    use atlas_app_model::{AppErrorCode, SavedListItemStatusView};
    use atlas_local_state::{NewSavedList, NewSavedListItem};

    use crate::test_support::fixture_worker;

    #[test]
    fn saved_lists_returns_local_state_summaries() {
        let fixture = fixture_worker();
        let store = fixture
            .worker
            .local_state_store()
            .expect("fixture local state should open");
        store
            .create_list(NewSavedList {
                slug: "z-last".to_string(),
                name: "Z Last".to_string(),
                description: None,
            })
            .expect("list should create");
        store
            .create_list(NewSavedList {
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
            .create_list(NewSavedList {
                slug: "research".to_string(),
                name: "Research".to_string(),
                description: None,
            })
            .expect("list should create");
        store
            .add_item(
                "research",
                NewSavedListItem {
                    record_key: "actions:testAction1".to_string(),
                    note: Some("active note".to_string()),
                    record_title_snapshot: "Old Action Name".to_string(),
                    record_kind_snapshot: Some("old-kind".to_string()),
                },
            )
            .expect("active item should insert");
        store
            .add_item(
                "research",
                NewSavedListItem {
                    record_key: "actions:missing".to_string(),
                    note: None,
                    record_title_snapshot: "Missing Action".to_string(),
                    record_kind_snapshot: Some("rule".to_string()),
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
}
