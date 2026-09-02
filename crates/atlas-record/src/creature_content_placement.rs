use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ContentOwner, CreatureEntityFamily, CreatureEntityId, CreatureEntityRelationship,
    CreatureEntityTarget, CreatureOccurrenceId, CreatureOccurrenceParent, CreatureRecord,
    OwnedRichContentDocument,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreatureContentAssociationFailure {
    DuplicateOccurrenceIdentity,
    AmbiguousEntityTarget,
    DuplicateContentIdentity,
    AmbiguousOwnerAssociation,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreatureContentPlacement {
    general: Vec<usize>,
    record_owned: Vec<usize>,
    unclaimed: Vec<usize>,
    entity_owned: BTreeMap<CreatureEntityId, Vec<usize>>,
    occurrence_owned: BTreeMap<CreatureOccurrenceId, Vec<usize>>,
    by_occurrence: BTreeMap<CreatureOccurrenceId, Vec<usize>>,
    standalone: BTreeMap<CreatureOccurrenceId, Vec<usize>>,
    claimed: BTreeSet<usize>,
    failures: BTreeMap<CreatureOccurrenceId, CreatureContentAssociationFailure>,
    relationships: Vec<CreatureEntityRelationship>,
}

impl CreatureContentPlacement {
    pub fn record_owned_documents<'a>(
        &'a self,
        creature: &'a CreatureRecord,
    ) -> impl Iterator<Item = &'a OwnedRichContentDocument> + 'a {
        self.record_owned
            .iter()
            .map(|index| &creature.content.documents[*index])
    }

    pub fn unclaimed_documents<'a>(
        &'a self,
        creature: &'a CreatureRecord,
    ) -> impl Iterator<Item = &'a OwnedRichContentDocument> + 'a {
        self.unclaimed
            .iter()
            .map(|index| &creature.content.documents[*index])
    }

    pub fn is_claimed(&self, document_index: usize) -> bool {
        self.claimed.contains(&document_index)
    }

    pub fn documents_for_occurrence<'a>(
        &'a self,
        creature: &'a CreatureRecord,
        occurrence_id: &CreatureOccurrenceId,
    ) -> impl Iterator<Item = &'a OwnedRichContentDocument> + 'a {
        self.by_occurrence
            .get(occurrence_id)
            .into_iter()
            .flatten()
            .map(|index| &creature.content.documents[*index])
    }

    pub fn general_documents<'a>(
        &'a self,
        creature: &'a CreatureRecord,
    ) -> impl Iterator<Item = &'a OwnedRichContentDocument> + 'a {
        self.general
            .iter()
            .map(|index| &creature.content.documents[*index])
    }

    pub fn entity_documents<'a>(
        &'a self,
        creature: &'a CreatureRecord,
        entity_id: &CreatureEntityId,
    ) -> impl Iterator<Item = &'a OwnedRichContentDocument> + 'a {
        self.entity_owned
            .get(entity_id)
            .into_iter()
            .flatten()
            .map(|index| &creature.content.documents[*index])
    }

    pub fn occurrence_documents<'a>(
        &'a self,
        creature: &'a CreatureRecord,
        occurrence_id: &CreatureOccurrenceId,
    ) -> impl Iterator<Item = &'a OwnedRichContentDocument> + 'a {
        self.occurrence_owned
            .get(occurrence_id)
            .into_iter()
            .flatten()
            .map(|index| &creature.content.documents[*index])
    }

    pub fn standalone_documents<'a>(
        &'a self,
        creature: &'a CreatureRecord,
        occurrence_id: &CreatureOccurrenceId,
    ) -> impl Iterator<Item = &'a OwnedRichContentDocument> + 'a {
        self.standalone
            .get(occurrence_id)
            .into_iter()
            .flatten()
            .map(|index| &creature.content.documents[*index])
    }

    pub fn failure(
        &self,
        occurrence_id: &CreatureOccurrenceId,
    ) -> Option<CreatureContentAssociationFailure> {
        self.failures.get(occurrence_id).copied()
    }

    pub fn relationships(&self) -> &[CreatureEntityRelationship] {
        &self.relationships
    }
}

pub fn place_creature_content(creature: &CreatureRecord) -> CreatureContentPlacement {
    place_creature_content_for_families(
        creature,
        &[
            CreatureEntityFamily::Action,
            CreatureEntityFamily::Strike,
            CreatureEntityFamily::SpellcastingEntry,
            CreatureEntityFamily::Spell,
            CreatureEntityFamily::Equipment,
            CreatureEntityFamily::Lore,
        ],
    )
}

pub fn place_creature_content_for_families(
    creature: &CreatureRecord,
    families: &[CreatureEntityFamily],
) -> CreatureContentPlacement {
    let mut placement = CreatureContentPlacement::default();
    let Some(embedded) = creature.embedded_entities.value.as_value() else {
        for (index, document) in creature.content.documents.iter().enumerate() {
            if matches!(document.owner, ContentOwner::Record(_)) {
                placement.record_owned.push(index);
                placement.general.push(index);
            } else {
                placement.unclaimed.push(index);
            }
        }
        return placement;
    };
    placement.relationships = embedded.relationships.clone();
    let candidates = embedded
        .occurrences
        .iter()
        .filter(|occurrence| families.contains(&occurrence.family))
        .collect::<Vec<_>>();
    let occurrence_counts = embedded
        .occurrences
        .iter()
        .fold(BTreeMap::new(), |mut map, value| {
            *map.entry(value.id.clone()).or_insert(0usize) += 1;
            map
        });
    let entity_counts = embedded
        .entities
        .iter()
        .fold(BTreeMap::new(), |mut map, value| {
            *map.entry(value.id.clone()).or_insert(0usize) += 1;
            map
        });
    let content_counts =
        creature
            .content
            .documents
            .iter()
            .fold(BTreeMap::new(), |mut map, value| {
                *map.entry(value.id.clone()).or_insert(0usize) += 1;
                map
            });

    for occurrence in &candidates {
        if occurrence_counts.get(&occurrence.id) != Some(&1) {
            placement.failures.insert(
                occurrence.id.clone(),
                CreatureContentAssociationFailure::DuplicateOccurrenceIdentity,
            );
        } else if let CreatureEntityTarget::ActorOwned(entity_id) = &occurrence.target
            && entity_counts.get(entity_id) != Some(&1)
        {
            placement.failures.insert(
                occurrence.id.clone(),
                CreatureContentAssociationFailure::AmbiguousEntityTarget,
            );
        }
    }

    for (index, document) in creature.content.documents.iter().enumerate() {
        match &document.owner {
            ContentOwner::Record(_) => {
                placement.record_owned.push(index);
                placement.general.push(index);
            }
            ContentOwner::CreatureEntity(owner) => placement
                .entity_owned
                .entry(owner.clone())
                .or_default()
                .push(index),
            ContentOwner::CreatureOccurrence(owner) => placement
                .occurrence_owned
                .entry(owner.clone())
                .or_default()
                .push(index),
        }
        let matches = candidates
            .iter()
            .filter(|occurrence| match &document.owner {
                ContentOwner::Record(_) => false,
                ContentOwner::CreatureOccurrence(owner) => owner == &occurrence.id,
                ContentOwner::CreatureEntity(owner) => {
                    matches!(&occurrence.target, CreatureEntityTarget::ActorOwned(target) if target == owner)
                }
            })
            .copied()
            .collect::<Vec<_>>();
        if matches.is_empty() {
            if !matches!(document.owner, ContentOwner::Record(_)) {
                placement.unclaimed.push(index);
                placement.general.push(index);
            }
            continue;
        }
        placement.claimed.insert(index);
        let failure = if content_counts.get(&document.id) != Some(&1) {
            Some(CreatureContentAssociationFailure::DuplicateContentIdentity)
        } else if matches.len() != 1 {
            Some(CreatureContentAssociationFailure::AmbiguousOwnerAssociation)
        } else {
            None
        };
        if let Some(failure) = failure {
            for occurrence in matches {
                placement.failures.insert(occurrence.id.clone(), failure);
            }
            continue;
        }
        let occurrence = matches[0];
        placement
            .by_occurrence
            .entry(occurrence.id.clone())
            .or_default()
            .push(index);
        if matches!(occurrence.parent, CreatureOccurrenceParent::Creature) {
            placement
                .standalone
                .entry(occurrence.id.clone())
                .or_default()
                .push(index);
        }
    }
    for indices in placement
        .by_occurrence
        .values_mut()
        .chain(placement.standalone.values_mut())
    {
        indices.sort_by_key(|index| {
            let document = &creature.content.documents[*index];
            (document.authored_order, document.id.content_key.as_str())
        });
    }
    for failed in placement.failures.keys() {
        placement.by_occurrence.remove(failed);
        placement.standalone.remove(failed);
    }
    placement.record_owned.sort_by_key(|index| {
        let document = &creature.content.documents[*index];
        (document.authored_order, document.id.content_key.as_str())
    });
    placement.unclaimed.sort_by_key(|index| {
        let document = &creature.content.documents[*index];
        (document.authored_order, document.id.content_key.as_str())
    });
    placement.general.sort_by_key(|index| {
        let document = &creature.content.documents[*index];
        (document.authored_order, document.id.content_key.as_str())
    });
    placement
}
