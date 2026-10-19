use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppRecoverableAction, RecordBodyView, RecordDetailRequest, RollTableRequest, TableRollView,
};

impl AtlasAppService {
    /// Reference roll only: no deck consumption or recursive linked-table execution.
    pub fn roll_table(&self, request: RollTableRequest) -> AppServiceResult<TableRollView> {
        self.with_retrieval(&ReadExecutionContext::uncancellable(), move |retrieval| {
            let (_, detail) = crate::records::read_record_detail(retrieval, request.selection)?;
            if detail.artifact_generation != request.artifact_generation {
                return Err(AppServiceError::artifact_generation_changed(
                    &request.artifact_generation,
                    &detail.artifact_generation,
                    AppRecoverableAction::ReloadRecord,
                ));
            }
            let RecordBodyView::RollTable(table) = detail.presentation.body else {
                return Err(AppServiceError::invalid_request("Choose a roll table."));
            };
            let formula = table.formula.value.as_deref().ok_or_else(|| {
                AppServiceError::invalid_request("This table has no usable roll formula.")
            })?;
            let rows = table.results.value.ok_or_else(|| {
                AppServiceError::invalid_request("This table has no usable results.")
            })?;
            let outcome = atlas_dice::roll(formula).map_err(|error| {
                AppServiceError::invalid_request(format!("This table cannot be rolled: {error}"))
            })?;
            let mut matches = Vec::new();
            let mut invalid_ranges = Vec::new();
            for row in rows {
                let navigation = row.link.navigation;
                match row.range.value {
                    None => invalid_ranges.push(navigation),
                    Some(range) if contains(range, outcome.total.into()) => {
                        let (_, result) = crate::records::read_record_detail(
                            retrieval,
                            RecordDetailRequest {
                                spell_rank: None,
                                record_key: navigation.record_key,
                                owners: navigation.owners,
                                fields: vec![],
                                passage: None,
                                source_fingerprint: navigation.source_fingerprint,
                            },
                        )?;
                        matches.push(result);
                    }
                    Some(_) => {}
                }
            }
            Ok(TableRollView {
                outcome,
                matches,
                invalid_ranges,
            })
        })
    }
}

fn contains([low, high]: [f64; 2], total: f64) -> bool {
    (low..=high).contains(&total)
}

#[cfg(test)]
mod tests {
    use super::contains;
    #[test]
    fn table_ranges_include_both_endpoints_and_leave_gaps() {
        assert!(contains([2.0, 4.0], 2.0));
        assert!(contains([2.0, 4.0], 4.0));
        assert!(contains([4.0, 4.0], 4.0));
        assert!(!contains([2.0, 4.0], 5.0));
        assert!(!contains([6.0, 9.0], 5.0));
        assert!(contains([4.0, 9.0], 4.0)); // Overlap is intentionally allowed.
    }
}

#[cfg(test)]
mod service_tests {
    use super::*;
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;

    #[test]
    fn table_roll_returns_all_matches_and_prepared_content_without_consuming_rows() {
        let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            || {
                Ok(open_source_fixture(
                    vec![
                        record(
                            "tables",
                            "RollTable",
                            json!({
                                "_id":"table00000000000", "name":"Ranges", "formula":"4", "replacement":false,
                                "results":[
                                    {"_id":"result0000000000", "type":"text", "range":[1,4], "description":"<p>First match</p>"},
                                    {"_id":"result0000000001", "type":"text", "range":[4,8], "description":"<p>Second match</p>"},
                                    {"_id":"result0000000002", "type":"text", "range":[9,2], "description":"Invalid"},
                                    {"_id":"result0000000003", "type":"text", "range":[10,12], "description":"No match"}
                                ]
                            }),
                        ),
                        record(
                            "tables",
                            "RollTable",
                            json!({
                                "_id":"table00000000001", "name":"Gap", "formula":"5", "results":[
                                    {"_id":"result0000000000", "type":"text", "range":[1,4], "description":"Not selected"}
                                ]
                            }),
                        ),
                    ],
                    false,
                )?)
            },
        ));
        let detail = fixture
            .worker
            .record_detail("tables:table00000000000")
            .unwrap();
        let mut request = RollTableRequest {
            artifact_generation: detail.artifact_generation,
            selection: RecordDetailRequest {
                spell_rank: None,
                record_key: "tables:table00000000000".into(),
                owners: vec![],
                fields: vec![],
                passage: None,
                source_fingerprint: None,
            },
        };
        let rolled = fixture.worker.roll_table(request.clone()).unwrap();
        assert_eq!(rolled.outcome.total, 4);
        assert_eq!(rolled.matches.len(), 2);
        assert_eq!(rolled.invalid_ranges.len(), 1);
        assert!(!rolled.matches[0].presentation.content.is_empty());
        assert_eq!(
            fixture
                .worker
                .roll_table(request.clone())
                .unwrap()
                .matches
                .len(),
            2,
            "reference rolls do not consume results even on non-replacing tables"
        );
        request.selection.record_key = "tables:table00000000001".into();
        assert!(
            fixture
                .worker
                .roll_table(request.clone())
                .unwrap()
                .matches
                .is_empty()
        );
        request.artifact_generation = "stale".into();
        assert_eq!(
            fixture
                .worker
                .roll_table(request)
                .unwrap_err()
                .into_app_error()
                .code,
            atlas_app_model::AppErrorCode::ArtifactGenerationChanged
        );
    }
}
