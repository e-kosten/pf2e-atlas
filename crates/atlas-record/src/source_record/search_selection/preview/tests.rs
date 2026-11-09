use super::*;
use crate::source_record::{SOURCE_CONTENT_SELECTION_VERSION, select_prepared_html_sections};

fn address(html: &str, ordinal: usize, window: Option<SourceByteRange>) -> SourcePassageAddress {
    let sections = select_prepared_html_sections(html).unwrap();
    let section = &sections[ordinal];
    SourcePassageAddress::HtmlSection {
        prepared_html_sha256: hash(html),
        canonical_text_sha256: hash(&section.text),
        selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
        section_ordinal: ordinal,
        label: section.label.clone(),
        chunk_bytes: window.unwrap_or(SourceByteRange {
            start: 0,
            end: section.text.len(),
        }),
    }
}

fn preview(html: &str, ordinal: usize) -> Option<SourceMatchPreview> {
    recover_source_match_previews(Some(html), None, &[&address(html, ordinal, None)])
        .unwrap()
        .pop()
        .flatten()
        .and_then(|evidence| evidence.preview)
}

#[test]
fn table_preview_uses_only_unambiguous_column_headers_and_preserves_passage() {
    let html = "<h2>Undead Skeletons</h2><table><thead><tr><th>Level</th><th>Size</th><th>Creature Name</th></tr></thead><tbody><tr><td>-1</td><td>Medium</td><td>Skeleton Guard</td></tr></tbody></table>";
    assert_eq!(
        preview(html, 0),
        Some(SourceMatchPreview::Table {
            columns: ["Level", "Size", "Creature Name"]
                .map(str::to_owned)
                .to_vec(),
        })
    );
    let address = address(html, 0, None);
    let raw = recover_source_passage(Some(html), None, &address).unwrap();
    assert!(raw.contains("Skeleton Guard"));
    assert!(raw.contains('─'));
    assert_eq!(
        preview(
            "<table><tr><td>Basic Actions</td><td>Strike</td></tr></table>",
            0
        ),
        Some(SourceMatchPreview::Table { columns: vec![] })
    );
}

#[test]
fn prose_uses_authored_outcome_and_stage_breaks() {
    let html = "<h2>Confusion</h2><p><strong>Critical Success</strong> The target is unaffected.</p><p><strong>Success</strong> The target babbles incoherently.</p>";
    assert_eq!(
        preview(html, 0),
        Some(SourceMatchPreview::Text {
            blocks: [
                "Critical Success The target is unaffected.",
                "Success The target babbles incoherently."
            ]
            .map(str::to_owned)
            .to_vec(),
            truncated: false,
        })
    );
    let poison = "<p><strong>DC</strong> 38<br><strong>Maximum Duration</strong> 6 rounds<br><strong>Stage 1</strong> 2d6 poison damage<br><strong>Stage 2</strong> 3d6 poison damage</p>";
    assert!(
        matches!(preview(poison, 0), Some(SourceMatchPreview::Text { blocks, .. }) if blocks.len() == 4)
    );
}

#[test]
fn mixed_sections_keep_proven_prose_and_omit_table_only_windows() {
    let html = "<h2>Dragon Form</h2><p>Exhale magical energy in an area.</p><table><tr><th>Dragon Type</th><th>Saving Throw</th></tr><tr><td>Black</td><td>Reflex</td></tr></table>";
    let text = select_prepared_html_sections(html).unwrap().remove(0).text;
    let boundary = text.find('─').unwrap();
    let table = address(
        html,
        0,
        Some(SourceByteRange {
            start: boundary,
            end: text.len(),
        }),
    );
    let prose = address(
        html,
        0,
        Some(SourceByteRange {
            start: 0,
            end: boundary,
        }),
    );
    let previews = recover_source_match_previews(Some(html), None, &[&table, &prose])
        .unwrap()
        .into_iter()
        .map(|evidence| evidence.and_then(|e| e.preview))
        .collect::<Vec<_>>();
    assert_eq!(
        previews,
        vec![
            None,
            Some(SourceMatchPreview::Text {
                blocks: vec!["Exhale magical energy in an area.".into()],
                truncated: false,
            })
        ]
    );
}

#[test]
fn unresolved_interaction_cannot_leak_even_from_a_partial_window() {
    let html = r#"<p>Exhale, dealing <span data-atlas-interaction="1" data-atlas-interaction-kind="damage">(@actor.level)d6[@actor.flags.system.barbarian.instinct.dragon.damageType]</span> damage.</p>"#;
    assert_eq!(preview(html, 0), None);
    let text = select_prepared_html_sections(html).unwrap().remove(0).text;
    let start = text.find("actor.flags").unwrap();
    let partial = address(
        html,
        0,
        Some(SourceByteRange {
            start,
            end: text.len(),
        }),
    );
    assert_eq!(
        recover_source_match_previews(Some(html), None, &[&partial]).unwrap()[0]
            .as_ref()
            .unwrap()
            .preview,
        None
    );
}

#[test]
fn shortening_keeps_unicode_and_whole_later_blocks() {
    assert_eq!(
        compact_blocks(["A complete first line.", &"🐉".repeat(240)], false),
        Some(SourceMatchPreview::Text {
            blocks: vec!["A complete first line.".into()],
            truncated: true
        })
    );
    assert!(matches!(
        compact_blocks(["🐉".repeat(241).as_str()], false),
        Some(SourceMatchPreview::Text { blocks, truncated: true }) if blocks[0].chars().count() == 240
    ));
}

#[test]
fn checked_nested_headings_supply_context_for_same_named_sections() {
    let html = "<h2>First Form</h2><h3>Transformation</h3><p>First body.</p><h2>Second Form</h2><h3>Transformation</h3><p>Second body.</p>";
    let sections = select_prepared_html_sections(html).unwrap();
    let addresses = sections
        .iter()
        .filter(|section| section.label.as_deref() == Some("Transformation"))
        .map(|section| address(html, section.section_ordinal, None))
        .collect::<Vec<_>>();
    let evidence =
        recover_source_match_previews(Some(html), None, &addresses.iter().collect::<Vec<_>>())
            .unwrap();
    assert_eq!(
        evidence
            .into_iter()
            .map(|e| e.unwrap().parent_heading.unwrap())
            .collect::<Vec<_>>(),
        ["First Form", "Second Form"]
    );
}
