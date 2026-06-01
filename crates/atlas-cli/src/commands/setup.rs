use std::io::IsTerminal;
use std::process::ExitCode;

use atlas_runtime::{
    AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, RuntimeSetupCleanOptions,
    RuntimeSetupOptions, SetupActionKind, SetupActionStatus, SetupCleanTargetKind,
    SetupCleanTargetStatus, SetupExitClass, SetupReadinessStatus, SetupTarget,
};
use serde::Serialize;

use crate::output::{format_duration_ms, write_json_data, write_json_error};
use crate::{SetupArgs, SetupCleanOptions, SetupCommand, SetupPathOptions, SetupRunOptions};

pub(crate) fn run_setup(args: SetupArgs) -> Result<ExitCode, String> {
    let json = args.paths.json;
    match args.command {
        Some(SetupCommand::Clean(options)) => {
            if args.run.check
                || args.run.no_embeddings
                || args.run.offline
                || args.run.force_rebuild
            {
                return invalid_setup_input(
                    json,
                    "setup install options must appear on atlas setup, not atlas setup clean; use setup clean --check for cleanup dry runs".to_string(),
                );
            }
            let runtime = match setup_runtime(args.paths) {
                Ok(runtime) => runtime,
                Err(message) => return invalid_setup_input(json, message),
            };
            run_setup_clean(runtime, options, json)
        }
        None => {
            let runtime = match setup_runtime(args.paths) {
                Ok(runtime) => runtime,
                Err(message) => return invalid_setup_input(json, message),
            };
            run_setup_install(runtime, args.run, json)
        }
    }
}

fn setup_runtime(options: SetupPathOptions) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })
}

fn run_setup_install(
    runtime: AtlasRuntime,
    options: SetupRunOptions,
    json: bool,
) -> Result<ExitCode, String> {
    let setup_options = RuntimeSetupOptions {
        target: if options.no_embeddings {
            SetupTarget::Records
        } else {
            SetupTarget::Full
        },
        check: options.check,
        offline: options.offline,
        force_rebuild: options.force_rebuild,
        embedding_model_id: options.embedding_model,
        embedding_batch_size: options.embedding_batch_size,
    };
    let report = runtime.ensure_setup(setup_options);
    let exit_class = report.exit_code_class();

    if json {
        write_json_data(setup_json_data(&report))?;
    } else {
        print_setup_report(&report);
    }

    Ok(match exit_class {
        SetupExitClass::Success => ExitCode::SUCCESS,
        SetupExitClass::NotReady => ExitCode::from(1),
        SetupExitClass::RuntimeFailure => ExitCode::from(3),
    })
}

fn run_setup_clean(
    runtime: AtlasRuntime,
    options: SetupCleanOptions,
    json: bool,
) -> Result<ExitCode, String> {
    let source = options.all || options.source_checkout;
    let embedding_cache = options.all || options.embeddings;
    let artifact = options.all || options.artifact;
    if !source && !embedding_cache && !artifact {
        return invalid_setup_clean_input(
            json,
            "setup clean requires at least one of --artifact, --embeddings, --source-checkout, or --all",
        );
    }
    if source && embedding_cache && artifact && !options.check && !options.yes {
        if json || !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
            return invalid_setup_clean_input(
                json,
                "setup clean all-target cleanup requires --yes unless --check is used",
            );
        }
        if !confirm_setup_clean_all()? {
            return Ok(ExitCode::from(1));
        }
    }

    let report = runtime.clean_setup(RuntimeSetupCleanOptions {
        source,
        embedding_cache,
        artifact,
        check: options.check,
    });
    let exit_class = report.exit_code_class();
    if json {
        write_json_data(setup_clean_json_data(&report))?;
    } else {
        print_setup_clean_report(&report);
    }

    Ok(match exit_class {
        SetupExitClass::Success => ExitCode::SUCCESS,
        SetupExitClass::NotReady => ExitCode::from(1),
        SetupExitClass::RuntimeFailure => ExitCode::from(3),
    })
}

fn invalid_setup_clean_input(json: bool, message: &'static str) -> Result<ExitCode, String> {
    invalid_setup_input(json, message.to_string())
}

fn confirm_setup_clean_all() -> Result<bool, String> {
    inquire::Confirm::new("Remove Atlas source, embedding cache, and artifact files?")
        .with_default(false)
        .prompt()
        .map_err(|error| format!("setup clean cancelled: {error}"))
}

fn invalid_setup_input(json: bool, message: String) -> Result<ExitCode, String> {
    if json {
        write_json_error("invalid_input", message)?;
        Ok(ExitCode::from(2))
    } else {
        Err(message)
    }
}

#[derive(Debug, Serialize)]
struct SetupData {
    target: &'static str,
    ready: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    not_ready_reasons: Vec<SetupNotReadyReasonData>,
    path_mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_root: Option<String>,
    offline: bool,
    check: bool,
    force_rebuild: bool,
    checks: Vec<SetupActionData>,
    actions: Vec<SetupActionData>,
    readiness: SetupReadinessData,
    paths: SetupPathsData,
    embedding: SetupEmbeddingData,
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<SetupBuildData>,
}

#[derive(Debug, Serialize)]
struct SetupActionData {
    kind: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct SetupNotReadyReasonData {
    code: &'static str,
    message: String,
    action: &'static str,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct SetupReadinessData {
    source: SetupReadinessItemData,
    embedding_model: SetupReadinessItemData,
    records: SetupReadinessItemData,
    semantic_search: SetupReadinessItemData,
}

#[derive(Debug, Serialize)]
struct SetupReadinessItemData {
    status: &'static str,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct SetupPathsData {
    source: String,
    embedding_cache: String,
    index: String,
}

#[derive(Debug, Serialize)]
struct SetupEmbeddingData {
    model: String,
    model_path: String,
    cache_root: String,
    ready: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SetupBuildData {
    source_signature: String,
    source_record_count: usize,
    artifact_record_count: usize,
    generated_record_count: usize,
    pending_document_embedding_count: usize,
    document_embedding_count: usize,
    reused_document_embedding_count: usize,
    generated_document_embedding_count: usize,
    build_duration_ms: u128,
    embedding_tokenization_duration_ms: u128,
    embedding_model_load_duration_ms: u128,
    embedding_generation_duration_ms: u128,
}

#[derive(Debug, Serialize)]
struct SetupCleanData {
    path_mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_root: Option<String>,
    check: bool,
    targets: Vec<SetupCleanTargetData>,
}

#[derive(Debug, Serialize)]
struct SetupCleanTargetData {
    kind: &'static str,
    status: &'static str,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

fn print_setup_report(report: &atlas_runtime::RuntimeSetupReport) {
    println!("mode: {}", report.path_mode);
    if let Some(repo_root) = &report.repo_root {
        println!("repo root: {repo_root}");
    }
    println!("target: {}", target_label(report.target));
    println!(
        "source: {}",
        readiness_label(&report.readiness.source.status)
    );
    println!(
        "embedding model: {} {}",
        report.embedding.model,
        readiness_label(&report.readiness.embedding_model.status)
    );
    println!("index: {}", report.paths.index);
    println!(
        "records: {}",
        readiness_label(&report.readiness.records.status)
    );
    println!(
        "semantic search: {}",
        readiness_label(&report.readiness.semantic_search.status)
    );
    if !report.checks.is_empty() {
        println!();
        println!("checks:");
        for check in &report.checks {
            print_setup_step(check);
        }
    }
    if !report.actions.is_empty() {
        println!();
        println!("actions:");
        for action in &report.actions {
            print_setup_step(action);
        }
    }
    if !report.ready {
        println!();
        println!("not ready: run atlas setup without --check after resolving blocked actions");
    }
    if let Some(build) = &report.build {
        println!();
        println!("build:");
        println!(
            "  duration: {}",
            format_duration_ms(build.build_duration_ms)
        );
        println!(
            "  records: source={} generated={} artifact={}",
            build.source_record_count, build.generated_record_count, build.artifact_record_count
        );
        println!(
            "  embeddings: pending_document={} document={} reused={} generated={}",
            build.pending_document_embedding_count,
            build.document_embedding_count,
            build.reused_document_embedding_count,
            build.generated_document_embedding_count
        );
        println!(
            "  embedding timing: tokenization={} model_load={} generation={}",
            format_duration_ms(build.embedding_tokenization_duration_ms),
            format_duration_ms(build.embedding_model_load_duration_ms),
            format_duration_ms(build.embedding_generation_duration_ms)
        );
    }
}

fn print_setup_clean_report(report: &atlas_runtime::RuntimeSetupCleanReport) {
    println!("mode: {}", report.path_mode);
    if let Some(repo_root) = &report.repo_root {
        println!("repo root: {repo_root}");
    }
    println!("check: {}", report.check);
    println!();
    println!("targets:");
    for target in &report.targets {
        let reason = target
            .reason
            .as_ref()
            .map(|reason| format!(": {reason}"))
            .unwrap_or_default();
        println!(
            "  {}: {} {}{}",
            clean_target_kind_label(&target.kind),
            clean_target_status_label(&target.status),
            target.path,
            reason
        );
    }
}

fn setup_json_data(report: &atlas_runtime::RuntimeSetupReport) -> SetupData {
    SetupData {
        target: target_label(report.target),
        ready: report.ready,
        not_ready_reasons: setup_not_ready_reasons(report),
        path_mode: report.path_mode,
        repo_root: report.repo_root.clone(),
        offline: report.offline,
        check: report.check,
        force_rebuild: report.force_rebuild,
        checks: report.checks.iter().map(action_json).collect(),
        actions: report.actions.iter().map(action_json).collect(),
        readiness: readiness_json(&report.readiness),
        paths: SetupPathsData {
            source: report.paths.source.clone(),
            embedding_cache: report.paths.embedding_cache.clone(),
            index: report.paths.index.clone(),
        },
        embedding: SetupEmbeddingData {
            model: report.embedding.model.clone(),
            model_path: report.embedding.model_path.clone(),
            cache_root: report.embedding.cache_root.clone(),
            ready: report.embedding.ready,
            missing_files: report.embedding.missing_files.clone(),
        },
        build: report.build.as_ref().map(|build| SetupBuildData {
            source_signature: build.source_signature.clone(),
            source_record_count: build.source_record_count,
            artifact_record_count: build.artifact_record_count,
            generated_record_count: build.generated_record_count,
            pending_document_embedding_count: build.pending_document_embedding_count,
            document_embedding_count: build.document_embedding_count,
            reused_document_embedding_count: build.reused_document_embedding_count,
            generated_document_embedding_count: build.generated_document_embedding_count,
            build_duration_ms: build.build_duration_ms,
            embedding_tokenization_duration_ms: build.embedding_tokenization_duration_ms,
            embedding_model_load_duration_ms: build.embedding_model_load_duration_ms,
            embedding_generation_duration_ms: build.embedding_generation_duration_ms,
        }),
    }
}

fn setup_not_ready_reasons(
    report: &atlas_runtime::RuntimeSetupReport,
) -> Vec<SetupNotReadyReasonData> {
    if report.ready {
        return Vec::new();
    }
    report
        .checks
        .iter()
        .chain(report.actions.iter())
        .filter(|action| {
            matches!(
                action.status,
                SetupActionStatus::Planned | SetupActionStatus::Blocked | SetupActionStatus::Failed
            )
        })
        .map(|action| {
            let message = action.reason.clone().unwrap_or_else(|| {
                format!(
                    "{} is {}",
                    action_kind_label(&action.kind),
                    action_status_label(&action.status)
                )
            });
            SetupNotReadyReasonData {
                code: setup_not_ready_code(action),
                message,
                action: action_kind_label(&action.kind),
                status: action_status_label(&action.status),
            }
        })
        .collect()
}

fn setup_not_ready_code(action: &atlas_runtime::SetupAction) -> &'static str {
    match action.reason.as_deref() {
        Some("source signature changed since the artifact was built") => {
            "artifact_stale_source_signature"
        }
        Some("source checkout is missing") | Some("source checkout is not ready") => {
            "source_not_ready"
        }
        Some("embedding model cache is missing required files")
        | Some("embedding model cache is not ready") => "embedding_model_not_ready",
        Some("force rebuild requested") => "force_rebuild_requested",
        Some("offline") => "offline_blocked",
        _ => match action.status {
            SetupActionStatus::Planned => "action_planned",
            SetupActionStatus::Blocked => "action_blocked",
            SetupActionStatus::Failed => "action_failed",
            SetupActionStatus::Done | SetupActionStatus::Skipped => "not_ready",
        },
    }
}

fn print_setup_step(action: &atlas_runtime::SetupAction) {
    let reason = action
        .reason
        .as_ref()
        .map(|reason| format!(": {reason}"))
        .unwrap_or_default();
    println!(
        "  {}: {}{}",
        action_kind_label(&action.kind),
        action_status_label(&action.status),
        reason
    );
}

fn setup_clean_json_data(report: &atlas_runtime::RuntimeSetupCleanReport) -> SetupCleanData {
    SetupCleanData {
        path_mode: report.path_mode,
        repo_root: report.repo_root.clone(),
        check: report.check,
        targets: report
            .targets
            .iter()
            .map(|target| SetupCleanTargetData {
                kind: clean_target_kind_label(&target.kind),
                status: clean_target_status_label(&target.status),
                path: target.path.clone(),
                reason: target.reason.clone(),
            })
            .collect(),
    }
}

fn action_json(action: &atlas_runtime::SetupAction) -> SetupActionData {
    SetupActionData {
        kind: action_kind_label(&action.kind),
        status: action_status_label(&action.status),
        reason: action.reason.clone(),
    }
}

fn readiness_json(readiness: &atlas_runtime::SetupReadiness) -> SetupReadinessData {
    SetupReadinessData {
        source: readiness_item_json(&readiness.source),
        embedding_model: readiness_item_json(&readiness.embedding_model),
        records: readiness_item_json(&readiness.records),
        semantic_search: readiness_item_json(&readiness.semantic_search),
    }
}

fn readiness_item_json(item: &atlas_runtime::SetupReadinessItem) -> SetupReadinessItemData {
    SetupReadinessItemData {
        status: readiness_json_label(&item.status),
        required: item.required,
        reason: item.reason.clone(),
    }
}

fn target_label(target: SetupTarget) -> &'static str {
    target.as_str()
}

fn action_kind_label(kind: &SetupActionKind) -> &'static str {
    match kind {
        SetupActionKind::FetchSource => "fetch_source",
        SetupActionKind::PrepareEmbeddingModel => "prepare_embedding_model",
        SetupActionKind::AnalyzeSource => "analyze_source",
        SetupActionKind::BuildIndex => "build_index",
        SetupActionKind::ValidateIndex => "validate_index",
    }
}

fn readiness_label(status: &SetupReadinessStatus) -> &'static str {
    match status {
        SetupReadinessStatus::Ready => "ready",
        SetupReadinessStatus::NotReady => "not ready",
        SetupReadinessStatus::Skipped => "skipped",
    }
}

fn readiness_json_label(status: &SetupReadinessStatus) -> &'static str {
    match status {
        SetupReadinessStatus::Ready => "ready",
        SetupReadinessStatus::NotReady => "not_ready",
        SetupReadinessStatus::Skipped => "skipped",
    }
}

fn action_status_label(status: &SetupActionStatus) -> &'static str {
    match status {
        SetupActionStatus::Planned => "planned",
        SetupActionStatus::Done => "done",
        SetupActionStatus::Skipped => "skipped",
        SetupActionStatus::Blocked => "blocked",
        SetupActionStatus::Failed => "failed",
    }
}

fn clean_target_kind_label(kind: &SetupCleanTargetKind) -> &'static str {
    match kind {
        SetupCleanTargetKind::Source => "source",
        SetupCleanTargetKind::EmbeddingCache => "embedding_cache",
        SetupCleanTargetKind::Artifact => "artifact",
    }
}

fn clean_target_status_label(status: &SetupCleanTargetStatus) -> &'static str {
    match status {
        SetupCleanTargetStatus::Planned => "planned",
        SetupCleanTargetStatus::Removed => "removed",
        SetupCleanTargetStatus::Skipped => "skipped",
        SetupCleanTargetStatus::Failed => "failed",
    }
}
