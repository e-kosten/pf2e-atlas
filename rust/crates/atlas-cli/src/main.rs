#![deny(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use atlas_domain::{ArtifactValidationReport, ValidationCode, ValidationStatus};
use atlas_index::{inspect_index, validate_index};
use atlas_ingest::{
    BuildArtifactOptions, build_artifact, load_foundry_source,
    report::{analyze_source_load, diagnostics_json, skipped_records_json},
};
use clap::{Args, Parser, Subcommand};
use serde_json::json;

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
}

#[derive(Debug, Args)]
struct IndexArgs {
    #[command(subcommand)]
    command: IndexCommand,
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

fn main() -> ExitCode {
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
        Command::Index(index) => match index.command {
            IndexCommand::Analyze(options) => run_index_analyze(options),
            IndexCommand::Build(options) => run_index_build(options),
            IndexCommand::Inspect(options) => run_index_inspect(options),
            IndexCommand::Validate(options) => run_index_validate(options),
        },
    }
}

fn run_index_analyze(options: AnalyzeIndexOptions) -> Result<ExitCode, String> {
    let source = load_foundry_source(&options.source, options.manifest.as_deref())
        .map_err(|error| error.to_string())?;
    let report = analyze_source_load(options.source, source);

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
    }

    Ok(ExitCode::SUCCESS)
}

fn run_index_build(options: BuildIndexOptions) -> Result<ExitCode, String> {
    let report = build_artifact(BuildArtifactOptions {
        source_root: options.source,
        output_path: options.output,
        manifest_path: options.manifest,
    })
    .map_err(|error| error.to_string())?;

    if options.json {
        let body = json!({
            "status": "ok",
            "output": report.output_path.display().to_string(),
            "pack_count": report.pack_count,
            "record_count": report.record_count,
            "source_signature": report.source_signature,
            "diagnostics": diagnostics_json(&report.diagnostics),
            "skipped_record_count": report.skipped_records.len(),
            "skipped_records": skipped_records_json(&report.skipped_records),
            "warnings": report.warnings,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&body).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "ok: wrote {} records from {} packs to {}",
            report.record_count,
            report.pack_count,
            report.output_path.display()
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
            report.tables.get("records").copied().unwrap_or_default(),
            report.tables.get("packs").copied().unwrap_or_default(),
            report
                .tables
                .get("reference_edges")
                .copied()
                .unwrap_or_default(),
            report
                .tables
                .get("record_aliases")
                .copied()
                .unwrap_or_default(),
            report
                .tables
                .get("remaster_links")
                .copied()
                .unwrap_or_default()
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
    let report = match validate_index(&options.index) {
        Ok(report) => report,
        Err(error) => ArtifactValidationReport {
            status: ValidationStatus::Error,
            code: match error {
                atlas_index::IndexValidationError::Unavailable(_) => {
                    ValidationCode::IndexUnavailable
                }
                atlas_index::IndexValidationError::QueryFailed(_) => ValidationCode::QueryFailed,
                atlas_index::IndexValidationError::InvalidArtifact(_) => {
                    ValidationCode::InvalidSourceMetadata
                }
            },
            index: options.index.display().to_string(),
            message: error.to_string(),
            artifact_contract_version: None,
            schema_version: None,
            source_kind: None,
            source_signature: None,
            source_record_count: None,
            content_hash_algorithm: None,
            embedding_provider_family: None,
            embedding_model_id: None,
            embedding_model_revision: None,
            embedding_tokenizer_id: None,
            embedding_pooling: None,
            embedding_normalization: None,
            embedding_dimensions: None,
            embedding_dtype: None,
            embedding_distance_metric: None,
            embedding_document_prefix: None,
            embedding_query_prefix: None,
            fts_tokenizer: None,
            adjacent_manifest_path: None,
            missing_keys: Vec::new(),
            diagnostics: Vec::new(),
            legacy_schema_version: None,
        },
    };

    let exit_code = match report.status {
        ValidationStatus::Ok => ExitCode::SUCCESS,
        ValidationStatus::Error => ExitCode::from(1),
    };

    if options.json {
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
        ValidationCode::QueryFailed => "query-failed",
    }
}
