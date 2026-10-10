//! Borrowed authored Actor facts. No Foundry runtime preparation occurs here.
use super::{SourceFieldView, SourceNodeView};
use atlas_foundry_model::{ActorSourcePF2e, ItemSourcePF2e, generated::*};
use serde_json::Number;

macro_rules! actor_projection {
    ($view:expr, $source:ident, [$($family:ident),+], [$($excluded:ident),+], $projection:expr) => {
        match $view.source {
            $(SourceNodeView::Actor(ActorSourcePF2e::$family($source)) => $projection,)+
            $(SourceNodeView::Actor(ActorSourcePF2e::$excluded(_)) => SourceFieldView::NotApplicable,)+
            _ => SourceFieldView::NotApplicable,
        }
    };
}

/// Named authored save, never an Army maneuver/morale substitute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorSave {
    Fortitude,
    Reflex,
    Will,
}

#[derive(Debug, Clone, Copy)]
pub struct ActorQueryView<'a> {
    pub source: SourceNodeView<'a>,
}

impl<'a> ActorQueryView<'a> {
    pub fn level(self) -> SourceFieldView<'a, &'a Number> {
        actor_projection!(
            self,
            s,
            [
                NPCSource,
                HazardSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource
            ],
            [FamiliarSource, PartySource],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.level).into())
                .and_then(|s| (&s.value).into())
        )
    }
    pub fn size(self) -> SourceFieldView<'a, &'a PhysicalSystemSourceSize> {
        actor_projection!(
            self,
            s,
            [NPCSource, HazardSource, VehicleSource],
            [
                CharacterSource,
                ArmySource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.traits).into())
                .and_then(|s| (&s.size).into())
                .and_then(|s| (&s.value).into())
        )
    }
    pub fn armor_class(self) -> SourceFieldView<'a, &'a Number> {
        actor_projection!(
            self,
            s,
            [NPCSource, HazardSource],
            [
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.attributes).into())
                .and_then(|s| (&s.ac).into())
                .and_then(|s| (&s.value).into())
        )
    }
    pub fn hp_maximum(self) -> SourceFieldView<'a, &'a Number> {
        actor_projection!(
            self,
            s,
            [NPCSource, HazardSource],
            [
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.attributes).into())
                .and_then(|s| (&s.hp).into())
                .and_then(|s| (&s.max).into())
        )
    }
    pub fn save(self, save: ActorSave) -> SourceFieldView<'a, &'a Number> {
        actor_projection!(
            self,
            s,
            [NPCSource, HazardSource],
            [
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.saves).into())
                .and_then(|s| match save {
                    ActorSave::Fortitude => (&s.fortitude).into(),
                    ActorSave::Reflex => (&s.reflex).into(),
                    ActorSave::Will => (&s.will).into(),
                })
                .and_then(|s| (&s.value).into())
        )
    }
    pub fn perception(self) -> SourceFieldView<'a, &'a Number> {
        actor_projection!(
            self,
            s,
            [NPCSource],
            [
                HazardSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.perception).into())
                .and_then(|s| (&s.r#mod).into())
        )
    }
    pub fn languages(self) -> SourceFieldView<'a, &'a [Language]> {
        actor_projection!(
            self,
            s,
            [NPCSource, CharacterSource],
            [
                HazardSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.languages).into())
                .and_then(|s| (&s.value).into())
                .map(Vec::as_slice)
        )
    }
    /// Primary authored speed in feet; a null value does not mask other speeds.
    pub fn land_speed(self) -> SourceFieldView<'a, &'a Number> {
        actor_projection!(
            self,
            s,
            [NPCSource, CharacterSource],
            [
                HazardSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.attributes).into())
                .and_then(|s| (&s.speed).into())
                .and_then(|s| (&s.value).into())
        )
    }
    pub fn speeds(self) -> SourceFieldView<'a, ActorSpeeds<'a>> {
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.speed).into())
                    .and_then(|s| (&s.other_speeds).into())
                    .map(|s| ActorSpeeds::Npc(s))
            }
            SourceNodeView::Actor(ActorSourcePF2e::CharacterSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.speed).into())
                    .and_then(|s| (&s.other_speeds).into())
                    .map(|s| ActorSpeeds::Character(s))
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn senses(self) -> SourceFieldView<'a, &'a [SenseConstructorParams]> {
        actor_projection!(
            self,
            s,
            [NPCSource],
            [
                HazardSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.perception).into())
                .and_then(|s| (&s.senses).into())
                .map(Vec::as_slice)
        )
    }
    /// The omission default is scoped inside both known typed ancestors.
    /// Explicit null/invalid ancestors or arrays remain unavailable. The DTO
    /// retains Missing, as required by the upstream-backed query policy.
    pub fn iwr(self, kind: ActorIwrKind) -> SourceFieldView<'a, ActorIwrEntries<'a>> {
        macro_rules! collection {
            ($s:expr, $immunity:ident, $weakness:ident, $resistance:ident) => {
                SourceFieldView::from(&$s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| match kind {
                        ActorIwrKind::Immunity => omitted_iwr((&a.immunities).into())
                            .map(|v| ActorIwrEntries::$immunity(v)),
                        ActorIwrKind::Weakness => omitted_iwr((&a.weaknesses).into())
                            .map(|v| ActorIwrEntries::$weakness(v)),
                        ActorIwrKind::Resistance => omitted_iwr((&a.resistances).into())
                            .map(|v| ActorIwrEntries::$resistance(v)),
                    })
            };
        }
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                collection!(s, Immunities, Weaknesses, Resistances)
            }
            SourceNodeView::Actor(ActorSourcePF2e::CharacterSource(s)) => {
                collection!(s, Immunities, Weaknesses, Resistances)
            }
            SourceNodeView::Actor(ActorSourcePF2e::ArmySource(s)) => {
                collection!(s, Immunities, Weaknesses, Resistances)
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                collection!(s, SchemaImmunities, SchemaWeaknesses, SchemaResistances)
            }
            SourceNodeView::Actor(ActorSourcePF2e::VehicleSource(s)) => {
                collection!(s, SchemaImmunities, SchemaWeaknesses, SchemaResistances)
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn hazard_hardness(self) -> SourceFieldView<'a, &'a Number> {
        actor_projection!(
            self,
            s,
            [HazardSource],
            [
                NPCSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.attributes).into())
                .and_then(|s| (&s.hardness).into())
        )
    }
    pub fn hazard_complexity(self) -> SourceFieldView<'a, bool> {
        actor_projection!(
            self,
            s,
            [HazardSource],
            [
                NPCSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.is_complex).into())
                .map(|v| *v)
        )
    }
    pub fn items(self) -> SourceFieldView<'a, &'a [ItemSourcePF2e]> {
        self.source.actor_items()
    }
    /// Authored adjustment is display/participant initialization context only;
    /// it never changes this view's baseline numeric projections.
    pub fn npc_adjustment(self) -> SourceFieldView<'a, &'a NPCAttributesSourceAdjustment> {
        actor_projection!(
            self,
            s,
            [NPCSource],
            [
                HazardSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.attributes).into())
                .and_then(|s| (&s.adjustment).into())
        )
    }
    pub fn npc_skills(self) -> SourceFieldView<'a, &'a NPCSystemSourceSkillsPartial> {
        actor_projection!(
            self,
            s,
            [NPCSource],
            [
                HazardSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system).and_then(|s| (&s.skills).into())
        )
    }
    pub fn npc_abilities(self) -> SourceFieldView<'a, &'a Abilities> {
        actor_projection!(
            self,
            s,
            [NPCSource],
            [
                HazardSource,
                CharacterSource,
                ArmySource,
                VehicleSource,
                LootSource,
                FamiliarSource,
                PartySource
            ],
            SourceFieldView::from(&s.system).and_then(|s| (&s.abilities).into())
        )
    }
}

fn omitted_iwr<'a, T>(field: SourceFieldView<'a, &'a Vec<T>>) -> SourceFieldView<'a, &'a [T]> {
    match field {
        SourceFieldView::Missing => SourceFieldView::Value(&[]),
        other => other.map(Vec::as_slice),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActorSpeeds<'a> {
    Npc(&'a [LabeledSpeed]),
    Character(&'a [CharacterAttributesSourceSpeedOtherSpeedsEntry]),
}
impl<'a> ActorSpeeds<'a> {
    pub fn len(self) -> usize {
        match self {
            Self::Npc(s) => s.len(),
            Self::Character(s) => s.len(),
        }
    }
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    pub fn get(self, index: usize) -> Option<ActorSpeedEntry<'a>> {
        match self {
            Self::Npc(s) => s.get(index).map(ActorSpeedEntry::Npc),
            Self::Character(s) => s.get(index).map(ActorSpeedEntry::Character),
        }
    }
    pub fn iter(self) -> Box<dyn ExactSizeIterator<Item = ActorSpeedEntry<'a>> + 'a> {
        match self {
            Self::Npc(v) => Box::new(v.iter().map(ActorSpeedEntry::Npc)),
            Self::Character(v) => Box::new(v.iter().map(ActorSpeedEntry::Character)),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ActorSpeedEntry<'a> {
    Npc(&'a LabeledSpeed),
    Character(&'a CharacterAttributesSourceSpeedOtherSpeedsEntry),
}
impl<'a> ActorSpeedEntry<'a> {
    pub fn speed_type(
        self,
    ) -> SourceFieldView<'a, &'a CharacterAttributesSourceSpeedOtherSpeedsEntryType> {
        match self {
            Self::Npc(s) => (&s.r#type).into(),
            Self::Character(s) => (&s.r#type).into(),
        }
    }
    pub fn value(self) -> SourceFieldView<'a, &'a Number> {
        match self {
            Self::Npc(s) => (&s.value).into(),
            Self::Character(s) => (&s.value).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorIwrKind {
    Immunity,
    Weakness,
    Resistance,
}
#[derive(Debug, Clone, Copy)]
pub enum ActorIwrEntries<'a> {
    Immunities(&'a [ActorAttributesSourceImmunitiesEntryImmunitySource]),
    Weaknesses(&'a [WeaknessSource]),
    Resistances(&'a [ResistanceSource]),
    SchemaImmunities(&'a [SourceFromSchemaHazardAttributesSchemaImmunitiesEntrySourceFromSchema]),
    SchemaWeaknesses(&'a [SourceFromSchemaHazardAttributesSchemaWeaknessesEntrySourceFromSchema]),
    SchemaResistances(&'a [SourceFromSchemaHazardAttributesSchemaResistancesEntrySourceFromSchema]),
}
impl<'a> ActorIwrEntries<'a> {
    pub fn len(self) -> usize {
        match self {
            Self::Immunities(v) => v.len(),
            Self::Weaknesses(v) => v.len(),
            Self::Resistances(v) => v.len(),
            Self::SchemaImmunities(v) => v.len(),
            Self::SchemaWeaknesses(v) => v.len(),
            Self::SchemaResistances(v) => v.len(),
        }
    }
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    pub fn get(self, index: usize) -> Option<ActorIwrEntry<'a>> {
        match self {
            Self::Immunities(v) => v.get(index).map(ActorIwrEntry::Immunity),
            Self::Weaknesses(v) => v.get(index).map(ActorIwrEntry::Weakness),
            Self::Resistances(v) => v.get(index).map(ActorIwrEntry::Resistance),
            Self::SchemaImmunities(v) => v.get(index).map(ActorIwrEntry::SchemaImmunity),
            Self::SchemaWeaknesses(v) => v.get(index).map(ActorIwrEntry::SchemaWeakness),
            Self::SchemaResistances(v) => v.get(index).map(ActorIwrEntry::SchemaResistance),
        }
    }
    pub fn iter(self) -> Box<dyn ExactSizeIterator<Item = ActorIwrEntry<'a>> + 'a> {
        match self {
            Self::Immunities(v) => Box::new(v.iter().map(ActorIwrEntry::Immunity)),
            Self::Weaknesses(v) => Box::new(v.iter().map(ActorIwrEntry::Weakness)),
            Self::Resistances(v) => Box::new(v.iter().map(ActorIwrEntry::Resistance)),
            Self::SchemaImmunities(v) => Box::new(v.iter().map(ActorIwrEntry::SchemaImmunity)),
            Self::SchemaWeaknesses(v) => Box::new(v.iter().map(ActorIwrEntry::SchemaWeakness)),
            Self::SchemaResistances(v) => Box::new(v.iter().map(ActorIwrEntry::SchemaResistance)),
        }
    }
}
/// A borrowed entry preserves custom predicates/exceptions for concrete display
/// consumers while filters select only authored type and magnitude.
#[derive(Debug, Clone, Copy)]
pub enum ActorIwrEntry<'a> {
    Immunity(&'a ActorAttributesSourceImmunitiesEntryImmunitySource),
    Weakness(&'a WeaknessSource),
    Resistance(&'a ResistanceSource),
    SchemaImmunity(&'a SourceFromSchemaHazardAttributesSchemaImmunitiesEntrySourceFromSchema),
    SchemaWeakness(&'a SourceFromSchemaHazardAttributesSchemaWeaknessesEntrySourceFromSchema),
    SchemaResistance(&'a SourceFromSchemaHazardAttributesSchemaResistancesEntrySourceFromSchema),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorIwrType<'a> {
    Immunity(&'a ActorAttributesSourceImmunitiesEntryImmunitySourceType),
    Weakness(&'a WeaknessSourceType),
    Resistance(&'a ResistanceSourceType),
}
impl<'a> ActorIwrEntry<'a> {
    pub fn iwr_type(self) -> SourceFieldView<'a, ActorIwrType<'a>> {
        match self {
            Self::Immunity(v) => SourceFieldView::from(&v.r#type).map(ActorIwrType::Immunity),
            Self::SchemaImmunity(v) => SourceFieldView::from(&v.r#type).map(ActorIwrType::Immunity),
            Self::Weakness(v) => SourceFieldView::from(&v.r#type).map(ActorIwrType::Weakness),
            Self::SchemaWeakness(v) => SourceFieldView::from(&v.r#type).map(ActorIwrType::Weakness),
            Self::Resistance(v) => SourceFieldView::from(&v.r#type).map(ActorIwrType::Resistance),
            Self::SchemaResistance(v) => {
                SourceFieldView::from(&v.r#type).map(ActorIwrType::Resistance)
            }
        }
    }
    pub fn value(self) -> SourceFieldView<'a, &'a Number> {
        match self {
            Self::Immunity(_) | Self::SchemaImmunity(_) => SourceFieldView::NotApplicable,
            Self::Weakness(v) => (&v.value).into(),
            Self::SchemaWeakness(v) => (&v.value).into(),
            Self::Resistance(v) => (&v.value).into(),
            Self::SchemaResistance(v) => (&v.value).into(),
        }
    }
}

#[cfg(test)]
#[path = "actor_query_tests.rs"]
mod tests;
