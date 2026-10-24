use super::{
    Result,
    args::PublishArgs,
    checks, files,
    model::{Assets, Manifest, Pointer},
};
use atlas_runtime::{
    AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, WORKSPACE_SCHEMA_VERSION,
};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn publish(options: &PublishArgs, checkout: &Path, store: &Path) -> Result<Value> {
    let binary = checkout.join(&options.binary).canonicalize()?;
    let source = checkout.join(&options.index).canonicalize()?;
    let commit = files::git(
        checkout,
        &[
            "rev-parse",
            "--verify",
            &format!("{}^{{commit}}", options.build_commit),
        ],
    )?;
    fs::create_dir_all(store.join("versions"))?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(store.join("publish.lock"))?;
    lock.try_lock()
        .map_err(|error| format!("Baseline publisher busy or unavailable: {error}"))?;
    let stage = tempfile::Builder::new()
        .prefix(".publish-")
        .tempdir_in(store)?;
    let now = time::OffsetDateTime::now_utc();
    let created_at = now.format(&time::format_description::well_known::Rfc3339)?;
    let identifier = format!(
        "{}-{}-{}",
        now.unix_timestamp(),
        &commit[..12],
        stage
            .path()
            .file_name()
            .ok_or("Missing staging name")?
            .to_string_lossy()
            .trim_start_matches(".publish-")
    );
    let executable = stage.path().join("atlas");
    let index = stage.path().join("workspace.sqlite");
    tracing::info!(target:"atlas_progress",phase="capture_binary","Copying and verifying baseline executable");
    let original_hash = files::hash(&binary)?;
    fs::copy(&binary, &executable)?;
    if files::hash(&executable)? != original_hash {
        return Err("Executable changed while copying".into());
    }
    tracing::info!(target:"atlas_progress",phase="capture_workspace","Copying a consistent database snapshot");
    atlas_storage::Workspace::at(&source)?.snapshot_to(&index)?;
    // Read reference provenance through its owning index API. Tool/schema skew
    // requires refreshing atlas-dev; it never causes a migration during capture.
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
        path_mode: AtlasPathMode::Global,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: Some(index.clone()),
        },
    })?;
    tracing::info!(target:"atlas_progress",phase="hash_workspace","Computing database snapshot checksum");
    let database = {
        let reader = runtime.open_index()?;
        json!({"schema_version":WORKSPACE_SCHEMA_VERSION,"workspace_key":reader.workspace_session().workspace_key()?,"reference_revision":reader.workspace_session().reference_revision()?,"context":reader.context(),"sha256":files::hash(&index)?})
    };
    let validation = checks::validate(
        &executable,
        &index,
        &Assets {
            source: options.source.as_ref().map(|p| checkout.join(p)),
            embedding_cache: options
                .embedding_cache_path
                .as_ref()
                .map(|p| checkout.join(p)),
        },
        stage.path(),
    )?;
    let assets: Assets = serde_json::from_value(validation["readiness"]["paths"].clone())?;
    let assets = stable_assets(checkout, assets)?;
    let version = Command::new(&executable).arg("--version").output()?;
    if !version.status.success() {
        return Err("Could not identify supplied Atlas executable".into());
    }
    let manifest = Manifest {
        version: 1,
        id: identifier.clone(),
        created_at,
        build: json!({"commit":commit,"profile":options.build_profile,"toolchain":options.toolchain,"provenance":"publisher-declared; existing executable, no compilation performed","sha256":original_hash,"version":String::from_utf8(version.stdout)?.trim()}),
        database,
        assets,
        captured_from: source,
        validation,
    };
    tracing::info!(target:"atlas_progress",phase="publish_baseline","Publishing validated baseline");
    files::replace(&stage.path().join("manifest.json"), &manifest)?;
    for (filename, mode) in [
        ("atlas", 0o555),
        ("workspace.sqlite", 0o444),
        ("manifest.json", 0o444),
    ] {
        files::mode(&stage.path().join(filename), mode)?;
    }
    let destination = store.join("versions").join(&identifier);
    fs::rename(stage.path(), &destination)?;
    files::replace(
        &store.join("current.json"),
        &Pointer {
            version: 1,
            baseline: identifier.clone(),
        },
    )?;
    tracing::info!(target:"atlas_progress",complete=true,"Baseline published");
    Ok(
        json!({"published":identifier,"directory":destination,"build":manifest.build,"assets":manifest.assets}),
    )
}
fn stable_assets(checkout: &Path, assets: Assets) -> Result<Assets> {
    let common = PathBuf::from(files::git(
        checkout,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?);
    let primary = common
        .parent()
        .ok_or("Missing primary checkout")?
        .canonicalize()?;
    let trees = files::git(checkout, &["worktree", "list", "--porcelain"])?;
    let linked: Vec<PathBuf> = trees
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .map(PathBuf::from)
        .filter(|p| p != &primary)
        .collect();
    let resolve = |path: Option<PathBuf>| -> Result<Option<PathBuf>> {
        let path = path.ok_or("Missing shared asset path")?.canonicalize()?;
        if linked.iter().any(|tree| path.starts_with(tree)) {
            return Err(format!(
                "Shared assets must live outside disposable worktrees: {}",
                path.display()
            )
            .into());
        }
        Ok(Some(path))
    };
    Ok(Assets {
        source: resolve(assets.source)?,
        embedding_cache: resolve(assets.embedding_cache)?,
    })
}
