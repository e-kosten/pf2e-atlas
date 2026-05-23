use std::collections::BTreeSet;

use atlas_record::NormalizedRecord;

use atlas_record::MetricValue;

pub(crate) fn record_filter_values(record: &NormalizedRecord) -> BTreeSet<(&'static str, String)> {
    let mut values = BTreeSet::new();
    values.extend(
        record
            .taxonomy_families
            .iter()
            .cloned()
            .map(|value| ("taxonomy_families", value)),
    );
    values.extend(
        record
            .variant_axes
            .iter()
            .cloned()
            .map(|value| ("variant_axes", value)),
    );
    if let Some(actor) = &record.actor_data {
        values.extend(
            actor
                .languages
                .iter()
                .cloned()
                .map(|value| ("languages", value)),
        );
        values.extend(
            actor
                .speed_types
                .iter()
                .cloned()
                .map(|value| ("speed_types", value)),
        );
        values.extend(actor.senses.iter().cloned().map(|value| ("senses", value)));
        values.extend(
            actor
                .immunities
                .iter()
                .cloned()
                .map(|value| ("immunities", value)),
        );
        values.extend(
            actor
                .resistances
                .iter()
                .cloned()
                .map(|value| ("resistances", value)),
        );
        values.extend(
            actor
                .weaknesses
                .iter()
                .cloned()
                .map(|value| ("weaknesses", value)),
        );
        values.extend(
            actor
                .disable_skills
                .iter()
                .cloned()
                .map(|value| ("disable_skills", value)),
        );
    }
    if let Some(item) = &record.item_data {
        values.extend(
            item.damage_types
                .iter()
                .cloned()
                .map(|value| ("damage_types", value)),
        );
    }
    if let Some(spell) = &record.spell_data {
        values.extend(
            spell
                .traditions
                .iter()
                .cloned()
                .map(|value| ("traditions", value)),
        );
        values.extend(
            spell
                .spell_kinds
                .iter()
                .cloned()
                .map(|value| ("spell_kinds", value)),
        );
        values.extend(
            spell
                .damage_types
                .iter()
                .cloned()
                .map(|value| ("damage_types", value)),
        );
    }
    values
}

pub(crate) fn filter_value_key(field: &str, value: &str) -> String {
    format!("{field}:{value}")
}

pub(crate) fn metric_value_type(value: &MetricValue) -> &'static str {
    match value {
        MetricValue::Number(_) => "number",
        MetricValue::Text(_) => "text",
        MetricValue::Boolean(_) => "boolean",
    }
}

pub(crate) fn alias_key(alias: &atlas_record::RecordAlias) -> String {
    format!(
        "{}#alias#{}#{}",
        alias.canonical_record_key, alias.normalized_alias, alias.source_ref
    )
}

pub(crate) fn metric_key_id(domain: &str, key: &str) -> String {
    format!("{domain}:{key}")
}

pub(crate) fn namespace_prefix(metric_key: &str) -> String {
    metric_key
        .rsplit_once('.')
        .map(|(prefix, _)| format!("{prefix}."))
        .unwrap_or_default()
}

pub(crate) fn publication_key(title: &str) -> String {
    title.trim().to_lowercase()
}
