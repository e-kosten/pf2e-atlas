use super::{PublishedIdentity, write_json};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

pub(super) const INTERRUPTED_EXIT: i32 = 72;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InterruptionMarker {
    pub version: u32,
    pub database: PathBuf,
    pub declared_candidate: String,
    pub executable_sha256: String,
    pub publication: PublishedIdentity,
    pub phase: String,
}
pub(super) struct Observer {
    started: Instant,
    events: Mutex<File>,
    failed: AtomicBool,
    interruption: Option<(PathBuf, InterruptionMarker)>,
}
impl Observer {
    pub fn new(events: File, interruption: Option<(PathBuf, InterruptionMarker)>) -> Self {
        Self {
            started: Instant::now(),
            events: Mutex::new(events),
            failed: AtomicBool::new(false),
            interruption,
        }
    }
    pub fn failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }
}
#[derive(Default)]
struct Fields(BTreeMap<String, Value>);
impl tracing::field::Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().into(), json!(format!("{value:?}")));
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().into(), json!(value));
    }
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0.insert(field.name().into(), json!(value));
    }
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.insert(field.name().into(), json!(value));
    }
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0.insert(field.name().into(), json!(value));
    }
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.insert(field.name().into(), json!(value));
    }
}
fn mutation_boundary(target: &str, fields: &Fields) -> bool {
    target == "atlas_progress"
        && fields.0.get("phase").and_then(Value::as_str) == Some("validate_build_input")
}
fn boundary_matches(marker: &InterruptionMarker, fields: &Fields) -> bool {
    fields.0.get("workspace_key").and_then(Value::as_str)
        == Some(marker.publication.instance().workspace_key())
        && fields.0.get("reference_revision").and_then(Value::as_u64)
            == Some(marker.publication.instance().reference_revision())
        && fields
            .0
            .get("context_sha256")
            .and_then(Value::as_str)
            .map(|hash| if hash.is_empty() { None } else { Some(hash) })
            == Some(marker.publication.context_sha256())
}
fn write_bound_marker(
    path: &std::path::Path,
    marker: &InterruptionMarker,
    fields: &Fields,
) -> Result<(), String> {
    if !boundary_matches(marker, fields) {
        return Err("publication changed before the durable maintenance boundary".into());
    }
    write_json(path, marker)
}
impl tracing::Subscriber for Observer {
    fn register_callsite(
        &self,
        _: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        tracing::subscriber::Interest::sometimes()
    }
    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::TRACE)
    }
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        matches!(
            metadata.target(),
            "atlas_progress" | "atlas_build_diagnostic"
        )
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        if !self.enabled(event.metadata()) {
            return;
        }
        let mut fields = Fields::default();
        event.record(&mut fields);
        let result = (|| -> std::io::Result<()> {
            let mut file = self
                .events
                .lock()
                .map_err(|_| std::io::Error::other("event output lock poisoned"))?;
            serde_json::to_writer(
                &mut *file,
                &json!({"elapsed_ns":self.started.elapsed().as_nanos(),"target":event.metadata().target(),"fields":fields.0}),
            )?;
            writeln!(file)?;
            file.flush()?;
            if self.interruption.is_some() && mutation_boundary(event.metadata().target(), &fields)
            {
                file.sync_all()?;
            }
            Ok(())
        })();
        if result.is_err() {
            self.failed.store(true, Ordering::SeqCst);
        }
        if mutation_boundary(event.metadata().target(), &fields)
            && let Some((path, marker)) = &self.interruption
        {
            // The index writer emits this event after UpdateOwner::maintenance
            // durably marks and owns exclusive admission, before validate_input
            // or the publication transaction can mutate reference projections.
            if !boundary_matches(marker, &fields) {
                eprintln!(
                    "ingest-experiment: publication changed before maintenance; no bound marker written"
                );
                std::process::exit(74);
            }
            if self.failed() || write_bound_marker(path, marker, &fields).is_err() {
                std::process::exit(73);
            }
            eprintln!("ingest-experiment: durable maintenance interruption marker synced");
            std::process::exit(INTERRUPTED_EXIT);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_boundary_identity_never_writes_a_completion_marker() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("marker.json");
        let marker=InterruptionMarker {version:1,database:PathBuf::from("/task/db"),declared_candidate:"candidate".into(),executable_sha256:"a".repeat(64),publication:serde_json::from_value(json!({"instance":{"workspace_key":"workspace","reference_revision":9},"context_sha256":"b".repeat(64)})).unwrap(),phase:"validate_build_input".into()};
        let mut fields = Fields::default();
        fields.0.insert("workspace_key".into(), json!("workspace"));
        fields.0.insert("reference_revision".into(), json!(9));
        fields
            .0
            .insert("context_sha256".into(), json!("b".repeat(64)));
        for (key, value) in [
            ("workspace_key", json!("other")),
            ("reference_revision", json!(10)),
            ("context_sha256", json!("c".repeat(64))),
        ] {
            let mut wrong = Fields(fields.0.clone());
            wrong.0.insert(key.into(), value);
            assert!(write_bound_marker(&path, &marker, &wrong).is_err());
            assert!(!path.exists());
        }
        assert!(write_bound_marker(&path, &marker, &Fields::default()).is_err());
        assert!(!path.exists());
        write_bound_marker(&path, &marker, &fields).unwrap();
        assert!(path.is_file());
    }
    #[test]
    fn interruption_uses_the_exact_owned_phase_not_a_message_substring() {
        let mut fields = Fields::default();
        fields
            .0
            .insert("message".into(), json!("validate_build_input"));
        assert!(!mutation_boundary("atlas_progress", &fields));
        fields
            .0
            .insert("phase".into(), json!("validate_build_input"));
        assert!(mutation_boundary("atlas_progress", &fields));
        assert!(!mutation_boundary("atlas_build_diagnostic", &fields));
        fields.0.insert("phase".into(), json!("write_artifact"));
        assert!(!mutation_boundary("atlas_progress", &fields));
    }
}
