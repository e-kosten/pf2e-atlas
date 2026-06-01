use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_record::{ContentDocument, PersistedRecord, visit_content_references_mut};

use crate::text::TextSearchRecord;
use crate::{AtlasRetrievalService, SearchError};

use super::RecordResolutionResult;

impl AtlasRetrievalService {
    pub(crate) fn enrich_reference_labels(
        &self,
        records: &mut [PersistedRecord],
    ) -> Result<(), SearchError> {
        self.enrich_reference_labels_for_items(records, persisted_record, persisted_record_mut)
    }

    pub(crate) fn enrich_resolution_reference_labels(
        &self,
        matches: &mut [RecordResolutionResult],
    ) -> Result<(), SearchError> {
        self.enrich_reference_labels_for_items(matches, resolution_record, resolution_record_mut)
    }

    pub(crate) fn enrich_text_record_reference_labels(
        &self,
        records: &mut [TextSearchRecord],
    ) -> Result<(), SearchError> {
        self.enrich_reference_labels_for_items(records, text_record, text_record_mut)
    }

    fn enrich_reference_labels_for_items<T>(
        &self,
        items: &mut [T],
        record: fn(&T) -> &PersistedRecord,
        record_mut: fn(&mut T) -> &mut PersistedRecord,
    ) -> Result<(), SearchError> {
        let mut target_keys = BTreeSet::new();
        for item in items.iter() {
            collect_reference_target_keys(record(item), &mut target_keys);
        }
        if target_keys.is_empty() {
            return Ok(());
        }

        let requested_keys = items
            .iter()
            .map(|item| record(item).key.clone())
            .collect::<BTreeSet<_>>();
        let keys_to_load = target_keys
            .into_iter()
            .filter(|key| !requested_keys.contains(key))
            .collect::<Vec<_>>();
        let loaded_targets = self
            .index
            .load_records_by_key(&keys_to_load)
            .map_err(SearchError::from_record_load)?;
        let names_by_key = items
            .iter()
            .map(record)
            .chain(loaded_targets.iter())
            .map(|record| (record.key.clone(), record.name.clone()))
            .collect::<BTreeMap<_, _>>();

        for item in items {
            apply_reference_target_names(record_mut(item), &names_by_key);
        }
        Ok(())
    }
}

fn persisted_record(record: &PersistedRecord) -> &PersistedRecord {
    record
}

fn persisted_record_mut(record: &mut PersistedRecord) -> &mut PersistedRecord {
    record
}

fn resolution_record(result: &RecordResolutionResult) -> &PersistedRecord {
    &result.record
}

fn resolution_record_mut(result: &mut RecordResolutionResult) -> &mut PersistedRecord {
    &mut result.record
}

fn text_record(result: &TextSearchRecord) -> &PersistedRecord {
    &result.record
}

fn text_record_mut(result: &mut TextSearchRecord) -> &mut PersistedRecord {
    &mut result.record
}

fn collect_reference_target_keys(record: &PersistedRecord, target_keys: &mut BTreeSet<RecordKey>) {
    if let Some(document) = &record.description {
        collect_document_reference_target_keys(document, target_keys);
    }
    if let Some(document) = &record.blurb {
        collect_document_reference_target_keys(document, target_keys);
    }
    for supplemental in &record.supplemental_content {
        collect_document_reference_target_keys(&supplemental.document, target_keys);
    }
}

fn collect_document_reference_target_keys(
    document: &ContentDocument,
    target_keys: &mut BTreeSet<RecordKey>,
) {
    for reference in atlas_record::iter_content_references(document) {
        if let Some(record_key) = &reference.resolved_key
            && reference.resolved_name.is_none()
        {
            target_keys.insert(record_key.clone());
        }
    }
}

fn apply_reference_target_names(
    record: &mut PersistedRecord,
    names_by_key: &BTreeMap<RecordKey, String>,
) {
    if let Some(document) = &mut record.description {
        apply_document_reference_target_names(document, names_by_key);
    }
    if let Some(document) = &mut record.blurb {
        apply_document_reference_target_names(document, names_by_key);
    }
    for supplemental in &mut record.supplemental_content {
        apply_document_reference_target_names(&mut supplemental.document, names_by_key);
    }
}

fn apply_document_reference_target_names(
    document: &mut ContentDocument,
    names_by_key: &BTreeMap<RecordKey, String>,
) {
    visit_content_references_mut(document, |reference| {
        if reference.resolved_name.is_none()
            && let Some(record_key) = &reference.resolved_key
            && let Some(name) = names_by_key.get(record_key)
        {
            reference.resolved_name = Some(name.clone());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_record::{ContentBlock, ContentInline, ContentReference, ContentReferenceLocator};

    #[test]
    fn reference_label_enrichment_uses_loaded_target_names() {
        let target_key = RecordKey::parse("feats-srd:UKXaMhb9qlPYw1HD").expect("key parses");
        let mut document = ContentDocument::new(vec![ContentBlock::Paragraph {
            content: vec![ContentInline::Reference {
                reference: ContentReference {
                    label: None,
                    locator: ContentReferenceLocator::FoundryUuid {
                        raw_target: "Compendium.pf2e.feats-srd.Item.UKXaMhb9qlPYw1HD".to_string(),
                    },
                    resolved_key: Some(target_key.clone()),
                    resolved_name: None,
                },
            }],
        }]);
        let mut target_keys = BTreeSet::new();
        collect_document_reference_target_keys(&document, &mut target_keys);

        assert_eq!(target_keys, BTreeSet::from([target_key.clone()]));

        let names_by_key = BTreeMap::from([(target_key, "Guardian's Deflection".to_string())]);
        apply_document_reference_target_names(&mut document, &names_by_key);

        assert_eq!(
            atlas_record::render_markdown_like(&document),
            "[Guardian's Deflection](record:feats-srd:UKXaMhb9qlPYw1HD)"
        );
    }
}
