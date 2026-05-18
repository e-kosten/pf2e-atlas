use std::fmt::{self, Write as FmtWrite};
use std::io::IsTerminal;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

pub(crate) fn init_tracing() {
    let subscriber = tracing_subscriber::registry()
        .with(CliProgressLayer::new(internal_logs_enabled_from_env()));
    let _ = tracing::subscriber::set_global_default(subscriber);
}

struct CliProgressLayer {
    state: Mutex<CliProgressState>,
    internal_logs_enabled: bool,
}

#[derive(Debug)]
struct CliProgressState {
    is_interactive: bool,
    started_at: Instant,
    progress_bar: Option<ProgressBar>,
    progress_phase: Option<String>,
}

#[derive(Default)]
struct EventFields {
    message: Option<String>,
    phase: Option<String>,
    complete: bool,
    current: Option<u64>,
    total: Option<u64>,
    fields: Vec<(String, String)>,
}

impl CliProgressLayer {
    fn new(internal_logs_enabled: bool) -> Self {
        Self {
            state: Mutex::new(CliProgressState {
                is_interactive: std::io::stderr().is_terminal(),
                started_at: Instant::now(),
                progress_bar: None,
                progress_phase: None,
            }),
            internal_logs_enabled,
        }
    }
}

impl<S> Layer<S> for CliProgressLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        if *event.metadata().level() > Level::INFO {
            return;
        }
        let target = event.metadata().target();
        if !should_render_event(target, self.internal_logs_enabled) {
            return;
        }

        let mut fields = EventFields::default();
        event.record(&mut fields);
        let message = fields
            .message
            .as_deref()
            .map(str::to_string)
            .unwrap_or_else(|| event.metadata().name().to_string());

        let mut state = self.state.lock().expect("progress state is not poisoned");
        if target == "atlas_progress" {
            state.progress(&message, &fields);
        } else {
            state.log(&message, &fields.fields);
        }
    }
}

fn should_render_event(target: &str, internal_logs_enabled: bool) -> bool {
    target == "atlas_progress" || (internal_logs_enabled && target.starts_with("atlas_"))
}

fn internal_logs_enabled_from_env() -> bool {
    std::env::var("ATLAS_LOG")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "info" | "debug" | "trace"))
}

impl CliProgressState {
    fn progress(&mut self, message: &str, fields: &EventFields) {
        if fields.complete {
            self.clear_progress();
            return;
        }
        let Some(total) = fields.total else {
            self.spinner(message, fields.phase.as_ref());
            return;
        };
        let current = fields.current.unwrap_or(0);
        if !self.is_interactive {
            eprintln!(
                "{} INFO {message}",
                elapsed_prefix(self.started_at.elapsed())
            );
            return;
        }

        let phase_changed = fields.phase.as_ref() != self.progress_phase.as_ref();
        if phase_changed {
            if let Some(progress_bar) = self.progress_bar.take() {
                progress_bar.finish_and_clear();
            }
            let progress_bar =
                ProgressBar::with_draw_target(Some(total), ProgressDrawTarget::stderr_with_hz(10));
            progress_bar.set_style(progress_style());
            self.progress_bar = Some(progress_bar);
            self.progress_phase = fields.phase.clone();
        } else if let Some(progress_bar) = &self.progress_bar {
            progress_bar.set_length(total);
        }

        if let Some(progress_bar) = &self.progress_bar {
            progress_bar.set_position(current);
            progress_bar.set_message(message.to_string());
            if current >= total {
                progress_bar.finish_and_clear();
                self.progress_bar = None;
                self.progress_phase = None;
            }
        }
    }

    fn spinner(&mut self, message: &str, phase: Option<&String>) {
        if !self.is_interactive {
            return;
        }

        let phase_changed = phase != self.progress_phase.as_ref();
        if phase_changed {
            self.clear_progress();
            let progress_bar = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr());
            progress_bar.set_style(spinner_style());
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            self.progress_bar = Some(progress_bar);
            self.progress_phase = phase.cloned();
        }

        if let Some(progress_bar) = &self.progress_bar {
            progress_bar.set_message(message.to_string());
        }
    }

    fn clear_progress(&mut self) {
        if let Some(progress_bar) = self.progress_bar.take() {
            progress_bar.finish_and_clear();
        }
        self.progress_phase = None;
    }

    fn log(&mut self, message: &str, fields: &[(String, String)]) {
        let line = format_log_line(self.started_at.elapsed(), message, fields);
        if let Some(progress_bar) = &self.progress_bar {
            progress_bar.suspend(|| eprintln!("{line}"));
        } else {
            eprintln!("{line}");
        }
    }
}

impl tracing::field::Visit for EventFields {
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        match field.name() {
            "current" => self.current = Some(value),
            "total" => self.total = Some(value),
            "complete" => self.complete = value != 0,
            name => self.fields.push((name.to_string(), value.to_string())),
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if let Ok(value) = u64::try_from(value) {
            self.record_u64(field, value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "message" => self.message = Some(value.to_string()),
            "phase" => self.phase = Some(value.to_string()),
            name => self.fields.push((name.to_string(), value.to_string())),
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        match field.name() {
            "complete" => self.complete = value,
            name => self.fields.push((name.to_string(), value.to_string())),
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        let value = format!("{value:?}");
        match field.name() {
            "message" => self.message = Some(value),
            "phase" => self.phase = Some(value),
            name => self.fields.push((name.to_string(), value)),
        }
    }
}

fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .expect("progress template is valid")
        .progress_chars("=> ")
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .expect("spinner progress template is valid")
}

fn format_log_line(elapsed: Duration, message: &str, fields: &[(String, String)]) -> String {
    let mut line = format!("{} INFO {message}", elapsed_prefix(elapsed));
    for (name, value) in fields {
        let _ = write!(line, " {name}={value}");
    }
    line
}

fn elapsed_prefix(elapsed: Duration) -> String {
    let total_millis = elapsed.as_millis();
    let total_seconds = total_millis / 1000;
    let millis = total_millis % 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("[{minutes:02}:{seconds:02}.{millis:03}]")
}

#[cfg(test)]
mod tests {
    use super::{CliProgressState, should_render_event};
    use std::time::Instant;

    #[test]
    fn progress_events_render_by_default() {
        assert!(should_render_event("atlas_progress", false));
    }

    #[test]
    fn internal_library_logs_are_quiet_by_default() {
        assert!(!should_render_event("atlas_embedding::minilm", false));
    }

    #[test]
    fn internal_library_logs_can_be_enabled_for_diagnostics() {
        assert!(should_render_event("atlas_embedding::minilm", true));
    }

    #[test]
    fn total_less_progress_is_quiet_when_not_interactive() {
        let mut state = CliProgressState {
            is_interactive: false,
            started_at: Instant::now(),
            progress_bar: None,
            progress_phase: None,
        };

        state.spinner("Searching records", Some(&"search".to_string()));

        assert!(state.progress_bar.is_none());
        assert!(state.progress_phase.is_none());
    }
}
