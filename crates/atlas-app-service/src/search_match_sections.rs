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

fn same_landing(a: &RecordNavigationView, b: &RecordNavigationView) -> bool {
    if !same_entry(a, b) || a.field != b.field {
        return false;
    }
    use SourcePassageAddress::{HtmlSection, PlainSection};
    match (&a.passage, &b.passage) {
        (
            Some(HtmlSection {
                prepared_html_sha256: a_html,
                canonical_text_sha256: a_text,
                selection_version: a_version,
                section_ordinal: a_ordinal,
                label: a_label,
                ..
            }),
            Some(HtmlSection {
                prepared_html_sha256: b_html,
                canonical_text_sha256: b_text,
                selection_version: b_version,
                section_ordinal: b_ordinal,
                label: b_label,
                ..
            }),
        ) => {
            (a_html, a_text, a_version, a_ordinal, a_label)
                == (b_html, b_text, b_version, b_ordinal, b_label)
        }
        (
            Some(PlainSection {
                source_text_sha256: a_text,
                selection_version: a_version,
                section_ordinal: a_ordinal,
                label: a_label,
                ..
            }),
            Some(PlainSection {
                source_text_sha256: b_text,
                selection_version: b_version,
                section_ordinal: b_ordinal,
                label: b_label,
                ..
            }),
        ) => (a_text, a_version, a_ordinal, a_label) == (b_text, b_version, b_ordinal, b_label),
        _ => a.passage == b.passage,
    }
}

fn field_label(field: &str) -> Option<&'static str> {
    match field
        .trim_start_matches('/')
        .trim_start_matches("system/")
        .split('/')
        .next()?
    {
        "description" => Some("Description"),
        "requirements" => Some("Requirements"),
        "prerequisites" => Some("Prerequisites"),
        "trigger" => Some("Trigger"),
        "cost" => Some("Cost"),
        "notes" => Some("Notes"),
        "rules" => Some("Rules"),
        _ => None,
    }
}

fn qualify_distinct_labels(
    sections: &mut [SearchMatchSectionView],
    parent_headings: &[Option<&str>],
) {
    let labels = sections
        .iter()
        .zip(parent_headings)
        .map(|(section, parent)| {
            let same_label = sections.iter().filter(|other| other.label == section.label);
            let has_other_landing = same_label
                .clone()
                .any(|other| !same_landing(&other.navigation, &section.navigation));
            if !has_other_landing {
                return section.label.clone();
            }
            let field = section.navigation.field.as_deref();
            let context = parent.or_else(|| field.and_then(field_label));
            match context {
                Some("Prerequisites")
                    if same_label
                        .clone()
                        .filter(|other| {
                            other.navigation.field.as_deref().and_then(field_label)
                                == Some("Prerequisites")
                                && !same_landing(&other.navigation, &section.navigation)
                        })
                        .count()
                        > 0 =>
                {
                    if let Some(position) = field
                        .and_then(|field| field.strip_prefix("/system/prerequisites/value/"))
                        .and_then(|path| path.split('/').next())
                        .and_then(|index| index.parse::<usize>().ok())
                        .and_then(|index| index.checked_add(1))
                    {
                        format!("{} — Prerequisite {position}", section.label)
                    } else {
                        format!("{} — Prerequisites", section.label)
                    }
                }
                Some(context) if section.label != context => {
                    format!("{} — {context}", section.label)
                }
                _ => section.label.clone(),
            }
        })
        .collect::<Vec<_>>();
    // Authored context can still coincide. Number distinct destinations in their
    // established order, giving all excerpts at the same landing the same name.
    let labels = labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let mut distinct = Vec::new();
            for (other, candidate) in sections.iter().zip(&labels) {
                if candidate == label
                    && !distinct
                        .iter()
                        .any(|nav| same_landing(nav, &other.navigation))
                {
                    distinct.push(other.navigation.clone());
                }
            }
            if distinct.len() <= 1 {
                return label.clone();
            }
            let number = distinct
                .iter()
                .position(|nav| same_landing(nav, &sections[index].navigation))
                .map_or(1, |index| index + 1);
            format!("{label} ({number})")
        })
        .collect::<Vec<_>>();
    for (section, label) in sections.iter_mut().zip(labels) {
        section.label = label;
    }
}

/// Present destinations, not retrieval lanes. A whole-entry hit contributes no
/// second destination when a passage already identifies that same checked entry.
/// Collapse equivalent previews at a checked landing, preserving different evidence.
pub(crate) fn matching_sections(
    witnesses: &[SearchWitnessView],
    parent_headings: &[Option<String>],
) -> Option<Vec<SearchMatchSectionView>> {
    let mut sections: Vec<SearchMatchSectionView> = Vec::new();
    let mut contexts = Vec::new();
    for (index, witness) in witnesses
        .iter()
        .enumerate()
        .filter(|(_, w)| has_location(&w.navigation))
    {
        let (selected_index, selected) = if whole_entry(&witness.navigation) {
            witnesses
                .iter()
                .enumerate()
                .find(|(_, other)| {
                    same_entry(&witness.navigation, &other.navigation)
                        && !whole_entry(&other.navigation)
                        && has_location(&other.navigation)
                })
                .unwrap_or((index, witness))
        } else {
            (index, witness)
        };
        if let Some(section) = sections.iter_mut().find(|s| {
            s.navigation == selected.navigation
                || (s.preview == selected.preview
                    && selected.label.as_deref() == Some(s.label.as_str())
                    && same_landing(&s.navigation, &selected.navigation))
        }) {
            if section.preview.is_none() {
                section.preview = selected.preview.clone();
            }
            continue;
        }
        contexts.push(
            parent_headings
                .get(selected_index)
                .and_then(Option::as_deref),
        );
        sections.push(SearchMatchSectionView {
            navigation: selected.navigation.clone(),
            label: selected
                .label
                .as_deref()
                .filter(|label| !label.trim().is_empty())
                .or(witness.label.as_deref())
                .unwrap_or("Matching passage")
                .to_owned(),
            preview: selected.preview.clone(),
        });
    }
    qualify_distinct_labels(&mut sections, &contexts);
    (!sections.is_empty()).then_some(sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::{MatchPreviewView, SearchLaneView};

    fn matching_sections(witnesses: &[SearchWitnessView]) -> Option<Vec<SearchMatchSectionView>> {
        super::matching_sections(witnesses, &[])
    }

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
            preview: field.map(|_| MatchPreviewView::Text {
                blocks: vec!["A cone of fire.".into()],
                truncated: false,
            }),
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
        assert!(
            matches!(&sections[0].preview, Some(MatchPreviewView::Text { blocks, .. }) if blocks == &["A cone of fire."])
        );
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
        b.preview = None;
        let sections = matching_sections(&[b, a]).unwrap();
        assert_eq!(sections.len(), 1);
        assert!(
            matches!(&sections[0].preview, Some(MatchPreviewView::Text { blocks, .. }) if blocks == &["A cone of fire."])
        );
    }
    #[test]
    fn equivalent_windows_collapse_without_discarding_distinct_evidence() {
        let mut first = hit(
            "",
            Some("/system/description/value"),
            SearchLaneView::Semantic,
        );
        first.navigation.passage = Some(SourcePassageAddress::HtmlSection {
            prepared_html_sha256: "html".into(),
            canonical_text_sha256: "text".into(),
            selection_version: "selection".into(),
            section_ordinal: 0,
            label: None,
            chunk_bytes: atlas_domain::SourceByteRange { start: 0, end: 100 },
        });
        let mut equivalent = first.clone();
        if let Some(SourcePassageAddress::HtmlSection { chunk_bytes, .. }) =
            &mut equivalent.navigation.passage
        {
            *chunk_bytes = atlas_domain::SourceByteRange {
                start: 50,
                end: 120,
            };
        }
        let mut other = equivalent.clone();
        if let Some(SourcePassageAddress::HtmlSection { chunk_bytes, .. }) =
            &mut other.navigation.passage
        {
            *chunk_bytes = atlas_domain::SourceByteRange {
                start: 95,
                end: 150,
            };
        }
        other.preview = Some(MatchPreviewView::Table {
            columns: vec!["Type".into()],
        });
        let sections = matching_sections(&[first.clone(), equivalent, other.clone()]).unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].navigation, first.navigation);
        assert_eq!(sections[1].navigation, other.navigation);
    }

    #[test]
    fn different_fields_supply_context_for_repeated_labels() {
        let mut requirements = hit("", Some("/system/requirements"), SearchLaneView::Lexical);
        let mut prerequisites = hit(
            "",
            Some("/system/prerequisites/value/0/value"),
            SearchLaneView::Semantic,
        );
        requirements.label = Some("Shifting Terrain".into());
        prerequisites.label = requirements.label.clone();
        let sections = matching_sections(&[requirements, prerequisites]).unwrap();
        assert_eq!(
            sections
                .iter()
                .map(|section| section.label.as_str())
                .collect::<Vec<_>>(),
            [
                "Shifting Terrain — Requirements",
                "Shifting Terrain — Prerequisites"
            ]
        );
    }

    #[test]
    fn distinct_prerequisites_and_nested_headings_get_distinct_labels() {
        let first = hit(
            "",
            Some("/system/prerequisites/value/0/value"),
            SearchLaneView::Lexical,
        );
        let mut second = first.clone();
        second.navigation.field = Some("/system/prerequisites/value/1/value".into());
        let sections = matching_sections(&[first, second]).unwrap();
        assert_eq!(
            sections
                .iter()
                .map(|s| s.label.as_str())
                .collect::<Vec<_>>(),
            [
                "Breath Fire — Prerequisite 1",
                "Breath Fire — Prerequisite 2"
            ]
        );
        let mut first = hit(
            "",
            Some("/system/description/value"),
            SearchLaneView::Semantic,
        );
        first.navigation.passage = Some(SourcePassageAddress::HtmlSection {
            prepared_html_sha256: "html".into(),
            canonical_text_sha256: "text".into(),
            selection_version: "selection".into(),
            section_ordinal: 1,
            label: None,
            chunk_bytes: atlas_domain::SourceByteRange { start: 0, end: 100 },
        });
        let mut second = first.clone();
        if let Some(SourcePassageAddress::HtmlSection {
            section_ordinal, ..
        }) = &mut second.navigation.passage
        {
            *section_ordinal = 3;
        }
        let sections = super::matching_sections(
            &[first, second],
            &[Some("First Form".into()), Some("Second Form".into())],
        )
        .unwrap();
        assert_eq!(
            sections
                .iter()
                .map(|s| s.label.as_str())
                .collect::<Vec<_>>(),
            ["Breath Fire — First Form", "Breath Fire — Second Form"]
        );
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
