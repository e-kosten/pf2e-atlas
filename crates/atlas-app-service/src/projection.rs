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
                trait_key: atlas_domain::TraitKey::new(value.clone())
                    .ok()
                    .map(|key| key.to_string()),
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
            spell_rank: None,
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
    // These source document/family tokens are finite upstream names, not product IDs.
    // Preserve the display boundary while keeping canonical kind/source tokens exact.
    let value = match value {
        "campaignFeature" => "campaign_feature",
        "TableResult" => "table_result",
        "JournalEntryPage" => "journal_entry_page",
        "JournalEntry" => "journal_entry",
        "RollTable" => "roll_table",
        "pdf" => return "PDF".into(),
        _ => value,
    };
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
        search_profile: result.search_profile,
        artifact_generation: result.artifact_generation,
        rows: result
            .records
            .into_iter()
            .map(|r| atlas_app_model::SearchResultRow {
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
pub(crate) fn content_reference_destination(
    resolution: atlas_record::source_content::ContentReferenceResolution,
    fingerprint: &str,
) -> (&'static str, Option<RecordNavigationView>, Option<String>) {
    use atlas_record::source_content::{
        ContentReferenceResolution as Resolution, ContentReferenceTarget as Target,
    };
    match resolution {
        Resolution::Resolved(Target::Record { key }) => (
            "resolved_record",
            Some(RecordNavigationView {
                spell_rank: None,
                record_key: key.to_string(),
                owners: vec![],
                field: None,
                passage: None,
                source_fingerprint: None,
            }),
            None,
        ),
        Resolution::Resolved(Target::OwnedNode { key, owners }) => (
            "resolved_embedded",
            Some(RecordNavigationView {
                spell_rank: None,
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
    }
}

pub(crate) fn relationship_view(
    o: atlas_search::SourceStoredRelationship,
    fingerprint: &str,
) -> atlas_app_model::RecordRelationshipView {
    let (status, target, url) = content_reference_destination(o.resolution, fingerprint);
    atlas_app_model::RecordRelationshipView {
        source_navigation: RecordNavigationView {
            spell_rank: None,
            record_key: o.locator.record.to_string(),
            source_fingerprint: navigation_fingerprint(&o.locator.owners, fingerprint),
            owners: o.locator.owners.clone(),
            field: Some(o.locator.field.clone()),
            passage: None,
        },
        source: o.locator.into(),
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
        effect_label: None,
        contextual_target: None,
    }
}

/// Compact labels are presentation only. Explicit source labels always win.
/// Choices are distinct resolved destinations within the nearest parsed block
/// (and exact field), never inferred from prose, names, or screen proximity.
pub(crate) fn relationship_views(
    occurrences: Vec<atlas_search::SourceStoredRelationship>,
    fingerprint: &str,
) -> Vec<atlas_app_model::RecordRelationshipView> {
    use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
    let eligible = |o: &atlas_search::SourceStoredRelationship| {
        o.details.label.as_ref().is_some_and(|label| {
            !label.explicit
                && label.item_family.as_deref() == Some("effect")
                && label
                    .target_name
                    .as_ref()
                    .is_some_and(|name| !name.is_empty())
        }) && matches!(
            o.resolution,
            ContentReferenceResolution::Resolved(
                ContentReferenceTarget::Record { .. } | ContentReferenceTarget::OwnedNode { .. }
            )
        )
    };
    occurrences
        .iter()
        .map(|o| {
            let mut view = relationship_view(o.clone(), fingerprint);
            if eligible(o)
                && let Some(label) = &o.details.label
            {
                let multiple = occurrences.iter().any(|other| {
                    eligible(other)
                        && other.locator == o.locator
                        && other
                            .details
                            .label
                            .as_ref()
                            .is_some_and(|l| l.block == label.block)
                        && other.resolution != o.resolution
                });
                if let Some(title) = &label.target_name {
                    // This removes only a display prefix after typed classification;
                    // it is never used to decide whether a destination is an effect.
                    let short = title
                        .strip_prefix("Spell Effect: ")
                        .or_else(|| title.strip_prefix("Effect: "))
                        .unwrap_or(title);
                    view.effect_label = Some(atlas_app_model::EffectReferenceLabelView {
                        text: if multiple {
                            if short.is_empty() {
                                title.clone()
                            } else {
                                short.into()
                            }
                        } else {
                            "Effect".into()
                        },
                        title: title.clone(),
                    });
                }
            }
            view
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_record::source_content::{
        ContentReferenceResolution, ContentReferenceTarget, OwnedContentIdentity,
        OwnedContentLocator, SourceContentLocator,
    };

    fn effect_occurrence(id: &str, title: &str) -> atlas_search::SourceStoredRelationship {
        let mut occurrence = atlas_search::SourceStoredRelationship {
            locator: SourceContentLocator {
                record: "spells:aaaaaaaaaaaaaaaa".parse().unwrap(),
                owners: vec![],
                field: "/system/description/value".into(),
            },
            ordinal: 0,
            origin: "content".into(),
            kind: "uuid".into(),
            authored_target: Some(format!("Compendium.pf2e.effects.Item.{id}")),
            occurrence_path: Some("/0/0".into()),
            resolution: ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
                key: format!("effects:{id}").parse().unwrap(),
            }),
            details: Default::default(),
        };
        occurrence.details.label = Some(atlas_record::source_content::ContentReferenceLabel {
            explicit: false,
            block: "/0".into(),
            target_name: Some(title.into()),
            item_family: Some("effect".into()),
        });
        occurrence
    }

    #[test]
    fn compact_effects_distinguish_destinations_only_within_the_same_field_block() {
        let first = effect_occurrence("bbbbbbbbbbbbbbbb", "Spell Effect: Enlarge");
        let mut repeated = first.clone();
        repeated.ordinal = 1;
        let single = relationship_views(vec![first.clone(), repeated.clone()], "snapshot");
        assert!(
            single
                .iter()
                .all(|v| v.effect_label.as_ref().unwrap().text == "Effect")
        );

        let second =
            effect_occurrence("cccccccccccccccc", "Spell Effect: Enlarge (Heightened 4th)");
        let distinct =
            relationship_views(vec![first.clone(), repeated, second.clone()], "snapshot");
        assert_eq!(distinct[0].effect_label.as_ref().unwrap().text, "Enlarge");
        assert_eq!(
            distinct[2].effect_label.as_ref().unwrap().text,
            "Enlarge (Heightened 4th)"
        );
        assert_eq!(
            distinct[0].effect_label.as_ref().unwrap().title,
            "Spell Effect: Enlarge"
        );
        assert_eq!(distinct[0].target, single[0].target);

        for change in 0..3 {
            let mut separate = second.clone();
            match change {
                0 => separate.details.label.as_mut().unwrap().block = "/1".into(),
                1 => separate.locator.field = "/system/details/publicNotes".into(),
                _ => separate.locator.owners.push(OwnedContentLocator {
                    collection: "/items".into(),
                    identity: OwnedContentIdentity::Stable("dddddddddddddddd".into()),
                }),
            }
            let views = relationship_views(vec![first.clone(), separate], "snapshot");
            assert!(
                views
                    .iter()
                    .all(|v| v.effect_label.as_ref().unwrap().text == "Effect")
            );
        }
    }

    #[test]
    fn explicit_unresolved_and_non_effect_references_keep_their_source_labels() {
        let first = effect_occurrence("bbbbbbbbbbbbbbbb", "Spell Effect: Enlarge");
        for change in 0..5 {
            let mut other = effect_occurrence("cccccccccccccccc", "Effect: Not proof of a family");
            match change {
                0 => other.details.label.as_mut().unwrap().explicit = true,
                1 => other.details.label.as_mut().unwrap().item_family = Some("spell".into()),
                2 => other.resolution = ContentReferenceResolution::Unresolved,
                3 => other.details.label.as_mut().unwrap().target_name = None,
                _ => other.details.label = None,
            }
            let views = relationship_views(vec![first.clone(), other], "snapshot");
            assert_eq!(views[0].effect_label.as_ref().unwrap().text, "Effect");
            assert!(views[1].effect_label.is_none());
        }
    }

    #[test]
    fn relationship_source_navigation_preserves_actual_occurrence_not_incoming_seed() {
        let source_key: atlas_domain::RecordKey = "actors:aaaaaaaaaaaaaaaa".parse().unwrap();
        let seed_key: atlas_domain::RecordKey = "items:bbbbbbbbbbbbbbbb".parse().unwrap();
        for owners in [
            vec![],
            vec![OwnedContentLocator {
                collection: "items".into(),
                identity: OwnedContentIdentity::Stable("cccccccccccccccc".into()),
            }],
            vec![OwnedContentLocator {
                collection: "items".into(),
                identity: OwnedContentIdentity::SnapshotLocal { index: 4 },
            }],
        ] {
            for ordinal in [0, 3] {
                let locator = SourceContentLocator {
                    record: source_key.clone(),
                    owners: owners.clone(),
                    field: "/system/description/value".into(),
                };
                let view = relationship_view(
                    atlas_search::SourceStoredRelationship {
                        locator: locator.clone(),
                        ordinal,
                        origin: "content".into(),
                        kind: "uuid".into(),
                        authored_target: None,
                        occurrence_path: Some(format!("/p/{ordinal}")),
                        resolution: ContentReferenceResolution::Resolved(
                            ContentReferenceTarget::Record {
                                key: seed_key.clone(),
                            },
                        ),
                        details: Default::default(),
                    },
                    "current-generation",
                );
                assert_eq!(view.source, locator.into());
                assert_eq!(view.source_navigation.record_key, source_key.to_string());
                assert_eq!(view.source_navigation.owners, owners);
                assert_eq!(
                    view.source_navigation.field.as_deref(),
                    Some("/system/description/value")
                );
                assert_eq!(
                    view.source_navigation.source_fingerprint,
                    navigation_fingerprint(&owners, "current-generation")
                );
                assert!(view.source_navigation.passage.is_none());
                assert_eq!(view.ordinal, ordinal);
                assert_eq!(view.target.unwrap().record_key, seed_key.to_string());
            }
        }
    }
}
