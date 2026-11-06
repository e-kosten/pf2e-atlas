use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AppErrorCode, RelatedCategoryView, RelatedRecordsRequest, RelatedRecordsView,
};
use atlas_domain::RecordKey;
use atlas_search::{GetRecordRequest, GetRecordsRequest, GraphContextRequest};

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
                crate::remaster::reading_context(&selection, r, None);
            let variants = if selection.owners.is_empty() {
                category(crate::retrieval::variant_group_view(r, &key))
            } else {
                RelatedCategoryView::Unavailable {
                    reason: "Variant verification applies to root records".into(),
                }
            };
            let show_versions = show_versions(
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
            })
        })
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
            let source_selection = match &content.source.resolution {
                atlas_app_model::RemasterEndpointResolutionView::Current { navigation, .. } => {
                    navigation.clone()
                }
                atlas_app_model::RemasterEndpointResolutionView::Absent { record_key, .. } => {
                    atlas_app_model::RecordNavigationView {
                        record_key: record_key.clone(),
                        owners: vec![],
                        field: None,
                        passage: None,
                        spell_rank: None,
                        source_fingerprint: None,
                    }
                }
                _ => selection.clone(),
            };
            if relationship_versions(&content.relationships, &source_selection) {
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
    use serde_json::json;
    fn nav(key: &str) -> RecordNavigationView {
        RecordNavigationView {
            record_key: key.into(),
            owners: vec![],
            field: None,
            passage: None,
            spell_rank: None,
            source_fingerprint: None,
        }
    }
    fn summary(key: &str) -> RecordSummaryView {
        serde_json::from_value(
            json!({"record_key":key,"title":"Entry","kind":"rule","kind_label":"Rule","traits":[]}),
        )
        .unwrap()
    }
    fn notes(parts: Vec<RemasterInlinePartView>) -> RelatedCategoryView<RemasterRelationshipsView> {
        RelatedCategoryView::Content {
            content: RemasterRelationshipsView {
                relationships: vec![RemasterRelationshipView {
                    id: "relation".into(),
                    label: "Former name".into(),
                    parts,
                    declaration_ids: vec![],
                    explanation_links: vec![RemasterExplanationLinkView {
                        label: "Remaster Changes — Spells".into(),
                        navigation: nav("journals:aaaaaaaaaaaaaaaa"),
                    }],
                    truncated: false,
                }],
                declarations: vec![],
                truncated: false,
            },
        }
    }
    fn reference(key: &str) -> RemasterInlinePartView {
        RemasterInlinePartView::Reference {
            label: "Other entry".into(),
            navigation: Box::new(nav(key)),
        }
    }
    #[test]
    fn versions_ignores_force_cleanse_nephilim_names_explanations_and_self_addresses() {
        let selected = nav("items:aaaaaaaaaaaaaaaa");
        for names in [
            "Magic Missile",
            "Remove Curse, Remove Disease, and Neutralize Poison",
            "Aasimar, Aphorite, Ganzi, and Tiefling",
        ] {
            let direct = notes(vec![RemasterInlinePartView::Text { text: names.into() }]);
            assert!(!show_versions(
                &selected,
                &direct,
                None,
                &RelatedCategoryView::Empty
            ));
        }
        assert!(!show_versions(
            &selected,
            &notes(vec![reference(&selected.record_key)]),
            None,
            &RelatedCategoryView::Empty
        ));
        let mut owned = selected.clone();
        owned.owners = vec![atlas_domain::OwnedContentLocator {
            collection: "/items".into(),
            identity: atlas_domain::OwnedContentIdentity::Stable("child".into()),
        }];
        assert!(!show_versions(
            &owned,
            &RelatedCategoryView::Empty,
            None,
            &RelatedCategoryView::Unavailable {
                reason: "Variant verification applies to root records".into()
            }
        ));
    }
    #[test]
    fn versions_retains_real_glossary_counterparts_suggestions_and_lookup_retries() {
        let selected = nav("bestiary-ability-glossary-srd:hFtNbo1LKYCoDy2O");
        assert!(show_versions(
            &selected,
            &notes(vec![reference(
                "bestiary-ability-glossary-srd:W7SbTykXrNwxDzJc"
            )]),
            None,
            &RelatedCategoryView::Empty
        ));
        let variants = |keys: Vec<&str>| RelatedCategoryView::Content {
            content: VariantGroupView {
                seed: summary(&selected.record_key),
                base_name: "Entry".into(),
                variants: keys.into_iter().map(summary).collect(),
                evidence: vec![],
                ambiguous: false,
                truncated: false,
            },
        };
        assert!(!show_versions(
            &selected,
            &RelatedCategoryView::Empty,
            None,
            &variants(vec![&selected.record_key])
        ));
        assert!(show_versions(
            &selected,
            &RelatedCategoryView::Empty,
            None,
            &variants(vec!["items:bbbbbbbbbbbbbbbb"])
        ));
        assert!(show_versions(
            &selected,
            &RelatedCategoryView::Error {
                error: AppError::new(AppErrorCode::InternalError, "Read failed")
            },
            None,
            &RelatedCategoryView::Empty
        ));
        assert!(show_versions(
            &selected,
            &RelatedCategoryView::Unavailable {
                reason: "Temporarily unavailable".into()
            },
            None,
            &RelatedCategoryView::Empty
        ));
        assert!(show_versions(
            &selected,
            &RelatedCategoryView::Empty,
            None,
            &RelatedCategoryView::Error {
                error: AppError::new(AppErrorCode::InternalError, "Variants failed")
            }
        ));
    }
    #[test]
    fn versions_source_context_compares_against_its_own_exact_entry_and_keeps_errors() {
        let selected = nav("actors:aaaaaaaaaaaaaaaa");
        let source_key = "items:bbbbbbbbbbbbbbbb";
        let source = |relationships| RelatedCategoryView::Content {
            content: RemasterSourceContextView {
                heading: "Source entry".into(),
                source: RemasterEndpointView {
                    label: "Source entry".into(),
                    context: None,
                    resolution: RemasterEndpointResolutionView::Current {
                        record: Box::new(summary(source_key)),
                        navigation: nav(source_key),
                    },
                },
                relationships,
            },
        };
        assert!(!show_versions(
            &selected,
            &RelatedCategoryView::Empty,
            Some(&source(notes(vec![reference(source_key)]))),
            &RelatedCategoryView::Empty
        ));
        assert!(show_versions(
            &selected,
            &RelatedCategoryView::Empty,
            Some(&source(notes(vec![reference("items:cccccccccccccccc")]))),
            &RelatedCategoryView::Empty
        ));
        assert!(show_versions(
            &selected,
            &RelatedCategoryView::Empty,
            Some(&source(RelatedCategoryView::Error {
                error: AppError::new(AppErrorCode::InternalError, "Source relationships failed")
            })),
            &RelatedCategoryView::Empty
        ));
        assert!(show_versions(
            &selected,
            &RelatedCategoryView::Empty,
            Some(&RelatedCategoryView::Error {
                error: AppError::new(AppErrorCode::InternalError, "Origin failed")
            }),
            &RelatedCategoryView::Empty
        ));
    }
}
