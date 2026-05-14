#![deny(unsafe_code)]

use std::fmt::{self, Write as FmtWrite};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use atlas_domain::SearchFilterNode;
use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingModelId};
use atlas_index::{
    ArtifactValidationReport, ValidationCode, ValidationStatus, inspect_index,
    validate_index_report, validate_vector_index_report, write_vector_index_report,
};
use atlas_ingest::{
    BuildArtifactOptions, analyze_foundry_source, build_artifact, report::build_artifact_json,
};
use atlas_search::{EmbeddingRuntimeConfig, SemanticSearchService};
use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde_json::json;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

#[derive(Debug, Parser)]
#[command(name = "atlas")]
#[command(about = "PF2e Atlas local search and index tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Build, validate, inspect, and analyze Atlas indexes")]
    Index(IndexArgs),
    #[command(about = "Run Atlas search commands")]
    Search(SearchArgs),
}

#[derive(Debug, Args)]
struct IndexArgs {
    #[command(subcommand)]
    command: IndexCommand,
}

#[derive(Debug, Args)]
struct SearchArgs {
    #[command(subcommand)]
    command: SearchCommand,
}

#[derive(Debug, Subcommand)]
enum IndexCommand {
    #[command(about = "Analyze Foundry source ingest without writing SQLite")]
    Analyze(AnalyzeIndexOptions),
    #[command(about = "Build a Rust SQLite artifact from Foundry source files")]
    Build(BuildIndexOptions),
    #[command(about = "Inspect artifact table and field coverage")]
    Inspect(IndexPathOptions),
    #[command(about = "Open an index read-only and validate Rust artifact metadata")]
    Validate(IndexPathOptions),
    #[command(about = "Validate sqlite-vec availability and record_vector_index coherence")]
    ValidateVectors(IndexPathOptions),
    #[command(about = "Build record_vector_index from document_embedding_cache")]
    BuildVectors(IndexPathOptions),
}

#[derive(Debug, Subcommand)]
enum SearchCommand {
    #[command(about = "Run semantic vector search and print raw record-key hits")]
    Semantic(SemanticSearchOptions),
}

#[derive(Debug, Args)]
struct AnalyzeIndexOptions {
    #[arg(long)]
    source: PathBuf,
    #[arg(long)]
    manifest: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct BuildIndexOptions {
    #[arg(long)]
    source: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    manifest: Option<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL)]
    embedding_model: EmbeddingModelId,
    #[arg(long)]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long, default_value_t = 32)]
    embedding_batch_size: usize,
    #[arg(long)]
    no_reuse_embeddings: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct IndexPathOptions {
    #[arg(long)]
    index: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct SemanticSearchOptions {
    #[arg(long)]
    index: PathBuf,
    #[arg(long)]
    query: String,
    #[arg(long, default_value_t = 10)]
    limit: u32,
    #[arg(long)]
    filter_json: Option<String>,
    #[arg(long, default_value = ".cache/hf-models")]
    embedding_cache_path: PathBuf,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL)]
    embedding_model: EmbeddingModelId,
    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    init_tracing();
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn init_tracing() {
    let subscriber = tracing_subscriber::registry().with(CliProgressLayer::new());
    let _ = tracing::subscriber::set_global_default(subscriber);
}

struct CliProgressLayer {
    state: Mutex<CliProgressState>,
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
    current: Option<u64>,
    total: Option<u64>,
    fields: Vec<(String, String)>,
}

impl CliProgressLayer {
    fn new() -> Self {
        Self {
            state: Mutex::new(CliProgressState {
                is_interactive: std::io::stderr().is_terminal(),
                started_at: Instant::now(),
                progress_bar: None,
                progress_phase: None,
            }),
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
        if target != "atlas_progress" && !target.starts_with("atlas_") {
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

impl CliProgressState {
    fn progress(&mut self, message: &str, fields: &EventFields) {
        let Some(total) = fields.total else {
            self.log(message, &fields.fields);
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

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Index(index) => match index.command {
            IndexCommand::Analyze(options) => run_index_analyze(options),
            IndexCommand::Build(options) => run_index_build(options),
            IndexCommand::Inspect(options) => run_index_inspect(options),
            IndexCommand::Validate(options) => run_index_validate(options),
            IndexCommand::ValidateVectors(options) => run_index_validate_vectors(options),
            IndexCommand::BuildVectors(options) => run_index_build_vectors(options),
        },
        Command::Search(search) => match search.command {
            SearchCommand::Semantic(options) => run_search_semantic(options),
        },
    }
}

fn run_index_analyze(options: AnalyzeIndexOptions) -> Result<ExitCode, String> {
    let report = analyze_foundry_source(&options.source, options.manifest.as_deref())
        .map_err(|error| error.to_string())?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "ok: analyzed {} records from {} packs in {}",
            report.record_count, report.pack_count, report.source.root
        );
        println!("source signature: {}", report.source.source_signature);
        println!(
            "records: source={} generated={} default_visible={} hidden={}",
            report.loaded_source_record_count,
            report.generated_record_count,
            report.default_visible_record_count,
            report.hidden_record_count
        );
        println!(
            "relationships: references={} aliases={} remaster_links={}",
            report.relationships.reference_edges,
            report.relationships.record_aliases,
            report.relationships.remaster_links
        );
        println!(
            "dropped inline macros: {}",
            report
                .diagnostics
                .get("dropped_inline_macros")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len)
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn run_index_build(options: BuildIndexOptions) -> Result<ExitCode, String> {
    let report = build_artifact(BuildArtifactOptions {
        source_root: options.source,
        output_path: options.output,
        manifest_path: options.manifest,
        embedding_model: options.embedding_model,
        embedding_cache_root: options.embedding_cache_path,
        reuse_embeddings: !options.no_reuse_embeddings,
        embedding_batch_size: options.embedding_batch_size,
    })
    .map_err(|error| error.to_string())?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&build_artifact_json(&report))
                .map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "ok: wrote {} records from {} packs to {}",
            report.artifact_record_count,
            report.pack_count,
            report.output_path.display()
        );
        eprintln!(
            "records: source={} generated={} artifact={}",
            report.source_record_count, report.generated_record_count, report.artifact_record_count
        );
        eprintln!(
            "embeddings: pending_document={} document={} reused={} generated={} truncated={} max_tokens={} max_observed_tokens={} build_duration_ms={}",
            report.pending_document_embedding_count,
            report.document_embedding_count,
            report.reused_document_embedding_count,
            report.generated_document_embedding_count,
            report
                .document_embedding_tokenization
                .truncated_document_count,
            report
                .document_embedding_tokenization
                .max_token_count
                .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
            report
                .document_embedding_tokenization
                .max_observed_token_count,
            report.build_duration_ms
        );
        eprintln!("source signature: {}", report.source_signature);
        eprintln!(
            "diagnostics: taxonomy folder={} glossary={} variants parenthetical={} suffix={} creature_blurb={} creature_suffix={} exact_base={}",
            report.diagnostics.taxonomy_folder_records,
            report.diagnostics.taxonomy_glossary_records,
            report.diagnostics.variant_parenthetical_records,
            report.diagnostics.variant_suffix_records,
            report.diagnostics.variant_creature_blurb_records,
            report.diagnostics.variant_creature_suffix_records,
            report.diagnostics.variant_exact_base_records
        );
        eprintln!(
            "generated afflictions: canonical={} instances={} edges={}",
            report.diagnostics.generated_affliction_canonical_records,
            report.diagnostics.generated_affliction_instance_records,
            report.diagnostics.generated_affliction_reference_edges
        );
        if !report.diagnostics.dropped_inline_macros.is_empty() {
            let dropped_inline_macros = report
                .diagnostics
                .dropped_inline_macros
                .iter()
                .map(|(name, diagnostic)| format!("{name}={}", diagnostic.count))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("dropped inline macros: {dropped_inline_macros}");
        }
        for skipped_record in &report.skipped_records {
            eprintln!(
                "skipped record: {}: {}",
                skipped_record.path.display(),
                skipped_record.reason
            );
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn run_index_inspect(options: IndexPathOptions) -> Result<ExitCode, String> {
    let report = inspect_index(&options.index).map_err(|error| error.to_string())?;

    if options.json {
        let body = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
        println!("{body}");
    } else {
        println!(
            "ok: inspected {} records in {}",
            report.records.total_records, report.index
        );
        println!(
            "tables: records={} packs={} references={} aliases={} remaster_links={}",
            report.records_table_count(),
            report.packs_table_count(),
            report.reference_edges_table_count(),
            report.record_aliases_table_count(),
            report.remaster_links_table_count()
        );
        println!(
            "coverage: taxonomy_records={} variant_records={} descriptions={} blurbs={}",
            report.taxonomy.records_with_taxonomy_families,
            report.variants.grouped_records,
            report.text.records_with_description,
            report.text.records_with_blurb
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn run_index_validate(options: IndexPathOptions) -> Result<ExitCode, String> {
    let report = validate_index_report(&options.index);
    write_validation_report(report, options.json)
}

fn run_index_validate_vectors(options: IndexPathOptions) -> Result<ExitCode, String> {
    let report = validate_vector_index_report(&options.index);
    write_validation_report(report, options.json)
}

fn run_index_build_vectors(options: IndexPathOptions) -> Result<ExitCode, String> {
    let report = write_vector_index_report(&options.index);
    write_validation_report(report, options.json)
}

fn run_search_semantic(options: SemanticSearchOptions) -> Result<ExitCode, String> {
    let filter = options
        .filter_json
        .as_deref()
        .map(|filter_json| {
            serde_json::from_str::<SearchFilterNode>(filter_json)
                .map_err(|error| format!("failed to parse --filter-json: {error}"))
        })
        .transpose()?;

    let config =
        EmbeddingRuntimeConfig::new(options.embedding_model, &options.embedding_cache_path);
    let mut search =
        SemanticSearchService::open(&options.index, &config).map_err(|error| error.to_string())?;
    let hits = search
        .semantic(&options.query, filter.as_ref(), options.limit)
        .map_err(|error| error.to_string())?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ok",
                "query": options.query,
                "limit": options.limit,
                "hits": hits.iter().map(|hit| {
                    json!({
                        "record_key": hit.record_key,
                        "distance": hit.distance,
                    })
                }).collect::<Vec<_>>()
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("ok: {} semantic hits", hits.len());
        for hit in hits {
            println!("{}\t{}", hit.record_key, hit.distance);
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn write_validation_report(
    report: ArtifactValidationReport,
    json: bool,
) -> Result<ExitCode, String> {
    let exit_code = match report.status {
        ValidationStatus::Ok => ExitCode::SUCCESS,
        ValidationStatus::Error => ExitCode::from(1),
    };

    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
        println!("{body}");
    } else {
        println!(
            "{}: {}",
            validation_code_label(&report.code),
            report.message
        );
    }

    Ok(exit_code)
}

fn validation_code_label(code: &ValidationCode) -> &'static str {
    match code {
        ValidationCode::Ok => "ok",
        ValidationCode::IndexUnavailable => "index-unavailable",
        ValidationCode::MissingArtifactMetadata => "missing-artifact-metadata",
        ValidationCode::MissingRequiredMetadata => "missing-required-metadata",
        ValidationCode::UnsupportedContractVersion => "unsupported-contract-version",
        ValidationCode::UnsupportedSchemaVersion => "unsupported-schema-version",
        ValidationCode::ArtifactContractViolation => "artifact-contract-violation",
        ValidationCode::InvalidSourceMetadata => "invalid-source-metadata",
        ValidationCode::StaleSourceSignature => "stale-source-signature",
        ValidationCode::EmbeddingMismatch => "embedding-mismatch",
        ValidationCode::FtsMismatch => "fts-mismatch",
        ValidationCode::ManifestMismatch => "manifest-mismatch",
        ValidationCode::VectorExtensionUnavailable => "vector-extension-unavailable",
        ValidationCode::QueryFailed => "query-failed",
    }
}
