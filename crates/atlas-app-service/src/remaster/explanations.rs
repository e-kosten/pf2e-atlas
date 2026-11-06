//! Bounded readable explanations, separate from diagnostic source evidence.
use super::{ProjectionBudget, navigation, text};
use atlas_app_model::{
    RecordNavigationView, RemasterExplanationLinkView, RemasterRelationshipView,
};
use atlas_domain::{OwnedContentLocator, RecordKey};
use atlas_record::source_content::SourceContentLocator;
use atlas_record::source_record::SourceNodeView;
use atlas_search::{
    AtlasRetrievalService, GetRecordRequest, RemasterCitationResult, RemasterCitationSource,
    RemasterOrigin, SourceRecordDetail,
};
use std::collections::{BTreeMap, btree_map::Entry};

type FieldKey = (RecordKey, Vec<OwnedContentLocator>, String);
#[derive(Default)]
pub(super) struct ExplanationCache {
    records: BTreeMap<RecordKey, Option<SourceRecordDetail>>,
    fields: BTreeMap<FieldKey, Option<RemasterExplanationLinkView>>,
}
impl ExplanationCache {
    fn resolve(
        &mut self,
        locator: &SourceContentLocator,
        retrieval: &AtlasRetrievalService,
    ) -> Option<RemasterExplanationLinkView> {
        let key = (
            locator.record.clone(),
            locator.owners.clone(),
            locator.field.clone(),
        );
        if let Some(result) = self.fields.get(&key) {
            return result.clone();
        }
        let result = self.read(locator, retrieval);
        self.fields.insert(key, result.clone());
        result
    }
    fn read(
        &mut self,
        locator: &SourceContentLocator,
        retrieval: &AtlasRetrievalService,
    ) -> Option<RemasterExplanationLinkView> {
        let record = match self.records.entry(locator.record.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(
                retrieval
                    .get_record(GetRecordRequest {
                        record_key: &locator.record,
                        selected_content: &[],
                    })
                    .ok()
                    .flatten(),
            ),
        }
        .as_ref()?;
        let node = record.source.node_at(&locator.owners)?;
        if !matches!(
            node,
            SourceNodeView::Journal(_) | SourceNodeView::JournalPage(_)
        ) || !node
            .content_selections()
            .iter()
            .any(|selection| selection.field.as_ref() == locator.field)
        {
            return None;
        }
        let content = retrieval.read_content(std::slice::from_ref(locator)).ok()?;
        if !content.fields.iter().any(|field| {
            field.locator == *locator
                && field.outcome == "prepared"
                && field
                    .html
                    .as_ref()
                    .is_some_and(|html| !html.trim().is_empty())
        }) {
            return None;
        }
        let root = crate::projection::localized_summary_view(&record.summary, retrieval).title;
        let label = match node
            .name()
            .value()
            .filter(|name| !name.trim().is_empty() && name.as_str() != root)
        {
            Some(name) => format!("{root} — {name}"),
            None => root,
        };
        let mut destination = navigation(&locator.record, &locator.owners, retrieval);
        destination.field = Some(locator.field.clone());
        Some(RemasterExplanationLinkView {
            label: text(&label),
            navigation: destination,
        })
    }
}

pub(super) fn attach(
    relationships: &mut [RemasterRelationshipView],
    evidence: &[(String, Vec<RemasterCitationResult>)],
    retrieval: &AtlasRetrievalService,
    selection: Option<&RecordNavigationView>,
    budget: &mut ProjectionBudget,
) {
    let evidence = evidence
        .iter()
        .map(|(id, citations)| (id.as_str(), citations.as_slice()))
        .collect::<BTreeMap<_, _>>();
    for relationship in relationships {
        'sources: for id in &relationship.declaration_ids {
            for citation in evidence.get(id.as_str()).copied().unwrap_or_default() {
                if citation.origin != RemasterOrigin::Foundry {
                    continue;
                }
                let RemasterCitationSource::Journal { locator } = &citation.source else {
                    continue;
                };
                if selection.is_some_and(|selected| {
                    selected.record_key == locator.record.to_string()
                        && selected.owners == locator.owners
                        && (selected.field.is_none()
                            || selected.field.as_deref() == Some(locator.field.as_str()))
                }) {
                    continue;
                }
                let Some(link) = budget.explanation_cache.resolve(locator, retrieval) else {
                    continue;
                };
                if relationship
                    .explanation_links
                    .iter()
                    .any(|prior| prior.navigation == link.navigation)
                {
                    continue;
                }
                if relationship.explanation_links.len() == 5 || budget.explanations == 0 {
                    relationship.truncated = true;
                    break 'sources;
                }
                relationship.explanation_links.push(link);
                budget.explanations -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::{RecordNavigationView, RemasterInlinePartView};
    use atlas_record::source_content::SourceContentLocator;
    use atlas_search::test_support::{
        capture_remaster_declarations, input, open_input_fixture, record,
    };
    use serde_json::json;
    fn locator(page: usize) -> SourceContentLocator {
        SourceContentLocator {
            record: "journals:journal000000001".parse().unwrap(),
            owners: vec![OwnedContentLocator {
                collection: "/pages".into(),
                identity: atlas_domain::OwnedContentIdentity::Stable(format!("page{page:012}")),
            }],
            field: "/text/content".into(),
        }
    }
    fn fixture() -> (
        AtlasRetrievalService,
        atlas_search::test_support::FixtureArtifact,
    ) {
        let mut pages=(0..26).map(|index|json!({"_id":format!("page{index:012}"),"name":if index==0{"Spells".into()}else{format!("Section {index}")},"type":"text","text":{"content":"<p>Readable remaster explanation.</p>","format":1}})).collect::<Vec<_>>();
        pages.push(json!({"_id":"page000000000099","name":"Unsupported body","type":"image","src":"image.png"}));
        let mut data = input(
            vec![
                record(
                    "journals",
                    "JournalEntry",
                    json!({"_id":"journal000000001","name":"Remaster Changes","pages":pages}),
                ),
                record(
                    "items",
                    "Item",
                    json!({"_id":"aaaaaaaaaaaaaaaa","name":"Not a journal","type":"action","system":{"description":{"value":"<p>Ordinary rules.</p>"}}}),
                ),
            ],
            false,
        );
        capture_remaster_declarations(&mut data).unwrap();
        open_input_fixture(data).unwrap()
    }
    fn citation(page: usize) -> RemasterCitationResult {
        RemasterCitationResult {
            origin: RemasterOrigin::Foundry,
            attribution: "Diagnostic label must not become the link caption".into(),
            revision: "revision".into(),
            statement: "Statement".into(),
            source: RemasterCitationSource::Journal {
                locator: locator(page),
            },
        }
    }
    fn relationship(id: &str) -> RemasterRelationshipView {
        RemasterRelationshipView {
            id: id.into(),
            label: "Former name".into(),
            parts: vec![RemasterInlinePartView::Text {
                text: "Magic Missile".into(),
            }],
            declaration_ids: vec![id.into()],
            explanation_links: vec![],
            truncated: false,
        }
    }
    #[test]
    fn remaster_explanation_uses_exact_readable_journal_field_before_diagnostic_budget() {
        let (retrieval, _artifact) = fixture();
        let mut invalid = citation(0);
        invalid.source = RemasterCitationSource::Journal {
            locator: SourceContentLocator {
                record: "items:aaaaaaaaaaaaaaaa".parse().unwrap(),
                owners: vec![],
                field: "/system/description/value".into(),
            },
        };
        let mut script = citation(0);
        script.source = RemasterCitationSource::CapturedFile {
            path: "src/module/migration/887.ts".into(),
        };
        let mut atlas = citation(1);
        atlas.origin = RemasterOrigin::AtlasReviewed;
        let evidence = vec![(
            "one".into(),
            vec![
                citation(0),
                citation(0),
                invalid,
                script,
                atlas,
                citation(99),
                citation(100),
            ],
        )];
        let mut relationships = vec![relationship("one")];
        let mut budget = ProjectionBudget {
            citations: 0,
            ..ProjectionBudget::default()
        };
        attach(&mut relationships, &evidence, &retrieval, None, &mut budget);
        let links = &relationships[0].explanation_links;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].label, "Remaster Changes — Spells");
        assert_eq!(links[0].navigation.record_key, "journals:journal000000001");
        assert_eq!(links[0].navigation.owners, locator(0).owners);
        assert_eq!(links[0].navigation.field.as_deref(), Some("/text/content"));
        assert_eq!(budget.citations, 0);
        assert!(!relationships[0].truncated);
        assert_eq!(evidence[0].1.len(), 7);
    }
    #[test]
    fn remaster_explanation_self_body_is_skipped_but_other_page_or_field_remains_distinct() {
        let (retrieval, _artifact) = fixture();
        let evidence = vec![("one".into(), vec![citation(0)])];
        let selection = |page, field| RecordNavigationView {
            record_key: "journals:journal000000001".into(),
            owners: locator(page).owners,
            field,
            passage: None,
            spell_rank: None,
            source_fingerprint: None,
        };
        for field in [None, Some("/text/content".into())] {
            let mut relationships = vec![relationship("one")];
            attach(
                &mut relationships,
                &evidence,
                &retrieval,
                Some(&selection(0, field)),
                &mut ProjectionBudget::default(),
            );
            assert!(relationships[0].explanation_links.is_empty());
        }
        for selected in [
            selection(1, None),
            selection(0, Some("/another-field".into())),
            RecordNavigationView {
                owners: vec![],
                ..selection(0, None)
            },
        ] {
            let mut relationships = vec![relationship("one")];
            attach(
                &mut relationships,
                &evidence,
                &retrieval,
                Some(&selected),
                &mut ProjectionBudget::default(),
            );
            assert_eq!(relationships[0].explanation_links.len(), 1);
        }
    }
    #[test]
    fn remaster_explanation_five_per_group_twenty_global_cut_is_truthful() {
        let (retrieval, _artifact) = fixture();
        let mut relationships = (0..6)
            .map(|index| relationship(&format!("group-{index}")))
            .collect::<Vec<_>>();
        let evidence = (0..6)
            .map(|index| (format!("group-{index}"), (0..6).map(citation).collect()))
            .collect::<Vec<_>>();
        let mut budget = ProjectionBudget::default();
        attach(&mut relationships, &evidence, &retrieval, None, &mut budget);
        assert_eq!(
            relationships
                .iter()
                .map(|value| value.explanation_links.len())
                .collect::<Vec<_>>(),
            [5, 5, 5, 5, 0, 0]
        );
        assert!(relationships.iter().all(|value| value.truncated));
        assert_eq!(budget.explanations, 0);
        assert_eq!(budget.citations, 200);
    }
}
