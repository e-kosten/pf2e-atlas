use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::*;
use atlas_domain::RecordKey;
use atlas_record::{
    remaster::{RecordRemasterContext, RemasterRelation, RemasterTarget},
    source_content::{ContentReferenceResolution, ContentReferenceTarget},
};
use atlas_search::AtlasRetrievalService;
use std::collections::{BTreeMap, BTreeSet};
mod explanations;
fn parse_key(value: &str) -> AppServiceResult<RecordKey> {
    RecordKey::parse(value).map_err(|e| AppServiceError::invalid_request(e.to_string()))
}
fn category<T>(result: AppServiceResult<RelatedCategoryView<T>>) -> RelatedCategoryView<T> {
    result.unwrap_or_else(|e| RelatedCategoryView::Error {
        error: e.into_app_error(),
    })
}
pub(super) fn navigation(
    key: &RecordKey,
    owners: &[atlas_domain::OwnedContentLocator],
    retrieval: &AtlasRetrievalService,
) -> RecordNavigationView {
    RecordNavigationView {
        record_key: key.to_string(),
        owners: owners.to_vec(),
        field: None,
        passage: None,
        spell_rank: None,
        source_fingerprint: crate::projection::navigation_fingerprint(
            owners,
            retrieval.source_fingerprint(),
        ),
    }
}
pub(crate) fn reading_context(
    selection: &RecordNavigationView,
    retrieval: &AtlasRetrievalService,
) -> (
    RelatedCategoryView<RemasterRelationshipsView>,
    Option<RelatedCategoryView<RemasterSourceContextView>>,
) {
    (
        reading_relationships(selection, retrieval),
        source_context(selection, retrieval),
    )
}
fn reading_relationships(
    selection: &RecordNavigationView,
    retrieval: &AtlasRetrievalService,
) -> RelatedCategoryView<RemasterRelationshipsView> {
    category((|| {
        let key = parse_key(&selection.record_key)?;
        let mut contexts = retrieval.record_remaster_context(&key, &selection.owners)?;
        contexts.retain(|c| c.owners == selection.owners);
        if contexts.is_empty() {
            return Ok(RelatedCategoryView::Empty);
        }
        Ok(RelatedCategoryView::Content {
            content: project(contexts, selection, retrieval)?,
        })
    })())
}
fn relation(relation: RemasterRelation) -> (&'static str, u8, bool) {
    match relation {
        RemasterRelation::FormerName => ("Former name", 0, true),
        RemasterRelation::MergedFrom => ("Merged from", 1, true),
        RemasterRelation::Replaces => ("Replaces", 2, true),
        RemasterRelation::RenamedTo => ("Renamed to", 0, false),
        RemasterRelation::MergedInto => ("Merged into", 1, false),
        RemasterRelation::ReplacedBy => ("Replaced by", 2, false),
    }
}
fn project(
    contexts: Vec<RecordRemasterContext>,
    selection: &RecordNavigationView,
    retrieval: &AtlasRetrievalService,
) -> AppServiceResult<RemasterRelationshipsView> {
    let mut truncated = contexts.len() > 100;
    let mut remaining_targets = 100;
    let mut groups = BTreeMap::<(bool, String, String), Vec<RecordRemasterContext>>::new();
    for mut context in contexts.into_iter().take(100) {
        if context.targets.len() > remaining_targets {
            context.targets.truncate(remaining_targets);
            truncated = true;
        }
        remaining_targets -= context.targets.len();
        if context.targets.is_empty() {
            continue;
        }
        groups
            .entry((
                relation(context.relation).2,
                context.scope.clone(),
                serde_json::to_string(&context.journal)
                    .map_err(|e| AppServiceError::invalid_request(e.to_string()))?,
            ))
            .or_default()
            .push(context);
    }
    let keys = groups
        .values()
        .flatten()
        .flat_map(|c| &c.targets)
        .filter_map(|t| t.record.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let summaries = retrieval
        .get_records(atlas_search::GetRecordsRequest { record_keys: &keys })?
        .records
        .into_iter()
        .map(|s| (s.key.clone(), s))
        .collect::<BTreeMap<_, _>>();
    let mut explanations = explanations::ExplanationCache::default();
    let mut relationships = vec![];
    for ((incoming, _, _), contexts) in groups {
        let Some((label, _, _)) = contexts
            .iter()
            .map(|c| relation(c.relation))
            .max_by_key(|(_, priority, _)| *priority)
        else {
            continue;
        };
        let mut targets = Vec::<RemasterTarget>::new();
        let mut links = vec![];
        for context in contexts {
            for target in context.targets {
                if target.record.as_ref().is_some_and(|key| {
                    key.to_string() == selection.record_key && target.owners == selection.owners
                }) {
                    continue;
                }
                let name = atlas_domain::normalize_record_name(&target.name);
                if let Some(existing) = targets.iter_mut().find(|prior| {
                    (prior.record == target.record
                        && prior.owners == target.owners
                        && prior.record.is_some())
                        || (atlas_domain::normalize_record_name(&prior.name) == name
                            && (prior.record.is_none() || target.record.is_none()))
                }) {
                    if existing.record.is_none() && target.record.is_some() {
                        *existing = target;
                    }
                } else {
                    targets.push(target);
                }
            }
            if let Some(journal) = context.journal
                && let Some(link) = explanations.resolve(&journal, retrieval)
                && !links.contains(&link)
            {
                links.push(link);
            }
        }
        let cut = targets.len() > 100;
        targets.truncate(100);
        let mut parts = vec![];
        let count = targets.len();
        for (i, target) in targets.into_iter().enumerate() {
            if i > 0 {
                parts.push(RemasterInlinePartView::Text {
                    text: if i + 1 == count {
                        if count == 2 {
                            " and ".into()
                        } else {
                            ", and ".into()
                        }
                    } else {
                        ", ".into()
                    },
                });
            }
            let available = target.record.as_ref().and_then(|k| summaries.get(k));
            let mut target_label = target.name;
            let valid = if let (Some(key), Some(summary)) = (&target.record, available) {
                if target.owners.is_empty() {
                    target_label =
                        crate::projection::localized_summary_view(summary, retrieval).title;
                    true
                } else {
                    retrieval
                        .get_record(atlas_search::GetRecordRequest {
                            record_key: key,
                            selected_content: &[],
                        })?
                        .and_then(|r| {
                            r.source
                                .node_at(&target.owners)
                                .map(|node| node.name().value().cloned())
                        })
                        .is_some_and(|name| {
                            if let Some(name) = name {
                                target_label = name;
                            }
                            true
                        })
                }
            } else {
                false
            };
            parts.push(match target.record.filter(|_| valid) {
                Some(key) => RemasterInlinePartView::Reference {
                    label: target_label,
                    navigation: Box::new(navigation(&key, &target.owners, retrieval)),
                },
                None => RemasterInlinePartView::Text { text: target_label },
            });
        }
        relationships.push(RemasterRelationshipView {
            id: format!("relationship-{}-{incoming}", relationships.len()),
            label: label.into(),
            parts,
            explanation_links: links,
            truncated: cut,
        });
    }
    Ok(RemasterRelationshipsView {
        relationships,
        truncated,
    })
}
fn source_context(
    selection: &RecordNavigationView,
    retrieval: &AtlasRetrievalService,
) -> Option<RelatedCategoryView<RemasterSourceContextView>> {
    if selection.owners.is_empty() {
        return None;
    }
    Some(category((|| {
        let key = parse_key(&selection.record_key)?;
        let provenance =
            retrieval.read_non_content_relationships(&atlas_search::SourceRelationshipRequest {
                record: key,
                owners: Some(selection.owners.clone()),
                field: None,
                direction: atlas_search::SourceRelationshipDirection::Outgoing,
                limit: 1024,
            })?;
        if provenance.truncated {
            return Ok(RelatedCategoryView::Unavailable {
                reason: "The source-entry references are incomplete".into(),
            });
        }
        let mut origin = None;
        for edge in provenance
            .occurrences
            .iter()
            .filter(|edge| edge.kind == "CompendiumProvenance")
        {
            let key = match &edge.resolution {
                ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) => {
                    Some(key.clone())
                }
                ContentReferenceResolution::Unresolved => edge
                    .authored_target
                    .as_deref()
                    .and_then(atlas_record::source_record::SourceReferenceIndex::explicit_root_key),
                _ => None,
            };
            let Some(key) = key else {
                return Ok(RelatedCategoryView::Unavailable {
                    reason: "The source entry could not be identified from its explicit reference"
                        .into(),
                });
            };
            if origin.as_ref().is_some_and(|prior| prior != &key) {
                return Ok(RelatedCategoryView::Unavailable {
                    reason: "The selected document identifies conflicting source entries".into(),
                });
            }
            origin = Some(key);
        }
        let Some(origin) = origin else {
            return Ok(RelatedCategoryView::Empty);
        };
        let source_selection = navigation(&origin, &[], retrieval);
        let summaries = retrieval.get_records(atlas_search::GetRecordsRequest {
            record_keys: std::slice::from_ref(&origin),
        })?;
        let Some(summary) = summaries.records.first() else {
            return Ok(RelatedCategoryView::Empty);
        };
        let label = crate::projection::localized_summary_view(summary, retrieval).title;
        let relationships = reading_relationships(&source_selection, retrieval);
        if matches!(relationships, RelatedCategoryView::Empty) {
            return Ok(RelatedCategoryView::Empty);
        }
        Ok(RelatedCategoryView::Content {
            content: RemasterSourceContextView {
                heading: format!("Source entry: {label}"),
                navigation: source_selection,
                relationships,
            },
        })
    })()))
}
