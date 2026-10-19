use super::actor_query::ActorQueryView;
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
            SourceNodeView::Actor(_) | SourceNodeView::Item(_) => {
                self.source.source_type().and_then(|family| {
                    source_family_record_kind(family)
                        .map_or(SourceFieldView::NotApplicable, SourceFieldView::Value)
                })
            }
            SourceNodeView::Journal(_) | SourceNodeView::JournalPage(_) => {
                SourceFieldView::Value(Lore)
            }
            SourceNodeView::Table(_) => SourceFieldView::Value(RollTable),
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
                AmmoSource,
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
                AmmoSource,
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

/// Product classification shared by standalone and embedded Actor/Item records.
/// Unclassified families stay not applicable; collection membership is separate.
pub fn source_family_record_kind(family: &str) -> Option<RecordKind> {
    use RecordKind::*;
    Some(match family {
        "npc" => Creature,
        "character" => Character,
        "familiar" => Companion,
        "army" => Army,
        "hazard" => Hazard,
        "vehicle" => Vehicle,
        "action" | "condition" | "effect" => Rule,
        "affliction" => Affliction,
        "feat" => Feat,
        "spell" => Spell,
        "deity" => Lore,
        "campaignFeature" => CampaignFeature,
        "ancestry" | "background" | "class" | "heritage" => CharacterOption,
        "ammo" | "armor" | "backpack" | "consumable" | "equipment" | "kit" | "shield"
        | "treasure" | "weapon" => Equipment,
        _ => return None,
    })
}
