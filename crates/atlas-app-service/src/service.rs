use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;

use atlas_app_model::{
    AppErrorCode, AppReadinessStatus, AppReadinessView, OpenResultWindowRequest,
    ReadResultWindowPageRequest, RecordDetailView, RecordListSortView, ResultMatchSummary,
    ResultWindowMode, ResultWindowModeSummary, ResultWindowPage, ResultWindowRow,
};
use atlas_domain::RecordKey;
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{
    AtlasRetrievalService, GetRecordRequest, ListRecordsRequest, RecordListSort, RecordRetrieval,
    SearchPage, TextRetrieval, TextSearchMatch, TextSearchRequest,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::filter::lower_basic_filter;
use crate::projection::{record_detail, record_summary, search_page_view, text_match_summary};

const MAX_RESULT_WINDOWS: usize = 64;
const MAX_EXPIRED_RESULT_WINDOWS: usize = MAX_RESULT_WINDOWS;

#[derive(Clone)]
pub struct AtlasAppService {
    sender: mpsc::Sender<AppServiceCommand>,
}

#[derive(Debug, Clone)]
pub struct AtlasAppServiceOptions {
    pub path_mode: AtlasPathMode,
    pub source_root: Option<PathBuf>,
    pub embedding_cache_root: Option<PathBuf>,
    pub index_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct StoredResultWindow {
    mode: ResultWindowMode,
    include_diagnostics: bool,
}

struct AppServiceWorker {
    retrieval: AtlasRetrievalService,
    windows: ResultWindowStore,
    next_window_id: AtomicU64,
}

#[derive(Debug)]
struct ResultWindowStore {
    windows: BTreeMap<u64, StoredResultWindow>,
    order: VecDeque<u64>,
    expired: BTreeSet<u64>,
    expired_order: VecDeque<u64>,
    capacity: usize,
    expired_capacity: usize,
}

enum AppServiceCommand {
    Readiness {
        reply: mpsc::Sender<AppServiceResult<AppReadinessView>>,
    },
    OpenResultWindow {
        request: OpenResultWindowRequest,
        reply: mpsc::Sender<AppServiceResult<ResultWindowPage>>,
    },
    ReadResultWindowPage {
        window_id: u64,
        request: ReadResultWindowPageRequest,
        reply: mpsc::Sender<AppServiceResult<ResultWindowPage>>,
    },
    RecordDetail {
        record_key: String,
        reply: mpsc::Sender<AppServiceResult<RecordDetailView>>,
    },
}

impl AtlasAppService {
    pub fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::channel();
        thread::Builder::new()
            .name("atlas-app-service".to_string())
            .spawn(move || {
                let startup = AppServiceWorker::start(options);
                match startup {
                    Ok(mut worker) => {
                        let _ = startup_sender.send(Ok(()));
                        worker.run(receiver);
                    }
                    Err(error) => {
                        let _ = startup_sender.send(Err(error));
                    }
                }
            })
            .map_err(|error| {
                AppServiceError::new(
                    AppErrorCode::InternalError,
                    format!("failed to start app-service worker: {error}"),
                )
            })?;

        startup_receiver.recv().map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("app-service worker failed to report startup: {error}"),
            )
        })??;

        Ok(Self { sender })
    }

    pub fn readiness(&self) -> AppReadinessView {
        match self.call(|reply| AppServiceCommand::Readiness { reply }) {
            Ok(view) => view,
            Err(error) => AppReadinessView {
                status: AppReadinessStatus::Blocked,
                message: error.into_app_error().message,
            },
        }
    }

    pub fn open_result_window(
        &self,
        request: OpenResultWindowRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        self.call(|reply| AppServiceCommand::OpenResultWindow { request, reply })
    }

    pub fn read_result_window_page(
        &self,
        window_id: u64,
        request: ReadResultWindowPageRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        self.call(|reply| AppServiceCommand::ReadResultWindowPage {
            window_id,
            request,
            reply,
        })
    }

    pub fn record_detail(&self, record_key: &str) -> AppServiceResult<RecordDetailView> {
        self.call(|reply| AppServiceCommand::RecordDetail {
            record_key: record_key.to_string(),
            reply,
        })
    }

    fn call<T>(
        &self,
        build: impl FnOnce(mpsc::Sender<AppServiceResult<T>>) -> AppServiceCommand,
    ) -> AppServiceResult<T> {
        let (reply, receiver) = mpsc::channel();
        self.sender.send(build(reply)).map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("app-service worker is unavailable: {error}"),
            )
        })?;
        receiver.recv().map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("app-service worker dropped response: {error}"),
            )
        })?
    }
}

impl AppServiceWorker {
    fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
            path_mode: options.path_mode,
            overrides: AtlasPathOverrides {
                source_root: options.source_root,
                embedding_cache_root: options.embedding_cache_root,
                index_path: options.index_path,
            },
        })?;
        let retrieval = runtime.open_retrieval_service()?;
        Ok(Self {
            retrieval,
            windows: ResultWindowStore::new(MAX_RESULT_WINDOWS),
            next_window_id: AtomicU64::new(1),
        })
    }

    fn run(&mut self, receiver: mpsc::Receiver<AppServiceCommand>) {
        for command in receiver {
            match command {
                AppServiceCommand::Readiness { reply } => {
                    let _ = reply.send(Ok(self.readiness()));
                }
                AppServiceCommand::OpenResultWindow { request, reply } => {
                    let _ = reply.send(self.open_result_window(request));
                }
                AppServiceCommand::ReadResultWindowPage {
                    window_id,
                    request,
                    reply,
                } => {
                    let _ = reply.send(self.read_result_window_page(window_id, request));
                }
                AppServiceCommand::RecordDetail { record_key, reply } => {
                    let _ = reply.send(self.record_detail(&record_key));
                }
            }
        }
    }

    fn readiness(&self) -> AppReadinessView {
        AppReadinessView {
            status: AppReadinessStatus::Ready,
            message: "Atlas web service is ready".to_string(),
        }
    }

    fn open_result_window(
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

    fn read_result_window_page(
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

    fn record_detail(&self, record_key: &str) -> AppServiceResult<RecordDetailView> {
        let record_key = RecordKey::parse(record_key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        let record = self
            .retrieval
            .get_record(GetRecordRequest {
                record_key: &record_key,
            })?
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::RecordNotFound,
                    format!("record `{record_key}` was not found"),
                )
            })?;
        record_detail(&record)
    }
}

impl Default for AtlasAppServiceOptions {
    fn default() -> Self {
        Self {
            path_mode: AtlasPathMode::Global,
            source_root: None,
            embedding_cache_root: None,
            index_path: None,
        }
    }
}

impl ResultWindowStore {
    fn new(capacity: usize) -> Self {
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
    use atlas_app_model::{BasicSearchFilter, ResultWindowMode};

    use super::*;

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
