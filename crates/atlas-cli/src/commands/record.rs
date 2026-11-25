use super::{filters::resolve_filter, product::report_app_error};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    EmbeddedReadingEntryView, EmbeddedReadingParent, EmbeddedReadingRequest,
    EmbeddedReadingSelection, RecordBodyView, RecordDetailRequest, RecordDetailView,
};
use atlas_cli_support::write_json_response;
use std::process::ExitCode;
pub(crate) mod args;
mod batch;
pub(crate) mod definition;
mod json;
mod links;
pub(crate) mod navigation;
mod sections;
pub(crate) use sections::render_encounter_actor;
mod presentation;
pub(crate) mod source;
pub(crate) mod table;
pub(crate) mod terminal;
pub(crate) mod variants;
use args::{RecordGetOptions, RecordResolveOptions, TerminalDetail};
#[cfg(test)]
use terminal::render_html;
pub(crate) fn run_record_get(o: RecordGetOptions) -> Result<ExitCode, String> {
    let connection = connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index.clone(),
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    });
    let s = match connection {
        Ok(service) => service,
        Err(error) => return super::product::report_app_error(error, o.json),
    };
    // A zero-key summary read admits the workspace without decoding source bodies.
    // Admission failures belong to the command, before any per-input work starts.
    if let Err(error) = s.get_records(Vec::new()) {
        return report_app_error(error.into_app_error(), o.json);
    }
    let keys = match &o.navigation {
        Some(locator) => vec![locator.address.record_key.clone()],
        None => o.keys,
    };
    let batch = batch::Batch::run(keys, |key| {
        let request = match &o.navigation {
            Some(locator) => locator.request(),
            None => RecordDetailRequest {
                expected_artifact_generation: None,
                spell_rank: o.rank,
                record_key: key.into(),
                owners: o.owners.as_ref().map(|v| v.0.clone()).unwrap_or_default(),
                fields: o.field.clone().into_iter().collect(),
                passage: o.passage.clone(),
                source_fingerprint: o.source_fingerprint.clone(),
            },
        };
        if !o.section.is_empty() && (!request.fields.is_empty() || request.passage.is_some()) {
            return Err(atlas_app_model::AppError::new(
                atlas_app_model::AppErrorCode::InvalidRequest,
                "Choose sections or an exact field/passage.",
            ));
        }
        if o.detail == TerminalDetail::Summary
            && o.section.is_empty()
            && request.owners.is_empty()
            && request.fields.is_empty()
            && request.passage.is_none()
            && request.spell_rank.is_none()
            && request.source_fingerprint.is_none()
        {
            let key = atlas_domain::RecordKey::parse(key).map_err(|e| {
                atlas_app_model::AppError::new(
                    atlas_app_model::AppErrorCode::InvalidRecordKey,
                    e.to_string(),
                )
            })?;
            let (generation, records) = s
                .record_summaries_at(vec![key.clone()], request.expected_artifact_generation)
                .map_err(|e| e.into_app_error())?;
            let record = records.first().ok_or_else(|| {
                atlas_app_model::AppError::new(
                    atlas_app_model::AppErrorCode::RecordNotFound,
                    format!("record `{key}` was not found"),
                )
            })?;
            if !o.json {
                print_header(record);
            }
            return Ok(json::header(
                record,
                &generation,
                &navigation::root(&key.to_string()),
            ));
        }
        let (record, excerpt) = s.record_read_at(request).map_err(|e| e.into_app_error())?;
        let abilities = if o.section.contains(&args::RecordSection::Abilities) {
            ability_reading(&s, &record)?
        } else {
            Vec::new()
        };
        if o.json {
            return json::selected(
                &record,
                o.detail,
                &o.section,
                &abilities,
                excerpt.as_deref(),
            )
            .map_err(render_error);
        }
        let mut output = String::new();
        if let Some(excerpt) = excerpt {
            presentation::selected_identity(&mut output, &record);
            output.push_str(&format!("\n{excerpt}\n"));
        } else if !o.section.is_empty() {
            output = sections::human(
                &record,
                &o.section,
                &abilities,
                terminal::TerminalLayout::current(width()),
            )
            .map_err(render_error)?;
        } else {
            let detail = if record.selected.passage.is_some() {
                TerminalDetail::Summary
            } else {
                o.detail
            };
            output = render_selected_record(&s, &record, detail)?;
        }
        if (!o.section.is_empty() && o.section.contains(&args::RecordSection::Rules))
            || (o.section.is_empty()
                && record.selected.passage.is_none()
                && (o.detail != TerminalDetail::Summary || record.selected.field.is_some()))
        {
            links::append(&mut output, &record).map_err(render_error)?;
        }
        print!("{}", terminal::wrap_facts(&output, width()));
        if !record.selected.owners.is_empty()
            || record.selected.field.is_some()
            || record.selected.passage.is_some()
            || record.selected.spell_rank.is_some()
        {
            println!(
                "Navigation: {}",
                serde_json::json!(navigation::Locator::new(
                    &record.artifact_generation,
                    &record.selected
                ))
            );
        }
        Ok(serde_json::Value::Null)
    });
    if o.json {
        write_json_response(batch.status(), &batch)?;
    } else {
        for outcome in &batch.results {
            match outcome {
                batch::Outcome::Ok { .. } => {}
                batch::Outcome::Error { input, error } => {
                    eprintln!("{input}: {}", error.message);
                    if error.code == atlas_app_model::AppErrorCode::InvalidRecordKey
                        && o.navigation.is_none()
                        && !input.trim().is_empty()
                        && !input.contains(':')
                    {
                        match super::followup::workspace_args(o.path_mode, o.index.as_deref()) {
                            Ok(workspace) => {
                                let mut args =
                                    vec!["atlas".into(), "record".into(), "resolve".into()];
                                args.extend(workspace);
                                args.extend(["--".into(), input.clone()]);
                                eprintln!(
                                    "Resolve by name: {}",
                                    super::followup::shell_command(&args)
                                );
                            }
                            Err(reason) => eprintln!("{reason}"),
                        }
                    }
                }
                batch::Outcome::Unprocessed { input, .. } => {
                    eprintln!("{input}: not processed after runtime failure")
                }
            }
        }
    }
    Ok(ExitCode::from(batch.exit))
}

#[derive(serde::Serialize)]
struct ResolvedRecord {
    selected: atlas_app_model::RecordSummaryView,
    navigation: navigation::Locator,
    match_kind: String,
    matched_text: String,
    evidence: Option<String>,
    total: usize,
}

pub(crate) fn run_record_resolve(o: RecordResolveOptions) -> Result<ExitCode, String> {
    let s = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index.clone(),
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let filter = match resolve_filter(&o.filter_options, &s, None) {
        Ok(filter) => filter,
        Err(error) => return report_app_error(error, o.json),
    };
    if let Err(error) = s.get_records(Vec::new()) {
        return report_app_error(error.into_app_error(), o.json);
    }
    let batch = batch::Batch::run(o.queries, |query| {
        let (generation, matches) = s
            .resolve_record_with_generation(query.to_owned(), filter.clone())
            .map_err(|e| e.into_app_error())?;
        let candidate = unique_match(matches, o.alternatives)?;
        let address =
            navigation::Locator::new(&generation, &navigation::root(&candidate.record.record_key));
        Ok(ResolvedRecord {
            selected: candidate.record,
            navigation: address,
            match_kind: candidate.match_kind,
            matched_text: candidate.matched_text,
            evidence: candidate.evidence,
            total: 1,
        })
    });
    if o.json {
        write_json_response(batch.status(), &batch)?;
    } else {
        for result in &batch.results {
            match result {
                batch::Outcome::Ok { value, .. } => {
                    let record = &value.selected;
                    println!(
                        "{} · {} · {} ({})",
                        record.title,
                        record.record_key,
                        record
                            .publication
                            .as_deref()
                            .unwrap_or("publication unavailable"),
                        value.match_kind
                    );
                    if let Some(evidence) = &value.evidence {
                        println!("  {evidence}");
                    }
                    match super::followup::workspace_args(o.path_mode, o.index.as_deref()) {
                        Ok(workspace) => {
                            let argv = super::followup::open_record(
                                &value.navigation.address,
                                &value.navigation.artifact_generation,
                                &workspace,
                                false,
                            );
                            println!("  {}", super::followup::shell_command(&argv));
                        }
                        Err(reason) => println!("  {reason}"),
                    }
                }
                batch::Outcome::Error { input, error } => {
                    eprintln!("{input}: {}", error.message);
                    if let Some(details) = &error.details
                        && let Some(candidates) = details["alternatives"].as_array()
                    {
                        for c in candidates {
                            let r = &c["record"];
                            eprintln!(
                                "  {} · {} · {}",
                                r["title"].as_str().unwrap_or_default(),
                                r["record_key"].as_str().unwrap_or_default(),
                                r["publication"]
                                    .as_str()
                                    .unwrap_or("publication unavailable")
                            );
                        }
                        eprintln!("  {} of {} matches", candidates.len(), details["total"]);
                    }
                }
                batch::Outcome::Unprocessed { input, .. } => {
                    eprintln!("{input}: not processed after runtime failure")
                }
            }
        }
    }
    Ok(ExitCode::from(batch.exit))
}

fn unique_match(
    mut candidates: Vec<atlas_app_model::RecordResolutionCandidateView>,
    limit: usize,
) -> Result<atlas_app_model::RecordResolutionCandidateView, atlas_app_model::AppError> {
    use atlas_app_model::{AppError, AppErrorCode};
    if candidates.len() == 1 {
        Ok(candidates.remove(0))
    } else {
        let (code, message) = if candidates.is_empty() {
            (
                AppErrorCode::RecordResolutionMiss,
                "no strict name or verified alias matched",
            )
        } else {
            (
                AppErrorCode::RecordResolutionAmbiguous,
                "ambiguous reference; specify a Foundry key",
            )
        };
        let total = candidates.len();
        candidates.truncate(limit);
        Err(AppError::new(code, message)
            .with_details(serde_json::json!({"total":total,"alternatives": candidates})))
    }
}

fn render_error(message: String) -> atlas_app_model::AppError {
    atlas_app_model::AppError::new(atlas_app_model::AppErrorCode::InternalError, message)
}

fn width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|v| *v >= 20)
        .unwrap_or(80)
}
fn render_selected_record(
    service: &atlas_app_service::AtlasAppService,
    record: &RecordDetailView,
    detail: TerminalDetail,
) -> Result<String, atlas_app_model::AppError> {
    let reading = if detail == TerminalDetail::Summary {
        Vec::new()
    } else {
        ability_reading(service, record)?
    };
    let mut output = presentation::render_record(
        record,
        detail,
        terminal::TerminalLayout::current(width()),
        &reading,
    )
    .map_err(|error| {
        atlas_app_model::AppError::new(atlas_app_model::AppErrorCode::InternalError, error)
    })?;
    if detail == TerminalDetail::Full && record.selected.field.is_none() {
        let activities = match &record.presentation.body {
            RecordBodyView::Creature(actor) => actor.activities.value.as_deref(),
            RecordBodyView::Hazard(hazard) => hazard.actor.activities.value.as_deref(),
            _ => None,
        };
        let navigation: Vec<_> = if let Some(activities) = activities {
            activities
                .iter()
                .map(|activity| &activity.navigation)
                .collect()
        } else {
            match &record.presentation.body {
                RecordBodyView::RollTable(table) => table
                    .results
                    .value
                    .iter()
                    .flatten()
                    .map(|row| &row.link.navigation)
                    .collect(),
                RecordBodyView::PhysicalReference(physical) => physical
                    .consumable
                    .as_ref()
                    .and_then(|c| c.spell.value.as_ref())
                    .map(|s| &s.navigation)
                    .into_iter()
                    .collect(),
                RecordBodyView::Journal(journal) => journal
                    .pages
                    .value
                    .iter()
                    .flatten()
                    .map(|page| &page.navigation)
                    .collect(),
                _ => record
                    .presentation
                    .owned
                    .iter()
                    .map(|owned| &owned.navigation)
                    .collect(),
            }
        };
        for navigation in navigation {
            // Actor abilities are already read inline from the checked parent.
            if reading.iter().any(|entry| &entry.navigation == navigation) {
                continue;
            }
            let child = service
                .record_detail_at(RecordDetailRequest {
                    expected_artifact_generation: Some(record.artifact_generation.clone()),
                    spell_rank: navigation.spell_rank,
                    record_key: navigation.record_key.clone(),
                    owners: navigation.owners.clone(),
                    fields: vec![],
                    passage: None,
                    source_fingerprint: navigation.source_fingerprint.clone(),
                })
                .map_err(|error| error.into_app_error())?;
            output.push_str("\n────────────────────────────────────────\n");
            output.push_str(&render_selected_record(service, &child, detail)?);
        }
    }
    Ok(output)
}
fn print_header(record: &atlas_app_model::RecordSummaryView) {
    let mut output = String::new();
    presentation::identity(&mut output, record);
    print!("{output}");
}
pub(super) fn resolve_seed(
    s: &atlas_app_service::AtlasAppService,
    name: String,
    json: bool,
) -> Result<Result<atlas_domain::RecordKey, ExitCode>, String> {
    let result = match s.resolve_record_ref(name, None, None) {
        Ok(value) => value,
        Err(error) => return Ok(Err(report_app_error(error.into_app_error(), json)?)),
    };
    let (code, message, candidates) = match result {
        atlas_app_model::RecordRefResolutionView::Key(k) => return Ok(Ok(k)),
        atlas_app_model::RecordRefResolutionView::Miss => (
            atlas_app_model::AppErrorCode::RecordResolutionMiss,
            "no strict name or verified alias matched",
            vec![],
        ),
        atlas_app_model::RecordRefResolutionView::Ambiguous(matches) => (
            atlas_app_model::AppErrorCode::RecordResolutionAmbiguous,
            "ambiguous reference; specify a Foundry key",
            matches.iter().map(|m| m.record.clone()).collect(),
        ),
    };
    Ok(Err(report_app_error(
        atlas_app_model::AppError::new(code, message)
            .with_details(serde_json::json!({"candidates":candidates})),
        json,
    )?))
}

fn ability_reading(
    service: &atlas_app_service::AtlasAppService,
    record: &RecordDetailView,
) -> Result<Vec<EmbeddedReadingEntryView>, atlas_app_model::AppError> {
    let mut reading = Vec::new();
    if record.selected.owners.is_empty()
        && record.selected.field.is_none()
        && sections::actor(&record.presentation.body).is_some()
    {
        let mut cursor = None;
        loop {
            let batch = service
                .embedded_reading(EmbeddedReadingRequest {
                    parent: EmbeddedReadingParent {
                        record_key: record.selected.record_key.clone(),
                    },
                    artifact_generation: record.artifact_generation.clone(),
                    selection: EmbeddedReadingSelection::Abilities {
                        group: None,
                        cursor,
                        target: None,
                    },
                })
                .map_err(|e| e.into_app_error())?;
            reading.extend(batch.entries);
            cursor = batch.next;
            if cursor.is_none() {
                break;
            }
        }
    }
    Ok(reading)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn html_library_preserves_structural_terminal_formatting_at_multiple_widths() {
        let html = "<h2>Ghoul Fever</h2><p><strong>Saving Throw</strong> DC 23 Fortitude</p><hr><ul><li>Stage one</li><li>Stage two</li></ul><table><caption>Local stages</caption><tr><th>Stage</th><th>Effect</th></tr><tr><td>1</td><td>Fever</td></tr></table><ol start='5'><li value='8'>Stage eight</li><li>Stage nine</li></ol><p><a href='https://example.com'>Reference</a><span data-atlas-interaction='0' data-atlas-interaction-kind='check'>DC 23 Fortitude</span></p><span>[image unavailable: Foundry portrait]</span>";
        let html = format!(
            "{html}<p>The creature carries the affliction within its own ability description, so opening this passage preserves its local saving throw and all stages.</p>"
        );
        for (width, expected) in [
            (40, include_str!("../../tests/golden/prepared-40.txt")),
            (80, include_str!("../../tests/golden/prepared-80.txt")),
            (120, include_str!("../../tests/golden/prepared-120.txt")),
        ] {
            let text = render_html(
                &html,
                terminal::TerminalLayout {
                    width,
                    styled: false,
                },
            )
            .expect("render");
            let normalized = text
                .lines()
                .map(str::trim_end)
                .collect::<Vec<_>>()
                .join("\n");
            let expected = expected.trim_end().lines().collect::<Vec<_>>().join("\n");
            assert_eq!(normalized, expected, "terminal width {width}");
        }
    }
}
