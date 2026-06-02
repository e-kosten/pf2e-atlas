use atlas_domain::MetricDomain;
use atlas_record::metrics as metric_definitions;
use atlas_record::{MetricRow, MetricValue};
use serde_json::Value;

use super::add_defined_metric_number;
use super::disable::extract_disable_metrics;
use super::emit::{EmittedDynamicMetric, emit_dynamic_specs, emit_static_specs};
use super::exact_metric_definition_key;
use super::specs::{ACTOR_DYNAMIC_SPECS, ACTOR_STATIC_SPECS};
use super::value::number_at_pointer;

pub(super) fn extract_actor_metrics(raw: &Value) -> Result<Vec<MetricRow>, String> {
    let mut metrics = Vec::new();

    emit_static_specs(raw, &mut metrics, ACTOR_STATIC_SPECS)?;
    let emitted_dynamic = emit_dynamic_specs(raw, &mut metrics, ACTOR_DYNAMIC_SPECS);
    let save_values = save_values_from_emitted_metrics(&emitted_dynamic);
    add_best_worst_save_metrics(&mut metrics, &save_values)?;
    extract_skill_proficiency_metrics(raw, &mut metrics);
    extract_stealth_metrics(raw, &mut metrics)?;
    extract_disable_metrics(raw, &mut metrics)?;
    Ok(metrics)
}

fn save_values_from_emitted_metrics(emitted: &[EmittedDynamicMetric]) -> Vec<(&'static str, f64)> {
    let mut save_values = Vec::new();
    for metric in emitted {
        if metric.definition != metric_definitions::actor::save::MOD {
            continue;
        }
        let save = match metric.capture.as_str() {
            "fort" => "fort",
            "ref" => "ref",
            "will" => "will",
            _ => continue,
        };
        save_values.push((save, metric.value));
    }

    save_values
}

fn add_best_worst_save_metrics(
    metrics: &mut Vec<MetricRow>,
    save_values: &[(&'static str, f64)],
) -> Result<(), String> {
    let Some((best_save, _)) = save_values
        .iter()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    else {
        return Ok(());
    };
    let Some((worst_save, _)) = save_values
        .iter()
        .min_by(|left, right| left.1.total_cmp(&right.1))
    else {
        return Ok(());
    };

    metrics.push(MetricRow {
        domain: MetricDomain::Actor,
        key: exact_metric_definition_key(metric_definitions::actor::save::BEST)?.to_string(),
        value: MetricValue::Text((*best_save).to_string()),
    });
    metrics.push(MetricRow {
        domain: MetricDomain::Actor,
        key: exact_metric_definition_key(metric_definitions::actor::save::WORST)?.to_string(),
        value: MetricValue::Text((*worst_save).to_string()),
    });
    Ok(())
}

fn extract_skill_proficiency_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let Some(skills) = raw.pointer("/system/skills").and_then(Value::as_object) else {
        return;
    };

    for (raw_key, value) in skills {
        let skill_key = super::value::slugify_metric_segment(raw_key);
        if skill_key.is_empty() {
            continue;
        }
        if let Some(rank) = number_at_pointer(value, "/rank") {
            metrics.push(MetricRow {
                domain: MetricDomain::Actor,
                key: metric_definitions::actor::skill::proficient_key(&skill_key),
                value: MetricValue::Boolean(rank >= 1.0),
            });
        }
    }
}

fn extract_stealth_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) -> Result<(), String> {
    let stealth_mod_key = exact_metric_definition_key(metric_definitions::actor::STEALTH_MOD)?;
    let stealth_mod = metrics.iter().find_map(|metric| {
        if metric.domain == MetricDomain::Actor
            && metric.key == stealth_mod_key
            && let MetricValue::Number(value) = metric.value
        {
            return Some(value);
        }
        None
    });
    add_defined_metric_number(
        metrics,
        metric_definitions::actor::STEALTH_DC,
        number_at_pointer(raw, "/system/attributes/stealth/dc")
            .or_else(|| stealth_mod.map(|value| value + 10.0)),
    )?;
    Ok(())
}
