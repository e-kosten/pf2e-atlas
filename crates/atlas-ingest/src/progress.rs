use std::time::{Duration, Instant};

pub(crate) fn elapsed_display(started_at: Instant) -> String {
    format_duration(started_at.elapsed())
}

pub fn format_duration_ms(duration_ms: u128) -> String {
    let millis = u64::try_from(duration_ms).unwrap_or(u64::MAX);
    format_duration(Duration::from_millis(millis))
}

fn format_duration(duration: Duration) -> String {
    let total_millis = duration.as_millis();
    if total_millis < 1_000 {
        return format!("{total_millis}ms");
    }

    let total_seconds = duration.as_secs();
    let millis = duration.subsec_millis();
    if total_seconds < 60 {
        return format!("{}.{:01}s", total_seconds, millis / 100);
    }

    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    if minutes < 60 {
        return format!("{minutes}m {seconds:02}.{:01}s", millis / 100);
    }

    let hours = minutes / 60;
    let minutes = minutes % 60;
    format!("{hours}h {minutes:02}m {seconds:02}s")
}

#[cfg(test)]
mod tests {
    use super::{format_duration, format_duration_ms};
    use std::time::Duration;

    #[test]
    fn formats_elapsed_durations_for_progress_logs() {
        assert_eq!(format_duration(Duration::from_millis(438)), "438ms");
        assert_eq!(format_duration_ms(438), "438ms");
        assert_eq!(format_duration(Duration::from_millis(1_617)), "1.6s");
        assert_eq!(format_duration(Duration::from_millis(152_738)), "2m 32.7s");
        assert_eq!(
            format_duration(Duration::from_millis(3_723_400)),
            "1h 02m 03s"
        );
    }
}
