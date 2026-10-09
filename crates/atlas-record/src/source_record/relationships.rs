use super::nodes::{actor_fields, item_fields};
use super::{FieldAvailability, ItemSourceView, SourceFieldView, SourceNodeEntry, SourceNodeView};
use crate::source_content::{
    ContentReferenceResolution, ContentReferenceResolver, SourceContentLocator,
};
use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceRelationshipKind {
    CompendiumProvenance,
    GrantedBy,
    ItemGrant,
    SpellcastingEntry,
    PreparedSpell,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRelationshipOccurrence {
    pub locator: SourceContentLocator,
    pub kind: SourceRelationshipKind,
    pub availability: FieldAvailability,
    pub authored_target: Option<String>,
    pub resolution: ContentReferenceResolution,
}

pub(super) fn collect_relationships(
    key: &RecordKey,
    entry: &SourceNodeEntry<'_>,
    resolver: Option<&super::SourceReferenceIndex>,
    output: &mut Vec<SourceRelationshipOccurrence>,
) {
    let mut add = |field: &str, kind, state: SourceFieldView<'_, &String>, local: bool| {
        if matches!(
            state,
            SourceFieldView::Missing | SourceFieldView::Null | SourceFieldView::NotApplicable
        ) {
            return;
        }
        let locator = SourceContentLocator {
            record: key.clone(),
            owners: entry.owners.clone(),
            field: field.into(),
        };
        let authored_target = state.value().cloned();
        let resolution = authored_target
            .as_ref()
            .and_then(|target| {
                resolver.and_then(|r| {
                    if local {
                        let family = match kind {
                            SourceRelationshipKind::SpellcastingEntry => Some("spellcastingEntry"),
                            SourceRelationshipKind::PreparedSpell => Some("spell"),
                            _ => None,
                        };
                        r.resolve_local_item(&locator, target, family)
                    } else {
                        r.resolve_reference(&locator, target)
                    }
                })
            })
            .map(|r| ContentReferenceResolution::Resolved(r.target))
            .unwrap_or(ContentReferenceResolution::Unresolved);
        output.push(SourceRelationshipOccurrence {
            locator,
            kind,
            availability: state.availability(),
            authored_target,
            resolution,
        });
    };
    let provenance = match entry.source {
        SourceNodeView::Item(i) => item_fields!(
            i,
            s,
            SourceFieldView::from(&s._stats).and_then(|s| (&s.compendium_source).into())
        ),
        SourceNodeView::Actor(a) => actor_fields!(
            a,
            s,
            SourceFieldView::from(&s._stats).and_then(|s| (&s.compendium_source).into())
        ),
        SourceNodeView::Journal(s) => {
            SourceFieldView::from(&s._stats).and_then(|s| (&s.compendium_source).into())
        }
        SourceNodeView::Macro(s) => {
            SourceFieldView::from(&s._stats).and_then(|s| (&s.compendium_source).into())
        }
        SourceNodeView::Table(s) => {
            SourceFieldView::from(&s._stats).and_then(|s| (&s.compendium_source).into())
        }
        SourceNodeView::JournalPage(s) => {
            SourceFieldView::from(&s._stats).and_then(|s| (&s.compendium_source).into())
        }
        SourceNodeView::Result(_) => SourceFieldView::NotApplicable,
    };
    if !matches!(provenance, SourceFieldView::NotApplicable) {
        add(
            "/_stats/compendiumSource",
            SourceRelationshipKind::CompendiumProvenance,
            provenance,
            false,
        );
    }
    let SourceNodeView::Item(item) = entry.source else {
        return;
    };
    let granted_by = item_fields!(
        item,
        s,
        SourceFieldView::from(&s.flags)
            .and_then(|s| (&s.pf2e).into())
            .and_then(|s| (&s.granted_by).into())
            .and_then(|s| (&s.id).into())
    );
    add(
        "/flags/pf2e/grantedBy/id",
        SourceRelationshipKind::GrantedBy,
        granted_by,
        true,
    );
    let grants = item_fields!(
        item,
        s,
        SourceFieldView::from(&s.flags)
            .and_then(|s| (&s.pf2e).into())
            .and_then(|s| (&s.item_grants).into())
    );
    match grants {
        SourceFieldView::Value(grants) => {
            for (name, grant) in &grants.entries {
                // JSON Pointer escaping preserves the authored map key.
                let name = name.replace('~', "~0").replace('/', "~1");
                add(
                    &format!("/flags/pf2e/itemGrants/{name}/id"),
                    SourceRelationshipKind::ItemGrant,
                    (&grant.id).into(),
                    true,
                );
            }
        }
        state => add(
            "/flags/pf2e/itemGrants",
            SourceRelationshipKind::ItemGrant,
            state.and_then(|_| SourceFieldView::NotApplicable),
            true,
        ),
    }
    if let ItemSourceView::SpellSource(s) = item {
        add(
            "/system/location/value",
            SourceRelationshipKind::SpellcastingEntry,
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.location).into())
                .and_then(|s| (&s.value).into()),
            true,
        );
    }
    if let ItemSourceView::SpellcastingEntrySource(s) = item {
        let slots = SourceFieldView::from(&s.system).and_then(|s| (&s.slots).into());
        if let SourceFieldView::Value(slots) = slots {
            for (rank, slot) in [
                (0, &slots.slot0),
                (1, &slots.slot1),
                (2, &slots.slot2),
                (3, &slots.slot3),
                (4, &slots.slot4),
                (5, &slots.slot5),
                (6, &slots.slot6),
                (7, &slots.slot7),
                (8, &slots.slot8),
                (9, &slots.slot9),
                (10, &slots.slot10),
            ] {
                let prepared = SourceFieldView::from(slot).and_then(|s| (&s.prepared).into());
                match prepared {
                    SourceFieldView::Value(spells) => {
                        for (index, spell) in spells.iter().enumerate() {
                            add(
                                &format!("/system/slots/slot{rank}/prepared/{index}/id"),
                                SourceRelationshipKind::PreparedSpell,
                                (&spell.id).into(),
                                true,
                            );
                        }
                    }
                    state => add(
                        &format!("/system/slots/slot{rank}/prepared"),
                        SourceRelationshipKind::PreparedSpell,
                        state.and_then(|_| SourceFieldView::NotApplicable),
                        true,
                    ),
                }
            }
        } else {
            add(
                "/system/slots",
                SourceRelationshipKind::PreparedSpell,
                slots.and_then(|_| SourceFieldView::NotApplicable),
                true,
            );
        }
    }
}
