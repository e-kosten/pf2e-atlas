use super::{
    damage, fact, fact_string, known_value, line, list_text, navigation, number, number_text,
    optional_number, optional_text, unavailable,
};
use atlas_app_model::{
    ActorActivityKindView, ActorActivityView, ActorPresentationView, FactView, IwrEntryView,
};
use atlas_domain::QueryFieldState;

pub(super) fn render(out: &mut String, actor: &ActorPresentationView) -> Result<(), String> {
    number(out, "Perception", &actor.perception, true, "");
    optional_text(out, "Perception details", &actor.perception_details);
    fact(out, "Senses", &actor.senses, |senses| {
        if senses.is_empty() {
            return "none".into();
        }
        senses
            .iter()
            .map(|sense| {
                let mut parts = vec![fact_string(&sense.sense, Clone::clone)];
                if sense.acuity.state != QueryFieldState::Missing {
                    parts.push(fact_string(&sense.acuity, Clone::clone));
                }
                if sense.range_feet.state != QueryFieldState::Missing {
                    parts.push(number_text(&sense.range_feet, false, " feet"));
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join(", ")
    });
    fact(out, "Languages", &actor.languages, |v| list_text(v));
    optional_text(out, "Language details", &actor.language_details);
    if let Some(skills) = collection(out, "Skills", &actor.skills) {
        for skill in skills {
            number(out, &skill.label, &skill.modifier, true, "");
            optional_text(out, "  Context", &skill.note);
            if skill.conditional.state == QueryFieldState::Value {
                if let Some(conditional) = &skill.conditional.value {
                    for value in conditional {
                        line(
                            out,
                            format!(
                                "  {}: {}",
                                fact_string(&value.label, Clone::clone),
                                number_text(&value.modifier, true, "")
                            ),
                        );
                    }
                }
            } else if !matches!(
                skill.conditional.state,
                QueryFieldState::Missing | QueryFieldState::NotApplicable
            ) {
                line(
                    out,
                    format!(
                        "  Conditional modifiers: {}",
                        unavailable(skill.conditional.state)
                    ),
                );
            }
        }
    }
    if let Some(abilities) = collection(out, "Ability modifiers", &actor.abilities)
        && !abilities.is_empty()
    {
        line(
            out,
            abilities
                .iter()
                .map(|v| format!("{} {}", v.label, number_text(&v.modifier, true, "")))
                .collect::<Vec<_>>()
                .join("; "),
        );
    }
    line(out, "");
    number(out, "AC", &actor.armor_class, false, "");
    number(out, "Fortitude", &actor.saves.fortitude, true, "");
    number(out, "Reflex", &actor.saves.reflex, true, "");
    number(out, "Will", &actor.saves.will, true, "");
    number(out, "HP", &actor.maximum_hp, false, "");
    iwr(out, "Immunities", &actor.immunities);
    iwr(out, "Weaknesses", &actor.weaknesses);
    iwr(out, "Resistances", &actor.resistances);
    line(out, "");
    optional_number(out, "Speed", &actor.land_speed, false, " feet");
    if let Some(movements) = collection(out, "Other speeds", &actor.movement) {
        for movement in movements {
            line(
                out,
                format!(
                    "Speed ({}): {}",
                    fact_string(&movement.movement_type, Clone::clone),
                    number_text(&movement.feet, false, " feet")
                ),
            );
        }
    }
    let Some(activities) = collection(out, "Owned entries", &actor.activities) else {
        return Ok(());
    };
    for (kind, heading) in [
        (ActorActivityKindView::Strike, "Strikes"),
        (ActorActivityKindView::Ability, "Abilities"),
        (ActorActivityKindView::CastingEntry, "Spellcasting"),
        (ActorActivityKindView::Spell, "Spells"),
        (ActorActivityKindView::Lore, "Lore"),
        (ActorActivityKindView::Gear, "Gear"),
        (ActorActivityKindView::Other, "Other owned entries"),
    ] {
        let activities = activities
            .iter()
            .filter(|v| v.kind == kind)
            .collect::<Vec<_>>();
        if activities.is_empty() {
            continue;
        }
        line(out, format!("\n{heading}"));
        for activity in activities {
            activity_facts(out, activity, true)?;
            navigation(out, &activity.navigation)?;
        }
    }
    Ok(())
}

pub(super) fn activity_facts(
    out: &mut String,
    activity: &ActorActivityView,
    show_identity: bool,
) -> Result<(), String> {
    if show_identity {
        line(
            out,
            format!(
                "{}{}",
                activity.title,
                activity
                    .usage
                    .as_ref()
                    .map(|v| format!(" — {v}"))
                    .unwrap_or_default()
            ),
        );
        if !activity.traits.is_empty() {
            line(
                out,
                format!(
                    "  Traits: {}",
                    activity
                        .traits
                        .iter()
                        .map(|v| v.label.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
    } else if let Some(usage) = &activity.usage {
        line(out, format!("Usage: {usage}"));
    }
    let attack_label = if activity.kind == ActorActivityKindView::CastingEntry {
        "  Spell attack"
    } else {
        "  Attack"
    };
    optional_number(out, attack_label, &activity.attack, true, "");
    optional_number(out, "  DC", &activity.difficulty_class, false, "");
    optional_number(out, "  Lore modifier", &activity.lore_modifier, true, "");
    optional_text(out, "  Tradition", &activity.casting_tradition);
    optional_text(out, "  Preparation", &activity.preparation);
    if activity.kind == ActorActivityKindView::Strike
        || !matches!(
            activity.damage.state,
            QueryFieldState::Missing | QueryFieldState::NotApplicable
        )
    {
        damage(out, &activity.damage);
    }
    optional_text(out, "  Association", &activity.association);
    if let Some(entry) = &activity.casting_entry {
        line(out, "  Casting entry:");
        navigation(out, entry)?;
    }
    for note in &activity.notes {
        line(out, format!("  {}: {}", note.label, note.reason));
    }
    Ok(())
}

fn collection<'a, T>(
    out: &mut String,
    label: &str,
    value: &'a FactView<Vec<T>>,
) -> Option<&'a [T]> {
    if let Some(values) = known_value(value) {
        if values.is_empty() {
            line(out, format!("{label}: none"));
        }
        Some(values)
    } else {
        if value.state != QueryFieldState::NotApplicable {
            line(out, format!("{label}: {}", unavailable(value.state)));
        }
        None
    }
}

fn iwr(out: &mut String, label: &str, value: &FactView<Vec<IwrEntryView>>) {
    fact(out, label, value, |entries| {
        if entries.is_empty() {
            return "none".into();
        }
        entries
            .iter()
            .map(|entry| {
                let mut value = fact_string(&entry.damage_type, Clone::clone);
                if !matches!(
                    entry.magnitude.state,
                    QueryFieldState::Missing | QueryFieldState::NotApplicable
                ) {
                    value.push_str(&format!(" {}", number_text(&entry.magnitude, false, "")));
                }
                for (label, qualifiers) in [
                    ("except", &entry.exceptions),
                    ("double against", &entry.double_against),
                ] {
                    if qualifiers.state == QueryFieldState::Value {
                        if let Some(values) = &qualifiers.value
                            && !values.is_empty()
                        {
                            value.push_str(&format!(" ({label} {})", values.join(", ")));
                        }
                    } else if !matches!(
                        qualifiers.state,
                        QueryFieldState::Missing | QueryFieldState::NotApplicable
                    ) {
                        value.push_str(&format!(" ({label} {})", unavailable(qualifiers.state)));
                    }
                }
                value
            })
            .collect::<Vec<_>>()
            .join("; ")
    });
}
