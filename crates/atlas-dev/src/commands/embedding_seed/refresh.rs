use std::{
    collections::BTreeMap,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use atlas_domain::SearchProfileId;
use atlas_runtime::{
    EmbeddingSeedDeltaPin, EmbeddingSeedPin, LocalEmbeddingSeed, RuntimeSetupOptions,
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
    if pins.profiles.len() != PROFILES.len()
        || PROFILES
            .iter()
            .any(|profile| !pins.profiles.contains_key(*profile))
    {
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
        if rebuild.contains(&id) {
            continue;
        }
        let pins = &pins.profiles[profile];
        files.push(
            download_embedding_seed(id.clone(), &pins.base, &downloads)
                .map_err(|error| format!("{profile}: {error}"))?,
        );
        if let Some(delta) = &pins.delta {
            let mut file = download_embedding_seed(id, &delta.asset, &downloads)
                .map_err(|error| format!("{profile} delta: {error}"))?;
            file.base_sha256 = Some(delta.base_sha256.clone());
            files.push(file);
        }
    }
    let producer = attempt.path().join("producer.sqlite");
    let options = RuntimeSetupOptions {
        search_profiles: PROFILES.into_iter().map(SearchProfileId::new).collect(),
        ..Default::default()
    };
    let producer_runtime = runtime(producer.clone(), None)?;
    let built = producer_runtime.refresh_embedding_seeds(options, &files, &rebuild)?;
    require_profiles(&built)?;
    let mut replacements = BTreeMap::new();
    let mut proposed_pins = pins.clone();
    let producer_index = producer_runtime
        .open_index()
        .map_err(|error| error.to_string())?;
    for profile in &built.profiles {
        let id = profile.profile_id.as_str();
        let profile_pins = proposed_pins
            .profiles
            .get_mut(id)
            .ok_or("unexpected refresh profile")?;
        let roll_base = rebuild.contains(&profile.profile_id);

        if profile.inferred_inputs == 0 {
            if revision_overrides.contains_key(id) {
                return Err(format!(
                    "{id} is already covered and needs no revision override"
                ));
            }
            if profile_pins.delta.is_some() {
                let base = local_base(&files, id)?.open(attempt.path())?;
                if base
                    .count_uncovered(&producer_index)
                    .map_err(|error| error.to_string())?
                    == 0
                {
                    profile_pins.delta = None;
                    files.retain(|entry| {
                        entry.profile != profile.profile_id || entry.base_sha256.is_none()
                    });
                }
            }
            continue;
        }

        let base_sha256 = (!roll_base).then(|| profile_pins.base.file.sha256.clone());
        let (tag_prefix, filename_prefix, prior_tag) = if roll_base {
            (
                format!("embedding-seed/{id}/v"),
                format!("{id}-seed-v"),
                Some(profile_pins.base.tag.as_str()),
            )
        } else {
            (
                format!("embedding-seed/{id}/delta/v"),
                format!("{id}-seed-delta-v"),
                profile_pins
                    .delta
                    .as_ref()
                    .map(|delta| delta.asset.tag.as_str()),
            )
        };
        let version = next_revision(id, &tag_prefix, prior_tag, &revision_overrides)?;
        let filename = format!("{filename_prefix}{version}.atlas-seed.gz");
        let path = result.join(&filename);
        let base = if roll_base {
            None
        } else {
            Some(local_base(&files, id)?.open(attempt.path())?)
        };
        let (file, header) = export_from(
            &producer_runtime,
            id,
            candidate.clone(),
            base.as_ref(),
            &path,
        )?;
        files.retain(|entry| {
            entry.profile != profile.profile_id || (!roll_base && entry.base_sha256.is_none())
        });
        files.push(LocalEmbeddingSeed {
            profile: profile.profile_id.clone(),
            path,
            file: file.clone(),
            base_sha256: base_sha256.clone(),
        });
        let pin = EmbeddingSeedPin {
            tag: format!("{tag_prefix}{version}"),
            asset: filename,
            file,
        };
        replacements.insert(
            id.to_owned(),
            json!({"pin": &pin, "base_sha256": &base_sha256, "header": header}),
        );
        if let Some(base_sha256) = base_sha256 {
            profile_pins.delta = Some(EmbeddingSeedDeltaPin {
                base_sha256,
                asset: pin,
            });
        } else {
            profile_pins.base = pin;
            profile_pins.delta = None;
        }
    }

    let proof = attempt.path().join("verification.sqlite");
    let verifier = runtime(proof.clone(), None)?;
    let (verified, _) = verifier.verify_local_embedding_seeds(&files)?;
    require_profiles(&verified)?;
    for profile in PROFILES {
        if query(&atlas, &producer, profile)? != query(&atlas, &proof, profile)? {
            return Err(format!("semantic query results differ for {profile}"));
        }
    }

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
    if proposed_pins != pins {
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

fn local_base<'a>(
    files: &'a [LocalEmbeddingSeed],
    profile: &str,
) -> Result<&'a LocalEmbeddingSeed, String> {
    files
        .iter()
        .find(|entry| entry.profile.as_str() == profile && entry.base_sha256.is_none())
        .ok_or_else(|| format!("missing local base for {profile}"))
}

fn require_profiles(build: &atlas_runtime::SetupBuildReport) -> Result<(), String> {
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
    Ok(())
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
    prefix: &str,
    prior_tag: Option<&str>,
    overrides: &BTreeMap<String, u32>,
) -> Result<u32, String> {
    let current = if let Some(tag) = prior_tag {
        tag.strip_prefix(prefix)
            .and_then(|version| version.parse::<u32>().ok())
            .filter(|version| *version > 0)
            .ok_or_else(|| format!("invalid baseline revision for {profile}: {tag}"))?
    } else {
        0
    };
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
