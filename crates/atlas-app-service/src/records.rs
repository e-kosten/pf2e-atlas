use atlas_app_model::{AppErrorCode, RecordDetailView};
use atlas_domain::RecordKey;
use atlas_search::{GetRecordRequest, RecordRetrieval};

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_detail;
use crate::service::AtlasAppService;

impl AtlasAppService {
    pub fn record_detail(&self, record_key: &str) -> AppServiceResult<RecordDetailView> {
        let record_key = RecordKey::parse(record_key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        self.retrieval.submit(move |retrieval| {
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
            record_detail(&record)
        })
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::AppErrorCode;

    use crate::test_support::fixture_worker;

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
}
