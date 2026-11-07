//! Explicit semantic fact decoration, independent of layout or gameplay evaluation.
use super::*;
use atlas_domain::{DefinitionKey, DefinitionNamespace as N, RecordKey};
use atlas_search::{AtlasRetrievalService, DefinitionReferenceTarget, SourceDefinitionTarget};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn annotate_summaries<'a>(
    summaries: impl Iterator<Item = &'a mut RecordSummaryView>,
    retrieval: &AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    let mut summaries = summaries.collect::<Vec<_>>();
    let keys = summaries
        .iter()
        .flat_map(|summary| {
            summary
                .rarity
                .iter()
                .filter_map(|rarity| DefinitionKey::new(N::Rarity, rarity.value.clone()).ok())
                .chain(
                    summary
                        .traits
                        .iter()
                        .filter_map(|badge| DefinitionKey::new(N::Trait, badge.value.clone()).ok()),
                )
        })
        .collect();
    let context = crate::definitions::HelpContext::load(retrieval, keys)?;
    for summary in &mut summaries {
        if let Some(rarity) = &mut summary.rarity {
            context.value(N::Rarity, rarity)?;
        }
        for badge in &mut summary.traits {
            context.keyed(N::Trait, &badge.value, &mut badge.label, &mut badge.help)?;
        }
    }
    Ok(())
}

pub(crate) fn annotate(
    presentation: &mut RecordPresentationView,
    retrieval: &AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    annotate_many(std::iter::once(presentation), retrieval)
}

pub(crate) fn annotate_many<'a>(
    presentations: impl Iterator<Item = &'a mut RecordPresentationView>,
    retrieval: &AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    let mut presentations = presentations.collect::<Vec<_>>();
    let mut keys = BTreeSet::new();
    let mut general = BTreeMap::new();
    for presentation in &mut presentations {
        visit_values(presentation, &mut |namespace, value| {
            if let Ok(key) = DefinitionKey::new(namespace, value.value.clone()) {
                keys.insert(key);
            }
            Ok(())
        })?;
        visit_keyed(presentation, &mut |namespace, value, _, _| {
            if let Ok(key) = DefinitionKey::new(namespace, value.to_owned()) {
                keys.insert(key);
            }
            Ok(())
        })?;
        visit_activities(presentation, &mut |activity| {
            let navigation = &activity.navigation;
            if navigation.owners.is_empty()
                || activity.family != "action"
                || activity.kind != ActorActivityKindView::Ability
            {
                return Ok(());
            }
            let key = RecordKey::parse(&navigation.record_key)
                .map_err(|e| crate::AppServiceError::invalid_request(e.to_string()))?;
            let identity = (navigation.record_key.clone(), navigation.owners.clone());
            if general.contains_key(&identity) {
                return Ok(());
            }
            if retrieval.reference_coverage() == atlas_search::ReferenceCoverage::Complete {
                let target =
                    match retrieval.source_definition_target(&key, &navigation.owners, None)? {
                        SourceDefinitionTarget::Unique {
                            target: DefinitionReferenceTarget::Definition { key },
                        } if key.namespace() != N::Rule
                            || !key
                                .identifier()
                                .starts_with("pf2e:PF2E.NPC.Abilities.Glossary.") =>
                        {
                            SourceDefinitionTarget::None
                        }
                        SourceDefinitionTarget::Unique {
                            target: DefinitionReferenceTarget::Record { key, .. },
                        } if !matches!(
                            key.pack().as_str(),
                            "bestiary-ability-glossary-srd" | "bestiary-family-ability-glossary"
                        ) =>
                        {
                            SourceDefinitionTarget::None
                        }
                        target => target,
                    };
                if let SourceDefinitionTarget::Unique {
                    target: DefinitionReferenceTarget::Definition { key },
                } = &target
                {
                    keys.insert(key.clone());
                }
                general.insert(
                    (navigation.record_key.clone(), navigation.owners.clone()),
                    target,
                );
            }
            Ok(())
        })?;
    }
    let context = crate::definitions::HelpContext::load(retrieval, keys)?;
    for presentation in &mut presentations {
        visit_values(presentation, &mut |namespace, value| {
            context.value(namespace, value)
        })?;
        visit_keyed(presentation, &mut |namespace, value, label, help| {
            context.keyed(namespace, value, label, help)
        })?;
        visit_activities(presentation, &mut |activity| {
            let identity = (
                activity.navigation.record_key.clone(),
                activity.navigation.owners.clone(),
            );
            activity.help = match general.get(&identity) {
                Some(SourceDefinitionTarget::Unique {
                    target: DefinitionReferenceTarget::Definition { key },
                }) => {
                    let mut value = ReferenceValueView {
                        value: key.identifier().into(),
                        label: activity.title.clone(),
                        help: None,
                    };
                    context.value(key.namespace(), &mut value)?;
                    value.help
                }
                Some(SourceDefinitionTarget::Unique {
                    target: DefinitionReferenceTarget::Record { key, owners },
                }) => Some(ReferenceHelpView {
                    definition_key: None,
                    destination: ReferenceDestinationView::Record {
                        navigation: crate::definitions::record_navigation(
                            key.clone(),
                            owners.clone(),
                            retrieval.source_fingerprint(),
                        )
                        .into(),
                    },
                }),
                _ => None,
            };
            Ok(())
        })?;
    }
    Ok(())
}

fn one(
    fact: &mut FactView<ReferenceValueView>,
    namespace: N,
    visit: &mut impl FnMut(N, &mut ReferenceValueView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    if let Some(value) = &mut fact.value {
        visit(namespace, value)?;
    }
    Ok(())
}
fn many(
    fact: &mut FactView<Vec<ReferenceValueView>>,
    namespace: N,
    visit: &mut impl FnMut(N, &mut ReferenceValueView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    for value in fact.value.iter_mut().flatten() {
        visit(namespace, value)?;
    }
    Ok(())
}
fn damage(
    damage: &mut FactView<Vec<DamageComponentView>>,
    visit: &mut impl FnMut(N, &mut ReferenceValueView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    for component in damage.value.iter_mut().flatten() {
        one(&mut component.damage_type, N::DamageType, visit)?;
    }
    Ok(())
}
fn activity_values(
    activity: &mut ActorActivityView,
    visit: &mut impl FnMut(N, &mut ReferenceValueView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    one(&mut activity.casting_tradition, N::Tradition, visit)?;
    damage(&mut activity.damage, visit)?;
    if let Some(effect) = &mut activity.spell_effect {
        for strike in &mut effect.strikes {
            one(&mut strike.damage_type, N::DamageType, visit)?;
        }
    }
    Ok(())
}
fn actor_values(
    actor: &mut ActorPresentationView,
    visit: &mut impl FnMut(N, &mut ReferenceValueView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    one(&mut actor.size, N::Size, visit)?;
    many(&mut actor.languages, N::Language, visit)?;
    for sense in actor.senses.value.iter_mut().flatten() {
        one(&mut sense.sense, N::Sense, visit)?;
    }
    for (entries, namespace) in [
        (&mut actor.immunities, N::ImmunityType),
        (&mut actor.weaknesses, N::WeaknessType),
        (&mut actor.resistances, N::ResistanceType),
    ] {
        for entry in entries.value.iter_mut().flatten() {
            one(&mut entry.damage_type, namespace, visit)?;
        }
    }
    for activity in actor.activities.value.iter_mut().flatten() {
        activity_values(activity, visit)?;
    }
    if let Some(runtime) = &mut actor.runtime {
        for sense in &mut runtime.senses {
            visit(N::Sense, &mut sense.sense_type)?;
        }
    }
    Ok(())
}
fn spell_changes(
    changes: &mut FactView<SpellChangeView>,
    visit: &mut impl FnMut(N, &mut ReferenceValueView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    if let Some(changes) = &mut changes.value {
        many(&mut changes.traits, N::Trait, visit)?;
        many(&mut changes.traditions, N::Tradition, visit)?;
        damage(&mut changes.damage, visit)?;
        if let Some(heightening) = &mut changes.heightening.value {
            for level in heightening.levels.value.iter_mut().flatten() {
                spell_changes(&mut level.changes, visit)?;
            }
        }
    }
    Ok(())
}
fn visit_values(
    presentation: &mut RecordPresentationView,
    visit: &mut impl FnMut(N, &mut ReferenceValueView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    if let Some(rarity) = &mut presentation.identity.rarity {
        visit(N::Rarity, rarity)?;
    }
    match &mut presentation.body {
        RecordBodyView::Creature(actor) => actor_values(actor, visit)?,
        RecordBodyView::Hazard(hazard) => actor_values(&mut hazard.actor, visit)?,
        RecordBodyView::Activity(activity) => activity_values(activity, visit)?,
        RecordBodyView::Spell(spell) => {
            many(&mut spell.traditions, N::Tradition, visit)?;
            damage(&mut spell.damage, visit)?;
            for form in spell.forms.value.iter_mut().flatten() {
                spell_changes(&mut form.changes, visit)?;
            }
            if let Some(SpellHeighteningView::Fixed { levels }) = &mut spell.heightening.value {
                for level in levels.value.iter_mut().flatten() {
                    spell_changes(&mut level.changes, visit)?;
                }
            }
        }
        RecordBodyView::PhysicalReference(physical) => {
            one(&mut physical.size, N::Size, visit)?;
            if let Some(consumable) = &mut physical.consumable
                && let Some(damage) = &mut consumable.damage.value
            {
                one(&mut damage.damage_type, N::DamageType, visit)?;
            }
            match &mut physical.details {
                Some(PhysicalDetailsView::Weapon(weapon)) => {
                    one(&mut weapon.group, N::WeaponGroup, visit)?;
                    if let Some(damage) = &mut weapon.damage.value {
                        one(&mut damage.damage_type, N::DamageType, visit)?;
                        if let Some(persistent) = &mut damage.persistent.value {
                            one(&mut persistent.damage_type, N::DamageType, visit)?;
                        }
                    }
                    if let Some(melee) = &mut weapon.melee_usage.value {
                        one(&mut melee.group, N::WeaponGroup, visit)?;
                        one(&mut melee.damage_type, N::DamageType, visit)?;
                        many(&mut melee.traits, N::Trait, visit)?;
                    }
                }
                Some(PhysicalDetailsView::Armor(armor)) => {
                    one(&mut armor.group, N::ArmorGroup, visit)?
                }
                _ => {}
            }
        }
        RecordBodyView::Ancestry(ancestry) => {
            one(&mut ancestry.size, N::Size, visit)?;
            one(&mut ancestry.vision, N::Sense, visit)?;
            many(&mut ancestry.languages, N::Language, visit)?;
            many(&mut ancestry.additional_languages, N::Language, visit)?;
            for choices in [&mut ancestry.boosts, &mut ancestry.flaws] {
                for choice in choices.value.iter_mut().flatten() {
                    many(&mut choice.choices, N::Ability, visit)?;
                }
            }
        }
        RecordBodyView::Class(class) => {
            many(&mut class.key_attributes, N::Ability, visit)?;
            many(&mut class.trained_skills, N::Skill, visit)?;
        }
        RecordBodyView::Deity(deity) => {
            many(&mut deity.attributes, N::Ability, visit)?;
            many(&mut deity.skills, N::Skill, visit)?;
            many(&mut deity.primary_domains, N::Domain, visit)?;
            many(&mut deity.alternate_domains, N::Domain, visit)?;
        }
        RecordBodyView::Army(army) => {
            for (entries, namespace) in [
                (&mut army.immunities, N::ImmunityType),
                (&mut army.weaknesses, N::WeaknessType),
                (&mut army.resistances, N::ResistanceType),
            ] {
                for entry in entries.value.iter_mut().flatten() {
                    one(&mut entry.damage_type, namespace, visit)?;
                }
            }
        }
        RecordBodyView::Vehicle(vehicle) => {
            one(&mut vehicle.size, N::Size, visit)?;
            for (entries, namespace) in [
                (&mut vehicle.defenses.immunities, N::ImmunityType),
                (&mut vehicle.defenses.weaknesses, N::WeaknessType),
                (&mut vehicle.defenses.resistances, N::ResistanceType),
            ] {
                for entry in entries.value.iter_mut().flatten() {
                    one(&mut entry.damage_type, namespace, visit)?;
                }
            }
        }
        RecordBodyView::Heritage(_)
        | RecordBodyView::RollTable(_)
        | RecordBodyView::TableResult(_)
        | RecordBodyView::Journal(_)
        | RecordBodyView::JournalPage(_)
        | RecordBodyView::Content => {}
    }
    Ok(())
}

fn visit_activities(
    presentation: &mut RecordPresentationView,
    visit: &mut impl FnMut(&mut ActorActivityView) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    match &mut presentation.body {
        RecordBodyView::Activity(activity) => visit(activity)?,
        RecordBodyView::Creature(actor) => {
            for activity in actor.activities.value.iter_mut().flatten() {
                visit(activity)?;
            }
        }
        RecordBodyView::Hazard(hazard) => {
            for activity in hazard.actor.activities.value.iter_mut().flatten() {
                visit(activity)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn visit_keyed(
    presentation: &mut RecordPresentationView,
    visit: &mut impl FnMut(
        N,
        &str,
        &mut String,
        &mut Option<ReferenceHelpView>,
    ) -> crate::AppServiceResult<()>,
) -> crate::AppServiceResult<()> {
    for badge in &mut presentation.identity.traits {
        visit(N::Trait, &badge.value, &mut badge.label, &mut badge.help)?;
    }
    let actor = match &mut presentation.body {
        RecordBodyView::Creature(actor) => Some(actor.as_mut()),
        RecordBodyView::Hazard(hazard) => Some(&mut hazard.actor),
        _ => None,
    };
    if let Some(actor) = actor {
        for ability in actor.abilities.value.iter_mut().flatten() {
            visit(
                N::Ability,
                &ability.key,
                &mut ability.label,
                &mut ability.help,
            )?;
        }
        for skill in actor.skills.value.iter_mut().flatten() {
            visit(N::Skill, &skill.key, &mut skill.label, &mut skill.help)?;
        }
    }
    visit_activities(presentation, &mut |activity| {
        for badge in &mut activity.traits {
            visit(N::Trait, &badge.value, &mut badge.label, &mut badge.help)?;
        }
        if let Some(effect) = &mut activity.spell_effect {
            for strike in &mut effect.strikes {
                for badge in &mut strike.traits {
                    visit(N::Trait, &badge.value, &mut badge.label, &mut badge.help)?;
                }
            }
        }
        Ok(())
    })
}
