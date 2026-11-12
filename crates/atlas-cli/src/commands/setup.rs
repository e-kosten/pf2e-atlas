use std::io::IsTerminal;
use std::process::ExitCode;

use atlas_runtime::{
    AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, RuntimeSetupCleanOptions,
    RuntimeSetupOptions, SetupActionKind, SetupActionStatus, SetupCleanTargetKind,
    SetupCleanTargetStatus, SetupExitClass, SetupReadinessStatus, SetupTarget,
};
use serde::Serialize;

use atlas_cli_support::format_duration_ms;
use atlas_cli_support::{write_json_data, write_json_error};

pub(crate) mod args;

use args::{
    DesktopStage, SetupArgs, SetupCleanOptions, SetupCommand, SetupPathOptions, SetupRunOptions,
};

pub(crate) fn run_setup(args: SetupArgs) -> Result<ExitCode, String> {
    let json = args.paths.json;
    if let Some(offer) = args.desktop_reference_offer.as_deref() {
        if !cfg!(feature = "web")
            || args.command.is_some()
            || !matches!(args.paths.path_mode, atlas_cli_support::CliPathMode::Global)
            || args.paths.source.is_some()
            || args.paths.index.is_some()
            || args.paths.embedding_cache_path.is_some()
            || args.run.check
            || args.run.no_embeddings
            || args.run.offline
            || args.run.force_rebuild
            || args.run.audit_embedding_coverage
            || args.run.locale.is_some()
            || !args.run.search_profiles.is_empty()
            || offer.len() != 64
            || !offer.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return invalid_setup_input(json, "invalid desktop reference refresh options".into());
        }
        #[cfg(feature = "web")]
        {
            let runtime = match setup_runtime(args.paths) {
                Ok(runtime) => runtime,
                Err(message) => return invalid_setup_input(json, message),
            };
            #[cfg(debug_assertions)]
            if runtime.holds_desktop_reference_fixture() {
                tracing::info!(target:"atlas_progress", phase="check_setup", "Checking reference data");
                loop {
                    std::thread::park();
                }
            }
            let report = runtime.refresh_desktop_reference(
                args.run.build_embeddings_locally,
                offer,
                |outcome| {
                    use atlas_desktop_protocol::ReferenceRefreshOutcome;
                    use atlas_runtime::DesktopReferenceOutcome;
                    let outcome = match outcome {
                        DesktopReferenceOutcome::Ready {
                            generation,
                            updated,
                            model_repaired,
                        } => ReferenceRefreshOutcome::Ready {
                            generation,
                            updated,
                            model_repaired,
                        },
                        DesktopReferenceOutcome::SeedUnavailable { retry_offer } => {
                            ReferenceRefreshOutcome::SeedUnavailable { retry_offer }
                        }
                        DesktopReferenceOutcome::Busy => ReferenceRefreshOutcome::Busy,
                    };
                    let event = atlas_desktop_protocol::ReferenceRefreshResult {
                        reference_refresh_version:
                            atlas_desktop_protocol::REFERENCE_REFRESH_VERSION,
                        outcome,
                    };
                    if let Ok(json) = serde_json::to_string(&event) {
                        eprintln!("{json}");
                    }
                },
            );
            if report.ready && !report.search_profiles.is_empty() {
                super::web::managed::notify_semantic_publication(&runtime);
            }
            return finish_setup_report(&report, true, false);
        }
    }
    let fixture = cfg!(debug_assertions)
        && matches!(args.desktop_stage, Some(DesktopStage::FirstRun))
        && std::env::var_os("ATLAS_PILOT_SMOKE")
            .is_some_and(|mode| mode == "normal" || mode == "retry" || mode == "preflight")
        && std::env::var_os("ATLAS_PILOT_HOME")
            .filter(|home| std::env::var_os("HOME").as_ref() == Some(home))
            .is_some_and(|home| {
                args.paths.source.as_ref() == Some(&std::path::PathBuf::from(home).join("source"))
            });
    if args.desktop_stage.is_some() && !cfg!(feature = "web") {
        return invalid_setup_input(
            json,
            "staged desktop setup requires the bundled Atlas command".into(),
        );
    }
    if args.desktop_stage.is_some()
        && (args.command.is_some()
            || (args.paths.source.is_some() && !fixture)
            || args.paths.index.is_some()
            || args.paths.embedding_cache_path.is_some()
            || args.run.audit_embedding_coverage
            || (args.run.offline && !fixture))
    {
        return invalid_setup_input(json, "invalid staged desktop setup options".into());
    }
    match args.command {
        Some(SetupCommand::Clean(options)) => {
            if args.run.check
                || args.run.no_embeddings
                || args.run.offline
                || args.run.force_rebuild
                || args.run.audit_embedding_coverage
                || args.run.build_embeddings_locally
                || !args.run.search_profiles.is_empty()
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
            run_setup_install(runtime, args.run, args.desktop_stage, fixture, json)
        }
    }
}

fn setup_runtime(options: SetupPathOptions) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
        path_mode: crate::cli::runtime_path_mode(options.path_mode),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })
    .map_err(|error| error.to_string())
}

fn run_setup_install(
    runtime: AtlasRuntime,
    options: SetupRunOptions,
    desktop_stage: Option<DesktopStage>,
    fixture: bool,
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
        build_embeddings_locally: options.build_embeddings_locally,
        search_profiles: options.search_profiles,
        embedding_batch_size: options.embedding_batch_size,
        locale: options.locale,
    };
    let report = if let Some(stage) = desktop_stage {
        let stage = match stage {
            DesktopStage::FirstRun if fixture => atlas_runtime::DesktopSetupStage::Fixture,
            DesktopStage::FirstRun => atlas_runtime::DesktopSetupStage::FirstRun,
            DesktopStage::Resume => atlas_runtime::DesktopSetupStage::Resume,
        };
        runtime.staged_desktop_setup(setup_options, stage, |publication| {
            #[cfg(feature = "web")]
            {
                let event = atlas_desktop_protocol::SetupPublication {
                    setup_event_version: atlas_desktop_protocol::SETUP_EVENT_VERSION,
                    workspace: publication.workspace,
                    generation: publication.generation,
                    semantic: publication.semantic,
                };
                if let Ok(json) = serde_json::to_string(&event) {
                    eprintln!("{json}");
                }
                // The packaged smoke test exercises the real first commit and
                // cancels through the product while the semantic stage is held.
                if fixture && !event.semantic {
                    loop {
                        std::thread::park();
                    }
                }
            }
            #[cfg(not(feature = "web"))]
            let _ = publication;
        })
    } else if options.audit_embedding_coverage {
        runtime.audit_embedding_seed(setup_options)?
    } else {
        runtime.ensure_setup(setup_options)
    };
    #[cfg(feature = "web")]
    if desktop_stage.is_some() && report.ready {
        super::web::managed::notify_semantic_publication(&runtime);
    }
    finish_setup_report(&report, json, options.audit_embedding_coverage)
}

fn finish_setup_report(
    report: &atlas_runtime::RuntimeSetupReport,
    json: bool,
    audit_embedding_coverage: bool,
) -> Result<ExitCode, String> {
    tracing::info!(target:"atlas_progress",complete=true,"Setup finished");
    let exit_class = report.exit_code_class();

    if json {
        let mut data = setup_json_data(report);
        if audit_embedding_coverage {
            data.embedding_seed_pins = Some(atlas_runtime::published_embedding_seed_pins()?);
        }
        write_json_data(data)?;
    } else {
        print_setup_report(report);
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
    if !source && !embedding_cache {
        return invalid_setup_clean_input(
            json,
            "setup clean requires at least one of --embeddings, --source-checkout, or --all",
        );
    }
    if source && embedding_cache && !options.check && !options.yes {
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
    inquire::Confirm::new("Remove Atlas source and embedding caches? Saved data will be preserved.")
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
    #[serde(skip_serializing_if = "Option::is_none")]
    embedding_coverage_gap: Option<atlas_runtime::EmbeddingCoverageGap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embedding_seed_pins: Option<atlas_runtime::EmbeddingSeedPins>,
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
    search_profiles: Vec<SetupEmbeddingData>,
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
    profile_id: atlas_domain::SearchProfileId,
    stored: bool,
    model: String,
    model_path: String,
    cache_root: String,
    ready: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SetupBuildData {
    admission: atlas_index::SourceAdmissionSummary,
    profiles: Vec<atlas_ingest::ProfileBuildReport>,
    source_fingerprint: String,
    record_count: usize,
    product_record_count: usize,
    semantic_unit_count: usize,
    reused_inputs: usize,
    inferred_inputs: usize,
    context_shortened_sections: usize,
    build_duration_ms: u128,
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
        report
            .search_profiles
            .iter()
            .map(|p| p.profile_id.to_string())
            .collect::<Vec<_>>()
            .join(", "),
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
            "  source files: {} discovered, {} admitted, {} partial, {} quarantined, {} excluded",
            build.admission.discovered_files,
            build.admission.admitted_records,
            build.admission.partial_records,
            build.admission.quarantined_files,
            build.admission.excluded_documents
        );
        println!(
            "  duration: {}",
            format_duration_ms(build.build_duration_ms)
        );
        println!(
            "  records: {} ({} available in Atlas, {} developer-only macros)",
            build.record_count,
            build.product_record_count,
            build
                .record_count
                .saturating_sub(build.product_record_count)
        );
        println!(
            "  semantic units={} reused inputs={} inferred inputs={} shortened contexts={}",
            build.semantic_unit_count,
            build.reused_inputs,
            build.inferred_inputs,
            build.context_shortened_sections
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
        embedding_coverage_gap: report.embedding_coverage_gap.clone(),
        embedding_seed_pins: None,
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
        search_profiles: report
            .search_profiles
            .iter()
            .map(|profile| SetupEmbeddingData {
                profile_id: profile.profile_id.clone(),
                stored: profile.stored,
                model: profile.model.clone(),
                model_path: profile.model_path.clone(),
                cache_root: profile.cache_root.clone(),
                ready: profile.ready,
                missing_files: profile.missing_files.clone(),
            })
            .collect(),
        build: report.build.as_ref().map(|build| SetupBuildData {
            admission: build.admission.clone(),
            profiles: build.profiles.clone(),
            source_fingerprint: build.source_fingerprint.clone(),
            record_count: build.record_count,
            product_record_count: build.product_record_count,
            semantic_unit_count: build.semantic_unit_count,
            reused_inputs: build.reused_inputs,
            inferred_inputs: build.inferred_inputs,
            context_shortened_sections: build.context_shortened_sections,
            build_duration_ms: build.build_duration_ms,
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
    if action.kind == SetupActionKind::PrepareEmbeddingSeed {
        return "embedding_seed_unavailable";
    }
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
        SetupActionKind::PrepareEmbeddingSeed => "prepare_embedding_seed",
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
