use atlas_record::{ContentBlock, ContentInline, ContentReferenceLocator, render_plain_text};

use super::parse_foundry_content;

#[test]
fn parses_headings_strong_text_and_uuid_references() {
    let parsed = parse_foundry_content(
        "<h2>Effect</h2><p><strong>Stage 1</strong> \
         @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1}</p>",
    );

    assert_eq!(
        render_plain_text(&parsed.document),
        "Effect\nStage 1 Sickened 1"
    );
    let ContentBlock::Paragraph { content } = &parsed.document.blocks[1] else {
        panic!("second block should be paragraph");
    };
    assert!(matches!(content[0], ContentInline::Strong { .. }));
    let ContentInline::Reference { reference } = &content[2] else {
        panic!("third inline should be reference");
    };
    assert_eq!(
        reference.locator,
        ContentReferenceLocator::FoundryUuid {
            raw_target: "Compendium.pf2e.conditionitems.Item.Sickened".to_string()
        }
    );
}

#[test]
fn parses_lists_tables_rolls_and_macro_signals() {
    let parsed = parse_foundry_content(
        "<ul><li><p>@Check[fortitude|dc:21] [[/r 2d6]]</p></li></ul>\
         <table><caption>Treasure</caption><tr><th>Level</th></tr><tr><td>1</td></tr></table>",
    );

    assert_eq!(parsed.document.blocks.len(), 2);
    assert_eq!(
        render_plain_text(&parsed.document),
        "fortitude 2d6\nTreasure\nLevel\n1"
    );
    assert!(parsed.diagnostics.dropped_macros.is_empty());
}

#[test]
fn preserves_template_macros_and_reports_unknown_tags() {
    let parsed = parse_foundry_content("<aside>@Template[type:burst|distance:10]</aside>");

    assert_eq!(parsed.diagnostics.unsupported_tags, vec!["aside"]);
    assert!(parsed.diagnostics.dropped_macros.is_empty());
    assert_eq!(render_plain_text(&parsed.document), "burst");
}
