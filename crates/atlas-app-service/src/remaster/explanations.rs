//! Bounded readable explanations, separate from diagnostic source evidence.
use super::navigation;
use atlas_app_model::RemasterExplanationLinkView;
use atlas_domain::{OwnedContentLocator, RecordKey};
use atlas_record::source_content::SourceContentLocator;
use atlas_record::source_record::SourceNodeView;
use atlas_search::{AtlasRetrievalService, GetRecordRequest, SourceRecordDetail};
use std::collections::{BTreeMap, btree_map::Entry};

type FieldKey = (RecordKey, Vec<OwnedContentLocator>, String);
#[derive(Default)]
pub(super) struct ExplanationCache {
    records: BTreeMap<RecordKey, Option<SourceRecordDetail>>,
    fields: BTreeMap<FieldKey, Option<RemasterExplanationLinkView>>,
}
impl ExplanationCache {
    pub(super) fn resolve(
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
            label,
            navigation: destination,
        })
    }
}
