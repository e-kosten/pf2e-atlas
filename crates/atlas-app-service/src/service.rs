use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;

use atlas_app_model::{
    AppErrorCode, AppReadinessStatus, AppReadinessView, DiscoverFilterEditorRequest,
    DiscoverFilterValuesRequest, FilterEditorView, FilterValueListView, OpenResultWindowRequest,
    ReadResultWindowPageRequest, RecordDetailView, RecordListSortView, ResultMatchSummary,
    ResultWindowMode, ResultWindowModeSummary, ResultWindowPage, ResultWindowRow,
};
use atlas_domain::RecordKey;
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{
    AtlasRetrievalService, DiscoverFilterFieldsRequest as SearchDiscoverFilterFieldsRequest,
    DiscoverFilterValuesRequest as SearchDiscoverFilterValuesRequest, FilterDiscoveryRetrieval,
    GetRecordRequest, ListRecordsRequest, MetricDiscoverySelector, RecordListSort, RecordRetrieval,
    SearchPage, TextRetrieval, TextSearchMatch, TextSearchRequest,
};

use crate::discovery::{filter_editor_view, filter_value_list_view};
use crate::error::{AppServiceError, AppServiceResult};
use crate::filter::{discovery_field_id, lower_basic_filter, lower_basic_filter_context};
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
    DiscoverFilterEditor {
        request: DiscoverFilterEditorRequest,
        reply: mpsc::Sender<AppServiceResult<FilterEditorView>>,
    },
    DiscoverFilterValues {
        request: DiscoverFilterValuesRequest,
        reply: mpsc::Sender<AppServiceResult<FilterValueListView>>,
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

    pub fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> AppServiceResult<FilterEditorView> {
        self.call(|reply| AppServiceCommand::DiscoverFilterEditor { request, reply })
    }

    pub fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> AppServiceResult<FilterValueListView> {
        self.call(|reply| AppServiceCommand::DiscoverFilterValues { request, reply })
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
                AppServiceCommand::DiscoverFilterEditor { request, reply } => {
                    let _ = reply.send(self.discover_filter_editor(request));
                }
                AppServiceCommand::DiscoverFilterValues { request, reply } => {
                    let _ = reply.send(self.discover_filter_values(request));
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

    fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> AppServiceResult<FilterEditorView> {
        let filter = lower_basic_filter_context(&request.context)?;
        let discovery =
            self.retrieval
                .discover_filter_fields(SearchDiscoverFilterFieldsRequest {
                    filter: filter.as_ref(),
                    filter_json: None,
                })?;
        Ok(filter_editor_view(discovery))
    }

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> AppServiceResult<FilterValueListView> {
        let filter = lower_basic_filter_context(&request.context)?;
        let discovery =
            self.retrieval
                .discover_filter_values(SearchDiscoverFilterValuesRequest {
                    field: discovery_field_id(&request.field_id),
                    filter: filter.as_ref(),
                    filter_json: None,
                    sort: None,
                    sample_limit: None,
                    metric_selector: metric_selector(request.metric_query.as_deref()),
                    metric_domain: request.metric_domain.clone(),
                })?;
        filter_value_list_view(&request.field_id, &request.context, discovery)
    }
}

fn metric_selector(query: Option<&str>) -> Option<MetricDiscoverySelector> {
    let query = query?.trim();
    if query.is_empty() {
        None
    } else {
        Some(MetricDiscoverySelector::Query(query.to_string()))
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
    use atlas_app_model::{
        BasicSearchFilter, FilterClause, FilterClauseOperator, FilterDiscoveryContext,
        OpenResultWindowRequest, ReadResultWindowPageRequest, RecordListSortView, ResultWindowMode,
        SearchPageRequest,
    };
    use atlas_search::test_support::{
        FixtureArtifact, minimal_fixture_retrieval_service_without_embeddings,
    };

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

    #[test]
    fn app_service_start_fails_when_artifact_is_unavailable() {
        let missing_path = std::env::temp_dir().join(format!(
            "atlas-app-service-missing-{}-{}.sqlite",
            std::process::id(),
            unique_suffix()
        ));
        let result = AtlasAppService::start(AtlasAppServiceOptions {
            path_mode: AtlasPathMode::Global,
            source_root: None,
            embedding_cache_root: None,
            index_path: Some(missing_path),
        });

        let error = match result {
            Ok(_) => panic!(
                "web app service startup should fail instead of using no-embeddings fallback"
            ),
            Err(error) => error.into_app_error(),
        };
        assert_eq!(error.code, AppErrorCode::IndexUnavailable);
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

    #[test]
    fn worker_record_detail_reports_valid_invalid_and_missing_keys() {
        let fixture = fixture_worker();
        let worker = &fixture.worker;

        let detail = worker
            .record_detail("actions:testAction1")
            .expect("fixture record should load");
        assert_eq!(detail.record_key, "actions:testAction1");
        assert_eq!(detail.title, "Test Action 1");
        assert_eq!(detail.kind, "rule");
        assert_eq!(detail.presentation.title, "Test Action 1");

        let invalid = worker
            .record_detail("not a key")
            .expect_err("invalid keys should be rejected")
            .into_app_error();
        assert_eq!(invalid.code, AppErrorCode::InvalidRecordKey);

        let missing = worker
            .record_detail("actions:missing")
            .expect_err("missing keys should return not found")
            .into_app_error();
        assert_eq!(missing.code, AppErrorCode::RecordNotFound);
    }

    #[test]
    fn worker_discovers_app_facing_filter_fields_and_values() {
        let fixture = fixture_worker();
        let worker = &fixture.worker;
        let context = FilterDiscoveryContext::Filtered {
            filter: BasicSearchFilter {
                clauses: vec![FilterClause {
                    id: "kind-include_any".to_string(),
                    field: "kind".to_string(),
                    operator: FilterClauseOperator::IncludeAny,
                    values: vec!["rule".to_string()],
                    range: None,
                    metric: None,
                }],
            },
        };

        let editor = worker
            .discover_filter_editor(atlas_app_model::DiscoverFilterEditorRequest {
                context: context.clone(),
            })
            .expect("fixture editor discovery should succeed");

        assert_eq!(editor.matching_record_count, 3);
        let fields = editor
            .groups
            .iter()
            .flat_map(|group| group.fields.iter())
            .collect::<Vec<_>>();
        assert!(fields.iter().any(|field| field.id == "kind"));
        assert!(fields.iter().any(|field| field.id == "pack"));
        assert!(!fields.iter().any(|field| field.id == "pack_label"));

        let values = worker
            .discover_filter_values(atlas_app_model::DiscoverFilterValuesRequest {
                context,
                field_id: "pack".to_string(),
                metric_query: None,
                metric_domain: None,
            })
            .expect("fixture value discovery should succeed");

        assert_eq!(values.field_id, "pack");
        assert_eq!(values.matching_record_count, 3);
        assert_eq!(values.options[0].value, "Actions");
        assert_eq!(values.options[0].count, Some(3));
        assert!(!values.options[0].disabled);
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

    struct FixtureWorker {
        worker: AppServiceWorker,
        _artifact: FixtureArtifact,
    }

    fn fixture_worker() -> FixtureWorker {
        let (retrieval, artifact) = minimal_fixture_retrieval_service_without_embeddings()
            .expect("fixture retrieval service should build");
        FixtureWorker {
            worker: AppServiceWorker {
                retrieval,
                windows: ResultWindowStore::new(MAX_RESULT_WINDOWS),
                next_window_id: AtomicU64::new(1),
            },
            _artifact: artifact,
        }
    }

    fn unique_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    }
}
