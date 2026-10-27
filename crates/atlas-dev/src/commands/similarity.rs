//! Frozen Similar evaluation orchestration. Product ranking remains in atlas-search.
pub(crate) mod args;
mod execution;
mod model;
mod performance;
#[cfg(test)]
mod tests;

use args::{EvaluateArgs, Phase, SimilarityArgs, SimilarityCommand};
use atlas_cli_support::write_json_data;
use atlas_domain::{RecordKey, SearchProfileId};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::AtlasRetrievalService;
use model::{Manifest, Scenario};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Instant;

pub(crate) fn run(args: SimilarityArgs) -> Result<ExitCode, String> {
    match args.command {
        SimilarityCommand::Evaluate(options) => evaluate(options),
    }
}
fn open_service(
    options: &EvaluateArgs,
    profile: &SearchProfileId,
    manifest: &Manifest,
) -> Result<AtlasRetrievalService, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(profile.clone()),
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: options.index.clone(),
        },
    })
    .map_err(|e| e.to_string())?;
    let index = runtime.open_index().map_err(|e| e.to_string())?;
    let service = AtlasRetrievalService::from_index(index, profile.clone(), None)
        .map_err(|e| e.to_string())?;
    execution::verify_generation(&service, &manifest.artifact_generation)?;
    if service.artifact_context().source_revision.as_deref()
        != Some(manifest.source_revision.as_str())
    {
        return Err("frozen source revision does not match active read snapshot".into());
    }
    Ok(service)
}
fn git_evidence() -> Value {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let get = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(&checkout)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| o.stdout)
    };
    let untracked = get(&[
        "ls-files",
        "--others",
        "--exclude-standard",
        "-z",
        "--",
        "crates",
    ])
    .and_then(|paths| {
        paths
            .split(|b| *b == 0)
            .filter(|p| !p.is_empty())
            .filter_map(|p| std::str::from_utf8(p).ok())
            .filter(|p| p.ends_with(".rs") || p.ends_with(".toml"))
            .map(|name| {
                std::fs::read(checkout.join(name))
                    .ok()
                    .map(|bytes| (name.to_owned(), model::hash(&bytes)))
            })
            .collect::<Option<std::collections::BTreeMap<_, _>>>()
    });
    json!({"head":get(&["rev-parse","HEAD"]).and_then(|x|String::from_utf8(x).ok()).map(|x|x.trim().to_owned()),"status_sha256":get(&["status","--porcelain=v1","-z"]).map(|x|model::hash(&x)),"tracked_diff_sha256":get(&["diff","--binary","HEAD"]).map(|x|model::hash(&x)),"untracked_rust_source_hashes":untracked,"identity_note":"git state is observed at invocation, not proof of the compiler input tree; bind executable hash to the build receipt"})
}
fn write_report(path: &Path, value: &Value) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, value).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())
}
fn phase_result(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => value,
        Err(message) => json!({"status":"phase_failed","message":message}),
    }
}
fn available_variants(
    variants: &[atlas_search::SimilarityEvaluationVariant],
    production: &Value,
) -> Vec<atlas_search::SimilarityEvaluationVariant> {
    let ids = production["variants"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| v["status"] == "ok")
        .filter_map(|v| v["id"].as_str())
        .collect::<BTreeSet<_>>();
    variants
        .iter()
        .filter(|v| ids.contains(v.id.as_str()))
        .cloned()
        .collect()
}
fn scenario_requested(options: &EvaluateArgs, scenario: &Scenario) -> bool {
    options.all_scenarios || options.scenarios.contains(&scenario.id)
}
fn evaluate(options: EvaluateArgs) -> Result<ExitCode, String> {
    let inputs = model::load(&options)?;
    let manifest = &inputs.manifest;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let executable_sha256 = model::hash(&std::fs::read(&executable).map_err(|e| e.to_string())?);
    let provenance = json!({"experiment_id":manifest.experiment_id,"declared_build_revision":options.build_revision,"build_revision_authority":"caller declaration, not inferred compile provenance","executable_sha256":executable_sha256,"runtime_git":git_evidence(),"manifest_sha256":inputs.manifest_sha256,"corpus_sha256":inputs.corpus_sha256,"freeze_sha256":inputs.freeze_sha256,"artifact_generation":manifest.artifact_generation,"source_revision":manifest.source_revision,"process_id":std::process::id(),"peak_rss_bytes":null,"build_features":{"atlas-search":["test-support"],"timing_scope":"complete production algorithm in instrumented developer executable; cross-check shipped CLI/web separately"},"sql_measurement":"instrumented index read statements, not a complete SQLite trace"});
    std::fs::create_dir_all(&options.output_dir).map_err(|e| e.to_string())?;
    let profiles = manifest
        .profiles
        .iter()
        .filter(|p| options.profiles.is_empty() || options.profiles.iter().any(|v| v == p.as_str()))
        .collect::<Vec<_>>();
    let profile_names = profiles.iter().map(|p| p.to_string()).collect::<Vec<_>>();
    let complete_profiles = profiles.len() == manifest.profiles.len();
    let corpus_keys = inputs
        .corpus
        .records
        .iter()
        .map(|r| r.record_key.clone())
        .collect::<Vec<_>>();
    let mut index = Vec::new();
    for scenario in manifest
        .scenarios
        .iter()
        .filter(|s| scenario_requested(&options, s))
    {
        let (variants, skipped) = model::variants(manifest, scenario, &options.variants)?;
        if variants.is_empty() {
            return Err(format!("{}: no applicable selected variants", scenario.id));
        }
        let mut productions = Vec::new();
        let need_production = matches!(
            options.phase,
            Phase::Capture | Phase::FixedPool | Phase::All
        );
        if need_production {
            for profile in &profiles {
                let service = open_service(&options, profile, manifest)?;
                productions.push(phase_result(execution::production(
                    &service,
                    scenario,
                    profile,
                    &variants,
                    options.repetitions.unwrap_or(manifest.repetitions),
                    &manifest.artifact_generation,
                )));
            }
        }
        let mut pool = BTreeSet::new();
        for production in &productions {
            for variant in production["variants"].as_array().into_iter().flatten() {
                for key in variant["returned_top20"].as_array().into_iter().flatten() {
                    pool.insert(
                        serde_json::from_value::<RecordKey>(key.clone())
                            .map_err(|e| e.to_string())?,
                    );
                }
            }
        }
        let pool = pool.into_iter().collect::<Vec<_>>();
        let pool_sha256 = model::hash(&serde_json::to_vec(&pool).map_err(|e| e.to_string())?);
        for (profile_index, profile) in profiles.iter().enumerate() {
            eprintln!(
                "{} / {}: evaluation {:?}",
                scenario.id, profile, options.phase
            );
            let started = Instant::now();
            let service = open_service(&options, profile, manifest)?;
            service.reset_read_metrics();
            let inventory_started = Instant::now();
            let inventory = service
                .similarity_parts(&scenario.seed, Some(profile))
                .map_err(|e| e.to_string())?;
            let inventory_ms = inventory_started.elapsed().as_secs_f64() * 1000.0;
            execution::verify_generation(&service, &manifest.artifact_generation)?;
            let production = productions.get(profile_index).cloned();
            let fixed = if matches!(options.phase, Phase::FixedPool | Phase::All) {
                let valid = production
                    .as_ref()
                    .map(|p| available_variants(&variants, p))
                    .unwrap_or_default();
                if valid.is_empty() {
                    Some(
                        json!({"status":"unavailable","reason":"no successful production variants"}),
                    )
                } else {
                    Some(phase_result(execution::fixed_pool(
                        &service,
                        &execution::Case {
                            scenario,
                            profile,
                            generation: &manifest.artifact_generation,
                        },
                        &valid,
                        &pool,
                        &profile_names,
                        complete_profiles,
                    )))
                }
            } else {
                None
            };
            let oracle = if matches!(options.phase, Phase::Oracle | Phase::All) {
                Some(phase_result(execution::oracle(
                    &service,
                    scenario,
                    profile,
                    &variants,
                    &corpus_keys,
                    &manifest.artifact_generation,
                )))
            } else {
                None
            };
            let performance = if matches!(options.phase, Phase::Performance | Phase::All) {
                Some(phase_result(performance::run(
                    &options,
                    manifest,
                    scenario,
                    profile,
                    &variants,
                    options.phase == Phase::Performance
                        && profiles.len() == 1
                        && manifest
                            .scenarios
                            .iter()
                            .filter(|s| scenario_requested(&options, s))
                            .count()
                            == 1
                        && variants.len() == 1,
                )))
            } else {
                None
            };
            execution::verify_generation(&service, &manifest.artifact_generation)?;
            let report = json!({"version":1,"provenance":provenance,"scenario_id":scenario.id,"split":scenario.split,"comparison_stratum":scenario.stratum,"quality_eligible":scenario.quality_eligible,"baseline_selection_scope":if matches!(scenario.selection,model::Selection::Explicit{..}){"legacy seed, unmatched contextual reference"}else{"whole-seed default reference"},"profile":profile,"skipped_inapplicable_variants":skipped,"effective_variants":variants,"inventory":{"elapsed_ms":inventory_ms,"value":inventory},"production":production,"fixed_pool":fixed,"fixed_pool_sha256":if need_production{Some(&pool_sha256)}else{None},"oracle":oracle,"performance":performance,"case_elapsed_ms":started.elapsed().as_secs_f64()*1000.0});
            let phase = format!("{:?}", options.phase).to_lowercase();
            let path =
                model::report_path(&options.output_dir, &scenario.id, profile.as_str(), &phase);
            write_report(&path, &report)?;
            index.push(json!({"scenario_id":scenario.id,"profile":profile,"path":path,"sha256":model::hash(&std::fs::read(&path).map_err(|e|e.to_string())?)}));
        }
    }
    let summary = json!({"version":1,"provenance":provenance,"phase":format!("{:?}",options.phase).to_lowercase(),"reports":index,"profile_coverage":profile_names,"complete_manifest_profile_coverage":complete_profiles,"note":"Report cells retain classified failures and unavailable metrics; successful command execution is not a quality or performance pass."});
    if options.json {
        write_json_data(summary)?;
    } else {
        println!(
            "{} evaluation reports written to {}",
            index.len(),
            options.output_dir.display()
        );
    }
    Ok(ExitCode::SUCCESS)
}
