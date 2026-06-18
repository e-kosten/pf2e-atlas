#![deny(unsafe_code)]

mod assets;
mod error;
mod handlers;
mod router;
mod service;

pub use router::router;

#[cfg(test)]
use assets::WebAssets;
#[cfg(test)]
use error::{parse_window_id, status_for_error};
#[cfg(test)]
use router::router_with_state;
#[cfg(test)]
use service::{AtlasWebService, AtlasWebState, call_service};

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        AddSavedListItemRequest, AppError, AppErrorCode, AppReadinessStatus, AppReadinessView,
        CreateSavedListRequest, DeleteSavedListView, DiscoverFilterEditorRequest,
        DiscoverFilterValuesRequest, FilterControlView, FilterEditorFieldView,
        FilterEditorGroupView, FilterEditorView, FilterFieldPlacement, FilterSavedListRequest,
        FilterValueListView, FilterValueOption, OpenResultWindowRequest,
        ReadResultWindowPageRequest, RecordDetailView, RecordSummaryView,
        RemoveSavedListItemRequest, ResultWindowModeSummary, ResultWindowPage, SavedListCreateView,
        SavedListDetailView, SavedListIndexView, SavedListItemMutationView,
        SavedListItemSnapshotView, SavedListItemStatusView, SavedListItemView,
        SavedListSummaryView, SavedListUpdateView, SearchPageView, UpdateSavedListRequest,
    };
    use atlas_app_service::AppServiceError;
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::RecordPresentationDocument;
    use axum::Router;
    use axum::body::Body;
    use axum::body::to_bytes;
    use axum::http::{Method, Request, StatusCode, header};
    use axum::response::{IntoResponse, Response};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn app_error_codes_map_to_expected_http_statuses() {
        assert_eq!(
            status_for_error(AppErrorCode::InvalidRequest),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(AppErrorCode::RecordResolutionMiss),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(AppErrorCode::RecordResolutionAmbiguous),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(AppErrorCode::WindowNotFound),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_for_error(AppErrorCode::SavedListNotFound),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_for_error(AppErrorCode::WindowExpired),
            StatusCode::GONE
        );
        assert_eq!(
            status_for_error(AppErrorCode::SavedListAlreadyExists),
            StatusCode::CONFLICT
        );
        assert_eq!(
            status_for_error(AppErrorCode::ArtifactNotReady),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            status_for_error(AppErrorCode::VectorReadinessRequired),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            status_for_error(AppErrorCode::ServiceBusy),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            status_for_error(AppErrorCode::InternalError),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            status_for_error(AppErrorCode::QueryFailed),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn invalid_window_id_maps_to_app_error() {
        let error = parse_window_id("not-a-number")
            .expect_err("invalid path segment should map into app error")
            .0;

        assert_eq!(error.code, AppErrorCode::InvalidRequest);
        assert!(error.message.contains("invalid result window id"));
    }

    #[tokio::test]
    async fn call_service_preserves_successful_result() {
        let value = call_service(AtlasWebState::from_service(MockService), || {
            Ok::<_, AppServiceError>("ready")
        })
        .await
        .expect("successful app-service call should pass through");

        assert_eq!(value, "ready");
    }

    #[tokio::test]
    async fn call_service_maps_service_error_to_http_envelope() {
        let error = call_service(AtlasWebState::from_service(MockService), || {
            Err::<(), _>(AppServiceError::new(AppErrorCode::WindowExpired, "expired"))
        })
        .await
        .expect_err("service errors should map to web errors");

        let response = error.into_response();
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("error body should be readable");
        let app_error: AppError =
            serde_json::from_slice(&body).expect("error body should be AppError JSON");

        assert_eq!(parts.status, StatusCode::GONE);
        assert_eq!(app_error.code, AppErrorCode::WindowExpired);
        assert_eq!(app_error.message, "expired");
    }

    #[tokio::test]
    async fn call_service_maps_panic_to_internal_error() {
        let error = call_service(
            AtlasWebState::from_service(MockService),
            || -> Result<(), AppServiceError> {
                panic!("simulated app-service panic");
            },
        )
        .await
        .expect_err("join errors should map to web errors");

        let response = error.into_response();
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("error body should be readable");
        let app_error: AppError =
            serde_json::from_slice(&body).expect("error body should be AppError JSON");

        assert_eq!(parts.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(app_error.code, AppErrorCode::InternalError);
    }

    #[tokio::test]
    async fn routes_return_service_busy_when_web_backpressure_is_full() {
        let app = router_with_state(AtlasWebState::from_service_with_blocking_capacity(
            MockService,
            0,
        ));
        let response = app
            .oneshot(
                Request::get("/api/records/actions:testAction1")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (status, body) = response_json(response).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["code"], "service_busy");
        assert_eq!(body["retryable"], true);
        assert!(
            body["message"]
                .as_str()
                .expect("message should be string")
                .contains("blocking service call limit is full")
        );
    }

    #[tokio::test]
    async fn readiness_route_returns_service_readiness() {
        let (status, body) = route_json(Method::GET, "/api/readiness", None).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ready");
        assert_eq!(body["message"], "fixture ready");
    }

    #[tokio::test]
    async fn root_serves_embedded_frontend_index() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::get("/")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("body should be readable");
        let body = String::from_utf8(body.to_vec()).expect("body should be UTF-8");

        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(
            parts.headers.get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            parts.headers.get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );
        assert!(body.contains("<div id=\"root\"></div>"));
    }

    #[tokio::test]
    async fn frontend_routes_fall_back_to_index_but_api_routes_do_not() {
        let app = test_router();
        let response = app
            .clone()
            .oneshot(
                Request::get("/search")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("body should be readable");
        let body = String::from_utf8(body.to_vec()).expect("body should be UTF-8");

        assert_eq!(parts.status, StatusCode::OK);
        assert!(body.contains("<div id=\"root\"></div>"));

        let response = app
            .oneshot(
                Request::get("/api/no-such-route")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn embedded_frontend_assets_get_long_cache_headers() {
        let asset_path = WebAssets::iter()
            .find(|path| path.starts_with("assets/") && path.ends_with(".js"))
            .expect("built frontend should include a JavaScript asset");
        let app = test_router();
        let response = app
            .oneshot(
                Request::get(format!("/{asset_path}"))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/javascript; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn result_window_routes_return_success_and_service_errors() {
        let open_body = json!({
            "mode": {
                "kind": "list_records",
                "filter": { "clauses": [] },
                "sort": { "kind": "record_key" }
            },
            "page": { "number": 1, "size": 25 },
            "include_diagnostics": false
        });
        let (status, body) = route_json(Method::POST, "/api/result-windows", Some(open_body)).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["window_id"], 42);
        assert_eq!(body["mode"]["kind"], "list_records");

        let page_body = json!({ "page": { "number": 2, "size": 25 } });
        let (status, body) = route_json(
            Method::POST,
            "/api/result-windows/42/page",
            Some(page_body.clone()),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["page"]["number"], 2);

        let (status, body) = route_json(
            Method::POST,
            "/api/result-windows/not-a-number/page",
            Some(page_body.clone()),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_request");
        assert!(
            body["message"]
                .as_str()
                .expect("message should be string")
                .contains("invalid result window id")
        );

        let (status, body) = route_json(
            Method::POST,
            "/api/result-windows/999/page",
            Some(page_body),
        )
        .await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "window_not_found");
    }

    #[tokio::test]
    async fn record_and_filter_routes_use_real_router_wiring() {
        let (status, body) =
            route_json(Method::GET, "/api/records/actions:testAction1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["record_key"], "actions:testAction1");
        assert_eq!(body["presentation"]["title"], "Test Action 1");

        let editor_request = json!({
            "context": { "kind": "filtered", "filter": { "clauses": [] } }
        });
        let (status, body) =
            route_json(Method::POST, "/api/filters/editor", Some(editor_request)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["groups"][0]["id"], "standard");
        assert_eq!(body["groups"][0]["fields"][0]["id"], "kind");

        let values_request = json!({
            "context": { "kind": "filtered", "filter": { "clauses": [] } },
            "field_id": "pack"
        });
        let (status, body) =
            route_json(Method::POST, "/api/filters/values", Some(values_request)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["field_id"], "pack");
        assert_eq!(body["options"][0]["label"], "Actions");
    }

    #[tokio::test]
    async fn saved_list_routes_use_real_router_wiring() {
        let (status, body) = route_json(Method::GET, "/api/lists", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["lists"][0]["list_key"], "list_research");
        assert_eq!(body["lists"][0]["slug"], "research");

        let (status, body) = route_json(
            Method::POST,
            "/api/lists",
            Some(json!({
                "slug": "encounters",
                "name": "Encounters",
                "description": "Session prep"
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list"]["list_key"], "list_created");
        assert_eq!(body["list"]["slug"], "encounters");

        let (status, body) = route_json(Method::GET, "/api/lists/list_research", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list"]["slug"], "research");
        assert_eq!(body["items"][0]["record_key"], "actions:testAction1");
        assert_eq!(body["items"][0]["status"], "active");

        let (status, body) = route_json(
            Method::POST,
            "/api/lists/list_research/filter",
            Some(json!({
                "list_ref": "ignored",
                "filter": {
                    "clauses": [
                        {
                            "id": "kind-include_any",
                            "field": "kind",
                            "operator": "include_any",
                            "values": ["action"]
                        }
                    ]
                }
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list"]["list_key"], "list_research");
        assert_eq!(body["items"][0]["record_key"], "actions:testAction1");

        let (status, body) = route_json(
            Method::PATCH,
            "/api/lists/list_research",
            Some(json!({
                "list_key": "ignored",
                "slug": "renamed-research",
                "name": "Renamed Research",
                "description": "Updated prep"
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list"]["list_key"], "list_research");
        assert_eq!(body["list"]["slug"], "renamed-research");
        assert_eq!(body["list"]["name"], "Renamed Research");
        assert_eq!(body["list"]["description"], "Updated prep");

        let (status, body) = route_json(
            Method::POST,
            "/api/lists/list_research/items",
            Some(json!({
                "list_ref": "ignored",
                "record_ref": "actions:testAction2",
                "note": null
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list_key"], "list_research");
        assert_eq!(body["slug"], "research");
        assert_eq!(body["record_key"], "actions:testAction2");
        assert_eq!(body["outcome"], "added");

        let (status, body) = route_json(
            Method::DELETE,
            "/api/lists/list_research/items",
            Some(json!({
                "list_ref": "ignored",
                "record_ref": "actions:testAction1"
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list_key"], "list_research");
        assert_eq!(body["slug"], "research");
        assert_eq!(body["record_key"], "actions:testAction1");
        assert_eq!(body["outcome"], "removed");

        let (status, body) = route_json(Method::DELETE, "/api/lists/list_research", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list_key"], "list_research");
        assert_eq!(body["slug"], "research");
        assert_eq!(body["deleted"], true);

        let (status, body) = route_json(Method::GET, "/api/lists/missing", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "saved_list_not_found");
    }

    #[tokio::test]
    async fn malformed_json_route_body_returns_app_error_envelope() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::post("/api/filters/editor")
                    .header("content-type", "application/json")
                    .body(Body::from("{"))
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (status, body) = response_json(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_request");
        assert!(
            body["message"]
                .as_str()
                .expect("message should be string")
                .contains("invalid JSON request body")
        );
    }

    async fn route_json(method: Method, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let app = test_router();
        let mut builder = Request::builder().method(method).uri(path);
        let body = if let Some(body) = body {
            builder = builder.header("content-type", "application/json");
            Body::from(body.to_string())
        } else {
            Body::empty()
        };
        let response = app
            .oneshot(builder.body(body).expect("request should build"))
            .await
            .expect("route should respond");
        response_json(response).await
    }

    async fn response_json(response: Response) -> (StatusCode, Value) {
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("body should be readable");
        let json = serde_json::from_slice(&body).expect("body should be JSON");
        (parts.status, json)
    }

    fn test_router() -> Router {
        router_with_state(AtlasWebState::from_service(MockService))
    }

    struct MockService;

    impl AtlasWebService for MockService {
        fn readiness(&self) -> AppReadinessView {
            AppReadinessView {
                status: AppReadinessStatus::Ready,
                message: "fixture ready".to_string(),
            }
        }

        fn discover_filter_editor(
            &self,
            _request: DiscoverFilterEditorRequest,
        ) -> Result<FilterEditorView, AppServiceError> {
            Ok(FilterEditorView {
                matching_record_count: 3,
                groups: vec![FilterEditorGroupView {
                    id: "standard".to_string(),
                    label: "Standard".to_string(),
                    fields: vec![FilterEditorFieldView {
                        id: "kind".to_string(),
                        label: "Kinds".to_string(),
                        control: FilterControlView::MultiSelect,
                        placement: FilterFieldPlacement::AlwaysVisible,
                        applicability: atlas_app_model::FilterFieldApplicability::Applicable,
                        allowed_operators: vec![],
                        default_operator: atlas_app_model::FilterClauseOperator::IncludeAny,
                        supports_counts: true,
                    }],
                }],
            })
        }

        fn discover_filter_values(
            &self,
            _request: DiscoverFilterValuesRequest,
        ) -> Result<FilterValueListView, AppServiceError> {
            Ok(FilterValueListView {
                field_id: "pack".to_string(),
                matching_record_count: 3,
                options: vec![FilterValueOption {
                    value: "Actions".to_string(),
                    label: "Actions".to_string(),
                    count: Some(3),
                    selected: false,
                    disabled: false,
                    status: "available".to_string(),
                }],
            })
        }

        fn open_result_window(
            &self,
            _request: OpenResultWindowRequest,
        ) -> Result<ResultWindowPage, AppServiceError> {
            Ok(result_window_page(42, 1))
        }

        fn read_result_window_page(
            &self,
            window_id: u64,
            request: ReadResultWindowPageRequest,
        ) -> Result<ResultWindowPage, AppServiceError> {
            if window_id == 999 {
                return Err(AppServiceError::new(
                    AppErrorCode::WindowNotFound,
                    "window missing",
                ));
            }
            Ok(result_window_page(window_id, request.page.number))
        }

        fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError> {
            Ok(RecordDetailView {
                record_key: record_key.to_string(),
                title: "Test Action 1".to_string(),
                kind: "rule".to_string(),
                presentation: RecordPresentationDocument {
                    record_key: RecordKey::parse(record_key).expect("fixture key should parse"),
                    kind: RecordKind::Rule,
                    title: "Test Action 1".to_string(),
                    identity: vec![],
                    badges: vec![],
                    sections: vec![],
                },
            })
        }

        fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError> {
            Ok(SavedListIndexView {
                lists: vec![saved_list_summary()],
            })
        }

        fn saved_list(&self, list_ref: &str) -> Result<SavedListDetailView, AppServiceError> {
            if list_ref == "missing" {
                return Err(AppServiceError::new(
                    AppErrorCode::SavedListNotFound,
                    "saved list missing",
                ));
            }
            Ok(SavedListDetailView {
                list: saved_list_summary(),
                items: vec![SavedListItemView {
                    record_key: "actions:testAction1".to_string(),
                    position: 1,
                    note: Some("fixture note".to_string()),
                    status: SavedListItemStatusView::Active,
                    snapshot: SavedListItemSnapshotView {
                        title: "Test Action 1".to_string(),
                        kind: Some("rule".to_string()),
                    },
                    record: Some(record_summary()),
                }],
            })
        }

        fn filter_saved_list(
            &self,
            request: FilterSavedListRequest,
        ) -> Result<SavedListDetailView, AppServiceError> {
            self.saved_list(&request.list_ref)
        }

        fn create_saved_list(
            &self,
            request: CreateSavedListRequest,
        ) -> Result<SavedListCreateView, AppServiceError> {
            Ok(SavedListCreateView {
                list: SavedListSummaryView {
                    list_key: "list_created".to_string(),
                    slug: request.slug,
                    name: request.name,
                    description: request.description,
                    created_at: "2026-01-01T00:00:00Z".to_string(),
                    updated_at: "2026-01-01T00:00:00Z".to_string(),
                },
            })
        }

        fn update_saved_list(
            &self,
            request: UpdateSavedListRequest,
        ) -> Result<SavedListUpdateView, AppServiceError> {
            Ok(SavedListUpdateView {
                list: SavedListSummaryView {
                    list_key: request.list_key,
                    slug: request.slug,
                    name: request.name,
                    description: request.description,
                    created_at: "2026-01-01T00:00:00Z".to_string(),
                    updated_at: "2026-01-02T00:00:00Z".to_string(),
                },
            })
        }

        fn add_saved_list_item(
            &self,
            request: AddSavedListItemRequest,
        ) -> Result<SavedListItemMutationView, AppServiceError> {
            Ok(SavedListItemMutationView {
                list_key: request.list_ref,
                slug: "research".to_string(),
                record_key: request.record_ref,
                outcome: atlas_app_model::SavedListItemMutationOutcomeView::Added,
            })
        }

        fn remove_saved_list_item(
            &self,
            request: RemoveSavedListItemRequest,
        ) -> Result<SavedListItemMutationView, AppServiceError> {
            Ok(SavedListItemMutationView {
                list_key: request.list_ref,
                slug: "research".to_string(),
                record_key: request.record_ref,
                outcome: atlas_app_model::SavedListItemMutationOutcomeView::Removed,
            })
        }

        fn delete_saved_list(
            &self,
            list_ref: &str,
        ) -> Result<DeleteSavedListView, AppServiceError> {
            Ok(DeleteSavedListView {
                list_key: list_ref.to_string(),
                slug: "research".to_string(),
                deleted: true,
            })
        }
    }

    fn saved_list_summary() -> SavedListSummaryView {
        SavedListSummaryView {
            list_key: "list_research".to_string(),
            slug: "research".to_string(),
            name: "Research".to_string(),
            description: Some("Campaign prep".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn record_summary() -> RecordSummaryView {
        RecordSummaryView {
            record_key: "actions:testAction1".to_string(),
            title: "Test Action 1".to_string(),
            kind: "rule".to_string(),
            kind_label: "Rule".to_string(),
            level_label: None,
            rarity: None,
            traits: vec![],
            taxonomy: vec![],
            publication: None,
            pack: Some("Actions".to_string()),
            preview: None,
        }
    }

    fn result_window_page(window_id: u64, page_number: u32) -> ResultWindowPage {
        ResultWindowPage {
            window_id,
            mode: ResultWindowModeSummary::ListRecords,
            page: SearchPageView {
                number: page_number,
                size: 25,
                count: 1,
                total: 3,
                has_more: page_number < 3,
                next_page: Some(page_number + 1),
            },
            rows: vec![atlas_app_model::ResultWindowRow {
                record: record_summary(),
                match_summary: None,
            }],
        }
    }
}
