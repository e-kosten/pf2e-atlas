//! Casting rank comes from a selected occurrence, not merely the spell's identity.
use super::*;
use atlas_foundry_model::SourcePresence;
use atlas_foundry_model::generated::*;

fn override_rank(value: &SourcePresence<serde_json::Number>) -> Result<Option<u8>, ()> {
    match value {
        SourcePresence::Missing | SourcePresence::Null => Ok(None),
        SourcePresence::Value(value) => value
            .as_u64()
            .filter(|rank| (1..=10).contains(rank))
            .map(|rank| Some(rank as u8))
            .ok_or(()),
        SourcePresence::Invalid(_) => Err(()),
    }
}

pub(crate) fn base_rank(source: &SpellSource) -> Option<u8> {
    source
        .system
        .as_value()?
        .level
        .as_value()?
        .value
        .as_value()?
        .as_u64()
        .filter(|rank| (1..=10).contains(rank))
        .map(|rank| rank as u8)
}

pub(crate) fn ranks(record: &SourceBackedRecord, source: &SpellSource) -> Vec<u8> {
    let Some(base) = base_rank(source) else {
        return vec![];
    };
    let Some(system) = source.system.as_value() else {
        return vec![];
    };
    let parent = record.node_at(&[]);
    // Only an immediate actor spell belongs to that actor's casting collections.
    // A consumable's nested spell has its own authored casting context.
    let mut direct_actor_spell = false;
    record.visit_immediate_actor_items(|_, _, item| {
        if let ItemSourceView::SpellSource(candidate) = item {
            direct_actor_spell |= std::ptr::eq(candidate, source);
        }
    });
    let parent = parent.filter(|_| direct_actor_spell);
    let Some(parent @ SourceNodeView::Actor(_)) = parent else {
        return match system.location.as_value().map_or_else(
            || {
                if matches!(system.location, SourcePresence::Invalid(_)) {
                    Err(())
                } else {
                    Ok(None)
                }
            },
            |location| override_rank(&location.heightened_level),
        ) {
            Ok(rank) => vec![rank.unwrap_or(base).max(base)],
            Err(()) => vec![],
        };
    };
    if matches!(system.location, SourcePresence::Invalid(_)) {
        return vec![];
    }
    let location = system.location.as_value();
    let entry_id = location.and_then(|l| l.value.as_value());
    let mut entries = vec![];
    record.visit_immediate_actor_items(|_, _, item| {
        if let ItemSourceView::SpellcastingEntrySource(entry) = item
            && entry._id.as_value() == entry_id
            && entry_id.is_some()
        {
            entries.push(entry);
        }
    });
    let entry = if entries.len() == 1 {
        entries[0].system.as_value()
    } else {
        None
    };
    let automatic = system
        .traits
        .as_value()
        .and_then(|t| t.value.as_value())
        .is_some_and(|traits| {
            traits
                .iter()
                .any(|t| matches!(t.as_str(), "cantrip" | "focus"))
        });
    if automatic {
        let rank = location.map_or(Ok(None), |l| override_rank(&l.auto_heighten_level));
        let rank = rank.and_then(|rank| {
            if rank.is_some() {
                return Ok(rank);
            }
            match entry.map(|e| &e.auto_heighten_level) {
                Some(SourcePresence::Value(value)) => override_rank(&value.value),
                Some(SourcePresence::Invalid(_)) => Err(()),
                _ => Ok(None),
            }
        });
        let Ok(rank) = rank else {
            return vec![];
        };
        let level = SourceQueryView {
            source: parent,
            pack_id: "",
            pack_label: "",
        }
        .actor()
        .level();
        return rank
            .or_else(|| {
                level
                    .value()
                    .and_then(|n| n.as_f64())
                    .map(|v| (v / 2.0).ceil().clamp(1.0, 10.0) as u8)
            })
            .map(|r| vec![r.max(base)])
            .unwrap_or_default();
    }
    if let Some(entry) = entry
        && entry.prepared.as_value().and_then(|p| p.value.as_value())
            == Some(&SpellCollectionTypeSourceValue::Prepared)
        && let Some(slots) = entry.slots.as_value()
    {
        let mut ranks = vec![];
        for (rank, slot) in [
            &slots.slot0,
            &slots.slot1,
            &slots.slot2,
            &slots.slot3,
            &slots.slot4,
            &slots.slot5,
            &slots.slot6,
            &slots.slot7,
            &slots.slot8,
            &slots.slot9,
            &slots.slot10,
        ]
        .into_iter()
        .enumerate()
        {
            if slot
                .as_value()
                .and_then(|s| s.prepared.as_value())
                .is_some_and(|slots| {
                    slots.iter().any(|s| {
                        s.id.as_value().is_some() && s.id.as_value() == source._id.as_value()
                    })
                })
            {
                ranks.push((rank as u8).max(base));
            }
        }
        ranks.dedup();
        if !ranks.is_empty() {
            return ranks;
        }
    }
    match location.map_or(Ok(None), |l| override_rank(&l.heightened_level)) {
        Ok(rank) => vec![rank.unwrap_or(base).max(base)],
        Err(()) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_search::test_support::record;
    use serde_json::json;

    #[test]
    fn rejected_overrides_do_not_infer_casting_ranks() {
        for (traits, location, entry_override, expected) in [
            (
                json!(["cantrip"]),
                json!({"autoHeightenLevel":"broken"}),
                json!({"value":3}),
                vec![],
            ),
            (
                json!(["cantrip"]),
                json!({}),
                json!({"value":"broken"}),
                vec![],
            ),
            (
                json!(["cantrip"]),
                json!({"autoHeightenLevel":2.5}),
                json!({"value":3}),
                vec![],
            ),
            (
                json!([]),
                json!({"heightenedLevel":"broken"}),
                json!({}),
                vec![],
            ),
            (json!([]), json!({"heightenedLevel":11}), json!({}), vec![]),
            (json!(["cantrip"]), json!({}), json!({}), vec![5]),
            (
                json!(["cantrip"]),
                json!({"autoHeightenLevel":null}),
                json!({"value":3}),
                vec![3],
            ),
            (
                json!(["cantrip"]),
                json!({"autoHeightenLevel":4}),
                json!({"value":"broken"}),
                vec![4],
            ),
            (json!([]), json!({}), json!({}), vec![1]),
        ] {
            let mut location = location;
            location["value"] = json!("eeeeeeeeeeeeeeee");
            let input = record(
                "actors",
                "Actor",
                json!({
                    "_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"details":{"level":{"value":9}}},
                    "items":[
                        {"_id":"eeeeeeeeeeeeeeee","type":"spellcastingEntry","system":{"prepared":{"value":"innate"},"autoHeightenLevel":entry_override}},
                        {"_id":"ssssssssssssssss","type":"spell","system":{"level":{"value":1},"traits":{"value":traits},"location":location}}
                    ]
                }),
            );
            input.record.visit_immediate_actor_items(|_, _, item| {
                if let ItemSourceView::SpellSource(spell) = item {
                    assert_eq!(ranks(&input.record, spell), expected, "{location}");
                }
            });
        }
    }
}
