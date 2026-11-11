//! Bounded vocabulary reading workflows. Retrieval and ranking stay in search.
use crate::{
    AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext,
    definitions::check_generation,
};
use atlas_app_model::{
    RecordNavigationView, TraitDetailRequest, TraitDetailView, TraitRecordView, TraitRecordsPage,
    TraitRecordsRequest,
};
use atlas_domain::TraitKey;
use atlas_search::{SearchPage, TraitAssociationRequest};

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
                    content: definition
                        .prepared
                        .filter(|prepared| {
                            atlas_record::source_content::has_prepared_reading_content(
                                &prepared.text,
                                &prepared.html,
                                prepared.interactions.len(),
                            )
                        })
                        .map(|prepared| {
                            crate::prepared_content::reference_body(
                                prepared,
                                retrieval,
                                "trait-definition-centroid/v1",
                            )
                        })
                        .transpose()?,
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
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            let generation = retrieval.artifact_generation()?;
            check_generation(Some(&request.expected_artifact_generation), &generation)?;
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
                    record: crate::projection::localized_summary_view(&member.record, retrieval),
                })
                .collect();
            projection_execution.check()?;
            Ok(TraitRecordsPage {
                artifact_generation: generation,
                trait_key: key.to_string(),
                rows,
                page: crate::projection::search_page_view(result.page),
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
            trait_key: "trait:attack".into(),
            page: SearchPageRequest {
                number: 1,
                size: 20,
            },
            expected_artifact_generation: "stale".into(),
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
                    trait_key: "trait:agile".into(),
                    page: SearchPageRequest {
                        number: 1,
                        size: 20,
                    },
                    expected_artifact_generation: definition.artifact_generation,
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
