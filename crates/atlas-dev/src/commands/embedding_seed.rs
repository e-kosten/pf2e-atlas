use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use atlas_domain::SearchProfileId;
use atlas_index::{EmbeddingSeedFile, export_embedding_seed};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, RuntimeSetupOptions};
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Debug, Args)]
pub(crate) struct SeedArgs {
    #[command(subcommand)]
    command: SeedCommand,
}

#[derive(Debug, Subcommand)]
enum SeedCommand {
    /// Export a profile from a ready, dedicated managed workspace.
    Export {
        #[arg(long)]
        index: PathBuf,
        #[arg(long)]
        profile: String,
        #[arg(long)]
        producer_commit: String,
        #[arg(long)]
        output: PathBuf,
    },
    /// Verify local seed files through managed setup before publishing.
    Verify {
        #[arg(long)]
        index: PathBuf,
        #[arg(long)]
        embedding_cache_path: Option<PathBuf>,
        /// Repeat for each profile. Format: PROFILE,SHA256,PATH.
        #[arg(long, required = true)]
        seed: Vec<String>,
        /// Exercise adding a profile to a disposable workspace instead of certifying full coverage.
        #[arg(long)]
        add_to_existing: bool,
    },
}

fn runtime(index: PathBuf, model_cache: Option<PathBuf>) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            index_path: Some(index),
            embedding_cache_root: model_cache,
            ..Default::default()
        },
        ..Default::default()
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn run(args: SeedArgs) -> Result<ExitCode, String> {
    match args.command {
        SeedCommand::Export {
            index,
            profile,
            producer_commit,
            output,
        } => export(index, &profile, producer_commit, &output),
        SeedCommand::Verify {
            index,
            embedding_cache_path,
            seed,
            add_to_existing,
        } => verify(index, embedding_cache_path, &seed, add_to_existing),
    }
}

fn export(
    index: PathBuf,
    profile: &str,
    producer_commit: String,
    output: &Path,
) -> Result<ExitCode, String> {
    let result = |error: atlas_index::IndexError| error.to_string();
    let runtime = runtime(index, None)?;
    let workspace = runtime.open_index().map_err(result)?;
    let requested = atlas_index::read_requested_reference_config(workspace.workspace_session())
        .map_err(result)?
        .ok_or("the producer workspace has no requested configuration")?;
    if requested.manifest_path.is_some()
        || requested.source_root != runtime.paths().managed_source_root
    {
        return Err(
            "export requires a dedicated workspace prepared from the managed source".into(),
        );
    }
    let model = workspace
        .context()
        .search_profile(&SearchProfileId::new(profile))
        .map_err(result)?;
    let directory = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged =
        tempfile::NamedTempFile::new_in(directory).map_err(|error| error.to_string())?;
    let header = export_embedding_seed(
        &workspace,
        model,
        producer_commit,
        BufWriter::new(staged.as_file_mut()),
    )
    .map_err(result)?;
    let file = EmbeddingSeedFile::measure(staged.as_file()).map_err(result)?;
    staged
        .persist_noclobber(output)
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        json!({ "file": file, "header": header, "path": output })
    );
    Ok(ExitCode::SUCCESS)
}

fn verify(
    index: PathBuf,
    model_cache: Option<PathBuf>,
    seeds: &[String],
    add_to_existing: bool,
) -> Result<ExitCode, String> {
    if index.try_exists().map_err(|error| error.to_string())? != add_to_existing {
        return Err(if add_to_existing {
            "--add-to-existing requires a disposable existing workspace".into()
        } else {
            "complete coverage verification requires a new workspace path".into()
        });
    }
    let mut files = Vec::new();
    for seed in seeds {
        let mut parts = seed.splitn(3, ',');
        let (Some(profile), Some(sha256), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err("--seed requires PROFILE,SHA256,PATH".into());
        };
        let profile = SearchProfileId::new(profile);
        if files
            .iter()
            .any(|entry: &atlas_runtime::LocalEmbeddingSeed| entry.profile == profile)
        {
            return Err(format!("duplicate seed for {profile}"));
        }
        let path = PathBuf::from(path);
        let bytes = File::open(&path)
            .and_then(|file| file.metadata())
            .map_err(|error| error.to_string())?
            .len();
        files.push(atlas_runtime::LocalEmbeddingSeed {
            profile,
            path,
            file: EmbeddingSeedFile {
                sha256: sha256.into(),
                bytes,
            },
        });
    }
    let runtime = runtime(index, model_cache)?;
    let started = Instant::now();
    let (report, loaded) = runtime.verify_local_embedding_seeds(
        RuntimeSetupOptions {
            search_profiles: files.iter().map(|entry| entry.profile.clone()).collect(),
            offline: true,
            ..Default::default()
        },
        &files,
    );
    if !report.ready {
        let reason = report
            .actions
            .iter()
            .rev()
            .find_map(|action| action.reason.as_deref())
            .unwrap_or("setup failed without a diagnostic");
        return Err(reason.to_owned());
    }
    let build = report
        .build
        .ok_or("verification did not run a source build")?;
    if build.inferred_inputs != 0 || loaded.len() != files.len() {
        return Err(
            "verification did not consume every supplied seed with zero corpus inference".into(),
        );
    }
    println!(
        "{}",
        json!({
            "complete_coverage": !add_to_existing,
        "build_ms": build.build_duration_ms,
            "setup_ms": started.elapsed().as_millis(),
            "inferred_inputs": build.inferred_inputs,
            "reused_inputs": build.reused_inputs,
            "profiles": build.profiles,
            "loaded_seeds": loaded,
        })
    );
    Ok(ExitCode::SUCCESS)
}
