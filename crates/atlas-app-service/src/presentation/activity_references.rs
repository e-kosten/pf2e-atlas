//! Attach only identity-verified destinations to authored activity references.
use super::*;

pub(super) fn resolve(
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
