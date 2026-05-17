use std::collections::BTreeMap;

use atlas_domain::MetricDomain;

use super::*;
use crate::{MetricRow, MetricValue};

#[test]
fn exact_static_definitions_win_before_patterns() {
    let matched = definition_for(MetricDomain::Actor, "ac.value").expect("metric is known");

    assert_eq!(matched.definition.group(), MetricGroup::Defense);
    assert!(matched.captures.is_empty());
    assert_eq!(matched.label().label, "Armor Class");
    assert_eq!(matched.label().short_label.as_deref(), Some("AC"));
}

#[test]
fn pattern_definitions_capture_and_label_variables() {
    let matched = definition_for(MetricDomain::Actor, "skill.arcana.mod").expect("metric is known");

    assert_eq!(matched.definition.group(), MetricGroup::Skills);
    assert_eq!(matched.captures[0].name, "skill");
    assert_eq!(matched.captures[0].raw, "arcana");
    assert_eq!(matched.captures[0].label, "Arcana");
    assert_eq!(matched.label().label, "Arcana modifier");
}

#[test]
fn open_vocabularies_titleize_captured_slugs() {
    let matched =
        definition_for(MetricDomain::Actor, "speed.spider_climb.value").expect("metric known");

    assert_eq!(matched.label().label, "Spider climb Speed");
}

#[test]
fn unknown_rows_fall_back_to_raw_key_labels() {
    let row = MetricRow {
        domain: MetricDomain::Actor,
        key: "unknown.metric".to_string(),
        value: MetricValue::Number(12.0),
    };

    assert_eq!(
        label_for_row(&row),
        MetricDisplayLabel {
            label: "unknown.metric".to_string(),
            short_label: None,
            known: false,
        }
    );
}

#[test]
fn current_emitted_metric_inventory_is_covered() {
    let known_keys = [
        (MetricDomain::Actor, "ability.cha.mod"),
        (MetricDomain::Actor, "ability.con.mod"),
        (MetricDomain::Actor, "ability.dex.mod"),
        (MetricDomain::Actor, "ability.int.mod"),
        (MetricDomain::Actor, "ability.str.mod"),
        (MetricDomain::Actor, "ability.wis.mod"),
        (MetricDomain::Actor, "ac.value"),
        (MetricDomain::Actor, "hardness.value"),
        (MetricDomain::Actor, "hp.bt"),
        (MetricDomain::Actor, "hp.max"),
        (MetricDomain::Actor, "hp.value"),
        (MetricDomain::Actor, "perception.mod"),
        (MetricDomain::Actor, "save.best"),
        (MetricDomain::Actor, "save.fort.mod"),
        (MetricDomain::Actor, "save.ref.mod"),
        (MetricDomain::Actor, "save.will.mod"),
        (MetricDomain::Actor, "save.worst"),
        (MetricDomain::Actor, "skill.arcana.mod"),
        (MetricDomain::Actor, "skill.arcana.proficient"),
        (MetricDomain::Actor, "skill.arcana.rank"),
        (MetricDomain::Actor, "speed.fly.value"),
        (MetricDomain::Actor, "sense.scent.range"),
        (MetricDomain::Actor, "stealth.dc"),
        (MetricDomain::Actor, "stealth.mod"),
        (MetricDomain::Actor, "disable.dc.min"),
        (MetricDomain::Actor, "disable.dc.max"),
        (MetricDomain::Actor, "disable.thievery.dc.min"),
        (MetricDomain::Actor, "disable.thievery.dc.max"),
        (MetricDomain::Actor, "disable.thievery.rank.min"),
        (MetricDomain::Item, "armor.ac_bonus"),
        (MetricDomain::Item, "armor.check_penalty"),
        (MetricDomain::Item, "armor.dex_cap"),
        (MetricDomain::Item, "armor.speed_penalty"),
        (MetricDomain::Item, "armor.strength"),
        (MetricDomain::Item, "shield.ac_bonus"),
        (MetricDomain::Item, "shield.bt"),
        (MetricDomain::Item, "shield.hardness"),
        (MetricDomain::Item, "shield.hp"),
        (MetricDomain::Item, "weapon.damage_dice"),
        (MetricDomain::Item, "weapon.damage_die_faces"),
        (MetricDomain::Item, "weapon.range_increment"),
        (MetricDomain::Item, "weapon.reload"),
    ];

    for (domain, key) in known_keys {
        assert!(
            is_known_key(domain, key),
            "{domain:?}/{key} should be known"
        );
    }
}

#[test]
fn pattern_definitions_do_not_overlap_on_current_inventory() {
    let current_dynamic_keys = [
        "ability.cha.mod",
        "ability.con.mod",
        "ability.dex.mod",
        "ability.int.mod",
        "ability.str.mod",
        "ability.wis.mod",
        "save.fort.mod",
        "save.ref.mod",
        "save.will.mod",
        "skill.arcana.mod",
        "skill.arcana.proficient",
        "skill.arcana.rank",
        "speed.fly.value",
        "speed.spider_climb.value",
        "sense.scent.range",
        "sense.infrared_vision.range",
        "disable.thievery.dc.min",
        "disable.thievery.dc.max",
        "disable.thievery.rank.min",
    ];

    for key in current_dynamic_keys {
        let mut matches_by_domain: BTreeMap<MetricDomain, usize> = BTreeMap::new();
        for definition in all_definitions() {
            let MetricDefinition::Pattern(pattern) = definition else {
                continue;
            };
            if matching::match_pattern(pattern.pattern, key).is_some() {
                *matches_by_domain.entry(pattern.domain).or_default() += 1;
            }
        }

        for (domain, count) in matches_by_domain {
            assert_eq!(
                count, 1,
                "{domain:?}/{key} should not match multiple patterns"
            );
        }
    }
}
