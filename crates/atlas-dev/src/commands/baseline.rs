//! Contributor orchestration; database operations remain storage/index-owned.
pub(crate) mod args;
mod checks;
mod files;
mod model;
mod preview;
mod publish;
#[cfg(test)]
mod tests;
use args::{BaselineArgs, BaselineCommand, Candidate};
use model::{Manifest, Pointer, Receipt};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn run(args: BaselineArgs) -> std::result::Result<ExitCode, String> {
    execute(args).map_err(|error| error.to_string())
}
fn execute(args: BaselineArgs) -> Result<ExitCode> {
    if matches!(args.command, BaselineCommand::Capabilities) {
        println!("{}", serde_json::to_string_pretty(&capabilities())?);
        return Ok(ExitCode::SUCCESS);
    }
    let checkout = PathBuf::from(files::git(
        &args.checkout,
        &["rev-parse", "--show-toplevel"],
    )?)
    .canonicalize()?;
    let common = PathBuf::from(files::git(
        &checkout,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?);
    let store = args.store.map(|p| checkout.join(p)).unwrap_or(
        common
            .parent()
            .ok_or("No common Git parent")?
            .join(".atlas/dev-baselines"),
    );
    let (report, code) = match args.command {
        BaselineCommand::Capabilities => (capabilities(), ExitCode::SUCCESS),
        BaselineCommand::Publish(options) => (
            publish::publish(&options, &checkout, &store)?,
            ExitCode::SUCCESS,
        ),
        BaselineCommand::Init(selection) => (
            initialize(&checkout, &store, selection.baseline.as_deref())?,
            ExitCode::SUCCESS,
        ),
        BaselineCommand::Status(candidate) => {
            let report = status(&checkout, &store, &candidate)?;
            let failed = report["exists"] == true
                && (report["readiness"]["exit_code"] != 0
                    || report["readiness"]["response"]["data"]["ready"] != true
                    || report["compatibility"]["exit_code"] != 0
                    || report["compatibility"]["response"]["status"] != "ok");
            (
                report,
                if failed {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                },
            )
        }
        BaselineCommand::Web(options) => {
            return if options.managed {
                preview::run(&checkout, &store, &options)
            } else {
                web(&checkout, &store, &options.candidate)
            };
        }
        BaselineCommand::Refresh(options) => (
            preview::request(
                &checkout,
                preview::Action::Refresh {
                    binary: checkout.join(options.binary),
                    frontend_index: checkout.join(options.frontend_index),
                    build_provenance: options.build_provenance.map(|path| checkout.join(path)),
                },
            )?,
            ExitCode::SUCCESS,
        ),
        BaselineCommand::Stop => (
            preview::request(&checkout, preview::Action::Stop)?,
            ExitCode::SUCCESS,
        ),
    };
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }
    Ok(code)
}
fn capabilities() -> Value {
    json!({"version":1,"managed_preview":true,"workspace_schema_version":atlas_runtime::WORKSPACE_SCHEMA_VERSION})
}

fn load(store: &Path, selected: Option<&str>, verify_data: bool) -> Result<(PathBuf, Manifest)> {
    let selected = match selected {
        Some(id) => id.to_owned(),
        None => {
            let pointer: Pointer = files::read(&store.join("current.json"))?;
            if pointer.version != 1 {
                return Err("Unsupported baseline pointer version".into());
            }
            pointer.baseline
        }
    };
    if selected.is_empty()
        || !selected.as_bytes()[0].is_ascii_alphanumeric()
        || !selected
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
    {
        return Err("Invalid baseline identity".into());
    }
    let directory = store.join("versions").join(&selected);
    let manifest: Manifest = files::read(&directory.join("manifest.json"))?;
    if manifest.version != 1 || manifest.id != selected {
        return Err("Unsupported or mismatched baseline manifest".into());
    }
    if files::hash(&directory.join("atlas"))?
        != manifest.build["sha256"]
            .as_str()
            .ok_or("Missing executable hash")?
    {
        return Err("Baseline executable hash mismatch".into());
    }
    if verify_data && files::hash(&directory.join("workspace.sqlite"))? != seed_hash(&manifest)? {
        return Err("Baseline database hash mismatch".into());
    }
    Ok((directory, manifest))
}
fn seed_hash(manifest: &Manifest) -> Result<&str> {
    manifest.database["sha256"]
        .as_str()
        .ok_or_else(|| "Missing seed hash".into())
}
fn workspace_paths(checkout: &Path) -> Result<(PathBuf, PathBuf)> {
    let directory = checkout.join(".atlas");
    if directory.is_symlink() {
        return Err("Checkout .atlas must be private, not a symlink".into());
    }
    Ok((
        directory.join("workspace.sqlite"),
        directory.join("dev-workspace.json"),
    ))
}
fn initialize(checkout: &Path, store: &Path, selected: Option<&str>) -> Result<Value> {
    let (index, receipt) = workspace_paths(checkout)?;
    let preserved = || json!({"initialized":false,"index":index,"reason":"Existing workspace or receipt preserved; no files replaced"});
    // Check before canonicalization as well, preserving dangling symlinks.
    if files::exists(&index) || files::exists(&receipt) {
        return Ok(preserved());
    }
    let workspace = atlas_storage::Workspace::at(&index)?;
    let Some(admission) = workspace.begin_initialization()? else {
        return Ok(preserved());
    };
    if files::exists(&receipt) {
        return Ok(preserved());
    }
    let (directory, manifest) = load(store, selected, true)?;
    let expected = seed_hash(&manifest)?;
    let method = admission.install(&directory.join("workspace.sqlite"), |path| {
        let actual = files::hash(path)
            .map_err(|error| atlas_storage::StorageError::Unsupported(error.to_string()))?;
        if actual != expected {
            return Err(atlas_storage::StorageError::Unsupported(
                "Workspace copy hash mismatch".into(),
            ));
        }
        Ok(())
    })?;
    files::replace(
        &receipt,
        &Receipt {
            version: 1,
            baseline: manifest.id.clone(),
            store: store.canonicalize()?,
            seed_sha256: expected.into(),
            copy_method: method.into(),
        },
    )?;
    Ok(json!({"initialized":true,"index":index,"baseline":manifest.id,"copy_method":method}))
}
fn workspace_baseline(
    checkout: &Path,
    store: &Path,
    selected: Option<&str>,
) -> Result<(PathBuf, PathBuf, Manifest)> {
    let (index, receipt) = workspace_paths(checkout)?;
    let (directory, manifest) = if receipt.is_file() {
        let receipt: Receipt = files::read(&receipt)?;
        if receipt.version != 1 {
            return Err("Unsupported workspace receipt".into());
        }
        load(&receipt.store, Some(&receipt.baseline), false)?
    } else {
        load(store, selected, false)?
    };
    Ok((index, directory, manifest))
}
fn binary(checkout: &Path, directory: &Path, candidate: &Candidate) -> Result<PathBuf> {
    Ok(match &candidate.binary {
        Some(p) => checkout.join(p).canonicalize()?,
        None => directory.join("atlas"),
    })
}
fn web_arguments(index: &Path, manifest: &Manifest, port: u16) -> Vec<String> {
    checks::paths(
        &["web", "--port", &port.to_string()],
        index,
        &manifest.assets,
    )
}
fn status(checkout: &Path, store: &Path, candidate: &Candidate) -> Result<Value> {
    let (index, directory, manifest) =
        workspace_baseline(checkout, store, candidate.selection.baseline.as_deref())?;
    let binary = binary(checkout, &directory, candidate)?;
    let mut result = json!({"baseline":manifest.id,"baseline_directory":directory,"baseline_build":manifest.build,"seed":manifest.database,"assets":manifest.assets,"checkout":checkout,"checkout_commit":files::git(checkout,&["rev-parse","HEAD"])? ,"binary":binary,"binary_sha256":files::hash(&binary)?,"binary_selection":if candidate.binary.is_some(){"explicit candidate; checkout commit is not executable provenance"}else{"frozen baseline build (use --binary for a candidate)"},"index":index,"exists":index.is_file(),"managed_preview":preview::status(checkout)?});
    if index.is_file() {
        result["readiness"] = checks::readiness(&binary, &index, &manifest.assets, checkout)?;
        result["compatibility"] = checks::invoke(
            &binary,
            &checks::paths(
                &["index", "check", "--json"],
                &index,
                &model::Assets::default(),
            ),
            checkout,
        )?;
    } else {
        result["next_step"] = json!("Run just dev-workspace-init to create a private working copy");
    }
    result["web_command"] = json!(files::quoted_command(
        &binary,
        &web_arguments(&index, &manifest, candidate.port)
    ));
    result["setup_command"] = json!(files::quoted_command(
        &binary,
        &checks::paths(&["setup", "--offline"], &index, &manifest.assets)
    ));
    Ok(result)
}
fn web(checkout: &Path, store: &Path, candidate: &Candidate) -> Result<ExitCode> {
    let (index, directory, manifest) =
        workspace_baseline(checkout, store, candidate.selection.baseline.as_deref())?;
    if !index.is_file() {
        return Err("No private workspace; run just dev-workspace-init first".into());
    }
    let binary = binary(checkout, &directory, candidate)?;
    let ready = checks::data(&checks::readiness(
        &binary,
        &index,
        &manifest.assets,
        checkout,
    )?)?;
    if ready["ready"] != true {
        return Err(
            "Candidate requires setup; run just dev-status for the explicit command".into(),
        );
    }
    let args = web_arguments(&index, &manifest, candidate.port);
    let selection = if candidate.binary.is_some() {
        "selected candidate"
    } else {
        "frozen baseline"
    };
    eprintln!(
        "Running: {selection}\nBinary: {}\nWorkspace: {}",
        binary.display(),
        index.display()
    );
    eprintln!("{}", files::quoted_command(&binary, &args));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(Command::new(binary)
            .args(args)
            .current_dir(checkout)
            .exec()
            .into())
    }
    #[cfg(not(unix))]
    {
        let status = Command::new(binary)
            .args(args)
            .current_dir(checkout)
            .status()?;
        Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
    }
}
fn print_report(report: &Value) {
    let managed = report
        .get("managed_preview")
        .filter(|value| value.is_object())
        .or_else(|| report.get("session").map(|_| report));
    if let Some(managed) = managed {
        println!(
            "Managed preview: {} ({}; owner running: {})",
            managed["url"].as_str().unwrap_or("unknown"),
            managed["state"].as_str().unwrap_or("unknown"),
            managed["owner_running"]
                .as_bool()
                .unwrap_or(managed["state"] == "ready")
        );
        if let Some(log) = managed["log"].as_str() {
            println!("Log: {log}");
        }
        if let Some(hash) = managed["selected_build"]["binary_sha256"].as_str() {
            println!("Running executable SHA-256: {hash}");
        }
    }
    for key in [
        "published",
        "directory",
        "reason",
        "baseline",
        "binary",
        "binary_selection",
        "index",
        "copy_method",
        "next_step",
    ] {
        if let Some(value) = report.get(key) {
            println!("{key}: {}", value.as_str().unwrap_or("unknown"));
        }
    }
    if report.get("readiness").is_some() {
        let ready = report["readiness"]["response"]["data"]["ready"] == true;
        let compatible = report["compatibility"]["exit_code"] == 0
            && report["compatibility"]["response"]["status"] == "ok";
        println!("Compatible: {compatible}; ready: {ready}");
        let key = if ready && compatible {
            "web_command"
        } else {
            "setup_command"
        };
        if let Some(command) = report[key].as_str() {
            println!("{key}: {command}");
        }
        if !ready || !compatible {
            println!("Use --json for check details. Setup is an explicit next step.");
        }
    }
}
