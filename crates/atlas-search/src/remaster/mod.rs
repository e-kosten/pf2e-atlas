use std::collections::BTreeMap;

use atlas_domain::{RecordKey, RemasterLinkSource};
use atlas_index::{IndexRemasterLinks, RemasterReadIndex};
use atlas_record::AtlasRecord;

use crate::{AtlasRetrievalService, GetRecordsRequest, RecordRetrieval, SearchError};

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinksRequest<'a> {
    pub record_key: &'a RecordKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinksResult {
    pub seed: AtlasRecord,
    pub links: Vec<RemasterLinkResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemasterLinkResult {
    pub remaster_record: AtlasRecord,
    pub legacy_record: AtlasRecord,
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
        let links = remaster_links_for_record(self.index.as_ref(), record_key)?
            .map(|links| -> Result<_, SearchError> {
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
                    .map(|record| (record.identity.key.clone(), record))
                    .collect::<BTreeMap<_, _>>();
                links
                    .links
                    .into_iter()
                    .map(|link| {
                        let remaster_record = records_by_key
                            .get(&link.remaster_record_key)
                            .cloned()
                            .ok_or_else(|| {
                                SearchError::from_record_load(
                                    atlas_index::RecordLoadError::InvalidData(format!(
                                        "remaster link target `{}` was not found",
                                        link.remaster_record_key
                                    )),
                                )
                            })?;
                        let legacy_record = records_by_key
                            .get(&link.legacy_record_key)
                            .cloned()
                            .ok_or_else(|| {
                                SearchError::from_record_load(
                                    atlas_index::RecordLoadError::InvalidData(format!(
                                        "remaster link target `{}` was not found",
                                        link.legacy_record_key
                                    )),
                                )
                            })?;
                        Ok(RemasterLinkResult {
                            remaster_record,
                            legacy_record,
                            source: link.source,
                            source_ref: link.source_ref,
                        })
                    })
                    .collect()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Some(RemasterLinksResult { seed, links }))
    }
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
