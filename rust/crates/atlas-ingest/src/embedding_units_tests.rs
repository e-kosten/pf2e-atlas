use crate::embedding_units::{
    extract_structured_embedding_units, strip_markup_for_embedding_units,
};
use atlas_embedding::EmbeddingUnitKind;

fn filler(word: &str, count: usize) -> String {
    std::iter::repeat_n(word, count)
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn compact_heading_document_does_not_create_child_units() {
    let units = extract_structured_embedding_units(
        "Earn Income",
        "<h2>Earn Income</h2><p>This text is long enough to pass the token threshold because it describes the parent action rather than a distinct child section with a separate user intent.</p><h2>Ending or Interrupting Tasks</h2><p>When a task is complete, or if you stop in the middle of one, you normally need to find a new task before you can keep earning income from downtime work.</p>",
    );

    assert!(units.is_empty());
}

#[test]
fn long_heading_equal_to_record_name_does_not_create_child_unit() {
    let units = extract_structured_embedding_units(
        "Earn Income",
        &format!(
            "<h2>Earn Income</h2><p>{}</p><h2>Ending or Interrupting Tasks</h2><p>{}</p>",
            filler("downtime", 220),
            filler("interruption", 220)
        ),
    );

    assert_eq!(units.len(), 1);
    assert_eq!(units[0].kind, EmbeddingUnitKind::HeadingSection);
    assert_eq!(units[0].label, "Ending or Interrupting Tasks");
}

#[test]
fn record_name_heading_can_still_split_nested_sections() {
    let units = extract_structured_embedding_units(
        "Downtime Procedures",
        &format!(
            "<h1>Downtime Procedures</h1><p>{}</p><h2>Earn Income</h2><p>{}</p><h2>Craft</h2><p>{}</p>",
            filler("overview", 80),
            filler("income", 180),
            filler("crafting", 180)
        ),
    );

    assert_eq!(
        units
            .iter()
            .map(|unit| (unit.kind, unit.label.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (EmbeddingUnitKind::HeadingSection, "Earn Income"),
            (EmbeddingUnitKind::HeadingSection, "Craft"),
        ]
    );
}

#[test]
fn long_heading_document_splits_only_to_needed_nested_level() {
    let units = extract_structured_embedding_units(
        "Downtime Procedures",
        &format!(
            "<h1>Chapter One</h1><p>{}</p><h2>First Task</h2><p>{}</p><h2>Second Task</h2><p>{}</p><h1>Chapter Two</h1><p>{}</p>",
            filler("overview", 40),
            filler("crafting", 140),
            filler("income", 140),
            filler("resting", 120)
        ),
    );

    assert_eq!(
        units
            .iter()
            .map(|unit| (unit.kind, unit.label.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (EmbeddingUnitKind::HeadingSection, "First Task"),
            (EmbeddingUnitKind::HeadingSection, "Second Task"),
            (EmbeddingUnitKind::HeadingSection, "Chapter Two"),
        ]
    );
}

#[test]
fn strong_result_labels_do_not_create_heading_units() {
    let units = extract_structured_embedding_units(
        "Craft",
        "<p><strong>Critical Success</strong> Your attempt is exceptionally successful. Each additional day spent Crafting reduces the materials needed to complete the item and gives you better progress toward completion.</p><p><strong>Success</strong> Your attempt is successful. Each additional day spent Crafting reduces the materials needed to complete the item and lets you continue downtime progress.</p><p><strong>Failure</strong> Your attempt is unsuccessful but you can continue working on the item with additional downtime and new checks later.</p><p><strong>Critical Failure</strong> Your attempt fails badly enough that materials can be wasted and your progress toward the item is interrupted.</p>",
    );

    assert!(units.is_empty());
}

#[test]
fn titled_option_lists_create_units_when_siblings_qualify() {
    let detail = filler("avatar", 140);
    let units = extract_structured_embedding_units(
        "Avatar",
        &format!(
            "<ul><li><strong>Abadar</strong> {detail}</li><li><strong>Achaekek</strong> {detail}</li><li><strong>Asmodeus</strong> {detail}</li></ul>"
        ),
    );

    assert_eq!(
        units
            .iter()
            .map(|unit| (unit.kind, unit.label.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (EmbeddingUnitKind::TitledOption, "Abadar"),
            (EmbeddingUnitKind::TitledOption, "Achaekek"),
            (EmbeddingUnitKind::TitledOption, "Asmodeus"),
        ]
    );
}

#[test]
fn titled_option_lists_ignore_nested_list_items() {
    let detail = filler("avatar", 140);
    let units = extract_structured_embedding_units(
        "Avatar",
        &format!(
            "<ul><li><strong>Abadar</strong> {detail}<ul><li><strong>Nested Speed</strong> {detail}</li><li><strong>Nested Strike</strong> {detail}</li></ul></li><li><strong>Achaekek</strong> {detail}<ul><li><strong>Nested Climb</strong> {detail}</li><li><strong>Nested Spine</strong> {detail}</li></ul></li><li><strong>Asmodeus</strong> {detail}<ul><li><strong>Nested Fire</strong> {detail}</li><li><strong>Nested Mace</strong> {detail}</li></ul></li></ul>"
        ),
    );

    assert_eq!(
        units
            .iter()
            .map(|unit| unit.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Abadar", "Achaekek", "Asmodeus"]
    );
}

#[test]
fn titled_option_paragraphs_create_units_when_siblings_qualify() {
    let overview = filler("overview", 80);
    let detail = filler("eidolon", 120);
    let units = extract_structured_embedding_units(
        "Elemental Eidolon",
        &format!(
            "<h2>Elemental Core</h2><p>{overview}</p><p><strong>Air:</strong> {detail}</p><p><strong>Earth:</strong> {detail}</p><p><strong>Fire:</strong> {detail}</p>"
        ),
    );

    let labels = units
        .iter()
        .map(|unit| (unit.kind, unit.label.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec![
            (EmbeddingUnitKind::TitledOption, "Air"),
            (EmbeddingUnitKind::TitledOption, "Earth"),
            (EmbeddingUnitKind::TitledOption, "Fire"),
        ]
    );
}

#[test]
fn heading_sections_omit_nested_titled_option_lists() {
    let overview = filler("mortar", 80);
    let detail = filler("shrapnel", 120);
    let units = extract_structured_embedding_units(
        "Light Mortar Innovation",
        &format!(
            "<h3>Innovation</h3><p>{overview}</p><ul><li><strong>Contained Shrapnel</strong> {detail}</li><li><strong>Enhanced Shrapnel</strong> {detail}</li><li><strong>Spring-Loaded</strong> {detail}</li></ul>"
        ),
    );

    assert_eq!(
        units
            .iter()
            .filter(|unit| unit.kind == EmbeddingUnitKind::TitledOption)
            .map(|unit| unit.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Contained Shrapnel", "Enhanced Shrapnel", "Spring-Loaded"]
    );
}

#[test]
fn activation_blocks_are_not_emitted_by_active_extractor() {
    let detail = filler("ritual", 220);
    let units = extract_structured_embedding_units(
        "Void Mirror",
        &format!(
            "<p><strong>Activate</strong> 1 hour (Interact)</p><p><strong>Research</strong> {detail}</p><p><strong>Frequency</strong> once per month</p><p><strong>Effect</strong> The first activation ritual is known as \"Speak to the Void\" and {detail}</p><p><strong>Activate</strong> 1 hour (Interact)</p><p><strong>Research</strong> {detail}</p><p><strong>Frequency</strong> once per year</p><p><strong>Effect</strong> The second activation ritual is known as \"Call from the Void\" and {detail}</p>"
        ),
    );

    assert!(units.is_empty());
}

#[test]
fn tables_keep_headers_and_drop_body_cells() {
    let rendered = strip_markup_for_embedding_units(
        "<h2>Table 4-2: Income Earned</h2><table><thead><tr><th>Task Level</th><th>Trained</th></tr></thead><tbody><tr><td>1</td><td>2 sp</td></tr></tbody></table>",
    );

    assert!(rendered.contains("Table 4-2: Income Earned"));
    assert!(rendered.contains("Task Level"));
    assert!(rendered.contains("Trained"));
    assert!(!rendered.contains("2 sp"));
}
