use super::{Result, model::Assets};
use serde_json::{Value, json};
use std::{path::Path, process::Command};

pub(super) fn invoke(binary: &Path, arguments: &[String], cwd: &Path) -> Result<Value> {
    let output = Command::new(binary)
        .args(arguments)
        .current_dir(cwd)
        .output()?;
    let response: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(
        |_| json!({"status":"error", "message":String::from_utf8_lossy(&output.stderr)}),
    );
    Ok(json!({"exit_code":output.status.code().unwrap_or(1), "response":response}))
}
pub(super) fn data(report: &Value) -> Result<Value> {
    if report["exit_code"] != 0 || report["response"]["status"] != "ok" {
        return Err(format!("Atlas check failed: {report}").into());
    }
    report["response"]
        .get("data")
        .cloned()
        .ok_or_else(|| "Atlas response has no data".into())
}
pub(super) fn paths(command: &[&str], index: &Path, assets: &Assets) -> Vec<String> {
    let mut arguments: Vec<String> = command.iter().map(|s| (*s).into()).collect();
    arguments.extend([
        "--path-mode".into(),
        "global".into(),
        "--index".into(),
        index.to_string_lossy().into_owned(),
    ]);
    if let Some(path) = &assets.source {
        arguments.extend(["--source".into(), path.to_string_lossy().into_owned()]);
    }
    if let Some(path) = &assets.embedding_cache {
        arguments.extend([
            "--embedding-cache-path".into(),
            path.to_string_lossy().into_owned(),
        ]);
    }
    arguments
}
pub(super) fn readiness(binary: &Path, index: &Path, assets: &Assets, cwd: &Path) -> Result<Value> {
    invoke(
        binary,
        &paths(&["setup", "--check", "--offline", "--json"], index, assets),
        cwd,
    )
}
pub(super) fn validate(binary: &Path, index: &Path, assets: &Assets, cwd: &Path) -> Result<Value> {
    tracing::info!(target:"atlas_progress",phase="check_baseline_readiness","Checking baseline readiness");
    let ready = data(&readiness(binary, index, assets, cwd)?)?;
    let profiles = ready["search_profiles"]
        .as_array()
        .ok_or("Missing search profiles")?;
    if ready["ready"] != true || profiles.is_empty() || profiles.iter().any(|p| p["ready"] != true)
    {
        return Err("Every configured profile must be ready before publishing".into());
    }
    tracing::info!(target:"atlas_progress",phase="validate_baseline","Validating reference records and vectors");
    let deep = data(&invoke(
        binary,
        &paths(&["index", "validate", "--json"], index, &Assets::default()),
        cwd,
    )?)?;
    let mut searches = Vec::new();
    for profile in profiles {
        let id = profile["profile_id"]
            .as_str()
            .ok_or("Missing profile identity")?;
        let cache = ready["paths"]["embedding_cache"]
            .as_str()
            .ok_or("Missing embedding cache")?;
        let arguments = paths(
            &[
                "search",
                "healing",
                "--retrieval",
                "vector",
                "--limit",
                "1",
                "--search-profile",
                id,
                "--json",
            ],
            index,
            &Assets {
                source: None,
                embedding_cache: Some(cache.into()),
            },
        );
        let phase = format!("check_baseline_search_{id}");
        tracing::info!(target:"atlas_progress",phase=phase.as_str(),profile=id,"Loading and testing {id} search");
        let report = data(&invoke(binary, &arguments, cwd)?)?;
        let results = report["results"]
            .as_array()
            .ok_or("Missing search results")?;
        if results.is_empty() {
            return Err(format!("Semantic query returned no results for {id}").into());
        }
        searches.push(json!({"profile":id,"result_count":results.len()}));
    }
    Ok(json!({"readiness":ready,"deep_validation":deep,"searches":searches}))
}
