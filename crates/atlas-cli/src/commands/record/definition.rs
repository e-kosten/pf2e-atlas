use std::{path::PathBuf, process::ExitCode};

use atlas_app_model::{DefinitionDetailRequest, PreparedFieldBodyView};
use atlas_app_service::{AppServiceRetrievalMode, ReadExecutionContext};
use atlas_cli_support::{CliPathMode, write_json_data};
use atlas_domain::DefinitionKey;
use clap::Args;

use super::{
    report_app_error,
    terminal::{TerminalLayout, render_html},
};
use crate::client::{ClientOptions, connect};

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record definition trait:agile\n  atlas record definition trait:deadly-d8 --json\n\nUse the exact identifier, including variant suffixes. Missing descriptions remain unavailable."
)]
pub(crate) struct DefinitionOptions {
    #[arg(help = "Exact definition key in namespace:identifier form")]
    key: DefinitionKey,
    #[arg(long, help = "Override the SQLite workspace path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

pub(crate) fn run(options: DefinitionOptions) -> Result<ExitCode, String> {
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
    let definition = match service.definition_detail(
        DefinitionDetailRequest {
            definition_key: options.key.to_string(),
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
        println!("{}\n{}\n", definition.label, definition.definition_key);
        if let Some(atlas_app_model::ReferenceHelpView {
            destination: atlas_app_model::ReferenceDestinationView::Record { navigation },
            ..
        }) = &definition.help
        {
            println!("Read with: atlas record get {}", navigation.record_key);
        } else {
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
        for publication in definition.publications {
            if let Some(title) = publication.title {
                match publication.page {
                    Some(page) => println!("Source: {title}, p. {page}"),
                    None => println!("Source: {title}"),
                }
            }
        }
        for notice in definition.notices {
            println!("{}", notice.text);
            if let Some(url) = notice.source_url {
                println!("{url}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
