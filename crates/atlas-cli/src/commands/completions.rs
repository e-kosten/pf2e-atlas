use std::process::ExitCode;

use clap::Command;
use clap_complete::generate;

use args::CompletionsArgs;

pub(crate) mod args;

pub(crate) fn run_completions(
    args: CompletionsArgs,
    mut command: Command,
) -> Result<ExitCode, String> {
    if let Some(shell) = clap_complete::env::Shells::builtins().completer(&args.shell.to_string()) {
        shell
            .write_registration(
                "ATLAS_COMPLETE",
                "atlas",
                "atlas",
                "atlas",
                &mut std::io::stdout(),
            )
            .map_err(|error| error.to_string())?;
    } else {
        generate(args.shell, &mut command, "atlas", &mut std::io::stdout());
    }
    Ok(ExitCode::SUCCESS)
}

fn options() -> Option<atlas_app_service::AtlasAppServiceOptions> {
    use atlas_app_service::{AppServiceRetrievalMode, AtlasAppServiceOptions};
    let mut options = AtlasAppServiceOptions {
        default_search_profile: None,
        path_mode: atlas_runtime::AtlasPathMode::Global,
        source_root: None,
        embedding_cache_root: None,
        index_path: None,
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
    };
    let args = std::env::args_os().collect::<Vec<_>>();
    let mut args = args.iter().skip(1);
    while let Some(arg) = args.next() {
        let arg = arg.to_str()?;
        if arg == "--index" {
            options.index_path = Some(args.next()?.into());
        } else if let Some(path) = arg.strip_prefix("--index=") {
            options.index_path = Some(path.into());
        } else {
            let mode = if arg == "--path-mode" {
                Some(args.next()?.to_str()?)
            } else {
                arg.strip_prefix("--path-mode=")
            };
            if let Some(mode) = mode {
                options.path_mode = match mode {
                    "global" => atlas_runtime::AtlasPathMode::Global,
                    "repo" => atlas_runtime::AtlasPathMode::Repo,
                    _ => return None,
                };
            }
        }
    }
    Some(options)
}

pub(crate) fn list_slugs(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    let Some((options, prefix)) = options().zip(current.to_str()) else {
        return Vec::new();
    };
    atlas_app_service::complete_saved_list_slugs(&options, prefix)
        .unwrap_or_default()
        .into_iter()
        .map(clap_complete::CompletionCandidate::new)
        .collect()
}

pub(crate) fn record_keys(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    let Some((options, prefix)) = options().zip(current.to_str()) else {
        return Vec::new();
    };
    atlas_app_service::complete_record_keys(&options, prefix)
        .unwrap_or_default()
        .into_iter()
        .map(clap_complete::CompletionCandidate::new)
        .collect()
}
