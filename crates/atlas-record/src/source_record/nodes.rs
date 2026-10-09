use super::identity::valid_source_id;
use super::{OwnedCollectionFact, SourceFieldView};
use crate::source_content::{OwnedContentIdentity, OwnedContentLocator, SourceContentLocator};
use atlas_domain::RecordKey;
use atlas_foundry_model::{ActorSourcePF2e, FoundryDocumentSource, ItemSourcePF2e, generated::*};

// Exhaustive dispatch shares declared components without cloning child bodies.
macro_rules! item_fields {
    ($item:expr, $s:ident, $body:expr) => {
        match $item {
            $crate::source_record::ItemSourceView::AbilitySource($s) => $body,
            $crate::source_record::ItemSourceView::AfflictionSource($s) => $body,
            $crate::source_record::ItemSourceView::AncestrySource($s) => $body,
            $crate::source_record::ItemSourceView::ArmorSource($s) => $body,
            $crate::source_record::ItemSourceView::BackgroundSource($s) => $body,
            $crate::source_record::ItemSourceView::BookSource($s) => $body,
            $crate::source_record::ItemSourceView::CampaignFeatureSource($s) => $body,
            $crate::source_record::ItemSourceView::ClassSource($s) => $body,
            $crate::source_record::ItemSourceView::ConditionSource($s) => $body,
            $crate::source_record::ItemSourceView::ConsumableSource($s) => $body,
            $crate::source_record::ItemSourceView::ContainerSource($s) => $body,
            $crate::source_record::ItemSourceView::DeitySource($s) => $body,
            $crate::source_record::ItemSourceView::EffectSource($s) => $body,
            $crate::source_record::ItemSourceView::EquipmentSource($s) => $body,
            $crate::source_record::ItemSourceView::FeatSource($s) => $body,
            $crate::source_record::ItemSourceView::HeritageSource($s) => $body,
            $crate::source_record::ItemSourceView::KitSource($s) => $body,
            $crate::source_record::ItemSourceView::LoreSource($s) => $body,
            $crate::source_record::ItemSourceView::MeleeSource($s) => $body,
            $crate::source_record::ItemSourceView::ShieldSource($s) => $body,
            $crate::source_record::ItemSourceView::SpellSource($s) => $body,
            $crate::source_record::ItemSourceView::SpellcastingEntrySource($s) => $body,
            $crate::source_record::ItemSourceView::TreasureSource($s) => $body,
            $crate::source_record::ItemSourceView::WeaponSource($s) => $body,
        }
    };
}
pub(super) use item_fields;
macro_rules! actor_fields {
    ($actor:expr, $s:ident, $body:expr) => {
        match $actor {
            atlas_foundry_model::ActorSourcePF2e::ArmySource($s) => $body,
            atlas_foundry_model::ActorSourcePF2e::CharacterSource($s) => $body,
            atlas_foundry_model::ActorSourcePF2e::FamiliarSource($s) => $body,
            atlas_foundry_model::ActorSourcePF2e::HazardSource($s) => $body,
            atlas_foundry_model::ActorSourcePF2e::LootSource($s) => $body,
            atlas_foundry_model::ActorSourcePF2e::NPCSource($s) => $body,
            atlas_foundry_model::ActorSourcePF2e::PartySource($s) => $body,
            atlas_foundry_model::ActorSourcePF2e::VehicleSource($s) => $body,
        }
    };
}
pub(super) use actor_fields;

#[derive(Debug, Clone, Copy)]
pub enum ItemSourceView<'a> {
    AbilitySource(&'a AbilitySource),
    AfflictionSource(&'a AfflictionSource),
    AncestrySource(&'a AncestrySource),
    ArmorSource(&'a ArmorSource),
    BackgroundSource(&'a BackgroundSource),
    BookSource(&'a BookSource),
    CampaignFeatureSource(&'a CampaignFeatureSource),
    ClassSource(&'a ClassSource),
    ConditionSource(&'a ConditionSource),
    ConsumableSource(&'a ConsumableSource),
    ContainerSource(&'a ContainerSource),
    DeitySource(&'a DeitySource),
    EffectSource(&'a EffectSource),
    EquipmentSource(&'a EquipmentSource),
    FeatSource(&'a FeatSource),
    HeritageSource(&'a HeritageSource),
    KitSource(&'a KitSource),
    LoreSource(&'a LoreSource),
    MeleeSource(&'a MeleeSource),
    ShieldSource(&'a ShieldSource),
    SpellSource(&'a SpellSource),
    SpellcastingEntrySource(&'a SpellcastingEntrySource),
    TreasureSource(&'a TreasureSource),
    WeaponSource(&'a WeaponSource),
}
impl<'a> From<&'a ItemSourcePF2e> for ItemSourceView<'a> {
    fn from(item: &'a ItemSourcePF2e) -> Self {
        match item {
            ItemSourcePF2e::AbilitySource(s) => Self::AbilitySource(s),
            ItemSourcePF2e::AfflictionSource(s) => Self::AfflictionSource(s),
            ItemSourcePF2e::AncestrySource(s) => Self::AncestrySource(s),
            ItemSourcePF2e::ArmorSource(s) => Self::ArmorSource(s),
            ItemSourcePF2e::BackgroundSource(s) => Self::BackgroundSource(s),
            ItemSourcePF2e::BookSource(s) => Self::BookSource(s),
            ItemSourcePF2e::CampaignFeatureSource(s) => Self::CampaignFeatureSource(s),
            ItemSourcePF2e::ClassSource(s) => Self::ClassSource(s),
            ItemSourcePF2e::ConditionSource(s) => Self::ConditionSource(s),
            ItemSourcePF2e::ConsumableSource(s) => Self::ConsumableSource(s),
            ItemSourcePF2e::ContainerSource(s) => Self::ContainerSource(s),
            ItemSourcePF2e::DeitySource(s) => Self::DeitySource(s),
            ItemSourcePF2e::EffectSource(s) => Self::EffectSource(s),
            ItemSourcePF2e::EquipmentSource(s) => Self::EquipmentSource(s),
            ItemSourcePF2e::FeatSource(s) => Self::FeatSource(s),
            ItemSourcePF2e::HeritageSource(s) => Self::HeritageSource(s),
            ItemSourcePF2e::KitSource(s) => Self::KitSource(s),
            ItemSourcePF2e::LoreSource(s) => Self::LoreSource(s),
            ItemSourcePF2e::MeleeSource(s) => Self::MeleeSource(s),
            ItemSourcePF2e::ShieldSource(s) => Self::ShieldSource(s),
            ItemSourcePF2e::SpellSource(s) => Self::SpellSource(s),
            ItemSourcePF2e::SpellcastingEntrySource(s) => Self::SpellcastingEntrySource(s),
            ItemSourcePF2e::TreasureSource(s) => Self::TreasureSource(s),
            ItemSourcePF2e::WeaponSource(s) => Self::WeaponSource(s),
        }
    }
}
impl<'a> From<&'a PhysicalItemSource> for ItemSourceView<'a> {
    fn from(item: &'a PhysicalItemSource) -> Self {
        match item {
            PhysicalItemSource::ArmorSource(s) => Self::ArmorSource(s),
            PhysicalItemSource::BookSource(s) => Self::BookSource(s),
            PhysicalItemSource::ConsumableSource(s) => Self::ConsumableSource(s),
            PhysicalItemSource::ContainerSource(s) => Self::ContainerSource(s),
            PhysicalItemSource::EquipmentSource(s) => Self::EquipmentSource(s),
            PhysicalItemSource::ShieldSource(s) => Self::ShieldSource(s),
            PhysicalItemSource::TreasureSource(s) => Self::TreasureSource(s),
            PhysicalItemSource::WeaponSource(s) => Self::WeaponSource(s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SourceNodeView<'a> {
    Actor(&'a ActorSourcePF2e),
    Item(ItemSourceView<'a>),
    Journal(&'a JournalEntrySource),
    JournalPage(&'a SourceFromSchemaJournalEntrySchemaPagesEntrySourceFromSchema),
    Macro(&'a MacroSource),
    Table(&'a RollTableSource),
    Result(&'a SourceFromSchemaTableResultSchema),
}
impl<'a> From<&'a FoundryDocumentSource> for SourceNodeView<'a> {
    fn from(source: &'a FoundryDocumentSource) -> Self {
        match source {
            FoundryDocumentSource::Actor(s) => Self::Actor(s),
            FoundryDocumentSource::Item(s) => Self::Item(s.as_ref().into()),
            FoundryDocumentSource::JournalEntry(s) => Self::Journal(s),
            FoundryDocumentSource::Macro(s) => Self::Macro(s),
            FoundryDocumentSource::RollTable(s) => Self::Table(s),
        }
    }
}
impl<'a> SourceNodeView<'a> {
    pub fn id(self) -> SourceFieldView<'a, &'a String> {
        match self {
            Self::Actor(a) => actor_fields!(a, s, (&s._id).into()),
            Self::Item(i) => item_fields!(i, s, (&s._id).into()),
            Self::Journal(s) => (&s._id).into(),
            Self::JournalPage(s) => (&s._id).into(),
            Self::Macro(s) => (&s._id).into(),
            Self::Table(s) => (&s._id).into(),
            Self::Result(s) => (&s._id).into(),
        }
    }
    pub fn name(self) -> SourceFieldView<'a, &'a String> {
        match self {
            Self::Actor(a) => actor_fields!(a, s, (&s.name).into()),
            Self::Item(i) => item_fields!(i, s, (&s.name).into()),
            Self::Journal(s) => (&s.name).into(),
            Self::JournalPage(s) => (&s.name).into(),
            Self::Macro(s) => (&s.name).into(),
            Self::Table(s) => (&s.name).into(),
            Self::Result(_) => SourceFieldView::NotApplicable,
        }
    }
    pub fn document_kind(self) -> &'static str {
        match self {
            Self::Actor(_) => "Actor",
            Self::Item(_) => "Item",
            Self::Journal(_) => "JournalEntry",
            Self::JournalPage(_) => "JournalEntryPage",
            Self::Macro(_) => "Macro",
            Self::Table(_) => "RollTable",
            Self::Result(_) => "TableResult",
        }
    }
    pub fn source_type(self) -> SourceFieldView<'a, &'a str> {
        match self {
            Self::Actor(a) => actor_fields!(
                a,
                s,
                SourceFieldView::from(&s.r#type).map(|_| match a {
                    ActorSourcePF2e::ArmySource(..) => "army",
                    ActorSourcePF2e::CharacterSource(..) => "character",
                    ActorSourcePF2e::FamiliarSource(..) => "familiar",
                    ActorSourcePF2e::HazardSource(..) => "hazard",
                    ActorSourcePF2e::LootSource(..) => "loot",
                    ActorSourcePF2e::NPCSource(..) => "npc",
                    ActorSourcePF2e::PartySource(..) => "party",
                    ActorSourcePF2e::VehicleSource(..) => "vehicle",
                })
            ),
            Self::Item(i) => {
                item_fields!(i, s, SourceFieldView::from(&s.r#type).map(|_| i.family()))
            }
            Self::Macro(s) => SourceFieldView::from(&s.r#type).map(|v| match v {
                MacroType::Chat => "chat",
                MacroType::Script => "script",
            }),
            Self::JournalPage(s) => SourceFieldView::from(&s.r#type).map(String::as_str),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn actor_items(self) -> SourceFieldView<'a, &'a [ItemSourcePF2e]> {
        match self {
            Self::Actor(a) => {
                actor_fields!(a, s, SourceFieldView::from(&s.items).map(Vec::as_slice))
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
}
impl<'a> ItemSourceView<'a> {
    pub fn family(self) -> &'static str {
        match self {
            Self::AbilitySource(..) => "action",
            Self::AfflictionSource(..) => "affliction",
            Self::AncestrySource(..) => "ancestry",
            Self::ArmorSource(..) => "armor",
            Self::BackgroundSource(..) => "background",
            Self::BookSource(..) => "book",
            Self::CampaignFeatureSource(..) => "campaignFeature",
            Self::ClassSource(..) => "class",
            Self::ConditionSource(..) => "condition",
            Self::ConsumableSource(..) => "consumable",
            Self::ContainerSource(..) => "backpack",
            Self::DeitySource(..) => "deity",
            Self::EffectSource(..) => "effect",
            Self::EquipmentSource(..) => "equipment",
            Self::FeatSource(..) => "feat",
            Self::HeritageSource(..) => "heritage",
            Self::KitSource(..) => "kit",
            Self::LoreSource(..) => "lore",
            Self::MeleeSource(..) => "melee",
            Self::ShieldSource(..) => "shield",
            Self::SpellSource(..) => "spell",
            Self::SpellcastingEntrySource(..) => "spellcastingEntry",
            Self::TreasureSource(..) => "treasure",
            Self::WeaponSource(..) => "weapon",
        }
    }
    fn subitems(self) -> SourceFieldView<'a, &'a [PhysicalItemSource]> {
        match self {
            Self::ArmorSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.subitems).into())
                .map(Vec::as_slice),
            Self::EquipmentSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.subitems).into())
                .map(Vec::as_slice),
            Self::ShieldSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.subitems).into())
                .map(Vec::as_slice),
            Self::WeaponSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.subitems).into())
                .map(Vec::as_slice),
            _ => SourceFieldView::NotApplicable,
        }
    }
}

#[derive(Debug)]
pub struct SourceNodeEntry<'a> {
    pub owners: Vec<OwnedContentLocator>,
    pub order: usize,
    pub source: SourceNodeView<'a>,
}

/// Typed traversal in authored order. Unavailable collections remain evidence;
/// they never yield a salvaged subset or prove an empty collection.
pub fn source_nodes<'a>(
    key: &RecordKey,
    source: &'a FoundryDocumentSource,
) -> (Vec<SourceNodeEntry<'a>>, Vec<OwnedCollectionFact>) {
    let mut output = Vec::new();
    let mut collections = Vec::new();
    let mut pending = vec![SourceNodeEntry {
        owners: vec![],
        order: 0,
        source: source.into(),
    }];
    while let Some(entry) = pending.pop() {
        let mut children = Vec::new();
        let mut collect = |field: &str, state: SourceFieldView<'a, Vec<SourceNodeView<'a>>>| {
            collections.push(OwnedCollectionFact {
                locator: SourceContentLocator {
                    record: key.clone(),
                    owners: entry.owners.clone(),
                    field: field.into(),
                },
                availability: state.availability(),
                length: match &state {
                    SourceFieldView::Value(v) => Some(v.len()),
                    _ => None,
                },
            });
            if let SourceFieldView::Value(nodes) = state {
                let mut ids = std::collections::BTreeMap::new();
                for node in &nodes {
                    if let Some(id) = node.id().value().filter(|id| valid_source_id(id)) {
                        *ids.entry(id.as_str()).or_insert(0usize) += 1;
                    }
                }
                for (index, node) in nodes.iter().copied().enumerate() {
                    let identity = match node.id().value() {
                        Some(id) if valid_source_id(id) && ids[id.as_str()] == 1 => {
                            OwnedContentIdentity::Stable(id.clone())
                        }
                        _ => OwnedContentIdentity::SnapshotLocal { index },
                    };
                    let mut owners = entry.owners.clone();
                    owners.push(OwnedContentLocator {
                        collection: field.into(),
                        identity,
                    });
                    children.push(SourceNodeEntry {
                        owners,
                        order: index,
                        source: node,
                    });
                }
            }
        };
        match entry.source {
            SourceNodeView::Actor(_) => collect(
                "/items",
                entry
                    .source
                    .actor_items()
                    .map(|v| v.iter().map(|i| SourceNodeView::Item(i.into())).collect()),
            ),
            SourceNodeView::Item(i) => {
                if !matches!(i.subitems(), SourceFieldView::NotApplicable) {
                    collect(
                        "/system/subitems",
                        i.subitems()
                            .map(|v| v.iter().map(|i| SourceNodeView::Item(i.into())).collect()),
                    );
                }
                if let ItemSourceView::ConsumableSource(s) = i {
                    collect(
                        "/system/spell",
                        SourceFieldView::from(&s.system)
                            .and_then(|s| (&s.spell).into())
                            .map(|s| vec![SourceNodeView::Item(ItemSourceView::SpellSource(s))]),
                    );
                }
            }
            SourceNodeView::Journal(s) => collect(
                "/pages",
                SourceFieldView::from(&s.pages)
                    .map(|v| v.iter().map(SourceNodeView::JournalPage).collect()),
            ),
            SourceNodeView::Table(s) => collect(
                "/results",
                SourceFieldView::from(&s.results)
                    .map(|v| v.iter().map(SourceNodeView::Result).collect()),
            ),
            _ => {}
        }
        pending.extend(children.into_iter().rev());
        output.push(entry);
    }
    (output, collections)
}
