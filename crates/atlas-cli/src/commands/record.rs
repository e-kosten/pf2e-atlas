use super::{filters::build_filter, product::app_result};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    OwnedReadingEntryView, OwnedReadingRequest, OwnedReadingSelection, RecordBodyView,
    RecordDetailRequest, RecordDetailView,
};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;
pub(crate) mod args;
mod presentation;
pub(crate) mod source;
mod terminal;
use args::{RecordGetOptions, RecordResolveOptions, TerminalDetail};
#[cfg(test)]
use terminal::render_html;
pub(crate) fn run_record_get(o: RecordGetOptions) -> Result<ExitCode, String> {
    let s = connect(ClientOptions {
        search_profile: None,
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .map_err(|e| e.message)?;
    if o.detail == TerminalDetail::Summary
        && !o.json
        && o.owners.as_ref().is_none_or(|owners| owners.0.is_empty())
        && o.field.is_none()
        && o.passage.is_none()
        && o.rank.is_none()
    {
        for key in o.keys {
            let key = atlas_domain::RecordKey::parse(&key).map_err(|e| e.to_string())?;
            let Some(record) = s
                .get_records(vec![key.clone()])
                .map_err(|e| e.into_app_error().message)?
                .into_iter()
                .next()
            else {
                return Err(format!("record `{key}` was not found"));
            };
            print_header(&record);
        }
        return Ok(ExitCode::SUCCESS);
    }
    let mut records = Vec::new();
    let mut failed = false;
    for key in o.keys {
        let result = s.record_detail_at(RecordDetailRequest {
            spell_rank: o.rank,
            record_key: key,
            owners: o
                .owners
                .as_ref()
                .map(|owners| owners.0.clone())
                .unwrap_or_default(),
            fields: o.field.clone().into_iter().collect(),
            passage: o.passage.clone(),
            source_fingerprint: o.source_fingerprint.clone(),
        });
        match app_result(result, o.json)? {
            Some(v) => records.push(v),
            None => failed = true,
        }
    }
    if o.json {
        if !failed {
            write_json_data(serde_json::json!({"records":records}))?;
        }
    } else {
        for r in records {
            print_record(&s, &r, o.detail)?;
        }
    }
    Ok(if failed {
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    })
}
pub(crate) fn run_record_resolve(o: RecordResolveOptions) -> Result<ExitCode, String> {
    let (filter, _) = build_filter(&o.filter_options).map_err(|e| e.message)?;
    let s = connect(ClientOptions {
        search_profile: None,
        path_mode: o.path_mode.into(),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .map_err(|e| e.message)?;
    let mut rows = Vec::new();
    let mut unresolved = false;
    for query in o.queries {
        let Some(matches) = app_result(s.resolve_record(query.clone(), filter.clone()), o.json)?
        else {
            return Ok(ExitCode::from(3));
        };
        let count = matches.len();
        unresolved |= count != 1;
        let alternatives = matches.iter().take(o.alternatives).collect::<Vec<_>>();
        rows.push(serde_json::json!({"query":query,"status":match count{0=>"miss",1=>"resolved",_=>"ambiguous"},"total":count,"alternatives":alternatives}));
    }
    if o.json {
        write_json_data(serde_json::json!({"resolutions":rows}))?;
    } else {
        for row in rows {
            println!(
                "{}",
                serde_json::to_string_pretty(&row).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(if unresolved {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
fn width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|v| *v >= 20)
        .unwrap_or(80)
}
fn print_record(
    service: &atlas_app_service::AtlasAppService,
    record: &RecordDetailView,
    detail: TerminalDetail,
) -> Result<(), String> {
    print!("{}", render_selected_record(service, record, detail)?);
    Ok(())
}

fn render_selected_record(
    service: &atlas_app_service::AtlasAppService,
    record: &RecordDetailView,
    detail: TerminalDetail,
) -> Result<String, String> {
    let mut reading: Vec<OwnedReadingEntryView> = vec![];
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
                .owned_reading(OwnedReadingRequest {
                    parent: record.selected.clone(),
                    artifact_generation: record.artifact_generation.clone(),
                    selection: OwnedReadingSelection::Abilities {
                        cursor,
                        target: None,
                    },
                })
                .map_err(|e| e.into_app_error().message)?;
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
    )?;
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
            let owned = match &record.presentation.body {
                RecordBodyView::RollTable(table) => {
                    table.results.value.as_deref().unwrap_or_default()
                }
                _ => &record.presentation.owned,
            };
            owned.iter().map(|owned| &owned.navigation).collect()
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
                .map_err(|error| error.into_app_error().message)?;
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
