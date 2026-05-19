use atlas_record::metrics as metric_definitions;
use serde_json::Value;

use crate::records::MetricRow;
use crate::source::normalize::pointer_string;

use super::specs::{
    CaptureNormalize, DynamicMetricSourceSpec, MetricCoercion, MetricPathCandidate,
    MetricPathTemplateCandidate, StaticMetricSourceSpec,
};
use super::value::{
    damage_die_faces, number_at_pointer, number_like_at_pointer, slugify_metric_segment,
};
use super::{add_defined_metric_number, add_metric_number, exact_metric_key};

pub(super) fn emit_static_specs(
    raw: &Value,
    metrics: &mut Vec<MetricRow>,
    specs: &[StaticMetricSourceSpec],
) {
    for spec in specs {
        add_defined_metric_number(
            metrics,
            spec.definition,
            first_value_at_paths(raw, spec.paths),
        );
    }
}

pub(super) fn emit_dynamic_specs(
    raw: &Value,
    metrics: &mut Vec<MetricRow>,
    specs: &[DynamicMetricSourceSpec],
) -> Vec<EmittedDynamicMetric> {
    let mut emitted = Vec::new();
    for spec in specs {
        match *spec {
            DynamicMetricSourceSpec::FixedCapture {
                definition,
                capture,
                key_builder,
                paths,
            } => emit_dynamic_metric(
                metrics,
                &mut emitted,
                definition,
                capture,
                key_builder,
                first_value_at_paths(raw, paths),
            ),
            DynamicMetricSourceSpec::ClosedVocabulary {
                definition,
                captures,
                key_builder,
                path_templates,
            } => {
                for capture in captures {
                    emit_dynamic_metric(
                        metrics,
                        &mut emitted,
                        definition,
                        capture,
                        key_builder,
                        first_value_at_templates(raw, capture, path_templates),
                    );
                }
            }
            DynamicMetricSourceSpec::ObjectEntries {
                definition,
                collection_path,
                capture_normalize,
                key_builder,
                value_paths,
            } => {
                let Some(entries) = raw.pointer(collection_path).and_then(Value::as_object) else {
                    continue;
                };
                for (raw_key, value) in entries {
                    let Some(capture) = normalize_capture(raw_key, capture_normalize) else {
                        continue;
                    };
                    emit_dynamic_metric(
                        metrics,
                        &mut emitted,
                        definition,
                        &capture,
                        key_builder,
                        first_value_at_paths(value, value_paths),
                    );
                }
            }
            DynamicMetricSourceSpec::ArrayEntries {
                definition,
                collection_path,
                capture_path,
                capture_normalize,
                key_builder,
                value_paths,
            } => {
                let Some(entries) = raw.pointer(collection_path).and_then(Value::as_array) else {
                    continue;
                };
                for entry in entries {
                    let Some(raw_capture) = pointer_string(entry, capture_path) else {
                        continue;
                    };
                    let Some(capture) = normalize_capture(&raw_capture, capture_normalize) else {
                        continue;
                    };
                    emit_dynamic_metric(
                        metrics,
                        &mut emitted,
                        definition,
                        &capture,
                        key_builder,
                        first_value_at_paths(entry, value_paths),
                    );
                }
            }
        }
    }
    emitted
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct EmittedDynamicMetric {
    pub definition: metric_definitions::MetricDefinition,
    pub capture: String,
    pub key: String,
    pub value: f64,
}

fn emit_dynamic_metric(
    metrics: &mut Vec<MetricRow>,
    emitted: &mut Vec<EmittedDynamicMetric>,
    definition: metric_definitions::MetricDefinition,
    capture: &str,
    key_builder: fn(&str) -> String,
    value: Option<f64>,
) {
    let Some(value) = value else {
        return;
    };
    let key = key_builder(capture);
    debug_assert_metric_key_matches_definition(definition, &key);
    add_metric_number(metrics, definition.domain(), &key, Some(value));
    emitted.push(EmittedDynamicMetric {
        definition,
        capture: capture.to_string(),
        key,
        value,
    });
}

fn first_value_at_paths(raw: &Value, paths: &[MetricPathCandidate]) -> Option<f64> {
    paths
        .iter()
        .find_map(|candidate| value_at_path(raw, candidate))
}

fn first_value_at_templates(
    raw: &Value,
    capture: &str,
    templates: &[MetricPathTemplateCandidate],
) -> Option<f64> {
    templates.iter().find_map(|template| {
        let path = format!("{}{}{}", template.prefix, capture, template.suffix);
        value_at_pointer(raw, &path, template.coercion)
    })
}

fn value_at_path(raw: &Value, candidate: &MetricPathCandidate) -> Option<f64> {
    value_at_pointer(raw, candidate.path, candidate.coercion)
}

fn value_at_pointer(raw: &Value, path: &str, coercion: MetricCoercion) -> Option<f64> {
    match coercion {
        MetricCoercion::Number => number_at_pointer(raw, path),
        MetricCoercion::NumberLike => number_like_at_pointer(raw, path),
        MetricCoercion::DamageDieFaces => damage_die_faces(raw.pointer(path)),
    }
}

fn normalize_capture(raw: &str, normalize: CaptureNormalize) -> Option<String> {
    match normalize {
        CaptureNormalize::Slug => {
            let capture = slugify_metric_segment(raw);
            (!capture.is_empty()).then_some(capture)
        }
        CaptureNormalize::SaveKey => normalize_save_key(raw).map(str::to_string),
    }
}

fn normalize_save_key(value: &str) -> Option<&'static str> {
    match slugify_metric_segment(value).as_str() {
        "fort" | "fortitude" => Some("fort"),
        "ref" | "reflex" => Some("ref"),
        "will" => Some("will"),
        _ => None,
    }
}

fn debug_assert_metric_key_matches_definition(
    definition: metric_definitions::MetricDefinition,
    key: &str,
) {
    debug_assert!(
        match metric_definitions::definition_for(definition.domain(), key) {
            Some(matched) => *matched.definition == definition,
            None => false,
        },
        "source spec key {}.{} does not match referenced metric definition",
        definition.domain().as_str(),
        key
    );
    if definition.exact_key().is_some() {
        debug_assert_eq!(exact_metric_key(definition), key);
    }
}
