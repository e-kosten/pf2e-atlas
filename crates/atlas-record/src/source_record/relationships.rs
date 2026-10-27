use super::nodes::{actor_fields, item_fields};
use super::{FieldAvailability, ItemSourceView, SourceFieldView, SourceNodeView};
use crate::source_content::{
    ContentReferenceResolution, ContentReferenceResolver, SourceContentLocator,
};
use atlas_domain::OwnedContentLocator;
use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceRelationshipKind {
    CompendiumProvenance,
    GrantedBy,
    ItemGrant,
    /// A rule or feature list names a grant target; no grant has been executed.
    DeclaredGrant,
    SpellcastingEntry,
    PreparedSpell,
    TableResult,
    SelfEffect,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRelationshipOccurrence {
    pub locator: SourceContentLocator,
    pub kind: SourceRelationshipKind,
    pub availability: FieldAvailability,
    pub authored_target: Option<String>,
    pub resolution: ContentReferenceResolution,
}

/// Resolve declared structured links without attaching a second model to the
/// source record. Occurrences precede graph deduplication and eligibility policy.
pub fn resolve_source_relationships(
    record: &super::SourceBackedRecord,
    resolver: Option<&super::SourceReferenceIndex>,
) -> Vec<SourceRelationshipOccurrence> {
    let mut output = Vec::new();
    super::traversal::visit_source_nodes(record.source(), |owners, source| {
        collect_relationships(record.key(), owners, source, resolver, &mut output);
    });
    output
}

pub(super) fn collect_relationships(
    key: &RecordKey,
    owners: &[OwnedContentLocator],
    source: SourceNodeView<'_>,
    resolver: Option<&super::SourceReferenceIndex>,
    output: &mut Vec<SourceRelationshipOccurrence>,
) {
    // Only explicit compendium UUIDs can target this local reference corpus.
    let resolver = match source {
        SourceNodeView::Result(result)
            if !SourceFieldView::from(&result.document_uuid)
                .value()
                .is_some_and(|uuid| uuid.starts_with("Compendium.")) =>
        {
            None
        }
        _ => resolver,
    };
    let mut add = |field: &str, kind, state: SourceFieldView<'_, &String>, local: bool| {
        if matches!(
            state,
            SourceFieldView::Missing | SourceFieldView::Null | SourceFieldView::NotApplicable
        ) {
            return;
        }
        let locator = SourceContentLocator {
            record: key.clone(),
            owners: owners.to_vec(),
            field: field.into(),
        };
        let mut occurrence = SourceRelationshipOccurrence {
            locator,
            kind,
            availability: state.availability(),
            authored_target: state.value().cloned(),
            resolution: ContentReferenceResolution::Unresolved,
        };
        if occurrence.availability == FieldAvailability::Value {
            occurrence.resolution = occurrence
                .authored_target
                .as_ref()
                .and_then(|target| {
                    resolver.and_then(|r| {
                        if local {
                            let family = match kind {
                                SourceRelationshipKind::SpellcastingEntry => {
                                    Some("spellcastingEntry")
                                }
                                SourceRelationshipKind::PreparedSpell => Some("spell"),
                                _ => None,
                            };
                            r.resolve_local_item(&occurrence.locator, target, family)
                        } else {
                            r.resolve_reference(
                                &occurrence.locator,
                                &crate::source_content::ContentReferenceKind::Uuid,
                                target,
                            )
                        }
                    })
                })
                .map(|r| ContentReferenceResolution::Resolved(r.target))
                .unwrap_or(ContentReferenceResolution::Unresolved);
        }
        output.push(occurrence);
    };
    if let SourceNodeView::Result(result) = source {
        use atlas_foundry_model::generated::TableResultType;
        let target = SourceFieldView::from(&result.r#type).and_then(|kind| match kind {
            TableResultType::Text => SourceFieldView::NotApplicable,
            TableResultType::Document => {
                SourceFieldView::from(&result.document_uuid).map(Clone::clone)
            }
        });
        let state = match &target {
            SourceFieldView::Value(v) => SourceFieldView::Value(v),
            SourceFieldView::Missing => SourceFieldView::Missing,
            SourceFieldView::Null => SourceFieldView::Null,
            SourceFieldView::Invalid(e) => SourceFieldView::Invalid(e),
            SourceFieldView::ProjectionInvalid {
                source_path,
                reason,
            } => SourceFieldView::ProjectionInvalid {
                source_path,
                reason,
            },
            SourceFieldView::NotApplicable => SourceFieldView::NotApplicable,
        };
        add(
            "/documentUuid",
            SourceRelationshipKind::TableResult,
            state,
            false,
        );
        return;
    }
    let provenance = match source {
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
    let SourceNodeView::Item(item) = source else {
        return;
    };
    add(
        "/flags/core/sourceId",
        SourceRelationshipKind::CompendiumProvenance,
        item.core_source_id(),
        false,
    );
    add(
        "/system/selfEffect/uuid",
        SourceRelationshipKind::SelfEffect,
        item.self_effect().and_then(|e| (&e.uuid).into()),
        false,
    );
    collect_declared_grants(item, key, |field, target| {
        add(field, SourceRelationshipKind::DeclaredGrant, target, false);
    });
    let granted_by = item_fields!(
        item,
        s,
        SourceFieldView::from(&s.flags)
            .and_then(|s| match s.entries.iter().find(|(key, _)| key == "pf2e") {
                Some((_, flags)) => SourceFieldView::Value(flags),
                None => SourceFieldView::Missing,
            })
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
            .and_then(|s| match s.entries.iter().find(|(key, _)| key == "pf2e") {
                Some((_, flags)) => SourceFieldView::Value(flags),
                None => SourceFieldView::Missing,
            })
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

fn collect_declared_grants(
    item: ItemSourceView<'_>,
    key: &RecordKey,
    mut add: impl FnMut(&str, SourceFieldView<'_, &String>),
) {
    use atlas_foundry_model::{RuleSource, SourceContext, SourcePresence};

    if let Some(rules) = item.rules().value() {
        for (index, rule) in rules.iter().enumerate() {
            if rule.key.as_value().map(String::as_str) != Some("GrantItem") {
                continue;
            }
            let field = format!("/system/rules/{index}/uuid");
            let Ok(RuleSource::GrantItem(grant)) = rule.interpret(SourceContext::new(
                key.to_string(),
                "checked source",
                format!("$.system.rules[{index}]"),
            )) else {
                continue;
            };
            if grant.ignored.as_value() == Some(&true) {
                continue;
            }
            if let SourcePresence::Value(uuid) = &grant.uuid
                && literal_compendium_target(uuid)
            {
                // Predicates/alterations do not change this reference's identity.
                // It is not evidence of eligibility or an actually applied grant.
                add(&field, SourceFieldView::Value(uuid));
            }
        }
    }

    match item {
        ItemSourceView::AncestrySource(source) => {
            if let Some(items) = SourceFieldView::from(&source.system)
                .and_then(|s| (&s.items).into())
                .value()
            {
                for (name, entry) in &items.entries {
                    add_feature(name, (&entry.uuid).into(), &mut add);
                }
            }
        }
        ItemSourceView::ClassSource(source) => {
            if let Some(items) = SourceFieldView::from(&source.system)
                .and_then(|s| (&s.items).into())
                .value()
            {
                for (name, entry) in &items.entries {
                    add_feature(name, (&entry.uuid).into(), &mut add);
                }
            }
        }
        ItemSourceView::BackgroundSource(source) => {
            if let Some(items) = SourceFieldView::from(&source.system)
                .and_then(|s| (&s.items).into())
                .value()
            {
                for (name, entry) in &items.entries {
                    add_feature(name, (&entry.uuid).into(), &mut add);
                }
            }
        }
        _ => {}
    }
}

fn literal_compendium_target(target: &str) -> bool {
    target.starts_with("Compendium.") && !target.contains(['{', '}'])
}

fn add_feature(
    name: &str,
    target: SourceFieldView<'_, &String>,
    add: &mut impl FnMut(&str, SourceFieldView<'_, &String>),
) {
    if target.value().is_some_and(|t| literal_compendium_target(t)) {
        let name = name.replace('~', "~0").replace('/', "~1");
        add(&format!("/system/items/{name}/uuid"), target);
    }
}
