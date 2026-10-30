use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppRecoverableAction, RecordBodyView, RecordDetailRequest, RollTableRequest, TableRollView,
};

impl AtlasAppService {
    /// Reference roll only: no deck consumption or recursive linked-table execution.
    pub fn roll_table(&self, request: RollTableRequest) -> AppServiceResult<TableRollView> {
        self.with_retrieval(&ReadExecutionContext::uncancellable(), move |retrieval| {
            let (loaded, detail) =
                crate::records::LoadedRecordDetail::read(retrieval, request.selection)?;
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
                        let result = loaded.project(RecordDetailRequest {
                            spell_rank: None,
                            record_key: navigation.record_key,
                            owners: navigation.owners,
                            fields: vec![],
                            passage: None,
                            source_fingerprint: navigation.source_fingerprint,
                        })?;
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
        assert_eq!(fixture.worker.last_read_metrics().source_body_decodes, 1);
        assert_eq!(rolled.outcome.total, 4);
        assert_eq!(rolled.matches.len(), 2);
        assert_eq!(rolled.invalid_ranges.len(), 1);
        for (result, expected) in rolled
            .matches
            .iter()
            .zip(["<p>First match</p>", "<p>Second match</p>"])
        {
            assert!(result.presentation.content.iter().any(|field| matches!(&field.body, atlas_app_model::PreparedFieldBodyView::Html { html, .. } if html == expected)));
        }
        for result in &rolled.matches {
            let ordinary = fixture
                .worker
                .record_detail_at(RecordDetailRequest {
                    spell_rank: None,
                    record_key: result.selected.record_key.clone(),
                    owners: result.selected.owners.clone(),
                    fields: vec![],
                    passage: None,
                    source_fingerprint: result.selected.source_fingerprint.clone(),
                })
                .unwrap();
            assert_eq!(
                serde_json::to_value(result).unwrap(),
                serde_json::to_value(ordinary).unwrap()
            );
        }
        assert_eq!(
            rolled.invalid_ranges[0].owners,
            vec![atlas_domain::OwnedContentLocator {
                collection: "/results".into(),
                identity: atlas_domain::OwnedContentIdentity::Stable("result0000000002".into()),
            }]
        );
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

    fn tables_fixture(tables: Vec<serde_json::Value>) -> crate::test_support::FixtureWorker {
        fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            move || {
                Ok(open_source_fixture(
                    tables
                        .into_iter()
                        .map(|table| record("tables", "RollTable", table))
                        .collect(),
                    false,
                )?)
            },
        ))
    }

    fn request(fixture: &crate::test_support::FixtureWorker, key: &str) -> RollTableRequest {
        let detail = fixture.worker.record_detail(key).unwrap();
        RollTableRequest {
            artifact_generation: detail.artifact_generation,
            selection: RecordDetailRequest {
                spell_rank: None,
                record_key: key.into(),
                owners: vec![],
                fields: vec![],
                passage: None,
                source_fingerprint: None,
            },
        }
    }

    #[test]
    fn table_roll_preserves_snapshot_local_results_links_and_unavailable_ranges() {
        let fixture = tables_fixture(vec![
            json!({
                "_id":"table00000000000", "name":"Local results", "formula":"4", "replacement":false,
                "results":[
                    {"type":"text", "range":[4,4], "description":"<p>@UUID[Compendium.pf2e.tables.RollTable.table00000000001]{Other table}</p>"},
                    {"type":"text", "range":null, "description":"Null range"},
                    {"type":"text", "description":"Missing range"},
                    {"type":"text", "range":[8,1], "description":"Inverted range"},
                    {"type":"text", "range":[3.5,4.5], "description":"<p>Fractional endpoints</p>"}
                ]
            }),
            json!({"_id":"table00000000001", "name":"Other table", "formula":"1", "results":[]}),
        ]);
        let request = request(&fixture, "tables:table00000000000");
        for _ in 0..2 {
            let rolled = fixture.worker.roll_table(request.clone()).unwrap();
            assert_eq!(fixture.worker.last_read_metrics().source_body_decodes, 1);
            assert_eq!(rolled.outcome.total, 4);
            assert_eq!(rolled.matches.len(), 2);
            assert_eq!(rolled.invalid_ranges.len(), 3);
            for (result, index) in rolled.matches.iter().zip([0, 4]) {
                assert_eq!(
                    result.selected.owners,
                    vec![atlas_domain::OwnedContentLocator {
                        collection: "/results".into(),
                        identity: atlas_domain::OwnedContentIdentity::SnapshotLocal { index },
                    }]
                );
                assert!(result.selected.source_fingerprint.is_some());
                let ordinary = fixture
                    .worker
                    .record_detail_at(RecordDetailRequest {
                        record_key: result.selected.record_key.clone(),
                        owners: result.selected.owners.clone(),
                        source_fingerprint: result.selected.source_fingerprint.clone(),
                        spell_rank: None,
                        fields: vec![],
                        passage: None,
                    })
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(result).unwrap(),
                    serde_json::to_value(ordinary).unwrap()
                );
            }
            assert!(rolled.matches[0].relationships.iter().any(|relationship| {
                relationship
                    .target
                    .as_ref()
                    .is_some_and(|target| target.record_key == "tables:table00000000001")
            }));
            assert!(rolled.matches[0].presentation.content.iter().any(|field| matches!(&field.body, atlas_app_model::PreparedFieldBodyView::Html { html, .. } if html.contains("Other table") && html.contains("data-atlas-reference"))));
            for (range, index) in rolled.invalid_ranges.iter().zip(1..=3) {
                assert_eq!(
                    range.owners[0].identity,
                    atlas_domain::OwnedContentIdentity::SnapshotLocal { index }
                );
            }
        }
    }

    #[test]
    fn table_roll_keeps_unavailable_collections_distinct_from_an_empty_result_set() {
        let fixture = tables_fixture(vec![
            json!({"_id":"table00000000000", "name":"Missing", "formula":"4"}),
            json!({"_id":"table00000000001", "name":"Null", "formula":"4", "results":null}),
            json!({"_id":"table00000000002", "name":"Invalid", "formula":"4", "results":42}),
            json!({"_id":"table00000000003", "name":"Empty", "formula":"4", "results":[]}),
        ]);
        for id in 0..3 {
            let key = format!("tables:table0000000000{id}");
            let request = request(&fixture, &key);
            let error = fixture
                .worker
                .roll_table(request)
                .unwrap_err()
                .into_app_error();
            assert_eq!(error.code, atlas_app_model::AppErrorCode::InvalidRequest);
            assert_eq!(error.message, "This table has no usable results.");
            assert_eq!(fixture.worker.last_read_metrics().source_body_decodes, 1);
        }
        let rolled = fixture
            .worker
            .roll_table(request(&fixture, "tables:table00000000003"))
            .unwrap();
        assert_eq!(rolled.outcome.total, 4);
        assert!(rolled.matches.is_empty());
        assert!(rolled.invalid_ranges.is_empty());
    }

    #[test]
    fn table_roll_validates_the_complete_detail_selection_before_generation_and_dice() {
        let fixture = tables_fixture(vec![json!({
            "_id":"table00000000000", "name":"Invalid formula", "formula":"not dice",
            "description":"<p>Table description</p>", "results":[{"type":"text", "range":[1,4]}]
        })]);
        let request = request(&fixture, "tables:table00000000000");
        let mut invalid = Vec::new();
        let mut selection = request.selection.clone();
        selection.record_key = "invalid-key".into();
        invalid.push(selection);
        let mut selection = request.selection.clone();
        selection.fields = vec!["/undeclared".into()];
        invalid.push(selection);
        let mut selection = request.selection.clone();
        selection.fields = vec!["/description".into(); 257];
        invalid.push(selection);
        let mut selection = request.selection.clone();
        selection.owners = vec![atlas_domain::OwnedContentLocator {
            collection: "/results".into(),
            identity: atlas_domain::OwnedContentIdentity::Stable("missing".into()),
        }];
        invalid.push(selection);
        let mut selection = request.selection.clone();
        selection.owners = vec![atlas_domain::OwnedContentLocator {
            collection: "/results".into(),
            identity: atlas_domain::OwnedContentIdentity::SnapshotLocal { index: 0 },
        }];
        invalid.push(selection);
        let mut selection = request.selection.clone();
        selection.spell_rank = Some(2);
        invalid.push(selection);
        let mut selection = request.selection.clone();
        selection.fields = vec!["/description".into()];
        selection.passage = Some(atlas_domain::SourcePassageAddress::Identity {});
        invalid.push(selection);
        let mut selection = request.selection.clone();
        selection.passage = Some(atlas_domain::SourcePassageAddress::PlainSection {
            source_text_sha256: "stale".into(),
            selection_version: "stale".into(),
            section_ordinal: 0,
            label: None,
            chunk_bytes: atlas_domain::SourceByteRange { start: 0, end: 1 },
        });
        invalid.push(selection);
        for selection in invalid {
            let expected = fixture
                .worker
                .record_detail_at(selection.clone())
                .unwrap_err()
                .into_app_error();
            let actual = fixture
                .worker
                .roll_table(RollTableRequest {
                    selection,
                    artifact_generation: "stale".into(),
                })
                .unwrap_err()
                .into_app_error();
            assert_eq!(
                (actual.code, actual.message),
                (expected.code, expected.message)
            );
        }
        let error = fixture
            .worker
            .roll_table(request)
            .unwrap_err()
            .into_app_error();
        assert!(error.message.starts_with("This table cannot be rolled:"));
    }

    #[test]
    fn loaded_detail_cannot_project_a_different_root() {
        let fixture = tables_fixture(vec![
            json!({"_id":"table00000000000", "name":"Table", "formula":"4", "results":[]}),
        ]);
        let selection = request(&fixture, "tables:table00000000000").selection;
        let error = fixture
            .worker
            .with_retrieval(&ReadExecutionContext::uncancellable(), move |r| {
                let (loaded, _) = crate::records::LoadedRecordDetail::read(r, selection.clone())?;
                loaded.project(RecordDetailRequest {
                    record_key: "tables:table00000000001".into(),
                    ..selection
                })
            })
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, atlas_app_model::AppErrorCode::InvalidRequest);
        assert_eq!(
            error.message,
            "detail selection belongs to another loaded record"
        );
    }
}
