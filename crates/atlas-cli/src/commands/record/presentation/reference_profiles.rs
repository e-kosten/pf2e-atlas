//! Terminal layout over the same supplied reference profiles as the web reader.
use super::{fact, line, number, optional_text, text};
use atlas_app_model::*;

fn grants(out: &mut String, values: &FactView<Vec<ReferenceGrantView>>, level_label: Option<&str>) {
    if let Some(values) = &values.value {
        let mut values = values.iter().collect::<Vec<_>>();
        if level_label.is_some() {
            values.sort_by_key(|grant| grant.level.value.as_ref().and_then(|value| value.as_u64()));
        }
        for value in values {
            let name = value
                .name
                .value
                .as_deref()
                .unwrap_or("Reference unavailable");
            if let Some(label) = level_label {
                line(
                    out,
                    format!(
                        "{}: {name}",
                        value.level.value.as_ref().map_or_else(
                            || format!("{label} unavailable"),
                            |level| format!("{label} {level}")
                        )
                    ),
                );
            } else {
                line(out, name);
            }
        }
    }
}

fn names(out: &mut String, label: &str, value: &FactView<Vec<String>>) {
    fact(out, label, value, |values| values.join(", "));
}
pub(super) fn render(out: &mut String, body: &RecordBodyView) {
    match body {
        RecordBodyView::Heritage(v) => optional_text(out, "Ancestry", &v.ancestry),
        RecordBodyView::Ancestry(v) => {
            number(out, "HP", &v.hit_points, false, "");
            number(out, "Speed", &v.speed, false, " feet");
            text(out, "Size", &v.size);
            text(out, "Vision", &v.vision);
            for (label, choices) in [
                ("Attribute boosts", &v.boosts),
                ("Attribute flaws", &v.flaws),
            ] {
                fact(out, label, choices, |slots| {
                    slots
                        .iter()
                        .map(|slot| {
                            if slot.free_choice {
                                return "Free".into();
                            }
                            slot.choices.value.as_ref().map_or_else(
                                || "Unavailable".into(),
                                |choices| choices.join(" or "),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ")
                });
            }
            names(out, "Languages", &v.languages);
            optional_text(out, "Languages", &v.language_details);
            names(out, "Additional language choices", &v.additional_languages);
            optional_text(out, "Additional languages", &v.additional_language_details);
            number(
                out,
                "Additional language slots",
                &v.additional_language_count,
                false,
                "",
            );
            number(out, "Hands", &v.hands, false, "");
            number(out, "Reach", &v.reach, false, " feet");
            grants(out, &v.grants, None);
        }
        RecordBodyView::Class(v) => {
            number(out, "Class HP", &v.hit_points, false, "");
            names(out, "Key attribute", &v.key_attributes);
            for (label, value) in [
                ("Perception", &v.perception),
                ("Fortitude", &v.fortitude),
                ("Reflex", &v.reflex),
                ("Will", &v.will),
                ("Simple weapons", &v.simple_weapons),
                ("Martial weapons", &v.martial_weapons),
                ("Advanced weapons", &v.advanced_weapons),
                ("Unarmed attacks", &v.unarmed),
                ("Unarmored defense", &v.unarmored),
                ("Light armor", &v.light_armor),
                ("Medium armor", &v.medium_armor),
                ("Heavy armor", &v.heavy_armor),
                ("Spellcasting", &v.spellcasting),
            ] {
                text(out, label, value);
            }
            if let Some(name) = &v.other_weapons.value {
                if !name.is_empty() {
                    text(out, name, &v.other_weapon_rank);
                }
            }
            names(out, "Trained skills", &v.trained_skills);
            number(
                out,
                "Additional skills before Intelligence modifier",
                &v.additional_skills,
                false,
                "",
            );
            grants(out, &v.grants, Some("Level"));
            for (label, levels) in [
                ("Ancestry feats", &v.ancestry_feat_levels),
                ("Class feats", &v.class_feat_levels),
                ("General feats", &v.general_feat_levels),
                ("Skill feats", &v.skill_feat_levels),
                ("Skill increases", &v.skill_increase_levels),
            ] {
                fact(out, label, levels, |values| {
                    values
                        .iter()
                        .map(u8::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                });
            }
        }
        RecordBodyView::Deity(v) => {
            names(out, "Divine attributes", &v.attributes);
            names(out, "Divine font", &v.fonts);
            names(out, "Divine skill", &v.skills);
            names(out, "Favored weapons", &v.favored_weapons);
            names(out, "Domains", &v.primary_domains);
            names(out, "Alternate domains", &v.alternate_domains);
            if let Some(value) = &v.sanctification.value {
                text(out, "Sanctification requirement", &value.requirement);
                names(out, "Sanctification choices", &value.choices);
            }
            grants(out, &v.spells, Some("Rank"));
        }
        RecordBodyView::Vehicle(v) => {
            text(out, "Crew", &v.crew);
            text(out, "Passengers", &v.passengers);
            text(out, "Speed", &v.speed);
            text(out, "Piloting", &v.piloting);
            text(out, "Size", &v.size);
            for (label, value, unit) in [
                ("AC", &v.defenses.armor_class, ""),
                ("HP", &v.defenses.hit_points, ""),
                ("Hardness", &v.defenses.hardness, ""),
                ("Fortitude", &v.fortitude, ""),
                ("Collision DC", &v.collision_dc, ""),
                ("Price", &v.price_gp, " gp"),
                ("Length", &v.length, " feet"),
                ("Width", &v.width, " feet"),
                ("Height", &v.height, " feet"),
            ] {
                number(out, label, value, label == "Fortitude", unit);
            }
            text(out, "Collision damage", &v.collision_damage);
            super::actor::iwr(out, "Immunities", &v.defenses.immunities);
            super::actor::iwr(out, "Weaknesses", &v.defenses.weaknesses);
            super::actor::iwr(out, "Resistances", &v.defenses.resistances);
        }
        RecordBodyView::Army(v) => {
            text(out, "Army type", &v.army_type);
            for (label, value) in [
                ("AC", &v.armor_class),
                ("HP", &v.hit_points),
                ("Rout threshold", &v.rout_threshold),
                ("Maneuver", &v.maneuver),
                ("Morale", &v.morale),
                ("Scouting", &v.scouting),
                ("Consumption", &v.consumption),
                ("Recruitment DC", &v.recruitment_dc),
                ("Starting ammunition", &v.ammunition),
                ("Starting potions", &v.potions),
            ] {
                number(out, label, value, false, "");
            }
            optional_text(out, "Melee weapon", &v.melee_weapon);
            optional_text(out, "Ranged weapon", &v.ranged_weapon);
            super::actor::iwr(out, "Immunities", &v.immunities);
            super::actor::iwr(out, "Weaknesses", &v.weaknesses);
            super::actor::iwr(out, "Resistances", &v.resistances);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::QueryFieldState;

    #[test]
    fn grant_labels_keep_ancestry_order_and_distinguish_spell_ranks() {
        let mut values = FactView {
            state: QueryFieldState::Value,
            value: Some(
                [("Later entry", 3), ("First entry", 0)]
                    .into_iter()
                    .map(|(name, level)| ReferenceGrantView {
                        name: FactView {
                            state: QueryFieldState::Value,
                            value: Some(name.into()),
                        },
                        target: FactView {
                            state: QueryFieldState::Missing,
                            value: None,
                        },
                        level: NumberFactView {
                            state: QueryFieldState::Value,
                            value: Some(serde_json::Number::from(level)),
                            adjustment: None,
                        },
                        navigation: None,
                    })
                    .collect(),
            ),
        };
        let mut ancestry = String::new();
        grants(&mut ancestry, &values, None);
        assert_eq!(ancestry, "Later entry\nFirst entry\n");
        values.value.as_mut().unwrap()[1].level.value = Some(1.into());
        let mut spells = String::new();
        grants(&mut spells, &values, Some("Rank"));
        assert_eq!(spells, "Rank 1: First entry\nRank 3: Later entry\n");
        let mut class = String::new();
        grants(&mut class, &values, Some("Level"));
        assert_eq!(class, "Level 1: First entry\nLevel 3: Later entry\n");
    }
}
