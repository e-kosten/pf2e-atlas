use crate::diagnostics::IngestDiagnostics;
use crate::records::LoadedSourceRecord;
use atlas_domain::MetricDomain;
use atlas_record::MetricValue;
use atlas_record::metrics as metric_definitions;
use serde_json::Value;

pub(crate) fn audit_emitted_metrics(
    records: &[LoadedSourceRecord],
    diagnostics: &mut IngestDiagnostics,
    warnings: &mut Vec<String>,
) {
    for loaded in records {
        for metric in &loaded.record.metrics {
            if atlas_record::is_known_key(metric.domain, &metric.key) {
                continue;
            }
            let audit_key = format!(
                "{}.{} type={}",
                metric.domain.as_str(),
                metric.key,
                metric_value_type(&metric.value)
            );
            let diagnostic = diagnostics
                .unknown_emitted_metrics
                .entry(audit_key)
                .or_default();
            diagnostic.count += 1;
            if diagnostic.examples.len() < 5 {
                diagnostic.examples.push(loaded.record.key.to_string());
            }
        }
    }

    if !diagnostics.unknown_emitted_metrics.is_empty() {
        warnings.push(format!(
            "{} emitted metric keys have no typed definition: {}",
            diagnostics.unknown_emitted_metrics.len(),
            metric_warning_examples(&diagnostics.unknown_emitted_metrics)
        ));
    }
    audit_source_metric_candidates(records, diagnostics, warnings);
}

fn audit_source_metric_candidates(
    records: &[LoadedSourceRecord],
    diagnostics: &mut IngestDiagnostics,
    warnings: &mut Vec<String>,
) {
    for loaded in records {
        let Ok(raw) = serde_json::from_str::<Value>(&loaded.record.raw_json) else {
            continue;
        };
        for candidate in source_metric_candidates(loaded, &raw) {
            if loaded
                .record
                .metrics
                .iter()
                .any(|metric| metric.domain == candidate.domain && metric.key == candidate.key)
            {
                continue;
            }
            let audit_key = format!(
                "{}.{}@{} type={}",
                candidate.domain.as_str(),
                candidate.key,
                candidate.source_path,
                candidate.value_type
            );
            let diagnostic = diagnostics
                .unemitted_source_metric_candidates
                .entry(audit_key)
                .or_default();
            diagnostic.count += 1;
            if diagnostic.examples.len() < 5 {
                diagnostic.examples.push(loaded.record.key.to_string());
            }
        }
    }

    if !diagnostics.unemitted_source_metric_candidates.is_empty() {
        warnings.push(format!(
            "{} source metric candidates were not emitted: {}",
            diagnostics.unemitted_source_metric_candidates.len(),
            metric_warning_examples(&diagnostics.unemitted_source_metric_candidates)
        ));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceMetricCandidate {
    domain: MetricDomain,
    key: String,
    source_path: String,
    value_type: &'static str,
}

fn source_metric_candidates(
    loaded: &LoadedSourceRecord,
    raw: &Value,
) -> Vec<SourceMetricCandidate> {
    match loaded.record.foundry_document_type.as_str() {
        "Actor" => actor_source_metric_candidates(raw),
        "Item" => item_source_metric_candidates(&loaded.record.foundry_record_type, raw),
        _ => Vec::new(),
    }
}

fn actor_source_metric_candidates(raw: &Value) -> Vec<SourceMetricCandidate> {
    let mut candidates = Vec::new();
    for ability in ["str", "dex", "con", "int", "wis", "cha"] {
        push_number_candidate(
            &mut candidates,
            raw,
            &format!("/system/abilities/{ability}/mod"),
            MetricDomain::Actor,
            metric_definitions::actor::ability::mod_key(ability),
        );
        push_number_candidate(
            &mut candidates,
            raw,
            &format!("/system/abilities/{ability}/modifier"),
            MetricDomain::Actor,
            metric_definitions::actor::ability::mod_key(ability),
        );
    }

    for (path, key) in [
        (
            "/system/attributes/ac/value",
            exact_metric_key(metric_definitions::actor::ARMOR_CLASS).to_string(),
        ),
        (
            "/system/attributes/hp/value",
            exact_metric_key(metric_definitions::actor::HP_VALUE).to_string(),
        ),
        (
            "/system/attributes/hp/max",
            exact_metric_key(metric_definitions::actor::HP_MAX).to_string(),
        ),
        (
            "/system/attributes/hp/brokenThreshold",
            exact_metric_key(metric_definitions::actor::HP_BROKEN_THRESHOLD).to_string(),
        ),
        (
            "/system/attributes/hp/broken",
            exact_metric_key(metric_definitions::actor::HP_BROKEN_THRESHOLD).to_string(),
        ),
        (
            "/system/attributes/hp/bt",
            exact_metric_key(metric_definitions::actor::HP_BROKEN_THRESHOLD).to_string(),
        ),
        (
            "/system/attributes/hardness",
            exact_metric_key(metric_definitions::actor::HARDNESS).to_string(),
        ),
        (
            "/system/attributes/stealth/value",
            exact_metric_key(metric_definitions::actor::STEALTH_MOD).to_string(),
        ),
        (
            "/system/attributes/stealth/mod",
            exact_metric_key(metric_definitions::actor::STEALTH_MOD).to_string(),
        ),
        (
            "/system/attributes/stealth/modifier",
            exact_metric_key(metric_definitions::actor::STEALTH_MOD).to_string(),
        ),
        (
            "/system/attributes/stealth/dc",
            exact_metric_key(metric_definitions::actor::STEALTH_DC).to_string(),
        ),
        (
            "/system/perception/mod",
            exact_metric_key(metric_definitions::actor::PERCEPTION_MOD).to_string(),
        ),
        (
            "/system/perception/modifier",
            exact_metric_key(metric_definitions::actor::PERCEPTION_MOD).to_string(),
        ),
        (
            "/system/perception/value",
            exact_metric_key(metric_definitions::actor::PERCEPTION_MOD).to_string(),
        ),
        (
            "/system/attributes/speed/value",
            metric_definitions::actor::speed::value_key("land"),
        ),
    ] {
        push_number_candidate(&mut candidates, raw, path, MetricDomain::Actor, key);
    }

    if let Some(disable_markup) = raw
        .pointer("/system/details/disable")
        .and_then(Value::as_str)
    {
        for key in crate::records::metrics::disable_metric_candidate_keys(disable_markup) {
            candidates.push(SourceMetricCandidate {
                domain: MetricDomain::Actor,
                key,
                source_path: "/system/details/disable".to_string(),
                value_type: "number",
            });
        }
    }

    if let Some(saves) = raw.pointer("/system/saves").and_then(Value::as_object) {
        for (raw_key, value) in saves {
            let Some(save_key) = normalize_save_key(raw_key) else {
                continue;
            };
            for path in ["/mod", "/modifier", "/value", "/totalModifier"] {
                push_number_candidate(
                    &mut candidates,
                    value,
                    path,
                    MetricDomain::Actor,
                    metric_definitions::actor::save::mod_key(save_key),
                );
            }
        }
    }

    if let Some(skills) = raw.pointer("/system/skills").and_then(Value::as_object) {
        for (raw_key, value) in skills {
            let skill_key = crate::records::metrics::slugify_metric_segment(raw_key);
            if skill_key.is_empty() {
                continue;
            }
            for path in ["/mod", "/modifier", "/value"] {
                push_number_candidate(
                    &mut candidates,
                    value,
                    path,
                    MetricDomain::Actor,
                    metric_definitions::actor::skill::mod_key(&skill_key),
                );
            }
            push_number_candidate(
                &mut candidates,
                value,
                "/rank",
                MetricDomain::Actor,
                metric_definitions::actor::skill::rank_key(&skill_key),
            );
        }
    }

    if let Some(other_speeds) = raw
        .pointer("/system/attributes/speed/otherSpeeds")
        .and_then(Value::as_array)
    {
        for speed in other_speeds {
            let Some(speed_type) = speed
                .pointer("/type")
                .and_then(Value::as_str)
                .map(crate::records::metrics::slugify_metric_segment)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            push_number_candidate(
                &mut candidates,
                speed,
                "/value",
                MetricDomain::Actor,
                metric_definitions::actor::speed::value_key(&speed_type),
            );
        }
    }

    if let Some(senses) = raw
        .pointer("/system/perception/senses")
        .and_then(Value::as_array)
    {
        for sense in senses {
            let Some(sense_type) = sense
                .pointer("/type")
                .and_then(Value::as_str)
                .map(crate::records::metrics::slugify_metric_segment)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            push_number_candidate(
                &mut candidates,
                sense,
                "/range",
                MetricDomain::Actor,
                metric_definitions::actor::sense::range_key(&sense_type),
            );
        }
    }

    candidates
}

fn item_source_metric_candidates(record_type: &str, raw: &Value) -> Vec<SourceMetricCandidate> {
    let mut candidates = Vec::new();
    match crate::records::metrics::slugify_metric_segment(record_type).as_str() {
        "weapon" => {
            for path in [
                "/system/range/increment",
                "/system/range/value",
                "/system/range",
            ] {
                push_number_candidate(
                    &mut candidates,
                    raw,
                    path,
                    MetricDomain::Item,
                    exact_metric_key(metric_definitions::item::weapon::RANGE_INCREMENT).to_string(),
                );
            }
            for path in ["/system/reload/value", "/system/reload"] {
                push_number_candidate(
                    &mut candidates,
                    raw,
                    path,
                    MetricDomain::Item,
                    exact_metric_key(metric_definitions::item::weapon::RELOAD).to_string(),
                );
            }
            push_number_candidate(
                &mut candidates,
                raw,
                "/system/damage/dice",
                MetricDomain::Item,
                exact_metric_key(metric_definitions::item::weapon::DAMAGE_DICE).to_string(),
            );
            push_damage_die_faces_candidate(
                &mut candidates,
                raw,
                "/system/damage/die",
                MetricDomain::Item,
                exact_metric_key(metric_definitions::item::weapon::DAMAGE_DIE_FACES).to_string(),
            );
        }
        "armor" => {
            for (path, definition) in [
                ("/system/acBonus", metric_definitions::item::armor::AC_BONUS),
                ("/system/dexCap", metric_definitions::item::armor::DEX_CAP),
                (
                    "/system/strength",
                    metric_definitions::item::armor::STRENGTH,
                ),
                (
                    "/system/checkPenalty",
                    metric_definitions::item::armor::CHECK_PENALTY,
                ),
                (
                    "/system/speedPenalty",
                    metric_definitions::item::armor::SPEED_PENALTY,
                ),
            ] {
                push_number_candidate(
                    &mut candidates,
                    raw,
                    path,
                    MetricDomain::Item,
                    exact_metric_key(definition).to_string(),
                );
            }
        }
        "shield" => {
            for (path, definition) in [
                (
                    "/system/acBonus",
                    metric_definitions::item::shield::AC_BONUS,
                ),
                (
                    "/system/hardness",
                    metric_definitions::item::shield::HARDNESS,
                ),
                ("/system/hp/value", metric_definitions::item::shield::HP),
                ("/system/hp/max", metric_definitions::item::shield::HP),
                (
                    "/system/hp/brokenThreshold",
                    metric_definitions::item::shield::BROKEN_THRESHOLD,
                ),
                (
                    "/system/hp/broken",
                    metric_definitions::item::shield::BROKEN_THRESHOLD,
                ),
                (
                    "/system/hp/bt",
                    metric_definitions::item::shield::BROKEN_THRESHOLD,
                ),
            ] {
                push_number_candidate(
                    &mut candidates,
                    raw,
                    path,
                    MetricDomain::Item,
                    exact_metric_key(definition).to_string(),
                );
            }
        }
        _ => {}
    }
    candidates
}

fn push_number_candidate(
    candidates: &mut Vec<SourceMetricCandidate>,
    raw: &Value,
    pointer: &str,
    domain: MetricDomain,
    key: String,
) {
    if raw
        .pointer(pointer)
        .and_then(crate::records::metrics::number_like_value)
        .is_some()
    {
        candidates.push(SourceMetricCandidate {
            domain,
            key,
            source_path: pointer.to_string(),
            value_type: "number",
        });
    }
}

fn push_damage_die_faces_candidate(
    candidates: &mut Vec<SourceMetricCandidate>,
    raw: &Value,
    pointer: &str,
    domain: MetricDomain,
    key: String,
) {
    if crate::records::metrics::damage_die_faces(raw.pointer(pointer)).is_some() {
        candidates.push(SourceMetricCandidate {
            domain,
            key,
            source_path: pointer.to_string(),
            value_type: "number",
        });
    }
}

fn metric_warning_examples(
    diagnostics: &std::collections::BTreeMap<String, crate::diagnostics::MetricAuditDiagnostic>,
) -> String {
    diagnostics
        .iter()
        .take(5)
        .map(|(key, diagnostic)| {
            format!(
                "{key} count={} examples=[{}]",
                diagnostic.count,
                diagnostic.examples.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn metric_value_type(value: &MetricValue) -> &'static str {
    match value {
        MetricValue::Number(_) => "number",
        MetricValue::Text(_) => "text",
        MetricValue::Boolean(_) => "boolean",
    }
}

fn exact_metric_key(definition: metric_definitions::MetricDefinition) -> &'static str {
    definition
        .exact_key()
        .expect("static metric definition should have an exact key")
}

fn normalize_save_key(value: &str) -> Option<&'static str> {
    match crate::records::metrics::slugify_metric_segment(value).as_str() {
        "fort" | "fortitude" => Some("fort"),
        "ref" | "reflex" => Some("ref"),
        "will" => Some("will"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{
        MetricDomain, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey,
    };
    use atlas_record::{MetricRow, MetricValue, NormalizedRecord};

    use super::*;
    use crate::records::SourceConstructionFacts;

    #[test]
    fn records_unknown_emitted_metric_keys_without_failing() {
        let records = vec![LoadedSourceRecord::new(
            record_with_metric(
                vec![MetricRow {
                    domain: MetricDomain::Actor,
                    key: "custom.future_metric".to_string(),
                    value: MetricValue::Number(1.0),
                }],
                "{}",
            ),
            SourceConstructionFacts {
                content_parse_diagnostics: Vec::new(),
            },
        )];
        let mut diagnostics = IngestDiagnostics::default();
        let mut warnings = Vec::new();

        audit_emitted_metrics(&records, &mut diagnostics, &mut warnings);

        assert_eq!(
            diagnostics
                .unknown_emitted_metrics
                .get("actor.custom.future_metric type=number")
                .map(|diagnostic| diagnostic.count),
            Some(1)
        );
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn records_source_metric_candidates_that_were_not_emitted() {
        let records = vec![LoadedSourceRecord::new(
            record_with_metric(Vec::new(), r#"{"system":{"abilities":{"str":{"mod":4}}}}"#),
            SourceConstructionFacts {
                content_parse_diagnostics: Vec::new(),
            },
        )];
        let mut diagnostics = IngestDiagnostics::default();
        let mut warnings = Vec::new();

        audit_emitted_metrics(&records, &mut diagnostics, &mut warnings);

        assert_eq!(
            diagnostics
                .unemitted_source_metric_candidates
                .get("actor.ability.str.mod@/system/abilities/str/mod type=number")
                .map(|diagnostic| diagnostic.count),
            Some(1)
        );
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn records_disable_source_metric_candidates_that_were_not_emitted() {
        let records = vec![LoadedSourceRecord::new(
            record_with_metric(
                Vec::new(),
                r#"{"system":{"details":{"disable":"@Check[thievery|dc:27] (expert) to disable the lock"}}}"#,
            ),
            SourceConstructionFacts {
                content_parse_diagnostics: Vec::new(),
            },
        )];
        let mut diagnostics = IngestDiagnostics::default();
        let mut warnings = Vec::new();

        audit_emitted_metrics(&records, &mut diagnostics, &mut warnings);

        assert_eq!(
            diagnostics
                .unemitted_source_metric_candidates
                .get("actor.disable.dc.min@/system/details/disable type=number")
                .map(|diagnostic| diagnostic.count),
            Some(1)
        );
        assert_eq!(
            diagnostics
                .unemitted_source_metric_candidates
                .get("actor.disable.thievery.rank.min@/system/details/disable type=number")
                .map(|diagnostic| diagnostic.count),
            Some(1)
        );
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("type=number"));
        assert!(warnings[0].contains("examples=[actions:test]"));
    }

    #[test]
    fn ignores_unparseable_weapon_damage_die_candidates() {
        let mut record = record_with_metric(Vec::new(), r#"{"system":{"damage":{"die":""}}}"#);
        record.foundry_document_type = "Item".to_string();
        record.foundry_record_type = "weapon".to_string();
        let records = vec![LoadedSourceRecord::new(
            record,
            SourceConstructionFacts {
                content_parse_diagnostics: Vec::new(),
            },
        )];
        let mut diagnostics = IngestDiagnostics::default();
        let mut warnings = Vec::new();

        audit_emitted_metrics(&records, &mut diagnostics, &mut warnings);

        assert!(
            diagnostics
                .unemitted_source_metric_candidates
                .keys()
                .all(|key| !key.contains("weapon.damage_die_faces"))
        );
        assert!(warnings.is_empty());
    }

    fn record_with_metric(metrics: Vec<MetricRow>, raw_json: &str) -> NormalizedRecord {
        NormalizedRecord {
            key: RecordKey::parse("actions:test").expect("record key should parse"),
            id: RecordId::new("test".to_string()).expect("record id should parse"),
            name: "Test".to_string(),
            normalized_name: "test".to_string(),
            record_family: RecordFamily::Rule,
            pack_name: PackName::new("actions".to_string()).expect("pack name should parse"),
            pack_label: "Actions".to_string(),
            foundry_document_type: "Actor".to_string(),
            foundry_record_type: "npc".to_string(),
            level: None,
            rarity: None,
            traits: Vec::new(),
            system_category: None,
            system_group: None,
            system_base_item: None,
            system_usage: None,
            system_price_json: None,
            system_actions_value: None,
            system_time_value: None,
            system_duration_value: None,
            price_cp: None,
            activation_time: None,
            duration: None,
            metrics,
            actor_data: None,
            item_data: None,
            spell_data: None,
            publication_title: None,
            publication_remaster: false,
            description: None,
            blurb: None,
            supplemental_content: Vec::new(),
            publication_family: PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: Vec::new(),
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "none".to_string(),
            source_path: "packs/actions/test.json".to_string(),
            is_default_visible: true,
            raw_json: raw_json.to_string(),
        }
    }
}
