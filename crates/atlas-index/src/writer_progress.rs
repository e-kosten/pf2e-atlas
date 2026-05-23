use std::time::Instant;

pub(crate) fn elapsed_display(started_at: Instant) -> String {
    format_duration_ms(started_at.elapsed().as_millis())
}

fn format_duration_ms(duration_ms: u128) -> String {
    let total_seconds = duration_ms / 1000;
    let millis = duration_ms % 1000;
    if total_seconds < 1 {
        return format!("{duration_ms}ms");
    }

    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    if total_minutes < 1 {
        return format!("{seconds}.{millis:03}s");
    }

    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;
    if hours == 0 {
        format!("{minutes}m {seconds:02}.{millis:03}s")
    } else {
        format!("{hours}h {minutes:02}m {seconds:02}.{millis:03}s")
    }
}
