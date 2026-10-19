use super::{
    damage, damage_changes, fact, fact_string, known_value, line, list_text, number_text,
    optional_number, optional_text, text, unavailable,
};
use atlas_app_model::{
    FactView, SpellAreaView, SpellChangeView, SpellDefenseView, SpellHeighteningChangeView,
    SpellHeighteningView, SpellPresentationView,
};
use atlas_domain::QueryFieldState;

pub(super) fn render(out: &mut String, spell: &SpellPresentationView) {
    if let Some(note) = &spell.rank_note {
        line(out, note);
    }
    fact(out, "Traditions", &spell.traditions, |v| list_text(v));
    fact(out, "Casting", &spell.cast, |v| v.label.clone());
    for (label, value) in [("Requirements", &spell.requirements), ("Cost", &spell.cost)] {
        if let Some(value) = value {
            text(out, label, value);
        }
    }
    if let Some(ritual) = known_value(&spell.ritual) {
        text(out, "Primary check", &ritual.primary_check);
        super::number(
            out,
            "Secondary casters",
            &ritual.secondary_casters,
            false,
            "",
        );
        text(out, "Secondary checks", &ritual.secondary_checks);
    } else if spell.ritual.state != QueryFieldState::Missing {
        fact(out, "Ritual casting", &spell.ritual, |_| String::new());
    }
    if known_value(&spell.counteraction) != Some(&false) {
        fact(out, "Counteract", &spell.counteraction, |_| "Yes".into());
    }
    text(out, "Range", &spell.range);
    optional_text(out, "Target", &spell.target);
    if let Some(value) = &spell.area {
        area(out, value);
    }
    if let Some(value) = &spell.defense {
        defense(out, value);
    }
    text(out, "Duration", &spell.duration);
    fact(out, "Sustained", &spell.sustained, |v| v.to_string());
    damage(out, &spell.damage);
    heightening(out, spell);
}

fn defense(out: &mut String, value: &FactView<SpellDefenseView>) {
    if let Some(defense) = known_value(value) {
        optional_text(out, "Save", &defense.statistic);
        if defense.basic.state != QueryFieldState::Missing {
            fact(out, "Basic save", &defense.basic, |v| v.to_string());
        }
        optional_text(out, "Passive defense", &defense.passive);
    } else if value.state != QueryFieldState::NotApplicable {
        line(out, format!("Defense: {}", unavailable(value.state)));
    }
}
fn heightening(out: &mut String, spell: &SpellPresentationView) {
    match known_value(&spell.heightening) {
        Some(SpellHeighteningView::Interval {
            interval,
            area: area_increment,
            damage: increments,
        }) => {
            line(
                out,
                format!("\nHeightening (+{})", number_text(interval, false, "")),
            );
            optional_number(out, "Area increase", area_increment, false, " feet");
            if let Some(increments) = known_value(increments) {
                for increment in increments {
                    if known_value(&increment.formula).is_some_and(|v| v.trim().is_empty()) {
                        continue;
                    }
                    line(
                        out,
                        format!(
                            "Damage increase: {}",
                            fact_string(&increment.formula, Clone::clone)
                        ),
                    );
                }
            } else if increments.state != QueryFieldState::NotApplicable {
                line(
                    out,
                    format!("Damage increments: {}", unavailable(increments.state)),
                );
            }
        }
        Some(SpellHeighteningView::Fixed { levels }) => {
            if let Some(levels) = known_value(levels) {
                for level in levels {
                    line(out, format!("\nHeightening (rank {})", level.rank));
                    line(out, "Changes to base");
                    changes(out, &level.changes);
                }
            } else {
                line(
                    out,
                    format!("Heightening levels: {}", unavailable(levels.state)),
                );
            }
        }
        None if !matches!(spell.heightening.state, QueryFieldState::NotApplicable) => line(
            out,
            format!("Heightening: {}", unavailable(spell.heightening.state)),
        ),
        None => {}
    }
    if let Some(forms) = known_value(&spell.forms) {
        for (index, form) in forms.iter().enumerate() {
            let name = known_value(&form.name)
                .filter(|v| !v.trim().is_empty())
                .or_else(|| {
                    known_value(&form.changes)
                        .and_then(|v| known_value(&v.cast))
                        .map(|v| &v.label)
                        .filter(|v| !v.trim().is_empty())
                })
                .map(String::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Alternative form {}", index + 1));
            line(out, format!("\n{name}"));
            line(out, "Changes to base");
            changes(out, &form.changes);
        }
    } else if !matches!(spell.forms.state, QueryFieldState::NotApplicable) {
        line(out, format!("Forms: {}", unavailable(spell.forms.state)));
    }
}

fn area(out: &mut String, value: &FactView<SpellAreaView>) {
    if let Some(area) = known_value(value) {
        let mut parts = Vec::new();
        for (required, fact) in [
            (true, &area.size),
            (true, &area.shape),
            (false, &area.details),
        ] {
            if fact.state != QueryFieldState::NotApplicable
                && (required
                    || (fact.state != QueryFieldState::Missing
                        && !known_value(fact).is_some_and(|v| v.trim().is_empty())))
            {
                parts.push(fact_string(fact, |v| {
                    if required && v.trim().is_empty() {
                        "Empty".into()
                    } else {
                        v.clone()
                    }
                }));
            }
        }
        if !parts.is_empty() {
            line(out, format!("Area: {}", parts.join(" ")));
        }
    } else if value.state != QueryFieldState::NotApplicable {
        line(out, format!("Area: {}", unavailable(value.state)));
    }
}
fn changes(out: &mut String, value: &FactView<SpellChangeView>) {
    if let Some(changes) = known_value(value) {
        change_fact(out, "Casting", &changes.cast, |v| {
            if v.label.trim().is_empty() {
                "cleared (empty)".into()
            } else {
                v.label.clone()
            }
        });
        change_text(out, "Range", &changes.range);
        change_text(out, "Target", &changes.target);
        if let Some(area) = known_value(&changes.area) {
            change_text(out, "Area size", &area.size);
            change_text(out, "Area shape", &area.shape);
            change_text(out, "Area details", &area.details);
        } else {
            change_fact(out, "Area", &changes.area, |_| String::new());
        }
        if let Some(defense) = known_value(&changes.defense) {
            change_text(out, "Save", &defense.statistic);
            change_fact(out, "Basic save", &defense.basic, |v| v.to_string());
            change_text(out, "Passive defense", &defense.passive);
        } else {
            change_fact(out, "Defense", &changes.defense, |_| String::new());
        }
        change_text(out, "Duration", &changes.duration);
        if !matches!(
            changes.sustained.state,
            QueryFieldState::Missing | QueryFieldState::NotApplicable
        ) {
            change_fact(out, "Sustained", &changes.sustained, |v| v.to_string());
        }
        for (label, value) in [
            ("Traits", &changes.traits),
            ("Traditions", &changes.traditions),
        ] {
            if !matches!(
                value.state,
                QueryFieldState::Missing | QueryFieldState::NotApplicable
            ) {
                change_fact(out, label, value, |v| list_text(v));
            }
        }
        if !matches!(
            changes.damage.state,
            QueryFieldState::Missing | QueryFieldState::NotApplicable
        ) {
            damage_changes(out, &changes.damage);
        }
        heightening_changes(out, &changes.heightening);
    } else {
        if value.state == QueryFieldState::Null {
            line(out, "Changes: cleared (null)");
        } else {
            line(out, format!("Changes: {}", unavailable(value.state)));
        }
    }
}

fn change_text(out: &mut String, label: &str, value: &FactView<String>) {
    change_fact(out, label, value, |v| {
        if v.trim().is_empty() {
            "Empty".into()
        } else {
            v.clone()
        }
    });
}
fn change_fact<T>(
    out: &mut String,
    label: &str,
    value: &FactView<T>,
    format: impl FnOnce(&T) -> String,
) {
    if matches!(
        value.state,
        QueryFieldState::Missing | QueryFieldState::NotApplicable
    ) {
        return;
    }
    if value.state == QueryFieldState::Null {
        line(out, format!("{label}: cleared (null)"));
    } else {
        fact(out, label, value, |v| {
            let text = format(v);
            if text.trim().is_empty() {
                "Empty".into()
            } else {
                text
            }
        });
    }
}
fn heightening_changes(out: &mut String, value: &FactView<SpellHeighteningChangeView>) {
    if let Some(changes) = known_value(value) {
        line(out, "Heightening changes");
        change_text(out, "Type", &changes.kind);
        change_number(out, "Interval", &changes.interval, "");
        change_number(out, "Area increase", &changes.area, " feet");
        if let Some(damage) = known_value(&changes.damage) {
            if damage.is_empty() {
                line(out, "Damage increments: Empty");
            }
            for (index, increment) in damage.iter().enumerate() {
                change_text(
                    out,
                    &format!(
                        "Damage increase{}",
                        if damage.len() > 1 {
                            format!(" {}", index + 1)
                        } else {
                            String::new()
                        }
                    ),
                    &increment.formula,
                );
            }
        } else {
            change_fact(out, "Damage increments", &changes.damage, |_| String::new());
        }
        if let Some(levels) = known_value(&changes.levels) {
            if levels.is_empty() {
                line(out, "Heightening levels: Empty");
            }
            for level in levels {
                line(out, format!("Rank {} changes", level.rank));
                self::changes(out, &level.changes);
            }
        } else {
            change_fact(out, "Heightening levels", &changes.levels, |_| {
                String::new()
            });
        }
    } else {
        change_fact(out, "Heightening", value, |_| String::new());
    }
}
fn change_number(
    out: &mut String,
    label: &str,
    value: &atlas_app_model::NumberFactView,
    unit: &str,
) {
    if value.state == QueryFieldState::Null {
        line(out, format!("{label}: cleared (null)"));
    } else {
        optional_number(out, label, value, false, unit);
    }
}
