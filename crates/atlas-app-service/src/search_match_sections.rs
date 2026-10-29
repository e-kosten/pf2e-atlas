use atlas_app_model::{RecordNavigationView, SearchMatchSectionView, SearchWitnessView};
use atlas_domain::SourcePassageAddress;

fn same_entry(a: &RecordNavigationView, b: &RecordNavigationView) -> bool {
    a.record_key == b.record_key
        && a.owners == b.owners
        && a.source_fingerprint == b.source_fingerprint
        && a.spell_rank == b.spell_rank
}

fn has_location(navigation: &RecordNavigationView) -> bool {
    !navigation.owners.is_empty()
        || navigation
            .field
            .as_deref()
            .is_some_and(|field| field != "identity")
        || navigation
            .passage
            .as_ref()
            .is_some_and(|passage| !matches!(passage, SourcePassageAddress::Identity {}))
}

fn whole_entry(navigation: &RecordNavigationView) -> bool {
    navigation
        .field
        .as_deref()
        .is_none_or(|field| field == "identity")
        && navigation
            .passage
            .as_ref()
            .is_none_or(|passage| matches!(passage, SourcePassageAddress::Identity {}))
}

/// Present destinations, not retrieval lanes. A whole-entry hit contributes no
/// second destination when a passage already identifies that same checked entry.
/// Keep distinct passages and owner identities, even when their labels coincide.
pub(crate) fn matching_sections(
    witnesses: &[SearchWitnessView],
) -> Option<Vec<SearchMatchSectionView>> {
    let mut sections: Vec<SearchMatchSectionView> = Vec::new();
    for witness in witnesses.iter().filter(|w| has_location(&w.navigation)) {
        let selected = if whole_entry(&witness.navigation) {
            witnesses
                .iter()
                .find(|other| {
                    same_entry(&witness.navigation, &other.navigation)
                        && !whole_entry(&other.navigation)
                        && has_location(&other.navigation)
                })
                .unwrap_or(witness)
        } else {
            witness
        };
        if let Some(section) = sections
            .iter_mut()
            .find(|s| s.navigation == selected.navigation)
        {
            if section.snippet.is_none() {
                section.snippet = selected.snippet.clone();
            }
            continue;
        }
        sections.push(SearchMatchSectionView {
            navigation: selected.navigation.clone(),
            label: selected
                .label
                .as_deref()
                .filter(|label| !label.trim().is_empty())
                .or(witness.label.as_deref())
                .unwrap_or("Matching passage")
                .to_owned(),
            snippet: selected.snippet.clone(),
        });
    }
    (!sections.is_empty()).then_some(sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::SearchLaneView;

    fn hit(owner: &str, field: Option<&str>, lane: SearchLaneView) -> SearchWitnessView {
        let navigation = serde_json::from_value(serde_json::json!({
            "record_key": "actors:firewyrm",
            "embedded_path": if owner.is_empty() { vec![] } else {
                vec![serde_json::json!({"collection":"/items", "identity":{"Stable":owner}})]
            },
            "field": field,
            "passage": null,
            "source_fingerprint": null,
        }))
        .unwrap();
        SearchWitnessView {
            navigation,
            lane,
            label: Some("Breath Fire".into()),
            snippet: field.map(|_| "A cone of fire.".into()),
            lexical_rank: None,
            semantic_similarity: None,
        }
    }

    #[test]
    fn coalesces_whole_entry_and_passage_lanes_without_changing_witnesses() {
        let broad = hit("firewyrm", None, SearchLaneView::Lexical);
        let precise = hit("firewyrm", Some("description"), SearchLaneView::Semantic);
        let input = vec![broad.clone(), precise.clone()];
        let sections = matching_sections(&input).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].navigation, precise.navigation);
        assert_eq!(sections[0].snippet.as_deref(), Some("A cone of fire."));
        assert_eq!(input, vec![broad, precise]);
    }

    #[test]
    fn preserves_distinct_entries_and_fields_with_the_same_label() {
        let a = hit("first", Some("description"), SearchLaneView::Lexical);
        let b = hit("second", Some("description"), SearchLaneView::Semantic);
        let c = hit("first", Some("notes"), SearchLaneView::Semantic);
        let input = [a, b, c];
        assert_eq!(
            matching_sections(&input)
                .unwrap()
                .iter()
                .map(|section| &section.navigation)
                .collect::<Vec<_>>(),
            input.iter().map(|hit| &hit.navigation).collect::<Vec<_>>()
        );
    }

    #[test]
    fn identity_only_hits_do_not_create_a_search_section() {
        assert!(matching_sections(&[hit("", None, SearchLaneView::Lexical)]).is_none());
    }

    #[test]
    fn identical_destinations_keep_the_available_excerpt_once() {
        let a = hit("firewyrm", Some("description"), SearchLaneView::Lexical);
        let mut b = a.clone();
        b.snippet = None;
        let sections = matching_sections(&[b, a]).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].snippet.as_deref(), Some("A cone of fire."));
    }
    #[test]
    fn explicit_identity_passages_do_not_create_a_section() {
        let mut identity = hit("", Some("identity"), SearchLaneView::Semantic);
        identity.navigation.passage = Some(SourcePassageAddress::Identity {});
        assert!(matching_sections(&[identity]).is_none());
    }

    #[test]
    fn source_contexts_and_distinct_passages_are_not_coalesced() {
        let a = hit("same-item", Some("description"), SearchLaneView::Semantic);
        let mut b = a.clone();
        b.navigation.source_fingerprint = Some("other-source".into());
        let mut first = a.clone();
        first.navigation.passage = Some(
            serde_json::from_value(serde_json::json!({
                "kind":"plain_section", "source_text_sha256":"source", "selection_version":"v1",
                "section_ordinal":0, "label":null, "chunk_bytes":{"start":0,"end":10}
            }))
            .unwrap(),
        );
        let mut second = first.clone();
        second.navigation.passage = Some(
            serde_json::from_value(serde_json::json!({
                "kind":"plain_section", "source_text_sha256":"source", "selection_version":"v1",
                "section_ordinal":1, "label":null, "chunk_bytes":{"start":10,"end":20}
            }))
            .unwrap(),
        );
        let input = [a, b, first, second];
        assert_eq!(
            matching_sections(&input)
                .unwrap()
                .iter()
                .map(|section| &section.navigation)
                .collect::<Vec<_>>(),
            input.iter().map(|hit| &hit.navigation).collect::<Vec<_>>()
        );
    }
}
