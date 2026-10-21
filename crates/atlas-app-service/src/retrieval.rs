use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::AppErrorCode;
use atlas_domain::{QueryPredicate, RecordKey};
use atlas_search::{
    GraphContextRequest, ListRecordsRequest, RecordRefResolutionResult, RemasterLinksRequest,
    ResolveRecordRefRequest, SearchPage, SimilarRecordRequest, VariantGroupRequest,
};
impl AtlasAppService {
    pub fn list_records(
        &self,
        filter: Option<QueryPredicate>,
        page: atlas_app_model::SearchPageRequest,
    ) -> AppServiceResult<atlas_app_model::RecordListView> {
        let page = SearchPage::new(page.number, page.size)?;
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            let result = r.list_records(ListRecordsRequest::new(filter.as_ref(), page))?;
            Ok(atlas_app_model::RecordListView {
                records: result
                    .records
                    .iter()
                    .map(|s| crate::projection::localized_summary_view(s, r))
                    .collect(),
                page: crate::projection::search_page_view(result.page),
            })
        })
    }
    pub fn similar_records(
        &self,
        record_ref: String,
        sort: Option<atlas_domain::ResultSort>,
        filter: Option<QueryPredicate>,
        page: atlas_app_model::SearchPageRequest,
        search_profile: Option<atlas_domain::SearchProfileId>,
    ) -> AppServiceResult<Option<atlas_app_model::SimilarRecordsView>> {
        let page = SearchPage::new(page.number, page.size)?;
        let sort = sort.unwrap_or(atlas_domain::ResultSort::Relevance);
        if sort == atlas_domain::ResultSort::SavedOrder {
            return Err(AppServiceError::invalid_request(
                "Saved order is available only for saved lists",
            ));
        }
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            let key = match r.resolve_record_ref(ResolveRecordRefRequest {
                record_ref: &record_ref,
                filter: None,
            })? {
                RecordRefResolutionResult::Key(key) => key,
                RecordRefResolutionResult::Miss => {
                    return Err(AppServiceError::new(
                        AppErrorCode::RecordResolutionMiss,
                        "similar seed not found",
                    ));
                }
                RecordRefResolutionResult::Ambiguous(_) => {
                    return Err(AppServiceError::new(
                        AppErrorCode::RecordResolutionAmbiguous,
                        "similar seed is ambiguous",
                    ));
                }
            };
            Ok(r.similar_records(SimilarRecordRequest {
                sort,
                search_profile: search_profile.as_ref(),
                record_key: &key,
                filter: filter.as_ref(),
                scope: atlas_search::RecordScope::All,
                page,
                prefer_remaster: true,
            })?
            .map(
                |s| -> AppServiceResult<atlas_app_model::SimilarRecordsView> {
                    let mut results = crate::projection::search_results_view(s.results, r);
                    if sort.is_unit_price() {
                        let mut records = results
                            .rows
                            .iter()
                            .map(|row| row.record.clone())
                            .collect::<Vec<_>>();
                        crate::result_order::price_labels(r, &mut records)?;
                        for (row, record) in results.rows.iter_mut().zip(records) {
                            row.record = record;
                        }
                    }
                    Ok(atlas_app_model::SimilarRecordsView {
                        seed: crate::projection::localized_summary_view(&s.seed, r),
                        results,
                    })
                },
            )
            .transpose()?)
        })
    }
    pub fn graph_context(
        &self,
        request: atlas_app_model::GraphContextViewRequest,
    ) -> AppServiceResult<Option<atlas_app_model::GraphContextView>> {
        let key = RecordKey::parse(&request.record_key)
            .map_err(|e| AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string()))?;
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            if let Some(owners) = &request.owners {
                if crate::projection::navigation_fingerprint(owners, r.source_fingerprint())
                    .is_some()
                    && request.source_fingerprint.as_deref() != Some(r.source_fingerprint())
                {
                    return Err(AppServiceError::invalid_request(
                        "snapshot-local graph navigation belongs to another artifact",
                    ));
                }
                let detail = r
                    .get_record(atlas_search::GetRecordRequest {
                        record_key: &key,
                        selected_content: &[],
                    })?
                    .ok_or_else(|| {
                        AppServiceError::new(AppErrorCode::RecordNotFound, "graph seed not found")
                    })?;
                if detail.source.node_at(owners).is_none() {
                    return Err(AppServiceError::invalid_request(
                        "graph selection is unavailable",
                    ));
                }
            }
            let request = GraphContextRequest {
                seed: key,
                owners: request.owners,
                outgoing_limit: request.outgoing_limit,
                backlink_limit: request.backlink_limit,
            };

            Ok(r.graph_context(request)?
                .map(|g| atlas_app_model::GraphContextView {
                    seed: crate::projection::localized_summary_view(&g.seed, r),
                    outgoing: crate::projection::graph_section_view(g.outgoing, r),
                    backlinks: crate::projection::graph_section_view(g.backlinks, r),
                }))
        })
    }
    pub fn variant_group(
        &self,
        record_key: RecordKey,
    ) -> AppServiceResult<atlas_app_model::RelatedCategoryView<atlas_app_model::VariantGroupView>>
    {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            variant_group_view(r, &record_key)
        })
    }
    pub fn remaster_links(
        &self,
        record_key: RecordKey,
    ) -> AppServiceResult<Option<atlas_app_model::RemasterLinksView>> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            Ok(r.remaster_links(RemasterLinksRequest {
                record_key: &record_key,
            })?
            .map(|g| atlas_app_model::RemasterLinksView {
                seed: crate::projection::localized_summary_view(&g.seed, r),
                links: g
                    .links
                    .into_iter()
                    .map(|l| atlas_app_model::RemasterLinkView {
                        legacy_record: crate::projection::localized_summary_view(
                            &l.legacy_record,
                            r,
                        ),
                        remaster_record: crate::projection::localized_summary_view(
                            &l.remaster_record,
                            r,
                        ),
                        evidence: l.evidence,
                    })
                    .collect(),
            }))
        })
    }
}

pub(crate) fn retrieval_mode(
    mode: atlas_app_model::RetrievalModeView,
) -> atlas_search::RetrievalMode {
    match mode {
        atlas_app_model::RetrievalModeView::Lexical => atlas_search::RetrievalMode::Fts,
        atlas_app_model::RetrievalModeView::Semantic => atlas_search::RetrievalMode::Vector,
        atlas_app_model::RetrievalModeView::Hybrid => atlas_search::RetrievalMode::Hybrid,
    }
}

pub(crate) fn variant_group_view(
    r: &atlas_search::AtlasRetrievalService,
    record_key: &RecordKey,
) -> AppServiceResult<atlas_app_model::RelatedCategoryView<atlas_app_model::VariantGroupView>> {
    use atlas_app_model::RelatedCategoryView;
    Ok(match r.variant_group(VariantGroupRequest { record_key })? {
        atlas_search::VariantGroupOutcome::Missing => {
            return Err(AppServiceError::new(
                AppErrorCode::RecordNotFound,
                "variant seed not found",
            ));
        }
        atlas_search::VariantGroupOutcome::Empty => RelatedCategoryView::Empty,
        atlas_search::VariantGroupOutcome::Unavailable { reason } => {
            RelatedCategoryView::Unavailable { reason }
        }
        atlas_search::VariantGroupOutcome::Content(g) => RelatedCategoryView::Content {
            content: atlas_app_model::VariantGroupView {
                seed: crate::projection::localized_summary_view(&g.seed, r),
                base_name: g.base_name,
                variants: g
                    .variants
                    .iter()
                    .map(|s| crate::projection::localized_summary_view(s, r))
                    .collect(),
                evidence: g
                    .evidence
                    .into_iter()
                    .map(|e| atlas_app_model::VariantEvidenceView {
                        record_key: e.record.to_string(),
                        naming_convention: e.naming_convention,
                        qualifier: e.qualifier,
                        compatibility: e.compatibility,
                    })
                    .collect(),
                ambiguous: g.ambiguous,
                truncated: g.truncated,
            },
        },
    })
}
