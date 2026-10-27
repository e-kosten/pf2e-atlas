//! Condition-aware damage protections. Table totals bypass this calculation entirely.
use super::super::conditions::ConditionRule;
use atlas_local_state::EncounterParticipant;
use atlas_record::source_record::{
    ActorIwrEntry, ActorIwrKind, ActorIwrType, SourceBackedRecord, SourceFieldView, SourceQueryView,
};

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::encounters) struct DamageProtection {
    pub immune: bool,
    pub weakness: i64,
    pub resistance: i64,
    pub hardness: i64,
}

impl DamageProtection {
    pub fn apply(self, amount: i64) -> i64 {
        if self.immune || amount <= 0 {
            return 0;
        }
        amount
            .saturating_add(self.weakness)
            .saturating_sub(self.resistance)
            .max(0)
    }
}

/// Unknown source predicates require a table-resolved amount; they are never ignored.
pub(in crate::encounters) fn persistent_protection(
    record: Option<&SourceBackedRecord>,
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    damage_type: &str,
) -> Result<DamageProtection, String> {
    let petrified = super::causes::condition_causes(participant, None, sources)
        .iter()
        .any(|cause| cause.rule == ConditionRule::Petrified);
    if petrified && object_damage_immunity(damage_type) {
        return Ok(DamageProtection {
            immune: true,
            ..Default::default()
        });
    }
    let record = record
        .ok_or_else(|| "Damage protections are unavailable; enter the table result.".to_owned())?;
    let actor = SourceQueryView::new(record.source(), record.key().pack().as_str(), "").actor();
    let mut result = DamageProtection::default();
    let immunities = super::effective_immunities::effective_immunities(record);
    let mut unresolved = (!immunities.authored_complete)
        .then(|| "Damage immunities are unavailable; enter the table result.".to_owned());
    for entry in immunities.entries {
        match matches_type(&entry.kind, damage_type) {
            Some(false) => {}
            Some(true) if !entry.authored.is_some_and(has_qualifiers) => {
                // One established unconditional immunity proves zero even when other facts are unknown.
                result.immune = true;
                return Ok(result);
            }
            Some(true) => {
                unresolved = Some(format!("Resolve {} exceptions at the table.", entry.kind))
            }
            None => unresolved = Some(format!("Resolve {} at the table.", entry.kind)),
        }
    }
    for kind in immunities.uncertain_types {
        if matches_type(kind, damage_type) != Some(false) {
            unresolved = Some(
                "Trait-derived damage immunities are unavailable; enter the table result.".into(),
            );
        }
    }
    if let Some(reason) = unresolved {
        return Err(reason);
    }
    if !petrified
        && (sources.unavailable_reason.is_some()
            || sources
                .unresolved_entries(participant)
                .iter()
                .any(|source| source.rule == ConditionRule::Petrified))
    {
        return Err("Petrified protections are unavailable; enter the table result.".into());
    }
    result.hardness = if petrified {
        8
    } else {
        match actor.hazard_hardness() {
            SourceFieldView::NotApplicable => 0,
            SourceFieldView::Value(value) => value
                .as_i64()
                .filter(|value| *value >= 0)
                .ok_or_else(|| "Hardness is unavailable; enter the table result.".to_owned())?,
            _ => return Err("Hardness is unavailable; enter the table result.".into()),
        }
    };
    for (kind, weakness) in [
        (ActorIwrKind::Weakness, true),
        (ActorIwrKind::Resistance, false),
    ] {
        let entries = actor.iwr(kind).value().ok_or_else(|| {
            "Damage protections are unavailable; enter the table result.".to_owned()
        })?;
        for entry in entries.iter() {
            // Serialize only the checked generated enum's identifier, never source JSON.
            let selector = match entry.iwr_type().value() {
                Some(ActorIwrType::Immunity(value)) => serde_json::to_value(value),
                Some(ActorIwrType::Weakness(value)) => serde_json::to_value(value),
                Some(ActorIwrType::Resistance(value)) => serde_json::to_value(value),
                None => {
                    return Err("A damage protection has no type; enter the table result.".into());
                }
            }
            .map_err(|_| "Damage protection type is unavailable.".to_owned())?;
            let selector = selector
                .as_str()
                .ok_or_else(|| "Damage protection type is unavailable.".to_owned())?;
            let applicable = matches_type(selector, damage_type)
                .ok_or_else(|| format!("Resolve {selector} at the table."))?;
            if !applicable {
                continue;
            }
            if has_qualifiers(entry) {
                return Err(format!("Resolve {selector} exceptions at the table."));
            }
            if weakness {
                result.weakness = result.weakness.max(magnitude(entry)?);
            } else {
                result.resistance = result.resistance.max(magnitude(entry)?);
            }
        }
    }
    Ok(result)
}

fn magnitude(entry: ActorIwrEntry<'_>) -> Result<i64, String> {
    entry
        .value()
        .value()
        .and_then(|value| value.as_i64())
        .filter(|value| *value >= 0)
        .ok_or_else(|| "Damage protection amount is unavailable; enter the table result.".into())
}

fn matches_type(selector: &str, damage_type: &str) -> Option<bool> {
    const TYPES: &[&str] = &[
        "acid",
        "bleed",
        "bludgeoning",
        "cold",
        "electricity",
        "fire",
        "force",
        "mental",
        "piercing",
        "poison",
        "slashing",
        "sonic",
        "spirit",
        "vitality",
        "void",
    ];
    if TYPES.contains(&selector) {
        return Some(selector == damage_type);
    }
    match selector {
        "all-damage" | "persistent-damage" => Some(true),
        "physical" => Some(matches!(
            damage_type,
            "bludgeoning" | "piercing" | "slashing"
        )),
        "energy" => Some(matches!(
            damage_type,
            "acid" | "cold" | "electricity" | "fire" | "force" | "sonic" | "vitality" | "void"
        )),
        "object-immunities" => Some(object_damage_immunity(damage_type)),
        // Pinned actor/data/iwr.ts describes fear/emotion as item:type:effect,
        // and sleep/death/healing/nonlethal as unhandled selectors. Persistent
        // condition/document.ts reconstructs a damage packet, not the original
        // disease/effect/attack. These effect immunities do not gate that packet.
        "sleep" | "fear-effects" | "emotion" | "death-effects" | "healing"
        | "nonlethal-attacks" | "disease" => Some(false),
        // Native condition identifiers describe effect immunity, not damage protection.
        kind if atlas_foundry_model::generated::condition_definitions::CONDITION_DEFINITIONS
            .iter()
            .any(|definition| definition.slug == kind) =>
        {
            Some(false)
        }
        _ => None,
    }
}

fn object_damage_immunity(damage_type: &str) -> bool {
    matches!(
        damage_type,
        "bleed" | "mental" | "poison" | "spirit" | "vitality" | "void"
    )
}

fn has_qualifiers(entry: ActorIwrEntry<'_>) -> bool {
    fn nonempty<T>(value: SourceFieldView<'_, &Vec<T>>) -> bool {
        match value {
            SourceFieldView::Missing => false,
            SourceFieldView::Value(values) => !values.is_empty(),
            _ => true,
        }
    }
    match entry {
        ActorIwrEntry::Immunity(value) => nonempty((&value.exceptions).into()),
        ActorIwrEntry::SchemaImmunity(value) => nonempty((&value.exceptions).into()),
        ActorIwrEntry::Weakness(value) => nonempty((&value.exceptions).into()),
        ActorIwrEntry::SchemaWeakness(value) => nonempty((&value.exceptions).into()),
        ActorIwrEntry::Resistance(value) => {
            nonempty((&value.exceptions).into()) || nonempty((&value.double_vs).into())
        }
        ActorIwrEntry::SchemaResistance(value) => {
            nonempty((&value.exceptions).into()) || nonempty((&value.double_vs).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protections_apply_in_order_and_zero_damage_does_not_trigger_weakness() {
        let protections = DamageProtection {
            immune: false,
            weakness: 5,
            resistance: 3,
            hardness: 8,
        };
        assert_eq!(protections.apply(10), 12);
        assert_eq!(protections.apply(0), 0);
        assert_eq!(
            DamageProtection {
                immune: true,
                ..protections
            }
            .apply(100),
            0
        );
    }

    #[test]
    fn object_immunities_are_specific_and_unknown_predicates_stay_unknown() {
        assert_eq!(matches_type("object-immunities", "bleed"), Some(true));
        assert_eq!(matches_type("object-immunities", "fire"), Some(false));
        assert_eq!(matches_type("physical", "bleed"), Some(false));
        assert_eq!(matches_type("silver", "slashing"), None);
        for effect in [
            "sleep",
            "fear-effects",
            "emotion",
            "death-effects",
            "healing",
            "nonlethal-attacks",
            "disease",
        ] {
            assert_eq!(matches_type(effect, "fire"), Some(false));
        }
        assert_eq!(matches_type("magical", "fire"), None);
    }
}
