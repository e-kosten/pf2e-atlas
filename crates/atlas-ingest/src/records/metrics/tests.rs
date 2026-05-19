use atlas_domain::MetricDomain;
use atlas_record::metrics as metric_definitions;
use serde_json::Value;

use crate::records::{MetricRow, MetricValue};

use super::actor::extract_actor_metrics;
use super::specs::{
    ACTOR_DYNAMIC_SPECS, ACTOR_STATIC_SPECS, ARMOR_STATIC_SPECS, DynamicMetricSourceSpec,
    SHIELD_STATIC_SPECS, WEAPON_STATIC_SPECS,
};
use super::value::{number_like_value, slugify_metric_segment};
use super::{dedupe_metrics, exact_metric_key, validate_metric_rows};

#[test]
fn slugifies_metric_segments_to_stable_keys() {
    assert_eq!(slugify_metric_segment("Fortitude Save"), "fortitude_save");
    assert_eq!(slugify_metric_segment("  Land-Speed! "), "land_speed");
}

#[test]
fn parses_first_numeric_prefix_from_text_values() {
    assert_eq!(
        number_like_value(&Value::String("30 feet".to_string())),
        Some(30.0)
    );
    assert_eq!(
        number_like_value(&Value::String("-5 penalty".to_string())),
        Some(-5.0)
    );
}

#[test]
fn dedupes_metrics_by_domain_and_key_with_later_values_winning() {
    let metrics = dedupe_metrics(vec![
        MetricRow {
            domain: MetricDomain::Actor,
            key: exact_metric_key(metric_definitions::actor::HP_VALUE).to_string(),
            value: MetricValue::Number(5.0),
        },
        MetricRow {
            domain: MetricDomain::Actor,
            key: exact_metric_key(metric_definitions::actor::HP_VALUE).to_string(),
            value: MetricValue::Number(9.0),
        },
    ]);

    assert_eq!(
        metrics,
        vec![MetricRow {
            domain: MetricDomain::Actor,
            key: exact_metric_key(metric_definitions::actor::HP_VALUE).to_string(),
            value: MetricValue::Number(9.0),
        }]
    );
}

#[test]
fn rejects_metrics_without_definitions() {
    let error = validate_metric_rows(&[MetricRow {
        domain: MetricDomain::Actor,
        key: "custom.future_metric".to_string(),
        value: MetricValue::Number(1.0),
    }])
    .expect_err("unknown metric keys should fail validation");

    assert_eq!(
        error,
        "emitted metric actor.custom.future_metric has no typed definition"
    );
}

#[test]
fn rejects_metric_value_type_mismatches() {
    let error = validate_metric_rows(&[MetricRow {
        domain: MetricDomain::Actor,
        key: exact_metric_key(metric_definitions::actor::ARMOR_CLASS).to_string(),
        value: MetricValue::Text("not numeric".to_string()),
    }])
    .expect_err("metric type mismatch should fail validation");

    assert_eq!(
        error,
        "emitted metric actor.ac.value has type text, expected number"
    );
}

#[test]
fn source_specs_reference_the_definitions_they_emit() {
    for spec in [
        ACTOR_STATIC_SPECS,
        WEAPON_STATIC_SPECS,
        ARMOR_STATIC_SPECS,
        SHIELD_STATIC_SPECS,
    ]
    .into_iter()
    .flatten()
    {
        let key = exact_metric_key(spec.definition);
        let matched = metric_definitions::definition_for(spec.definition.domain(), key)
            .expect("static source spec should reference a known metric definition");
        assert_eq!(*matched.definition, spec.definition);
    }

    for spec in ACTOR_DYNAMIC_SPECS {
        let (definition, key) = match *spec {
            DynamicMetricSourceSpec::FixedCapture {
                definition,
                capture,
                key_builder,
                ..
            } => (definition, key_builder(capture)),
            DynamicMetricSourceSpec::ClosedVocabulary {
                definition,
                captures,
                key_builder,
                ..
            } => (
                definition,
                key_builder(captures.first().expect("closed vocabulary has a sample")),
            ),
            DynamicMetricSourceSpec::ObjectEntries {
                definition,
                key_builder,
                ..
            } => {
                let sample = match definition {
                    metric_definitions::actor::save::MOD => "fort",
                    metric_definitions::actor::skill::MOD
                    | metric_definitions::actor::skill::RANK => "arcana",
                    _ => panic!("dynamic object source spec needs a representative capture"),
                };
                (definition, key_builder(sample))
            }
            DynamicMetricSourceSpec::ArrayEntries {
                definition,
                key_builder,
                ..
            } => {
                let sample = match definition {
                    metric_definitions::actor::speed::VALUE => "fly",
                    metric_definitions::actor::sense::RANGE => "darkvision",
                    _ => panic!("dynamic array source spec needs a representative capture"),
                };
                (definition, key_builder(sample))
            }
        };
        let matched = metric_definitions::definition_for(definition.domain(), &key)
            .expect("dynamic source spec should emit a known metric key");
        assert_eq!(*matched.definition, definition);
    }
}

#[test]
fn source_specs_emit_first_valid_static_metric_path() {
    let raw = serde_json::json!({
        "system": {
            "attributes": {
                "hp": {
                    "broken": "7",
                    "bt": 9
                }
            }
        }
    });

    let metrics = extract_actor_metrics(&raw);

    assert_number_metric(
        &metrics,
        exact_metric_key(metric_definitions::actor::HP_BROKEN_THRESHOLD),
        7.0,
    );
}

#[test]
fn source_specs_emit_dynamic_pattern_metrics() {
    let raw = serde_json::json!({
        "system": {
            "abilities": {
                "str": { "modifier": 4 }
            },
            "saves": {
                "fortitude": { "totalModifier": 9 }
            },
            "skills": {
                "Arcana": { "value": 11, "rank": 2 }
            },
            "attributes": {
                "speed": {
                    "value": "25 feet",
                    "otherSpeeds": [
                        { "type": "Fly Speed", "value": "40 feet" }
                    ]
                }
            },
            "perception": {
                "senses": [
                    { "type": "Darkvision", "range": "60 feet" }
                ]
            }
        }
    });

    let metrics = extract_actor_metrics(&raw);

    assert_number_metric(
        &metrics,
        &metric_definitions::actor::ability::mod_key("str"),
        4.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::save::mod_key("fort"),
        9.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::skill::mod_key("arcana"),
        11.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::skill::rank_key("arcana"),
        2.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::speed::value_key("land"),
        25.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::speed::value_key("fly_speed"),
        40.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::sense::range_key("darkvision"),
        60.0,
    );
}

#[test]
fn extracts_disable_dc_and_rank_metrics_from_hazard_checks() {
    let raw = serde_json::json!({
        "system": {
            "details": {
                "disable": "@Check[thievery|dc:27] (expert) to disable the lock @Check[crafting|dc:30] or Thievery (master) to jam the gears"
            }
        }
    });

    let metrics = extract_actor_metrics(&raw);

    assert_number_metric(
        &metrics,
        exact_metric_key(metric_definitions::actor::disable::DC_MIN),
        27.0,
    );
    assert_number_metric(
        &metrics,
        exact_metric_key(metric_definitions::actor::disable::DC_MAX),
        30.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::disable::skill_dc_min_key("thievery"),
        27.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::disable::skill_dc_max_key("thievery"),
        30.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::disable::skill_rank_min_key("thievery"),
        3.0,
    );
    assert_number_metric(
        &metrics,
        &metric_definitions::actor::disable::skill_dc_min_key("crafting"),
        30.0,
    );
}

fn assert_number_metric(metrics: &[MetricRow], key: &str, expected: f64) {
    let actual = metrics.iter().find_map(|metric| {
        if metric.domain == MetricDomain::Actor
            && metric.key == key
            && let MetricValue::Number(value) = metric.value
        {
            return Some(value);
        }
        None
    });
    assert_eq!(actual, Some(expected), "metric {key} should match");
}
