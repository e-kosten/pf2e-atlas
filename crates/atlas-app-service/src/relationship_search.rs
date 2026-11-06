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
pub(crate) fn record_destination(
    r: &AtlasRetrievalService,
    selection: &atlas_domain::QueryRecordSelection,
) -> atlas_app_model::ReferenceDestinationView {
    atlas_app_model::ReferenceDestinationView::Record {
        navigation: Box::new(atlas_app_model::RecordNavigationView {
            record_key: selection.record_key.to_string(),
            owners: selection.owners.clone(),
            field: selection.field.clone(),
            passage: selection.passage.clone(),
            source_fingerprint: crate::projection::navigation_fingerprint(
                &selection.owners,
                r.source_fingerprint(),
            ),
            spell_rank: selection.spell_rank,
        }),
    }
}
fn root_selection(
    key: RecordKey,
    owners: Vec<atlas_domain::OwnedContentLocator>,
) -> atlas_domain::QueryRecordSelection {
    atlas_domain::QueryRecordSelection {
        record_key: key,
        owners,
        field: None,
        passage: None,
        source_fingerprint: None,
        spell_rank: None,
    }
}
pub(crate) fn clause_summary(
    r: &AtlasRetrievalService,
    id: &str,
    condition: &atlas_app_model::ReferenceClauseConditionView,
) -> AppServiceResult<atlas_app_model::RelationshipClauseSummaryView> {
    use atlas_app_model::ReferenceClauseConditionView as Condition;
    let (record, title, destination) = match condition {
        Condition::Record { relationship } => {
            let checked = r.checked_relationship_selection(&relationship.selection)?;
            let record = crate::projection::localized_summary_view(&checked.record, r);
            let title = checked
                .selected_name
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| {
                    if relationship.selection.owners.is_empty() {
                        "Unnamed record".into()
                    } else {
                        "Unnamed entry".into()
                    }
                });
            (
                Some(Box::new(record)),
                title,
                Some(record_destination(r, &relationship.selection)),
            )
        }
        Condition::Definition { reference } => {
            match r.checked_definition_reference_target(&reference.target)? {
                atlas_search::CheckedDefinitionReferenceTarget::Record {
                    record,
                    selected_name,
                } => {
                    let atlas_domain::QueryDefinitionReferenceTarget::Record { selection } =
                        &reference.target
                    else {
                        return Err(AppServiceError::new(
                            AppErrorCode::InternalError,
                            "Definition target metadata disagrees with its identity",
                        ));
                    };
                    let title = selected_name
                        .filter(|title| !title.trim().is_empty())
                        .unwrap_or_else(|| {
                            if selection.owners.is_empty() {
                                "Unnamed record".into()
                            } else {
                                "Unnamed entry".into()
                            }
                        });
                    (
                        Some(Box::new(crate::projection::localized_summary_view(
                            &record, r,
                        ))),
                        title,
                        Some(record_destination(r, selection)),
                    )
                }
                atlas_search::CheckedDefinitionReferenceTarget::Definition { definition } => {
                    let destination = crate::definitions::definition_reading_destination(
                        &definition,
                        r.source_fingerprint(),
                    );
                    let record = match &definition.backing {
                        atlas_search::ReferenceSummaryBacking::Record { key } => {
                            let summary = r
                                .get_records(GetRecordsRequest {
                                    record_keys: std::slice::from_ref(key),
                                })?
                                .records
                                .into_iter()
                                .next()
                                .ok_or_else(|| {
                                    AppServiceError::new(
                                        AppErrorCode::RecordNotFound,
                                        "The definition record is unavailable",
                                    )
                                })?;
                            Some(Box::new(crate::projection::localized_summary_view(
                                &summary, r,
                            )))
                        }
                        _ => None,
                    };
                    (record, definition.label, destination)
                }
            }
        }
    };
    Ok(atlas_app_model::RelationshipClauseSummaryView {
        clause_id: id.into(),
        condition: condition.clone(),
        destination,
        record,
        title,
    })
}
fn conditions(
    filter: &atlas_domain::QueryPredicate,
) -> AppServiceResult<Vec<(&str, atlas_app_model::ReferenceClauseConditionView)>> {
    Ok(atlas_domain::reference_clauses(filter)?
        .into_iter()
        .map(|(id, clause)| {
            (
                id,
                match clause {
                    atlas_domain::QueryReferenceClause::Record(relationship) => {
                        atlas_app_model::ReferenceClauseConditionView::Record {
                            relationship: Box::new(relationship.clone()),
                        }
                    }
                    atlas_domain::QueryReferenceClause::Definition(reference) => {
                        atlas_app_model::ReferenceClauseConditionView::Definition {
                            reference: reference.clone(),
                        }
                    }
                },
            )
        })
        .collect())
}
pub(crate) fn prepare_filter(
    r: &AtlasRetrievalService,
    filter: Option<&atlas_domain::QueryPredicate>,
) -> AppServiceResult<Vec<atlas_app_model::RelationshipClauseSummaryView>> {
    let Some(filter) = filter else {
        return Ok(vec![]);
    };
    r.validate_filter(filter)?;
    conditions(filter)?
        .into_iter()
        .map(|(id, condition)| clause_summary(r, id, &condition))
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
    conditions(filter)?
        .into_iter()
        .map(|(id, condition)| match clause_summary(r, id, &condition) {
            Ok(summary) => Ok(View::Available {
                clause_id: summary.clause_id,
                condition: summary.condition,
                destination: summary.destination,
                record: summary.record,
                title: summary.title,
            }),
            Err(error) => unavailable_clause(id, &condition, error),
        })
        .collect()
}
fn unavailable_clause(
    id: &str,
    condition: &atlas_app_model::ReferenceClauseConditionView,
    error: AppServiceError,
) -> AppServiceResult<atlas_app_model::RelationshipClausePresentationView> {
    let error = error.into_app_error();
    match error.code {
        AppErrorCode::InvalidRequest
        | AppErrorCode::InvalidRecordKey
        | AppErrorCode::RecordNotFound => Ok(
            atlas_app_model::RelationshipClausePresentationView::Unavailable {
                clause_id: id.into(),
                condition: condition.clone(),
                message: "This reference is unavailable. Open it again or remove this condition."
                    .into(),
            },
        ),
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

fn canonical_evidence_views(
    r: &AtlasRetrievalService,
    occurrences: Vec<atlas_search::DefinitionReferenceInput>,
) -> AppServiceResult<Vec<atlas_app_model::ReferenceOccurrenceView>> {
    use atlas_search::{DefinitionReferenceSource as Source, DefinitionReferenceTarget as Target};
    let mut locations = Vec::new();
    let mut definitions = std::collections::BTreeSet::new();
    for occurrence in &occurrences {
        match &occurrence.source {
            Source::Record { key, owners, .. } => {
                locations.push(location(key.clone(), owners.clone()))
            }
            Source::Definition { key } => {
                definitions.insert(key.clone());
            }
        }
        match &occurrence.target {
            Target::Record { key, owners } => locations.push(location(key.clone(), owners.clone())),
            Target::Definition { key } => {
                definitions.insert(key.clone());
            }
        }
    }
    let keys = locations
        .iter()
        .map(|location| location.record.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let records = r
        .get_records(GetRecordsRequest { record_keys: &keys })?
        .records;
    let owned = r.owned_search_labels(&locations)?;
    let mut definitions_by_key = std::collections::BTreeMap::new();
    for chunk in definitions.into_iter().collect::<Vec<_>>().chunks(100) {
        for summary in r.reference_summaries(chunk)?.entries {
            definitions_by_key.insert(summary.key.clone(), summary);
        }
    }
    let title = |target: &atlas_domain::QueryDefinitionReferenceTarget| match target {
        atlas_domain::QueryDefinitionReferenceTarget::Definition { key } => definitions_by_key
            .get(key)
            .map(|summary| summary.label.clone())
            .unwrap_or_else(|| "Unavailable definition".into()),
        atlas_domain::QueryDefinitionReferenceTarget::Record { selection } => {
            if !selection.owners.is_empty() {
                owned
                    .iter()
                    .find(|(key, owners, _)| {
                        key == &selection.record_key && owners == &selection.owners
                    })
                    .map(|(_, _, name)| name.clone())
                    .unwrap_or_else(|| "Unnamed entry".into())
            } else {
                records
                    .iter()
                    .find(|record| record.key == selection.record_key)
                    .and_then(|record| record.name.as_value())
                    .filter(|name| !name.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| "Unavailable record".into())
            }
        }
    };
    Ok(occurrences
        .into_iter()
        .map(|occurrence| {
            let (source, source_field, source_destination) = match occurrence.source {
                Source::Record { key, owners, field } => {
                    let mut selection = root_selection(key, owners);
                    selection.field = field.clone();
                    selection.source_fingerprint = crate::projection::navigation_fingerprint(
                        &selection.owners,
                        r.source_fingerprint(),
                    );
                    let destination = record_destination(r, &selection);
                    (
                        atlas_domain::QueryDefinitionReferenceTarget::Record {
                            selection: Box::new(selection),
                        },
                        field,
                        Some(destination),
                    )
                }
                Source::Definition { key } => {
                    let destination = definitions_by_key.get(&key).and_then(|summary| {
                        crate::definitions::definition_reading_destination(
                            summary,
                            r.source_fingerprint(),
                        )
                    });
                    (
                        atlas_domain::QueryDefinitionReferenceTarget::Definition { key },
                        None,
                        destination,
                    )
                }
            };
            let target =
                crate::definitions::reference_target(occurrence.target, r.source_fingerprint());
            let target_destination =
                crate::definitions::reference_reading_destination(&target, &definitions_by_key, r);
            atlas_app_model::ReferenceOccurrenceView::Definition {
                occurrence: Box::new(atlas_app_model::DefinitionReferenceEvidenceView {
                    source_title: title(&source),
                    source,
                    source_destination,
                    source_field,
                    target_title: title(&target),
                    target,
                    target_destination,
                    occurrence_path: occurrence.occurrence_path,
                    evidence: occurrence.evidence,
                    label: occurrence.label,
                    visible: occurrence.visible,
                }),
            }
        })
        .collect())
}

pub(crate) fn evidence_page(
    r: &AtlasRetrievalService,
    filter: &atlas_domain::QueryPredicate,
    summary: &atlas_app_model::RelationshipClauseSummaryView,
    result: &RecordKey,
    page: SearchPage,
) -> AppServiceResult<(
    Vec<atlas_app_model::ReferenceOccurrenceView>,
    atlas_app_model::SearchPageView,
)> {
    match &summary.condition {
        atlas_app_model::ReferenceClauseConditionView::Record { relationship } => {
            let (occurrences, info) = r.relationship_occurrence_page(relationship, result, page)?;
            Ok((
                evidence_views(r, occurrences)?
                    .into_iter()
                    .map(graph_evidence)
                    .collect::<AppServiceResult<Vec<_>>>()?,
                crate::projection::search_page_view(info),
            ))
        }
        atlas_app_model::ReferenceClauseConditionView::Definition { .. } => {
            let (occurrences, info) =
                r.definition_reference_evidence(filter, &summary.clause_id, result, page)?;
            let mut ordered = Vec::new();
            let mut graphs = Vec::new();
            let mut canonical = Vec::new();
            for occurrence in occurrences {
                match occurrence {
                    atlas_search::DefinitionReferenceOccurrence::Record { occurrence } => {
                        ordered.push(false);
                        graphs.push(*occurrence);
                    }
                    atlas_search::DefinitionReferenceOccurrence::Definition { occurrence } => {
                        ordered.push(true);
                        canonical.push(*occurrence);
                    }
                }
            }
            let mut graphs = evidence_views(r, graphs)?.into_iter();
            let mut canonical = canonical_evidence_views(r, canonical)?.into_iter();
            let occurrences = ordered
                .into_iter()
                .map(|definition| {
                    if definition {
                        canonical.next().ok_or_else(|| {
                            AppServiceError::new(
                                AppErrorCode::InternalError,
                                "Definition evidence projection is incomplete",
                            )
                        })
                    } else {
                        graphs.next().map(graph_evidence).ok_or_else(|| {
                            AppServiceError::new(
                                AppErrorCode::InternalError,
                                "Record evidence projection is incomplete",
                            )
                        })?
                    }
                })
                .collect::<AppServiceResult<Vec<_>>>()?;
            Ok((occurrences, crate::projection::search_page_view(info)))
        }
    }
}

fn graph_evidence(
    occurrence: atlas_app_model::RecordRelationshipView,
) -> AppServiceResult<atlas_app_model::ReferenceOccurrenceView> {
    let navigation = occurrence.target.clone().ok_or_else(|| {
        AppServiceError::new(
            AppErrorCode::InternalError,
            "Resolved reference evidence has no record destination",
        )
    })?;
    Ok(atlas_app_model::ReferenceOccurrenceView::Record {
        occurrence: Box::new(occurrence),
        destination: Box::new(atlas_app_model::ReferenceDestinationView::Record {
            navigation: navigation.into(),
        }),
    })
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
            let (occurrences, page) = evidence_page(r, &request.filter, &summary, &key, page)?;
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
                &atlas_app_model::ReferenceClauseConditionView::Record {
                    relationship: Box::new(relationship.clone()),
                },
                AppServiceError::new(code, "workspace failure"),
            )
            .unwrap_err()
            .into_app_error();
            assert_eq!(error.code, code);
            assert_eq!(error.message, "workspace failure");
        }
    }
}
