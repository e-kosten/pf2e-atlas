//! Definition reading and filter discovery through the shared app workflows.
use std::{path::PathBuf, process::ExitCode};

use atlas_app_model::{
    DefinitionDetailRequest, DefinitionFilterOptionsRequest, PreparedFieldBodyView,
    RecordDetailRequest, ReferenceDestinationView,
};
use atlas_app_service::{AppServiceRetrievalMode, ReadExecutionContext};
use atlas_cli_support::{CliPathMode, write_json_data};
use atlas_domain::DefinitionKey;
use clap::{Args, Subcommand};

use super::{
    product::report_app_error,
    record::{
        args::TerminalDetail,
        terminal::{TerminalLayout, render_html},
    },
};
use crate::client::{ClientOptions, connect};

#[derive(Debug, Args)]
pub(crate) struct DefinitionArgs {
    #[command(subcommand)]
    pub(crate) command: DefinitionCommand,
}
#[derive(Debug, Subcommand)]
pub(crate) enum DefinitionCommand {
    #[command(about = "Read an exact glossary definition and its sources")]
    Get(DefinitionGetOptions),
}
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas definition get language:chthonian\n  atlas definition get sense:darkvision --json\n\nUse an exact key from atlas filters definitions. Existing record definitions retain their original record identity."
)]
pub(crate) struct DefinitionGetOptions {
    key: DefinitionKey,
    #[arg(long, help = "Require the artifact generation returned by discovery")]
    artifact_generation: Option<String>,
    #[command(flatten)]
    paths: DefinitionPaths,
}
#[derive(Debug, Args)]
pub(crate) struct DefinitionPaths {
    #[arg(long, help = "Override the SQLite workspace path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
impl DefinitionGetOptions {
    pub(crate) fn uses_json(&self) -> bool {
        self.paths.json
    }
}
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas filters definitions --text darkvision\n  atlas filters definitions --text chthonian --json\n\nOptions include source-backed definitions and existing glossary records, including supported values with zero matches. Related filters retain their exact property or reference meaning."
)]
pub(crate) struct DefinitionFilterOptions {
    #[arg(long, help = "Find definitions by their supplied label or exact key")]
    text: Option<String>,
    #[arg(long, default_value_t = 0)]
    offset: usize,
    #[arg(long, default_value_t = 50)]
    limit: usize,
    #[arg(
        long,
        help = "Require the artifact generation returned by earlier discovery"
    )]
    artifact_generation: Option<String>,
    #[command(flatten)]
    paths: DefinitionPaths,
}
impl DefinitionFilterOptions {
    pub(crate) fn uses_json(&self) -> bool {
        self.paths.json
    }
}
fn client(
    paths: DefinitionPaths,
) -> Result<atlas_app_service::AtlasAppService, atlas_app_model::AppError> {
    connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(paths.path_mode),
        index: paths.index,
        embedding_cache: None,
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
    })
}
pub(crate) fn run_definition_get(options: DefinitionGetOptions) -> Result<ExitCode, String> {
    let json = options.paths.json;
    let service = match client(options.paths) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, json),
    };
    let detail = match service.definition_detail(
        DefinitionDetailRequest {
            definition_key: options.key.to_string(),
            expected_artifact_generation: options.artifact_generation,
        },
        &ReadExecutionContext::uncancellable(),
    ) {
        Ok(detail) => detail,
        Err(error) => return report_app_error(error.into_app_error(), json),
    };
    if json {
        write_json_data(&detail)?;
        return Ok(ExitCode::SUCCESS);
    }
    if let Some(ReferenceDestinationView::Record {
        navigation,
        reading_context,
    }) = &detail.help.destination
    {
        let record = match service.record_detail_at(RecordDetailRequest {
            record_key: navigation.record_key.clone(),
            owners: navigation.owners.clone(),
            fields: navigation.field.clone().into_iter().collect(),
            passage: navigation.passage.clone(),
            source_fingerprint: navigation.source_fingerprint.clone(),
            spell_rank: navigation.spell_rank,
            reading_context: *reading_context,
            expected_artifact_generation: Some(detail.artifact_generation.clone()),
        }) {
            Ok(record) => record,
            Err(error) => return report_app_error(error.into_app_error(), false),
        };
        match super::record::render_selected_record(&service, &record, TerminalDetail::Full) {
            Ok(text) => print!("{text}"),
            Err(error) => return report_app_error(error, false),
        }
    } else {
        println!("{}\n{}\n", detail.label, detail.definition_key);
        match detail.content.as_ref().map(|content| &content.body) {
            Some(PreparedFieldBodyView::Html { html, .. }) => {
                print!("{}", render_html(html, TerminalLayout::current(80))?)
            }
            Some(PreparedFieldBodyView::Plain { text }) => println!("{text}"),
            Some(PreparedFieldBodyView::Bounded { reason }) => println!("{reason}"),
            _ => println!(
                "{}",
                if detail.help.availability
                    == atlas_app_model::DefinitionAvailabilityView::SetupRequired
                {
                    "Run atlas setup to prepare glossary definitions."
                } else {
                    "No description available."
                }
            ),
        }
    }
    if let Some(attribution) = &detail.attribution {
        for publication in &attribution.publications {
            if let Some(title) = &publication.title {
                println!(
                    "Source: {title}{}",
                    publication
                        .page
                        .map(|page| format!(" p. {page}"))
                        .unwrap_or_default()
                );
            }
        }
    }
    for notice in &detail.notices {
        println!(
            "Notice: {}{}",
            notice.id,
            notice
                .source_url
                .as_deref()
                .map(|url| format!(" — {url}"))
                .unwrap_or_default()
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_definition_filters(options: DefinitionFilterOptions) -> Result<ExitCode, String> {
    let json = options.paths.json;
    let service = match client(options.paths) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, json),
    };
    let page = match service.definition_filter_options(
        DefinitionFilterOptionsRequest {
            text: options.text,
            offset: options.offset,
            limit: options.limit,
            selected: vec![],
            expected_artifact_generation: options.artifact_generation,
        },
        &ReadExecutionContext::uncancellable(),
    ) {
        Ok(page) => page,
        Err(error) => return report_app_error(error.into_app_error(), json),
    };
    if json {
        write_json_data(&page)?;
    } else {
        for option in &page.options {
            println!(
                "{}\t{}",
                option.label,
                serde_json::to_string(&option.target).map_err(|error| error.to_string())?
            );
        }
        println!(
            "\n{} definitions; generation {}",
            page.total, page.artifact_generation
        );
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::Parser;

    #[test]
    fn exact_definition_and_artifact_scoped_discovery_use_existing_command_grammar() {
        let cli =
            Cli::try_parse_from(["atlas", "definition", "get", "language:chthonian", "--json"])
                .unwrap();
        let Command::Definition(DefinitionArgs {
            command: DefinitionCommand::Get(options),
        }) = cli.command
        else {
            panic!("definition get expected")
        };
        assert_eq!(options.key.to_string(), "language:chthonian");
        assert!(options.uses_json());
        assert!(Cli::try_parse_from(["atlas", "definition", "get", "Chthonian"]).is_err());
        assert!(Cli::try_parse_from(["atlas", "definition", "get", "language:"]).is_err());
        let cli = Cli::try_parse_from([
            "atlas",
            "filters",
            "definitions",
            "--text",
            "Darkvision",
            "--json",
        ])
        .unwrap();
        let Command::Filters(crate::commands::filter_discovery::args::FiltersArgs {
            command: crate::commands::filter_discovery::args::FiltersCommand::Definitions(options),
        }) = cli.command
        else {
            panic!("definition discovery expected")
        };
        assert_eq!(options.text.as_deref(), Some("Darkvision"));
        assert!(options.uses_json());
        assert!(
            Cli::try_parse_from(["atlas", "filters", "definitions", "--where", "false"]).is_err()
        );
    }
}
