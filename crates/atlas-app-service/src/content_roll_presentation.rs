//! Reader presentation for checked literal rolls; authored captions stay evidence.
use atlas_app_model::{ContentRollComponentView, ContentRollPresentationView, RollUnitView};
use atlas_record::source_content::{LiteralContentRoll, LiteralDamageComponent, LiteralDamageKind};

pub(crate) fn damage_presentation(
    parts: Vec<LiteralDamageComponent>,
) -> ContentRollPresentationView {
    let heading = if parts.len() == 1 {
        sentence_start(&parts[0].label)
    } else if parts
        .iter()
        .all(|part| part.kind == LiteralDamageKind::Damage)
    {
        "Damage".into()
    } else if parts
        .iter()
        .all(|part| part.kind == LiteralDamageKind::Healing)
    {
        "Healing".into()
    } else {
        "Damage and healing".into()
    };
    ContentRollPresentationView {
        heading,
        components: parts
            .into_iter()
            .map(|part| ContentRollComponentView {
                formula: part.formula,
                label: part.label,
                expression: part.expression,
                unit: None,
            })
            .collect(),
    }
}

pub(crate) fn command_presentation(part: LiteralContentRoll) -> ContentRollPresentationView {
    // Only exact authored English time-unit captions acquire a unit role. Other
    // captions remain verbatim purposes; a d20 formula never implies a check.
    let unit = match part.caption.to_ascii_lowercase().as_str() {
        "second" | "seconds" => Some(("second", "seconds")),
        "round" | "rounds" => Some(("round", "rounds")),
        "minute" | "minutes" => Some(("minute", "minutes")),
        "hour" | "hours" => Some(("hour", "hours")),
        "day" | "days" => Some(("day", "days")),
        "week" | "weeks" => Some(("week", "weeks")),
        "month" | "months" => Some(("month", "months")),
        "year" | "years" => Some(("year", "years")),
        _ => None,
    }
    .map(|(singular, plural)| RollUnitView {
        singular: singular.into(),
        plural: plural.into(),
    });
    let heading = if unit.is_some() {
        "Duration".into()
    } else if part.caption.is_empty() {
        part.formula.clone()
    } else {
        part.caption.clone()
    };
    ContentRollPresentationView {
        heading,
        components: vec![ContentRollComponentView {
            formula: part.formula,
            label: part.caption,
            expression: part.expression,
            unit,
        }],
    }
}

fn sentence_start(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_record::source_content::{literal_content_roll, literal_damage};

    #[test]
    fn authored_purposes_time_units_and_unlabelled_formulas_have_distinct_presentations() {
        for (caption, heading, expected_unit) in [
            ("Counteract", "Counteract", None),
            ("Counteract Check", "Counteract Check", None),
            ("Recharge Breath Weapon", "Recharge Breath Weapon", None),
            ("hours", "Duration", Some(("hour", "hours"))),
            ("Rounds", "Duration", Some(("round", "rounds"))),
            ("Fatigued duration", "Fatigued duration", None),
            ("", "1d20+32", None),
        ] {
            let raw = format!("1d20+32 #{caption}");
            let view = command_presentation(literal_content_roll("r", &raw).unwrap());
            assert_eq!(view.heading, heading);
            assert_eq!(view.components.len(), 1);
            assert_eq!(view.components[0].formula, "1d20+32");
            assert_eq!(view.components[0].label, caption);
            assert_eq!(
                view.components[0]
                    .unit
                    .as_ref()
                    .map(|unit| (unit.singular.as_str(), unit.plural.as_str())),
                expected_unit
            );
        }
    }

    #[test]
    fn damage_heading_preserves_single_type_and_mixed_damage_healing_categories() {
        for (formula, expected, count) in [
            ("1d6[fire]", "Fire damage", 1),
            ("1d6[fire],1d4[cold]", "Damage", 2),
            ("1d6[vitality,healing]", "Vitality healing", 1),
            ("1d6[vitality,healing],1d4[healing]", "Healing", 2),
            ("1d6[fire],1d4[vitality,healing]", "Damage and healing", 2),
            (
                "1d6[vitality,healing,damage],1d4[healing]",
                "Damage and healing",
                2,
            ),
        ] {
            let view = damage_presentation(literal_damage(formula).unwrap());
            assert_eq!(view.heading, expected);
            assert_eq!(view.components.len(), count);
            assert!(view.components.iter().all(|part| part.unit.is_none()));
        }
    }
}
