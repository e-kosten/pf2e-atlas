//! Source-backed candidates for creature family review.

use atlas_cli_support::write_json_data;
use atlas_domain::RecordKey;
use atlas_foundry_model::{ActorSourcePF2e, FoundryDocumentSource};
use clap::Args;
use serde::Serialize;
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, process::ExitCode};

#[derive(Debug, Args)]
pub(crate) struct CreatureAuditOptions {
    #[arg(long, help = "Path to the Foundry PF2e source checkout")]
    source: PathBuf,
    #[arg(long, help = "Emit exact actor records and shared-token candidates")]
    pub(crate) json: bool,
}

#[derive(Serialize)]
struct Actor<'a> {
    key: RecordKey,
    source_path: &'a str,
    name: &'a str,
    publication: Option<&'a str>,
    level: Option<&'a serde_json::Number>,
    token_name: Option<&'a str>,
    authored_adjustment: Option<&'a atlas_foundry_model::generated::NPCAttributesSourceAdjustment>,
}

pub(crate) fn run(options: CreatureAuditOptions) -> Result<ExitCode, String> {
    let source = atlas_ingest::load_foundry_documents(&options.source, None)
        .map_err(|error| error.to_string())?;
    let mut actors = Vec::new();
    for document in source.packs.iter().flat_map(|pack| &pack.documents) {
        let Some(FoundryDocumentSource::Actor(actor)) = document.admission.model.as_ref() else {
            continue;
        };
        let ActorSourcePF2e::NPCSource(npc) = actor.as_ref() else {
            continue;
        };
        let (Some(id), Some(name)) = (npc._id.as_value(), npc.name.as_value()) else {
            continue;
        };
        let key = RecordKey::parse(&format!("{}:{id}", document.provenance.pack_name))
            .map_err(|error| error.to_string())?;
        let system = npc.system.as_value();
        let details = system.and_then(|system| system.details.as_value());
        actors.push(Actor {
            key,
            source_path: &document.provenance.source_path,
            name,
            publication: details
                .and_then(|details| details.publication.as_value())
                .and_then(|publication| publication.title.as_value())
                .map(String::as_str),
            level: details
                .and_then(|details| details.level.as_value())
                .and_then(|level| level.value.as_value()),
            token_name: npc
                .prototype_token
                .as_value()
                .and_then(|token| token.name.as_value())
                .map(String::as_str),
            authored_adjustment: system
                .and_then(|system| system.attributes.as_value())
                .and_then(|attributes| attributes.adjustment.as_value()),
        });
    }
    actors.sort_by(|a, b| a.key.cmp(&b.key));
    let mut by_token = BTreeMap::<_, Vec<&Actor<'_>>>::new();
    for actor in &actors {
        if let (Some(publication), Some(token_name)) = (actor.publication, actor.token_name)
            && !token_name.is_empty()
        {
            by_token
                .entry((publication, token_name))
                .or_default()
                .push(actor);
        }
    }
    let candidates: Vec<_> = by_token.into_iter().filter_map(|((publication, token_name), members)| {
        let first = members.first()?;
        members.iter().any(|member| member.name != first.name)
            .then(|| json!({ "publication": publication, "token_name": token_name, "members": members }))
    }).collect();
    if options.json {
        write_json_data(json!({ "records": actors, "shared_token_candidates": candidates }))?;
    } else {
        println!(
            "Audited {} NPCs and found {} shared-token candidates.",
            actors.len(),
            candidates.len()
        );
    }
    Ok(ExitCode::SUCCESS)
}
