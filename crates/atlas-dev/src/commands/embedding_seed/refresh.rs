use std::{
    collections::BTreeMap,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use atlas_domain::SearchProfileId;
use atlas_runtime::{
    EmbeddingSeedPin, LocalEmbeddingSeed, RuntimeSetupOptions, RuntimeSetupReport,
    download_embedding_seed, parse_embedding_seed_pins,
};
use clap::Args;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{export_from, runtime};

const PROFILES: [&str; 2] = ["bge-base", "embeddinggemma"];
const PINS: &str = "crates/atlas-runtime/embedding-seeds.json";

#[derive(Debug, Args)]
pub(super) struct RefreshArgs {
    /// Reviewed Git ref that contains the previously published pins.
    #[arg(long)]
    baseline_ref: String,
    /// Candidate Atlas CLI built from the same clean checkout.
    #[arg(long)]
    atlas: PathBuf,
    /// Task-owned output directory. Verified downloads can survive a failed attempt.
    #[arg(long)]
    output: PathBuf,
    /// Full rebuild for a reviewed model contract change, as PROFILE=REASON.
    #[arg(long, value_name = "PROFILE=REASON")]
    rebuild_profile: Vec<String>,
    /// Override a proposed tag revision, as PROFILE=REVISION.
    #[arg(long)]
    next_revision: Vec<String>,
}

pub(super) fn run(args: RefreshArgs) -> Result<ExitCode, String> {
    let candidate = git(&["rev-parse", "HEAD"])?;
    let baseline = git(&[
        "rev-parse",
        "--verify",
        "--end-of-options",
        &format!("{}^{{commit}}", args.baseline_ref),
    ])?;
    if !git(&["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("refresh requires a clean tracked candidate checkout".into());
    }
    let pins = parse_embedding_seed_pins(&git(&["show", &format!("{baseline}:{PINS}")])?)?;
    if pins.len() != PROFILES.len() || PROFILES.iter().any(|profile| !pins.contains_key(*profile)) {
        return Err("the baseline ref must pin both supported profiles".into());
    }
    let revision_overrides = revision_overrides(&args.next_revision)?;
    let rebuild_reasons = rebuild_reasons(&args.rebuild_profile)?;
    let atlas = args.atlas.canonicalize().map_err(io_error)?;
    let atlas_digest = digest(&atlas)?;
    let dev_digest = digest(&std::env::current_exe().map_err(io_error)?)?;

    let output = &args.output;
    let result_path = output.join("result");
    if result_path.try_exists().map_err(io_error)? {
        return Err("refresh output already has a completed result; use a new directory".into());
    }
    let downloads = output.join("downloads");
    fs::create_dir_all(&downloads).map_err(io_error)?;
    let attempt = tempfile::tempdir_in(output).map_err(io_error)?;
    let result = attempt.path().join("result");
    fs::create_dir(&result).map_err(io_error)?;
    let rebuild = rebuild_reasons
        .keys()
        .copied()
        .map(SearchProfileId::new)
        .collect::<Vec<_>>();
    let mut files = Vec::new();
    for profile in PROFILES {
        let id = SearchProfileId::new(profile);
        if !rebuild.contains(&id) {
            files.push(
                download_embedding_seed(id, &pins[profile], &downloads)
                    .map_err(|error| format!("{profile}: {error}"))?,
            );
        }
    }
    let producer = attempt.path().join("producer.sqlite");
    let options = RuntimeSetupOptions {
        search_profiles: PROFILES.into_iter().map(SearchProfileId::new).collect(),
        ..Default::default()
    };
    let producer_runtime = runtime(producer.clone(), None)?;
    let built =
        build_report(producer_runtime.refresh_embedding_seeds(options.clone(), &files, &rebuild))?;
    let mut replacements = BTreeMap::new();
    let mut proposed_pins = pins.clone();
    for profile in &built.profiles {
        let id = profile.profile_id.as_str();
        if profile.inferred_inputs == 0 {
            if revision_overrides.contains_key(id) {
                return Err(format!(
                    "{id} is already covered and needs no revision override"
                ));
            }
            continue;
        }
        let version = next_revision(id, &pins[id].tag, &revision_overrides)?;
        let filename = format!("{id}-seed-v{version}.atlas-seed.gz");
        let path = result.join(&filename);
        let (file, header) = export_from(&producer_runtime, id, candidate.clone(), &path)?;
        files.retain(|entry| entry.profile != profile.profile_id);
        files.push(LocalEmbeddingSeed {
            profile: profile.profile_id.clone(),
            path,
            file: file.clone(),
        });
        let pin = EmbeddingSeedPin {
            tag: format!("embedding-seed/{id}/v{version}"),
            asset: filename,
            file,
        };
        replacements.insert(id.to_owned(), json!({ "pin": &pin, "header": header }));
        proposed_pins.insert(id.to_owned(), pin);
    }

    let proof = attempt.path().join("verification.sqlite");
    let verifier = runtime(proof.clone(), None)?;
    let (report, loaded) = verifier.verify_local_embedding_seeds(
        RuntimeSetupOptions {
            offline: true,
            ..options
        },
        &files,
    );
    let verified = build_report(report)?;
    if verified.inferred_inputs != 0 || loaded.len() != PROFILES.len() {
        return Err("strict verification did not consume both seeds with zero inference".into());
    }
    for profile in PROFILES {
        if query(&atlas, &producer, profile)? != query(&atlas, &proof, profile)? {
            return Err(format!("semantic query results differ for {profile}"));
        }
    }

    let producer_index = producer_runtime
        .open_index()
        .map_err(|error| error.to_string())?;
    let receipt = json!({
        "candidate": candidate,
        "baseline": baseline,
        "atlas_sha256": atlas_digest,
        "atlas_dev_sha256": dev_digest,
        "baseline_pins": pins,
        "source": producer_index.context(),
        "refresh": built.profiles,
        "explicit_rebuild_reasons": rebuild_reasons,
        "replacements": replacements,
        "verification": verified.profiles,
    });
    if !replacements.is_empty() {
        write_json(&result.join("proposed-pins.json"), &proposed_pins)?;
    }
    write_json(&result.join("receipt.json"), &receipt)?;
    fs::rename(&result, &result_path).map_err(io_error)?;
    println!(
        "{}",
        json!({"result": result_path, "replacements": replacements})
    );
    Ok(ExitCode::SUCCESS)
}

fn build_report(report: RuntimeSetupReport) -> Result<atlas_runtime::SetupBuildReport, String> {
    if !report.ready {
        return Err(report
            .actions
            .iter()
            .chain(report.checks.iter())
            .find_map(|action| {
                matches!(
                    action.status,
                    atlas_runtime::SetupActionStatus::Failed
                        | atlas_runtime::SetupActionStatus::Blocked
                )
                .then_some(action.reason.as_deref())
                .flatten()
            })
            .unwrap_or("embedding setup did not become ready")
            .to_owned());
    }
    let build = report
        .build
        .ok_or("embedding setup did not build a fresh workspace")?;
    if build.profiles.len() != PROFILES.len()
        || PROFILES.iter().any(|profile| {
            !build.profiles.iter().any(|entry| {
                entry.profile_id.as_str() == *profile
                    && entry.reused_inputs + entry.inferred_inputs > 0
            })
        })
    {
        return Err("embedding setup did not cover both profiles".into());
    }
    Ok(build)
}

fn query(atlas: &Path, index: &Path, profile: &str) -> Result<Value, String> {
    let output = Command::new(atlas)
        .args([
            "search",
            "fireball explosion",
            "--retrieval",
            "vector",
            "--search-profile",
            profile,
            "--limit",
            "3",
            "--json",
            "--index",
        ])
        .arg(index)
        .output()
        .map_err(io_error)?;
    let result: Value =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    let rows = &result["data"]["rows"];
    if !output.status.success()
        || result["status"] != "ok"
        || rows.as_array().is_none_or(Vec::is_empty)
    {
        return Err(format!("semantic search failed for {profile}: {result}"));
    }
    Ok(rows.clone())
}

fn rebuild_reasons(values: &[String]) -> Result<BTreeMap<&str, &str>, String> {
    let mut reasons = BTreeMap::new();
    for value in values {
        let (profile, reason) = value
            .split_once('=')
            .ok_or("--rebuild-profile requires PROFILE=REASON")?;
        let reason = reason.trim();
        if !PROFILES.contains(&profile)
            || reason.is_empty()
            || reasons.insert(profile, reason).is_some()
        {
            return Err(format!(
                "unknown profile, duplicate profile, or missing reason: {profile}"
            ));
        }
    }
    Ok(reasons)
}

fn revision_overrides(values: &[String]) -> Result<BTreeMap<String, u32>, String> {
    let mut revisions = BTreeMap::new();
    for value in values {
        let (profile, version) = value
            .split_once('=')
            .ok_or("--next-revision requires PROFILE=REVISION")?;
        let version = version
            .parse::<u32>()
            .ok()
            .filter(|version| *version > 0)
            .ok_or("revision must be a positive integer")?;
        if !PROFILES.contains(&profile) || revisions.insert(profile.to_owned(), version).is_some() {
            return Err(format!(
                "unknown or duplicate revision override for {profile}"
            ));
        }
    }
    Ok(revisions)
}

fn next_revision(
    profile: &str,
    tag: &str,
    overrides: &BTreeMap<String, u32>,
) -> Result<u32, String> {
    let current = tag
        .strip_prefix(&format!("embedding-seed/{profile}/v"))
        .and_then(|version| version.parse::<u32>().ok())
        .filter(|version| *version > 0)
        .ok_or_else(|| format!("invalid baseline revision for {profile}: {tag}"))?;
    let proposed = overrides
        .get(profile)
        .copied()
        .or_else(|| current.checked_add(1))
        .ok_or("embedding seed revision overflow")?;
    if proposed <= current {
        return Err(format!(
            "replacement revision for {profile} must exceed {current}"
        ));
    }
    Ok(proposed)
}

fn git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git").args(args).output().map_err(io_error)?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| error.to_string())
}

fn digest(path: &Path) -> Result<String, String> {
    let mut hash = Sha256::new();
    io::copy(&mut File::open(path).map_err(io_error)?, &mut hash).map_err(io_error)?;
    Ok(format!("{:x}", hash.finalize()))
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    let mut json = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    json.push(b'\n');
    fs::write(path, json).map_err(io_error)
}

fn io_error(error: io::Error) -> String {
    error.to_string()
}
