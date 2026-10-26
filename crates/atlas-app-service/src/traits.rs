//! Bounded vocabulary reading workflows. Retrieval and ranking stay in search.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppRecoverableAction, RecordNavigationView, TraitAssociationKind, TraitDetailRequest,
    TraitDetailView, TraitRecordView, TraitRecordsPage, TraitRecordsRequest,
};
use atlas_domain::TraitKey;
use atlas_search::{RecordScope, SearchPage, SimilarTraitRequest, TraitAssociationRequest};

impl AtlasAppService {
    pub fn trait_definition(
        &self,
        request: TraitDetailRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<TraitDetailView> {
        execution.check()?;
        let key = parse_key(&request.trait_key)?;
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            let generation = retrieval.artifact_generation()?;
            check_generation(request.expected_artifact_generation.as_deref(), &generation)?;
            let definition = retrieval.trait_definition(&key)?;
            projection_execution.check()?;
            Ok(match definition {
                Some(definition) => TraitDetailView {
                    artifact_generation: generation,
                    trait_key: definition.key.to_string(),
                    label: definition.label,
                    content: definition.prepared.map(|prepared| {
                        atlas_app_model::TraitContentView {
                            body: atlas_app_model::PreparedFieldBodyView::Html {
                                inclusions: vec![],
                                html: prepared.html,
                                controls: crate::prepared_content::content_controls(
                                    &prepared.interactions,
                                    None,
                                ),
                            },
                            references: prepared
                                .references
                                .into_iter()
                                .filter(|reference| reference.visible)
                                .map(|reference| {
                                    let (status, target, url) =
                                        crate::projection::content_reference_destination(
                                            reference.resolution,
                                            retrieval.source_fingerprint(),
                                        );
                                    atlas_app_model::PreparedReferenceView {
                                        self_action_reference: None,
                                        ordinal: reference.ordinal,
                                        status: status.into(),
                                        target,
                                        url,
                                        effect_label: None,
                                        contextual_target: None,
                                    }
                                })
                                .collect(),
                        }
                    }),
                    catalogs: definition.catalogs,
                    source_revision: definition.source_revision,
                    indexing_locale: Some(definition.indexing_locale),
                },
                None => TraitDetailView {
                    artifact_generation: generation,
                    trait_key: key.to_string(),
                    label: key.identifier().to_owned(),
                    content: None,
                    catalogs: vec![],
                    source_revision: None,
                    indexing_locale: None,
                },
            })
        })
    }

    pub fn trait_records(
        &self,
        request: TraitRecordsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<TraitRecordsPage> {
        execution.check()?;
        let key = parse_key(&request.trait_key)?;
        let page = SearchPage::new(request.page.number, request.page.size)?;
        let sort = if request.kind == TraitAssociationKind::Similar {
            let sort = request
                .sort
                .unwrap_or(atlas_domain::ResultSort::Relevance {});
            sort.validate().map_err(AppServiceError::invalid_request)?;
            if matches!(sort, atlas_domain::ResultSort::SavedOrder {}) {
                return Err(AppServiceError::invalid_request(
                    "Saved order is available only for saved lists",
                ));
            }
            Some(sort)
        } else {
            if request.sort.is_some() {
                return Err(AppServiceError::invalid_request(
                    "Sorting is available for Similar results",
                ));
            }
            None
        };
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            let generation = retrieval.artifact_generation()?;
            check_generation(Some(&request.expected_artifact_generation), &generation)?;
            let (rows, page, coverage, search_profile) = match request.kind {
                TraitAssociationKind::Members => {
                    let result = retrieval.trait_memberships(TraitAssociationRequest {
                        trait_key: &key,
                        page,
                    })?;
                    projection_execution.check()?;
                    let rows = result
                        .entries
                        .into_iter()
                        .map(|member| TraitRecordView {
                            source_label: member.source_label,
                            reference_direction: None,
                            navigation: RecordNavigationView {
                                spell_rank: None,
                                record_key: member.record.key.to_string(),
                                source_fingerprint: crate::projection::navigation_fingerprint(
                                    &member.owners,
                                    retrieval.source_fingerprint(),
                                ),
                                owners: member.owners,
                                field: None,
                                passage: None,
                            },
                            record: crate::projection::localized_summary_view(
                                &member.record,
                                retrieval,
                            ),
                        })
                        .collect();
                    (
                        rows,
                        crate::projection::search_page_view(result.page),
                        None,
                        None,
                    )
                }
                TraitAssociationKind::References => {
                    let result = retrieval.trait_references(TraitAssociationRequest {
                        trait_key: &key,
                        page,
                    })?;
                    projection_execution.check()?;
                    let rows = result
                        .entries
                        .into_iter()
                        .map(|reference| TraitRecordView {
                            source_label: reference.source_label,
                            navigation: RecordNavigationView {
                                spell_rank: None,
                                record_key: reference.record.key.to_string(),
                                source_fingerprint: crate::projection::navigation_fingerprint(
                                    &reference.owners,
                                    retrieval.source_fingerprint(),
                                ),
                                owners: reference.owners,
                                field: reference.field,
                                passage: None,
                            },
                            record: crate::projection::localized_summary_view(
                                &reference.record,
                                retrieval,
                            ),
                            reference_direction: Some(reference.direction),
                        })
                        .collect();
                    (
                        rows,
                        crate::projection::search_page_view(result.page),
                        None,
                        None,
                    )
                }
                TraitAssociationKind::Similar => {
                    let result = retrieval
                        .similar_trait(SimilarTraitRequest {
                            sort: sort
                                .clone()
                                .unwrap_or(atlas_domain::ResultSort::Relevance {}),
                            trait_key: &key,
                            search_profile: request.search_profile.as_ref(),
                            ranking: request.ranking.unwrap_or_default(),
                            filter: None,
                            scope: RecordScope::All,
                            page,
                            prefer_remaster: true,
                        })?
                        .ok_or_else(|| {
                            AppServiceError::invalid_request(
                                "Trait definition is not available for Similar",
                            )
                        })?;
                    projection_execution.check()?;
                    let mut results =
                        crate::projection::search_results_view(result.results, retrieval);
                    if sort.as_ref().is_some_and(|s| s.is_unit_price()) {
                        let mut records = results
                            .rows
                            .iter()
                            .map(|r| r.record.clone())
                            .collect::<Vec<_>>();
                        crate::result_order::price_labels(retrieval, &mut records)?;
                        for (row, record) in results.rows.iter_mut().zip(records) {
                            row.record = record;
                        }
                    }
                    let rows = results
                        .rows
                        .into_iter()
                        .map(|row| TraitRecordView {
                            source_label: None,
                            reference_direction: None,
                            navigation: root_navigation(row.record.record_key.clone()),
                            record: row.record,
                        })
                        .collect();
                    (
                        rows,
                        results.page,
                        Some(results.coverage),
                        Some(results.search_profile),
                    )
                }
            };
            projection_execution.check()?;
            Ok(TraitRecordsPage {
                sort,
                search_profile,
                artifact_generation: generation,
                trait_key: key.to_string(),
                kind: request.kind,
                rows,
                page,
                coverage,
            })
        })
    }
}
fn parse_key(value: &str) -> AppServiceResult<TraitKey> {
    value
        .parse()
        .map_err(|e: atlas_domain::TraitKeyParseError| {
            AppServiceError::invalid_request(e.to_string())
        })
}
fn check_generation(expected: Option<&str>, actual: &str) -> AppServiceResult<()> {
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(AppServiceError::artifact_generation_changed(
            expected,
            actual,
            AppRecoverableAction::ReloadRecord,
        ));
    }
    Ok(())
}
fn root_navigation(record_key: String) -> RecordNavigationView {
    RecordNavigationView {
        spell_rank: None,
        record_key,
        owners: vec![],
        field: None,
        passage: None,
        source_fingerprint: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::{AppErrorCode, SearchPageRequest};

    #[test]
    fn unknown_definition_is_honest_and_associations_require_current_generation() {
        let fixture = crate::test_support::fixture_worker();
        let execution = ReadExecutionContext::uncancellable();
        let view = fixture
            .worker
            .trait_definition(
                TraitDetailRequest {
                    trait_key: "trait:custom-trait".into(),
                    expected_artifact_generation: None,
                },
                &execution,
            )
            .unwrap();
        assert_eq!(view.trait_key, "trait:custom-trait");
        assert!(view.content.is_none());
        assert!(view.source_revision.is_none());
        assert!(view.indexing_locale.is_none());
        let request = TraitRecordsRequest {
            sort: None,
            trait_key: "trait:attack".into(),
            kind: TraitAssociationKind::Members,
            page: SearchPageRequest {
                number: 1,
                size: 20,
            },
            expected_artifact_generation: "stale".into(),
            search_profile: None,
            ranking: None,
        };
        let error = fixture
            .worker
            .trait_records(request.clone(), &execution)
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
        let result = fixture
            .worker
            .trait_records(
                TraitRecordsRequest {
                    sort: None,
                    expected_artifact_generation: view.artifact_generation,
                    ..request
                },
                &execution,
            )
            .unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(
            result.rows[0].navigation.record_key,
            "actions:testAction100000"
        );
        assert!(result.rows[0].navigation.owners.is_empty());
        assert_eq!(
            result.rows[0].record.traits[0].trait_key.as_deref(),
            Some("trait:attack")
        );
    }
    #[test]
    fn embedded_membership_navigates_to_the_exact_owner_without_tagging_its_parent() {
        let fixture = crate::test_support::fixture_worker_with_executor(
            crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, || {
                Ok(atlas_search::test_support::open_source_fixture(
                    vec![atlas_search::test_support::record(
                        "actors",
                        "Actor",
                        serde_json::json!({
                            "_id":"creature00000000", "name":"Creature", "type":"npc",
                            "system":{"traits":{"value":[]}},
                            "items":[{"_id":"claw000000000000","name":"Claw","type":"melee","system":{"traits":{"value":["agile"]}}}]
                        }),
                    )],
                    false,
                )?)
            }),
        );
        let execution = ReadExecutionContext::uncancellable();
        let definition = fixture
            .worker
            .trait_definition(
                TraitDetailRequest {
                    trait_key: "trait:agile".into(),
                    expected_artifact_generation: None,
                },
                &execution,
            )
            .unwrap();
        let rows = fixture
            .worker
            .trait_records(
                TraitRecordsRequest {
                    sort: None,
                    trait_key: "trait:agile".into(),
                    kind: TraitAssociationKind::Members,
                    page: SearchPageRequest {
                        number: 1,
                        size: 20,
                    },
                    expected_artifact_generation: definition.artifact_generation,
                    search_profile: None,
                    ranking: None,
                },
                &execution,
            )
            .unwrap()
            .rows;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].navigation.record_key, "actors:creature00000000");
        assert_eq!(rows[0].source_label.as_deref(), Some("Claw"));
        assert_eq!(
            rows[0].navigation.owners,
            vec![atlas_domain::OwnedContentLocator {
                collection: "/items".into(),
                identity: atlas_domain::OwnedContentIdentity::Stable("claw000000000000".into()),
            }]
        );
        assert!(
            rows[0].record.traits.is_empty(),
            "embedded trait must not become a parent trait"
        );
    }
}
