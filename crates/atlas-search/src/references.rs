use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_index::RecordLoadOptions;
use atlas_record::{ContentDocument, PersistedRecord, visit_content_references_mut};

use crate::{AtlasRetrievalService, RecordResolutionResult, SearchError, TextSearchRecord};

impl AtlasRetrievalService {
    pub(crate) fn enrich_reference_labels(
        &self,
        records: &mut [PersistedRecord],
    ) -> Result<(), SearchError> {
        let mut target_keys = BTreeSet::new();
        for record in records.iter() {
            collect_reference_target_keys(record, &mut target_keys);
        }
        if target_keys.is_empty() {
            return Ok(());
        }

        let requested_keys = records
            .iter()
            .map(|record| record.key.clone())
            .collect::<BTreeSet<_>>();
        let keys_to_load = target_keys
            .into_iter()
            .filter(|key| !requested_keys.contains(key))
            .collect::<Vec<_>>();
        let loaded_targets = self
            .index
            .load_records_by_key_with_options(&keys_to_load, RecordLoadOptions::omit_raw_json())?;
        let names_by_key = records
            .iter()
            .chain(loaded_targets.iter())
            .map(|record| (record.key.clone(), record.name.clone()))
            .collect::<BTreeMap<_, _>>();

        for record in records {
            apply_reference_target_names(record, &names_by_key);
        }
        Ok(())
    }

    pub(crate) fn enrich_resolution_reference_labels(
        &self,
        matches: &mut [RecordResolutionResult],
    ) -> Result<(), SearchError> {
        let mut target_keys = BTreeSet::new();
        for resolution in matches.iter() {
            collect_reference_target_keys(&resolution.record, &mut target_keys);
        }
        if target_keys.is_empty() {
            return Ok(());
        }

        let requested_keys = matches
            .iter()
            .map(|resolution| resolution.record.key.clone())
            .collect::<BTreeSet<_>>();
        let keys_to_load = target_keys
            .into_iter()
            .filter(|key| !requested_keys.contains(key))
            .collect::<Vec<_>>();
        let loaded_targets = self
            .index
            .load_records_by_key_with_options(&keys_to_load, RecordLoadOptions::omit_raw_json())?;
        let names_by_key = matches
            .iter()
            .map(|resolution| &resolution.record)
            .chain(loaded_targets.iter())
            .map(|record| (record.key.clone(), record.name.clone()))
            .collect::<BTreeMap<_, _>>();

        for resolution in matches {
            apply_reference_target_names(&mut resolution.record, &names_by_key);
        }
        Ok(())
    }

    pub(crate) fn enrich_text_record_reference_labels(
        &self,
        records: &mut [TextSearchRecord],
    ) -> Result<(), SearchError> {
        let mut target_keys = BTreeSet::new();
        for item in records.iter() {
            collect_reference_target_keys(&item.record, &mut target_keys);
        }
        if target_keys.is_empty() {
            return Ok(());
        }

        let requested_keys = records
            .iter()
            .map(|item| item.record.key.clone())
            .collect::<BTreeSet<_>>();
        let keys_to_load = target_keys
            .into_iter()
            .filter(|key| !requested_keys.contains(key))
            .collect::<Vec<_>>();
        let loaded_targets = self
            .index
            .load_records_by_key_with_options(&keys_to_load, RecordLoadOptions::omit_raw_json())?;
        let names_by_key = records
            .iter()
            .map(|item| &item.record)
            .chain(loaded_targets.iter())
            .map(|record| (record.key.clone(), record.name.clone()))
            .collect::<BTreeMap<_, _>>();

        for item in records {
            apply_reference_target_names(&mut item.record, &names_by_key);
        }
        Ok(())
    }
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
