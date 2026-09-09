use atlas_app_model::{
    AppErrorCode, RecordSurfaceReferenceEdgeView, RecordSurfaceReferenceRecordView,
    RecordSurfaceReferenceSectionView, RecordSurfaceReferenceSourceView,
    RecordSurfaceReferencesView,
};
use atlas_search::{
    GraphContextResult, GraphContextSection, MAX_GRAPH_CONTEXT_LIMIT, SearchError, SearchErrorKind,
};

pub(crate) fn project_record_references(
    result: GraphContextResult,
    outgoing_limit: u8,
    backlink_limit: u8,
) -> RecordSurfaceReferencesView {
    RecordSurfaceReferencesView {
        outgoing: project_section(result.outgoing, outgoing_limit),
        backlinks: project_section(result.backlinks, backlink_limit),
    }
}

pub(crate) fn unavailable_record_references(
    outgoing_limit: u8,
    backlink_limit: u8,
    error: &SearchError,
) -> RecordSurfaceReferencesView {
    let code = app_error_code(error.kind());
    RecordSurfaceReferencesView {
        outgoing: unavailable_section(outgoing_limit, code),
        backlinks: unavailable_section(backlink_limit, code),
    }
}

pub(crate) fn missing_record_references(
    outgoing_limit: u8,
    backlink_limit: u8,
) -> RecordSurfaceReferencesView {
    RecordSurfaceReferencesView {
        outgoing: unavailable_section(outgoing_limit, AppErrorCode::QueryFailed),
        backlinks: unavailable_section(backlink_limit, AppErrorCode::QueryFailed),
    }
}

pub(crate) fn not_requested_record_references() -> RecordSurfaceReferencesView {
    RecordSurfaceReferencesView {
        outgoing: RecordSurfaceReferenceSectionView::NotRequested,
        backlinks: RecordSurfaceReferenceSectionView::NotRequested,
    }
}

fn project_section(
    section: GraphContextSection,
    requested_limit: u8,
) -> RecordSurfaceReferenceSectionView {
    if requested_limit == 0 {
        return RecordSurfaceReferenceSectionView::NotRequested;
    }

    let Some(total_records) = u32::try_from(section.total_records).ok() else {
        return unavailable_section(requested_limit, AppErrorCode::QueryFailed);
    };
    let Some(total_edges) = u32::try_from(section.total_edges).ok() else {
        return unavailable_section(requested_limit, AppErrorCode::QueryFailed);
    };
    RecordSurfaceReferenceSectionView::Available {
        requested_limit,
        next_limit: next_reference_limit(requested_limit, total_records, section.truncated),
        records: section
            .records
            .into_iter()
            .map(|record| RecordSurfaceReferenceRecordView {
                record_key: record.record.identity.key.to_string(),
                title: record.record.identity.name,
                kind: record.record.classification.kind.as_str().to_string(),
            })
            .collect(),
        edges: section
            .edges
            .into_iter()
            .map(|edge| RecordSurfaceReferenceEdgeView {
                from_record_key: edge.from.to_string(),
                to_record_key: edge.to.to_string(),
                source_child_locator: edge
                    .source_child
                    .as_ref()
                    .map(atlas_record::encode_content_child_selector),
                target_child_locator: edge
                    .target_child
                    .as_ref()
                    .map(atlas_record::encode_content_child_selector),
                display_text: edge.display_text,
                reference_text: edge.reference_text,
                source: RecordSurfaceReferenceSourceView {
                    kind: edge.source.kind,
                    visibility: edge.source.visibility,
                    relation_kind: edge.source.relation_kind,
                },
            })
            .collect(),
        total_records,
        total_edges,
        truncated: section.truncated,
    }
}

fn next_reference_limit(requested_limit: u8, total_records: u32, truncated: bool) -> Option<u8> {
    let cap = u8::try_from(MAX_GRAPH_CONTEXT_LIMIT).ok()?;
    if !truncated || u32::from(requested_limit) >= total_records || requested_limit >= cap {
        return None;
    }
    Some(requested_limit.saturating_mul(2).min(cap))
}

fn unavailable_section(
    requested_limit: u8,
    code: AppErrorCode,
) -> RecordSurfaceReferenceSectionView {
    if requested_limit == 0 {
        RecordSurfaceReferenceSectionView::NotRequested
    } else {
        RecordSurfaceReferenceSectionView::Unavailable {
            requested_limit,
            code,
            message: "Record references are temporarily unavailable.".to_string(),
        }
    }
}

fn app_error_code(kind: SearchErrorKind) -> AppErrorCode {
    match kind {
        SearchErrorKind::IndexUnavailable => AppErrorCode::IndexUnavailable,
        SearchErrorKind::ArtifactContractViolation => AppErrorCode::ArtifactIncompatible,
        SearchErrorKind::InvalidFilter => AppErrorCode::FilterInvalid,
        SearchErrorKind::InvalidOptions => AppErrorCode::InvalidRequest,
        SearchErrorKind::VectorReadinessRequired => AppErrorCode::VectorReadinessRequired,
        SearchErrorKind::EmbeddingUnavailable => AppErrorCode::EmbeddingModelUnavailable,
        SearchErrorKind::QueryFailed => AppErrorCode::QueryFailed,
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::RecordSurfaceReferenceSectionView;
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        ContentChildIdentity, ContentChildKind, ContentChildLocator, SourceDocumentId,
    };
    use atlas_search::{
        GraphContextEdge, GraphContextEdgeSource, GraphContextResult, GraphContextSection,
        SearchError,
    };

    use super::*;
    use crate::test_support::encounter_fixture_worker;

    #[test]
    fn reference_expansion_stops_at_the_product_cap() {
        for (current, next) in [(8, Some(16)), (16, Some(32)), (32, Some(50)), (50, None)] {
            assert_eq!(super::next_reference_limit(current, 75, true), next);
        }
        assert_eq!(super::next_reference_limit(8, 8, false), None);
        assert_eq!(super::next_reference_limit(16, 12, true), None);
    }

    #[test]
    fn record_references_preserve_graph_order_totals_and_truncation_for_every_family() {
        let fixture = encounter_fixture_worker();
        let mut fixture_record = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");

        for (kind, seed_key) in [
            (RecordKind::Creature, "actors:referenceSeed"),
            (RecordKind::Hazard, "hazards:referenceSeed"),
            (RecordKind::Spell, "spells:referenceSeed"),
        ] {
            fixture_record.record.classification.kind = kind;
            fixture_record.record.identity.key =
                RecordKey::parse(seed_key).expect("seed key should parse");
            let mut neighbors = Vec::new();
            for index in 0..8 {
                let mut neighbor = fixture_record.clone();
                neighbor.record.identity.key =
                    RecordKey::parse(&format!("actions:neighbor{index}"))
                        .expect("neighbor key should parse");
                neighbor.record.identity.name = format!("Neighbor {index}");
                neighbors.push(neighbor);
            }
            let expected_keys = neighbors
                .iter()
                .map(|record| record.record.identity.key.to_string())
                .collect::<Vec<_>>();
            let edges = neighbors
                .iter()
                .map(|record| GraphContextEdge {
                    from: fixture_record.record.identity.key.clone(),
                    to: record.record.identity.key.clone(),
                    source_child: None,
                    target_child: None,
                    display_text: Some(record.record.identity.name.clone()),
                    reference_text: record.record.identity.name.clone(),
                    source: GraphContextEdgeSource {
                        kind: "content_reference".to_string(),
                        visibility: "public".to_string(),
                        relation_kind: "mentions".to_string(),
                    },
                })
                .collect::<Vec<_>>();
            let result = GraphContextResult {
                seed: fixture_record.clone(),
                outgoing: GraphContextSection {
                    records: neighbors,
                    edges,
                    total_records: 9,
                    total_edges: 10,
                    truncated: true,
                },
                backlinks: empty_section(),
            };

            let projected = project_record_references(result, 8, 0);
            let RecordSurfaceReferenceSectionView::Available {
                requested_limit,
                records,
                edges,
                total_records,
                total_edges,
                truncated,
                next_limit,
            } = projected.outgoing
            else {
                panic!("outgoing references should be available");
            };
            assert_eq!(requested_limit, 8);
            assert_eq!(next_limit, Some(16));
            assert_eq!(
                records
                    .iter()
                    .map(|record| record.record_key.clone())
                    .collect::<Vec<_>>(),
                expected_keys
            );
            assert_eq!(
                edges
                    .iter()
                    .map(|edge| edge.to_record_key.clone())
                    .collect::<Vec<_>>(),
                expected_keys
            );
            assert_eq!(total_records, 9);
            assert_eq!(total_edges, 10);
            assert!(truncated);
            assert_eq!(
                projected.backlinks,
                RecordSurfaceReferenceSectionView::NotRequested
            );
        }
    }

    #[test]
    fn record_references_preserve_distinct_child_edges_to_one_parent_record() {
        let fixture = encounter_fixture_worker();
        let mut seed = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");
        seed.record.identity.key = RecordKey::parse("tables:seed").expect("seed key");
        seed.record.classification.kind = RecordKind::RollTable;
        let mut target = seed.clone();
        target.record.identity.key = RecordKey::parse("journals:target").expect("target key");
        target.record.identity.name = "Target journal".to_string();
        target.record.classification.kind = RecordKind::Journal;
        let seed_key = seed.record.identity.key.clone();
        let target_key = target.record.identity.key.clone();
        let child = |id: &str| ContentChildLocator {
            parent: target_key.clone(),
            kind: ContentChildKind::JournalPage,
            identity: ContentChildIdentity::Stable(
                SourceDocumentId::new(id).expect("child source ID"),
            ),
        };
        let edge = |id: &str, label: &str| GraphContextEdge {
            from: seed_key.clone(),
            to: target_key.clone(),
            source_child: None,
            target_child: Some(child(id)),
            display_text: Some(label.to_string()),
            reference_text: label.to_string(),
            source: GraphContextEdgeSource {
                kind: "table_result".to_string(),
                visibility: "public".to_string(),
                relation_kind: "reference".to_string(),
            },
        };

        let projected = project_record_references(
            GraphContextResult {
                seed,
                outgoing: GraphContextSection {
                    records: vec![target],
                    edges: vec![
                        edge("first-page", "First page"),
                        edge("second-page", "Second page"),
                    ],
                    total_records: 1,
                    total_edges: 2,
                    truncated: false,
                },
                backlinks: empty_section(),
            },
            8,
            0,
        );
        let RecordSurfaceReferenceSectionView::Available {
            records,
            edges,
            total_records,
            total_edges,
            truncated,
            next_limit,
            ..
        } = projected.outgoing
        else {
            panic!("outgoing references should be available");
        };
        assert_eq!(records.len(), 1);
        assert_eq!(edges.len(), 2);
        assert_ne!(edges[0].target_child_locator, edges[1].target_child_locator);
        assert_eq!(total_records, 1);
        assert_eq!(total_edges, 2);
        assert!(!truncated);
        assert_eq!(next_limit, None);
    }

    #[test]
    fn references_distinguish_empty_explicit_backlinks_errors_and_not_requested() {
        let fixture = encounter_fixture_worker();
        let seed = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");
        let empty = project_record_references(
            GraphContextResult {
                seed,
                outgoing: empty_section(),
                backlinks: empty_section(),
            },
            8,
            8,
        );
        assert!(matches!(
            empty.outgoing,
            RecordSurfaceReferenceSectionView::Available {
                requested_limit: 8,
                total_records: 0,
                total_edges: 0,
                truncated: false,
                ..
            }
        ));
        assert!(matches!(
            empty.backlinks,
            RecordSurfaceReferenceSectionView::Available {
                requested_limit: 8,
                total_records: 0,
                total_edges: 0,
                truncated: false,
                ..
            }
        ));

        let unavailable = unavailable_record_references(
            8,
            0,
            &SearchError::index_unavailable("private index detail"),
        );
        assert!(matches!(
            unavailable.outgoing,
            RecordSurfaceReferenceSectionView::Unavailable {
                requested_limit: 8,
                code: AppErrorCode::IndexUnavailable,
                ref message,
            } if message == "Record references are temporarily unavailable."
        ));
        assert_eq!(
            unavailable.backlinks,
            RecordSurfaceReferenceSectionView::NotRequested
        );
        let serialized = serde_json::to_value(unavailable).expect("references should serialize");
        assert!(serialized["backlinks"].get("total_records").is_none());
        assert!(
            serialized
                .to_string()
                .find("private index detail")
                .is_none()
        );
    }

    fn empty_section() -> GraphContextSection {
        GraphContextSection {
            records: Vec::new(),
            edges: Vec::new(),
            total_records: 0,
            total_edges: 0,
            truncated: false,
        }
    }
}
