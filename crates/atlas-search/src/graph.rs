use atlas_domain::{RecordKey, SourceRecordSummary};
use atlas_index::SourceStoredRelationship;
use atlas_record::source_content::OwnedContentLocator;
pub struct GraphContextRequest {
    pub seed: RecordKey,
    pub owners: Option<Vec<OwnedContentLocator>>,
    pub outgoing_limit: usize,
    pub backlink_limit: usize,
}
impl GraphContextRequest {
    pub fn new(seed: RecordKey) -> Self {
        Self {
            seed,
            owners: None,
            outgoing_limit: 64,
            backlink_limit: 0,
        }
    }
    pub fn uses(seed: RecordKey) -> Self {
        Self {
            seed,
            owners: None,
            outgoing_limit: 0,
            backlink_limit: 64,
        }
    }
}
#[derive(Debug, Clone)]
pub struct GraphContextSection {
    pub records: Vec<SourceRecordSummary>,
    pub occurrences: Vec<SourceStoredRelationship>,
    pub truncated: bool,
}
#[derive(Debug, Clone)]
pub struct GraphContextResult {
    pub seed: SourceRecordSummary,
    pub outgoing: GraphContextSection,
    pub backlinks: GraphContextSection,
}
pub struct RemasterLinksRequest<'a> {
    pub record_key: &'a RecordKey,
}
#[derive(Debug, Clone)]
pub struct RemasterLinkResult {
    pub legacy_record: SourceRecordSummary,
    pub remaster_record: SourceRecordSummary,
    pub evidence: String,
}
#[derive(Debug, Clone)]
pub struct RemasterLinksResult {
    pub seed: SourceRecordSummary,
    pub links: Vec<RemasterLinkResult>,
}
use crate::{AtlasRetrievalService, SearchError};
use atlas_index::{SourceRelationshipDirection, SourceRelationshipRequest};
use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
use std::collections::BTreeSet;
impl AtlasRetrievalService {
    pub fn read_relationships(
        &self,
        request: &atlas_index::SourceRelationshipRequest,
    ) -> Result<atlas_index::SourceRelationshipBundle, SearchError> {
        Ok(self.index.read_relationships(request)?)
    }
    pub fn graph_context(
        &self,
        request: GraphContextRequest,
    ) -> Result<Option<GraphContextResult>, SearchError> {
        let Some(seed) = self.index.read_summary(&request.seed)? else {
            return Ok(None);
        };
        let section = |direction, limit: usize| -> Result<GraphContextSection, SearchError> {
            if limit > 1024 {
                return Err(SearchError::invalid_search_options(
                    "graph occurrence limit exceeds 1024",
                ));
            }
            if limit == 0 {
                return Ok(GraphContextSection {
                    records: vec![],
                    occurrences: vec![],
                    truncated: false,
                });
            }
            let bundle = self.index.read_relationships(&SourceRelationshipRequest {
                record: request.seed.clone(),
                owners: request.owners.clone(),
                field: None,
                direction,
                limit,
            })?;
            let keys = bundle
                .occurrences
                .iter()
                .filter_map(|o| match direction {
                    SourceRelationshipDirection::Incoming => Some(o.locator.record.clone()),
                    SourceRelationshipDirection::Outgoing => match &o.resolution {
                        ContentReferenceResolution::Resolved(
                            ContentReferenceTarget::Record { key }
                            | ContentReferenceTarget::OwnedNode { key, .. },
                        ) => Some(key.clone()),
                        _ => None,
                    },
                })
                .filter(|k| k != &request.seed)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            // A preserved reference may target excluded tooling; product summaries stay hidden.
            let records = self
                .get_records(crate::GetRecordsRequest { record_keys: &keys })?
                .records;
            Ok(GraphContextSection {
                records,
                occurrences: bundle.occurrences,
                truncated: bundle.truncated,
            })
        };
        Ok(Some(GraphContextResult {
            seed,
            outgoing: section(
                SourceRelationshipDirection::Outgoing,
                request.outgoing_limit,
            )?,
            backlinks: section(
                SourceRelationshipDirection::Incoming,
                request.backlink_limit,
            )?,
        }))
    }
    pub fn remaster_links(
        &self,
        request: RemasterLinksRequest<'_>,
    ) -> Result<Option<RemasterLinksResult>, SearchError> {
        let Some(seed) = self.index.read_summary(request.record_key)? else {
            return Ok(None);
        };
        let mut links = Vec::new();
        for pair in self.index.read_remaster_links(request.record_key)? {
            let records = self.summaries(&[pair.legacy, pair.remaster])?;
            let mut records = records.into_iter();
            links.push(RemasterLinkResult {
                legacy_record: records.next().ok_or_else(|| {
                    SearchError::artifact_contract_violation("verified pair legacy summary absent")
                })?,
                remaster_record: records.next().ok_or_else(|| {
                    SearchError::artifact_contract_violation(
                        "verified pair remaster summary absent",
                    )
                })?,
                evidence: pair.evidence,
            });
        }
        Ok(Some(RemasterLinksResult { seed, links }))
    }
}
