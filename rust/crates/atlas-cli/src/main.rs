#![deny(unsafe_code)]

use std::fmt::{self, Write as FmtWrite};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};
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
use atlas_search::{EmbeddingRuntimeConfig, SemanticSearchMode, SemanticSearchService};
use clap::{Args, Parser, Subcommand, ValueEnum};
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
    #[command(about = "Check or prepare local Atlas data and model paths")]
    Setup(SetupOptions),
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

#[derive(Debug, Args)]
struct SetupOptions {
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long)]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long)]
    fetch_source: bool,
    #[arg(long)]
    json: bool,
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
    source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    manifest: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct BuildIndexOptions {
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
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
    no_embeddings: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct IndexPathOptions {
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct SemanticSearchOptions {
    #[arg(long)]
    index: Option<PathBuf>,
    #[arg(long)]
    query: String,
    #[arg(long, default_value_t = 10)]
    limit: u32,
    #[arg(long)]
    filter_json: Option<String>,
    #[arg(long)]
    embedding_cache_path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Auto)]
    path_mode: CliPathMode,
    #[arg(long, default_value_t = DEFAULT_EMBEDDING_MODEL)]
    embedding_model: EmbeddingModelId,
    #[arg(long, value_enum, default_value_t = CliSemanticMode::WeightedChunks)]
    semantic_mode: CliSemanticMode,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliSemanticMode {
    ParentOnly,
    Chunks,
    WeightedChunks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliPathMode {
    Auto,
    Repo,
    User,
}

impl From<CliSemanticMode> for SemanticSearchMode {
    fn from(mode: CliSemanticMode) -> Self {
        match mode {
            CliSemanticMode::ParentOnly => Self::ParentOnly,
            CliSemanticMode::Chunks => Self::Chunks,
            CliSemanticMode::WeightedChunks => Self::WeightedChunks,
        }
    }
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
        Command::Setup(options) => run_setup(options),
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

fn run_setup(options: SetupOptions) -> Result<ExitCode, String> {
    let paths = resolve_atlas_paths(
        options.path_mode,
        AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    )?;

    if options.fetch_source {
        fetch_pf2e_source(&paths.source_root)?;
    }

    let source_exists = paths.source_root.is_dir();
    let model_status = embedding_model_cache_status(&paths.embedding_cache_root);
    let index_exists = paths.index_path.is_file();
    let ready = source_exists && model_status.ready;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": if ready { "ready" } else { "not_ready" },
                "path_mode": paths.mode.as_str(),
                "repo_root": paths.repo_root.as_ref().map(|path| path.display().to_string()),
                "source": {
                    "path": paths.source_root.display().to_string(),
                    "exists": source_exists,
                },
                "embedding": {
                    "model": DEFAULT_EMBEDDING_MODEL.to_string(),
                    "cache_root": paths.embedding_cache_root.display().to_string(),
                    "model_path": model_status.model_dir.display().to_string(),
                    "ready": model_status.ready,
                    "missing_files": model_status
                        .missing_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>(),
                },
                "index": {
                    "path": paths.index_path.display().to_string(),
                    "exists": index_exists,
                },
                "next": {
                    "build_index": format!(
                        "atlas index build --path-mode {}",
                        paths.mode.suggested_path_mode()
                    ),
                },
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("mode: {}", paths.mode.label());
        if let Some(repo_root) = &paths.repo_root {
            println!("repo root: {}", repo_root.display());
        }
        println!(
            "source: {} {}",
            status_label(source_exists),
            paths.source_root.display()
        );
        println!("embedding model: {}", DEFAULT_EMBEDDING_MODEL);
        println!(
            "embedding cache: {} {}",
            status_label(model_status.ready),
            model_status.model_dir.display()
        );
        for missing_file in &model_status.missing_files {
            println!("missing model file: {}", missing_file.display());
        }
        println!(
            "index: {} {}",
            status_label(index_exists),
            paths.index_path.display()
        );
        if ready {
            println!();
            println!("ready:");
            println!(
                "  atlas index build --path-mode {}",
                paths.mode.suggested_path_mode()
            );
        } else {
            println!();
            println!("not ready:");
            if !source_exists {
                println!("  run atlas setup --fetch-source");
            }
            if !model_status.ready {
                println!("  prepare the default embedding model cache, then rerun atlas setup");
            }
        }
    }

    Ok(if ready {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn run_index_analyze(options: AnalyzeIndexOptions) -> Result<ExitCode, String> {
    let paths = resolve_atlas_paths(
        options.path_mode,
        AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: None,
            index_path: None,
        },
    )?;
    let report = analyze_foundry_source(&paths.source_root, options.manifest.as_deref())
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
    let paths = resolve_atlas_paths(
        options.path_mode,
        AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.output,
        },
    )?;
    let report = build_artifact(BuildArtifactOptions {
        source_root: paths.source_root,
        output_path: paths.index_path,
        manifest_path: options.manifest,
        embedding_model: options.embedding_model,
        embedding_cache_root: if options.no_embeddings {
            None
        } else {
            Some(paths.embedding_cache_root)
        },
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
    let paths = resolve_index_path(options.path_mode, options.index)?;
    let report = inspect_index(&paths.index_path).map_err(|error| error.to_string())?;

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
    let paths = resolve_index_path(options.path_mode, options.index)?;
    let report = validate_index_report(&paths.index_path);
    write_validation_report(report, options.json)
}

fn run_index_validate_vectors(options: IndexPathOptions) -> Result<ExitCode, String> {
    let paths = resolve_index_path(options.path_mode, options.index)?;
    let report = validate_vector_index_report(&paths.index_path);
    write_validation_report(report, options.json)
}

fn run_index_build_vectors(options: IndexPathOptions) -> Result<ExitCode, String> {
    let paths = resolve_index_path(options.path_mode, options.index)?;
    let report = write_vector_index_report(&paths.index_path);
    write_validation_report(report, options.json)
}

fn run_search_semantic(options: SemanticSearchOptions) -> Result<ExitCode, String> {
    let total_started_at = Instant::now();
    let filter = options
        .filter_json
        .as_deref()
        .map(|filter_json| {
            serde_json::from_str::<SearchFilterNode>(filter_json)
                .map_err(|error| format!("failed to parse --filter-json: {error}"))
        })
        .transpose()?;

    let paths = resolve_atlas_paths(
        options.path_mode,
        AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    )?;
    let config = EmbeddingRuntimeConfig::new(options.embedding_model, &paths.embedding_cache_root);
    let service_open_started_at = Instant::now();
    let mut search = SemanticSearchService::open(&paths.index_path, &config)
        .map_err(|error| error.to_string())?;
    let service_open_duration_ms = service_open_started_at.elapsed().as_millis();
    let semantic_mode = SemanticSearchMode::from(options.semantic_mode);
    let result = search
        .semantic_with_timing(
            &options.query,
            filter.as_ref(),
            options.limit,
            semantic_mode,
        )
        .map_err(|error| error.to_string())?;
    let hits = result.hits;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ok",
                "query": options.query,
                "limit": options.limit,
                "semantic_mode": semantic_mode.as_str(),
                "timing": {
                    "service_open_duration_ms": service_open_duration_ms,
                    "query_embedding_duration_ms": result.timing.query_embedding_duration_ms,
                    "vector_search_duration_ms": result.timing.vector_search_duration_ms,
                    "semantic_duration_ms": result.timing.total_duration_ms,
                    "total_duration_ms": total_started_at.elapsed().as_millis(),
                },
                "hits": hits.iter().map(|hit| {
                    json!({
                        "record_key": hit.record_key,
                        "embedding_unit_key": hit.embedding_unit_key,
                        "unit_kind": hit.unit_kind,
                        "label": hit.label,
                        "distance": hit.distance,
                        "rank_distance": hit.rank_distance,
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

#[derive(Debug, Clone, Default)]
struct AtlasPathOverrides {
    source_root: Option<PathBuf>,
    embedding_cache_root: Option<PathBuf>,
    index_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ResolvedAtlasPaths {
    mode: ResolvedPathMode,
    repo_root: Option<PathBuf>,
    source_root: PathBuf,
    embedding_cache_root: PathBuf,
    index_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolvedPathMode {
    Repo,
    User,
}

impl ResolvedPathMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::User => "user",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Repo => "repo checkout",
            Self::User => "user install",
        }
    }

    const fn suggested_path_mode(self) -> &'static str {
        self.as_str()
    }
}

struct EmbeddingModelCacheStatus {
    model_dir: PathBuf,
    ready: bool,
    missing_files: Vec<PathBuf>,
}

fn resolve_index_path(
    path_mode: CliPathMode,
    index_override: Option<PathBuf>,
) -> Result<ResolvedAtlasPaths, String> {
    resolve_atlas_paths(
        path_mode,
        AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index_override,
        },
    )
}

fn resolve_atlas_paths(
    path_mode: CliPathMode,
    overrides: AtlasPathOverrides,
) -> Result<ResolvedAtlasPaths, String> {
    let current_dir = std::env::current_dir().map_err(|error| error.to_string())?;
    let repo_root = find_git_repo_root(&current_dir);
    let resolved_mode = match path_mode {
        CliPathMode::Auto => {
            if repo_root.is_some() {
                ResolvedPathMode::Repo
            } else {
                ResolvedPathMode::User
            }
        }
        CliPathMode::Repo => {
            if repo_root.is_none() {
                return Err(
                    "--path-mode repo requires running inside a git checkout with rust/Cargo.toml"
                        .to_string(),
                );
            }
            ResolvedPathMode::Repo
        }
        CliPathMode::User => ResolvedPathMode::User,
    };

    let defaults = match resolved_mode {
        ResolvedPathMode::Repo => {
            let repo_root = repo_root
                .clone()
                .expect("repo path mode only selected when repo root exists");
            AtlasPathOverrides {
                source_root: Some(repo_root.join("vendor").join("pf2e")),
                embedding_cache_root: Some(repo_root.join(".cache").join("hf-models")),
                index_path: Some(repo_root.join(".cache").join("pf2e-rust-index.sqlite")),
            }
        }
        ResolvedPathMode::User => {
            let cache_root = platform_cache_root()?.join("pf2e-atlas");
            AtlasPathOverrides {
                source_root: Some(cache_root.join("vendor").join("pf2e")),
                embedding_cache_root: Some(cache_root.join("hf-models")),
                index_path: Some(cache_root.join("pf2e-rust-index.sqlite")),
            }
        }
    };

    Ok(ResolvedAtlasPaths {
        mode: resolved_mode,
        repo_root: if resolved_mode == ResolvedPathMode::Repo {
            repo_root
        } else {
            None
        },
        source_root: overrides
            .source_root
            .or(defaults.source_root)
            .expect("source default is always resolved"),
        embedding_cache_root: overrides
            .embedding_cache_root
            .or(defaults.embedding_cache_root)
            .expect("embedding cache default is always resolved"),
        index_path: overrides
            .index_path
            .or(defaults.index_path)
            .expect("index default is always resolved"),
    })
}

fn find_git_repo_root(current_dir: &Path) -> Option<PathBuf> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(current_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let root = PathBuf::from(stdout.trim());
    if root.join("rust").join("Cargo.toml").is_file()
        && root
            .join("rust")
            .join("crates")
            .join("atlas-cli")
            .join("Cargo.toml")
            .is_file()
    {
        Some(root)
    } else {
        None
    }
}

fn platform_cache_root() -> Result<PathBuf, String> {
    if cfg!(target_os = "macos") {
        return home_dir()
            .map(|home| home.join("Library").join("Caches"))
            .ok_or_else(|| "could not resolve HOME for user cache path".to_string());
    }
    if cfg!(target_os = "windows") {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data));
        }
        return home_dir()
            .map(|home| home.join("AppData").join("Local"))
            .ok_or_else(|| "could not resolve LOCALAPPDATA or USERPROFILE".to_string());
    }
    if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(cache_home));
    }
    home_dir()
        .map(|home| home.join(".cache"))
        .ok_or_else(|| "could not resolve HOME for user cache path".to_string())
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn embedding_model_cache_status(cache_root: &Path) -> EmbeddingModelCacheStatus {
    let config = EmbeddingRuntimeConfig::new(DEFAULT_EMBEDDING_MODEL, cache_root);
    let model_dir = config.model_dir();
    let required_files = [
        model_dir.join("tokenizer.json"),
        model_dir.join("onnx").join("model.onnx"),
    ];
    let missing_files = required_files
        .into_iter()
        .filter(|path| !path.is_file())
        .collect::<Vec<_>>();
    EmbeddingModelCacheStatus {
        model_dir,
        ready: missing_files.is_empty(),
        missing_files,
    }
}

fn fetch_pf2e_source(source_root: &Path) -> Result<(), String> {
    if source_root.exists() {
        if source_root.join(".git").exists() {
            let status = ProcessCommand::new("git")
                .args([
                    "-C",
                    &source_root.display().to_string(),
                    "pull",
                    "--ff-only",
                ])
                .status()
                .map_err(|error| format!("failed to run git pull: {error}"))?;
            if status.success() {
                return Ok(());
            }
            return Err(format!(
                "failed to update PF2E source at {}",
                source_root.display()
            ));
        }
        return Err(format!(
            "source path already exists but is not a git checkout: {}",
            source_root.display()
        ));
    }
    if let Some(parent) = source_root.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create source parent directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let status = ProcessCommand::new("git")
        .args([
            "clone",
            "https://github.com/foundryvtt/pf2e.git",
            &source_root.display().to_string(),
        ])
        .status()
        .map_err(|error| format!("failed to run git clone: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to clone PF2E source into {}",
            source_root.display()
        ))
    }
}

fn status_label(ok: bool) -> &'static str {
    if ok { "ok" } else { "missing" }
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
