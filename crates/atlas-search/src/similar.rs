use crate::{RecordScope, SearchPage, TextSearchResult};
use atlas_domain::{
    QueryPredicate, RecordKey, SearchContentScope, SearchProfileId, SourceRecordSummary,
};
pub struct SimilarRecordRequest<'a> {
    pub content_scope: atlas_domain::SearchContentScope,
    pub sort: atlas_domain::ResultSort,
    pub search_profile: Option<&'a atlas_domain::SearchProfileId>,
    pub record_key: &'a RecordKey,
    pub filter: Option<&'a QueryPredicate>,
    pub scope: RecordScope<'a>,
    pub page: SearchPage,
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
        let (roots, window) = self.rank_similar_seed(
            &seed,
            &search_profile,
            request.content_scope,
            request.filter,
            request.scope,
            atlas_domain::SearchResultDetail::Evidence,
        )?;
        let results = self.finish_results(
            &search_profile,
            roots,
            request.page,
            window,
            request.sort,
            atlas_domain::SearchResultDetail::Evidence,
        )?;
        Ok(Some(SimilarRecordResult { seed, results }))
    }

    pub(super) fn rank_similar_seed(
        &self,
        seed: &SourceRecordSummary,
        profile: &SearchProfileId,
        content_scope: SearchContentScope,
        filter: Option<&QueryPredicate>,
        scope: RecordScope<'_>,
        result_detail: atlas_domain::SearchResultDetail,
    ) -> Result<(Vec<crate::text::RankedRoot>, Option<usize>), crate::SearchError> {
        let vector = self
            .index
            .seed_identity_vector(profile, &seed.key)?
            .ok_or_else(|| {
                crate::SearchError::vector_readiness_required(
                    "seed has no stored root identity vector; rebuild with embeddings",
                )
            })?;
        let predicate = self.predicate(filter)?;
        self.rank_candidates(crate::text::CandidateRequest {
            result_detail,
            lexical: Default::default(),
            content_scope,
            ranking: Default::default(),
            search_profile: profile,
            text: "",
            mode: crate::RetrievalMode::Vector,
            query: &predicate,
            scope,
            vector: Some(&vector.vector),
            exclude: Some(&seed.key),
        })
    }
}
