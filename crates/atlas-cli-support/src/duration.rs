use std::time::Duration;

/// Human-readable command duration. JSON reports retain their numeric milliseconds.
pub fn format_duration_ms(milliseconds: u128) -> String {
    if milliseconds < 1000 {
        format!("{milliseconds}ms")
    } else if milliseconds < 60_000 {
        format!("{:.1}s", milliseconds as f64 / 1000.0)
    } else {
        format_seconds(milliseconds / 1000)
    }
}

pub(crate) fn format_elapsed(elapsed: Duration) -> String {
    format_seconds(u128::from(elapsed.as_secs()))
}

fn format_seconds(seconds: u128) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_use_readable_units_and_preserve_precision_for_short_runs() {
        for (milliseconds, expected) in [
            (0, "0ms"),
            (125, "125ms"),
            (999, "999ms"),
            (1000, "1.0s"),
            (1200, "1.2s"),
            (60_000, "1m"),
            (61_200, "1m 1s"),
            (3_600_000, "1h"),
            (17_709_200, "4h 55m 9s"),
            (90_061_000, "25h 1m 1s"),
        ] {
            assert_eq!(format_duration_ms(milliseconds), expected);
            if milliseconds >= 60_000 {
                assert_eq!(
                    format_elapsed(Duration::from_millis(milliseconds as u64)),
                    expected
                );
            }
        }
    }
}
