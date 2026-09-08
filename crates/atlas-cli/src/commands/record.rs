use std::collections::BTreeMap;
use std::process::ExitCode;

use atlas_app_model::{AppError, AppErrorCode};
use atlas_domain::{DetailLevel, RecordKey};
use atlas_record::RecordJsonOptions;
use atlas_search::{GraphContextRequest, MAX_GRAPH_CONTEXT_LIMIT, RecordResolutionResult};
use serde::Serialize;

use crate::client::{
    AtlasClient, AtlasClientConfig, AtlasClientHandle, LocalAtlasClientOptions, connect,
};
use crate::output::{CliError, write_json_data, write_json_error, write_json_error_data};
use crate::terminal::TerminalStyle;

pub(crate) mod args;
pub(crate) mod context;
mod provenance;
mod render;

use args::{RecordGetOptions, RecordProvenanceOptions, RecordResolveOptions};
use context::{project_record, project_record_provenance_with_remaster};

use super::filters::build_filter;

const MAX_GET_KEYS: usize = 100;
const MAX_RESOLVE_QUERIES: usize = 25;

#[derive(Debug, Serialize)]
struct RecordGetData<T> {
    detail: String,
    #[serde(flatten)]
    body: T,
}

#[derive(Debug, Serialize)]
struct SingleRecordBody {
    record: atlas_record::RecordJson,
}

#[derive(Debug, Serialize)]
struct BatchRecordBody {
    results: Vec<RecordGetItem>,
    counts: BatchCounts,
    partial: bool,
}

#[derive(Debug, Serialize)]
struct RecordGetItem {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<atlas_record::RecordJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<CliError>,
}

#[derive(Debug, Serialize)]
struct RecordResolveData<T> {
    detail: String,
    #[serde(flatten)]
    body: T,
}

#[derive(Debug, Serialize)]
struct SingleResolveBody {
    result: RecordResolveItem,
}

#[derive(Debug, Serialize)]
struct BatchResolveBody {
    results: Vec<RecordResolveItem>,
    counts: BatchCounts,
    partial: bool,
}

#[derive(Debug, Serialize)]
struct RecordResolveItem {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<atlas_record::RecordJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<RecordResolutionJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alternatives: Option<Vec<RecordResolveAlternative>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<CliError>,
}

#[derive(Debug, Serialize)]
struct RecordResolveAlternative {
    record: atlas_record::RecordJson,
    resolution: RecordResolutionJson,
}

#[derive(Debug, Serialize)]
struct RecordResolutionJson {
    query: String,
    normalized_query: String,
    match_kind: &'static str,
    matched_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_source_ref: Option<String>,
}

#[derive(Debug, Serialize)]
struct BatchCounts {
    requested: usize,
    matched: usize,
    failed: usize,
}

pub(crate) fn run_record_get(options: RecordGetOptions) -> Result<ExitCode, String> {
    if options.keys.len() > MAX_GET_KEYS {
        return invalid_input(
            options.json,
            format!("record get accepts at most {MAX_GET_KEYS} keys"),
        );
    }
    let mut keys = Vec::new();
    for key in &options.keys {
        match RecordKey::parse(key) {
            Ok(parsed) => keys.push(parsed),
            Err(error) => {
                return invalid_record_key(options.json, key, error.to_string());
            }
        }
    }
    let client = match record_client(options.path_mode.into(), options.index, options.json)? {
        RecordCommandStep::Ready(client) => client,
        RecordCommandStep::Exit(code) => return Ok(code),
    };
    let records = match client.get_records(keys.clone()) {
        Ok(records) => records,
        Err(error) => return app_error(error, options.json),
    };
    let by_key = records
        .into_iter()
        .map(|record| (record.record.identity.key.to_string(), record))
        .collect::<BTreeMap<_, _>>();
    let record_options = RecordJsonOptions {
        detail: options.detail,
        include_source_json: options.include_raw,
    };

    if keys.len() == 1 {
        let key = keys[0].to_string();
        if let Some(record) = by_key.get(&key) {
            let data = RecordGetData {
                detail: options.detail.to_string(),
                body: SingleRecordBody {
                    record: project_record(&client, record, record_options)
                        .map_err(|error| error.message)?,
                },
            };
            if options.json {
                write_json_data(data)?;
            } else {
                print_single_record(&data.body.record, options.detail);
            }
            return Ok(ExitCode::SUCCESS);
        }
        if options.json {
            write_json_error("record_not_found", format!("record not found: {key}"))?;
        } else {
            eprintln!("record not found: {key}");
        }
        return Ok(ExitCode::from(1));
    }

    let mut failed = 0;
    let results = keys
        .iter()
        .map(|key| {
            let key_text = key.to_string();
            if let Some(record) = by_key.get(&key_text) {
                Ok(RecordGetItem {
                    key: key_text,
                    record: Some(
                        project_record(&client, record, record_options)
                            .map_err(|error| error.message)?,
                    ),
                    error: None,
                })
            } else {
                failed += 1;
                Ok(RecordGetItem {
                    key: key_text.clone(),
                    record: None,
                    error: Some(CliError {
                        code: "record_not_found",
                        message: format!("record not found: {key_text}"),
                    }),
                })
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    let data = RecordGetData {
        detail: options.detail.to_string(),
        body: BatchRecordBody {
            counts: BatchCounts {
                requested: keys.len(),
                matched: keys.len() - failed,
                failed,
            },
            partial: failed > 0,
            results,
        },
    };
    if options.json {
        write_json_data(&data)?;
    } else {
        print_record_get_batch(&data.body, options.detail);
    }
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

pub(crate) fn run_record_resolve(options: RecordResolveOptions) -> Result<ExitCode, String> {
    if options.queries.len() > MAX_RESOLVE_QUERIES {
        return invalid_input(
            options.json,
            format!("record resolve accepts at most {MAX_RESOLVE_QUERIES} queries"),
        );
    }
    let (filter, _) = match build_filter(options.filter_json.as_deref(), &options.filter_options) {
        Ok(filter) => filter,
        Err(error) if options.json => {
            write_json_error(error.code, error.message)?;
            return Ok(ExitCode::from(2));
        }
        Err(error) => return Err(error.message),
    };
    let client = match record_client(options.path_mode.into(), options.index, options.json)? {
        RecordCommandStep::Ready(client) => client,
        RecordCommandStep::Exit(code) => return Ok(code),
    };
    let record_options = RecordJsonOptions {
        detail: options.detail,
        include_source_json: options.include_raw,
    };

    let mut failed = 0;
    let mut results = Vec::new();
    for query in &options.queries {
        let matches = match client.resolve_record(query.clone(), filter.clone()) {
            Ok(matches) => matches,
            Err(error) => return app_error(error, options.json),
        };
        let item = resolve_item(
            &client,
            query,
            matches,
            record_options,
            options.alternatives,
        )?;
        if item.error.is_some() {
            failed += 1;
        }
        results.push(item);
    }

    if results.len() == 1 {
        let result = results.remove(0);
        if let Some(error) = result.error.as_ref() {
            if options.json && error.code == "record_resolution_ambiguous" {
                let data = RecordResolveData {
                    detail: options.detail.to_string(),
                    body: SingleResolveBody { result },
                };
                let message = data
                    .body
                    .result
                    .error
                    .as_ref()
                    .map(|error| error.message.clone())
                    .unwrap_or_else(|| "record resolution ambiguous".to_string());
                write_json_error_data("record_resolution_ambiguous", message, data)?;
                return Ok(ExitCode::from(1));
            }
            if result.alternatives.is_some() {
                let data = RecordResolveData {
                    detail: options.detail.to_string(),
                    body: SingleResolveBody { result },
                };
                print_single_resolve(&data.body.result, options.detail);
                return Ok(ExitCode::from(1));
            }
            if options.json {
                write_json_error(error.code, error.message.clone())?;
            } else {
                eprintln!("{}", error.message);
            }
            return Ok(ExitCode::from(1));
        }
        let data = RecordResolveData {
            detail: options.detail.to_string(),
            body: SingleResolveBody { result },
        };
        if options.json {
            write_json_data(&data)?;
        } else {
            print_single_resolve(&data.body.result, options.detail);
        }
        return Ok(ExitCode::SUCCESS);
    }

    let data = RecordResolveData {
        detail: options.detail.to_string(),
        body: BatchResolveBody {
            counts: BatchCounts {
                requested: results.len(),
                matched: results.len() - failed,
                failed,
            },
            partial: failed > 0,
            results,
        },
    };
    if options.json {
        write_json_data(&data)?;
    } else {
        print_resolve_batch(&data.body, options.detail);
    }
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

pub(crate) fn run_record_provenance(options: RecordProvenanceOptions) -> Result<ExitCode, String> {
    let key = match RecordKey::parse(&options.key) {
        Ok(key) => key,
        Err(error) => {
            return invalid_record_key(options.json, &options.key, error.to_string());
        }
    };
    let client = match record_client(options.path_mode.into(), options.index, options.json)? {
        RecordCommandStep::Ready(client) => client,
        RecordCommandStep::Exit(code) => return Ok(code),
    };
    let remaster = match client.remaster_links(key.clone()) {
        Ok(Some(result)) => result,
        Ok(None) => {
            if options.json {
                write_json_error("record_not_found", format!("record not found: {key}"))?;
            } else {
                eprintln!("record not found: {key}");
            }
            return Ok(ExitCode::from(1));
        }
        Err(error) => return app_error(error, options.json),
    };
    let record = match project_record_provenance_with_remaster(
        &remaster.seed,
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: false,
        },
        &remaster,
    ) {
        Ok(record) => record,
        Err(error) => return provenance_failure(options.json, error.message),
    };
    let graph = match client.graph_context(
        GraphContextRequest::new(key)
            .with_outgoing_limit(MAX_GRAPH_CONTEXT_LIMIT)
            .with_backlink_limit(MAX_GRAPH_CONTEXT_LIMIT),
    ) {
        Ok(Some(graph)) => graph,
        Ok(None) => {
            return provenance_failure(
                options.json,
                "record disappeared during provenance lookup".to_string(),
            );
        }
        Err(error) => return app_error(error, options.json),
    };
    let data = provenance::provenance_data(&record, Some(&graph));
    if options.json {
        write_json_data(data)?;
    } else {
        print!("{}", provenance::render_provenance(&data));
    }
    Ok(ExitCode::SUCCESS)
}

fn resolve_item(
    client: &impl AtlasClient,
    query: &str,
    matches: Vec<RecordResolutionResult>,
    record_options: RecordJsonOptions,
    alternatives: u8,
) -> Result<RecordResolveItem, String> {
    if matches.is_empty() {
        return Ok(RecordResolveItem {
            query: query.to_string(),
            record: None,
            resolution: None,
            alternatives: None,
            error: Some(CliError {
                code: "record_resolution_miss",
                message: format!("record resolution miss: {query}"),
            }),
        });
    }
    if matches.len() > 1 {
        let projected_alternatives = matches
            .iter()
            .take(alternatives as usize)
            .map(|resolution| {
                Ok(RecordResolveAlternative {
                    record: project_record(client, &resolution.record, record_options)
                        .map_err(|error| error.message)?,
                    resolution: resolution_json(resolution, record_options),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(RecordResolveItem {
            query: query.to_string(),
            record: None,
            resolution: None,
            alternatives: Some(projected_alternatives)
                .filter(|alternatives| !alternatives.is_empty()),
            error: Some(CliError {
                code: "record_resolution_ambiguous",
                message: format!("record resolution ambiguous: {query}"),
            }),
        });
    }
    let Some(resolution) = matches.into_iter().next() else {
        return Ok(RecordResolveItem {
            query: query.to_string(),
            record: None,
            resolution: None,
            alternatives: None,
            error: Some(CliError {
                code: "record_resolution_miss",
                message: format!("record resolution miss: {query}"),
            }),
        });
    };
    Ok(RecordResolveItem {
        query: query.to_string(),
        record: Some(
            project_record(client, &resolution.record, record_options)
                .map_err(|error| error.message)?,
        ),
        resolution: Some(resolution_json(&resolution, record_options)),
        alternatives: None,
        error: None,
    })
}

fn resolution_json(
    resolution: &RecordResolutionResult,
    record_options: RecordJsonOptions,
) -> RecordResolutionJson {
    let full = record_options.detail == atlas_domain::DetailLevel::Full;
    RecordResolutionJson {
        query: resolution.query.clone(),
        normalized_query: resolution.normalized_query.clone(),
        match_kind: resolution.match_kind.as_str(),
        matched_text: resolution.matched_text.clone(),
        alias_source: full.then(|| resolution.alias_source.clone()).flatten(),
        alias_source_ref: full.then(|| resolution.alias_source_ref.clone()).flatten(),
    }
}

fn print_single_record(record: &atlas_record::RecordJson, detail: DetailLevel) {
    if detail_outputs_description(detail) {
        print_record_for_detail(record, detail);
        return;
    }
    println!("{}\t{}\t{}", record.key, record.name, record.kind);
}

fn print_record_get_batch(batch: &BatchRecordBody, detail: DetailLevel) {
    println!(
        "matched {}/{} records",
        batch.counts.matched, batch.counts.requested
    );
    let mut printed_record = false;
    let style = TerminalStyle::stdout();
    for result in &batch.results {
        if let Some(record) = &result.record {
            if detail_outputs_description(detail) && printed_record {
                println!();
                println!("{}", style.separator());
                println!();
            }
            print_single_record(record, detail);
            printed_record = true;
        } else if let Some(error) = &result.error {
            eprintln!("{}\t{}", result.key, error.message);
        }
    }
}

fn print_single_resolve(result: &RecordResolveItem, detail: DetailLevel) {
    if let Some(record) = &result.record {
        let match_kind = result
            .resolution
            .as_ref()
            .map(|resolution| resolution.match_kind)
            .unwrap_or("unknown");
        if detail_outputs_description(detail) {
            print_record_for_detail(record, detail);
            let style = TerminalStyle::stdout();
            println!("{}: {match_kind}", style.label("Match"));
            return;
        }
        println!("{}\t{}\t{}", record.key, record.name, match_kind);
        return;
    }
    if let Some(error) = &result.error {
        eprintln!("{}", error.message);
    }
    if let Some(alternatives) = &result.alternatives {
        for alternative in alternatives {
            if detail_outputs_description(detail) {
                print_record_for_detail(&alternative.record, detail);
                let style = TerminalStyle::stdout();
                println!(
                    "{}: {}",
                    style.label("Match"),
                    alternative.resolution.match_kind
                );
            } else {
                println!(
                    "{}\t{}\t{}",
                    alternative.record.key,
                    alternative.record.name,
                    alternative.resolution.match_kind
                );
            }
        }
    }
}

fn print_resolve_batch(batch: &BatchResolveBody, detail: DetailLevel) {
    println!(
        "matched {}/{} queries",
        batch.counts.matched, batch.counts.requested
    );
    let mut printed = false;
    let style = TerminalStyle::stdout();
    for result in &batch.results {
        if detail_outputs_description(detail) && printed {
            println!();
            println!("{}", style.separator());
            println!();
        }
        print_single_resolve(result, detail);
        printed = true;
    }
}

pub(crate) fn print_record_for_detail(record: &atlas_record::RecordJson, detail: DetailLevel) {
    print!(
        "{}",
        render::render_record(
            record,
            detail,
            render::effective_width(),
            TerminalStyle::stdout()
        )
    );
}

pub(crate) fn detail_outputs_description(detail: DetailLevel) -> bool {
    detail != DetailLevel::Summary
}

fn invalid_record_key(json: bool, key: &str, message: String) -> Result<ExitCode, String> {
    if json {
        write_json_error(
            "invalid_record_key",
            format!("invalid record key `{key}`: {message}"),
        )?;
        Ok(ExitCode::from(2))
    } else {
        Err(format!("invalid record key `{key}`: {message}"))
    }
}

enum RecordCommandStep<T> {
    Ready(T),
    Exit(ExitCode),
}

fn record_client(
    path_mode: atlas_runtime::AtlasPathMode,
    index: Option<std::path::PathBuf>,
    json: bool,
) -> Result<RecordCommandStep<AtlasClientHandle>, String> {
    match connect(AtlasClientConfig::Local(LocalAtlasClientOptions {
        path_mode,
        index_path: index,
        embedding_cache_root: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })) {
        Ok(client) => Ok(RecordCommandStep::Ready(client)),
        Err(error) if json => {
            write_app_json_error(error)?;
            Ok(RecordCommandStep::Exit(ExitCode::from(3)))
        }
        Err(error) => Err(error.message),
    }
}

fn app_error(error: AppError, json: bool) -> Result<ExitCode, String> {
    let (code, exit) = app_error_code(error.code);
    if json {
        write_app_json_error(error)?;
        return Ok(exit);
    }
    if exit == ExitCode::from(1) {
        eprintln!("{}", error.message);
        Ok(exit)
    } else {
        Err(format!("{code}: {}", error.message))
    }
}

fn write_app_json_error(error: AppError) -> Result<(), String> {
    let (code, _) = app_error_code(error.code);
    if let Some(details) = error.details {
        write_json_error_data(code, error.message, details)
    } else {
        write_json_error(code, error.message)
    }
}

fn app_error_code(code: AppErrorCode) -> (&'static str, ExitCode) {
    match code {
        AppErrorCode::InvalidRecordKey => ("invalid_record_key", ExitCode::from(2)),
        AppErrorCode::RecordNotFound => ("record_not_found", ExitCode::from(1)),
        AppErrorCode::RecordResolutionMiss => ("record_resolution_miss", ExitCode::from(1)),
        AppErrorCode::RecordResolutionAmbiguous => {
            ("record_resolution_ambiguous", ExitCode::from(1))
        }
        AppErrorCode::IndexUnavailable => ("index_unavailable", ExitCode::from(3)),
        AppErrorCode::QueryFailed | AppErrorCode::EmbeddingModelUnavailable => {
            ("query_failed", ExitCode::from(3))
        }
        AppErrorCode::ArtifactIncompatible => ("artifact_contract_violation", ExitCode::from(3)),
        AppErrorCode::InvalidRequest => ("invalid_input", ExitCode::from(2)),
        AppErrorCode::FilterInvalid => ("invalid_filter", ExitCode::from(2)),
        AppErrorCode::VectorReadinessRequired => ("vector_readiness_required", ExitCode::from(3)),
        AppErrorCode::ArtifactNotReady
        | AppErrorCode::SetupRequired
        | AppErrorCode::SetupInProgress => ("runtime_error", ExitCode::from(3)),
        AppErrorCode::SavedListNotFound
        | AppErrorCode::SavedListAlreadyExists
        | AppErrorCode::EncounterNotFound
        | AppErrorCode::EncounterAlreadyExists
        | AppErrorCode::EncounterParticipantNotFound
        | AppErrorCode::WindowNotFound
        | AppErrorCode::WindowExpired
        | AppErrorCode::FilterFieldInvalid
        | AppErrorCode::FilterOptionInvalid
        | AppErrorCode::FilterFieldNotApplicable
        | AppErrorCode::FilterMetricAmbiguous
        | AppErrorCode::FilterEditorConflict
        | AppErrorCode::ServiceBusy
        | AppErrorCode::OperationCancelled
        | AppErrorCode::OperationTimeout
        | AppErrorCode::InternalError => ("query_failed", ExitCode::from(3)),
    }
}

fn invalid_input(json: bool, message: String) -> Result<ExitCode, String> {
    if json {
        write_json_error("invalid_input", message)?;
        Ok(ExitCode::from(2))
    } else {
        Err(message)
    }
}

fn provenance_failure(json: bool, message: String) -> Result<ExitCode, String> {
    if json {
        write_json_error("query_failed", message)?;
        Ok(ExitCode::from(3))
    } else {
        Err(message)
    }
}
