use super::followup::{EvidenceDetail, open_record, shell_command, workspace_args};
use super::product::report_app_error;
use crate::client::{ClientOptions, connect};
use atlas_app_model::{GraphContextViewRequest, GraphSectionView, RecordSummaryView};
use atlas_app_service::AtlasAppService;
use atlas_cli_support::{CliPathMode, write_json_data};
use serde_json::{Value, json};
use std::{path::PathBuf, process::ExitCode};
pub(crate) mod args;
pub(super) mod evidence;
pub(crate) mod occurrences;
use args::*;

fn client(
    mode: CliPathMode,
    index: Option<PathBuf>,
) -> Result<AtlasAppService, atlas_app_model::AppError> {
    connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(mode),
        index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    })
}

fn section(
    view: &GraphSectionView,
    enabled: bool,
    seed: &RecordSummaryView,
    generation: &str,
) -> Value {
    let edges: Vec<_> = view
        .occurrences
        .iter()
        .map(|edge| {
            let owner = std::iter::once(seed)
                .chain(&view.records)
                .find(|r| r.record_key == edge.source_navigation.record_key);
            evidence::occurrence(edge, owner, generation)
        })
        .collect();
    json!({"enabled":enabled, "returned_records":view.records.len(), "returned_occurrences":edges.len(),
        "truncated":view.truncated, "records":view.records.iter().map(evidence::identity).collect::<Vec<_>>(), "occurrences":edges})
}

pub(crate) fn run_graph_links(o: GraphLinksOptions) -> Result<ExitCode, String> {
    let workspace = workspace_args(o.path_mode, o.index.as_deref());
    let s = match client(o.path_mode, o.index) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let key = match super::record::resolve_seed(&s, o.record_ref, o.json)? {
        Ok(key) => key,
        Err(exit) => return Ok(exit),
    };
    let (generation, graph) = match s.graph_context(GraphContextViewRequest {
        record_key: key.to_string(),
        owners: None,
        source_fingerprint: None,
        outgoing_limit: o.outgoing,
        backlink_limit: o.backlinks,
    }) {
        Ok(Some(graph)) => graph,
        Ok(None) => {
            return report_app_error(
                atlas_app_model::AppError::new(
                    atlas_app_model::AppErrorCode::RecordNotFound,
                    "record not found",
                ),
                o.json,
            );
        }
        Err(error) => return report_app_error(error.into_app_error(), o.json),
    };
    if o.json {
        let mut data = if o.detail == EvidenceDetail::Full {
            let mut full = json!(graph);
            for direction in ["outgoing", "backlinks"] {
                crate::commands::record::navigation::bind_nested(
                    &mut full[direction]["occurrences"],
                    &generation,
                );
            }
            full
        } else {
            json!({"seed":evidence::identity(&graph.seed),
                "outgoing":section(&graph.outgoing,o.outgoing>0,&graph.seed,&generation),
                "backlinks":section(&graph.backlinks,o.backlinks>0,&graph.seed,&generation)})
        };
        data["artifact_generation"] = json!(generation);
        write_json_data(data)?;
    } else {
        println!("{}", graph.seed.title);
        for (label, section, limit) in [
            ("Outgoing references", &graph.outgoing, o.outgoing),
            ("Backlinks", &graph.backlinks, o.backlinks),
        ] {
            if limit == 0 {
                println!("{label}: disabled");
                continue;
            }
            println!(
                "{label}: {} records, {} occurrences{}",
                section.records.len(),
                section.occurrences.len(),
                if section.truncated {
                    " (truncated)"
                } else {
                    ""
                }
            );
            for edge in &section.occurrences {
                let owner = std::iter::once(&graph.seed)
                    .chain(&section.records)
                    .find(|r| r.record_key == edge.source_navigation.record_key);
                println!("  {}", evidence::line(edge, owner));
                if o.detail == EvidenceDetail::Full
                    && let Ok(workspace) = &workspace
                {
                    println!(
                        "    {}",
                        shell_command(&open_record(
                            &edge.source_navigation,
                            &generation,
                            workspace,
                            false
                        ))
                    );
                }
            }
        }
        if o.detail == EvidenceDetail::Full
            && let Err(reason) = workspace
        {
            println!("{reason}");
        }
    }
    Ok(ExitCode::SUCCESS)
}
