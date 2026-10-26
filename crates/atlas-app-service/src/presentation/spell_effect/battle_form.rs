use super::*;
use atlas_foundry_model::SourceMap;
type Overrides = SourceFromSchemaBattleFormRuleOverrideSchema;

fn merge<T: Clone>(
    base: &mut SourcePresence<T>,
    patch: &SourcePresence<T>,
    nested: impl FnOnce(&mut T, &T),
) {
    match (base, patch) {
        (_, SourcePresence::Missing) => {}
        (SourcePresence::Value(base), SourcePresence::Value(patch)) => nested(base, patch),
        (base, patch) => *base = patch.clone(),
    }
}
fn replace<T: Clone>(base: &mut SourcePresence<T>, patch: &SourcePresence<T>) {
    if !matches!(patch, SourcePresence::Missing) {
        *base = patch.clone();
    }
}
macro_rules! fields {
    ($base:expr, $patch:expr, $($field:ident),+ $(,)?) => { $(replace(&mut $base.$field, &$patch.$field);)+ };
}
fn merge_strike(base: &mut BattleFormStrike, patch: &BattleFormStrike) {
    fields!(
        base,
        patch,
        ability,
        base_type,
        category,
        group,
        hands_held,
        img,
        label,
        modifier,
        own_if_higher,
        predicate,
        range,
        trait_toggles,
        traits
    );
    merge(&mut base.damage, &patch.damage, |base, patch| {
        fields!(base, patch, damage_type, dice, die, modifier, persistent);
    });
}
fn compose(base: &mut Overrides, patch: &Overrides) {
    fields!(
        base,
        patch,
        immunities,
        resistances,
        size,
        temp_hp,
        traits,
        weaknesses
    );
    merge(&mut base.speeds, &patch.speeds, |base, patch| {
        fields!(base, patch, land, burrow, climb, fly, swim);
    });
    merge(&mut base.senses, &patch.senses, |base, patch| {
        macro_rules! senses { ($($key:ident),+) => { $(merge(&mut base.$key, &patch.$key, |base, patch| { fields!(base, patch, acuity, range); });)+ }; }
        senses!(
            bloodsense,
            darkvision,
            echolocation,
            electromagnetic_sense,
            greater_darkvision,
            infrared_vision,
            lifesense,
            low_light_vision,
            magicsense,
            motion_sense,
            scent,
            see_invisibility,
            spiritsense,
            thoughtsense,
            tremorsense,
            truesight,
            wavesense
        );
    });
    merge(&mut base.armor_class, &patch.armor_class, |base, patch| {
        fields!(
            base,
            patch,
            ignore_check_penalty,
            ignore_speed_penalty,
            modifier,
            own_if_higher
        );
    });
    merge(&mut base.skills, &patch.skills, |base, patch| {
        macro_rules! skills { ($($key:ident),+) => { $(merge(&mut base.$key, &patch.$key, |base, patch| { fields!(base, patch, modifier, own_if_higher); });)+ }; }
        skills!(
            acrobatics,
            arcana,
            athletics,
            computers,
            crafting,
            deception,
            diplomacy,
            intimidation,
            medicine,
            nature,
            occultism,
            performance,
            piloting,
            religion,
            society,
            stealth,
            survival,
            thievery
        );
    });
    merge(
        &mut base.strikes,
        &patch.strikes,
        |base: &mut SourceMap<BattleFormStrike>, patch| {
            for (key, strike) in &patch.entries {
                if let Some((_, base)) = base.entries.iter_mut().find(|(k, _)| k == key) {
                    merge_strike(base, strike);
                } else {
                    base.entries.push((key.clone(), strike.clone()));
                }
            }
        },
    );
}
fn at_rank(rule: &BattleFormRule, rank: u8) -> Result<Overrides, String> {
    let mut overrides = rule
        .overrides
        .as_value()
        .cloned()
        .ok_or("The form's statistics are unavailable.")?;
    match &rule.brackets {
        SourcePresence::Missing => return Ok(overrides),
        SourcePresence::Value(brackets) => {
            let mut selected = None;
            for bracket in brackets {
                let start = bracket
                    .start
                    .as_value()
                    .and_then(|n| n.as_u64())
                    .ok_or("A form rank is unavailable.")?;
                if start <= u64::from(rank) {
                    selected = Some(bracket);
                }
            }
            if let Some(bracket) = selected {
                let patch = bracket
                    .interpret_overrides(SourceContext::new(
                        "effect",
                        "checked source",
                        "$.system.rules.brackets.value",
                    ))
                    .map_err(|_| "The form's rank statistics are unavailable.")?;
                compose(&mut overrides, &patch);
            }
        }
        _ => return Err("The form's rank statistics are unavailable.".into()),
    }
    Ok(overrides)
}
fn title(key: &str) -> String {
    let text = key.replace('-', " ");
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_uppercase().collect::<String>() + chars.as_str()
    })
}
// Exact English labels at model pin 28d9d156, static/lang/{en,re-en}.json.
// Other authored labels remain literal; an unknown localization key uses the
// authored strike key rather than leaking an internal catalog address.
fn strike_label(label: &str, key: &str) -> String {
    match label {
        "PF2E.BattleForm.Attack.FlamingSword" => "Flaming Sword".into(),
        "PF2E.BattleForm.Attack.PiercingHymn" => "Piercing Hymn".into(),
        "PF2E.Weapon.Base.bastard-sword" => "Bastard Sword".into(),
        "PF2E.Weapon.Base.composite-longbow" => "Composite Longbow".into(),
        "PF2E.Weapon.Base.composite-shortbow" => "Composite Shortbow".into(),
        "PF2E.Weapon.Base.fist" => "Fist".into(),
        "PF2E.Weapon.Base.glaive" => "Glaive".into(),
        "PF2E.Weapon.Base.longspear" => "Longspear".into(),
        "PF2E.Weapon.Base.longsword" => "Longsword".into(),
        "PF2E.Weapon.Base.morningstar" => "Morningstar".into(),
        "PF2E.Weapon.Base.scimitar" => "Scimitar".into(),
        "PF2E.Weapon.Base.scythe" => "Scythe".into(),
        "PF2E.Weapon.Base.trident" => "Trident".into(),
        "PF2E.Weapon.Base.whip" => "Whip".into(),
        label if label.starts_with("PF2E.") => title(key),
        label => label.into(),
    }
}
fn size(size: &SourcePresence<String>) -> FactView<String> {
    fact(SourceFieldView::from(size).map(|v| match v.as_str() {
        "med" | "medium" => "Medium".into(),
        "sm" | "small" => "Small".into(),
        "lg" | "large" => "Large".into(),
        "huge" => "Huge".into(),
        "tiny" => "Tiny".into(),
        "grg" | "gargantuan" => "Gargantuan".into(),
        _ => v.clone(),
    }))
}
fn add(
    facts: &mut Vec<SpellEffectFactView>,
    label: &str,
    current: FactView<String>,
    base: FactView<String>,
    qualifier: Option<String>,
) {
    if !matches!(
        current.state,
        QueryFieldState::Missing | QueryFieldState::NotApplicable
    ) {
        facts.push(rank_fact(label, current, Some(&base), qualifier));
    }
}
fn skills(
    out: &mut Vec<SpellEffectFactView>,
    current: &BattleFormSkills,
    base: Option<&BattleFormSkills>,
) {
    macro_rules! each { ($($key:ident),+) => { $(
        if let Some(skill) = current.$key.as_value() {
            let original = base.and_then(|b| b.$key.as_value()).map(|s| modifier(&s.modifier)).unwrap_or_else(unavailable);
            add(out, &title(stringify!($key)), modifier(&skill.modifier), original,
                (skill.own_if_higher.as_value() != Some(&false)).then(|| "Use your own modifier if higher.".into()));
        } else if matches!(current.$key, SourcePresence::Invalid(_) | SourcePresence::Null) {
            add(out, &title(stringify!($key)), fact(SourceFieldView::from(&current.$key).map(|_| String::new())), unavailable(), None);
        }
    )+ }; }
    each!(
        acrobatics,
        arcana,
        athletics,
        computers,
        crafting,
        deception,
        diplomacy,
        intimidation,
        medicine,
        nature,
        occultism,
        performance,
        piloting,
        religion,
        society,
        stealth,
        survival,
        thievery
    );
}
fn senses(
    source: &SourcePresence<SourceFromSchemaBattleFormRuleOverrideSchemaSensesPartial>,
) -> FactView<String> {
    fact(SourceFieldView::from(source).map(|senses| {
        let mut parts = vec![];
        macro_rules! each { ($($key:ident),+) => { $(if let Some(sense) = senses.$key.as_value() {
            let mut text = title(stringify!($key).replace('_', "-").as_str());
            if let Some(acuity) = sense.acuity.as_value().and_then(token) { text.push_str(&format!(" ({acuity})")); }
            if let Some(range) = sense.range.as_value() { text.push_str(&format!(" {range} feet")); }
            parts.push(text);
        })+ }; }
        each!(bloodsense, darkvision, echolocation, electromagnetic_sense, greater_darkvision, infrared_vision, lifesense, low_light_vision, magicsense, motion_sense, scent, see_invisibility, spiritsense, thoughtsense, tremorsense, truesight, wavesense);
        parts.join(", ")
    }))
}
fn speeds(source: &SourcePresence<BattleFormSpeeds>) -> FactView<String> {
    fact(SourceFieldView::from(source).map(|speeds| {
        let mut parts = vec![];
        macro_rules! each { ($($key:ident),+) => { $(if let Some(speed) = speeds.$key.as_value() { parts.push(format!("{} {speed} feet", title(stringify!($key)))); })+ }; }
        each!(land, burrow, climb, fly, swim); parts.join(", ")
    }))
}
fn damage(source: &SourcePresence<WeaponDamage>) -> FactView<String> {
    fact(SourceFieldView::from(source).and_then(|d| {
        SourceFieldView::from(&d.dice).and_then(|dice| {
            SourceFieldView::from(&d.die).and_then(|die| {
                let Some(die) = token(die) else {
                    return SourceFieldView::ProjectionInvalid {
                        source_path: "system.rules.overrides.strikes.damage.die",
                        reason: "Damage die unavailable",
                    };
                };
                SourceFieldView::from(&d.modifier).and_then(|modifier| {
                    SourceFieldView::from(&d.damage_type).map(|kind| {
                        let kind = token(kind).unwrap_or_default();
                        format!(
                            "{dice}{die}{}{modifier} {kind}",
                            if modifier.as_f64().is_some_and(|v| v >= 0.0) {
                                "+"
                            } else {
                                ""
                            }
                        )
                    })
                })
            })
        })
    }))
}

fn attack_qualifier(strike: &BattleFormStrike) -> Option<String> {
    let mut notes = Vec::new();
    if strike.own_if_higher.as_value() != Some(&false) {
        notes.push("Use your own unarmed attack modifier if higher.".into());
    }
    match &strike.range {
        SourcePresence::Value(range) => {
            if let Some(increment) = range.increment.as_value() {
                notes.push(format!("Range increment {increment} feet."));
            }
            if let Some(max) = range.max.as_value() {
                notes.push(format!("Maximum range {max} feet."));
            }
            if matches!(range.increment, SourcePresence::Invalid(_))
                || matches!(range.max, SourcePresence::Invalid(_))
                || !range.additional_fields.fields().is_empty()
            {
                notes.push("Additional range details are listed in Rules.".into());
            }
        }
        SourcePresence::Invalid(_) => notes.push("Range unavailable; see Rules.".into()),
        // Missing or null range denotes no authored ranged-strike statistic.
        SourcePresence::Missing | SourcePresence::Null => {}
    }
    if let Some(ability) = strike.ability.as_value().and_then(token) {
        notes.push(format!("{} based.", title(&ability)));
    }
    (!notes.is_empty()).then(|| notes.join(" "))
}

pub(super) fn project(
    rule: &BattleFormRule,
    rank: u8,
    base_rank: u8,
    preview: &mut SpellEffectPreviewView,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> Result<(), String> {
    if rule.ignored.as_value() == Some(&true)
        || (!matches!(rule.predicate, SourcePresence::Missing)
            && !rule.predicate.as_value().is_some_and(Vec::is_empty))
    {
        return Err("This form depends on additional conditions; read its rules.".into());
    }
    let current = at_rank(rule, rank)?;
    if rule.own_unarmed.as_value() == Some(&true)
        || current.strikes.as_value().is_some_and(|strikes| {
            strikes.entries.iter().any(|(_, strike)| {
                !matches!(strike.predicate, SourcePresence::Missing)
                    && !strike.predicate.as_value().is_some_and(Vec::is_empty)
            })
        })
    {
        return Err(
            "This form's attacks depend on your character or a choice; read its rules.".into(),
        );
    }
    let base = at_rank(rule, base_rank)?;
    add(
        &mut preview.facts,
        "Size",
        size(&current.size),
        size(&base.size),
        None,
    );
    add(
        &mut preview.facts,
        "Temporary HP",
        scalar(&current.temp_hp),
        scalar(&base.temp_hp),
        None,
    );
    let ac = |v: &Overrides| {
        fact(
            SourceFieldView::from(&v.armor_class)
                .and_then(|a| (&a.modifier).into())
                .and_then(scalar_text),
        )
    };
    add(
        &mut preview.facts,
        "AC",
        ac(&current),
        ac(&base),
        current.armor_class.as_value().and_then(|ac| {
            let mut notes = vec![];
            if ac.ignore_check_penalty.as_value() != Some(&false) {
                notes.push("Ignore armor check penalty");
            }
            if ac.ignore_speed_penalty.as_value() != Some(&false) {
                notes.push("Ignore armor Speed reduction");
            }
            if ac.own_if_higher.as_value() == Some(&true) {
                notes.push("Use your own AC if higher");
            }
            (!notes.is_empty()).then(|| notes.join("; "))
        }),
    );
    if let Some(current_skills) = current.skills.as_value() {
        skills(&mut preview.facts, current_skills, base.skills.as_value());
    }
    add(
        &mut preview.facts,
        "Senses",
        senses(&current.senses),
        senses(&base.senses),
        current
            .senses
            .as_value()
            .filter(|s| !s.additional_fields.fields().is_empty())
            .map(|_| "Additional senses are listed in Rules.".into()),
    );
    add(
        &mut preview.facts,
        "Speed",
        speeds(&current.speeds),
        speeds(&base.speeds),
        None,
    );
    add(
        &mut preview.facts,
        "Traits",
        identifiers(SourceFieldView::from(&current.traits))
            .value
            .map(|v| value(v.join(", ")))
            .unwrap_or_else(unavailable),
        identifiers(SourceFieldView::from(&base.traits))
            .value
            .map(|v| value(v.join(", ")))
            .unwrap_or_else(unavailable),
        None,
    );
    if let Some(strikes) = current.strikes.as_value() {
        for (key, strike) in &strikes.entries {
            let original = base
                .strikes
                .as_value()
                .and_then(|s| s.entries.iter().find(|(k, _)| k == key))
                .map(|(_, s)| s);
            let old_attack = original.map(|s| modifier(&s.modifier));
            let old_damage = original.map(|s| damage(&s.damage));
            let traits = strike
                .traits
                .as_value()
                .into_iter()
                .flatten()
                .map(|trait_| RecordBadgeView {
                    trait_key: atlas_domain::TraitKey::new(trait_.clone())
                        .ok()
                        .map(|k| k.to_string()),
                    kind: "trait".into(),
                    value: trait_.clone(),
                    label: retrieval.trait_label(trait_).unwrap_or(trait_).into(),
                })
                .collect();
            preview.strikes.push(SpellEffectStrikeView {
                id: key.clone(),
                title: strike
                    .label
                    .as_value()
                    .map(|label| strike_label(label, key))
                    .unwrap_or_else(|| title(key)),
                traits,
                attack: rank_fact(
                    "Attack",
                    modifier(&strike.modifier),
                    old_attack.as_ref(),
                    attack_qualifier(strike),
                ),
                damage: rank_fact("Damage", damage(&strike.damage), old_damage.as_ref(), None),
            });
        }
    }
    Ok(())
}
