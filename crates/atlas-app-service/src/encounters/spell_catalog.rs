//! Bounded casting resources from checked source fields; no Foundry rule execution.
use crate::presentation::spell_context;
use atlas_app_model::EncounterSpellResourceScope;
use atlas_app_model::{
    EncounterSpellRankView, EncounterSpellResourceView, EncounterSpellView,
    EncounterSpellcastingView, RecordNavigationView,
};
use atlas_foundry_model::{ActorSourcePF2e, SourcePresence, generated::*};
use atlas_local_state::SpellResource;
use atlas_record::source_record::{ItemSourceView, SourceBackedRecord, SourceNodeView};
use std::collections::BTreeMap;

fn count(value: &SourcePresence<serde_json::Number>) -> Option<i64> {
    value
        .as_value()?
        .as_f64()
        .filter(|v| v.is_finite() && v.fract() == 0.0 && (0.0..=100_000.0).contains(v))
        .map(|v| v as i64)
}
fn key(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|p| format!("{}:{p}", p.len()))
        .collect::<Vec<_>>()
        .join("/")
}
fn slots(system: &SpellcastingEntrySystemSource) -> Vec<(u8, &SpellSlotData)> {
    system
        .slots
        .as_value()
        .map(|s| {
            [
                &s.slot0, &s.slot1, &s.slot2, &s.slot3, &s.slot4, &s.slot5, &s.slot6, &s.slot7,
                &s.slot8, &s.slot9, &s.slot10,
            ]
            .into_iter()
            .enumerate()
            .filter_map(|(r, s)| s.as_value().map(|s| (r as u8, s)))
            .collect()
        })
        .unwrap_or_default()
}

pub(super) fn catalog(
    record: &SourceBackedRecord,
    saved: &[SpellResource],
    expended: &[String],
    fingerprint: Option<&str>,
) -> EncounterSpellcastingView {
    let mut entries: BTreeMap<&str, Vec<&SpellcastingEntrySource>> = BTreeMap::new();
    record.visit_immediate_actor_items(|_, _, item| {
        if let ItemSourceView::SpellcastingEntrySource(entry) = item
            && let Some(id) = entry._id.as_value()
        {
            entries.entry(id).or_default().push(entry);
        }
    });
    let mut resources: BTreeMap<String, EncounterSpellResourceView> = BTreeMap::new();
    let mut add_resource = |resource_key: String,
                            entry_id: &str,
                            label: String,
                            capacity: i64,
                            scope: EncounterSpellResourceScope| {
        let stored = saved.iter().find(|s| s.key == resource_key);
        resources
            .entry(resource_key.clone())
            .or_insert_with(|| EncounterSpellResourceView {
                scope,
                key: resource_key,
                entry_id: entry_id.into(),
                label,
                capacity: stored.map_or(capacity, |s| s.capacity),
                remaining: stored.map_or(capacity, |s| s.remaining),
                can_undo: stored.is_some_and(|s| s.undo_remaining.is_some()),
            });
    };
    let focus = match record.node_at(&[]) {
        Some(SourceNodeView::Actor(ActorSourcePF2e::NPCSource(npc))) => npc
            .system
            .as_value()
            .and_then(|s| s.resources.as_value())
            .and_then(|r| r.focus.as_value())
            .and_then(|f| count(&f.max)),
        _ => None,
    };
    let mut spell_ids = BTreeMap::new();
    record.visit_immediate_actor_items(|_, _, item| {
        if let ItemSourceView::SpellSource(spell) = item
            && let Some(id) = spell._id.as_value()
        {
            *spell_ids.entry(id).or_insert(0) += 1;
        }
    });
    let mut spells = Vec::new();
    record.visit_immediate_actor_items(|_, owner, item| {
        let ItemSourceView::SpellSource(spell) = item else {
            return;
        };
        let Some(system) = spell.system.as_value() else {
            return;
        };
        let Some(base) = spell_context::base_rank(spell) else {
            return;
        };
        let Some(id) = spell._id.as_value() else {
            return;
        };
        // Ambiguous or snapshot-local identities cannot own durable resources.
        if spell_ids.get(id) != Some(&1)
            || !matches!(
                owner.identity,
                atlas_domain::OwnedContentIdentity::Stable(_)
            )
        {
            return;
        }
        let location = system.location.as_value();
        let entry_id = location
            .and_then(|l| l.value.as_value())
            .map(String::as_str)
            .unwrap_or("");
        let entry = entries
            .get(entry_id)
            .filter(|v| v.len() == 1)
            .and_then(|v| v[0].system.as_value());
        let mode = entry
            .and_then(|e| e.prepared.as_value())
            .and_then(|p| p.value.as_value());
        let flexible = entry
            .and_then(|e| e.prepared.as_value())
            .and_then(|p| p.flexible.as_value())
            .copied()
            .unwrap_or(false);
        let traits = system.traits.as_value().and_then(|t| t.value.as_value());
        let has_trait = |name: &str| traits.is_some_and(|ts| ts.iter().any(|t| t.as_str() == name));
        let automatic = has_trait("cantrip") || has_trait("focus");
        let known_ranks = spell_context::ranks(record, spell);
        let title = spell
            .name
            .as_value()
            .cloned()
            .unwrap_or_else(|| "Spell".into());
        let missing_uses = location.is_some_and(|l| match &l.uses {
            SourcePresence::Missing => true,
            SourcePresence::Value(uses) => matches!(uses.max, SourcePresence::Missing),
            _ => false,
        });
        let manual =
            mode == Some(&SpellCollectionTypeSourceValue::Innate) && !automatic && missing_uses;
        let mut ranks = Vec::new();
        for rank in base..=10 {
            let mut resource_key = None;
            let mut reason = Some("No tracked resource for this rank".to_owned());
            if automatic
                && known_ranks.contains(&rank)
                && has_trait("cantrip")
                && entry.is_some()
                && (mode != Some(&SpellCollectionTypeSourceValue::Prepared)
                    || entry.is_some_and(|e| {
                        slots(e).into_iter().any(|(r, s)| {
                            r == 0
                                && s.prepared.as_value().is_some_and(|ps| {
                                    ps.iter().any(|p| p.id.as_value() == Some(id))
                                })
                        })
                    }))
            {
                reason = None;
            } else if automatic
                && known_ranks.contains(&rank)
                && has_trait("focus")
                && entry.is_some()
            {
                if let Some(capacity) = focus {
                    let pool = "focus".to_string();
                    add_resource(
                        pool.clone(),
                        "focus",
                        "Focus points".into(),
                        capacity,
                        EncounterSpellResourceScope::Entry,
                    );
                    resource_key = Some(pool);
                    reason = None;
                }
            } else if !automatic && let Some(entry) = entry {
                let slot = slots(entry)
                    .into_iter()
                    .find(|(r, _)| *r == rank)
                    .map(|(_, s)| s);
                let rank_text = rank.to_string();
                match mode {
                    Some(SpellCollectionTypeSourceValue::Prepared) if !flexible => {
                        if let Some(preparations) = slot.and_then(|s| s.prepared.as_value()) {
                            let capacity = preparations
                                .iter()
                                .filter(|p| p.id.as_value() == Some(id))
                                .count() as i64;
                            if capacity > 0 {
                                let pool = key(&[entry_id, &rank_text, id]);
                                add_resource(
                                    pool.clone(),
                                    entry_id,
                                    format!("{title} · Rank {rank}"),
                                    capacity,
                                    EncounterSpellResourceScope::Spell,
                                );
                                resource_key = Some(pool);
                                reason = None;
                            }
                        }
                    }
                    Some(SpellCollectionTypeSourceValue::Spontaneous)
                    | Some(SpellCollectionTypeSourceValue::Prepared) => {
                        let signature =
                            location.and_then(|l| l.signature.as_value()) == Some(&true);
                        let eligible = (!flexible && known_ranks.contains(&rank)) || signature;
                        if eligible && let Some(capacity) = slot.and_then(|s| count(&s.max)) {
                            let pool = key(&[entry_id, &rank_text]);
                            add_resource(
                                pool.clone(),
                                entry_id,
                                format!("Rank {rank} slots"),
                                capacity,
                                EncounterSpellResourceScope::Rank { rank },
                            );
                            resource_key = Some(pool);
                            reason = None;
                        }
                    }
                    Some(SpellCollectionTypeSourceValue::Innate) if known_ranks.contains(&rank) => {
                        if let Some(capacity) = location
                            .and_then(|l| l.uses.as_value())
                            .and_then(|u| count(&u.max))
                        {
                            let pool = key(&[entry_id, id]);
                            add_resource(
                                pool.clone(),
                                entry_id,
                                title.clone(),
                                capacity,
                                EncounterSpellResourceScope::Spell,
                            );
                            resource_key = Some(pool);
                            reason = None;
                        }
                    }
                    _ => {}
                }
            }
            ranks.push(EncounterSpellRankView {
                rank,
                resource_key,
                unavailable_reason: reason,
            });
        }
        spells.push(EncounterSpellView {
            navigation: RecordNavigationView {
                record_key: record.key().to_string(),
                owners: vec![owner.clone()],
                spell_rank: None,
                field: None,
                passage: None,
                source_fingerprint: fingerprint.map(str::to_owned),
            },
            entry_id: entry_id.into(),
            title,
            expended: manual.then(|| expended.contains(id)),
            ranks,
        });
    });
    EncounterSpellcastingView {
        resources: resources.into_values().collect(),
        spells,
    }
}

pub(super) fn initial_resources(record: &SourceBackedRecord) -> Vec<SpellResource> {
    catalog(record, &[], &[], None)
        .resources
        .into_iter()
        .map(|r| SpellResource {
            key: r.key,
            capacity: r.capacity,
            remaining: r.capacity,
            undo_remaining: None,
        })
        .collect()
}
