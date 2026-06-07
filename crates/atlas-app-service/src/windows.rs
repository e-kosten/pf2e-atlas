use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::Ordering;

use atlas_app_model::{
    AppErrorCode, OpenResultWindowRequest, ReadResultWindowPageRequest, RecordListSortView,
    ResultMatchSummary, ResultWindowMode, ResultWindowModeSummary, ResultWindowPage,
    ResultWindowRow,
};
use atlas_search::{
    AtlasRetrievalService, ListRecordsRequest, RecordListSort, RecordRetrieval, SearchPage,
    TextRetrieval, TextSearchMatch, TextSearchRequest,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::filter::lower_basic_filter;
use crate::projection::{record_summary, search_page_view, text_match_summary};
use crate::service::AppServiceWorker;

pub(super) const MAX_RESULT_WINDOWS: usize = 64;
const MAX_EXPIRED_RESULT_WINDOWS: usize = MAX_RESULT_WINDOWS;

#[derive(Debug, Clone)]
struct StoredResultWindow {
    mode: ResultWindowMode,
    include_diagnostics: bool,
}

#[derive(Debug)]
pub(super) struct ResultWindowStore {
    windows: BTreeMap<u64, StoredResultWindow>,
    order: VecDeque<u64>,
    expired: BTreeSet<u64>,
    expired_order: VecDeque<u64>,
    capacity: usize,
    expired_capacity: usize,
}

impl AppServiceWorker {
    pub(super) fn open_result_window(
        &mut self,
        request: OpenResultWindowRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        let window_id = self.next_window_id.fetch_add(1, Ordering::Relaxed);
        let window = StoredResultWindow {
            mode: request.mode,
            include_diagnostics: request.include_diagnostics,
        };
        let retrieval = &mut self.retrieval;
        insert_after_success(&mut self.windows, window_id, window, |window| {
            render_result_window_page(
                retrieval,
                window_id,
                window,
                ReadResultWindowPageRequest { page: request.page },
            )
        })
    }

    pub(super) fn read_result_window_page(
        &mut self,
        window_id: u64,
        request: ReadResultWindowPageRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        let window = self.windows.get(window_id)?;
        self.render_result_window_page(window_id, &window, request)
    }

    fn render_result_window_page(
        &mut self,
        window_id: u64,
        window: &StoredResultWindow,
        request: ReadResultWindowPageRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        render_result_window_page(&mut self.retrieval, window_id, window, request)
    }
}

impl ResultWindowStore {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            windows: BTreeMap::new(),
            order: VecDeque::new(),
            expired: BTreeSet::new(),
            expired_order: VecDeque::new(),
            capacity: capacity.max(1),
            expired_capacity: capacity.clamp(1, MAX_EXPIRED_RESULT_WINDOWS),
        }
    }

    fn insert(&mut self, id: u64, window: StoredResultWindow) {
        if !self.windows.contains_key(&id) {
            self.order.push_back(id);
        }
        self.windows.insert(id, window);
        while self.windows.len() > self.capacity {
            let Some(expired_id) = self.order.pop_front() else {
                break;
            };
            if self.windows.remove(&expired_id).is_some() {
                self.insert_expired(expired_id);
            }
        }
    }

    fn insert_expired(&mut self, id: u64) {
        if self.expired.insert(id) {
            self.expired_order.push_back(id);
        }
        while self.expired.len() > self.expired_capacity {
            let Some(oldest_id) = self.expired_order.pop_front() else {
                break;
            };
            self.expired.remove(&oldest_id);
        }
    }

    fn get(&self, id: u64) -> AppServiceResult<StoredResultWindow> {
        self.windows.get(&id).cloned().ok_or_else(|| {
            if self.expired.contains(&id) {
                AppServiceError::new(
                    AppErrorCode::WindowExpired,
                    format!("result window `{id}` has expired"),
                )
            } else {
                AppServiceError::new(
                    AppErrorCode::WindowNotFound,
                    format!("result window `{id}` was not found"),
                )
            }
        })
    }
}

fn insert_after_success<T>(
    store: &mut ResultWindowStore,
    window_id: u64,
    window: StoredResultWindow,
    render: impl FnOnce(&StoredResultWindow) -> AppServiceResult<T>,
) -> AppServiceResult<T> {
    let result = render(&window)?;
    store.insert(window_id, window);
    Ok(result)
}

fn render_result_window_page(
    retrieval: &mut AtlasRetrievalService,
    window_id: u64,
    window: &StoredResultWindow,
    request: ReadResultWindowPageRequest,
) -> AppServiceResult<ResultWindowPage> {
    let page = SearchPage::new(request.page.number, request.page.size)?;
    let filter = match &window.mode {
        ResultWindowMode::ListRecords { filter, .. }
        | ResultWindowMode::TextSearch { filter, .. } => lower_basic_filter(filter.as_ref())?,
    };
    let filter_ref = filter.as_ref();

    match &window.mode {
        ResultWindowMode::ListRecords { sort, .. } => {
            let result = retrieval.list_records(
                ListRecordsRequest::new(filter_ref, page).with_sort(record_list_sort(*sort)),
            )?;
            Ok(ResultWindowPage {
                window_id,
                mode: ResultWindowModeSummary::ListRecords,
                page: search_page_view(result.page),
                rows: result
                    .records
                    .iter()
                    .map(|record| ResultWindowRow {
                        record: record_summary(record),
                        match_summary: None,
                    })
                    .collect(),
            })
        }
        ResultWindowMode::TextSearch { query, exclude, .. } => {
            let result = retrieval.search_text(TextSearchRequest {
                query,
                exclude: exclude.as_deref(),
                filter: filter_ref,
                page,
                tuning: None,
                explain: window.include_diagnostics,
            })?;
            Ok(ResultWindowPage {
                window_id,
                mode: ResultWindowModeSummary::TextSearch {
                    query: query.clone(),
                },
                page: search_page_view(result.page),
                rows: result
                    .records
                    .iter()
                    .map(|record| ResultWindowRow {
                        record: record_summary(&record.record),
                        match_summary: Some(match_summary(&record.match_info)),
                    })
                    .collect(),
            })
        }
    }
}

fn record_list_sort(value: RecordListSortView) -> RecordListSort {
    match value {
        RecordListSortView::Alphabetical => RecordListSort::Alphabetical,
        RecordListSortView::LevelAsc => RecordListSort::LevelAsc,
        RecordListSortView::LevelDesc => RecordListSort::LevelDesc,
        RecordListSortView::PriceAsc => RecordListSort::PriceAsc,
        RecordListSortView::PriceDesc => RecordListSort::PriceDesc,
        RecordListSortView::RecordKey => RecordListSort::RecordKey,
        RecordListSortView::Random { seed } => RecordListSort::Random { seed },
    }
}

fn match_summary(match_info: &TextSearchMatch) -> ResultMatchSummary {
    match match_info {
        TextSearchMatch::Identity {
            identity_match_kind,
            ..
        } => text_match_summary(format!("identity: {}", identity_match_kind.as_str())),
        TextSearchMatch::Ranked { retrieval, .. } => {
            text_match_summary(format!("ranked: {}", retrieval.as_str()))
        }
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        BasicSearchFilter, OpenResultWindowRequest, ReadResultWindowPageRequest,
        RecordListSortView, ResultWindowMode, SearchPageRequest,
    };

    use super::*;
    use crate::test_support::fixture_worker;

    #[test]
    fn result_window_store_expires_oldest_window_when_capacity_is_exceeded() {
        let mut store = ResultWindowStore::new(2);
        store.insert(1, stored_window("first"));
        store.insert(2, stored_window("second"));
        store.insert(3, stored_window("third"));

        assert_eq!(
            store
                .get(1)
                .expect_err("first window should expire")
                .into_app_error()
                .code,
            AppErrorCode::WindowExpired
        );
        assert!(matches!(
            store.get(2).expect("second window should remain").mode,
            ResultWindowMode::ListRecords { .. }
        ));
        assert!(matches!(
            store.get(3).expect("third window should remain").mode,
            ResultWindowMode::ListRecords { .. }
        ));
    }

    #[test]
    fn result_window_store_reports_unknown_window_as_not_found() {
        let store = ResultWindowStore::new(2);

        assert_eq!(
            store
                .get(99)
                .expect_err("unknown window should be missing")
                .into_app_error()
                .code,
            AppErrorCode::WindowNotFound
        );
    }

    #[test]
    fn result_window_store_bounds_expired_tombstones() {
        let mut store = ResultWindowStore::new(2);

        for id in 1..=10 {
            store.insert(id, stored_window("window"));
        }

        assert_eq!(store.windows.len(), 2);
        assert_eq!(store.expired.len(), 2);
        assert_eq!(
            store
                .get(1)
                .expect_err("old tombstone should be pruned")
                .into_app_error()
                .code,
            AppErrorCode::WindowNotFound
        );
        assert_eq!(
            store
                .get(8)
                .expect_err("recent tombstone should remain expired")
                .into_app_error()
                .code,
            AppErrorCode::WindowExpired
        );
        assert!(store.get(9).is_ok());
        assert!(store.get(10).is_ok());
    }

    #[test]
    fn failed_result_window_open_does_not_retain_hidden_window() {
        let mut store = ResultWindowStore::new(2);
        let window_id = 7;
        let result = insert_after_success(&mut store, window_id, stored_window("failed"), |_| {
            Err::<(), _>(AppServiceError::invalid_request("invalid first page"))
        });

        assert!(result.is_err());
        assert_eq!(store.windows.len(), 0);
        assert_eq!(
            store
                .get(window_id)
                .expect_err("failed open should not retain a hidden window")
                .into_app_error()
                .code,
            AppErrorCode::WindowNotFound
        );
    }

    #[test]
    fn successful_result_window_open_inserts_after_render() {
        let mut store = ResultWindowStore::new(2);
        let window_id = 7;

        let result = insert_after_success(&mut store, window_id, stored_window("success"), |_| {
            Ok::<_, AppServiceError>("page")
        })
        .expect("successful render should insert window");

        assert_eq!(result, "page");
        assert!(store.get(window_id).is_ok());
    }

    #[test]
    fn worker_opens_and_reads_list_result_window_pages() {
        let mut fixture = fixture_worker();
        let worker = &mut fixture.worker;
        let first_page = worker
            .open_result_window(OpenResultWindowRequest {
                mode: ResultWindowMode::ListRecords {
                    filter: None,
                    sort: RecordListSortView::RecordKey,
                },
                page: SearchPageRequest { number: 1, size: 2 },
                include_diagnostics: false,
            })
            .expect("fixture list window should open");

        assert_eq!(first_page.window_id, 1);
        assert_eq!(first_page.page.number, 1);
        assert_eq!(first_page.page.count, 2);
        assert_eq!(first_page.page.total, 3);
        assert_eq!(first_page.rows[0].record.record_key, "actions:testAction1");
        assert_eq!(first_page.rows[0].record.pack.as_deref(), Some("Actions"));

        let second_page = worker
            .read_result_window_page(
                first_page.window_id,
                ReadResultWindowPageRequest {
                    page: SearchPageRequest { number: 2, size: 2 },
                },
            )
            .expect("stored fixture window should read later page");

        assert_eq!(second_page.page.number, 2);
        assert_eq!(second_page.page.count, 1);
        assert_eq!(second_page.rows[0].record.record_key, "actions:testAction3");
    }

    fn stored_window(id: &str) -> StoredResultWindow {
        StoredResultWindow {
            mode: ResultWindowMode::ListRecords {
                filter: Some(BasicSearchFilter {
                    clauses: Vec::new(),
                }),
                sort: atlas_app_model::RecordListSortView::RecordKey,
            },
            include_diagnostics: id == "first",
        }
    }
}
