use crate::equipment_family::{family_choice, family_slice};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AppErrorCode, RelatedCategoryView, RelatedRecordsRequest, RelatedRecordsView, VariantFamilyView,
};
use atlas_domain::RecordKey;
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, GraphContextRequest, VariantFamilyContext,
    VariantGroupRequest,
};

const RELATED_OCCURRENCE_LIMIT: usize = 100;

impl AtlasAppService {
    /// One bounded secondary workflow. It never runs similarity ranking to discover categories.
    pub fn related_records(
        &self,
        request: RelatedRecordsRequest,
    ) -> AppServiceResult<RelatedRecordsView> {
        let selection = request.selection;
        crate::records::validate_address_bounds(&selection.owners)?;
        crate::records::validate_field_bounds(selection.field.as_slice())?;
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
            if !crate::records::address_fingerprint_matches(
                &selection.owners,
                selection.source_fingerprint.as_deref(),
                r.source_fingerprint(),
            ) {
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
                    AppServiceError::invalid_request("related selection is unavailable")
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
                                    outgoing: scoped_section(
                                        crate::projection::graph_section_view(g.outgoing, r),
                                        &selection,
                                        atlas_domain::QueryRelationshipDirection::Outgoing,
                                        false,
                                    )?,
                                    backlinks: scoped_section(
                                        crate::projection::graph_section_view(g.backlinks, r),
                                        &selection,
                                        atlas_domain::QueryRelationshipDirection::Incoming,
                                        true,
                                    )?,
                                },
                            }
                        },
                    )
                }),
            );
            let (remaster_relationships, source_remaster_context) =
                crate::remaster::reading_context(&selection, r);
            let (variants, family_context) = if selection.owners.is_empty() {
                match r.variant_exploration(VariantGroupRequest { record_key: &key }) {
                    Ok(exploration) => (
                        category(crate::retrieval::project_variant_outcome(
                            r,
                            exploration.variants,
                        )),
                        exploration.family.map(|context| family_view(r, context)),
                    ),
                    Err(error) => (category(Err(error.into())), None),
                }
            } else {
                (
                    RelatedCategoryView::Unavailable {
                        reason: "Variant verification applies to root records".into(),
                    },
                    None,
                )
            };
            let show_versions = family_context.is_some()
                || show_versions(
                    &selection,
                    &remaster_relationships,
                    source_remaster_context.as_ref(),
                    &variants,
                );
            Ok(RelatedRecordsView {
                show_versions,
                selection,
                artifact_generation: generation,
                seed: crate::projection::localized_summary_view(&seed, r),
                references,
                remaster_relationships,
                source_remaster_context,
                variants,
                family_context,
            })
        })
    }
}

fn family_view(
    r: &atlas_search::AtlasRetrievalService,
    context: VariantFamilyContext,
) -> VariantFamilyView {
    let keys = |keys: Vec<RecordKey>| keys.iter().map(ToString::to_string).collect();
    let summary = |record| crate::projection::localized_summary_view(record, r);
    match context {
        VariantFamilyContext::Concrete {
            slices,
            other_combinations,
            general,
        } => VariantFamilyView::Concrete {
            slices: slices.into_iter().map(family_slice).collect(),
            other_combinations: keys(other_combinations),
            general: general.as_deref().map(|record| Box::new(summary(record))),
        },
        VariantFamilyContext::General { heading, members } => VariantFamilyView::General {
            heading,
            members: members
                .into_iter()
                .map(|member| family_choice(r, member))
                .collect(),
        },
    }
}

fn distinct_address(
    left: &atlas_app_model::RecordNavigationView,
    right: &atlas_app_model::RecordNavigationView,
) -> bool {
    left.record_key != right.record_key || left.owners != right.owners
}
fn relationship_versions(
    category: &RelatedCategoryView<atlas_app_model::RemasterRelationshipsView>,
    selection: &atlas_app_model::RecordNavigationView,
) -> bool {
    match category {
        RelatedCategoryView::Content{content}=>content.relationships.iter().flat_map(|relationship|&relationship.parts).any(|part|matches!(part,atlas_app_model::RemasterInlinePartView::Reference{navigation,..} if distinct_address(navigation,selection))),
        RelatedCategoryView::Error{..}|RelatedCategoryView::Unavailable{..}=>true,
        RelatedCategoryView::Empty=>false,
    }
}
fn show_versions(
    selection: &atlas_app_model::RecordNavigationView,
    direct: &RelatedCategoryView<atlas_app_model::RemasterRelationshipsView>,
    source: Option<&RelatedCategoryView<atlas_app_model::RemasterSourceContextView>>,
    variants: &RelatedCategoryView<atlas_app_model::VariantGroupView>,
) -> bool {
    if relationship_versions(direct, selection) {
        return true;
    }
    match source {
        Some(RelatedCategoryView::Content { content }) => {
            let source_selection = &content.navigation;
            if relationship_versions(&content.relationships, source_selection) {
                return true;
            }
        }
        Some(RelatedCategoryView::Error { .. }) => return true,
        // An unidentified/conflicting origin is not a temporarily failed remaster lookup.
        Some(RelatedCategoryView::Unavailable { .. } | RelatedCategoryView::Empty) | None => {}
    }
    if !selection.owners.is_empty() {
        return false;
    }
    match variants {
        RelatedCategoryView::Content { content } => content
            .variants
            .iter()
            .any(|record| record.record_key != selection.record_key),
        RelatedCategoryView::Error { .. } => true,
        RelatedCategoryView::Empty | RelatedCategoryView::Unavailable { .. } => false,
    }
}

fn category<T>(result: AppServiceResult<RelatedCategoryView<T>>) -> RelatedCategoryView<T> {
    result.unwrap_or_else(|error| RelatedCategoryView::Error {
        error: error.into_app_error(),
    })
}

fn scoped_section(
    mut section: atlas_app_model::GraphSectionView,
    selection: &atlas_app_model::RecordNavigationView,
    direction: atlas_domain::QueryRelationshipDirection,
    include_contents: bool,
) -> AppServiceResult<atlas_app_model::GraphSectionView> {
    let mut selection = crate::relationship_search::query_selection(selection)?;
    selection.field = None;
    selection.passage = None;
    let mut filter =
        atlas_domain::QueryPredicate::new(atlas_domain::QueryExpression::Relationship {
            relationship: atlas_domain::QueryRelationship {
                direction,
                selection,
                include_contents,
            },
        });
    atlas_domain::assign_relationship_clause_ids(&mut filter)?;
    section.filter = Some(filter);
    Ok(section)
}

#[cfg(test)]
mod versions_tests {
    use super::*;
    use atlas_app_model::*;
    #[test]
    fn plain_former_name_and_journal_do_not_create_versions_but_an_actual_alternative_does() {
        let selection = RecordNavigationView {
            record_key: "items:aaaaaaaaaaaaaaaa".into(),
            owners: vec![],
            field: None,
            passage: None,
            spell_rank: None,
            source_fingerprint: None,
        };
        let mut content = RemasterRelationshipsView {
            relationships: vec![RemasterRelationshipView {
                id: "rename".into(),
                label: "Former name".into(),
                parts: vec![RemasterInlinePartView::Text {
                    text: "Magic Missile".into(),
                }],
                explanation_links: vec![RemasterExplanationLinkView {
                    label: "Remaster Changes — Spells".into(),
                    navigation: RecordNavigationView {
                        record_key: "journals:cccccccccccccccc".into(),
                        ..selection.clone()
                    },
                }],
                truncated: false,
            }],
            truncated: false,
        };
        assert!(!show_versions(
            &selection,
            &RelatedCategoryView::Content {
                content: content.clone()
            },
            None,
            &RelatedCategoryView::Empty
        ));
        content.relationships[0].parts = vec![RemasterInlinePartView::Reference {
            label: "Other record".into(),
            navigation: Box::new(RecordNavigationView {
                record_key: "items:bbbbbbbbbbbbbbbb".into(),
                ..selection.clone()
            }),
        }];
        assert!(show_versions(
            &selection,
            &RelatedCategoryView::Content { content },
            None,
            &RelatedCategoryView::Empty
        ));
    }
}
