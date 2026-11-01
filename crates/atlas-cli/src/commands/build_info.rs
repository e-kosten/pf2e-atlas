use std::process::ExitCode;

use clap::Args;
use serde::Serialize;

#[derive(Debug, Args)]
pub(crate) struct BuildInfoArgs {
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Serialize)]
struct BuildInfo {
    version: &'static str,
    variant: &'static str,
}

pub(crate) fn run(args: BuildInfoArgs) -> Result<ExitCode, String> {
    let info = BuildInfo {
        version: env!("CARGO_PKG_VERSION"),
        variant: if cfg!(feature = "web") {
            "full"
        } else {
            "cli-only"
        },
    };
    if args.json {
        atlas_cli_support::write_json_data(info)?;
    } else {
        println!("Atlas {}\nBuild variant: {}", info.version, info.variant);
    }
    Ok(ExitCode::SUCCESS)
}
