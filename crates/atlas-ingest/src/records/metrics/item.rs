use serde_json::Value;

use crate::records::MetricRow;

use super::emit::emit_static_specs;
use super::specs::{ARMOR_STATIC_SPECS, SHIELD_STATIC_SPECS, WEAPON_STATIC_SPECS};
use super::value::slugify_metric_segment;

pub(super) fn extract_item_metrics(raw: &Value, record_type: &str) -> Vec<MetricRow> {
    let mut metrics = Vec::new();
    match slugify_metric_segment(record_type).as_str() {
        "weapon" => emit_static_specs(raw, &mut metrics, WEAPON_STATIC_SPECS),
        "armor" => emit_static_specs(raw, &mut metrics, ARMOR_STATIC_SPECS),
        "shield" => emit_static_specs(raw, &mut metrics, SHIELD_STATIC_SPECS),
        _ => {}
    }
    metrics
}
