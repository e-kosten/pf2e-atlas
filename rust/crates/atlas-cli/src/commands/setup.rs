use std::process::ExitCode;

use atlas_runtime::{
    AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, RuntimeSetupOptions, SetupActionKind,
    SetupActionStatus, SetupExitClass, SetupReadinessStatus, SetupTarget,
};
use serde::Serialize;

use crate::SetupOptions;
use crate::output::write_json_data;

pub(crate) fn run_setup(options: SetupOptions) -> Result<ExitCode, String> {
    let json = options.json;
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })?;
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

#[derive(Debug, Serialize)]
struct SetupData {
    target: &'static str,
    ready: bool,
    path_mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_root: Option<String>,
    offline: bool,
    check: bool,
    force_rebuild: bool,
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
    document_embedding_count: usize,
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
    if !report.actions.is_empty() {
        println!();
        println!("actions:");
        for action in &report.actions {
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
    }
    if !report.ready {
        println!();
        println!("not ready: run atlas setup without --check after resolving blocked actions");
    }
}

fn setup_json_data(report: &atlas_runtime::RuntimeSetupReport) -> SetupData {
    SetupData {
        target: target_label(report.target),
        ready: report.ready,
        path_mode: report.path_mode,
        repo_root: report.repo_root.clone(),
        offline: report.offline,
        check: report.check,
        force_rebuild: report.force_rebuild,
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
            document_embedding_count: build.document_embedding_count,
        }),
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
