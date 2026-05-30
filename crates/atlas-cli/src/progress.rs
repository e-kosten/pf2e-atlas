use std::fmt::{self, Write as FmtWrite};
use std::io::IsTerminal;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgressMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProgressOptions {
    pub(crate) mode: ProgressMode,
    pub(crate) json: bool,
    pub(crate) setup_timing: bool,
}

pub(crate) fn init_tracing(options: ProgressOptions) {
    let subscriber = tracing_subscriber::registry().with(CliProgressLayer::new(
        options,
        internal_logs_enabled_from_env(),
    ));
    let _ = tracing::subscriber::set_global_default(subscriber);
}

struct CliProgressLayer {
    state: Mutex<CliProgressState>,
    options: ProgressOptions,
    internal_logs_enabled: bool,
}

#[derive(Debug)]
struct CliProgressState {
    is_interactive: bool,
    started_at: Instant,
    phase_started_at: Instant,
    progress_bar: Option<ProgressBar>,
    progress_phase: Option<String>,
    progress_has_total: bool,
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
    fn new(options: ProgressOptions, internal_logs_enabled: bool) -> Self {
        Self {
            state: Mutex::new(CliProgressState {
                is_interactive: std::io::stderr().is_terminal(),
                started_at: Instant::now(),
                phase_started_at: Instant::now(),
                progress_bar: None,
                progress_phase: None,
                progress_has_total: false,
            }),
            options,
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

        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        if target == "atlas_progress" {
            let render = progress_render_mode(self.options, state.is_interactive);
            if render.enabled() {
                state.progress(&message, &fields, self.options.setup_timing, render);
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProgressRenderMode {
    Interactive,
    Plain,
    Hidden,
}

impl ProgressRenderMode {
    const fn enabled(self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

fn progress_render_mode(options: ProgressOptions, is_interactive: bool) -> ProgressRenderMode {
    progress_render_mode_for_mode(
        progress_mode_from_env().unwrap_or(options.mode),
        options,
        is_interactive,
    )
}

fn progress_render_mode_for_mode(
    mode: ProgressMode,
    options: ProgressOptions,
    is_interactive: bool,
) -> ProgressRenderMode {
    match mode {
        ProgressMode::Always if is_interactive => ProgressRenderMode::Interactive,
        ProgressMode::Always => ProgressRenderMode::Plain,
        ProgressMode::Never => ProgressRenderMode::Hidden,
        ProgressMode::Auto if is_interactive && !options.json => ProgressRenderMode::Interactive,
        ProgressMode::Auto => ProgressRenderMode::Hidden,
    }
}

fn progress_mode_from_env() -> Option<ProgressMode> {
    match std::env::var("ATLAS_PROGRESS").ok()?.as_str() {
        "always" | "1" | "true" => Some(ProgressMode::Always),
        "never" | "0" | "false" | "off" => Some(ProgressMode::Never),
        "auto" => Some(ProgressMode::Auto),
        _ => None,
    }
}

impl CliProgressState {
    fn progress(
        &mut self,
        message: &str,
        fields: &EventFields,
        setup_timing: bool,
        render: ProgressRenderMode,
    ) {
        if fields.complete {
            self.clear_progress();
            return;
        }
        let Some(total) = fields.total else {
            self.spinner(message, fields.phase.as_ref(), setup_timing, render);
            return;
        };
        let current = fields.current.unwrap_or(0);
        if render == ProgressRenderMode::Plain {
            eprintln!(
                "{} INFO {message}",
                log_elapsed_prefix(self.started_at.elapsed())
            );
            return;
        }

        let phase_changed = fields.phase.as_ref() != self.progress_phase.as_ref();
        if phase_changed || !self.progress_has_total {
            if phase_changed {
                self.phase_started_at = Instant::now();
            }
            if let Some(progress_bar) = self.progress_bar.take() {
                progress_bar.finish_and_clear();
            }
            let progress_bar =
                ProgressBar::with_draw_target(Some(total), ProgressDrawTarget::stderr_with_hz(10));
            progress_bar.set_style(progress_style(setup_timing));
            self.progress_bar = Some(progress_bar);
            self.progress_phase = fields.phase.clone();
            self.progress_has_total = true;
        } else if let Some(progress_bar) = &self.progress_bar {
            progress_bar.set_length(total);
        }

        if let Some(progress_bar) = &self.progress_bar {
            progress_bar.set_position(current);
            progress_bar.set_message(message.to_string());
            if setup_timing {
                progress_bar.set_prefix(progress_prefix(
                    self.started_at.elapsed(),
                    self.phase_started_at.elapsed(),
                ));
            }
            if current >= total {
                progress_bar.finish_and_clear();
                self.progress_bar = None;
                self.progress_phase = None;
                self.progress_has_total = false;
            }
        }
    }

    fn spinner(
        &mut self,
        message: &str,
        phase: Option<&String>,
        setup_timing: bool,
        render: ProgressRenderMode,
    ) {
        if render == ProgressRenderMode::Hidden {
            return;
        }
        if render == ProgressRenderMode::Plain {
            eprintln!(
                "{} INFO {message}",
                log_elapsed_prefix(self.started_at.elapsed())
            );
            return;
        }

        let phase_changed = phase != self.progress_phase.as_ref();
        if phase_changed {
            self.clear_progress();
            self.phase_started_at = Instant::now();
            let progress_bar = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr());
            progress_bar.set_style(spinner_style(setup_timing));
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            self.progress_bar = Some(progress_bar);
            self.progress_phase = phase.cloned();
            self.progress_has_total = false;
        }

        if let Some(progress_bar) = &self.progress_bar {
            progress_bar.set_message(message.to_string());
            if setup_timing {
                progress_bar.set_prefix(progress_prefix(
                    self.started_at.elapsed(),
                    self.phase_started_at.elapsed(),
                ));
            }
        }
    }

    fn clear_progress(&mut self) {
        if let Some(progress_bar) = self.progress_bar.take() {
            progress_bar.finish_and_clear();
        }
        self.progress_phase = None;
        self.progress_has_total = false;
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

fn progress_style(setup_timing: bool) -> ProgressStyle {
    let template = if setup_timing {
        "{prefix} [{bar:40.cyan/blue}] {pos}/{len} {msg}"
    } else {
        "[{bar:40.cyan/blue}] {pos}/{len} {msg}"
    };
    ProgressStyle::with_template(template)
        .unwrap_or_else(|error| {
            eprintln!("invalid progress bar template: {error}");
            ProgressStyle::default_bar()
        })
        .progress_chars("=> ")
}

fn spinner_style(setup_timing: bool) -> ProgressStyle {
    let template = if setup_timing {
        "{spinner:.cyan} {prefix} {msg}"
    } else {
        "{spinner:.cyan} {msg}"
    };
    ProgressStyle::with_template(template).unwrap_or_else(|error| {
        eprintln!("invalid progress spinner template: {error}");
        ProgressStyle::default_spinner()
    })
}

fn format_log_line(elapsed: Duration, message: &str, fields: &[(String, String)]) -> String {
    let mut line = format!("{} INFO {message}", log_elapsed_prefix(elapsed));
    for (name, value) in fields {
        let _ = write!(line, " {name}={value}");
    }
    line
}

fn progress_prefix(total_elapsed: Duration, phase_elapsed: Duration) -> String {
    format!(
        "[{} | {}]",
        compact_elapsed(total_elapsed),
        compact_elapsed(phase_elapsed)
    )
}

fn log_elapsed_prefix(elapsed: Duration) -> String {
    format!("[{}]", detailed_elapsed(elapsed))
}

fn compact_elapsed(elapsed: Duration) -> String {
    let total_seconds = elapsed.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn detailed_elapsed(elapsed: Duration) -> String {
    let total_millis = elapsed.as_millis();
    let total_seconds = total_millis / 1000;
    let millis = total_millis % 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}.{millis:03}")
}

#[cfg(test)]
mod tests {
    use super::{
        CliProgressState, ProgressMode, ProgressOptions, ProgressRenderMode, progress_prefix,
        progress_render_mode_for_mode, should_render_event,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn progress_events_render_by_default() {
        assert!(should_render_event("atlas_progress", false));
    }

    #[test]
    fn internal_library_logs_are_quiet_by_default() {
        assert!(!should_render_event("atlas_embedding::minilm", false));
    }

    #[test]
    fn auto_progress_renders_only_for_human_text_terminals() {
        assert_eq!(
            progress_render_mode_for_mode(
                ProgressMode::Auto,
                ProgressOptions {
                    mode: ProgressMode::Auto,
                    json: false,
                    setup_timing: false,
                },
                true,
            ),
            ProgressRenderMode::Interactive
        );
        assert_eq!(
            progress_render_mode_for_mode(
                ProgressMode::Auto,
                ProgressOptions {
                    mode: ProgressMode::Auto,
                    json: true,
                    setup_timing: false,
                },
                true,
            ),
            ProgressRenderMode::Hidden
        );
        assert_eq!(
            progress_render_mode_for_mode(
                ProgressMode::Auto,
                ProgressOptions {
                    mode: ProgressMode::Auto,
                    json: false,
                    setup_timing: false,
                },
                false,
            ),
            ProgressRenderMode::Hidden
        );
    }

    #[test]
    fn explicit_progress_modes_override_auto_policy() {
        assert_eq!(
            progress_render_mode_for_mode(
                ProgressMode::Always,
                ProgressOptions {
                    mode: ProgressMode::Always,
                    json: true,
                    setup_timing: false,
                },
                false,
            ),
            ProgressRenderMode::Plain
        );
        assert_eq!(
            progress_render_mode_for_mode(
                ProgressMode::Never,
                ProgressOptions {
                    mode: ProgressMode::Never,
                    json: false,
                    setup_timing: false,
                },
                true,
            ),
            ProgressRenderMode::Hidden
        );
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
            phase_started_at: Instant::now(),
            progress_bar: None,
            progress_phase: None,
            progress_has_total: false,
        };

        state.spinner(
            "Searching records",
            Some(&"search".to_string()),
            false,
            ProgressRenderMode::Hidden,
        );

        assert!(state.progress_bar.is_none());
        assert!(state.progress_phase.is_none());
    }

    #[test]
    fn forced_total_less_progress_uses_plain_noninteractive_output() {
        let mut state = CliProgressState {
            is_interactive: false,
            started_at: Instant::now(),
            phase_started_at: Instant::now(),
            progress_bar: None,
            progress_phase: None,
            progress_has_total: false,
        };

        state.spinner(
            "Loading embedding model",
            Some(&"load-embeddings".to_string()),
            false,
            ProgressRenderMode::Plain,
        );

        assert!(state.progress_bar.is_none());
        assert!(state.progress_phase.is_none());
    }

    #[test]
    fn progress_prefix_shows_total_and_phase_elapsed() {
        assert_eq!(
            progress_prefix(Duration::from_secs(134), Duration::from_secs(37)),
            "[02:14 | 00:37]"
        );
    }
}
