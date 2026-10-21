use super::*;
use atlas_foundry_model::{
    ActorSourcePF2e,
    generated::{
        ActorAttributesSourceImmunitiesEntryImmunitySourceExceptionsEntryIWRException as ImmunityQualifier,
        ResistanceSourceDoubleVsEntryIWRException as ResistanceQualifier,
        WeaknessSourceExceptionsEntryIWRException as WeaknessQualifier,
    },
};
use atlas_record::source_record::{ActorIwrEntry, ActorIwrKind, ActorIwrType, ActorSave};

#[cfg(test)]
thread_local! {
    static AUTHORED_PROJECTIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(crate) fn authored_projection_count(reset: bool) -> usize {
    AUTHORED_PROJECTIONS.with(|count| if reset { count.replace(0) } else { count.get() })
}

pub(crate) fn authored_actor(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    fingerprint: &str,
) -> ActorPresentationView {
    #[cfg(test)]
    AUTHORED_PROJECTIONS.with(|count| count.set(count.get() + 1));
    let a = SourceQueryView {
        source: node,
        pack_id: record.key().pack().as_str(),
        pack_label: "",
    }
    .actor();
    let mut abilities = vec![];
    a.visit_abilities(|key, v| {
        abilities.push(ActorModifierView {
            key: key.into(),
            label: key.to_uppercase(),
            modifier: number(v),
        })
    });
    let mut skills = vec![];
    a.visit_skills(|key, v| {
        if matches!(v, SourceFieldView::Missing) {
            return;
        }
        skills.push(ActorSkillView {
            key: key.into(),
            label: key.into(),
            modifier: number(v.and_then(|s| (&s.base).into())),
            note: text(v.and_then(|s| (&s.note).into())),
            conditional: fact(v.and_then(|s| (&s.special).into()).map(|entries| {
                entries
                    .iter()
                    .map(|e| ConditionalSkillView {
                        label: text((&e.label).into()),
                        modifier: number((&e.base).into()),
                    })
                    .collect()
            })),
        });
    });
    let movement = a
        .speeds()
        .value()
        .map(|s| {
            s.iter()
                .map(|v| ActorMovementView {
                    movement_type: identifier(v.speed_type()),
                    feet: number(v.value()),
                })
                .collect()
        })
        .unwrap_or_default();
    let (senses, perception_details, languages, language_details) = match node {
        SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
            let sys = SourceFieldView::from(&s.system);
            let p = sys.and_then(|s| (&s.perception).into());
            let l = sys
                .and_then(|s| (&s.details).into())
                .and_then(|d| (&d.languages).into());
            (
                fact(p.and_then(|p| (&p.senses).into()).map(|s| {
                    s.iter()
                        .map(|s| ActorSenseView {
                            sense: identifier((&s.r#type).into()),
                            acuity: identifier((&s.acuity).into()),
                            range_feet: number((&s.range).into()),
                        })
                        .collect()
                })),
                text(p.and_then(|p| (&p.details).into())),
                identifiers(l.and_then(|l| (&l.value).into())),
                text(l.and_then(|l| (&l.details).into())),
            )
        }
        _ => (unavailable(), unavailable(), unavailable(), unavailable()),
    };
    ActorPresentationView {
        size: fact(a.size().map(|size| {
            use atlas_foundry_model::generated::PhysicalSystemSourceSize::*;
            match size {
                Grg => "Gargantuan",
                Huge => "Huge",
                Lg => "Large",
                Med => "Medium",
                Sm => "Small",
                Tiny => "Tiny",
            }
            .to_owned()
        })),
        hardness: number(a.creature_hardness()),
        ritual_difficulty_class: number(a.ritual_difficulty_class()),
        defense_notes: ActorDefenseNotesView {
            armor_class: fact(a.armor_class_details().map(str::to_owned)),
            hit_points: fact(a.hit_point_details().map(str::to_owned)),
            all_saves: fact(a.general_save_details().map(str::to_owned)),
            fortitude: fact(a.save_details(ActorSave::Fortitude).map(str::to_owned)),
            reflex: fact(a.save_details(ActorSave::Reflex).map(str::to_owned)),
            will: fact(a.save_details(ActorSave::Will).map(str::to_owned)),
        },
        level: number(a.level()),
        armor_class: number(a.armor_class()),
        maximum_hp: number(a.hp_maximum()),
        perception: number(a.perception()),
        saves: ActorSavesView {
            fortitude: number(a.save(ActorSave::Fortitude)),
            reflex: number(a.save(ActorSave::Reflex)),
            will: number(a.save(ActorSave::Will)),
        },
        abilities: fact(a.npc_abilities().map(|_| abilities)),
        skills: fact(a.npc_skills().map(|_| skills)),
        movement: fact(a.speeds().map(|_| movement)),
        land_speed: number(a.land_speed()),
        immunities: iwr(a.iwr(ActorIwrKind::Immunity)),
        weaknesses: iwr(a.iwr(ActorIwrKind::Weakness)),
        resistances: iwr(a.iwr(ActorIwrKind::Resistance)),
        senses,
        perception_details,
        languages,
        language_details,
        activities: authored_activities(record, node, fingerprint),
        runtime: None,
    }
}
pub(crate) fn authored_activities(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    fingerprint: &str,
) -> FactView<Vec<ActorActivityView>> {
    let mut casting = std::collections::BTreeMap::<String, Option<RecordNavigationView>>::new();
    record.visit_immediate_actor_items(|_, owner, item| {
        if item.family() == "spellcastingEntry"
            && let Some(id) = SourceNodeView::Item(item).id().value()
        {
            let owners = vec![owner.clone()];
            let n = RecordNavigationView {
                spell_rank: None,
                record_key: record.key().to_string(),
                source_fingerprint: navigation_fingerprint(&owners, fingerprint),
                owners,
                field: None,
                passage: None,
            };
            casting
                .entry(id.clone())
                .and_modify(|v| *v = None)
                .or_insert(Some(n));
        }
    });
    let mut activities = vec![];
    record.visit_immediate_actor_items(|_, o, i| {
        let owners = vec![o.clone()];
        let navigation = RecordNavigationView {
            spell_rank: None,
            record_key: record.key().to_string(),
            source_fingerprint: navigation_fingerprint(&owners, fingerprint),
            owners,
            field: None,
            passage: None,
        };
        let mut activity = authored_activity(i, navigation);
        if activity.preparation.value.as_deref() == Some("focus") {
            activity.focus_capacity = number(
                SourceQueryView {
                    source: node,
                    pack_id: "",
                    pack_label: "",
                }
                .actor()
                .focus_capacity(),
            );
        }
        if let ItemSourceView::SpellSource(s) = i {
            activity.casting_ranks = spell_context::ranks(record, s);
            if activity.casting_ranks.len() == 1 {
                activity.navigation.spell_rank = activity.casting_ranks.first().copied();
            }
            let entry = SourceFieldView::from(&s.system)
                .and_then(|s| (&s.location).into())
                .and_then(|l| (&l.value).into());
            activity.association = text(entry);
            activity.casting_entry = entry
                .value()
                .and_then(|id| casting.get(id))
                .and_then(Clone::clone);
        }
        if let Some(variants) = i.lore_variants().value() {
            for (_, v) in &variants.entries {
                if let Some(label) = SourceFieldView::from(&v.label).value() {
                    activity.notes.push(RuntimeEffectNoteView {
                        source: activity.title.clone(),
                        label: label.clone(),
                        reason: SourceFieldView::from(&v.options)
                            .value()
                            .cloned()
                            .unwrap_or_else(|| "Lore variant context is unavailable.".into()),
                    });
                }
            }
        }
        activities.push(activity);
    });
    let items = SourceQueryView {
        source: node,
        pack_id: "",
        pack_label: "",
    }
    .actor()
    .items();
    fact(items.map(|_| activities))
}

fn iwr(
    v: SourceFieldView<'_, atlas_record::source_record::ActorIwrEntries<'_>>,
) -> FactView<Vec<IwrEntryView>> {
    fact(v.map(|v| {
        v.iter()
            .map(|e| {
                let (exceptions, double_against) = match e {
                    ActorIwrEntry::Immunity(v) => (
                        qualifiers((&v.exceptions).into(), |q| match q {
                            ImmunityQualifier::Alternative107(v) => text((&v.label).into()),
                            _ => identifier(SourceFieldView::Value(q)),
                        }),
                        unavailable(),
                    ),
                    ActorIwrEntry::SchemaImmunity(v) => {
                        (identifiers((&v.exceptions).into()), unavailable())
                    }
                    ActorIwrEntry::Weakness(v) => (
                        qualifiers((&v.exceptions).into(), |q| match q {
                            WeaknessQualifier::Alternative72(v) => text((&v.label).into()),
                            _ => identifier(SourceFieldView::Value(q)),
                        }),
                        unavailable(),
                    ),
                    ActorIwrEntry::SchemaWeakness(v) => {
                        (identifiers((&v.exceptions).into()), unavailable())
                    }
                    ActorIwrEntry::Resistance(v) => (
                        resistance_qualifiers((&v.exceptions).into()),
                        resistance_qualifiers((&v.double_vs).into()),
                    ),
                    ActorIwrEntry::SchemaResistance(v) => (
                        identifiers((&v.exceptions).into()),
                        identifiers((&v.double_vs).into()),
                    ),
                };
                IwrEntryView {
                    damage_type: fact(
                        e.iwr_type()
                            .and_then(|t| match t {
                                ActorIwrType::Immunity(t) => SourceFieldView::Value(t).map(token),
                                ActorIwrType::Weakness(t) => SourceFieldView::Value(t).map(token),
                                ActorIwrType::Resistance(t) => SourceFieldView::Value(t).map(token),
                            })
                            .and_then(|v| {
                                v.map(SourceFieldView::Value).unwrap_or(
                                    SourceFieldView::ProjectionInvalid {
                                        source_path: "$",
                                        reason: "unrepresentable IWR identifier",
                                    },
                                )
                            }),
                    ),
                    magnitude: number(e.value()),
                    exceptions,
                    double_against,
                }
            })
            .collect()
    }))
}
fn resistance_qualifiers(
    v: SourceFieldView<'_, &Vec<ResistanceQualifier>>,
) -> FactView<Vec<String>> {
    qualifiers(v, |q| match q {
        ResistanceQualifier::Alternative69(v) => text((&v.label).into()),
        _ => identifier(SourceFieldView::Value(q)),
    })
}
fn qualifiers<T>(
    v: SourceFieldView<'_, &Vec<T>>,
    label: impl Fn(&T) -> FactView<String>,
) -> FactView<Vec<String>> {
    let state = field_state(v.availability());
    let Some(entries) = v.value() else {
        return FactView { state, value: None };
    };
    let mut labels = Vec::with_capacity(entries.len());
    for e in entries {
        let f = label(e);
        let Some(label) = f.value else {
            return FactView {
                state: f.state,
                value: None,
            };
        };
        labels.push(label);
    }
    FactView {
        state: QueryFieldState::Value,
        value: Some(labels),
    }
}
pub(super) fn authored_activity(
    item: ItemSourceView<'_>,
    navigation: RecordNavigationView,
) -> ActorActivityView {
    let family = item.family();
    let damage = if family == "spell" {
        spell::damage(item.spell_damage())
    } else {
        fact(item.melee_damage().map(|d| {
            d.entries
                .iter()
                .map(|(id, c)| DamageComponentView {
                    id: id.clone(),
                    formula: text((&c.damage).into()),
                    damage_type: identifier((&c.damage_type).into()),
                    kinds: FactView {
                        state: QueryFieldState::Value,
                        value: Some(vec!["damage".into()]),
                    },
                    // PF2e prepares bleed as persistent, including missing source categories.
                    category: if identifier((&c.damage_type).into()).value.as_deref()
                        == Some("bleed")
                    {
                        FactView {
                            state: QueryFieldState::Value,
                            value: Some("persistent".into()),
                        }
                    } else {
                        identifier((&c.category).into())
                    },
                    materials: unavailable(),
                    apply_modifier: unavailable(),
                })
                .collect()
        }))
    };
    let (attack, difficulty_class, casting_tradition, preparation) = match item {
        ItemSourceView::SpellcastingEntrySource(s) => {
            let s = SourceFieldView::from(&s.system);
            (
                number(
                    s.and_then(|s| (&s.spelldc).into())
                        .and_then(|v| (&v.value).into()),
                ),
                number(
                    s.and_then(|s| (&s.spelldc).into())
                        .and_then(|v| (&v.dc).into()),
                ),
                identifier(
                    s.and_then(|s| (&s.tradition).into())
                        .and_then(|v| (&v.value).into()),
                ),
                identifier(
                    s.and_then(|s| (&s.prepared).into())
                        .and_then(|v| (&v.value).into()),
                ),
            )
        }
        _ => (
            number(item.melee_attack()),
            number(SourceFieldView::NotApplicable),
            unavailable(),
            unavailable(),
        ),
    };
    let mut activity = ActorActivityView {
        is_ranged: fact(match item {
            ItemSourceView::MeleeSource(s) => {
                match SourceFieldView::from(&s.system).and_then(|s| (&s.range).into()) {
                    SourceFieldView::Value(_) => SourceFieldView::Value(true),
                    SourceFieldView::Null => SourceFieldView::Value(false),
                    other => other.map(|_| false),
                }
            }
            _ => SourceFieldView::NotApplicable,
        }),
        effect_duration: item_reference::effect_duration(item),
        self_effect: fact(
            match item.self_effect() {
                SourceFieldView::Null => SourceFieldView::NotApplicable,
                other => other,
            }
            .map(|effect| ActivityEffectReferenceView {
                name: text((&effect.name).into()),
                authored_target: text((&effect.uuid).into()),
                navigation: None,
            }),
        ),
        strike_effects: fact(item.strike_effects().map(Clone::clone)),
        focus_capacity: number(SourceFieldView::NotApplicable),
        frequency: activity_frequency(item),
        casting_ranks: vec![],
        has_reference_details: has_reference_details(item),
        title: node_title(SourceNodeView::Item(item)),
        family: family.into(),
        kind: match family {
            "melee" => ActorActivityKindView::Strike,
            "spell" => ActorActivityKindView::Spell,
            "spellcastingEntry" => ActorActivityKindView::CastingEntry,
            "lore" => ActorActivityKindView::Lore,
            "action" | "effect" | "feat" => ActorActivityKindView::Ability,
            "ammo" | "weapon" | "armor" | "shield" | "equipment" | "consumable" | "backpack"
            | "treasure" => ActorActivityKindView::Gear,
            _ => ActorActivityKindView::Other,
        },
        navigation,
        usage: item_usage(item),
        action_cost: item_action_cost(item),
        traits: SourceQueryView {
            source: SourceNodeView::Item(item),
            pack_id: "",
            pack_label: "",
        }
        .traits()
        .value()
        .into_iter()
        .flatten()
        .map(|t| RecordBadgeView {
            kind: "trait".into(),
            trait_key: atlas_domain::TraitKey::new(t.clone())
                .ok()
                .map(|key| key.to_string()),
            label: t.clone(),
            value: t.clone(),
        })
        .collect(),
        attack,
        attack_options: vec![],
        lore_modifier: number(item.lore_modifier()),
        damage,
        difficulty_class,
        casting_tradition,
        preparation,
        casting_entry: None,
        association: unavailable(),
        base_rank: match item {
            ItemSourceView::SpellSource(s) => number(
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.level).into())
                    .and_then(|l| (&l.value).into()),
            ),
            _ => number(SourceFieldView::NotApplicable),
        },
        authored_cast_rank: match item {
            ItemSourceView::SpellSource(s) => number(
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.location).into())
                    .and_then(|l| (&l.heightened_level).into()),
            ),
            _ => number(SourceFieldView::NotApplicable),
        },
        casting_context: None,
        notes: vec![],
    };
    activity.attack_options = if family == "melee" {
        let traits = SourceQueryView {
            source: SourceNodeView::Item(item),
            pack_id: "",
            pack_label: "",
        }
        .traits();
        super::strike::project_attack_options(
            item.melee_attack()
                .value()
                .and_then(serde_json::Number::as_i64),
            traits
                .value()
                .map(|traits| traits.iter().any(|value| value == "agile")),
        )
    } else {
        vec![]
    };
    activity
}
fn has_reference_details(item: ItemSourceView<'_>) -> bool {
    match selected_item_body(item) {
        SelectedItemBody::Spell(_)
        | SelectedItemBody::Physical
        | SelectedItemBody::Activity {
            casting_context: true,
        } => return true,
        SelectedItemBody::Activity {
            casting_context: false,
        }
        | SelectedItemBody::Content => {}
    }
    let fields = SourceNodeView::Item(item).content_selections();
    // This metadata check preserves content-free/lazy projection: it neither
    // prepares HTML nor reads child content. Markup-only strings remain linked.
    let public = fields
        .iter()
        .find(|field| field.field == "/system/description/value");
    if !matches!(public.map(|field| field.text), Some(SourceFieldView::Value(text)) if text.trim().is_empty())
    {
        // Missing, null or invalid content cannot prove a fully presented summary.
        return true;
    }
    fields
        .iter()
        .filter(|field| field.field != "/name" && field.field != "/system/description/value")
        .any(|field| match field.text {
            SourceFieldView::Value(text) => !text.trim().is_empty(),
            SourceFieldView::Missing | SourceFieldView::NotApplicable => false,
            _ => true,
        })
}

pub(crate) fn compose_participant(
    mut presentation: RecordPresentationView,
    block: &crate::encounters::mechanics::StatBlockView,
) -> RecordPresentationView {
    let (a, hazard) = match &mut presentation.body {
        RecordBodyView::Creature(a) => (a.as_mut(), None),
        RecordBodyView::Hazard(h) => (&mut h.actor, Some((&mut h.stealth, &mut h.hardness))),
        _ => return presentation,
    };
    let overlay = |fact: &mut NumberFactView, target: &str| {
        if block
            .unresolved_stats
            .iter()
            .any(|unresolved| unresolved.key() == target)
        {
            fact.value = None;
            fact.adjustment = None;
            fact.state = QueryFieldState::Missing;
            return;
        }
        if let Some(replacement) = block
            .replacements
            .iter()
            .find(|replacement| replacement.target.key() == target)
        {
            if let Some(authored) = fact.value.clone() {
                fact.adjustment = Some(NumberAdjustmentView {
                    authored,
                    applied: vec![],
                    suppressed: vec![],
                    notes: vec![RuntimeEffectNoteView {
                        source: format!("condition:{}", replacement.condition_id),
                        label: replacement.condition_name.clone(),
                        reason: format!("Sets {} to {}.", target, replacement.value),
                    }],
                });
            }
            fact.value = Some(replacement.value.clone());
            fact.state = QueryFieldState::Value;
            return;
        }
        if let Some(v) = block.values.iter().find(|v| v.target.key() == target) {
            fact.value = Some(v.adjusted_value.clone());
            if !v.modifiers.is_empty() || !v.suppressed_modifiers.is_empty() {
                fact.adjustment = Some(NumberAdjustmentView {
                    authored: v.base_value.clone(),
                    applied: modifier_views(&v.modifiers),
                    suppressed: modifier_views(&v.suppressed_modifiers),
                    notes: vec![],
                });
            }
        }
    };
    overlay(&mut a.armor_class, "ac");
    overlay(&mut a.maximum_hp, "hp.max");
    overlay(&mut a.hardness, "hazard.hardness");
    overlay(&mut a.perception, "perception");
    overlay(&mut a.saves.fortitude, "save.fortitude");
    overlay(&mut a.saves.reflex, "save.reflex");
    overlay(&mut a.saves.will, "save.will");
    if let (Some(authored), Some(effective)) = (&block.level, &block.adjusted_level) {
        a.level.value = Some(effective.clone());
        if authored != effective {
            a.level.adjustment = Some(NumberAdjustmentView {
                authored: authored.clone(),
                applied: vec![],
                suppressed: vec![],
                notes: vec![],
            });
        }
        presentation.identity.level_label = Some(format!("Level {effective}"));
        presentation.classification = format!("{} {effective}", presentation.identity.kind_label);
    }
    if let Some((stealth, hardness)) = hazard {
        overlay(stealth, "hazard.stealth");
        overlay(hardness, "hazard.hardness");
    }
    for v in a.abilities.value.iter_mut().flatten() {
        overlay(&mut v.modifier, &format!("ability.{}", v.key));
    }
    for v in a.skills.value.iter_mut().flatten() {
        overlay(&mut v.modifier, &format!("skill.{}", v.key));
    }
    for v in &block.speeds {
        let fact = if v.movement_type == "land" {
            Some(&mut a.land_speed)
        } else {
            a.movement
                .value
                .iter_mut()
                .flatten()
                .find(|s| s.movement_type.value.as_deref() == Some(&v.movement_type))
                .map(|s| &mut s.feet)
        };
        if let Some(f) = fact {
            f.value = Some(v.adjusted_value_feet.clone());
            if v.adjusted_value_feet != v.base_value_feet || !v.notes.is_empty() {
                f.adjustment = Some(NumberAdjustmentView {
                    authored: v.base_value_feet.clone(),
                    applied: vec![],
                    suppressed: vec![],
                    notes: v.notes.clone(),
                });
            }
        }
    }
    for (index, activity) in a.activities.value.iter_mut().flatten().enumerate() {
        overlay(&mut activity.lore_modifier, &format!("skill.lore.{index}"));
        if let Some(v) = block
            .activities
            .iter()
            .find(|v| v.navigation == activity.navigation)
        {
            if v.calculation_unavailable.is_some() {
                activity.attack.value = None;
                activity.attack.state = atlas_domain::QueryFieldState::Missing;
                activity.difficulty_class.value = None;
                activity.difficulty_class.state = atlas_domain::QueryFieldState::Missing;
            }
            if let Some(roll) = v.rolls.first() {
                activity.attack.value = Some(roll.adjusted_value.clone());
                if !roll.modifiers.is_empty() || !roll.suppressed_modifiers.is_empty() {
                    activity.attack.adjustment = Some(NumberAdjustmentView {
                        authored: roll.base_value.clone(),
                        applied: modifier_views(&roll.modifiers),
                        suppressed: modifier_views(&roll.suppressed_modifiers),
                        notes: vec![],
                    });
                }
            }
            if let Some(dc) = &v.difficulty_class {
                activity.difficulty_class.value = Some(dc.adjusted_value.clone());
                if !dc.modifiers.is_empty() || !dc.suppressed_modifiers.is_empty() {
                    activity.difficulty_class.adjustment = Some(NumberAdjustmentView {
                        authored: dc.base_value.clone(),
                        applied: modifier_views(&dc.modifiers),
                        suppressed: modifier_views(&dc.suppressed_modifiers),
                        notes: vec![],
                    });
                }
            }
            for damage in activity.damage.value.iter_mut().flatten() {
                if let Some(effective) = v.damage.iter().find(|d| d.damage_id == damage.id) {
                    if effective.unavailable_reason.is_some() {
                        damage.formula.value = None;
                        damage.formula.state = atlas_domain::QueryFieldState::Missing;
                    } else if let Some(formula) = &effective.adjusted_formula {
                        damage.formula.value = Some(formula.clone());
                    }
                }
            }
            activity.notes.extend(v.notes.clone());
            activity.attack_options =
                if v.kind == crate::encounters::mechanics::MechanicActivityKindView::Strike {
                    super::strike::project_attack_options(
                        v.rolls
                            .first()
                            .and_then(|roll| roll.adjusted_value.as_i64()),
                        v.traits_known
                            .then(|| v.traits.iter().any(|value| value == "agile")),
                    )
                } else {
                    vec![]
                };
        }
    }
    a.runtime = Some(ActorRuntimeView {
        action_budget: block.action_budget.clone(),
        senses: block.senses.clone(),
        contextual_armor_class: block.contextual_armor_class.clone(),
        unapplied_effects: block.unapplied_effects.clone(),
    });
    if let RecordBodyView::Hazard(hazard) = &mut presentation.body
        && !hazard.show_saves
    {
        hazard.show_saves = [
            &hazard.actor.saves.fortitude,
            &hazard.actor.saves.reflex,
            &hazard.actor.saves.will,
        ]
        .into_iter()
        .any(|save| {
            save.state != QueryFieldState::Value
                || save.value.as_ref().and_then(serde_json::Number::as_f64) != Some(0.0)
                || save.adjustment.is_some()
        });
    }
    presentation
}
pub(crate) fn manual_presentation(
    participant: &atlas_local_state::EncounterParticipant,
    block: &crate::encounters::mechanics::StatBlockView,
) -> RecordPresentationView {
    let unknown = || NumberFactView {
        state: QueryFieldState::NotApplicable,
        value: None,
        adjustment: None,
    };
    RecordPresentationView {
        classification: "PC".into(),
        identity: RecordSummaryView {
            record_key: participant.participant_key.clone(),
            title: participant.display_name.clone(),
            kind: "pc".into(),
            kind_label: "PC".into(),
            source_type: None,
            level_label: None,
            level_basis: None,
            rarity: None,
            traits: vec![],
            publication: None,
            pack: None,
        },
        content: vec![],
        owned: vec![],
        body: RecordBodyView::Creature(Box::new(ActorPresentationView {
            size: unavailable(),
            hardness: number(SourceFieldView::NotApplicable),
            ritual_difficulty_class: number(SourceFieldView::NotApplicable),
            defense_notes: ActorDefenseNotesView {
                armor_class: unavailable(),
                hit_points: unavailable(),
                all_saves: unavailable(),
                fortitude: unavailable(),
                reflex: unavailable(),
                will: unavailable(),
            },
            level: unknown(),
            armor_class: unknown(),
            maximum_hp: unknown(),
            perception: unknown(),
            saves: ActorSavesView {
                fortitude: unknown(),
                reflex: unknown(),
                will: unknown(),
            },
            abilities: unavailable(),
            skills: unavailable(),
            movement: unavailable(),
            land_speed: unknown(),
            immunities: unavailable(),
            weaknesses: unavailable(),
            resistances: unavailable(),
            senses: unavailable(),
            perception_details: unavailable(),
            languages: unavailable(),
            language_details: unavailable(),
            activities: unavailable(),
            runtime: Some(ActorRuntimeView {
                action_budget: block.action_budget.clone(),
                senses: block.senses.clone(),
                contextual_armor_class: block.contextual_armor_class.clone(),
                unapplied_effects: block.unapplied_effects.clone(),
            }),
        })),
    }
}

fn activity_frequency(item: ItemSourceView<'_>) -> FactView<ActivityFrequencyView> {
    use atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer;
    let source = match item.activity_frequency() {
        SourceFieldView::Null => SourceFieldView::NotApplicable,
        value => value,
    };
    fact(source.map(|frequency| ActivityFrequencyView {
        maximum: number((&frequency.max).into()),
        interval: fact(SourceFieldView::from(&frequency.per).map(|per| {
            match per {
                AbilitySystemSourceFrequencyPer::P1M => "month",
                AbilitySystemSourceFrequencyPer::P1W => "week",
                AbilitySystemSourceFrequencyPer::P1Y => "year",
                AbilitySystemSourceFrequencyPer::PT10M => "10 minutes",
                AbilitySystemSourceFrequencyPer::PT1H => "hour",
                AbilitySystemSourceFrequencyPer::PT1M => "minute",
                AbilitySystemSourceFrequencyPer::PT24H => "24 hours",
                AbilitySystemSourceFrequencyPer::Day => "day",
                AbilitySystemSourceFrequencyPer::Round => "round",
                AbilitySystemSourceFrequencyPer::Turn => "turn",
            }
            .to_owned()
        })),
    }))
}

fn modifier_views(modifiers: &[crate::encounters::mechanics::Modifier]) -> Vec<StatModifierView> {
    use crate::encounters::mechanics::ModifierType;
    modifiers
        .iter()
        .map(|modifier| StatModifierView {
            source: modifier.source.clone(),
            label: modifier.label.clone(),
            modifier_type: match modifier.modifier_type {
                ModifierType::Status => StatModifierTypeView::Status,
                ModifierType::Circumstance => StatModifierTypeView::Circumstance,
                ModifierType::Adjustment => StatModifierTypeView::Adjustment,
            },
            value: modifier.value,
        })
        .collect()
}
