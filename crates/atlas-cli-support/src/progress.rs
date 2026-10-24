use crate::duration::format_elapsed as compact_elapsed;
use std::fmt::{self, Write as FmtWrite};
use std::io::IsTerminal;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressState, ProgressStyle};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressOptions {
    pub mode: ProgressMode,
    pub json: bool,
    pub setup_timing: bool,
}

pub fn init_tracing(options: ProgressOptions) {
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
    unit: Option<String>,
    fields: Vec<(String, String)>,
}

impl CliProgressLayer {
    fn new(options: ProgressOptions, internal_logs_enabled: bool) -> Self {
        Self {
            state: Mutex::new(CliProgressState {
                is_interactive: std::io::stderr().is_terminal(),
                started_at: Instant::now(),
                progress_bar: None,
                progress_phase: None,
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
            if self.internal_logs_enabled && !fields.fields.is_empty() {
                state.log(&message, &fields.fields);
            }
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
            let message = progress_message(message, current, total, fields.unit.as_deref());
            self.ensure_display(
                fields.phase.clone(),
                Some(total),
                Some(current),
                ProgressDrawTarget::hidden(),
            );
            let elapsed = self
                .progress_bar
                .as_ref()
                .map(ProgressBar::elapsed)
                .unwrap_or_default();
            let prefix = if setup_timing {
                format!("[total {}] ", compact_elapsed(self.started_at.elapsed()))
            } else {
                String::new()
            };
            eprintln!(
                "{prefix}{message} · {}",
                work_timing(current, total, elapsed),
            );
            self.update_progress_bar(current, &message);
            return;
        }

        self.ensure_display(
            fields.phase.clone(),
            Some(total),
            Some(current),
            interactive_draw_target(),
        );

        if let Some(bar) = &self.progress_bar {
            bar.set_style(counted_style(
                setup_timing,
                fields.unit.as_deref(),
                self.started_at,
            ));
        }
        self.update_progress_bar(current, message);
    }

    fn ensure_display(
        &mut self,
        phase: Option<String>,
        total: Option<u64>,
        current: Option<u64>,
        draw_target: ProgressDrawTarget,
    ) {
        if let Some(bar) = &self.progress_bar {
            if phase != self.progress_phase
                || current.is_some_and(|current| current < bar.position())
            {
                bar.reset_elapsed();
                bar.set_position(0);
            }
            match total {
                Some(total) => bar.set_length(total),
                None => bar.unset_length(),
            }
        } else {
            let bar = ProgressBar::with_draw_target(total, draw_target);
            bar.enable_steady_tick(Duration::from_millis(100));
            self.progress_bar = Some(bar);
        }
        self.progress_phase = phase;
    }

    fn update_progress_bar(&self, current: u64, message: &str) {
        if let Some(bar) = &self.progress_bar {
            bar.set_position(current);
            bar.set_message(message.to_string());
        }
    }

    #[cfg(test)]
    fn progress_with_draw_target_for_test(
        &mut self,
        fields: &EventFields,
        message: &str,
        setup_timing: bool,
        draw_target: ProgressDrawTarget,
    ) {
        let total = fields.total.expect("test progress event should have total");
        let current = fields.current.unwrap_or(0);
        self.ensure_display(
            fields.phase.clone(),
            Some(total),
            Some(current),
            draw_target,
        );
        if let Some(bar) = &self.progress_bar {
            bar.disable_steady_tick();
            bar.set_style(counted_style(
                setup_timing,
                fields.unit.as_deref(),
                self.started_at,
            ));
        }
        self.update_progress_bar(current, message);
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
            self.ensure_display(phase.cloned(), None, None, ProgressDrawTarget::hidden());
            if setup_timing {
                eprintln!(
                    "[total {}] {message}",
                    compact_elapsed(self.started_at.elapsed())
                );
            } else {
                eprintln!("{message}");
            }
            return;
        }

        self.ensure_display(phase.cloned(), None, None, interactive_draw_target());
        self.update_spinner(message, setup_timing);
    }

    fn update_spinner(&mut self, message: &str, setup_timing: bool) {
        if let Some(progress_bar) = &self.progress_bar {
            let started_at = self.started_at;
            progress_bar.set_style(spinner_style(setup_timing).with_key(
                "total_elapsed",
                move |_: &ProgressState, writer: &mut dyn fmt::Write| {
                    let _ = write!(writer, "{}", compact_elapsed(started_at.elapsed()));
                },
            ));
            progress_bar.set_message(message.to_string());
        }
    }

    #[cfg(test)]
    fn spinner_with_draw_target_for_test(
        &mut self,
        phase: Option<String>,
        message: &str,
        setup_timing: bool,
        draw_target: ProgressDrawTarget,
    ) {
        self.ensure_display(phase, None, None, draw_target);
        self.progress_bar.as_ref().unwrap().disable_steady_tick();
        self.update_spinner(message, setup_timing);
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
            "unit" => self.unit = Some(value.to_string()),
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
            "unit" => self.unit = Some(value),
            name => self.fields.push((name.to_string(), value)),
        }
    }
}

fn progress_style(setup_timing: bool) -> ProgressStyle {
    progress_style_with_phase_elapsed(progress_template(setup_timing), ProgressStyle::default_bar)
}

fn interactive_draw_target() -> ProgressDrawTarget {
    // indicatif's terminal target hides all progress under NO_COLOR. A TermLike
    // target retains monochrome feedback while console still disables colors.
    ProgressDrawTarget::term_like_with_hz(Box::new(console::Term::buffered_stderr()), 10)
}

fn spinner_style(setup_timing: bool) -> ProgressStyle {
    progress_style_with_phase_elapsed(
        spinner_template(setup_timing),
        ProgressStyle::default_spinner,
    )
}

fn progress_template(setup_timing: bool) -> &'static str {
    if setup_timing {
        "[total {total_elapsed}] {wide_msg}\n{wide_bar:.cyan} {counter} · {work_timing}"
    } else {
        "{wide_msg}\n{wide_bar:.cyan} {counter} · {work_timing}"
    }
}

fn counted_style(setup_timing: bool, unit: Option<&str>, started_at: Instant) -> ProgressStyle {
    let suffix = unit.map(|unit| format!(" {unit}")).unwrap_or_default();
    progress_style(setup_timing)
        .progress_chars("=>-")
        .with_key(
            "counter",
            move |state: &ProgressState, writer: &mut dyn fmt::Write| {
                let _ = write!(
                    writer,
                    "{}/{}{suffix}",
                    state.pos(),
                    state.len().unwrap_or(0)
                );
            },
        )
        .with_key(
            "total_elapsed",
            move |_: &ProgressState, writer: &mut dyn fmt::Write| {
                let _ = write!(writer, "{}", compact_elapsed(started_at.elapsed()));
            },
        )
}

fn spinner_template(setup_timing: bool) -> &'static str {
    if setup_timing {
        "[total {total_elapsed}] {wide_msg}\n{spinner:.cyan} {phase_elapsed}"
    } else {
        "{spinner:.cyan} {wide_msg} · {phase_elapsed}"
    }
}

fn progress_style_with_phase_elapsed(
    template: &str,
    fallback: fn() -> ProgressStyle,
) -> ProgressStyle {
    ProgressStyle::with_template(template)
        .unwrap_or_else(|error| {
            eprintln!("invalid progress template: {error}");
            fallback()
        })
        .with_key(
            "phase_elapsed",
            |state: &ProgressState, writer: &mut dyn fmt::Write| {
                let _ = write!(writer, "{}", compact_elapsed(state.elapsed()));
            },
        )
        .with_key(
            "work_timing",
            |state: &ProgressState, writer: &mut dyn fmt::Write| {
                let _ = write!(
                    writer,
                    "{}",
                    work_timing(state.pos(), state.len().unwrap_or(0), state.elapsed())
                );
            },
        )
}

// Stages supply labels and units; the renderer owns counters and time formatting.
fn progress_message(message: &str, current: u64, total: u64, unit: Option<&str>) -> String {
    let suffix = unit.map(|unit| format!(" {unit}")).unwrap_or_default();
    format!("{message} · {current}/{total}{suffix}")
}

fn work_timing(current: u64, total: u64, elapsed: Duration) -> String {
    let mut text = compact_elapsed(elapsed);
    // Wait for a useful sample. Keep estimates deliberately coarser than elapsed
    // time so batch completion does not make long-running inference look jittery.
    if current > 0 && current < total && elapsed.as_secs() >= 5 {
        let remaining = elapsed.as_secs_f64() * (total - current) as f64 / current as f64;
        if let Ok(remaining) = Duration::try_from_secs_f64(remaining) {
            let seconds = remaining.as_secs();
            let quantum = if seconds >= 60 { 60 } else { 5 };
            let rounded = seconds.div_ceil(quantum).saturating_mul(quantum);
            let _ = write!(
                text,
                " · ETA ~{}",
                compact_elapsed(Duration::from_secs(rounded))
            );
        }
    }
    text
}

fn format_log_line(elapsed: Duration, message: &str, fields: &[(String, String)]) -> String {
    let mut line = format!("{} INFO {message}", log_elapsed_prefix(elapsed));
    for (name, value) in fields {
        let _ = write!(line, " {name}={value}");
    }
    line
}

fn log_elapsed_prefix(elapsed: Duration) -> String {
    format!("[{}]", compact_elapsed(elapsed))
}

#[cfg(test)]
mod tests {
    use super::{
        CliProgressState, EventFields, ProgressMode, ProgressOptions, ProgressRenderMode,
        progress_render_mode_for_mode, progress_template, should_render_event, spinner_template,
    };
    use indicatif::{ProgressBar, ProgressDrawTarget, TermLike};
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    #[derive(Clone, Debug)]
    struct RecordingTerm {
        contents: Arc<Mutex<String>>,
        width: u16,
    }

    impl Default for RecordingTerm {
        fn default() -> Self {
            Self {
                contents: Arc::default(),
                width: 80,
            }
        }
    }

    impl RecordingTerm {
        fn contents(&self) -> String {
            self.contents.lock().unwrap().clone()
        }
    }

    impl TermLike for RecordingTerm {
        fn width(&self) -> u16 {
            self.width
        }

        fn move_cursor_up(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn move_cursor_down(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn move_cursor_right(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn move_cursor_left(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn write_line(&self, value: &str) -> io::Result<()> {
            self.write_str(value)?;
            self.write_str("\n")
        }

        fn write_str(&self, value: &str) -> io::Result<()> {
            self.contents.lock().unwrap().push_str(value);
            Ok(())
        }

        fn clear_line(&self) -> io::Result<()> {
            self.contents.lock().unwrap().clear();
            Ok(())
        }

        fn flush(&self) -> io::Result<()> {
            Ok(())
        }
    }

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
            progress_bar: None,
            progress_phase: None,
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
            progress_bar: None,
            progress_phase: None,
        };

        state.spinner(
            "Loading embedding model",
            Some(&"load-embeddings".to_string()),
            false,
            ProgressRenderMode::Plain,
        );

        assert!(state.progress_bar.is_some());
        assert_eq!(state.progress_phase.as_deref(), Some("load-embeddings"));
    }

    #[test]
    fn setup_timing_templates_put_elapsed_prefix_first() {
        assert_eq!(
            progress_template(true),
            "[total {total_elapsed}] {wide_msg}\n{wide_bar:.cyan} {counter} · {work_timing}"
        );
        assert_eq!(
            spinner_template(true),
            "[total {total_elapsed}] {wide_msg}\n{spinner:.cyan} {phase_elapsed}"
        );
    }

    #[test]
    fn setup_timing_progress_renders_total_and_phase_elapsed() {
        let term = RecordingTerm::default();
        let progress_bar = ProgressBar::with_draw_target(
            Some(5),
            ProgressDrawTarget::term_like(Box::new(term.clone())),
        );
        progress_bar.set_style(super::counted_style(
            true,
            None,
            Instant::now() - Duration::from_secs(42),
        ));
        progress_bar.set_prefix("42s");
        progress_bar.set_position(2);
        progress_bar.set_message("Rendering progress");
        progress_bar.tick();

        let rendered = term.contents();
        assert!(
            rendered.starts_with("[total 42s] "),
            "rendered progress should start with total and phase elapsed: {rendered:?}"
        );
        assert!(rendered.contains("Rendering progress"));
    }

    #[test]
    fn progress_state_renders_total_and_phase_elapsed() {
        let term = RecordingTerm::default();
        let mut state = CliProgressState {
            is_interactive: true,
            started_at: Instant::now() - Duration::from_secs(42),
            progress_bar: None,
            progress_phase: None,
        };
        let fields = EventFields {
            message: None,
            phase: Some("phase_one".to_string()),
            current: Some(2),
            total: Some(5),
            unit: None,
            complete: false,
            fields: Vec::new(),
        };

        state.progress_with_draw_target_for_test(
            &fields,
            "State progress",
            true,
            ProgressDrawTarget::term_like(Box::new(term.clone())),
        );

        let rendered = term.contents();
        assert!(
            rendered.starts_with("[total 42s] "),
            "rendered state progress should use total elapsed and per-phase elapsed: {rendered:?}"
        );
        assert!(rendered.contains("State progress"));
        assert!(rendered.contains("2/5"));
    }

    #[test]
    fn setup_timing_spinner_renders_total_and_phase_elapsed() {
        let term = RecordingTerm::default();
        let progress_bar = ProgressBar::with_draw_target(
            None,
            ProgressDrawTarget::term_like(Box::new(term.clone())),
        );
        progress_bar.set_style(super::spinner_style(true).with_key(
            "total_elapsed",
            |_: &indicatif::ProgressState, writer: &mut dyn std::fmt::Write| {
                let _ = write!(writer, "42s");
            },
        ));
        progress_bar.set_prefix("42s");
        progress_bar.set_message("Rendering spinner");
        progress_bar.tick();

        let rendered = term.contents();
        assert!(
            rendered.starts_with("[total 42s] "),
            "rendered spinner should start with total and phase elapsed: {rendered:?}"
        );
        assert!(rendered.contains("Rendering spinner"));
    }

    #[test]
    fn spinner_state_renders_total_and_phase_elapsed() {
        let term = RecordingTerm::default();
        let mut state = CliProgressState {
            is_interactive: true,
            started_at: Instant::now() - Duration::from_secs(42),
            progress_bar: None,
            progress_phase: None,
        };

        state.spinner_with_draw_target_for_test(
            Some("phase_one".to_string()),
            "State spinner",
            true,
            ProgressDrawTarget::term_like(Box::new(term.clone())),
        );

        let rendered = term.contents();
        assert!(
            rendered.starts_with("[total 42s] "),
            "rendered state spinner should use total elapsed and per-phase elapsed: {rendered:?}"
        );
        assert!(rendered.contains("State spinner"));
    }
    #[test]
    fn progress_timing_omits_unknown_and_completed_estimates() {
        assert_eq!(super::work_timing(0, 100, Duration::from_secs(10)), "10s");
        assert_eq!(super::work_timing(1, 100, Duration::from_secs(1)), "1s");
        assert_eq!(super::work_timing(100, 100, Duration::from_secs(50)), "50s");
        assert_eq!(super::work_timing(0, 0, Duration::ZERO), "0s");
        assert!(!super::work_timing(1, u64::MAX, Duration::MAX).contains("ETA"));
    }

    #[test]
    fn long_estimates_do_not_change_with_every_small_batch() {
        for current in [1000, 1001, 1002] {
            assert_eq!(
                super::work_timing(current, 2000, Duration::from_secs(3600)),
                "1h · ETA ~1h"
            );
        }
        assert_eq!(
            super::work_timing(20, 100, Duration::from_secs(10)),
            "10s · ETA ~40s"
        );
        assert_eq!(
            super::work_timing(20, 100, Duration::from_secs(11)),
            "11s · ETA ~45s"
        );
    }

    #[test]
    fn rendered_setup_progress_has_one_counter_and_no_diagnostic_fields() {
        let term = RecordingTerm::default();
        let mut state = CliProgressState {
            is_interactive: true,
            started_at: Instant::now(),
            progress_bar: None,
            progress_phase: None,
        };
        let fields = EventFields {
            phase: Some("prepare_search".into()),
            current: Some(6000),
            total: Some(25641),
            unit: Some("records".into()),
            fields: vec![("semantic_units".into(), "70991".into())],
            ..Default::default()
        };
        state.progress_with_draw_target_for_test(
            &fields,
            "Preparing search content",
            false,
            ProgressDrawTarget::term_like(Box::new(term.clone())),
        );
        let rendered = term.contents();
        assert_eq!(rendered.matches("6000/25641").count(), 1, "{rendered}");
        assert!(rendered.contains("6000/25641 records"), "{rendered}");
        assert!(
            !rendered.contains("70991") && !rendered.contains("unknown"),
            "{rendered}"
        );
    }
    #[test]
    fn counted_progress_keeps_the_initial_spinner_elapsed_time() {
        let mut state = CliProgressState {
            is_interactive: false,
            started_at: Instant::now(),
            progress_bar: None,
            progress_phase: None,
        };
        state.ensure_display(
            Some("prepare".into()),
            None,
            None,
            ProgressDrawTarget::hidden(),
        );
        state
            .progress_bar
            .as_ref()
            .unwrap()
            .set_elapsed(Duration::from_secs(20));
        state.ensure_display(
            Some("prepare".into()),
            Some(100),
            Some(0),
            ProgressDrawTarget::hidden(),
        );
        assert!(state.progress_bar.as_ref().unwrap().elapsed() >= Duration::from_secs(20));
        state.ensure_display(
            Some("next".into()),
            Some(100),
            Some(0),
            ProgressDrawTarget::hidden(),
        );
        assert!(state.progress_bar.as_ref().unwrap().elapsed() < Duration::from_secs(5));
        state.ensure_display(
            Some("next".into()),
            None,
            None,
            ProgressDrawTarget::hidden(),
        );
        assert_eq!(state.progress_bar.as_ref().unwrap().length(), None);
    }

    #[test]
    fn completed_counts_and_phase_changes_keep_feedback_until_command_completion() {
        let mut state = CliProgressState {
            is_interactive: true,
            started_at: Instant::now(),
            progress_bar: None,
            progress_phase: None,
        };
        state.ensure_display(
            Some("first".into()),
            Some(100),
            Some(0),
            ProgressDrawTarget::hidden(),
        );
        let display = state.progress_bar.as_ref().unwrap().downgrade();
        state.update_progress_bar(100, "First stage complete");
        assert_eq!(display.upgrade().unwrap().position(), 100);

        state.ensure_display(
            Some("next".into()),
            None,
            None,
            ProgressDrawTarget::hidden(),
        );
        state.update_spinner("Checking the next stage", true);
        assert_eq!(
            display.upgrade().unwrap().message(),
            "Checking the next stage"
        );

        state.ensure_display(
            Some("last".into()),
            Some(50),
            Some(0),
            ProgressDrawTarget::hidden(),
        );
        state.update_progress_bar(25, "Last stage");
        assert_eq!(display.upgrade().unwrap().position(), 25);

        state.progress(
            "Finished",
            &EventFields {
                complete: true,
                ..Default::default()
            },
            true,
            ProgressRenderMode::Interactive,
        );
        assert!(display.upgrade().is_none());
    }

    #[test]
    fn repeated_counted_stages_restart_the_phase_timer_without_clearing_feedback() {
        let mut state = CliProgressState {
            is_interactive: true,
            started_at: Instant::now(),
            progress_bar: None,
            progress_phase: None,
        };
        state.ensure_display(
            Some("upgrade".into()),
            Some(100),
            Some(0),
            ProgressDrawTarget::hidden(),
        );
        let bar = state.progress_bar.as_ref().unwrap().downgrade();
        state.update_progress_bar(100, "First model");
        bar.upgrade().unwrap().set_elapsed(Duration::from_secs(60));
        state.ensure_display(
            Some("upgrade".into()),
            Some(100),
            Some(1),
            ProgressDrawTarget::hidden(),
        );
        assert!(bar.upgrade().unwrap().elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn narrow_terminals_keep_counts_bar_and_timing_visible() {
        for width in [60, 80] {
            let term = RecordingTerm {
                width,
                ..Default::default()
            };
            let mut state = CliProgressState {
                is_interactive: true,
                started_at: Instant::now() - Duration::from_secs(215),
                progress_bar: None,
                progress_phase: None,
            };
            state.progress_with_draw_target_for_test(
                &EventFields {
                    phase: Some("upgrade".into()),
                    current: Some(127_000),
                    total: Some(161_731),
                    unit: Some("passages".into()),
                    ..Default::default()
                },
                "Upgrading embedding storage — 狐 — a long operation label",
                true,
                ProgressDrawTarget::term_like(Box::new(term.clone())),
            );
            let bar = state.progress_bar.as_ref().unwrap();
            bar.set_elapsed(Duration::from_secs(86));
            bar.tick();
            let rendered = term.contents();
            assert!(rendered.contains("127000/161731 passages"), "{rendered}");
            assert!(rendered.contains("ETA ~25s"), "{rendered}");
            assert!(rendered.contains("=>"), "{rendered}");
            for line in rendered.lines() {
                assert!(
                    console::measure_text_width(line) <= usize::from(width),
                    "{line:?}"
                );
            }
        }
    }
}
