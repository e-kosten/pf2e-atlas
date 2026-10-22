//! Reference search workflows and human display projection over typed retrieval.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppErrorCode, AppRecoverableAction, RelationshipOccurrencesPage, RelationshipOccurrencesRequest,
};
use atlas_domain::RecordKey;
use atlas_search::{
    AtlasRetrievalService, GetRecordsRequest, SearchPage, SourceStoredRelationship,
    SourceUnitLocation,
};

pub(crate) fn query_selection(
    selection: &atlas_app_model::RecordNavigationView,
) -> AppServiceResult<atlas_domain::QueryRecordSelection> {
    Ok(atlas_domain::QueryRecordSelection {
        record_key: RecordKey::parse(&selection.record_key).map_err(|_| {
            AppServiceError::invalid_request("The relationship record is unavailable")
        })?,
        owners: selection.owners.clone(),
        field: selection.field.clone(),
        passage: selection.passage.clone(),
        source_fingerprint: selection.source_fingerprint.clone(),
        spell_rank: selection.spell_rank,
    })
}
fn clause_summary(
    r: &AtlasRetrievalService,
    id: &str,
    relationship: &atlas_domain::QueryRelationship,
) -> AppServiceResult<atlas_app_model::RelationshipClauseSummaryView> {
    let checked = r.checked_relationship_selection(&relationship.selection)?;
    let mut record = crate::projection::localized_summary_view(&checked.record, r);
    if record.title.trim().is_empty() {
        record.title = "Unnamed record".into();
    }
    let title = checked
        .selected_name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if relationship.selection.owners.is_empty() {
                "Unnamed record".into()
            } else {
                "Unnamed entry".into()
            }
        });
    Ok(atlas_app_model::RelationshipClauseSummaryView {
        clause_id: id.into(),
        relationship: relationship.clone(),
        record,
        title,
    })
}
pub(crate) fn prepare_filter(
    r: &AtlasRetrievalService,
    filter: Option<&atlas_domain::QueryPredicate>,
) -> AppServiceResult<Vec<atlas_app_model::RelationshipClauseSummaryView>> {
    let Some(filter) = filter else {
        return Ok(vec![]);
    };
    r.validate_filter(filter)?;
    atlas_domain::relationship_clauses(filter)?
        .into_iter()
        .map(|(id, relationship)| clause_summary(r, id, relationship))
        .collect()
}
pub(crate) fn presentations(
    r: &AtlasRetrievalService,
    filter: Option<&atlas_domain::QueryPredicate>,
) -> AppServiceResult<Vec<atlas_app_model::RelationshipClausePresentationView>> {
    use atlas_app_model::RelationshipClausePresentationView as View;
    let Some(filter) = filter else {
        return Ok(vec![]);
    };
    atlas_domain::relationship_clauses(filter)?
        .into_iter()
        .map(
            |(id, relationship)| match clause_summary(r, id, relationship) {
                Ok(summary) => Ok(View::Available {
                    clause_id: summary.clause_id,
                    relationship: summary.relationship,
                    record: summary.record,
                    title: summary.title,
                }),
                Err(error) => unavailable_clause(id, relationship, error),
            },
        )
        .collect()
}

fn unavailable_clause(
    id: &str,
    relationship: &atlas_domain::QueryRelationship,
    error: AppServiceError,
) -> AppServiceResult<atlas_app_model::RelationshipClausePresentationView> {
    let error = error.into_app_error();
    match error.code {
        AppErrorCode::InvalidRequest | AppErrorCode::InvalidRecordKey | AppErrorCode::RecordNotFound =>
            Ok(atlas_app_model::RelationshipClausePresentationView::Unavailable {
                clause_id: id.into(),
                relationship: relationship.clone(),
                message: "This linked entry is unavailable. Open the entry again or remove this condition.".into(),
            }),
        _ => Err(error.into()),
    }
}

fn location(key: RecordKey, owners: Vec<atlas_domain::OwnedContentLocator>) -> SourceUnitLocation {
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
                v
            })
            .collect(),
    )
}

pub(crate) fn evidence_page(
    r: &AtlasRetrievalService,
    relationship: &atlas_domain::QueryRelationship,
    result: &RecordKey,
    page: SearchPage,
) -> AppServiceResult<(
    Vec<atlas_app_model::RecordRelationshipView>,
    atlas_app_model::SearchPageView,
)> {
    let (occurrences, info) = r.relationship_occurrence_page(relationship, result, page)?;
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
            let summaries = prepare_filter(r, Some(&request.filter))?;
            let id = atlas_domain::select_relationship_clause_id(
                &request.filter,
                Some(&request.clause_id),
            )?;
            let summary = summaries
                .into_iter()
                .find(|s| s.clause_id == id)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Choose a link condition from this filter")
                })?;
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
            if !r
                .relationship_proofs(&request.filter, std::slice::from_ref(&key))?
                .iter()
                .any(|(_, clause)| clause == &id)
            {
                return Err(AppServiceError::invalid_request(
                    "This condition does not provide a matching link for this record",
                ));
            }
            let (occurrences, page) = evidence_page(r, &summary.relationship, &key, page)?;
            Ok(RelationshipOccurrencesPage {
                artifact_generation: generation,
                clause: summary,
                result_record: crate::projection::localized_summary_view(&result, r),
                occurrences,
                page,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relationship_presentation_preserves_infrastructure_errors() {
        let relationship = atlas_domain::QueryRelationship {
            direction: atlas_domain::QueryRelationshipDirection::Incoming,
            selection: atlas_domain::QueryRecordSelection {
                record_key: RecordKey::parse("items:aaaaaaaaaaaaaaaa").unwrap(),
                owners: vec![],
                field: None,
                passage: None,
                source_fingerprint: None,
                spell_rank: None,
            },
            include_contents: true,
        };
        for code in [
            AppErrorCode::InternalError,
            AppErrorCode::ServiceBusy,
            AppErrorCode::ArtifactIncompatible,
        ] {
            let error = unavailable_clause(
                "link",
                &relationship,
                AppServiceError::new(code, "workspace failure"),
            )
            .unwrap_err()
            .into_app_error();
            assert_eq!(error.code, code);
            assert_eq!(error.message, "workspace failure");
        }
    }
}
