//! Checked document identity and its directly addressable comparison sections.
use crate::{AtlasRetrievalService, SearchError, SearchErrorKind};
use atlas_domain::{
    OwnedContentLocator, SearchProfileId, SimilarityPartSelector, SimilarityParts, SimilaritySeed,
    SourcePassageAddress, SourceRecordSummary,
};
use atlas_record::source_record::SourceBackedRecord;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct CheckedSimilaritySource {
    pub record: SourceRecordSummary,
    /// Checked parent snapshot for projecting the selected embedded document's identity.
    pub embedded_source: Option<SourceBackedRecord>,
}

impl AtlasRetrievalService {
    /// Resolve document identity without reading an inventory or using a search profile.
    pub fn similarity_source(
        &self,
        seed: &SimilaritySeed,
    ) -> Result<Option<CheckedSimilaritySource>, SearchError> {
        let owners = seed.embedded_path();
        if !OwnedContentLocator::within_address_bounds(owners) {
            return Err(SearchError::invalid_search_options(
                "similarity document address exceeds its bounds",
            ));
        }
        if OwnedContentLocator::requires_fingerprint(owners) {
            if seed.source_fingerprint() != Some(self.source_fingerprint()) {
                return Err(SearchError::new(
                    SearchErrorKind::SimilaritySourceStale,
                    "This document address belongs to a different reference collection. Open the document again.",
                ));
            }
        } else if seed.source_fingerprint().is_some() {
            return Err(SearchError::invalid_search_options(
                "stable similarity document addresses do not use a source fingerprint",
            ));
        }
        let Some(record) = self.index.read_summary(seed.record_key())? else {
            return Ok(None);
        };
        let embedded_source = if owners.is_empty() {
            None
        } else {
            let source = self
                .index
                .read_source_record(seed.record_key())?
                .ok_or_else(|| {
                    SearchError::artifact_contract_violation(
                        "record summary has no checked source body",
                    )
                })?;
            if source.node_at(owners).is_none() {
                return Err(SearchError::new(
                    SearchErrorKind::SimilaritySourceMissing,
                    "The selected document is unavailable.",
                ));
            }
            Some(source)
        };
        Ok(Some(CheckedSimilaritySource {
            record,
            embedded_source,
        }))
    }

    pub fn similarity_part_addresses(
        &self,
        seed: &SimilaritySeed,
    ) -> Result<BTreeMap<SimilarityPartSelector, SourcePassageAddress>, SearchError> {
        Ok(self.index.similarity_part_addresses(seed)?)
    }

    pub fn similarity_parts_with_source(
        &self,
        seed: &SimilaritySeed,
        profile: Option<&SearchProfileId>,
    ) -> Result<(CheckedSimilaritySource, SimilarityParts), SearchError> {
        let source = self.similarity_source(seed)?.ok_or_else(|| {
            SearchError::new(
                SearchErrorKind::SimilaritySourceMissing,
                "The selected record is unavailable.",
            )
        })?;
        let profile = self.resolve_search_profile(profile, false)?;
        Ok((source, self.index.similarity_parts(seed, Some(&profile))?))
    }

    pub fn similarity_parts(
        &self,
        seed: &SimilaritySeed,
        profile: Option<&SearchProfileId>,
    ) -> Result<SimilarityParts, SearchError> {
        Ok(self.similarity_parts_with_source(seed, profile)?.1)
    }
}
