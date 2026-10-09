use super::nodes::item_fields;
use super::{ItemSourceView, SourceFieldView, SourceNodeView};
use atlas_domain::RecordKind;
use atlas_foundry_model::{ActorSourcePF2e, FoundryDocumentSource, PublicationData, generated::*};
use serde_json::Number;

// Expand one typed access expression across explicitly applicable families.
// Excluded families are explicit so a new upstream variant needs a decision.
macro_rules! item_projection {
    ($item:expr, $source:ident, [$($family:ident),+], [$($excluded:ident),*], $projection:expr) => {
        match $item {
            $(ItemSourceView::$family($source) => $projection,)+
            $(ItemSourceView::$excluded(_) => SourceFieldView::NotApplicable,)*
        }
    };
}

/// Storage-neutral meanings, deliberately no catalog registry or expression DSL.
#[derive(Debug, Clone, Copy)]
pub struct SourceQueryView<'a> {
    pub source: SourceNodeView<'a>,
    pub pack_id: &'a str,
    pub pack_label: &'a str,
}
impl<'a> SourceQueryView<'a> {
    pub fn new(source: &'a FoundryDocumentSource, pack_id: &'a str, pack_label: &'a str) -> Self {
        Self {
            source: source.into(),
            pack_id,
            pack_label,
        }
    }
    pub fn publication(self) -> SourceFieldView<'a, &'a PublicationData> {
        match self.source {
            SourceNodeView::Item(i) => i.publication(),
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.details).into())
                    .and_then(|s| (&s.publication).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.details).into())
                    .and_then(|s| (&s.publication).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::VehicleSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.details).into())
                    .and_then(|s| (&s.publication).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn publication_title(self) -> SourceFieldView<'a, &'a str> {
        self.publication()
            .and_then(|p| (&p.title).into())
            .map(String::as_str)
    }
    pub fn publication_remaster(self) -> SourceFieldView<'a, bool> {
        self.publication()
            .and_then(|p| (&p.remaster).into())
            .map(|v| *v)
    }
    pub fn traits(self) -> SourceFieldView<'a, &'a [String]> {
        match self.source {
            SourceNodeView::Item(i) => i.traits(),
            SourceNodeView::Actor(ActorSourcePF2e::ArmySource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.value).into())
                    .map(Vec::as_slice)
            }
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.value).into())
                    .map(Vec::as_slice)
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.value).into())
                    .map(Vec::as_slice)
            }
            SourceNodeView::Actor(ActorSourcePF2e::VehicleSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.value).into())
                    .map(Vec::as_slice)
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn rarity(self) -> SourceFieldView<'a, &'a ItemTraitsItemTraitRarity> {
        match self.source {
            SourceNodeView::Item(i) => i.rarity(),
            SourceNodeView::Actor(ActorSourcePF2e::ArmySource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.rarity).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.rarity).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.rarity).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::VehicleSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.traits).into())
                    .and_then(|s| (&s.rarity).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    /// Existing Atlas family classification; families without a current product
    /// classification remain explicitly not applicable rather than guessed.
    pub fn record_kind(self) -> SourceFieldView<'a, RecordKind> {
        use RecordKind::*;
        match self.source {
            SourceNodeView::Actor(a) => self.source.source_type().and_then(|_| match a {
                ActorSourcePF2e::NPCSource(_) => SourceFieldView::Value(Creature),
                ActorSourcePF2e::CharacterSource(_) => SourceFieldView::Value(Character),
                ActorSourcePF2e::FamiliarSource(_) => SourceFieldView::Value(Companion),
                ActorSourcePF2e::ArmySource(_) => SourceFieldView::Value(Army),
                ActorSourcePF2e::HazardSource(_) => SourceFieldView::Value(Hazard),
                ActorSourcePF2e::VehicleSource(_) => SourceFieldView::Value(Vehicle),
                _ => SourceFieldView::NotApplicable,
            }),
            SourceNodeView::Item(i) => self.source.source_type().and_then(|_| match i {
                ItemSourceView::AbilitySource(_)
                | ItemSourceView::ConditionSource(_)
                | ItemSourceView::EffectSource(_) => SourceFieldView::Value(Rule),
                ItemSourceView::AfflictionSource(_) => SourceFieldView::Value(Affliction),
                ItemSourceView::FeatSource(_) => SourceFieldView::Value(Feat),
                ItemSourceView::SpellSource(_) => SourceFieldView::Value(Spell),
                ItemSourceView::DeitySource(_) => SourceFieldView::Value(Lore),
                ItemSourceView::CampaignFeatureSource(_) => SourceFieldView::Value(CampaignFeature),
                ItemSourceView::AncestrySource(_)
                | ItemSourceView::BackgroundSource(_)
                | ItemSourceView::ClassSource(_)
                | ItemSourceView::HeritageSource(_) => SourceFieldView::Value(CharacterOption),
                ItemSourceView::ArmorSource(_)
                | ItemSourceView::ContainerSource(_)
                | ItemSourceView::ConsumableSource(_)
                | ItemSourceView::EquipmentSource(_)
                | ItemSourceView::KitSource(_)
                | ItemSourceView::ShieldSource(_)
                | ItemSourceView::TreasureSource(_)
                | ItemSourceView::WeaponSource(_) => SourceFieldView::Value(Equipment),
                _ => SourceFieldView::NotApplicable,
            }),
            SourceNodeView::Journal(_) | SourceNodeView::JournalPage(_) => {
                SourceFieldView::Value(Lore)
            }
            SourceNodeView::Table(_) => SourceFieldView::Value(Tooling),
            SourceNodeView::Macro(_) => self.source.source_type().and_then(|t| {
                if t == "script" {
                    SourceFieldView::Value(Tooling)
                } else {
                    SourceFieldView::NotApplicable
                }
            }),
            SourceNodeView::Result(_) => SourceFieldView::NotApplicable,
        }
    }
    pub fn actor(self) -> ActorQueryView<'a> {
        ActorQueryView {
            source: self.source,
        }
    }
}

impl<'a> ItemSourceView<'a> {
    pub fn publication(self) -> SourceFieldView<'a, &'a PublicationData> {
        item_fields!(
            self,
            s,
            SourceFieldView::from(&s.system).and_then(|s| (&s.publication).into())
        )
    }
    pub fn traits(self) -> SourceFieldView<'a, &'a [String]> {
        item_projection!(
            self,
            s,
            [
                AbilitySource,
                AfflictionSource,
                AncestrySource,
                ArmorSource,
                BackgroundSource,
                BookSource,
                CampaignFeatureSource,
                ConditionSource,
                ConsumableSource,
                ContainerSource,
                EffectSource,
                EquipmentSource,
                FeatSource,
                HeritageSource,
                KitSource,
                MeleeSource,
                ShieldSource,
                SpellSource,
                TreasureSource,
                WeaponSource
            ],
            [
                ClassSource,
                DeitySource,
                LoreSource,
                SpellcastingEntrySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.traits).into())
                .and_then(|s| (&s.value).into())
                .map(Vec::as_slice)
        )
    }
    pub fn rarity(self) -> SourceFieldView<'a, &'a ItemTraitsItemTraitRarity> {
        item_projection!(
            self,
            s,
            [
                AncestrySource,
                ArmorSource,
                BackgroundSource,
                BookSource,
                ClassSource,
                ConsumableSource,
                ContainerSource,
                EquipmentSource,
                FeatSource,
                HeritageSource,
                ShieldSource,
                SpellSource,
                TreasureSource,
                WeaponSource
            ],
            [
                AbilitySource,
                AfflictionSource,
                CampaignFeatureSource,
                ConditionSource,
                DeitySource,
                EffectSource,
                KitSource,
                LoreSource,
                MeleeSource,
                SpellcastingEntrySource
            ],
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.traits).into())
                .and_then(|s| (&s.rarity).into())
        )
    }
    /// Authored base rank, never heightening, slot rank or a prepared spell form.
    pub fn spell_rank(self) -> SourceFieldView<'a, &'a Number> {
        match self {
            Self::SpellSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.level).into())
                .and_then(|s| (&s.value).into()),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn spell_traditions(
        self,
    ) -> SourceFieldView<'a, &'a [PatchSpellOverlayOverrideSystemTraitsTraditionsEntry]> {
        match self {
            Self::SpellSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.traits).into())
                .and_then(|s| (&s.traditions).into())
                .map(Vec::as_slice),
            _ => SourceFieldView::NotApplicable,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ActorQueryView<'a> {
    pub source: SourceNodeView<'a>,
}
impl<'a> ActorQueryView<'a> {
    pub fn level(self) -> SourceFieldView<'a, &'a Number> {
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.details).into())
                    .and_then(|s| (&s.level).into())
                    .and_then(|s| (&s.value).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.details).into())
                    .and_then(|s| (&s.level).into())
                    .and_then(|s| (&s.value).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn armor_class(self) -> SourceFieldView<'a, &'a Number> {
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.ac).into())
                    .and_then(|s| (&s.value).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.ac).into())
                    .and_then(|s| (&s.value).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn hp_maximum(self) -> SourceFieldView<'a, &'a Number> {
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.hp).into())
                    .and_then(|s| (&s.max).into())
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.hp).into())
                    .and_then(|s| (&s.max).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn hazard_hardness(self) -> SourceFieldView<'a, &'a Number> {
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.hardness).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn hazard_complexity(self) -> SourceFieldView<'a, bool> {
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.details).into())
                    .and_then(|s| (&s.is_complex).into())
                    .map(|v| *v)
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn items(self) -> SourceFieldView<'a, &'a [atlas_foundry_model::ItemSourcePF2e]> {
        self.source.actor_items()
    }
}
