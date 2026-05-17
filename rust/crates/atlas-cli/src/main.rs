#![deny(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use atlas_domain::SearchFilterNode;
use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingModelId};
use atlas_index::{ArtifactValidationReport, ValidationCode, ValidationStatus};
use atlas_ingest::{
    BuildArtifactOptions, analyze_foundry_source, build_artifact, build_artifact_json,
};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{AtlasSearchRequest, AtlasSearchResult, SemanticSearchMode};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;

mod progress;

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

impl From<CliPathMode> for AtlasPathMode {
    fn from(mode: CliPathMode) -> Self {
        match mode {
            CliPathMode::Auto => Self::Auto,
            CliPathMode::Repo => Self::Repo,
            CliPathMode::User => Self::User,
        }
    }
}

fn main() -> ExitCode {
    progress::init_tracing();
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
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
        },
        Command::Search(search) => match search.command {
            SearchCommand::Semantic(options) => run_search_semantic(options),
        },
    }
}

fn run_setup(options: SetupOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })?;

    if options.fetch_source {
        runtime.fetch_source()?;
    }

    let paths = runtime.paths();
    let status = runtime.setup_status();
    let ready = status.ready();

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": if ready { "ready" } else { "not_ready" },
                "path_mode": paths.mode.as_str(),
                "repo_root": paths.repo_root.as_ref().map(|path| path.display().to_string()),
                "source": {
                    "path": paths.source_root.display().to_string(),
                    "exists": status.source_exists,
                },
                "embedding": {
                    "model": status.embedding_model,
                    "cache_root": paths.embedding_cache_root.display().to_string(),
                    "model_path": status.model_cache.model_dir.display().to_string(),
                    "ready": status.model_cache.ready,
                    "missing_files": status.model_cache
                        .missing_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>(),
                },
                "index": {
                    "path": paths.index_path.display().to_string(),
                    "exists": status.index_exists,
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
            status_label(status.source_exists),
            paths.source_root.display()
        );
        println!("embedding model: {}", status.embedding_model);
        println!(
            "embedding cache: {} {}",
            status_label(status.model_cache.ready),
            status.model_cache.model_dir.display()
        );
        for missing_file in &status.model_cache.missing_files {
            println!("missing model file: {}", missing_file.display());
        }
        println!(
            "index: {} {}",
            status_label(status.index_exists),
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
            if !status.source_exists {
                println!("  run atlas setup --fetch-source");
            }
            if !status.model_cache.ready {
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
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: None,
            index_path: None,
        },
    })?;
    let paths = runtime.paths();
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
    let no_embeddings = options.no_embeddings;
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.output,
        },
    })?;
    let paths = runtime.paths();
    let report = build_artifact(BuildArtifactOptions {
        source_root: paths.source_root.clone(),
        output_path: paths.index_path.clone(),
        manifest_path: options.manifest,
        embedding_model_id: options.embedding_model.to_string(),
        embedding_cache_root: if no_embeddings {
            None
        } else {
            Some(paths.embedding_cache_root.clone())
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
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime
        .open_index()
        .map_err(|error| error.to_string())?
        .inspect()
        .map_err(|error| error.to_string())?;

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
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime.validate_index_report();
    write_validation_report(report, options.json)
}

fn run_index_validate_vectors(options: IndexPathOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime.validate_vector_index_report();
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

    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })?;
    let service_open_started_at = Instant::now();
    let mut search = runtime
        .open_retrieval_service_with_model(options.embedding_model.to_string())
        .map_err(|error| error.to_string())?;
    let service_open_duration_ms = service_open_started_at.elapsed().as_millis();
    let semantic_mode = SemanticSearchMode::from(options.semantic_mode);
    let AtlasSearchResult::Semantic(result) = search
        .search(AtlasSearchRequest::Semantic {
            query: &options.query,
            filter: filter.as_ref(),
            limit: options.limit,
            mode: semantic_mode,
        })
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

fn index_runtime(path_mode: AtlasPathMode, index: Option<PathBuf>) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })
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
