use atlas_app_model::{AppErrorCode, RecordDetailView};
use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, RecordRefResolutionResult, RecordResolutionResult,
    RecordRetrieval, RemasterLinksRequest, RemasterRetrieval, ResolveRecordRefRequest,
    ResolveRecordRequest,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_detail;
use crate::service::AtlasAppService;

impl AtlasAppService {
    pub fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> AppServiceResult<Vec<atlas_record::RetrievedRecord>> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?)
        })
    }

    pub fn resolve_record(
        &self,
        query: String,
        filter: Option<SearchFilterNode>,
    ) -> AppServiceResult<Vec<RecordResolutionResult>> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.resolve_record(ResolveRecordRequest {
                query: &query,
                filter: filter.as_ref(),
            })?)
        })
    }

    pub fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
    ) -> AppServiceResult<RecordRefResolutionResult> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.resolve_record_ref(ResolveRecordRefRequest {
                record_ref: &record_ref,
                filter: filter.as_ref(),
            })?)
        })
    }

    pub fn record_detail(&self, record_key: &str) -> AppServiceResult<RecordDetailView> {
        let record_key = RecordKey::parse(record_key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        self.submit_retrieval(move |retrieval| {
            let record = retrieval
                .get_record(GetRecordRequest {
                    record_key: &record_key,
                })?
                .ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::RecordNotFound,
                        format!("record `{record_key}` was not found"),
                    )
                })?;
            let remaster_links = retrieval.remaster_links(RemasterLinksRequest {
                record_key: &record_key,
            })?;
            record_detail(&record, remaster_links.as_ref())
        })
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        AppErrorCode, RecordSurfaceEditionStatusView, RecordSurfacePresentationView,
        RecordSurfaceProfileView, SurfaceUnavailableReasonView,
    };

    use crate::test_support::fixture_worker;

    #[test]
    fn worker_record_detail_reports_valid_invalid_and_missing_keys() {
        let fixture = fixture_worker();
        let worker = &fixture.worker;

        let detail = worker
            .record_detail("actions:testAction1")
            .expect("fixture record should load");
        assert_eq!(
            detail.surface.metadata.record_key.as_deref(),
            Some("actions:testAction1")
        );
        assert_eq!(detail.surface.metadata.title, "Test Action 1");
        assert_eq!(detail.surface.metadata.kind, "rule");
        let edition = detail
            .surface
            .metadata
            .edition
            .expect("canonical record should expose edition metadata");
        assert_eq!(edition.status, RecordSurfaceEditionStatusView::Legacy);
        assert!(edition.counterparts.is_empty());
        assert_eq!(
            detail.surface.profile,
            RecordSurfaceProfileView::RecordDetail
        );
        assert!(matches!(
            detail.surface.presentation,
            RecordSurfacePresentationView::Unavailable { unavailable }
                if unavailable.reason == SurfaceUnavailableReasonView::RecordFamilyNotMigrated
        ));

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
}
