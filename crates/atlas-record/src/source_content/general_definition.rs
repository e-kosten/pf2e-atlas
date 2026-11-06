//! Checked authored-node origins for the finite generic glossary templates.
use super::parsed_markup::{ParsedMacro, ParsedNode};
use super::parser::parse_foundry_content_with_localization;
use super::{
    GeneralDefinitionParameterBinding, GeneralDefinitionParameterRole as Role,
    GeneralDefinitionReading, PreparedSourceContent, SourceByteRange,
};
use atlas_domain::RecordKey;
use atlas_foundry_model::generated::FoundryGeneralDefinitionTemplate;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};

pub const GENERAL_DEFINITION_READING_RECIPE: &str = "general-definition-reading/v2";
fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

/// Only original authored nodes participate: expanded localization cannot supply
/// a parameter's origin or make an ambiguous authored macro appear unique.
fn authored_macros<'a>(
    nodes: &'a [ParsedNode],
    path: &str,
    out: &mut Vec<(String, &'a ParsedMacro)>,
) {
    for (index, node) in nodes.iter().enumerate() {
        let path = format!("{path}/{index}");
        match node {
            ParsedNode::HtmlElement { children, .. } => authored_macros(children, &path, out),
            ParsedNode::Foundry { node } => out.push((path, node)),
            _ => {}
        }
    }
}
fn role(value: &str) -> Option<Role> {
    Some(match value {
        "check_dc" => Role::CheckDc,
        "damage" => Role::Damage,
        "damage_type" => Role::DamageType,
        "escape_dc" => Role::EscapeDc,
        "rupture_threshold" => Role::RuptureThreshold,
        "range" => Role::Range,
        "size_limit" => Role::SizeLimit,
        "size" => Role::Size,
        "strike" => Role::Strike,
        "aura_radius" => Role::AuraRadius,
        "harm_damage_type" => Role::HarmDamageType,
        "harm_damage" => Role::HarmDamage,
        "area_or_persistent_damage" => Role::AreaOrPersistentDamage,
        "healing_damage_type" => Role::HealingDamageType,
        "healing_amount" => Role::HealingAmount,
        "slowing_damage_type" => Role::SlowingDamageType,
        "winding_duration" => Role::WindingDuration,
        "disable_device_dc" => Role::DisableDeviceDc,
        "standby_mode" => Role::StandbyMode,
        _ => return None,
    })
}

pub(crate) fn general_definition_origin(
    key: &RecordKey,
    markup: &str,
    template: &FoundryGeneralDefinitionTemplate,
) -> Option<GeneralDefinitionReading> {
    let parsed = parse_foundry_content_with_localization(markup, None);
    let mut macros = Vec::new();
    authored_macros(&parsed.document.nodes, "", &mut macros);
    let selected = markup.get(template.localized_start..template.localized_end)?;
    if selected != format!("@Localize[{}]", template.localization_key) {
        return None;
    }
    let paths: Vec<_> = macros
        .iter()
        .filter_map(|(path, node)| match node {
            ParsedMacro::Localize { key, .. } if key == template.localization_key => {
                Some(path.clone())
            }
            _ => None,
        })
        .collect();
    let [localization_node_path] = paths.as_slice() else {
        return None;
    };
    let mut parameters = Vec::new();
    for parameter in template.parameter_roles {
        let (source_span, source_node_path) =
            match (parameter.start, parameter.end, parameter.fragment_sha256) {
                (Some(start), Some(end), Some(expected)) => {
                    let fragment = markup.get(start..end)?;
                    if hash(fragment) != expected {
                        return None;
                    }
                    let fragment_ast = parse_foundry_content_with_localization(fragment, None);
                    let mut fragment_macros = Vec::new();
                    authored_macros(&fragment_ast.document.nodes, "", &mut fragment_macros);
                    let node_path = match fragment_macros.as_slice() {
                        [] => None, // Plain parameter text has no invented control node.
                        [(_, needle)] => {
                            let paths: Vec<_> = macros
                                .iter()
                                .filter(|(_, node)| node == needle)
                                .map(|(path, _)| path.clone())
                                .collect();
                            let [path] = paths.as_slice() else {
                                return None;
                            };
                            Some(path.clone())
                        }
                        _ => return None,
                    };
                    (Some(SourceByteRange { start, end }), node_path)
                }
                (None, None, None) if parameter.expected_value_json.is_some() => (None, None),
                _ => return None,
            };
        parameters.push(GeneralDefinitionParameterBinding {
            role: role(parameter.role)?,
            source_pointer: parameter.source_pointer.into(),
            source_span,
            source_node_path,
            reference_ordinals: Vec::new(),
            interaction_ordinals: Vec::new(),
        });
    }
    let selection_marker = hash(&format!(
        "{GENERAL_DEFINITION_READING_RECIPE}\n{key}\n{}\n{}\n{localization_node_path}",
        template.description_sha256, template.rules_sha256
    ));
    Some(GeneralDefinitionReading {
        template_key: key.clone(),
        source_package: template.source_package.into(),
        recipe: GENERAL_DEFINITION_READING_RECIPE.into(),
        authored_description_sha256: template.description_sha256.into(),
        rules_sha256: template.rules_sha256.into(),
        localization_key: template.localization_key.into(),
        localization_source_span: SourceByteRange {
            start: template.localized_start,
            end: template.localized_end,
        },
        localization_node_path: localization_node_path.clone(),
        selection_marker,
        parameters,
        reference_ordinals: Vec::new(),
        interaction_ordinals: Vec::new(),
        excluded_parameter_interaction_ordinals: Vec::new(),
    })
}
fn within(path: &str, root: &str) -> bool {
    path == root
        || path
            .strip_prefix(root)
            .is_some_and(|tail| tail.starts_with('/'))
}

pub(crate) fn finish_general_definition_reading(
    prepared: &PreparedSourceContent,
    mut reading: GeneralDefinitionReading,
) -> Option<GeneralDefinitionReading> {
    let root = format!("{}/localize", reading.localization_node_path);
    reading.reference_ordinals = prepared
        .references
        .iter()
        .filter(|r| r.visible && within(&r.path, &root))
        .map(|r| r.ordinal)
        .collect();
    reading.interaction_ordinals = prepared
        .interactions
        .iter()
        .filter(|r| within(&r.path, &root))
        .map(|r| r.ordinal)
        .collect();
    for parameter in &mut reading.parameters {
        if let Some(path) = &parameter.source_node_path {
            parameter.reference_ordinals = prepared
                .references
                .iter()
                .filter(|r| r.visible && within(&r.path, path))
                .map(|r| r.ordinal)
                .collect();
            parameter.interaction_ordinals = prepared
                .interactions
                .iter()
                .filter(|r| within(&r.path, path))
                .map(|r| r.ordinal)
                .collect();
        }
    }
    reading.excluded_parameter_interaction_ordinals = reading
        .parameters
        .iter()
        .flat_map(|p| p.interaction_ordinals.iter().copied())
        .collect();
    reading
        .excluded_parameter_interaction_ordinals
        .sort_unstable();
    reading.excluded_parameter_interaction_ordinals.dedup();
    if reading
        .interaction_ordinals
        .iter()
        .any(|x| reading.excluded_parameter_interaction_ordinals.contains(x))
    {
        return None;
    }
    validate_general_definition_markers(prepared, &reading).ok()?;
    Some(reading)
}

/// The same DOM owner that checks canonical reference/control markers checks the
/// selection boundary. Ordinals come from actual projected node paths.
pub(crate) fn validate_general_definition_markers(
    prepared: &PreparedSourceContent,
    reading: &GeneralDefinitionReading,
) -> Result<(), String> {
    let document = Html::parse_fragment(&prepared.html);
    let selector =
        Selector::parse("[data-atlas-definition-selection]").map_err(|e| e.to_string())?;
    let elements: Vec<_> = document.select(&selector).collect();
    let [element] = elements.as_slice() else {
        return Err("expected one general-definition selection marker".into());
    };
    if element.attr("data-atlas-definition-selection") != Some(reading.selection_marker.as_str()) {
        return Err("general-definition selection marker mismatch".into());
    }
    for (attribute, expected) in [
        ("data-atlas-reference", &reading.reference_ordinals),
        ("data-atlas-interaction", &reading.interaction_ordinals),
    ] {
        let selector = Selector::parse(&format!("[{attribute}]")).map_err(|e| e.to_string())?;
        let mut actual: Vec<usize> = element
            .select(&selector)
            .map(|e| {
                e.attr(attribute)
                    .unwrap_or("")
                    .parse()
                    .map_err(|_| "invalid selection ordinal".to_string())
            })
            .collect::<Result<_, _>>()?;
        actual.sort_unstable();
        let mut expected = expected.clone();
        expected.sort_unstable();
        if actual != expected {
            return Err(format!("general-definition {attribute} ordinals mismatch"));
        }
    }
    Ok(())
}

/// Inspect actual attributes, allowing ordinary prose to quote marker names.
pub(crate) fn has_general_definition_marker(html: &str) -> bool {
    let document = Html::parse_fragment(html);
    let selector = Selector::parse("[data-atlas-definition-selection]").expect("fixed selector");
    document.select(&selector).next().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_localization_origin_uses_utf8_bytes_and_rejects_ambiguity() {
        let key: RecordKey = "test:aaaaaaaaaaaaaaaa".parse().unwrap();
        let mut template = FoundryGeneralDefinitionTemplate {
            record_key: "test:aaaaaaaaaaaaaaaa",
            source_package: "pf2e",
            description_sha256: "test",
            rules_sha256: "test",
            localization_key: "Key",
            localized_start: 7,
            localized_end: 21,
            parameter_roles: &[],
        };
        let reading =
            general_definition_origin(&key, "<p>雪 @Localize[Key]</p>", &template).unwrap();
        assert_eq!(
            reading.localization_source_span,
            SourceByteRange { start: 7, end: 21 }
        );
        template.localized_start = 5; // Invalid UTF-8 boundary, never a shifted role.
        assert!(general_definition_origin(&key, "<p>雪 @Localize[Key]</p>", &template).is_none());
        template.localized_start = 7;
        assert!(
            general_definition_origin(&key, "<p>雪 @Localize[Key] @Localize[Key]</p>", &template)
                .is_none()
        );
        assert!(!has_general_definition_marker(
            "<code>data-atlas-definition-selection</code>"
        ));
        assert!(has_general_definition_marker(
            "<span data-atlas-definition-selection=\"x\"></span>"
        ));
    }
}
