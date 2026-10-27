//! Readable checks and areas from retained authored parameters.
use std::collections::BTreeMap;

/// A noun immediately outside a macro belongs to its authored sentence. Move it
/// into the derived label so an opposing defense follows the noun, not before it.
pub(super) fn check_noun(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_start();
    if trimmed.len() == text.len() {
        return None;
    }
    for noun in [
        "saving throws",
        "saving throw",
        "checks",
        "check",
        "saves",
        "save",
    ] {
        if let Some(tail) = trimmed.get(noun.len()..)
            && trimmed[..noun.len()].eq_ignore_ascii_case(noun)
            && !tail.starts_with(char::is_alphanumeric)
        {
            return Some((&trimmed[..noun.len()], tail));
        }
    }
    None
}

pub(super) fn damage_noun(text: &str) -> Option<&str> {
    let trimmed = text.trim_start();
    let tail = trimmed.get(6..)?;
    (trimmed.len() != text.len()
        && trimmed[..6].eq_ignore_ascii_case("damage")
        && !tail.starts_with(char::is_alphanumeric))
    .then_some(tail)
}

/// Absorb only a source clause that exactly repeats the generated symbolic DC.
pub(super) fn redundant_dc_clause<'a>(
    text: &'a str,
    dc: &str,
    context: &super::FormulaContext,
) -> Option<&'a str> {
    let super::contextual_formula::DcText::Rule(rule) =
        super::contextual_formula::contextual_dc(dc, "0", context)?
    else {
        return None;
    };
    let tail = text.strip_prefix(&format!(" {rule}"))?;
    (!tail.starts_with(char::is_alphanumeric)).then_some(tail)
}

/// The surrounding authored unit already names untyped healing. Mixed or
/// typed pools keep their labels so this never discards mechanical information.
pub(super) fn authored_healing_points(formula: &str, text: &str) -> bool {
    let text = text.trim_start();
    if !text
        .get(..10)
        .is_some_and(|word| word.eq_ignore_ascii_case("hit points"))
        || text
            .get(10..)
            .is_some_and(|tail| tail.starts_with(char::is_alphanumeric))
    {
        return false;
    }
    let Some(parts) = super::literal_damage::damage_instances(formula) else {
        return false;
    };
    let [part] = parts.as_slice() else {
        return false;
    };
    let Some((_, flavor)) = part
        .strip_suffix(']')
        .and_then(|part| part.rsplit_once('['))
    else {
        return false;
    };
    super::literal_damage(&format!("1[{flavor}]"))
        .is_some_and(|parts| parts.len() == 1 && parts[0].label == "healing")
}

pub(super) fn check_text(
    statistic: &str,
    options: &BTreeMap<String, String>,
    authored_noun: Option<&str>,
    already_secret: bool,
    context: &super::FormulaContext,
) -> String {
    let adjustments: Vec<_> = options
        .get("adjustment")
        .map(|value| value.split(',').map(str::trim).collect())
        .unwrap_or_default();
    let secret = ["traits", "overrideTraits"].into_iter().any(|key| {
        options
            .get(key)
            .is_some_and(|value| value.split(',').any(|part| part.trim() == "secret"))
    });
    statistic
        .split(',')
        .enumerate()
        .map(|(index, statistic)| {
            let statistic = statistic.trim();
            let noun = authored_noun.unwrap_or(match statistic {
                "fortitude" | "reflex" | "will" => "save",
                _ => "check",
            });
            let name = if statistic == "flat" {
                "flat".into()
            } else {
                display_name(statistic)
            };
            let mut text = format!("{name} {noun}");
            if options.contains_key("basic") {
                text = format!("basic {text}");
            }
            if secret && !already_secret {
                text = format!("secret {text}");
            }
            if let Some(dc) = options
                .get("dc")
                .map(|dc| dc.trim())
                .filter(|dc| !dc.is_empty())
            {
                let adjustment = adjustments.get(index).copied().unwrap_or("0");
                text = match super::contextual_formula::contextual_dc(dc, adjustment, context) {
                    Some(super::contextual_formula::DcText::Number(dc)) => {
                        format!("DC {dc} {text}")
                    }
                    Some(super::contextual_formula::DcText::Rule(rule)) => format!("{text} {rule}"),
                    None => format!("DC {dc} {text}"),
                };
            }
            if let Some(defense) = options.get("against").or_else(|| options.get("defense")) {
                text.push_str(&format!(" vs {}", defense_name(defense)));
            }
            text
        })
        .collect::<Vec<_>>()
        .join(" or ")
}

fn display_name(slug: &str) -> String {
    // Only word formatting: custom statistic names and contextual expressions
    // acquire no inferred mechanics or runtime values.
    if slug.contains('@') || slug.contains("resolve(") {
        return slug.into();
    }
    slug.split('-')
        .flat_map(str::split_whitespace)
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn defense_name(defense: &str) -> String {
    match defense.trim() {
        "ac" | "armor" => "AC".into(),
        "class-spell" | "spell-class" | "class-or-spell" => "class or spell DC".into(),
        "class" => "class DC".into(),
        "spell" => "spell DC".into(),
        defense => format!("{} DC", display_name(defense)),
    }
}

pub(super) fn template_text(
    shape: Option<&str>,
    options: &BTreeMap<String, String>,
    preceding: Option<&str>,
) -> Option<String> {
    let shape = shape?;
    let distance = options.get("distance")?;
    if !matches!(
        shape,
        "burst" | "cone" | "cube" | "cylinder" | "emanation" | "line" | "square"
    ) || !distance.parse::<f64>().is_ok_and(f64::is_finite)
        || !options
            .get("width")
            .is_none_or(|width| width.parse::<f64>().is_ok_and(f64::is_finite))
    {
        return None;
    }
    let mut text = format!("{distance}-foot {shape}");
    if let Some(width) = options.get("width")
        && !preceding.is_some_and(|text| authored_width(text, width))
    {
        let unit = if width.parse::<f64>().ok() == Some(1.0) {
            "foot"
        } else {
            "feet"
        };
        text.push_str(&format!(" ({width} {unit} wide)"));
    }
    Some(text)
}

fn authored_width(text: &str, width: &str) -> bool {
    let text = text.trim_end().trim_end_matches(',').trim_end();
    [
        format!("{width}-foot-wide"),
        format!("{width} feet wide"),
        format!("{width} foot wide"),
    ]
    .iter()
    .any(|suffix| {
        text.strip_suffix(suffix)
            .is_some_and(|prefix| !prefix.ends_with(char::is_alphanumeric))
    })
}
