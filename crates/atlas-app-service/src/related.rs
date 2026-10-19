use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AppErrorCode, RelatedCategoryView, RelatedRecordsRequest, RelatedRecordsView,
};
use atlas_domain::RecordKey;
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, GraphContextRequest, RemasterLinksRequest,
};

const RELATED_OCCURRENCE_LIMIT: usize = 100;

impl AtlasAppService {
    /// One bounded secondary workflow. It never runs similarity ranking to discover categories.
    pub fn related_records(
        &self,
        request: RelatedRecordsRequest,
    ) -> AppServiceResult<RelatedRecordsView> {
        let selection = request.selection;
        crate::records::validate_request(&atlas_app_model::RecordDetailRequest {
            spell_rank: None,
            record_key: selection.record_key.clone(),
            owners: selection.owners.clone(),
            fields: selection.field.iter().cloned().collect(),
            passage: selection.passage.clone(),
            source_fingerprint: selection.source_fingerprint.clone(),
        })?;
        let key = RecordKey::parse(&selection.record_key)
            .map_err(|e| AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string()))?;
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            let generation = r.artifact_generation()?;
            if request.artifact_generation != generation {
                return Err(AppServiceError::artifact_generation_changed(
                    &request.artifact_generation,
                    &generation,
                    atlas_app_model::AppRecoverableAction::ReloadRecord,
                ));
            }
            if crate::projection::navigation_fingerprint(&selection.owners, r.source_fingerprint())
                .is_some()
                && selection.source_fingerprint.as_deref() != Some(r.source_fingerprint())
            {
                return Err(AppServiceError::invalid_request(
                    "snapshot-local related navigation belongs to another artifact",
                ));
            }
            if !selection.owners.is_empty() || selection.field.is_some() {
                let detail = r
                    .get_record(GetRecordRequest {
                        record_key: &key,
                        selected_content: &[],
                    })?
                    .ok_or_else(|| {
                        AppServiceError::new(AppErrorCode::RecordNotFound, "related seed not found")
                    })?;
                let node = detail.source.node_at(&selection.owners).ok_or_else(|| {
                    AppServiceError::invalid_request("related owner is unavailable")
                })?;
                if let Some(field) = &selection.field
                    && !node
                        .content_selections()
                        .iter()
                        .any(|s| s.field == field.as_str())
                {
                    return Err(AppServiceError::invalid_request(
                        "related selection field is unavailable",
                    ));
                }
            }
            let seed = r
                .get_records(GetRecordsRequest {
                    record_keys: std::slice::from_ref(&key),
                })?
                .records
                .into_iter()
                .next()
                .ok_or_else(|| {
                    AppServiceError::new(AppErrorCode::RecordNotFound, "related seed not found")
                })?;
            let references = category(
                r.graph_context(GraphContextRequest {
                    seed: key.clone(),
                    owners: Some(selection.owners.clone()),
                    outgoing_limit: RELATED_OCCURRENCE_LIMIT,
                    backlink_limit: RELATED_OCCURRENCE_LIMIT,
                })
                .map_err(AppServiceError::from)
                .and_then(|result| {
                    let g = result.ok_or_else(|| {
                        AppServiceError::new(AppErrorCode::RecordNotFound, "related seed not found")
                    })?;
                    Ok(
                        if g.outgoing.occurrences.is_empty() && g.backlinks.occurrences.is_empty() {
                            RelatedCategoryView::Empty
                        } else {
                            RelatedCategoryView::Content {
                                content: atlas_app_model::GraphContextView {
                                    seed: crate::projection::localized_summary_view(&g.seed, r),
                                    outgoing: crate::projection::graph_section_view(g.outgoing, r),
                                    backlinks: crate::projection::graph_section_view(
                                        g.backlinks,
                                        r,
                                    ),
                                },
                            }
                        },
                    )
                }),
            );
            let (counterparts, variants) = if selection.owners.is_empty() {
                let counterparts = category(
                    r.remaster_links(RemasterLinksRequest { record_key: &key })
                        .map_err(AppServiceError::from)
                        .and_then(|result| {
                            let g = result.ok_or_else(|| {
                                AppServiceError::new(
                                    AppErrorCode::RecordNotFound,
                                    "related seed not found",
                                )
                            })?;
                            Ok(if g.links.is_empty() {
                                RelatedCategoryView::Empty
                            } else {
                                RelatedCategoryView::Content {
                                    content: atlas_app_model::RemasterLinksView {
                                        seed: crate::projection::localized_summary_view(&g.seed, r),
                                        links: g
                                            .links
                                            .into_iter()
                                            .map(|l| atlas_app_model::RemasterLinkView {
                                                legacy_record:
                                                    crate::projection::localized_summary_view(
                                                        &l.legacy_record,
                                                        r,
                                                    ),
                                                remaster_record:
                                                    crate::projection::localized_summary_view(
                                                        &l.remaster_record,
                                                        r,
                                                    ),
                                                evidence: l.evidence,
                                            })
                                            .collect(),
                                    },
                                }
                            })
                        }),
                );
                (
                    counterparts,
                    category(crate::retrieval::variant_group_view(r, &key)),
                )
            } else {
                (
                    RelatedCategoryView::Unavailable {
                        reason: "Counterparts apply to root records".into(),
                    },
                    RelatedCategoryView::Unavailable {
                        reason: "Variant verification applies to root records".into(),
                    },
                )
            };
            Ok(RelatedRecordsView {
                selection,
                artifact_generation: generation,
                seed: crate::projection::localized_summary_view(&seed, r),
                references,
                counterparts,
                variants,
            })
        })
    }
}

fn category<T>(result: AppServiceResult<RelatedCategoryView<T>>) -> RelatedCategoryView<T> {
    result.unwrap_or_else(|error| RelatedCategoryView::Error {
        error: error.into_app_error(),
    })
}
