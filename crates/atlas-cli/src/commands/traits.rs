use std::{path::PathBuf, process::ExitCode};

use atlas_app_model::{PreparedFieldBodyView, TraitDetailRequest};
use atlas_app_service::{AppServiceRetrievalMode, ReadExecutionContext};
use atlas_cli_support::{CliPathMode, write_json_data};
use atlas_domain::TraitKey;
use clap::{Args, Subcommand};

use super::{
    product::report_app_error,
    record::terminal::{TerminalLayout, render_html},
};
use crate::client::{ClientOptions, connect};

#[derive(Debug, Args)]
pub(crate) struct TraitArgs {
    #[command(subcommand)]
    pub(crate) command: TraitCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum TraitCommand {
    #[command(about = "Read a trait definition")]
    Get(TraitGetOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas trait get trait:agile\n  atlas trait get trait:deadly-d8 --json\n\nUse the exact identifier, including variant suffixes. Missing descriptions remain unavailable."
)]
pub(crate) struct TraitGetOptions {
    #[arg(help = "Exact trait key in trait:<identifier> form")]
    key: TraitKey,
    #[arg(long, help = "Override the SQLite workspace path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

pub(crate) fn run_trait_get(options: TraitGetOptions) -> Result<ExitCode, String> {
    let service = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(options.path_mode),
        index: options.index,
        embedding_cache: None,
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, options.json),
    };
    let definition = match service.trait_definition(
        TraitDetailRequest {
            trait_key: options.key.to_string(),
            expected_artifact_generation: None,
        },
        &ReadExecutionContext::uncancellable(),
    ) {
        Ok(definition) => definition,
        Err(error) => return report_app_error(error.into_app_error(), options.json),
    };
    if options.json {
        write_json_data(&definition)?;
    } else {
        println!("{}\n{}\n", definition.label, definition.trait_key);
        match definition.content.map(|content| content.body) {
            Some(PreparedFieldBodyView::Html { html, .. }) => {
                print!("{}", render_html(&html, TerminalLayout::current(80))?);
            }
            Some(PreparedFieldBodyView::Plain { text }) => println!("{text}"),
            Some(PreparedFieldBodyView::Bounded { reason }) => println!("{reason}"),
            Some(PreparedFieldBodyView::Unavailable { .. }) | None => {
                println!("No description available.");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::{CommandFactory, Parser};

    #[test]
    fn trait_get_accepts_exact_canonical_keys_and_json() {
        let parsed =
            Cli::try_parse_from(["atlas", "trait", "get", "trait:deadly-d8", "--json"]).unwrap();
        let Command::Trait(TraitArgs {
            command: TraitCommand::Get(options),
        }) = parsed.command
        else {
            panic!("trait get expected")
        };
        assert_eq!(options.key.identifier(), "deadly-d8");
        assert!(options.json);
        assert!(Cli::try_parse_from(["atlas", "trait", "get", "agile"]).is_err());
        assert!(Cli::try_parse_from(["atlas", "trait", "get", "trait:"]).is_err());
    }

    #[test]
    fn trait_help_advertises_definition_reading() {
        let mut cli = Cli::command();
        let traits = cli.find_subcommand_mut("trait").unwrap();
        assert!(
            traits
                .render_help()
                .to_string()
                .contains("Read a trait definition")
        );
    }
}
