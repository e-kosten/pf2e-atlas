//! Contributor reports over the same variant lookup used by Explore.

mod proposals;

use atlas_cli_support::{CliPathMode, write_json_data};
use atlas_domain::{RecordKey, SearchProfileId};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::{
    AtlasRetrievalService, ListRecordsRequest, MAX_SEARCH_PAGE_SIZE, SearchPage,
    VariantGroupOutcome, VariantGroupRequest,
};
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::json;
use std::{path::PathBuf, process::ExitCode};

const EQUIPMENT_PACK: &str = "equipment-srd";

#[derive(Debug, Args)]
pub(crate) struct FamiliesArgs {
    #[command(subcommand)]
    pub(crate) command: FamiliesCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum FamiliesCommand {
    /// Audit equipment variants in a development workspace.
    Audit(AuditOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas-dev families audit --path-mode repo\n  atlas-dev families audit --index .atlas/workspace.sqlite --propose-grades --json"
)]
pub(crate) struct AuditOptions {
    #[arg(long, help = "Override the SQLite workspace path")]
    index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    path_mode: CliPathMode,
    #[arg(long, help = "Report candidate grade progressions for source review")]
    propose_grades: bool,
    #[arg(long, help = "Emit the full report in the standard JSON envelope")]
    pub(crate) json: bool,
}

pub(crate) fn run(options: AuditOptions) -> Result<ExitCode, String> {
    let profile = SearchProfileId::new("bge-base");
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(profile.clone()),
        path_mode: crate::cli::runtime_path_mode(options.path_mode),
        overrides: AtlasPathOverrides {
            index_path: options.index,
            ..AtlasPathOverrides::default()
        },
    })
    .map_err(|error| error.to_string())?;
    let index = runtime.open_index().map_err(|error| error.to_string())?;
    let retrieval = AtlasRetrievalService::from_index(index, profile, None)
        .map_err(|error| error.to_string())?;
    let keys = equipment_keys(&retrieval)?;

    if options.propose_grades {
        let proposals = proposals::proposals(&retrieval, &keys)?;
        if options.json {
            write_json_data(json!({"proposals": proposals}))?;
        } else {
            for proposal in &proposals {
                println!("{} ({} grades)", proposal.family, proposal.members.len());
            }
            println!(
                "{} grade progression candidates across {} equipment records.",
                proposals.len(),
                keys.len()
            );
        }
    } else {
        let records = audit(&retrieval, &keys)?;
        if options.json {
            write_json_data(json!({"records": records}))?;
        } else {
            let mut verified = 0;
            let mut possible = 0;
            let mut unavailable = 0;
            for record in &records {
                match &record.outcome {
                    AuditOutcome::Content { verified: true, .. } => verified += 1,
                    AuditOutcome::Content {
                        verified: false, ..
                    } => possible += 1,
                    AuditOutcome::Unavailable { .. } => unavailable += 1,
                    AuditOutcome::Empty | AuditOutcome::Missing => {}
                }
            }
            println!(
                "Audited {} equipment records. {verified} have verified variants, {possible} have possible variants, and {unavailable} could not be checked.",
                records.len()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn equipment_keys(retrieval: &AtlasRetrievalService) -> Result<Vec<RecordKey>, String> {
    let filter = atlas_index::parse_where(&format!("source.pack.id == '{EQUIPMENT_PACK}'"))
        .map_err(|error| error.to_string())?;
    let mut keys = Vec::new();
    let mut number = 1;
    loop {
        let page =
            SearchPage::new(number, MAX_SEARCH_PAGE_SIZE).map_err(|error| error.to_string())?;
        let result = retrieval
            .list_records(ListRecordsRequest::new(Some(filter.predicate()), page))
            .map_err(|error| error.to_string())?;
        keys.extend(result.record_keys);
        let Some(next) = result.page.next_page else {
            break;
        };
        number = next;
    }
    keys.sort();
    Ok(keys)
}

#[derive(Serialize)]
struct AuditRecord {
    key: RecordKey,
    outcome: AuditOutcome,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum AuditOutcome {
    Content {
        family: String,
        members: Vec<RecordKey>,
        verified: bool,
        qualifiers: Vec<String>,
        ambiguous: bool,
        truncated: bool,
    },
    Missing,
    Empty,
    Unavailable {
        reason: String,
    },
}

fn audit(
    retrieval: &AtlasRetrievalService,
    keys: &[RecordKey],
) -> Result<Vec<AuditRecord>, String> {
    keys.iter()
        .map(|key| {
            let outcome = match retrieval
                .variant_group(VariantGroupRequest { record_key: key })
                .map_err(|error| error.to_string())?
            {
                VariantGroupOutcome::Content(group) => AuditOutcome::Content {
                    family: group.base_name,
                    members: group
                        .variants
                        .into_iter()
                        .map(|member| member.key)
                        .collect(),
                    verified: group.verified,
                    qualifiers: group
                        .evidence
                        .into_iter()
                        .map(|evidence| evidence.qualifier)
                        .collect(),
                    ambiguous: group.ambiguous,
                    truncated: group.truncated,
                },
                VariantGroupOutcome::Missing => AuditOutcome::Missing,
                VariantGroupOutcome::Empty => AuditOutcome::Empty,
                VariantGroupOutcome::Unavailable { reason } => AuditOutcome::Unavailable { reason },
            };
            Ok(AuditRecord {
                key: key.clone(),
                outcome,
            })
        })
        .collect()
}
