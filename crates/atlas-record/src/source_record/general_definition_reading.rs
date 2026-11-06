//! Exact-source generic template selection; customized/owned actions read normally.
use super::{
    ItemSourceView, SourceBackedRecord, SourceContentOutcome, SourceContentStatus, SourceNodeView,
    SourceReferenceIndex,
};
use crate::source_content::{
    self, ContentAudience, ContentReferenceResolver, GeneralDefinitionReading,
    LocalizationResolver, PreparedSourceContent,
};
use atlas_foundry_model::{SourceValue, generated::general_definition_templates};
use sha2::{Digest, Sha256};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// This is serialization of the existing model-owned authored-value reconstruction,
// not serialization of tagged DTOs or another source admission/parser.
fn canonical(value: SourceValue) -> Option<serde_json::Value> {
    Some(match value {
        SourceValue::Null => serde_json::Value::Null,
        SourceValue::Boolean(x) => x.into(),
        SourceValue::Number(x) => x.into(),
        SourceValue::String(x) => x.into(),
        SourceValue::Array(x) => {
            serde_json::Value::Array(x.into_iter().map(canonical).collect::<Option<_>>()?)
        }
        SourceValue::Object(x) => {
            let mut object = std::collections::BTreeMap::new();
            for (key, value) in x.fields() {
                if object
                    .insert(key.clone(), canonical(value.clone())?)
                    .is_some()
                {
                    return None;
                }
            }
            serde_json::Value::Object(object.into_iter().collect())
        }
    })
}
fn checked_origin(record: &SourceBackedRecord, package: &str) -> Option<GeneralDefinitionReading> {
    let template = general_definition_templates()
        .iter()
        .find(|t| t.record_key == record.key().to_string() && t.source_package == package)?;
    let SourceNodeView::Item(ItemSourceView::AbilitySource(item)) = record.node_at(&[])? else {
        return None;
    };
    let system = item.system.as_value()?;
    let description = system.description.as_value()?.value.as_value()?;
    if hash(description.as_bytes()) != template.description_sha256 {
        return None;
    }
    let rules = system.rules.as_value()?;
    let rules = rules
        .iter()
        .map(|r| canonical(r.authored_value().ok()?))
        .collect::<Option<Vec<_>>>()?;
    let bytes = serde_json::to_vec(&rules).ok()?;
    if hash(&bytes) != template.rules_sha256 {
        return None;
    }
    // Whole-rule equality includes rule-only parameters, including Aura radius.
    for parameter in template.parameter_roles {
        if let Some(expected) = parameter.expected_value_json {
            let pointer = parameter.source_pointer.strip_prefix("/system/rules")?;
            let actual = serde_json::Value::Array(rules.clone());
            if serde_json::to_string(actual.pointer(pointer)?)
                .ok()?
                .as_str()
                != expected
            {
                return None;
            }
        }
    }
    source_content::general_definition_origin(record.key(), description, template)
}

/// Upgrade only the matching canonical field through the ordinary interpreter.
/// A stale selection is removed by ordinary preparation when policy/source changes.
pub fn bind_general_definition_reading(
    record: &SourceBackedRecord,
    source_package: &str,
    audience: ContentAudience,
    localization: Option<&dyn LocalizationResolver>,
    references: Option<&SourceReferenceIndex>,
    content: &mut [SourceContentOutcome],
) -> Result<(), html2text::Error> {
    let origin = checked_origin(record, source_package);
    for outcome in content {
        let SourceContentStatus::Prepared(prepared) = &mut outcome.status else {
            continue;
        };
        let eligible = prepared.locator.owners.is_empty()
            && prepared.locator.field == "/system/description/value";
        let reading = eligible.then(|| origin.clone()).flatten();
        if reading.is_none() && prepared.general_definition_reading.is_none() {
            continue;
        }
        let super::SourceFieldView::Value(markup) = record.authored_content_at(&prepared.locator)
        else {
            continue;
        };
        let formula = record
            .node_at(&prepared.locator.owners)
            .map(super::source_formula_labels)
            .unwrap_or_default();
        let selection = reading
            .as_ref()
            .filter(|_| localization.is_some())
            .map(|r| {
                (
                    r.localization_node_path.as_str(),
                    r.selection_marker.as_str(),
                )
            });
        let mut replacement = source_content::prepare_source_content_with_selection(
            prepared.locator.clone(),
            markup,
            audience,
            outcome.visibility,
            localization,
            references.map(|r| r as &dyn ContentReferenceResolver),
            &formula,
            selection,
        )?;
        if selection.is_some() {
            replacement.general_definition_reading = reading
                .and_then(|r| source_content::finish_general_definition_reading(&replacement, r));
            // Binding failure must restore the ordinary body, including no orphan marker.
            if replacement.general_definition_reading.is_none() {
                replacement = source_content::prepare_source_content(
                    prepared.locator.clone(),
                    markup,
                    audience,
                    outcome.visibility,
                    localization,
                    references.map(|r| r as &dyn ContentReferenceResolver),
                    &formula,
                )?;
            }
        }
        **prepared = replacement;
    }
    Ok(())
}

/// Validate retained bindings with the admitted source and canonical prepared facts;
/// no Foundry files, localization acquisition or runtime inference is required.
pub fn validate_general_definition_reading(
    record: &SourceBackedRecord,
    source_package: &str,
    prepared: &PreparedSourceContent,
) -> Result<(), String> {
    let Some(actual) = &prepared.general_definition_reading else {
        if source_content::has_general_definition_marker(&prepared.html) {
            return Err("unbound general-definition marker".into());
        }
        return Ok(());
    };
    if prepared.locator.record != *record.key()
        || !prepared.locator.owners.is_empty()
        || prepared.locator.field != "/system/description/value"
    {
        return Err("general-definition field identity mismatch".into());
    }
    let origin = checked_origin(record, source_package)
        .ok_or("general-definition source signature mismatch")?;
    if prepared.authored_markup_sha256 != origin.authored_description_sha256 {
        return Err("general-definition authored hash mismatch".into());
    }
    let expected = source_content::finish_general_definition_reading(prepared, origin)
        .ok_or("general-definition marker binding mismatch")?;
    if *actual != expected {
        return Err("general-definition role binding mismatch".into());
    }
    source_content::validate_general_definition_markers(prepared, actual)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_content::{ContentVisibilityRule, LocalizationResolver};
    use atlas_foundry_model::{SourceContext, admit_document_source};
    use serde_json::{Value, json};
    struct Prose;
    impl LocalizationResolver for Prose {
        fn localized_value(&self, _: &str) -> Option<&str> {
            Some(
                "<p>Reusable prose @UUID[Compendium.pf2e.test.Item.aaaaaaaaaaaaaaaa] @Damage[1d4] <span data-visibility=\"gm\">@Check[fortitude|dc:18]</span>.</p>",
            )
        }
    }
    fn audience(gm: bool) -> ContentAudience {
        ContentAudience {
            include_gm: gm,
            include_owner: gm,
            implicit_check_dc: ContentVisibilityRule::All,
        }
    }
    fn fixtures() -> Vec<Value> {
        serde_json::from_str::<Value>(include_str!("fixtures/general-definition-templates.json"))
            .unwrap()["items"]
            .as_array()
            .unwrap()
            .clone()
    }
    fn record(value: Value, kind: &str, pack: &str) -> SourceBackedRecord {
        let admitted = admit_document_source(
            kind,
            SourceContext::new("template", "fixture.json", "$"),
            &serde_json::to_vec(&value).unwrap(),
        )
        .unwrap()
        .model
        .unwrap();
        SourceBackedRecord::new(pack, admitted).unwrap()
    }
    fn template_record(item: Value) -> SourceBackedRecord {
        let id = item["_id"].as_str().unwrap();
        let template = general_definition_templates()
            .iter()
            .find(|t| t.record_key.rsplit_once(':').unwrap().1 == id)
            .unwrap();
        record(item, "Item", template.record_key.split_once(':').unwrap().0)
    }
    fn prepare(record: &SourceBackedRecord, gm: bool) -> Vec<SourceContentOutcome> {
        let mut content =
            super::super::prepare_record_content(record, audience(gm), Some(&Prose), None);
        bind_general_definition_reading(
            record,
            "pf2e",
            audience(gm),
            Some(&Prose),
            None,
            &mut content,
        )
        .unwrap();
        content
    }
    fn body(content: &[SourceContentOutcome]) -> &PreparedSourceContent {
        content
            .iter()
            .find_map(|c| match &c.status {
                SourceContentStatus::Prepared(p)
                    if p.locator.field == "/system/description/value" =>
                {
                    Some(p.as_ref())
                }
                _ => None,
            })
            .unwrap()
    }
    #[test]
    fn all_ten_exact_signatures_bind_roles_to_actual_controls_and_reusable_prose() {
        let items = fixtures();
        assert_eq!(items.len(), 10);
        for item in items {
            let record = template_record(item);
            for gm in [false, true] {
                let content = prepare(&record, gm);
                let prepared = body(&content);
                let reading = prepared
                    .general_definition_reading
                    .as_ref()
                    .unwrap_or_else(|| panic!("missing {}", record.key()));
                validate_general_definition_reading(&record, "pf2e", prepared).unwrap();
                assert_eq!(reading.reference_ordinals.len(), 1);
                assert_eq!(reading.interaction_ordinals.len(), if gm { 2 } else { 1 });
                assert!(reading.parameters.iter().all(
                    |p| p.source_span.is_some() || p.source_pointer == "/system/rules/0/radius"
                ));
                assert!(
                    reading
                        .parameters
                        .iter()
                        .any(|p| p.source_node_path.is_some())
                );
                assert!(!reading.excluded_parameter_interaction_ordinals.is_empty());
                assert!(prepared.text.contains("Reusable prose"));
                assert_eq!(reading.template_key, *record.key());
                let mut corrupt = prepared.clone();
                corrupt
                    .general_definition_reading
                    .as_mut()
                    .unwrap()
                    .interaction_ordinals
                    .clear();
                assert!(validate_general_definition_reading(&record, "pf2e", &corrupt).is_err());
                let mut corrupt = prepared.clone();
                corrupt.html = corrupt
                    .html
                    .replace("data-atlas-definition-selection", "data-removed");
                assert!(validate_general_definition_reading(&record, "pf2e", &corrupt).is_err());
            }
        }
    }
    #[test]
    fn all_ten_real_localized_bodies_keep_a_valid_single_canonical_selection() {
        struct English(std::collections::BTreeMap<String, String>);
        impl LocalizationResolver for English {
            fn localized_value(&self, key: &str) -> Option<&str> {
                self.0.get(key).map(String::as_str)
            }
        }
        let fixture: Value =
            serde_json::from_str(include_str!("fixtures/general-definition-templates.json"))
                .unwrap();
        let english = English(serde_json::from_value(fixture["localizations"].clone()).unwrap());
        for item in fixtures() {
            let record = template_record(item);
            let mut content =
                super::super::prepare_record_content(&record, audience(true), Some(&english), None);
            bind_general_definition_reading(
                &record,
                "pf2e",
                audience(true),
                Some(&english),
                None,
                &mut content,
            )
            .unwrap();
            let prepared = body(&content);
            assert!(
                prepared.general_definition_reading.is_some(),
                "{}",
                record.key()
            );
            assert!(!prepared.text.contains("@Localize["));
            let document = scraper::Html::parse_fragment(&prepared.html);
            let selector = scraper::Selector::parse("[data-atlas-definition-selection]").unwrap();
            let selected = document.select(&selector).next().unwrap();
            let selected_html = selected.inner_html();
            let selected_text = selected.text().collect::<String>();
            if record.key().to_string().ends_with("r34QDwKiWZoVymJa") {
                assert!(selected_text.contains("2d6 rounds"));
                assert!(!selected_html.contains("0d8"));
                assert!(!selected_html.contains("0d4"));
            }
            if record.key().to_string().ends_with("xS8ybzuqPSi3Jb8k") {
                assert!(selected_text.contains("usually 24 hours"));
                assert!(!selected_html.contains("dc=0"));
                assert!(
                    prepared
                        .general_definition_reading
                        .as_ref()
                        .unwrap()
                        .interaction_ordinals
                        .is_empty()
                );
            }
            validate_general_definition_reading(&record, "pf2e", prepared).unwrap();
        }
    }
    #[test]
    fn changed_whole_description_or_rules_and_wrong_package_read_normally() {
        for item in fixtures() {
            for rules in [false, true] {
                let mut changed = item.clone();
                if rules {
                    changed["system"]["rules"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"key":"RollOption","domain":"all","option":"changed"}));
                } else {
                    let desc = changed["system"]["description"]["value"]
                        .as_str()
                        .unwrap()
                        .to_owned();
                    changed["system"]["description"]["value"] =
                        format!("{desc}<p>Actual 22 DC and 5-foot rider.</p>").into();
                }
                let record = template_record(changed);
                let content = prepare(&record, true);
                let prepared = body(&content);
                assert!(prepared.general_definition_reading.is_none());
                assert!(!prepared.html.contains("data-atlas-definition-selection"));
                validate_general_definition_reading(&record, "pf2e", prepared).unwrap();
            }
            let record = template_record(item);
            let mut content = prepare(&record, true);
            assert!(body(&content).general_definition_reading.is_some());
            bind_general_definition_reading(
                &record,
                "custom",
                audience(true),
                Some(&Prose),
                None,
                &mut content,
            )
            .unwrap();
            assert!(body(&content).general_definition_reading.is_none());
            assert!(
                !body(&content)
                    .html
                    .contains("data-atlas-definition-selection")
            );
        }
    }
    #[test]
    fn owned_template_actual_values_and_real_zero_are_never_suppressed() {
        let mut owned = fixtures().remove(2);
        let desc = owned["system"]["description"]["value"]
            .as_str()
            .unwrap()
            .replace("dc:|", "dc:22|");
        owned["system"]["description"]["value"] =
            format!("{desc}<p>5-foot rider @Damage[0].</p>").into();
        let actor = record(
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[owned]}),
            "Actor",
            "test",
        );
        let content = prepare(&actor, true);
        let prepared = body(&content);
        assert!(!prepared.locator.owners.is_empty());
        assert!(prepared.general_definition_reading.is_none());
        assert!(prepared.text.contains("22"));
        assert!(prepared.text.contains("5-foot rider"));
        assert!(!prepared.interactions.is_empty());
        let zero = record(
            json!({"_id":"bbbbbbbbbbbbbbbb","type":"action","system":{"description":{"value":"<p>@Check[fortitude|dc:0] @Damage[0]</p>"},"rules":[]}}),
            "Item",
            "test",
        );
        let content = prepare(&zero, true);
        let prepared = body(&content);
        assert!(prepared.general_definition_reading.is_none());
        assert_eq!(prepared.interactions.len(), 2);
        assert!(prepared.text.contains('0'));
        assert!(
            !serde_json::to_string(prepared)
                .unwrap()
                .contains("general_definition_reading")
        );
    }
    #[test]
    fn stale_cached_selection_is_removed_for_same_key_customization() {
        let original = fixtures().remove(2);
        let source = record(original.clone(), "Item", "bestiary-ability-glossary-srd");
        let mut cached = prepare(&source, true);
        let mut custom = original;
        custom["system"]["description"]["value"] =
            "<p>Actual custom ability @Check[reflex|dc:22] with a 5-foot rider.</p>".into();
        let source = record(custom, "Item", "bestiary-ability-glossary-srd");
        bind_general_definition_reading(
            &source,
            "pf2e",
            audience(true),
            Some(&Prose),
            None,
            &mut cached,
        )
        .unwrap();
        assert!(body(&cached).general_definition_reading.is_none());
        assert!(body(&cached).text.contains("22"));
        assert!(!body(&cached).html.contains("definition-selection"));
    }
}
