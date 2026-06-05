use atlas_record::{FoundryLinkMacroKind, RichLinkTarget, RichNode, render_plain_text};

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
    let RichNode::HtmlElement { tag, children, .. } = &parsed.document.nodes[1] else {
        panic!("second node should be paragraph");
    };
    assert_eq!(tag, "p");
    assert!(matches!(
        children[0],
        RichNode::HtmlElement { ref tag, .. } if tag == "strong"
    ));
    let RichNode::FoundryLink { link } = &children[2] else {
        panic!("third child should be reference link");
    };
    assert_eq!(link.source.macro_kind, FoundryLinkMacroKind::Uuid);
    assert_eq!(
        link.source.authored_target,
        "Compendium.pf2e.conditionitems.Item.Sickened"
    );
    assert!(matches!(
        &link.target,
        RichLinkTarget::Unresolved { target, .. }
            if target == "Compendium.pf2e.conditionitems.Item.Sickened"
    ));
}

#[test]
fn parses_lists_tables_rolls_and_macro_signals() {
    let parsed = parse_foundry_content(
        "<ul><li><p>@Check[fortitude|dc:21] [[/r 2d6]]</p></li></ul>\
         <table><caption>Treasure</caption><tr><th>Level</th></tr><tr><td>1</td></tr></table>",
    );

    assert_eq!(parsed.document.nodes.len(), 2);
    assert_eq!(
        render_plain_text(&parsed.document),
        "fortitude 2d6\nTreasure\nLevel |\n1 |"
    );
    assert!(parsed.diagnostics.dropped_macros.is_empty());
}

#[test]
fn uuid_targets_can_contain_commas() {
    let parsed =
        parse_foundry_content("@UUID[Compendium.pf2e.actionspf2e.Item.Strike, Breathe, Rend]");

    let RichNode::FoundryLink { link } = &parsed.document.nodes[0] else {
        panic!("first node should be reference link");
    };
    assert_eq!(
        link.source.authored_target,
        "Compendium.pf2e.actionspf2e.Item.Strike, Breathe, Rend"
    );
    assert!(matches!(
        &link.target,
        RichLinkTarget::Unresolved { target, fallback_label }
            if target == "Compendium.pf2e.actionspf2e.Item.Strike, Breathe, Rend"
                && fallback_label == "Strike, Breathe, Rend"
    ));
}

#[test]
fn preserves_template_macros_and_reports_unknown_tags() {
    let parsed = parse_foundry_content("<aside>@Template[type:burst|distance:10]</aside>");

    assert_eq!(parsed.diagnostics.unsupported_tags, vec!["aside"]);
    assert!(parsed.diagnostics.dropped_macros.is_empty());
    assert_eq!(render_plain_text(&parsed.document), "burst");
}
