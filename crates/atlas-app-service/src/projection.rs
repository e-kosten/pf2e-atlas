//! Product DTO conversion carries selected facts without source provenance or developer reports.
use atlas_app_model::{
    RecordBadgeView, RecordNavigationView, RecordSummaryView, SearchCoverageView, SearchLaneView,
    SearchPageView, SearchWitnessView,
};
use atlas_domain::{SourceLevelBasis, SourceRecordSummary};
use atlas_search::{SearchCoverage, SearchLane, SearchPageInfo, SearchWitness};

pub fn record_summary_view(record: &SourceRecordSummary) -> RecordSummaryView {
    let kind = record
        .record_kind
        .as_value()
        .map(|k| k.as_str().to_owned())
        .unwrap_or_else(|| record.document_kind.to_lowercase());
    RecordSummaryView {
        record_key: record.key.to_string(),
        title: record
            .name
            .as_value()
            .cloned()
            .unwrap_or_else(|| record.key.to_string()),
        kind_label: kind_label(&kind),
        kind,
        source_type: record.source_type.as_value().cloned(),
        level_label: record.level.as_value().map(|level| {
            format!(
                "{} {level}",
                if record.level_basis == Some(SourceLevelBasis::SpellRank) {
                    "Rank"
                } else {
                    "Level"
                }
            )
        }),
        level_basis: record.level_basis.map(|basis| {
            match basis {
                SourceLevelBasis::ActorLevel => "actor_level",
                SourceLevelBasis::ItemLevel => "item_level",
                SourceLevelBasis::SpellRank => "spell_rank",
            }
            .into()
        }),
        rarity: record.rarity.as_value().cloned(),
        traits: record
            .traits
            .as_value()
            .into_iter()
            .flatten()
            .map(|value| RecordBadgeView {
                kind: "trait".into(),
                label: value.clone(),
                value: value.clone(),
            })
            .collect(),
        publication: record.publication_title.as_value().cloned(),
        pack: Some(record.pack_label.clone()),
    }
}
pub(crate) fn search_page_view(page: SearchPageInfo) -> SearchPageView {
    SearchPageView {
        number: page.number,
        size: page.size,
        count: page.count,
        total: page.total,
        has_more: page.has_more,
        next_page: page.next_page,
    }
}
pub(crate) fn search_witness_view(witness: SearchWitness, fingerprint: &str) -> SearchWitnessView {
    SearchWitnessView {
        navigation: RecordNavigationView {
            record_key: witness.location.record.to_string(),
            owners: witness.location.owners.clone(),
            field: witness.location.field,
            source_fingerprint: navigation_fingerprint(&witness.location.owners, fingerprint),
            passage: witness.location.address,
        },
        lane: match witness.lane {
            SearchLane::Lexical => SearchLaneView::Lexical,
            SearchLane::Semantic => SearchLaneView::Semantic,
        },
        label: witness.label,
        snippet: witness.snippet,
        lexical_rank: witness.lexical_rank,
        semantic_similarity: witness.semantic_similarity,
    }
}
pub(crate) fn coverage_view(coverage: SearchCoverage) -> SearchCoverageView {
    SearchCoverageView {
        exhaustive: coverage.exhaustive,
        semantic_unit_window: coverage.semantic_unit_window,
        candidate_roots: coverage.candidate_roots,
        count_basis: coverage.count_basis,
    }
}
pub(crate) fn kind_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn graph_section_view(
    section: atlas_search::GraphContextSection,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> atlas_app_model::GraphSectionView {
    atlas_app_model::GraphSectionView {
        records: section
            .records
            .iter()
            .map(|s| localized_summary_view(s, retrieval))
            .collect(),
        occurrences: section
            .occurrences
            .into_iter()
            .map(|o| relationship_view(o, retrieval.source_fingerprint()))
            .collect(),
        truncated: section.truncated,
    }
}
pub(crate) fn search_results_view(
    result: atlas_search::TextSearchResult,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> atlas_app_model::SearchResultsView {
    atlas_app_model::SearchResultsView {
        rows: result
            .records
            .into_iter()
            .map(|r| atlas_app_model::ResultWindowRow {
                record: localized_summary_view(&r.record, retrieval),
                matches: r
                    .matches
                    .into_iter()
                    .map(|w| search_witness_view(w, retrieval.source_fingerprint()))
                    .collect(),
            })
            .collect(),
        page: search_page_view(result.page),
        coverage: coverage_view(result.coverage),
    }
}
pub(crate) fn relationship_view(
    o: atlas_search::SourceStoredRelationship,
    fingerprint: &str,
) -> atlas_app_model::RecordRelationshipView {
    use atlas_record::source_content::{
        ContentReferenceResolution as Resolution, ContentReferenceTarget as Target,
    };
    let (status, target, url) = match o.resolution {
        Resolution::Resolved(Target::Record { key }) => (
            "resolved_record",
            Some(RecordNavigationView {
                record_key: key.to_string(),
                owners: vec![],
                field: None,
                passage: None,
                source_fingerprint: None,
            }),
            None,
        ),
        Resolution::Resolved(Target::OwnedNode { key, owners }) => (
            "resolved_owned",
            Some(RecordNavigationView {
                record_key: key.to_string(),
                source_fingerprint: navigation_fingerprint(&owners, fingerprint),
                owners,
                field: None,
                passage: None,
            }),
            None,
        ),
        Resolution::Resolved(Target::Url { url }) => ("resolved_url", None, Some(url)),
        Resolution::UnverifiedUrl { url } => ("unverified_url", None, Some(url)),
        Resolution::Unresolved => ("unresolved", None, None),
        Resolution::Blocked => ("blocked", None, None),
    };
    atlas_app_model::RecordRelationshipView {
        source: o.locator,
        ordinal: o.ordinal,
        origin: o.origin,
        kind: o.kind,
        authored_target: o.authored_target,
        occurrence_path: o.occurrence_path,
        audiences: o.details.audiences,
        availability: o.details.availability.map(field_state),
        status: status.into(),
        target,
        url,
    }
}
pub(crate) fn field_state(
    state: atlas_record::source_record::FieldAvailability,
) -> atlas_domain::QueryFieldState {
    use atlas_domain::QueryFieldState as Q;
    use atlas_record::source_record::FieldAvailability as A;
    match state {
        A::Value => Q::Value,
        A::Missing => Q::Missing,
        A::Null => Q::Null,
        A::Invalid { .. } => Q::Invalid,
        A::NotApplicable => Q::NotApplicable,
    }
}

pub(crate) fn navigation_fingerprint(
    owners: &[atlas_record::source_content::OwnedContentLocator],
    fingerprint: &str,
) -> Option<String> {
    owners
        .iter()
        .any(|o| {
            matches!(
                o.identity,
                atlas_record::source_content::OwnedContentIdentity::SnapshotLocal { .. }
            )
        })
        .then(|| fingerprint.to_owned())
}

/// Labels use the verified indexing locale while identifiers remain authored values.
pub(crate) fn localized_summary_view(
    record: &SourceRecordSummary,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> RecordSummaryView {
    let mut view = record_summary_view(record);
    for badge in &mut view.traits {
        if let Some(label) = retrieval.trait_label(&badge.value) {
            badge.label = label.into();
        }
    }
    view
}

pub(crate) fn resolution_view(
    v: atlas_search::RecordResolutionResult,
    r: &atlas_search::AtlasRetrievalService,
) -> atlas_app_model::RecordResolutionCandidateView {
    atlas_app_model::RecordResolutionCandidateView {
        record: localized_summary_view(&v.record, r),
        query: v.query,
        normalized_query: v.normalized_query,
        match_kind: match v.match_kind {
            atlas_search::RecordResolutionMatchKind::Name => "name",
            atlas_search::RecordResolutionMatchKind::NormalizedName => "normalized_name",
            atlas_search::RecordResolutionMatchKind::Alias => "alias",
        }
        .into(),
        matched_text: v.matched_text,
        evidence: v.evidence,
    }
}
