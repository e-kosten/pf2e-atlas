use super::{filters::build_filter, product::report_app_error};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    EmbeddedReadingEntryView, EmbeddedReadingRequest, EmbeddedReadingSelection, RecordBodyView,
    RecordDetailRequest, RecordDetailView,
};
use atlas_cli_support::write_json_response;
use std::process::ExitCode;
pub(crate) mod args;
mod batch;
mod json;
mod presentation;
pub(crate) mod source;
pub(crate) mod terminal;
use args::{RecordGetOptions, RecordResolveOptions, TerminalDetail};
#[cfg(test)]
use terminal::render_html;
pub(crate) fn run_record_get(o: RecordGetOptions) -> Result<ExitCode, String> {
    let connection = connect(ClientOptions {
        search_profile: None,
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
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
    let summary = o.detail == TerminalDetail::Summary
        && !o.json
        && o.owners.as_ref().is_none_or(|owners| owners.0.is_empty())
        && o.field.is_none()
        && o.passage.is_none()
        && o.rank.is_none();
    let batch = batch::Batch::run(o.keys, |key| {
        if summary {
            let key = atlas_domain::RecordKey::parse(key).map_err(|e| {
                atlas_app_model::AppError::new(
                    atlas_app_model::AppErrorCode::InvalidRecordKey,
                    e.to_string(),
                )
            })?;
            let record = s
                .get_records(vec![key.clone()])
                .map_err(|e| e.into_app_error())?
                .into_iter()
                .next()
                .ok_or_else(|| {
                    atlas_app_model::AppError::new(
                        atlas_app_model::AppErrorCode::RecordNotFound,
                        format!("record `{key}` was not found"),
                    )
                })?;
            print_header(&record);
            return Ok(None);
        }
        let record = s
            .record_detail_at(RecordDetailRequest {
                spell_rank: o.rank,
                record_key: key.to_owned(),
                owners: o
                    .owners
                    .as_ref()
                    .map(|owners| owners.0.clone())
                    .unwrap_or_default(),
                fields: o.field.clone().into_iter().collect(),
                passage: o.passage.clone(),
                source_fingerprint: o.source_fingerprint.clone(),
            })
            .map_err(|e| e.into_app_error())?;
        if !o.json {
            print!("{}", render_selected_record(&s, &record, o.detail)?);
            return Ok(None);
        }
        Ok(Some(record))
    });
    if o.json {
        let response = batch.map_values(|record| record.as_ref().map(json::RecordResponse::from));
        write_json_response(batch.status(), &response)?;
    } else {
        for outcome in &batch.results {
            match outcome {
                batch::Outcome::Ok { .. } => {}
                batch::Outcome::Error { input, error } => eprintln!("{input}: {}", error.message),
                batch::Outcome::Unprocessed { input, .. } => {
                    eprintln!("{input}: not processed after runtime failure")
                }
            }
        }
    }
    Ok(ExitCode::from(batch.exit))
}

pub(crate) fn run_record_resolve(o: RecordResolveOptions) -> Result<ExitCode, String> {
    let (filter, _) = match build_filter(&o.filter_options) {
        Ok(filter) => filter,
        Err(error) => {
            return report_app_error(
                atlas_app_model::AppError::new(
                    atlas_app_model::AppErrorCode::FilterInvalid,
                    error.message,
                ),
                o.json,
            );
        }
    };
    let s = match connect(ClientOptions {
        search_profile: None,
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    if let Err(error) = s.get_records(Vec::new()) {
        return report_app_error(error.into_app_error(), o.json);
    }
    let batch = batch::Batch::run(o.queries, |query| {
        let matches = s
            .resolve_record(query.to_owned(), filter.clone())
            .map_err(|e| e.into_app_error())?;
        let count = matches.len();
        let alternatives = matches.iter().take(o.alternatives).collect::<Vec<_>>();
        let value = serde_json::json!({"total":count,"alternatives":alternatives});
        match count {
            1 => Ok(value),
            _ => Err(atlas_app_model::AppError::new(
                if count == 0 {
                    atlas_app_model::AppErrorCode::RecordResolutionMiss
                } else {
                    atlas_app_model::AppErrorCode::RecordResolutionAmbiguous
                },
                if count == 0 {
                    "no strict name or verified alias matched"
                } else {
                    "ambiguous reference; specify a Foundry key"
                },
            )
            .with_details(value)),
        }
    });
    if o.json {
        write_json_response(batch.status(), &batch)?;
    } else {
        for outcome in &batch.results {
            println!(
                "{}",
                serde_json::to_string_pretty(outcome).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(ExitCode::from(batch.exit))
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
    let mut reading: Vec<EmbeddedReadingEntryView> = vec![];
    if detail != TerminalDetail::Summary
        && record.selected.owners.is_empty()
        && record.selected.field.is_none()
        && matches!(
            record.presentation.body,
            RecordBodyView::Creature(_) | RecordBodyView::Hazard(_)
        )
    {
        let mut cursor = None;
        loop {
            let batch = service
                .embedded_reading(EmbeddedReadingRequest {
                    parent: record.selected.clone(),
                    artifact_generation: record.artifact_generation.clone(),
                    selection: EmbeddedReadingSelection::Abilities {
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
            assert_eq!(normalized, expected.trim_end(), "terminal width {width}");
            assert!(text.contains("Ghoul Fever"));
            assert!(text.contains("Saving Throw"));
            assert!(text.contains("────────────"));
            assert!(text.contains("Local stages"));
            assert!(text.contains("8. Stage eight"));
            assert!(text.contains("9. Stage nine"));
            assert!(text.contains("DC 23 Fortitude"));
            assert!(text.contains("Stage one"));
            assert!(text.contains("Fever"));
            assert!(text.contains("https://example.com"));
            assert!(text.contains("image unavailable"));
            assert!(!text.contains("data-atlas"));
            assert!(!text.contains("<table>"));
        }
    }
}
