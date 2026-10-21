//! Reference search workflows and human display projection over typed retrieval.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppErrorCode, AppRecoverableAction, RelationshipConstraintSummaryView,
    RelationshipConstraintView, RelationshipDirectionView, RelationshipOccurrencesPage,
    RelationshipOccurrencesRequest,
};
use atlas_domain::RecordKey;
use atlas_search::{
    AtlasRetrievalService, GetRecordRequest, GetRecordsRequest, RelationshipConstraint, SearchPage,
    SourceRelationshipDirection, SourceStoredRelationship, SourceUnitLocation,
};

pub(crate) fn prepare_constraint(
    r: &AtlasRetrievalService,
    view: &RelationshipConstraintView,
) -> AppServiceResult<(RelationshipConstraint, RelationshipConstraintSummaryView)> {
    crate::records::validate_request(&atlas_app_model::RecordDetailRequest {
        spell_rank: view.selection.spell_rank,
        record_key: view.selection.record_key.clone(),
        owners: view.selection.owners.clone(),
        fields: view.selection.field.iter().cloned().collect(),
        passage: view.selection.passage.clone(),
        source_fingerprint: view.selection.source_fingerprint.clone(),
    })?;
    if crate::projection::navigation_fingerprint(&view.selection.owners, r.source_fingerprint())
        .is_some()
        && view.selection.source_fingerprint.as_deref() != Some(r.source_fingerprint())
    {
        return Err(AppServiceError::invalid_request(
            "This selection belongs to an older reference library. Open the record again.",
        ));
    }
    let key = RecordKey::parse(&view.selection.record_key).map_err(|_| {
        AppServiceError::new(
            AppErrorCode::InvalidRecordKey,
            "The relationship record is unavailable",
        )
    })?;
    let summary = r
        .get_records(GetRecordsRequest {
            record_keys: std::slice::from_ref(&key),
        })?
        .records
        .into_iter()
        .next()
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::RecordNotFound,
                "The relationship record is unavailable. Open the record again.",
            )
        })?;
    let mut record = crate::projection::localized_summary_view(&summary, r);
    // Relationship labels never use the normal technical identity fallback.
    record.title = summary
        .name
        .as_value()
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "Unnamed record".into());
    let title = if view.selection.owners.is_empty() && view.selection.field.is_none() {
        record.title.clone()
    } else {
        let detail = r
            .get_record(GetRecordRequest {
                record_key: &key,
                selected_content: &[],
            })?
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::RecordNotFound,
                    "The relationship record is unavailable",
                )
            })?;
        let node = detail
            .source
            .node_at(&view.selection.owners)
            .ok_or_else(|| {
                AppServiceError::invalid_request(
                    "The selected entry is unavailable. Open the record again.",
                )
            })?;
        if let Some(field) = &view.selection.field
            && !node.content_selections().iter().any(|s| s.field == *field)
        {
            return Err(AppServiceError::invalid_request(
                "The selected text is unavailable. Open the record again.",
            ));
        }
        if view.selection.owners.is_empty() {
            record.title.clone()
        } else {
            node.name()
                .value()
                .filter(|v| !v.trim().is_empty())
                .map(|s| (*s).clone())
                .unwrap_or_else(|| "Unnamed entry".into())
        }
    };
    let constraint = RelationshipConstraint {
        direction: match view.direction {
            RelationshipDirectionView::Incoming => SourceRelationshipDirection::Incoming,
            RelationshipDirectionView::Outgoing => SourceRelationshipDirection::Outgoing,
        },
        record: key,
        owners: view.selection.owners.clone(),
        field: if view.direction == RelationshipDirectionView::Outgoing {
            view.selection.field.clone()
        } else {
            None
        },
        include_contents: view.include_contents,
    };
    Ok((
        constraint,
        RelationshipConstraintSummaryView {
            constraint: view.clone(),
            record,
            title,
        },
    ))
}

pub(crate) fn prepare_constraints(
    r: &AtlasRetrievalService,
    views: &[RelationshipConstraintView],
) -> AppServiceResult<(
    Vec<RelationshipConstraint>,
    Vec<RelationshipConstraintSummaryView>,
)> {
    if views.len() > 16 {
        return Err(AppServiceError::invalid_request(
            "Use at most 16 relationship filters",
        ));
    }
    views
        .iter()
        .map(|v| prepare_constraint(r, v))
        .collect::<AppServiceResult<Vec<_>>>()
        .map(|v| v.into_iter().unzip())
}

fn location(
    key: RecordKey,
    owners: Vec<atlas_record::source_content::OwnedContentLocator>,
) -> SourceUnitLocation {
    SourceUnitLocation {
        unit_id: 0,
        record: key,
        owners,
        field: None,
        address: None,
    }
}

pub(crate) fn evidence_views(
    r: &AtlasRetrievalService,
    occurrences: Vec<SourceStoredRelationship>,
) -> AppServiceResult<Vec<atlas_app_model::RecordRelationshipView>> {
    let mut locations = Vec::new();
    for o in &occurrences {
        locations.push(location(o.locator.record.clone(), o.locator.owners.clone()));
        match &o.resolution {
            atlas_record::source_content::ContentReferenceResolution::Resolved(
                atlas_record::source_content::ContentReferenceTarget::Record { key },
            ) => locations.push(location(key.clone(), vec![])),
            atlas_record::source_content::ContentReferenceResolution::Resolved(
                atlas_record::source_content::ContentReferenceTarget::OwnedNode { key, owners },
            ) => locations.push(location(key.clone(), owners.clone())),
            _ => {}
        }
    }
    let keys = locations
        .iter()
        .map(|l| l.record.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let summaries = r
        .get_records(GetRecordsRequest { record_keys: &keys })?
        .records;
    let labels = r.owned_search_labels(&locations)?;
    let label = |nav: &atlas_app_model::RecordNavigationView| -> String {
        if !nav.owners.is_empty() {
            return labels
                .iter()
                .find(|(key, owners, _)| key.to_string() == nav.record_key && owners == &nav.owners)
                .map(|(_, _, name)| name.clone())
                .unwrap_or_else(|| "Unnamed entry".into());
        }
        summaries
            .iter()
            .find(|s| s.key.to_string() == nav.record_key)
            .and_then(|s| s.name.as_value())
            .filter(|v| !v.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| "Unavailable record".into())
    };
    Ok(
        crate::projection::relationship_views(occurrences, r.source_fingerprint())
            .into_iter()
            .map(|mut v| {
                v.source_title = Some(label(&v.source_navigation));
                v.target_title = v.target.as_ref().map(label);
                v.origin_label = Some(
                    match v.origin.as_str() {
                        "content" => "Text reference",
                        "structured" if v.kind == "CompendiumProvenance" => "Compendium source",
                        _ => "Source reference",
                    }
                    .into(),
                );
                v
            })
            .collect(),
    )
}

pub(crate) fn evidence_page(
    r: &AtlasRetrievalService,
    constraint: &RelationshipConstraint,
    result: &RecordKey,
    page: SearchPage,
) -> AppServiceResult<(
    Vec<atlas_app_model::RecordRelationshipView>,
    atlas_app_model::SearchPageView,
)> {
    let (occurrences, info) = r.relationship_occurrence_page(constraint, result, page)?;
    Ok((
        evidence_views(r, occurrences)?,
        crate::projection::search_page_view(info),
    ))
}

impl AtlasAppService {
    pub fn relationship_occurrences(
        &self,
        request: RelationshipOccurrencesRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<RelationshipOccurrencesPage> {
        execution.check()?;
        let page = SearchPage::new(request.page.number, request.page.size)?;
        if request.page.size > 100 {
            return Err(AppServiceError::invalid_request(
                "Show at most 100 references per page",
            ));
        }
        self.with_retrieval(execution, move |r| {
            let generation = r.artifact_generation()?;
            if generation != request.expected_artifact_generation {
                return Err(AppServiceError::artifact_generation_changed(
                    &request.expected_artifact_generation,
                    &generation,
                    AppRecoverableAction::RefreshResults,
                ));
            }
            let (constraint, summary) = prepare_constraint(r, &request.constraint)?;
            let key = RecordKey::parse(&request.result_record_key).map_err(|_| {
                AppServiceError::new(
                    AppErrorCode::InvalidRecordKey,
                    "The result record is unavailable",
                )
            })?;
            let result = r
                .get_records(GetRecordsRequest {
                    record_keys: std::slice::from_ref(&key),
                })?
                .records
                .into_iter()
                .next()
                .ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::RecordNotFound,
                        "The result record is unavailable",
                    )
                })?;
            let (occurrences, page) = evidence_page(r, &constraint, &key, page)?;
            Ok(RelationshipOccurrencesPage {
                artifact_generation: generation,
                constraint: summary,
                result_record: crate::projection::localized_summary_view(&result, r),
                occurrences,
                page,
            })
        })
    }
}
