use atlas_domain::QueryLiteral;

pub(super) fn sort_values(path: &str, values: &mut [QueryLiteral]) {
    values.sort_by_cached_key(|value| {
        let text = match value {
            QueryLiteral::String(text) => text.to_lowercase(),
            _ => format!("{value:?}"),
        };
        let duration = (path == "spell.duration_text")
            .then(|| duration_seconds(&text))
            .flatten();
        (duration.is_none(), duration.unwrap_or(0), text)
    });
}

// Presentation ordering only. Conditional and variable text has no numeric
// interpretation here and remains unchanged in the filter's actual value.
fn duration_seconds(text: &str) -> Option<u64> {
    let mut words = text
        .strip_prefix("up to ")
        .unwrap_or(text)
        .split_whitespace();
    let amount: u64 = words.next()?.parse().ok()?;
    let unit = words.next()?.trim_end_matches('s');
    if words.next().is_some() {
        return None;
    }
    let multiplier = match unit {
        "round" => 6,
        "minute" => 60,
        "hour" => 3600,
        "day" => 86400,
        "week" => 604800,
        "month" => 2592000,
        "year" => 31536000,
        _ => return None,
    };
    amount.checked_mul(multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timed_values_precede_conditional_text_without_rewriting_it() {
        let mut values = [
            "1 hour",
            "varies",
            "1 minute or 8 hours",
            "10 minutes",
            "2 rounds",
        ]
        .map(|value| QueryLiteral::String(value.into()));
        sort_values("spell.duration_text", &mut values);
        assert_eq!(
            values,
            [
                "2 rounds",
                "10 minutes",
                "1 hour",
                "1 minute or 8 hours",
                "varies"
            ]
            .map(|value| QueryLiteral::String(value.into()))
        );
    }
}
