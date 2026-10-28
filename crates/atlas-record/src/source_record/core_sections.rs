//! Exact Core membership over checked, already audience-admitted search content.
use super::{ItemSourceView, SourceBackedRecord, SourceFieldView, SourceNodeView};
use super::{SourceSearchSelection, SourceSelectionError, traversal::visit_source_nodes};
use crate::source_content::SourceContentLocator;
use atlas_domain::{CoreFallbackReason, CoreSectionFamily, OwnedContentLocator};
use atlas_foundry_model::{ActorSourcePF2e, generated::*};

/// An ephemeral membership projection, not another source or content inventory.
/// Fields select every existing structural section/chunk under the exact address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreSearchSelection {
    pub family: CoreSectionFamily,
    /// Admitted identity names for exact eligible owners, including empty prose.
    pub owners: Vec<Vec<OwnedContentLocator>>,
    pub fields: Vec<SourceContentLocator>,
}

impl CoreSearchSelection {
    pub fn fallback_reason(&self) -> Option<CoreFallbackReason> {
        if !self.family.is_modelled() {
            Some(CoreFallbackReason::UnmodelledKind)
        } else if self.fields.is_empty() {
            Some(CoreFallbackReason::NoCoreProse)
        } else {
            None
        }
    }
}

/// Restrict the canonical audience-admitted selection using typed family, direct
/// ownership and category. No text preparation, admission or vector reads occur.
/// The caller supplies the existing selection from this exact checked snapshot.
pub fn core_search_selection(
    record: &SourceBackedRecord,
    selected: &SourceSearchSelection,
) -> Result<CoreSearchSelection, SourceSelectionError> {
    if selected.root != *record.key()
        || selected
            .fields
            .iter()
            .any(|field| field.locator.record != *record.key())
    {
        return Err(SourceSelectionError(
            "Core selection belongs to a different checked record".into(),
        ));
    }
    let family = core_section_family(SourceNodeView::from(record.source()));
    let mut allowed_owners = Vec::new();
    if matches!(
        family,
        CoreSectionFamily::Hazard
            | CoreSectionFamily::Vehicle
            | CoreSectionFamily::Army
            | CoreSectionFamily::Character
            | CoreSectionFamily::Familiar
    ) {
        visit_source_nodes(record.source(), |owners, node| {
            if matches!(owners, [owner] if owner.collection == "/items")
                && core_owned_document(family, node)
            {
                allowed_owners.push(owners.to_vec());
            }
        });
    }
    let owners = selected
        .identities
        .iter()
        .filter(|identity| identity.owners.is_empty() || allowed_owners.contains(&identity.owners))
        .map(|identity| identity.owners.clone())
        .collect();
    let fields = selected
        .fields
        .iter()
        .filter(|field| {
            !field.sections.is_empty()
                && if field.locator.owners.is_empty() {
                    core_root_field(family, &field.locator.field)
                } else {
                    field.locator.field == "/system/description/value"
                        && allowed_owners.contains(&field.locator.owners)
                }
        })
        .map(|field| field.locator.clone())
        .collect();
    Ok(CoreSearchSelection {
        family,
        owners,
        fields,
    })
}

fn core_section_family(root: SourceNodeView<'_>) -> CoreSectionFamily {
    use CoreSectionFamily as F;
    match root {
        SourceNodeView::Actor(actor) => match actor {
            ActorSourcePF2e::NPCSource(_) => F::Npc,
            ActorSourcePF2e::HazardSource(_) => F::Hazard,
            ActorSourcePF2e::VehicleSource(_) => F::Vehicle,
            ActorSourcePF2e::ArmySource(_) => F::Army,
            ActorSourcePF2e::CharacterSource(_) => F::Character,
            ActorSourcePF2e::FamiliarSource(_) => F::Familiar,
            ActorSourcePF2e::LootSource(_) | ActorSourcePF2e::PartySource(_) => F::Unmodelled,
        },
        SourceNodeView::Item(item) => match item {
            ItemSourceView::SpellSource(_) => F::Spell,
            ItemSourceView::AbilitySource(_)
            | ItemSourceView::FeatSource(_)
            | ItemSourceView::EffectSource(_)
            | ItemSourceView::ConditionSource(_) => F::Rule,
            ItemSourceView::AmmoSource(_)
            | ItemSourceView::ArmorSource(_)
            | ItemSourceView::BookSource(_)
            | ItemSourceView::ConsumableSource(_)
            | ItemSourceView::ContainerSource(_)
            | ItemSourceView::EquipmentSource(_)
            | ItemSourceView::KitSource(_)
            | ItemSourceView::ShieldSource(_)
            | ItemSourceView::TreasureSource(_)
            | ItemSourceView::WeaponSource(_) => F::Equipment,
            ItemSourceView::AncestrySource(_)
            | ItemSourceView::HeritageSource(_)
            | ItemSourceView::BackgroundSource(_)
            | ItemSourceView::ClassSource(_) => F::CharacterOption,
            ItemSourceView::DeitySource(_) => F::Deity,
            ItemSourceView::CampaignFeatureSource(_) => F::CampaignFeature,
            ItemSourceView::AfflictionSource(_)
            | ItemSourceView::LoreSource(_)
            | ItemSourceView::MeleeSource(_)
            | ItemSourceView::SpellcastingEntrySource(_) => F::Unmodelled,
        },
        SourceNodeView::Journal(_)
        | SourceNodeView::JournalPage(_)
        | SourceNodeView::Macro(_)
        | SourceNodeView::Table(_)
        | SourceNodeView::Result(_) => F::Unmodelled,
    }
}

fn core_root_field(family: CoreSectionFamily, field: &str) -> bool {
    use CoreSectionFamily as F;
    match family {
        F::Npc => matches!(
            field,
            "/system/details/publicNotes" | "/system/details/blurb"
        ),
        F::Hazard => matches!(
            field,
            "/system/details/description" | "/system/details/routine"
        ),
        F::Vehicle | F::Army => field == "/system/details/description",
        F::Character => matches!(
            field,
            "/system/details/biography/appearance" | "/system/details/biography/backstory"
        ),
        F::Spell | F::Rule | F::Equipment | F::CharacterOption | F::Deity | F::CampaignFeature => {
            field == "/system/description/value"
        }
        F::Familiar | F::Trait | F::Unmodelled => false,
    }
}

fn core_owned_document(family: CoreSectionFamily, node: SourceNodeView<'_>) -> bool {
    use CoreSectionFamily as F;
    let SourceNodeView::Item(item) = node else {
        return false;
    };
    match (family, item) {
        (F::Hazard, ItemSourceView::AbilitySource(_) | ItemSourceView::MeleeSource(_))
        | (F::Vehicle, ItemSourceView::AbilitySource(_))
        | (
            F::Character,
            ItemSourceView::AncestrySource(_)
            | ItemSourceView::HeritageSource(_)
            | ItemSourceView::BackgroundSource(_)
            | ItemSourceView::ClassSource(_),
        ) => true,
        (F::Army, ItemSourceView::CampaignFeatureSource(source)) => {
            matches!(
                SourceFieldView::from(&source.system)
                    .and_then(|system| (&system.category).into())
                    .value(),
                Some(
                    CampaignFeatureSystemSourceCategory::ArmyTactic
                        | CampaignFeatureSystemSourceCategory::ArmyWarAction
                )
            )
        }
        (F::Familiar, ItemSourceView::AbilitySource(source)) => {
            matches!(
                SourceFieldView::from(&source.system)
                    .and_then(|system| (&system.category).into())
                    .value(),
                Some(AbilitySystemSourceCategory::Familiar)
            )
        }
        _ => false,
    }
}
