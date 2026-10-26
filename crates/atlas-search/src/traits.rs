//! Typed vocabulary retrieval on the same admitted reference snapshot.
use crate::{
    AtlasRetrievalService, RecordScope, SearchError, SearchPage, SearchPageInfo, TextSearchResult,
};
use atlas_domain::{QueryPredicate, SearchProfileId, SearchRanking, TraitKey};
pub use atlas_index::{TraitDefinition, TraitMembership, TraitReference, TraitSourceLocation};
pub struct TraitAssociationRequest<'a> {
    pub trait_key: &'a TraitKey,
    pub page: SearchPage,
}
pub struct TraitMembershipResult {
    pub entries: Vec<TraitMembership>,
    pub page: SearchPageInfo,
}
pub struct TraitReferenceResult {
    pub entries: Vec<TraitReference>,
    pub records: Vec<atlas_domain::SourceRecordSummary>,
    pub page: SearchPageInfo,
}
pub struct SimilarTraitRequest<'a> {
    pub sort: atlas_domain::ResultSort,
    pub trait_key: &'a TraitKey,
    pub search_profile: Option<&'a SearchProfileId>,
    pub filter: Option<&'a QueryPredicate>,
    pub scope: RecordScope<'a>,
    pub page: SearchPage,
    pub prefer_remaster: bool,
    pub ranking: SearchRanking,
}
pub struct SimilarTraitResult {
    pub seed: TraitDefinition,
    pub results: TextSearchResult,
}
impl AtlasRetrievalService {
    pub fn trait_definition(&self, key: &TraitKey) -> Result<Option<TraitDefinition>, SearchError> {
        Ok(self.index.trait_definition(key)?)
    }
    pub fn trait_memberships(
        &self,
        request: TraitAssociationRequest<'_>,
    ) -> Result<TraitMembershipResult, SearchError> {
        let (entries, total) = self.index.trait_memberships(
            request.trait_key,
            request.page.size(),
            request.page.offset()?,
        )?;
        let page = SearchPageInfo::from_page(request.page, entries.len(), total)?;
        Ok(TraitMembershipResult { entries, page })
    }
    /// Full root scope for composing trait references with ordinary retrieval filters.
    pub fn trait_reference_record_keys(
        &self,
        key: &TraitKey,
    ) -> Result<Vec<atlas_domain::RecordKey>, SearchError> {
        Ok(self.index.trait_reference_record_keys(key)?)
    }
    pub fn trait_references(
        &self,
        request: TraitAssociationRequest<'_>,
    ) -> Result<TraitReferenceResult, SearchError> {
        let (entries, total) = self.index.trait_reference_entries(
            request.trait_key,
            request.page.size(),
            request.page.offset()?,
        )?;
        let page = SearchPageInfo::from_page(request.page, entries.len(), total)?;
        let records = entries.iter().map(|entry| entry.record.clone()).collect();
        Ok(TraitReferenceResult {
            records,
            entries,
            page,
        })
    }
    pub fn similar_trait(
        &self,
        request: SimilarTraitRequest<'_>,
    ) -> Result<Option<SimilarTraitResult>, SearchError> {
        Self::validate_search_sort(&request.sort)?;
        let Some(seed) = self.index.trait_definition(request.trait_key)? else {
            return Ok(None);
        };
        let profile = self.resolve_search_profile(request.search_profile, true)?;
        let vector=self.index.trait_vector(request.trait_key,&profile)?.ok_or_else(||SearchError::vector_readiness_required("trait has no stored definition vector; update the reference workspace with embeddings"))?;
        let predicate = self.predicate(request.filter)?;
        let (roots, window) = self.rank_candidates(crate::text::CandidateRequest {
            ranking: request.ranking,
            search_profile: &profile,
            text: "",
            mode: crate::RetrievalMode::Vector,
            query: &predicate,
            scope: request.scope,
            prefer: request.prefer_remaster,
            suppress: true,
            vector: Some(&vector),
            exclude: None,
        })?;
        let results =
            self.finish_results(&profile, roots, request.page, true, window, request.sort)?;
        Ok(Some(SimilarTraitResult { seed, results }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{hash, input, open_input_fixture, record};
    use serde_json::json;
    fn fixture(vector: bool) -> (atlas_index::IndexBuildInput, TraitKey) {
        let record = record(
            "items",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Flame","type":"effect","system":{"description":{"value":"Fire burns."}}}),
        );
        let mut fixture = input(vec![record], vector);
        let key = TraitKey::new("fire").unwrap();
        fixture.traits.push(TraitDefinition {
            key: key.clone(),
            label: "Fire".into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec![],
            source_locations: vec![],
            source_revision: fixture.context.source_revision.clone(),
            indexing_locale: fixture.context.indexing_locale.clone(),
        });
        if vector {
            let mut vector = vec![0.; 768];
            vector[0] = 1.;
            let chunk_hash = hash(b"Fire\n");
            fixture.trait_vectors.push(atlas_index::TraitVectorInput {
                key: key.clone(),
                profile_id: SearchProfileId::new("bge-base"),
                input_hash: hash(format!("trait-definition-centroid/v1:{chunk_hash}").as_bytes()),
                vector: vector.clone(),
                chunks: vec![atlas_index::TraitSeedChunk {
                    input: "Fire\n".into(),
                    input_hash: chunk_hash.clone(),
                    token_count: 2,
                    vector,
                    address: atlas_domain::SourcePassageAddress::PlainSection {
                        source_text_sha256: chunk_hash,
                        selection_version: "trait-definition-centroid/v1".into(),
                        section_ordinal: 0,
                        label: Some("Fire".into()),
                        chunk_bytes: atlas_domain::SourceByteRange { start: 0, end: 5 },
                    },
                }],
            });
        }
        (fixture, key)
    }
    #[test]
    fn trait_similar_uses_stored_seed_and_normal_record_candidates() {
        let (fixture, key) = fixture(true);
        let (service, _artifact) = open_input_fixture(fixture).unwrap();
        let result = service
            .similar_trait(SimilarTraitRequest {
                sort: atlas_domain::ResultSort::Relevance {},
                trait_key: &key,
                search_profile: None,
                filter: None,
                scope: RecordScope::All,
                page: SearchPage::default(),
                prefer_remaster: true,
                ranking: SearchRanking::Score,
            })
            .unwrap()
            .unwrap();
        assert_eq!(result.seed.key, key);
        assert_eq!(result.results.records.len(), 1);
        assert_eq!(
            result.results.records[0].record.key.to_string(),
            "items:aaaaaaaaaaaaaaaa"
        );
        assert_eq!(result.results.coverage.semantic_unit_window, Some(4096));
    }
    #[test]
    fn trait_similar_without_configured_vectors_has_explicit_readiness_failure() {
        let (fixture, key) = fixture(false);
        let (service, _artifact) = open_input_fixture(fixture).unwrap();
        let result = service.similar_trait(SimilarTraitRequest {
            sort: atlas_domain::ResultSort::Relevance {},
            trait_key: &key,
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            prefer_remaster: true,
            ranking: SearchRanking::Score,
        });
        assert!(result.is_err());
    }
}
