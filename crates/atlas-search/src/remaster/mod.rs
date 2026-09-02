use std::collections::BTreeMap;

use atlas_domain::{RecordKey, RemasterLinkSource};
use atlas_index::{IndexRemasterLinks, RemasterReadIndex};
use atlas_record::RetrievedRecord;

use crate::{AtlasRetrievalService, GetRecordsRequest, RecordRetrieval, SearchError};

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinksRequest<'a> {
    pub record_key: &'a RecordKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinksResult {
    pub seed: RetrievedRecord,
    pub links: Vec<RemasterLinkResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinkResult {
    pub remaster_record: RetrievedRecord,
    pub legacy_record: RetrievedRecord,
    pub source: RemasterLinkSource,
    pub source_ref: String,
}

pub trait RemasterRetrieval {
    fn remaster_links(
        &self,
        request: RemasterLinksRequest<'_>,
    ) -> Result<Option<RemasterLinksResult>, SearchError>;
}

impl RemasterRetrieval for AtlasRetrievalService {
    fn remaster_links(
        &self,
        request: RemasterLinksRequest<'_>,
    ) -> Result<Option<RemasterLinksResult>, SearchError> {
        let record_key = request.record_key;
        let Some(seed) = self
            .get_records(GetRecordsRequest {
                record_keys: std::slice::from_ref(record_key),
            })?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };
        let links = if let Some(links) = remaster_links_for_record(self.index.as_ref(), record_key)?
        {
            let record_keys = links
                .links
                .iter()
                .flat_map(|link| {
                    [
                        link.remaster_record_key.clone(),
                        link.legacy_record_key.clone(),
                    ]
                })
                .collect::<Vec<_>>();
            let records_by_key = self
                .get_records(GetRecordsRequest {
                    record_keys: &record_keys,
                })?
                .into_iter()
                .map(|record| (record.record.identity.key.clone(), record))
                .collect::<BTreeMap<_, _>>();
            complete_remaster_links(links, &records_by_key)
        } else {
            Vec::new()
        };
        Ok(Some(RemasterLinksResult { seed, links }))
    }
}

fn complete_remaster_links(
    links: IndexRemasterLinks,
    records_by_key: &BTreeMap<RecordKey, RetrievedRecord>,
) -> Vec<RemasterLinkResult> {
    links
        .links
        .into_iter()
        .filter_map(|link| {
            let remaster_record = records_by_key.get(&link.remaster_record_key).cloned()?;
            let legacy_record = records_by_key.get(&link.legacy_record_key).cloned()?;
            Some(RemasterLinkResult {
                remaster_record,
                legacy_record,
                source: link.source,
                source_ref: link.source_ref,
            })
        })
        .collect()
}

fn remaster_links_for_record<I>(
    index: &I,
    record_key: &RecordKey,
) -> Result<Option<IndexRemasterLinks>, SearchError>
where
    I: RemasterReadIndex + ?Sized,
{
    index
        .remaster_links_for_record(record_key)
        .map_err(SearchError::from_record_load)
}

#[cfg(test)]
mod tests {
    use atlas_domain::{RecordKind, RemasterLinkSource};
    use atlas_index::IndexRemasterLinkRecord;
    use atlas_record::{
        AtlasRecord, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordIdentity, RecordProvenance,
    };

    use super::*;

    #[test]
    fn missing_target_drops_only_the_incomplete_counterpart() {
        let legacy = record("legacy-pack:seed", "Legacy Seed");
        let remaster = record("remaster-pack:complete", "Complete Remaster");
        let missing_key = RecordKey::parse("remaster-pack:missing").expect("missing key");
        let mut records_by_key = BTreeMap::new();
        records_by_key.insert(legacy.record.identity.key.clone(), legacy.clone());
        records_by_key.insert(remaster.record.identity.key.clone(), remaster.clone());
        let links = IndexRemasterLinks {
            links: vec![
                IndexRemasterLinkRecord {
                    remaster_record_key: missing_key,
                    legacy_record_key: legacy.record.identity.key.clone(),
                    source: RemasterLinkSource::Migration,
                    source_ref: "missing".to_string(),
                },
                IndexRemasterLinkRecord {
                    remaster_record_key: remaster.record.identity.key.clone(),
                    legacy_record_key: legacy.record.identity.key.clone(),
                    source: RemasterLinkSource::RemasterJournal,
                    source_ref: "journal:Bestiaries".to_string(),
                },
            ],
        };

        let hydrated = complete_remaster_links(links, &records_by_key);
        assert_eq!(hydrated.len(), 1);
        assert_eq!(
            hydrated[0].remaster_record.record.identity.key,
            remaster.record.identity.key
        );
        assert_eq!(
            hydrated[0].legacy_record.record.identity.key,
            legacy.record.identity.key
        );
    }

    fn record(record_key: &str, title: &str) -> RetrievedRecord {
        RetrievedRecord {
            record: AtlasRecord::new(
                RecordIdentity::new(
                    RecordKey::parse(record_key).expect("fixture key should parse"),
                    title,
                ),
                RecordClassification::new(RecordKind::Rule),
                FoundryRecordInfo::new(
                    "Fixture",
                    FoundryDocumentType::Item,
                    FoundryRecordType::Action,
                ),
                RecordProvenance::new(format!("fixtures/{record_key}.json")),
            ),
            body: None,
        }
    }
}
