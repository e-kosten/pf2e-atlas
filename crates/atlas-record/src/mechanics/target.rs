use crate::{CreatureComponentId, CreatureOccurrenceId, CreatureSaveKind, CreatureSkillKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicTarget {
    ArmorClass,
    MaxHp,
    Perception,
    Save {
        save: SaveKind,
    },
    /// Transitional sparse-record target. Canonical creature projections use
    /// `CreatureSkill`, whose component identity cannot be inferred from a metric key.
    Skill {
        slug: String,
    },
    CreatureSkill {
        skill_id: CreatureComponentId,
        kind: CreatureSkillKind,
    },
    AbilityModifier {
        ability: AbilityKind,
    },
    Movement {
        speed_id: CreatureComponentId,
    },
    ResourceMaximum {
        resource_id: CreatureComponentId,
    },
    ActivityActionCost {
        occurrence_id: CreatureOccurrenceId,
    },
    ActivityFrequency {
        occurrence_id: CreatureOccurrenceId,
    },
    ActivityUses {
        occurrence_id: CreatureOccurrenceId,
    },
    ActivityRoll {
        occurrence_id: CreatureOccurrenceId,
        roll_id: String,
    },
    ActivityDamage {
        occurrence_id: CreatureOccurrenceId,
        damage_id: String,
    },
    SpellcastingAttack {
        entry_occurrence_id: CreatureOccurrenceId,
    },
    SpellcastingDc {
        entry_occurrence_id: CreatureOccurrenceId,
    },
    SpellSlotMaximum {
        entry_occurrence_id: CreatureOccurrenceId,
        rank: i64,
    },
    ActorRitualDc,
}

impl MechanicTarget {
    pub fn id(&self) -> String {
        match self {
            Self::ArmorClass => "ac".to_string(),
            Self::MaxHp => "hp.max".to_string(),
            Self::Perception => "perception".to_string(),
            Self::Save { save } => format!("save.{}", save.as_str()),
            Self::Skill { slug } => skill_target_id(slug),
            Self::CreatureSkill { skill_id, kind } => {
                structured_target_id("creature-skill", &[skill_id.as_str(), kind.source_slug()])
            }
            Self::AbilityModifier { ability } => format!("ability.{}", ability.as_str()),
            Self::Movement { speed_id } => structured_target_id("movement", &[speed_id.as_str()]),
            Self::ResourceMaximum { resource_id } => {
                structured_target_id("resource-maximum", &[resource_id.as_str()])
            }
            Self::ActivityActionCost { occurrence_id } => {
                structured_target_id("activity-action-cost", &[occurrence_id.as_str()])
            }
            Self::ActivityFrequency { occurrence_id } => {
                structured_target_id("activity-frequency", &[occurrence_id.as_str()])
            }
            Self::ActivityUses { occurrence_id } => {
                structured_target_id("activity-uses", &[occurrence_id.as_str()])
            }
            Self::ActivityRoll {
                occurrence_id,
                roll_id,
            } => structured_target_id("activity-roll", &[occurrence_id.as_str(), roll_id]),
            Self::ActivityDamage {
                occurrence_id,
                damage_id,
            } => structured_target_id("activity-damage", &[occurrence_id.as_str(), damage_id]),
            Self::SpellcastingAttack {
                entry_occurrence_id,
            } => structured_target_id("spellcasting-attack", &[entry_occurrence_id.as_str()]),
            Self::SpellcastingDc {
                entry_occurrence_id,
            } => structured_target_id("spellcasting-dc", &[entry_occurrence_id.as_str()]),
            Self::SpellSlotMaximum {
                entry_occurrence_id,
                rank,
            } => structured_target_id(
                "spell-slot-maximum",
                &[entry_occurrence_id.as_str(), &rank.to_string()],
            ),
            Self::ActorRitualDc => structured_target_id("actor-ritual-dc", &[]),
        }
    }
}

fn skill_target_id(slug: &str) -> String {
    if !slug.is_empty()
        && slug.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        format!("skill.{slug}")
    } else {
        structured_target_id("legacy-skill", &[slug])
    }
}

/// Builds a printable, injective external identity for structured mechanics targets.
///
/// Variant tags are closed and unique. Each arbitrary UTF-8 component is encoded as
/// lowercase hexadecimal, so `/` can delimit components without escaping or aliasing.
fn structured_target_id(tag: &str, components: &[&str]) -> String {
    const PREFIX: &str = "mechanic-target/v1";
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let encoded_capacity = components
        .iter()
        .map(|component| 1 + component.len() * 2)
        .sum::<usize>();
    let mut id = String::with_capacity(PREFIX.len() + 1 + tag.len() + encoded_capacity);
    id.push_str(PREFIX);
    id.push('/');
    id.push_str(tag);
    for component in components {
        id.push('/');
        for byte in component.as_bytes() {
            id.push(HEX[(byte >> 4) as usize] as char);
            id.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
    id
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveKind {
    Fortitude,
    Reflex,
    Will,
}

impl SaveKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fortitude => "fort",
            Self::Reflex => "ref",
            Self::Will => "will",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Fortitude => "Fortitude",
            Self::Reflex => "Reflex",
            Self::Will => "Will",
        }
    }
}

impl From<CreatureSaveKind> for SaveKind {
    fn from(value: CreatureSaveKind) -> Self {
        match value {
            CreatureSaveKind::Fortitude => Self::Fortitude,
            CreatureSaveKind::Reflex => Self::Reflex,
            CreatureSaveKind::Will => Self::Will,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbilityKind {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl AbilityKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Strength => "str",
            Self::Dexterity => "dex",
            Self::Constitution => "con",
            Self::Intelligence => "int",
            Self::Wisdom => "wis",
            Self::Charisma => "cha",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Dexterity",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{AbilityKind, MechanicTarget, SaveKind};
    use crate::{CreatureComponentId, CreatureOccurrenceId, CreatureSkillKind};

    #[test]
    fn every_target_variant_has_one_injective_external_identity() {
        let adversarial_component = component("a.roll/b:%25.雪");
        let adversarial_occurrence = occurrence("occurrence:a.roll/b:%25.雪");
        let cases = vec![
            (MechanicTarget::ArmorClass, None),
            (MechanicTarget::MaxHp, None),
            (MechanicTarget::Perception, None),
            (
                MechanicTarget::Save {
                    save: SaveKind::Fortitude,
                },
                None,
            ),
            (
                MechanicTarget::Skill {
                    slug: "a.roll/b:%25.雪\0".to_string(),
                },
                Some(("legacy-skill", vec!["a.roll/b:%25.雪\0"])),
            ),
            (
                MechanicTarget::CreatureSkill {
                    skill_id: adversarial_component.clone(),
                    kind: CreatureSkillKind::Lore,
                },
                Some(("creature-skill", vec!["a.roll/b:%25.雪", "lore"])),
            ),
            (
                MechanicTarget::AbilityModifier {
                    ability: AbilityKind::Wisdom,
                },
                None,
            ),
            (
                MechanicTarget::Movement {
                    speed_id: adversarial_component.clone(),
                },
                Some(("movement", vec!["a.roll/b:%25.雪"])),
            ),
            (
                MechanicTarget::ResourceMaximum {
                    resource_id: adversarial_component,
                },
                Some(("resource-maximum", vec!["a.roll/b:%25.雪"])),
            ),
            (
                MechanicTarget::ActivityActionCost {
                    occurrence_id: adversarial_occurrence.clone(),
                },
                Some(("activity-action-cost", vec!["occurrence:a.roll/b:%25.雪"])),
            ),
            (
                MechanicTarget::ActivityFrequency {
                    occurrence_id: adversarial_occurrence.clone(),
                },
                Some(("activity-frequency", vec!["occurrence:a.roll/b:%25.雪"])),
            ),
            (
                MechanicTarget::ActivityUses {
                    occurrence_id: adversarial_occurrence.clone(),
                },
                Some(("activity-uses", vec!["occurrence:a.roll/b:%25.雪"])),
            ),
            (
                MechanicTarget::ActivityRoll {
                    occurrence_id: adversarial_occurrence.clone(),
                    roll_id: "b.action_cost/.:%雪\0".to_string(),
                },
                Some((
                    "activity-roll",
                    vec!["occurrence:a.roll/b:%25.雪", "b.action_cost/.:%雪\0"],
                )),
            ),
            (
                MechanicTarget::ActivityDamage {
                    occurrence_id: adversarial_occurrence.clone(),
                    damage_id: "b.roll/.:%雪\0".to_string(),
                },
                Some((
                    "activity-damage",
                    vec!["occurrence:a.roll/b:%25.雪", "b.roll/.:%雪\0"],
                )),
            ),
            (
                MechanicTarget::SpellcastingAttack {
                    entry_occurrence_id: adversarial_occurrence.clone(),
                },
                Some(("spellcasting-attack", vec!["occurrence:a.roll/b:%25.雪"])),
            ),
            (
                MechanicTarget::SpellcastingDc {
                    entry_occurrence_id: adversarial_occurrence.clone(),
                },
                Some(("spellcasting-dc", vec!["occurrence:a.roll/b:%25.雪"])),
            ),
            (
                MechanicTarget::SpellSlotMaximum {
                    entry_occurrence_id: adversarial_occurrence,
                    rank: i64::MIN,
                },
                Some((
                    "spell-slot-maximum",
                    vec!["occurrence:a.roll/b:%25.雪", "-9223372036854775808"],
                )),
            ),
            (
                MechanicTarget::ActorRitualDc,
                Some(("actor-ritual-dc", Vec::new())),
            ),
        ];

        let mut ids = BTreeMap::new();
        for (target, structured_parts) in cases {
            assert_variant_is_exhaustive(&target);
            let id = target.id();
            assert!(
                ids.insert(id.clone(), target.clone()).is_none(),
                "duplicate external target id {id}"
            );
            if let Some((expected_tag, expected_components)) = structured_parts {
                let (tag, components) = decode_structured_id(&id);
                assert_eq!(tag, expected_tag);
                assert_eq!(components, expected_components);
            }
        }
    }

    #[test]
    fn delimiter_collision_regression_and_identity_mutations_are_distinct() {
        let reviewed_collision = [
            MechanicTarget::ActivityActionCost {
                occurrence_id: occurrence("occurrence:a.roll.b"),
            },
            MechanicTarget::ActivityRoll {
                occurrence_id: occurrence("occurrence:a"),
                roll_id: "b.action_cost".to_string(),
            },
        ];
        assert_ne!(reviewed_collision[0].id(), reviewed_collision[1].id());
        assert_eq!(
            MechanicTarget::Skill {
                slug: "athletics".to_string(),
            }
            .id(),
            "skill.athletics"
        );

        let mutation_pairs = vec![
            (
                MechanicTarget::Save {
                    save: SaveKind::Fortitude,
                },
                MechanicTarget::Save {
                    save: SaveKind::Will,
                },
            ),
            (
                MechanicTarget::Skill {
                    slug: String::new(),
                },
                MechanicTarget::Skill {
                    slug: ".:/%雪\0".to_string(),
                },
            ),
            (
                MechanicTarget::CreatureSkill {
                    skill_id: component("skill:a/b.c"),
                    kind: CreatureSkillKind::Arcana,
                },
                MechanicTarget::CreatureSkill {
                    skill_id: component("skill:a/b.c"),
                    kind: CreatureSkillKind::Lore,
                },
            ),
            (
                MechanicTarget::AbilityModifier {
                    ability: AbilityKind::Strength,
                },
                MechanicTarget::AbilityModifier {
                    ability: AbilityKind::Dexterity,
                },
            ),
            structured_single_component_pair("movement"),
            structured_single_component_pair("resource"),
            structured_occurrence_pair("action-cost"),
            structured_occurrence_pair("frequency"),
            structured_occurrence_pair("uses"),
            (
                MechanicTarget::ActivityRoll {
                    occurrence_id: occurrence("occurrence:a"),
                    roll_id: "b/c.d".to_string(),
                },
                MechanicTarget::ActivityRoll {
                    occurrence_id: occurrence("occurrence:a/b"),
                    roll_id: "c.d".to_string(),
                },
            ),
            (
                MechanicTarget::ActivityDamage {
                    occurrence_id: occurrence("occurrence:a"),
                    damage_id: "b/c.d".to_string(),
                },
                MechanicTarget::ActivityDamage {
                    occurrence_id: occurrence("occurrence:a/b"),
                    damage_id: "c.d".to_string(),
                },
            ),
            structured_occurrence_pair("spellcasting-attack"),
            structured_occurrence_pair("spellcasting-dc"),
            (
                MechanicTarget::SpellSlotMaximum {
                    entry_occurrence_id: occurrence("occurrence:a/b"),
                    rank: -1,
                },
                MechanicTarget::SpellSlotMaximum {
                    entry_occurrence_id: occurrence("occurrence:a/b"),
                    rank: 1,
                },
            ),
        ];

        let mut ids = BTreeMap::new();
        for target in reviewed_collision.iter().chain(
            mutation_pairs
                .iter()
                .flat_map(|(left, right)| [left, right]),
        ) {
            if let Some(existing) = ids.insert(target.id(), target) {
                assert_eq!(existing, target, "distinct targets shared an external id");
            }
        }

        for (left, right) in mutation_pairs {
            assert_ne!(left, right);
            assert_ne!(left.id(), right.id(), "{left:?} aliased {right:?}");
        }
    }

    fn structured_single_component_pair(kind: &str) -> (MechanicTarget, MechanicTarget) {
        match kind {
            "movement" => (
                MechanicTarget::Movement {
                    speed_id: component("component:a/b.c"),
                },
                MechanicTarget::Movement {
                    speed_id: component("component:a.bc"),
                },
            ),
            "resource" => (
                MechanicTarget::ResourceMaximum {
                    resource_id: component("component:a/b.c"),
                },
                MechanicTarget::ResourceMaximum {
                    resource_id: component("component:a.bc"),
                },
            ),
            _ => panic!("unknown target kind"),
        }
    }

    fn structured_occurrence_pair(kind: &str) -> (MechanicTarget, MechanicTarget) {
        let left = occurrence("occurrence:a/b.c");
        let right = occurrence("occurrence:a.bc");
        match kind {
            "action-cost" => (
                MechanicTarget::ActivityActionCost {
                    occurrence_id: left,
                },
                MechanicTarget::ActivityActionCost {
                    occurrence_id: right,
                },
            ),
            "frequency" => (
                MechanicTarget::ActivityFrequency {
                    occurrence_id: left,
                },
                MechanicTarget::ActivityFrequency {
                    occurrence_id: right,
                },
            ),
            "uses" => (
                MechanicTarget::ActivityUses {
                    occurrence_id: left,
                },
                MechanicTarget::ActivityUses {
                    occurrence_id: right,
                },
            ),
            "spellcasting-attack" => (
                MechanicTarget::SpellcastingAttack {
                    entry_occurrence_id: left,
                },
                MechanicTarget::SpellcastingAttack {
                    entry_occurrence_id: right,
                },
            ),
            "spellcasting-dc" => (
                MechanicTarget::SpellcastingDc {
                    entry_occurrence_id: left,
                },
                MechanicTarget::SpellcastingDc {
                    entry_occurrence_id: right,
                },
            ),
            _ => panic!("unknown target kind"),
        }
    }

    fn decode_structured_id(id: &str) -> (&str, Vec<String>) {
        let body = id
            .strip_prefix("mechanic-target/v1/")
            .expect("structured target prefix");
        let mut segments = body.split('/');
        let tag = segments.next().expect("variant tag");
        let components = segments.map(decode_hex).collect();
        (tag, components)
    }

    fn decode_hex(value: &str) -> String {
        assert_eq!(value.len() % 2, 0, "hex byte pairs");
        let bytes = value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| (nibble(pair[0]) << 4) | nibble(pair[1]))
            .collect::<Vec<_>>();
        String::from_utf8(bytes).expect("encoded UTF-8")
    }

    const fn nibble(value: u8) -> u8 {
        match value {
            b'0'..=b'9' => value - b'0',
            b'a'..=b'f' => value - b'a' + 10,
            _ => panic!("invalid hex digit"),
        }
    }

    fn assert_variant_is_exhaustive(target: &MechanicTarget) {
        match target {
            MechanicTarget::ArmorClass
            | MechanicTarget::MaxHp
            | MechanicTarget::Perception
            | MechanicTarget::Save { .. }
            | MechanicTarget::Skill { .. }
            | MechanicTarget::CreatureSkill { .. }
            | MechanicTarget::AbilityModifier { .. }
            | MechanicTarget::Movement { .. }
            | MechanicTarget::ResourceMaximum { .. }
            | MechanicTarget::ActivityActionCost { .. }
            | MechanicTarget::ActivityFrequency { .. }
            | MechanicTarget::ActivityUses { .. }
            | MechanicTarget::ActivityRoll { .. }
            | MechanicTarget::ActivityDamage { .. }
            | MechanicTarget::SpellcastingAttack { .. }
            | MechanicTarget::SpellcastingDc { .. }
            | MechanicTarget::SpellSlotMaximum { .. }
            | MechanicTarget::ActorRitualDc => {}
        }
    }

    fn component(value: &str) -> CreatureComponentId {
        CreatureComponentId::new(value).expect("component id")
    }

    fn occurrence(value: &str) -> CreatureOccurrenceId {
        CreatureOccurrenceId::new(value).expect("occurrence id")
    }
}
