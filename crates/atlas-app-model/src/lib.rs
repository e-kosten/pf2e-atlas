#![deny(unsafe_code)]

mod error;
mod filter;
mod readiness;
mod record;
mod result_window;

pub use error::{AppError, AppErrorCode, AppRecoverableAction};
pub use filter::{
    BasicSearchFilter, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest, FilterClause,
    FilterClauseOperator, FilterControlView, FilterDiscoveryContext, FilterEditorFieldView,
    FilterEditorGroupView, FilterEditorView, FilterFieldApplicability, FilterFieldPlacement,
    FilterRange, FilterValidationCode, FilterValidationMessage, FilterValidationResult,
    FilterValueListView, FilterValueOption, MetricComparison,
};
pub use readiness::{AppReadinessStatus, AppReadinessView};
pub use record::{RecordBadgeView, RecordDetailView, RecordSummaryView};
pub use result_window::{
    BasicSearchMode, BasicSearchState, OpenResultWindowRequest, ReadResultWindowPageRequest,
    RecordListSortView, ResultMatchSummary, ResultWindowMode, ResultWindowModeSummary,
    ResultWindowPage, ResultWindowRow, SearchPageRequest, SearchPageView,
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use ts_rs::TS;

    use super::*;

    #[test]
    fn typescript_bindings_are_fresh() {
        let temp_dir = fresh_temp_dir("atlas-app-model-bindings-check");
        export_bindings_to(&temp_dir);

        let expected = read_binding_dir(&binding_dir());
        let actual = read_binding_dir(&temp_dir);
        assert_eq!(
            expected, actual,
            "generated TypeScript bindings are stale; run `cargo test -p atlas-app-model export_typescript_bindings -- --ignored`"
        );

        fs::remove_dir_all(&temp_dir).expect("temporary binding directory should be removable");
    }

    #[test]
    #[ignore = "regenerates checked-in TypeScript bindings"]
    fn export_typescript_bindings() {
        let bindings = binding_dir();
        fs::create_dir_all(&bindings).expect("binding directory should be creatable");
        export_bindings_to(&bindings);
    }

    fn export_bindings_to(path: &Path) {
        fs::create_dir_all(path).expect("binding export directory should be creatable");
        AppError::export_all_to(path).expect("AppError bindings should export");
        AppReadinessView::export_all_to(path).expect("AppReadinessView bindings should export");
        BasicSearchFilter::export_all_to(path).expect("BasicSearchFilter bindings should export");
        BasicSearchState::export_all_to(path).expect("BasicSearchState bindings should export");
        DiscoverFilterEditorRequest::export_all_to(path)
            .expect("DiscoverFilterEditorRequest bindings should export");
        DiscoverFilterValuesRequest::export_all_to(path)
            .expect("DiscoverFilterValuesRequest bindings should export");
        FilterEditorView::export_all_to(path).expect("FilterEditorView bindings should export");
        FilterDiscoveryContext::export_all_to(path)
            .expect("FilterDiscoveryContext bindings should export");
        FilterValidationResult::export_all_to(path)
            .expect("FilterValidationResult bindings should export");
        FilterValueListView::export_all_to(path)
            .expect("FilterValueListView bindings should export");
        FilterValueOption::export_all_to(path).expect("FilterValueOption bindings should export");
        OpenResultWindowRequest::export_all_to(path)
            .expect("OpenResultWindowRequest bindings should export");
        ReadResultWindowPageRequest::export_all_to(path)
            .expect("ReadResultWindowPageRequest bindings should export");
        RecordDetailView::export_all_to(path).expect("RecordDetailView bindings should export");
        RecordSummaryView::export_all_to(path).expect("RecordSummaryView bindings should export");
        ResultWindowPage::export_all_to(path).expect("ResultWindowPage bindings should export");
    }

    fn read_binding_dir(path: &Path) -> BTreeMap<String, String> {
        let mut files = BTreeMap::new();
        for entry in fs::read_dir(path).expect("binding directory should be readable") {
            let entry = entry.expect("binding directory entry should be readable");
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("ts") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .expect("binding file name should be utf-8")
                .to_string();
            let contents = fs::read_to_string(&path).expect("binding file should be readable");
            files.insert(name, contents);
        }
        files
    }

    fn binding_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bindings")
    }

    fn fresh_temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).expect("stale temporary binding directory should be removed");
        }
        path
    }
}
