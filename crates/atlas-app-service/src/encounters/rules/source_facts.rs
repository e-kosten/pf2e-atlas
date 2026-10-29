//! Adapt checked record facts without interpreting raw source or display text.
use super::super::stat_block::StatBlockView;
use super::senses::{Precision, SenseFacts};
use atlas_app_model::EncounterSenseView;
use atlas_foundry_model::generated::{
    FeatSystemSourceSubfeaturesSourceFromSchemaSensesPartialBloodsenseSourceFromSchemaAcuity as Acuity,
    SenseConstructorParamsType as SenseType,
};
use atlas_record::source_record::{SourceBackedRecord, SourceFieldView, SourceQueryView};

pub(in crate::encounters) fn adapt_senses(block: &mut StatBlockView, record: &SourceBackedRecord) {
    let actor = SourceQueryView::new(record.source(), record.key().pack().as_str(), "").actor();
    // Vision is an explicit independent source fact; never synthesize it from a name.
    if let Some(vision) = actor.vision().value() {
        block.sense_facts.push(SenseFacts {
            visual: true,
            auditory: false,
            precision: Some(Precision::Precise),
            range_feet: None,
            available: Some(vision),
        });
        block.senses.push(EncounterSenseView {
            sense_type: "vision".into(),
            precision: Some("precise".into()),
            range_feet: None,
            available: Some(vision),
            perception: None,
            reason: None,
        });
    }
    if let Some(senses) = actor.senses().value() {
        block.senses_known = true;
        for sense in senses {
            let kind = SourceFieldView::from(&sense.r#type).value().map(|v| {
                match v {
                    SenseType::Bloodsense => "bloodsense",
                    SenseType::Darkvision => "darkvision",
                    SenseType::Echolocation => "echolocation",
                    SenseType::ElectromagneticSense => "electromagnetic-sense",
                    SenseType::GreaterDarkvision => "greater-darkvision",
                    SenseType::InfraredVision => "infrared-vision",
                    SenseType::Lifesense => "lifesense",
                    SenseType::LowLightVision => "low-light-vision",
                    SenseType::Magicsense => "magicsense",
                    SenseType::MotionSense => "motion-sense",
                    SenseType::Scent => "scent",
                    SenseType::SeeInvisibility => "see-invisibility",
                    SenseType::Spiritsense => "spiritsense",
                    SenseType::Thoughtsense => "thoughtsense",
                    SenseType::Tremorsense => "tremorsense",
                    SenseType::Truesight => "truesight",
                    SenseType::Wavesense => "wavesense",
                }
                .to_owned()
            });
            let Some(kind) = kind else {
                continue;
            };
            let typed_precision = SourceFieldView::from(&sense.r#type)
                .value()
                .and_then(|kind| effective_precision(*kind, SourceFieldView::from(&sense.acuity)));
            let precision = typed_precision.map(|precision| {
                match precision {
                    Precision::Precise => "precise",
                    Precision::Imprecise => "imprecise",
                    Precision::Vague => "vague",
                }
                .to_owned()
            });
            let range = SourceFieldView::from(&sense.range).value().cloned();
            let visual = SourceFieldView::from(&sense.r#type)
                .value()
                .is_some_and(|kind| is_visual(*kind));
            let available = if visual {
                actor.vision().value()
            } else {
                Some(true)
            };
            block.sense_facts.push(SenseFacts {
                visual,
                auditory: SourceFieldView::from(&sense.r#type)
                    .value()
                    .is_some_and(|kind| *kind == SenseType::Echolocation),
                precision: typed_precision,
                range_feet: range.as_ref().and_then(|n| n.as_f64()),
                available,
            });
            block.senses.push(EncounterSenseView {
                sense_type: kind,
                precision,
                range_feet: range,
                available,
                perception: None,
                reason: None,
            });
        }
    }
}

/// Classify checked sense identity, independently of its presentation label.
fn is_visual(kind: SenseType) -> bool {
    matches!(
        kind,
        SenseType::Darkvision
            | SenseType::GreaterDarkvision
            | SenseType::InfraredVision
            | SenseType::LowLightVision
            | SenseType::SeeInvisibility
            | SenseType::Truesight
    )
}

/// Pinned Sense.defineSchema defaults omitted acuity to precise; constructor applies
/// mandatory acuities. Rejected/null authored values stay unknown.
fn effective_precision(kind: SenseType, acuity: SourceFieldView<'_, &Acuity>) -> Option<Precision> {
    let value = match acuity {
        SourceFieldView::Missing => Precision::Precise,
        SourceFieldView::Value(Acuity::Precise) => Precision::Precise,
        SourceFieldView::Value(Acuity::Imprecise) => Precision::Imprecise,
        SourceFieldView::Value(Acuity::Vague) => Precision::Vague,
        _ => return None,
    };
    if is_visual(kind) || kind == SenseType::Echolocation {
        Some(Precision::Precise)
    } else {
        Some(value)
    }
}

pub(in crate::encounters) fn authored_stat_baseline(
    record: &SourceBackedRecord,
    fingerprint: &str,
    selection: Option<(&[atlas_domain::OwnedContentLocator], u8)>,
) -> Option<StatBlockView> {
    use super::super::stat_block::*;
    use atlas_record::source_record::{ActorSave, SourceNodeView};
    let query = SourceQueryView::new(record.source(), record.key().pack().as_str(), "");
    if !matches!(query.source, SourceNodeView::Actor(_)) {
        return None;
    }
    #[cfg(test)]
    super::super::mechanics::record_baseline();
    let actor = query.actor();
    let mut block = StatBlockView {
        source_conditions: Default::default(),
        level: actor.level().value().cloned(),
        adjusted_level: actor.level().value().cloned(),
        values: vec![],
        replacements: vec![],
        unresolved_stats: vec![],
        senses: vec![],
        senses_known: false,
        sense_facts: vec![],
        contextual_armor_class: vec![],
        speeds: vec![],
        action_budget: None,
        activities: vec![],
        unapplied_effects: vec![],
    };
    let mut push = |target: StatisticTarget,
                    label: String,
                    ability: Option<Ability>,
                    number: Option<&serde_json::Number>| {
        if let Some(number) = number {
            block.values.push(StatValueView {
                notes: vec![],
                target,
                label,
                ability,
                base_value: number.clone(),
                adjusted_value: number.clone(),
                modifiers: vec![],
                suppressed_modifiers: vec![],
            });
        }
    };
    push(
        StatisticTarget::ArmorClass,
        "AC".into(),
        Some(Ability::Dexterity),
        actor.armor_class().value(),
    );
    push(
        StatisticTarget::HitPointMaximum,
        "Maximum HP".into(),
        None,
        actor.hp_maximum().value(),
    );
    push(
        StatisticTarget::Perception,
        "Perception".into(),
        Some(Ability::Wisdom),
        actor.perception().value(),
    );
    for (key, save, save_kind, ability) in [
        (
            "fortitude",
            ActorSave::Fortitude,
            StatisticSave::Fortitude,
            Ability::Constitution,
        ),
        (
            "reflex",
            ActorSave::Reflex,
            StatisticSave::Reflex,
            Ability::Dexterity,
        ),
        (
            "will",
            ActorSave::Will,
            StatisticSave::Will,
            Ability::Wisdom,
        ),
    ] {
        push(
            StatisticTarget::Save(save_kind),
            key.into(),
            Some(ability),
            actor.save(save).value(),
        );
    }
    push(
        StatisticTarget::HazardStealth,
        "Stealth".into(),
        Some(Ability::Dexterity),
        actor.hazard_stealth().value(),
    );
    push(
        StatisticTarget::Hardness,
        "Hardness".into(),
        None,
        actor
            .hazard_hardness()
            .value()
            .or_else(|| actor.creature_hardness().value()),
    );
    actor.visit_abilities(|key, value| {
        if let Some(ability) = Ability::from_key(key) {
            push(
                StatisticTarget::Ability(ability),
                key.to_uppercase(),
                Some(ability),
                value.value(),
            );
        }
    });
    actor.visit_skills(|key, value| {
        push(
            StatisticTarget::Skill(key.into()),
            key.into(),
            skill_ability(key),
            value.and_then(|value| (&value.base).into()).value(),
        )
    });
    actor.visit_skills(|key, skill| {
        if let Some(skill) = skill.value() {
            if let Some(note) = SourceFieldView::from(&skill.note).value() {block.unapplied_effects.push(atlas_app_model::UnappliedEffectView {condition_id: None, source: format!("skill.{key}"), label: key.into(), reason: note.clone()});}
            if let Some(special) = SourceFieldView::from(&skill.special).value() {
                for conditional in special {
                    if let (Some(label), Some(modifier)) = (SourceFieldView::from(&conditional.label).value(), SourceFieldView::from(&conditional.base).value()) {
                        block.unapplied_effects.push(atlas_app_model::UnappliedEffectView {condition_id: None, source: format!("skill.{key}"), label: label.clone(), reason: format!("Conditional modifier {modifier}; apply only when its authored predicate matches.")});
                    }
                }
            }
        }
    });
    if let Some(speed) = actor.land_speed().value() {
        block.speeds.push(speed_fact("land", speed));
    }
    if let Some(speeds) = actor.speeds().value() {
        for speed in speeds.iter() {
            if let (Some(kind), Some(number)) = (speed.speed_type().value(), speed.value().value())
            {
                let key = match kind {
                    atlas_foundry_model::generated::LabeledSpeedType::Burrow => "burrow",
                    atlas_foundry_model::generated::LabeledSpeedType::Climb => "climb",
                    atlas_foundry_model::generated::LabeledSpeedType::Fly => "fly",
                    atlas_foundry_model::generated::LabeledSpeedType::Swim => "swim",
                };
                block.speeds.push(speed_fact(key, number));
            }
        }
    }
    // Exact embedded identities and source generation are attribution, not arithmetic input.
    let mut entries = std::collections::BTreeMap::new();
    record.visit_immediate_actor_items(|_, _, item| {
        if let atlas_record::source_record::ItemSourceView::SpellcastingEntrySource(entry) = item
            && let Some(id) = SourceFieldView::from(&entry._id).value()
        {
            entries
                .entry(id.as_str())
                .and_modify(|entry| *entry = None)
                .or_insert(Some(item));
        }
    });
    let mut activity_index = 0;
    record.visit_immediate_actor_items(|_, owner, item| {
        use atlas_record::source_record::{ItemSourceView, SourceNodeView};
        let heightened = match (item, selection) {
            (ItemSourceView::SpellSource(source), Some((owners, rank))) if owners == std::slice::from_ref(owner) => Some(crate::presentation::spell_rank::heighten(source, rank)),
            _ => None,
        };
        let heightening_error = heightened.as_ref().and_then(|result|result.as_ref().err()).cloned();
        let item = heightened.as_ref().and_then(|result|result.as_ref().ok()).map(ItemSourceView::SpellSource).unwrap_or(item);
        let node = SourceNodeView::Item(item);
        let index = activity_index;
        activity_index += 1;
        if let Some(number) = item.lore_modifier().value() {
            block.values.push(StatValueView { notes: vec![],target: StatisticTarget::Skill(format!("lore.{index}")), label: node.name().value().cloned().unwrap_or_default(), ability: Some(Ability::Intelligence), base_value: number.clone(), adjusted_value: number.clone(), modifiers: vec![], suppressed_modifiers: vec![]});
        }
        if let Some(variants)=item.lore_variants().value() {
            for (_,variant) in &variants.entries {
                if let Some(label)=SourceFieldView::from(&variant.label).value()
                    && let Some(reason)=SourceFieldView::from(&variant.options).value() {
                        block.unapplied_effects.push(atlas_app_model::UnappliedEffectView {condition_id:None,source:node.name().value().cloned().unwrap_or_default(),label:label.clone(),reason:reason.clone()});
                    }
            }
        }
        let kind = match item { ItemSourceView::MeleeSource(_) => MechanicActivityKindView::Strike, ItemSourceView::SpellSource(_) => MechanicActivityKindView::Spell, ItemSourceView::SpellcastingEntrySource(_) => MechanicActivityKindView::CastingEntry, _ => MechanicActivityKindView::Other };
        let owners = vec![owner.clone()];
        let navigation = atlas_app_model::RecordNavigationView {spell_rank: None, record_key: record.key().to_string(), source_fingerprint: crate::projection::navigation_fingerprint(&owners, fingerprint), owners, field: None, passage: None};
        let traits = SourceQueryView {source: node, pack_id: "", pack_label: ""}.traits().value().map(<[String]>::to_vec);
        let ranged = match item {ItemSourceView::MeleeSource(source) => match SourceFieldView::from(&source.system).and_then(|s| (&s.range).into()) {SourceFieldView::Value(_) => Some(true), SourceFieldView::Null | SourceFieldView::Missing => Some(false), _ => None}, _ => None};
        let (attack_ability, damage_ability) = strike_abilities(ranged, traits.as_deref());
        let entry = match item {
            ItemSourceView::SpellcastingEntrySource(_) => Some(item),
            ItemSourceView::SpellSource(source) => SourceFieldView::from(&source.system).and_then(|s| (&s.location).into()).and_then(|l| (&l.value).into()).value().and_then(|id| entries.get(id.as_str())).and_then(|entry| *entry),
            _ => None,
        };
        let casting_ability = entry.and_then(casting_ability);
        let attack = entry.and_then(|entry| match entry {ItemSourceView::SpellcastingEntrySource(source) => SourceFieldView::from(&source.system).and_then(|s| (&s.spelldc).into()).and_then(|s| (&s.value).into()).value(), _ => None}).or_else(|| item.melee_attack().value());
        let dc = entry.and_then(|entry| match entry {ItemSourceView::SpellcastingEntrySource(source) => SourceFieldView::from(&source.system).and_then(|s| (&s.spelldc).into()).and_then(|s| (&s.dc).into()).value(), _ => None});
        let mut activity = MechanicActivityView {calculation_unavailable: heightening_error, traits_known: traits.is_some(), is_ranged: ranged, activity_id: node.id().value().cloned().unwrap_or_default(), label: node.name().value().cloned().unwrap_or_default(), kind, traits: traits.unwrap_or_default(), navigation, notes: vec![], rolls: vec![], damage: vec![], attack_ability: if kind == MechanicActivityKindView::Strike {attack_ability} else {casting_ability}, difficulty_class: dc.map(|n| StatValueView { notes: vec![],target: StatisticTarget::SpellDc, label: "Spell DC".into(), ability: casting_ability, base_value: n.clone(), adjusted_value: n.clone(), modifiers: vec![], suppressed_modifiers: vec![]}), spell_usage: spell_usage(item)};
        if let Some(number) = attack { activity.rolls.push(ActivityRollView {roll_id: "attack".into(), label: "Attack".into(), base_value: number.clone(), adjusted_value: number.clone(), surface: ActivityRollSurfaceView::AttackRoll, modifiers: vec![], suppressed_modifiers: vec![]}); }
        if let Some(damage) = item.melee_damage().value() {
            for (source_ordinal, (id, damage)) in damage.entries.iter().enumerate() {
                let Some(formula) = SourceFieldView::from(&damage.damage).value() else { continue; };
                if formula.trim().is_empty() {continue;}
                let damage_type = SourceFieldView::from(&damage.damage_type).value().copied();
                let category = if damage_type == Some(atlas_foundry_model::generated::ModularConfigDamageType::Bleed) {Some("persistent".into())} else {SourceFieldView::from(&damage.category).value().map(|category| match category {atlas_foundry_model::generated::RawModifierDamageCategory::Persistent => "persistent", atlas_foundry_model::generated::RawModifierDamageCategory::Precision => "precision", atlas_foundry_model::generated::RawModifierDamageCategory::Splash => "splash"}.into())};
                activity.damage.push(DamageExpressionView {ability_known: source_ordinal != 0 || category.is_some() || (ranged.is_some() && activity.traits_known), ability: if source_ordinal == 0 && category.is_none() {damage_ability} else {None}, unavailable_reason: None, source_ordinal, damage_id: id.clone(), label: None, formula: formula.clone(), adjusted_formula: None, damage_type: damage_type.map(atlas_foundry_model::generated::ModularConfigDamageType::as_str).map(str::to_owned), category, effect_kind: DamageEffectKindView::Damage, modifiers: vec![], suppressed_modifiers: vec![]});
            }
        }
        if let Some(damage) = item.spell_damage().value() {
            for (source_ordinal, (id, damage)) in damage.entries.iter().enumerate() {
                let Some(formula) = SourceFieldView::from(&damage.formula).value() else {continue;};
                let kinds = match SourceFieldView::from(&damage.kinds) {
                    SourceFieldView::Missing => Some(&[atlas_foundry_model::generated::DamageKind::Damage][..]),
                    SourceFieldView::Value(kinds) if kinds.is_empty() => Some(&[atlas_foundry_model::generated::DamageKind::Damage][..]),
                    SourceFieldView::Value(kinds) => Some(kinds.as_slice()),
                    _ => None,
                };
                let effect_kind = match kinds {Some([atlas_foundry_model::generated::DamageKind::Damage]) => DamageEffectKindView::Damage, Some([atlas_foundry_model::generated::DamageKind::Healing]) => DamageEffectKindView::Healing, Some(kinds) if kinds.len() > 1 => DamageEffectKindView::Mixed, _ => DamageEffectKindView::Unknown};
                let category = SourceFieldView::from(&damage.category).value().and_then(|category| match category {atlas_foundry_model::generated::SpellDamageSourceCategory::Empty => None, atlas_foundry_model::generated::SpellDamageSourceCategory::Persistent => Some("persistent"), atlas_foundry_model::generated::SpellDamageSourceCategory::Precision => Some("precision"), atlas_foundry_model::generated::SpellDamageSourceCategory::Splash => Some("splash")}).map(str::to_owned);
                let apply_ability = match SourceFieldView::from(&damage.apply_mod) {
                    SourceFieldView::Missing => Some(false),
                    SourceFieldView::Value(value) => Some(*value),
                    _ => None,
                };
                let ability = apply_ability.filter(|value|*value).and(casting_ability);
                let ability_value = ability.and_then(|ability|block.values.iter().find(|value|value.target==StatisticTarget::Ability(ability)).and_then(|value|value.base_value.as_i64()));
                let unavailable_reason = match (apply_ability,ability_value) {
                    (None,_) => Some("Spell damage modifier is unavailable.".into()),
                    (Some(true),None) => Some("Spellcasting ability modifier is unavailable.".into()),
                    _ => None,
                };
                let ability_modifiers = ability_value.map(|value|super::modifiers::Modifier {source:"spellcasting-ability".into(),label:"Spellcasting ability".into(),modifier_type:super::modifiers::ModifierType::Adjustment,value}).into_iter().collect();
                activity.damage.push(DamageExpressionView {ability_known: apply_ability == Some(false) || ability.is_some(), ability, unavailable_reason, source_ordinal, damage_id: id.clone(), label: None, formula: formula.clone(), adjusted_formula: None, damage_type: SourceFieldView::from(&damage.r#type).value().copied().map(atlas_foundry_model::generated::ModularConfigDamageType::as_str).map(str::to_owned), category, effect_kind, modifiers: ability_modifiers, suppressed_modifiers: vec![]});
            }
        }
        if let Some(reason) = &activity.calculation_unavailable {
            activity.rolls.clear();
            activity.difficulty_class = None;
            for damage in &mut activity.damage {damage.unavailable_reason=Some(reason.clone());}
            activity.notes.push(atlas_app_model::RuntimeEffectNoteView {source:"spell-rank".into(),label:"Spell rank".into(),reason:reason.clone()});
        }
        block.activities.push(activity);
    });
    adapt_senses(&mut block, record);
    Some(block)
}

fn speed_fact(
    kind: &str,
    number: &serde_json::Number,
) -> super::super::stat_block::MovementSpeedView {
    super::super::stat_block::MovementSpeedView {
        movement_type: kind.into(),
        label: kind.into(),
        base_value_feet: number.clone(),
        adjusted_value_feet: number.clone(),
        adjustments: vec![],
        suppressed_adjustments: vec![],
        notes: vec![],
    }
}
fn skill_ability(key: &str) -> Option<super::super::stat_block::Ability> {
    use super::super::stat_block::Ability::*;
    match key {
        "acrobatics" | "stealth" | "thievery" => Some(Dexterity),
        "athletics" => Some(Strength),
        "arcana" | "crafting" | "occultism" | "society" => Some(Intelligence),
        "medicine" | "nature" | "religion" | "survival" => Some(Wisdom),
        "deception" | "diplomacy" | "intimidation" | "performance" => Some(Charisma),
        _ => None,
    }
}
fn strike_abilities(
    ranged: Option<bool>,
    traits: Option<&[String]>,
) -> (
    Option<super::super::stat_block::Ability>,
    Option<super::super::stat_block::Ability>,
) {
    use super::super::stat_block::Ability;
    let Some(traits) = traits else {
        return (None, None);
    };
    let has = |needle: &str| traits.iter().any(|trait_| trait_ == needle);
    let attack = ranged.map(|ranged| {
        if (ranged && !has("brutal")) || (!ranged && has("finesse")) {
            Ability::Dexterity
        } else {
            Ability::Strength
        }
    });
    let damage = if ranged == Some(false)
        || has("propulsive")
        || (traits.iter().any(|trait_| trait_.starts_with("thrown-")) && !has("splash"))
    {
        Some(Ability::Strength)
    } else {
        None
    };
    (attack, damage)
}
fn casting_ability(
    item: atlas_record::source_record::ItemSourceView<'_>,
) -> Option<super::super::stat_block::Ability> {
    use super::super::stat_block::Ability;
    use atlas_foundry_model::generated::SpellcastingEntrySystemSourceAbilityValue as SourceAbility;
    let atlas_record::source_record::ItemSourceView::SpellcastingEntrySource(source) = item else {
        return None;
    };
    SourceFieldView::from(&source.system)
        .and_then(|s| (&s.ability).into())
        .and_then(|s| (&s.value).into())
        .value()
        .map(|ability| match ability {
            SourceAbility::Str => Ability::Strength,
            SourceAbility::Dex => Ability::Dexterity,
            SourceAbility::Con => Ability::Constitution,
            SourceAbility::Int => Ability::Intelligence,
            SourceAbility::Wis => Ability::Wisdom,
            SourceAbility::Cha => Ability::Charisma,
            SourceAbility::Empty => Ability::Charisma,
        })
}
fn spell_usage(
    item: atlas_record::source_record::ItemSourceView<'_>,
) -> Option<super::super::stat_block::SpellUsage> {
    use super::super::stat_block::SpellUsage;
    let atlas_record::source_record::ItemSourceView::SpellSource(source) = item else {
        return None;
    };
    let system = SourceFieldView::from(&source.system);
    let traits = system
        .and_then(|s| (&s.traits).into())
        .and_then(|t| (&t.value).into())
        .value()?;
    if traits.iter().any(|trait_| trait_ == "cantrip") {
        return Some(SpellUsage::AtWill);
    }
    match system.and_then(|s| (&s.ritual).into()) {
        SourceFieldView::Value(_) if !traits.iter().any(|trait_| trait_ == "focus") => {
            Some(SpellUsage::AtWill)
        }
        SourceFieldView::Value(_) | SourceFieldView::Null | SourceFieldView::Missing => {
            Some(SpellUsage::Limited)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn infrared_is_visual_but_echolocation_and_motion_sense_are_not() {
        assert!(is_visual(SenseType::InfraredVision));
        assert!(!is_visual(SenseType::Echolocation));
        assert!(!is_visual(SenseType::MotionSense));
    }
    #[test]
    fn source_acuity_defaults_and_mandatory_acuity_preserve_unknowns() {
        assert_eq!(
            effective_precision(SenseType::Darkvision, SourceFieldView::Missing),
            Some(Precision::Precise)
        );
        assert_eq!(
            effective_precision(SenseType::MotionSense, SourceFieldView::Missing),
            Some(Precision::Precise)
        );
        assert_eq!(
            effective_precision(
                SenseType::Echolocation,
                SourceFieldView::Value(&Acuity::Imprecise)
            ),
            Some(Precision::Precise)
        );
        assert_eq!(
            effective_precision(
                SenseType::MotionSense,
                SourceFieldView::Value(&Acuity::Imprecise)
            ),
            Some(Precision::Imprecise)
        );
        assert_eq!(
            effective_precision(SenseType::Darkvision, SourceFieldView::Null),
            None
        );
    }
}

/// Eligibility consumes authored facts, independently of record rendering.
pub(in crate::encounters) fn source_applicability(
    record: &SourceBackedRecord,
    hazard: bool,
) -> super::ConditionApplicabilityFacts {
    use atlas_record::source_record::ActorIwrEntry;
    let actor = SourceQueryView::new(record.source(), record.key().pack().as_str(), "").actor();
    let effective = super::effective_immunities::effective_immunities(record);
    let immunities = super::ConditionImmunityFacts {
        entries: effective
            .entries
            .into_iter()
            .map(|entry| {
                let coverage = match entry.authored {
                    Some(ActorIwrEntry::Immunity(source)) => {
                        immunity_coverage(&source.exceptions, true)
                    }
                    Some(ActorIwrEntry::SchemaImmunity(source)) => {
                        immunity_coverage(&source.exceptions, false)
                    }
                    Some(_) => super::ImmunityCoverage::Unknown,
                    None => super::ImmunityCoverage::Unconditional,
                };
                super::ConditionImmunity {
                    slug: entry.kind,
                    coverage,
                }
            })
            .collect(),
        authored_complete: effective.authored_complete,
        uncertain_types: effective.uncertain_types,
    };
    // Pinned hazard/document.ts derives BT=floor(max/2) and hasHealth=max>0.
    let maximum = hazard
        .then(|| {
            actor
                .hp_maximum()
                .value()
                .and_then(serde_json::Number::as_f64)
        })
        .flatten();
    super::ConditionApplicabilityFacts {
        immunities,
        broken_threshold_known_absent: maximum.is_some_and(|max| max <= 0.0),
        broken_threshold: maximum
            .filter(|max| *max > 0.0 && *max <= 9_007_199_254_740_991.0)
            .map(|max| serde_json::Number::from((max / 2.0).floor() as i64)),
    }
}

fn immunity_coverage<T>(
    source: &atlas_foundry_model::SourcePresence<Vec<T>>,
    optional: bool,
) -> super::ImmunityCoverage {
    use super::ImmunityCoverage;
    use atlas_foundry_model::SourcePresence;
    match source {
        // Pinned actor/data/iwr.ts defaults omitted optional exceptions with `?? []`.
        SourcePresence::Missing if optional => ImmunityCoverage::Unconditional,
        SourcePresence::Value(values) if values.is_empty() => ImmunityCoverage::Unconditional,
        SourcePresence::Value(_) => ImmunityCoverage::Conditional,
        _ => ImmunityCoverage::Unknown,
    }
}

#[cfg(test)]
mod immunity_tests {
    use super::super::ImmunityCoverage::*;
    use super::*;
    use atlas_foundry_model::SourcePresence;
    #[test]
    fn source_defaults_do_not_turn_unknown_exceptions_into_unconditional_immunity() {
        let missing: SourcePresence<Vec<String>> = SourcePresence::Missing;
        assert_eq!(immunity_coverage(&missing, true), Unconditional);
        assert_eq!(immunity_coverage(&missing, false), Unknown);
        assert_eq!(
            immunity_coverage::<String>(&SourcePresence::Null, true),
            Unknown
        );
        assert_eq!(
            immunity_coverage(&SourcePresence::Value(Vec::<String>::new()), false),
            Unconditional
        );
        assert_eq!(
            immunity_coverage(&SourcePresence::Value(vec!["critical-hits"]), true),
            Conditional
        );
    }
}

#[cfg(test)]
mod strike_default_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn omitted_melee_range_uses_the_source_default_but_invalid_range_stays_unknown() {
        for (range, expected) in [
            (None, Some(false)),
            (Some(json!(null)), Some(false)),
            (Some(json!({"increment":30,"max":null})), Some(true)),
            (Some(json!("invalid")), None),
        ] {
            let mut item = json!({"_id":"bbbbbbbbbbbbbbbb","type":"melee","name":"Claw","system":{"bonus":{"value":12},"traits":{"value":[]},"damageRolls":{"main":{"damage":"1d6+4","damageType":"slashing"}}}});
            if let Some(range) = range {
                item["system"]["range"] = range;
            }
            let source = atlas_search::test_support::record("actors","Actor",json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Range actor","system":{"details":{"level":{"value":4}},"attributes":{"adjustment":null}},"items":[item]})).record;
            let baseline = authored_stat_baseline(&source, "fixture", None).unwrap();
            let strike = &baseline.activities[0];
            assert_eq!(strike.is_ranged, expected);
            assert_eq!(
                strike.attack_ability,
                expected.map(|ranged| if ranged {
                    super::super::super::stat_block::Ability::Dexterity
                } else {
                    super::super::super::stat_block::Ability::Strength
                })
            );
        }
    }
}
