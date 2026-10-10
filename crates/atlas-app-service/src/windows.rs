use crate::projection::{
    coverage_view, localized_summary_view, search_page_view, search_witness_view,
};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AppErrorCode, OpenResultWindowRequest, ReadResultWindowPageRequest, ResultWindowMode,
    ResultWindowModeSummary, ResultWindowPage, ResultWindowRow, RetrievalModeView,
};
use atlas_search::{
    AtlasRetrievalService, ListRecordsRequest, RetrievalMode, SearchPage, TextSearchRequest,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::Ordering;
pub(super) const MAX_RESULT_WINDOWS: usize = 64;
const MAX_EXPIRED_RESULT_WINDOWS: usize = MAX_RESULT_WINDOWS;
#[derive(Debug, Clone)]
struct StoredResultWindow {
    mode: ResultWindowMode,
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
impl AtlasAppService {
    pub fn open_result_window(
        &self,
        request: OpenResultWindowRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        let id = self.next_window_id.fetch_add(1, Ordering::Relaxed);
        let window = StoredResultWindow { mode: request.mode };
        let render = window.clone();
        let page = self.submit_retrieval(move |r| {
            render_result_window_page(
                r,
                id,
                &render,
                ReadResultWindowPageRequest { page: request.page },
            )
        })?;
        self.windows()?.insert(id, window);
        Ok(page)
    }
    pub fn read_result_window_page(
        &self,
        id: u64,
        request: ReadResultWindowPageRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        let window = self.windows()?.get(id)?;
        self.submit_retrieval(move |r| render_result_window_page(r, id, &window, request))
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

fn render_result_window_page(
    retrieval: &mut AtlasRetrievalService,
    id: u64,
    window: &StoredResultWindow,
    request: ReadResultWindowPageRequest,
) -> AppServiceResult<ResultWindowPage> {
    let page = SearchPage::new(request.page.number, request.page.size)?;
    match &window.mode {
        ResultWindowMode::ListRecords { filter } => {
            let result = retrieval.list_records(ListRecordsRequest::new(filter.as_ref(), page))?;
            Ok(ResultWindowPage {
                window_id: id,
                mode: ResultWindowModeSummary::ListRecords,
                page: search_page_view(result.page),
                rows: result
                    .records
                    .iter()
                    .map(|r| ResultWindowRow {
                        record: localized_summary_view(r, retrieval),
                        matches: vec![],
                    })
                    .collect(),
                coverage: None,
            })
        }
        ResultWindowMode::TextSearch {
            query,
            filter,
            mode,
        } => {
            let result = retrieval.search_text(TextSearchRequest {
                query,
                mode: retrieval_mode(*mode),
                filter: filter.as_ref(),
                scope: atlas_search::RecordScope::All,
                page,
                prefer_remaster: true,
            })?;
            Ok(ResultWindowPage {
                window_id: id,
                mode: ResultWindowModeSummary::TextSearch {
                    query: query.clone(),
                    mode: *mode,
                },
                page: search_page_view(result.page),
                rows: result
                    .records
                    .into_iter()
                    .map(|r| ResultWindowRow {
                        record: localized_summary_view(&r.record, retrieval),
                        matches: r
                            .matches
                            .into_iter()
                            .map(|w| {
                                search_witness_view(
                                    w,
                                    &retrieval.artifact_context().source_fingerprint,
                                )
                            })
                            .collect(),
                    })
                    .collect(),
                coverage: Some(coverage_view(result.coverage)),
            })
        }
    }
}
pub(crate) fn retrieval_mode(mode: RetrievalModeView) -> RetrievalMode {
    match mode {
        RetrievalModeView::Lexical => RetrievalMode::Fts,
        RetrievalModeView::Semantic => RetrievalMode::Vector,
        RetrievalModeView::Hybrid => RetrievalMode::Hybrid,
    }
}
