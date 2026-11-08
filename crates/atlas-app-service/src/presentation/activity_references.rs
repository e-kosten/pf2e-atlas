//! Attach only identity-verified destinations to authored activity references.
use super::*;

pub(super) fn resolve_self_effects(
    body: &mut RecordBodyView,
    key: &atlas_domain::RecordKey,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    let activities: Vec<&mut ActorActivityView> = match body {
        RecordBodyView::Creature(actor) => actor.activities.value.iter_mut().flatten().collect(),
        RecordBodyView::Hazard(hazard) => {
            hazard.actor.activities.value.iter_mut().flatten().collect()
        }
        RecordBodyView::Activity(activity) => vec![activity],
        _ => vec![],
    };
    if !activities.iter().any(|a| a.self_effect.value.is_some()) {
        return Ok(());
    }
    // A single bounded read covers all immediate entries; omitted occurrences
    // remain honest unresolved references rather than guessed destinations.
    let relationships =
        retrieval.read_non_content_relationships(&atlas_search::SourceRelationshipRequest {
            record: key.clone(),
            owners: None,
            field: Some("/system/selfEffect/uuid".into()),
            direction: atlas_search::SourceRelationshipDirection::Outgoing,
            limit: 1024,
        })?;
    let relationships = crate::projection::relationship_views(
        relationships.occurrences,
        retrieval.source_fingerprint(),
    );
    for activity in activities {
        if let Some(effect) = &mut activity.self_effect.value {
            effect.navigation = relationships
                .iter()
                .find(|relationship| {
                    relationship.kind == "SelfEffect"
                        && relationship.source.owners == activity.navigation.owners
                        && relationship.authored_target == effect.authored_target.value
                })
                .and_then(|relationship| relationship.target.clone());
        }
    }
    Ok(())
}

/// Attach reading destinations using Foundry's separate owned and glossary lookups.
pub(super) fn resolve_strike_effects(
    body: &mut RecordBodyView,
    record: &SourceBackedRecord,
    owners: &[OwnedContentLocator],
    retrieval: &atlas_search::AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    use crate::definitions::record_navigation;
    use atlas_record::source_record::foundry_slug;

    let activities = match body {
        RecordBodyView::Creature(actor) => actor.activities.value.as_deref_mut(),
        RecordBodyView::Hazard(hazard) => hazard.actor.activities.value.as_deref_mut(),
        RecordBodyView::Activity(activity) if owners.len() == 1 => {
            Some(std::slice::from_mut(activity.as_mut()))
        }
        _ => None,
    };
    let Some(activities) = activities else {
        return Ok(());
    };
    if !activities.iter().any(|activity| {
        activity
            .strike_effects
            .value
            .as_ref()
            .is_some_and(|effects| !effects.is_empty())
    }) {
        return Ok(());
    }

    let mut targets = std::collections::BTreeMap::new();
    let actor_items = record.visit_immediate_actor_items(|_, owner, item| {
        if let Some(slug) = item
            .owned_attack_effect_slug()
            .filter(|slug| !slug.is_empty())
        {
            targets
                .entry(slug)
                .and_modify(|target| *target = None)
                .or_insert_with(|| Some((owner.clone(), item)));
        }
    });
    if actor_items.value().is_none() {
        return Ok(());
    }
    let fingerprint = retrieval.source_fingerprint();
    for activity in activities {
        for effect in activity.strike_effects.value.iter_mut().flatten() {
            let Some(token) = effect.authored_target.value.as_deref() else {
                continue;
            };
            match targets.get(foundry_slug(token).as_str()) {
                Some(Some((owner, item))) => {
                    effect.name = text(SourceFieldView::Value(&node_title(SourceNodeView::Item(
                        *item,
                    ))));
                    effect.navigation = Some(record_navigation(
                        record.key().clone(),
                        vec![owner.clone()],
                        fingerprint,
                    ));
                }
                // Keep duplicate owned keys unresolved, even if a glossary item shares the key.
                Some(None) => {}
                None => {
                    if let Some((key, name)) = retrieval.bestiary_attack_effect(token)? {
                        effect.name = fact(SourceFieldView::Value(name.to_owned()));
                        effect.navigation =
                            Some(record_navigation(key.clone(), vec![], fingerprint));
                    }
                }
            }
        }
    }
    Ok(())
}
