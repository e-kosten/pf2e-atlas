use atlas_domain::{TimeKind, TimeUnit};
use atlas_record::NormalizedTime;

use crate::source::normalize::normalize_text;

pub(crate) fn normalize_activation_time(
    system_actions_value: Option<i64>,
    system_time_value: Option<&str>,
) -> Option<NormalizedTime> {
    if let Some(actions) = system_actions_value {
        return Some(NormalizedTime {
            kind: TimeKind::Actions,
            actions: Some(actions),
            duration_value: None,
            duration_unit: None,
            text: actions.to_string(),
        });
    }
    system_time_value.and_then(normalize_time_text)
}

pub(crate) fn normalize_time_text(value: &str) -> Option<NormalizedTime> {
    let text = value.trim();
    if text.is_empty() {
        return None;
    }

    let normalized = normalize_text(text);
    if let Ok(actions) = normalized.parse::<i64>() {
        return Some(NormalizedTime {
            kind: TimeKind::Actions,
            actions: Some(actions),
            duration_value: None,
            duration_unit: None,
            text: text.to_string(),
        });
    }

    match normalized.as_str() {
        "free" | "free action" => {
            return Some(time_with_kind(TimeKind::Free, text));
        }
        "reaction" => {
            return Some(time_with_kind(TimeKind::Reaction, text));
        }
        _ => {}
    }

    if let Some((duration_value, duration_unit)) = parse_duration_unit(&normalized) {
        return Some(NormalizedTime {
            kind: TimeKind::Duration,
            actions: None,
            duration_value: Some(duration_value),
            duration_unit: Some(duration_unit),
            text: text.to_string(),
        });
    }

    let kind = if normalized.contains(" to ")
        || normalized.contains(" or ")
        || normalized.contains("varies")
        || normalized.contains("variable")
    {
        TimeKind::Variable
    } else {
        TimeKind::Other
    };
    Some(time_with_kind(kind, text))
}

fn time_with_kind(kind: TimeKind, text: &str) -> NormalizedTime {
    NormalizedTime {
        kind,
        actions: None,
        duration_value: None,
        duration_unit: None,
        text: text.to_string(),
    }
}

fn parse_duration_unit(value: &str) -> Option<(i64, TimeUnit)> {
    let mut parts = value.split_whitespace();
    let amount = parts.next()?.parse::<i64>().ok()?;
    let unit = canonical_time_unit(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((amount, unit))
}

fn canonical_time_unit(value: &str) -> Option<TimeUnit> {
    match value.trim_end_matches('s') {
        "round" => Some(TimeUnit::Round),
        "minute" => Some(TimeUnit::Minute),
        "hour" => Some(TimeUnit::Hour),
        "day" => Some(TimeUnit::Day),
        "week" => Some(TimeUnit::Week),
        "month" => Some(TimeUnit::Month),
        "year" => Some(TimeUnit::Year),
        _ => None,
    }
}
