use crate::{
    ReadExecutionContext, executor::RetrievalExecutor, test_support::fixture_worker_with_executor,
};
use atlas_app_model::*;
use atlas_domain::{QueryFacetOutcome, QueryLiteral};
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

fn fixture() -> crate::test_support::FixtureWorker {
    fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![
                record(
                    "items",
                    "Item",
                    json!({"_id":"target0000000000","name":"Leather Armor","type":"armor"}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"actor00000000000","name":"Linked Guard","type":"npc","items":[{"_id":"owned00000000000","name":"Armor","type":"armor","_stats":{"compendiumSource":"Compendium.pf2e.items.Item.target0000000000"},"system":{"description":{"value":"<p>@UUID[Compendium.pf2e.items.Item.target0000000000]</p>"}}}]}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"actor00000000001","name":"Unlinked Guard","type":"npc"}),
                ),
            ],
            false,
        )?)
    }))
}
fn relation(f: &crate::test_support::FixtureWorker) -> (atlas_domain::QueryRelationship, String) {
    let target = f.worker.record_detail("items:target0000000000").unwrap();
    (
        atlas_domain::QueryRelationship {
            direction: atlas_domain::QueryRelationshipDirection::Incoming,
            selection: crate::relationship_search::query_selection(&target.selected).unwrap(),
            include_contents: true,
        },
        target.artifact_generation,
    )
}
fn predicate(relationship: atlas_domain::QueryRelationship) -> atlas_domain::QueryPredicate {
    atlas_domain::QueryPredicate {
        clause_id: Some("link".into()),
        expression: atlas_domain::QueryExpression::Relationship { relationship },
    }
}
fn filtered(
    constraint: atlas_domain::QueryRelationship,
    generation: &str,
) -> FilterDiscoveryContext {
    FilterDiscoveryContext::Filtered {
        search_content: None,
        search_profile: None,
        expected_artifact_generation: Some(generation.into()),
        filter: Some(predicate(constraint)),
        text: None,
        mode: RetrievalModeView::Lexical,
    }
}
fn counts_request(context: FilterDiscoveryContext) -> DiscoverFilterCountsRequest {
    DiscoverFilterCountsRequest {
        context,
        field_id: "kind".into(),
        clause_id: None,
        values: vec![
            QueryLiteral::String("creature".into()),
            QueryLiteral::String("equipment".into()),
        ],
    }
}
fn count_values(view: FilterCountsView) -> Vec<u64> {
    let QueryFacetOutcome::Available { value } = view.counts else {
        panic!("relationship counts unavailable")
    };
    value
        .options
        .into_iter()
        .map(|o| o.distinct_roots)
        .collect()
}
fn values_request(context: FilterDiscoveryContext) -> DiscoverFilterValuesRequest {
    DiscoverFilterValuesRequest::Contextual {
        only_matching: true,
        context: Box::new(context),
        field_id: "kind".into(),
        clause_id: None,
        text: None,
        offset: 0,
        limit: 20,
    }
}
#[test]
fn relationship_discovery_validates_scope_and_intersects_saved_list_keys() {
    let f = fixture();
    let execution = ReadExecutionContext::uncancellable();
    let (constraint, generation) = relation(&f);
    assert_eq!(
        count_values(
            f.worker
                .discover_filter_counts(
                    counts_request(filtered(constraint.clone(), &generation)),
                    &execution
                )
                .unwrap()
        ),
        vec![1, 0]
    );
    let values = f
        .worker
        .discover_filter_values(
            values_request(filtered(constraint.clone(), &generation)),
            &execution,
        )
        .unwrap();
    let FilterValueListView::Contextual {
        outcome: QueryFacetOutcome::Available { value },
        artifact_generation,
        ..
    } = values
    else {
        panic!("expected relationship-scoped values")
    };
    assert_eq!(artifact_generation, generation);
    assert_eq!(value.options.len(), 1);
    assert_eq!(value.options[0].distinct_roots, 1);
    let mut main = constraint.clone();
    main.include_contents = false;
    assert_eq!(
        count_values(
            f.worker
                .discover_filter_counts(counts_request(filtered(main, &generation)), &execution)
                .unwrap()
        ),
        vec![0, 0]
    );
    for (slug, keys) in [
        (
            "linked",
            vec![
                "actors:actor00000000000",
                "actors:actor00000000001",
                "items:missing000000000",
            ],
        ),
        ("unlinked", vec!["actors:actor00000000001"]),
        ("empty", vec![]),
    ] {
        f.worker
            .with_write(|write| {
                Ok(write.local_state().saved_lists().import(
                    atlas_local_state::ImportSavedList {
                        slug: slug.into(),
                        name: slug.into(),
                        description: None,
                        items: keys
                            .iter()
                            .map(|key| atlas_local_state::ImportSavedListItem {
                                record_key: (*key).into(),
                                note: None,
                                record_title_snapshot: "Saved record".into(),
                                record_kind_snapshot: None,
                            })
                            .collect(),
                        tags: vec![],
                        replace: false,
                    },
                )?)
            })
            .unwrap();
        let context = FilterDiscoveryContext::SavedList {
            search_content: None,
            search_profile: None,
            expected_artifact_generation: Some(generation.clone()),
            list_ref: slug.into(),
            filter: Some(predicate(constraint.clone())),
            text: None,
            mode: RetrievalModeView::Lexical,
        };
        assert_eq!(
            count_values(
                f.worker
                    .discover_filter_counts(counts_request(context.clone()), &execution)
                    .unwrap()
            ),
            if slug == "linked" {
                vec![1, 0]
            } else {
                vec![0, 0]
            }
        );
        let values = f
            .worker
            .discover_filter_values(values_request(context), &execution)
            .unwrap();
        let FilterValueListView::Contextual {
            outcome: QueryFacetOutcome::Available { value },
            ..
        } = values
        else {
            panic!("saved relationship values unavailable")
        };
        assert_eq!(value.options.len(), usize::from(slug == "linked"));
    }
    let mut text_context = filtered(constraint, &generation);
    let FilterDiscoveryContext::Filtered { text, .. } = &mut text_context else {
        unreachable!()
    };
    *text = Some("Guard".into());
    let values = f
        .worker
        .discover_filter_values(values_request(text_context.clone()), &execution)
        .unwrap();
    let FilterValueListView::Contextual {
        outcome: QueryFacetOutcome::Available { value },
        ..
    } = values
    else {
        panic!("lexical relationship values unavailable")
    };
    assert_eq!(
        value.options[0].distinct_roots, 1,
        "unlinked text matches must remain excluded"
    );
    assert!(matches!(
        f.worker
            .discover_filter_counts(counts_request(text_context), &execution)
            .unwrap()
            .counts,
        QueryFacetOutcome::Unavailable {
            reason: atlas_domain::FacetUnavailableReason::TextContext
        }
    ));
}

#[test]
fn relationship_discovery_checks_generation_and_selection_before_semantic_inference() {
    let f = fixture();
    let execution = ReadExecutionContext::uncancellable();
    let (constraint, generation) = relation(&f);
    let stale = filtered(constraint.clone(), "stale-generation");
    assert_eq!(
        f.worker
            .discover_filter_counts(counts_request(stale.clone()), &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::ArtifactGenerationChanged
    );
    assert_eq!(
        f.worker
            .discover_filter_values(values_request(stale), &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::ArtifactGenerationChanged
    );
    let mut invalid = constraint;
    invalid.selection.owners = vec![atlas_domain::OwnedContentLocator {
        collection: "/items".into(),
        identity: atlas_domain::OwnedContentIdentity::SnapshotLocal { index: 0 },
    }];
    invalid.selection.source_fingerprint = Some("old-fingerprint".into());
    let mut context = filtered(invalid, &generation);
    let FilterDiscoveryContext::Filtered { text, mode, .. } = &mut context else {
        unreachable!()
    };
    *text = Some("needs unavailable query inference".into());
    *mode = RetrievalModeView::Semantic;
    assert_eq!(
        f.worker
            .discover_filter_values(values_request(context.clone()), &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    assert_eq!(
        f.worker
            .discover_filter_counts(counts_request(context), &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
}

#[test]
fn relationship_editor_labels_negative_and_unmatched_leaves_without_result_rows() {
    let f = fixture();
    let (relationship, generation) = relation(&f);
    let filter = atlas_domain::QueryPredicate::new(atlas_domain::QueryExpression::Not {
        predicate: Box::new(predicate(relationship.clone())),
    });
    let view = f
        .worker
        .discover_filter_editor(DiscoverFilterEditorRequest {
            filter: Some(filter),
            selected_field_ids: vec![],
            expected_artifact_generation: Some(generation.clone()),
        })
        .unwrap();
    assert_eq!(view.artifact_generation, generation);
    assert!(
        matches!(&view.relationship_clauses[..],[RelationshipClausePresentationView::Available {clause_id,title,..}] if clause_id=="link" && title=="Leather Armor")
    );
    let mut missing = relationship;
    missing.selection.record_key =
        atlas_domain::RecordKey::parse("items:missing000000000").unwrap();
    let invalid = f
        .worker
        .discover_filter_editor(DiscoverFilterEditorRequest {
            filter: Some(predicate(missing)),
            selected_field_ids: vec![],
            expected_artifact_generation: Some(generation),
        })
        .unwrap();
    assert!(
        !invalid.groups.is_empty(),
        "catalog remains usable to repair unavailable selections"
    );
    let [
        RelationshipClausePresentationView::Unavailable {
            clause_id, message, ..
        },
    ] = &invalid.relationship_clauses[..]
    else {
        panic!("expected unavailable label")
    };
    assert_eq!(clause_id, "link");
    assert!(!message.contains("missing000000000"));
}
