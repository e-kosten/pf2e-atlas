use crate::{RecordScope, SearchPage, TextSearchResult};
use atlas_domain::{QueryPredicate, RecordKey};
pub struct SimilarRecordRequest<'a> {
    pub sort: atlas_domain::ResultSort,
    pub search_profile: Option<&'a atlas_domain::SearchProfileId>,
    pub record_key: &'a RecordKey,
    pub filter: Option<&'a QueryPredicate>,
    pub scope: RecordScope<'a>,
    pub page: SearchPage,
    pub prefer_remaster: bool,
}
pub struct SimilarRecordResult {
    pub seed: atlas_domain::SourceRecordSummary,
    pub results: TextSearchResult,
}
impl crate::AtlasRetrievalService {
    pub fn similar_records(
        &self,
        request: SimilarRecordRequest<'_>,
    ) -> Result<Option<SimilarRecordResult>, crate::SearchError> {
        Self::validate_search_sort(&request.sort)?;
        self.resolve_search_profile(request.search_profile, request.search_profile.is_some())?;
        let Some(seed) = self.index.read_summary(request.record_key)? else {
            return Ok(None);
        };
        let search_profile = self.resolve_search_profile(request.search_profile, true)?;
        let vector = self
            .index
            .seed_identity_vector(&search_profile, request.record_key)?
            .ok_or_else(|| {
                crate::SearchError::vector_readiness_required(
                    "seed has no stored root identity vector; rebuild with embeddings",
                )
            })?;
        let predicate = self
            .predicate(request.filter)?
            .with_relationships(request.scope.relationships());
        let (roots, window) = self.rank_candidates(crate::text::CandidateRequest {
            ranking: Default::default(),
            search_profile: &search_profile,
            text: "",
            mode: crate::RetrievalMode::Vector,
            query: &predicate,
            scope: request.scope,
            prefer: request.prefer_remaster,
            suppress: true,
            vector: Some(&vector.vector),
            exclude: Some(request.record_key),
        })?;
        let results = self.finish_results(
            &search_profile,
            roots,
            request.page,
            true,
            window,
            request.sort,
        )?;
        Ok(Some(SimilarRecordResult { seed, results }))
    }
}
