use crate::{RecordScope, SearchPage, TextSearchResult};
use atlas_domain::{QueryPredicate, RecordKey};
pub struct SimilarRecordRequest<'a> {
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
        let Some(seed) = self.index.read_summary(request.record_key)? else {
            return Ok(None);
        };
        let vector = self
            .index
            .seed_identity_vector(request.record_key)?
            .ok_or_else(|| {
                crate::SearchError::vector_readiness_required(
                    "seed has no stored root identity vector; rebuild with embeddings",
                )
            })?;
        let predicate = self.predicate(request.filter)?;
        let (roots, window) = self.rank_candidates(crate::text::CandidateRequest {
            text: "",
            mode: crate::RetrievalMode::Vector,
            query: &predicate,
            scope: request.scope,
            prefer: request.prefer_remaster,
            suppress: true,
            vector: Some(&vector.vector),
            exclude: Some(request.record_key),
        })?;
        let results = self.finish_results(roots, request.page, true, window)?;
        Ok(Some(SimilarRecordResult { seed, results }))
    }
}
