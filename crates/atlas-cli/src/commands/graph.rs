use super::product::app_result;
use crate::client::{ClientOptions, connect};
use atlas_app_model::{GraphContextViewRequest, RecordRefResolutionView};
use atlas_app_service::AtlasAppService;
use atlas_cli_support::{CliPathMode, write_json_data, write_json_error_data};
use std::{path::PathBuf, process::ExitCode};
pub(crate) mod args;
use args::*;
fn client(mode: CliPathMode, index: Option<PathBuf>) -> Result<AtlasAppService, String> {
    connect(ClientOptions {
        path_mode: mode.into(),
        index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .map_err(|e| e.message)
}
fn seed(
    s: &AtlasAppService,
    name: String,
    json: bool,
) -> Result<Option<atlas_domain::RecordKey>, String> {
    let Some(result) = app_result(s.resolve_record_ref(name, None), json)? else {
        return Ok(None);
    };
    let (code, message, candidates) = match result {
        RecordRefResolutionView::Key(k) => return Ok(Some(k)),
        RecordRefResolutionView::Miss => (
            "record_resolution_miss",
            "no strict name or verified alias matched",
            vec![],
        ),
        RecordRefResolutionView::Ambiguous(matches) => (
            "record_resolution_ambiguous",
            "ambiguous reference; specify a Foundry key",
            matches.iter().map(|m| m.record.clone()).collect(),
        ),
    };
    if json {
        write_json_error_data(
            code,
            message.to_owned(),
            serde_json::json!({"candidates":candidates}),
        )?;
        Ok(None)
    } else {
        Err(format!(
            "{message}: {}",
            candidates
                .iter()
                .map(|c| c.record_key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}
fn show_graph(
    s: &AtlasAppService,
    request: GraphContextViewRequest,
    json: bool,
) -> Result<ExitCode, String> {
    let Some(result) = app_result(s.graph_context(request), json)? else {
        return Ok(ExitCode::from(3));
    };
    let Some(result) = result else {
        return Err("record not found".into());
    };
    let data = serde_json::to_value(result).map_err(|e| e.to_string())?;
    if json {
        write_json_data(data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_graph_links(o: GraphLinksOptions) -> Result<ExitCode, String> {
    let s = client(o.path_mode, o.index)?;
    let Some(key) = seed(&s, o.record_ref, o.json)? else {
        return Ok(ExitCode::from(3));
    };
    show_graph(
        &s,
        GraphContextViewRequest {
            record_key: key.to_string(),
            owners: None,
            source_fingerprint: None,
            outgoing_limit: o.outgoing,
            backlink_limit: o.backlinks,
        },
        o.json,
    )
}
pub(crate) fn run_graph_uses(o: GraphUsesOptions) -> Result<ExitCode, String> {
    let s = client(o.path_mode, o.index)?;
    let Some(key) = seed(&s, o.record_ref, o.json)? else {
        return Ok(ExitCode::from(3));
    };
    show_graph(
        &s,
        GraphContextViewRequest {
            record_key: key.to_string(),
            owners: None,
            source_fingerprint: None,
            outgoing_limit: 0,
            backlink_limit: o.limit,
        },
        o.json,
    )
}
pub(crate) fn run_graph_remaster(o: GraphRemasterOptions) -> Result<ExitCode, String> {
    let s = client(o.path_mode, o.index)?;
    let Some(key) = seed(&s, o.record_ref, o.json)? else {
        return Ok(ExitCode::from(3));
    };
    let Some(r) = app_result(s.remaster_links(key), o.json)? else {
        return Ok(ExitCode::from(3));
    };
    let Some(r) = r else {
        return Err("record not found".into());
    };
    let data = serde_json::to_value(r).map_err(|e| e.to_string())?;
    if o.json {
        write_json_data(data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_graph_variants(o: GraphVariantsOptions) -> Result<ExitCode, String> {
    let s = client(o.path_mode, o.index)?;
    let Some(key) = seed(&s, o.record_ref, o.json)? else {
        return Ok(ExitCode::from(3));
    };
    let Some(r) = app_result(s.variant_group(key), o.json)? else {
        return Ok(ExitCode::from(3));
    };
    let Some(r) = r else {
        if o.json {
            write_json_data(serde_json::json!({"label":"Suggested variants","group":null}))?;
        } else {
            println!("No suggested variants");
        }
        return Ok(ExitCode::SUCCESS);
    };
    let data = serde_json::json!({"label":"Suggested variants","group":r});
    if o.json {
        write_json_data(data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
