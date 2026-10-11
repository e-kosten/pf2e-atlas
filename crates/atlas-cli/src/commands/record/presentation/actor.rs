use super::{
    content_field, damage, fact, fact_string, known_value, line, list_text, number, number_text,
    optional_number, optional_text, unavailable,
};
use crate::commands::record::args::TerminalDetail;
use atlas_app_model::{
    ActorActivityKindView, ActorActivityView, ActorPresentationView, FactView, IwrEntryView,
    OwnedReadingEntryView,
};
use atlas_domain::QueryFieldState;

pub(super) fn render(
    out: &mut String,
    actor: &ActorPresentationView,
    primary_defenses: bool,
    saves: bool,
    reading: &[OwnedReadingEntryView],
    width: usize,
    detail: TerminalDetail,
) -> Result<(), String> {
    number(out, "Perception", &actor.perception, true, "");
    optional_text(out, "Perception details", &actor.perception_details);
    fact(out, "Senses", &actor.senses, |senses| {
        if senses.is_empty() {
            return String::new();
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
    if let Some(abilities) = collection(out, "Ability modifiers", &actor.abilities) {
        line(
            out,
            abilities
                .iter()
                .map(|v| format!("{} {}", v.label, number_text(&v.modifier, true, "")))
                .collect::<Vec<_>>()
                .join("; "),
        );
    }
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
    line(out, "");
    if primary_defenses {
        number(out, "AC", &actor.armor_class, false, "");
        number(out, "HP", &actor.maximum_hp, false, "");
    }
    if saves {
        number(out, "Fortitude", &actor.saves.fortitude, true, "");
        number(out, "Reflex", &actor.saves.reflex, true, "");
        number(out, "Will", &actor.saves.will, true, "");
    }
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
        (ActorActivityKindView::Lore, "Lore"),
        (ActorActivityKindView::Gear, "Gear"),
        (ActorActivityKindView::Other, "Other owned entries"),
    ] {
        if detail == TerminalDetail::Full && kind != ActorActivityKindView::Ability {
            continue;
        }
        if kind == ActorActivityKindView::CastingEntry {
            spellcasting(out, activities);
            continue;
        }
        let group = activities
            .iter()
            .filter(|v| v.kind == kind)
            .collect::<Vec<_>>();
        if group.is_empty() {
            continue;
        }
        line(out, format!("\n{heading}"));
        for activity in group {
            line(out, "");
            activity_facts(out, activity, true);
            if let Some(entry) = reading
                .iter()
                .find(|entry| entry.navigation == activity.navigation)
            {
                for field in &entry.content {
                    content_field(out, field, width, field.role != "description")?;
                }
            }
        }
    }
    Ok(())
}

pub(super) fn activity_facts(out: &mut String, activity: &ActorActivityView, show_identity: bool) {
    if show_identity {
        line(
            out,
            format!(
                "{}{}",
                activity.title,
                activity
                    .action_cost
                    .value
                    .as_ref()
                    .map(|v| format!(" — {}", v.label))
                    .or_else(|| activity.usage.as_ref().map(|v| format!(" — {v}")))
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
    if activity.action_cost.state != atlas_domain::QueryFieldState::NotApplicable
        && (!show_identity || activity.action_cost.state != QueryFieldState::Value)
    {
        fact(out, "Action cost", &activity.action_cost, |v| {
            v.label.clone()
        });
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
    optional_number(out, "  Rank", &activity.base_rank, false, "");
    optional_number(out, "  Cast rank", &activity.authored_cast_rank, false, "");
    if let Some(context) = &activity.casting_context {
        line(out, format!("Spellcasting for {}", context.parent_title));
        if let Some(spells) = &context.spells.value {
            spell_roster(out, spells.iter());
        } else {
            fact(out, "Associated spells", &context.spells, |_| String::new());
        }
    }
    for note in &activity.notes {
        line(out, format!("  {}: {}", note.label, note.reason));
    }
}

fn spellcasting(out: &mut String, activities: &[ActorActivityView]) {
    let entries = activities
        .iter()
        .filter(|a| a.kind == ActorActivityKindView::CastingEntry)
        .collect::<Vec<_>>();
    if !entries.is_empty() {
        line(out, "\nSpellcasting");
    }
    for entry in &entries {
        line(out, "");
        activity_facts(out, entry, true);
        spell_roster(
            out,
            activities.iter().filter(|spell| {
                spell.kind == ActorActivityKindView::Spell
                    && spell.casting_entry.as_ref() == Some(&entry.navigation)
            }),
        );
    }
    let unassociated = activities
        .iter()
        .filter(|spell| {
            spell.kind == ActorActivityKindView::Spell
                && !entries
                    .iter()
                    .any(|entry| spell.casting_entry.as_ref() == Some(&entry.navigation))
        })
        .collect::<Vec<_>>();
    if !unassociated.is_empty() {
        line(out, "\nSpells");
        spell_roster(out, unassociated.into_iter());
    }
}

fn spell_roster<'a>(out: &mut String, spells: impl Iterator<Item = &'a ActorActivityView>) {
    let mut ranks = std::collections::BTreeMap::<(i64, String), Vec<&str>>::new();
    for spell in spells {
        let rank = if spell.traits.iter().any(|t| t.value == "cantrip") {
            (0, "Cantrips".into())
        } else {
            let rank = if matches!(
                spell.authored_cast_rank.state,
                QueryFieldState::Missing | QueryFieldState::NotApplicable
            ) {
                &spell.base_rank
            } else {
                &spell.authored_cast_rank
            };
            (
                rank.value
                    .as_ref()
                    .and_then(serde_json::Number::as_i64)
                    .unwrap_or(i64::MAX),
                format!("Rank {}", number_text(rank, false, "")),
            )
        };
        ranks.entry(rank).or_default().push(&spell.title);
    }
    for ((_, rank), titles) in ranks {
        line(out, format!("  {rank}: {}", titles.join(", ")));
    }
}

fn collection<'a, T>(
    out: &mut String,
    label: &str,
    value: &'a FactView<Vec<T>>,
) -> Option<&'a [T]> {
    if let Some(values) = known_value(value) {
        if values.is_empty() {
            return None;
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
            return String::new();
        }
        entries
            .iter()
            .map(|entry| {
                let mut value = fact_string(&entry.damage_type, Clone::clone);
                if entry.magnitude.state != QueryFieldState::NotApplicable {
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
