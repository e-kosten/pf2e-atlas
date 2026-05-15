use crate::embedding_units::{
    extract_structured_embedding_units, strip_markup_for_embedding_units,
};
use crate::embeddings::EmbeddingUnitKind;

#[test]
fn heading_equal_to_record_name_does_not_create_child_unit() {
    let units = extract_structured_embedding_units(
        "Earn Income",
        "<h2>Earn Income</h2><p>This text is long enough to pass the token threshold because it describes the parent action rather than a distinct child section with a separate user intent.</p><h2>Ending or Interrupting Tasks</h2><p>When a task is complete, or if you stop in the middle of one, you normally need to find a new task before you can keep earning income from downtime work.</p>",
    );

    assert_eq!(units.len(), 1);
    assert_eq!(units[0].kind, EmbeddingUnitKind::HeadingSection);
    assert_eq!(units[0].label, "Ending or Interrupting Tasks");
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
    let units = extract_structured_embedding_units(
        "Avatar",
        "<ul><li><strong>Abadar</strong><ul><li>Speed 50 feet, burrow Speed 30 feet, immune to immobilized.</li><li>Ranged crossbow with a long range increment and piercing damage for the deity form.</li></ul></li><li><strong>Achaekek</strong><ul><li>Speed 70 feet and climb Speed 50 feet, ignoring difficult terrain.</li><li>Melee mantis claw and ranged spine volley attacks for the deity form.</li></ul></li><li><strong>Asmodeus</strong><ul><li>Speed 70 feet and fly, with mace and hell fire attacks.</li><li>The battle form uses the spell attack modifier and listed fire damage.</li></ul></li></ul>",
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
    let units = extract_structured_embedding_units(
        "Avatar",
        "<ul><li><strong>Abadar</strong> The Abadar avatar form includes a nested list of movement and attack details that should stay part of the Abadar unit.<ul><li><strong>Nested Speed</strong> This nested option has enough words to qualify if nested list items were treated as top-level siblings.</li><li><strong>Nested Strike</strong> This nested option also has enough words to qualify if nested list items were treated as top-level siblings.</li></ul></li><li><strong>Achaekek</strong> The Achaekek avatar form includes a nested list of movement and attack details that should stay part of the Achaekek unit.<ul><li><strong>Nested Climb</strong> This nested option has enough words to qualify if nested list items were treated as top-level siblings.</li><li><strong>Nested Spine</strong> This nested option also has enough words to qualify if nested list items were treated as top-level siblings.</li></ul></li><li><strong>Asmodeus</strong> The Asmodeus avatar form includes a nested list of movement and attack details that should stay part of the Asmodeus unit.<ul><li><strong>Nested Fire</strong> This nested option has enough words to qualify if nested list items were treated as top-level siblings.</li><li><strong>Nested Mace</strong> This nested option also has enough words to qualify if nested list items were treated as top-level siblings.</li></ul></li></ul>",
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
    let units = extract_structured_embedding_units(
        "Elemental Eidolon",
        "<h2>Elemental Core</h2><p>The elemental core gives the eidolon a durable identity for movement, defenses, damage, and battlefield roles. This introduction should remain searchable as the section overview while individual elemental choices become separate option units.</p><p><strong>Air:</strong> Air eidolons move with speed and precision, emphasizing mobility, flight, lightning, thunder, and evasive positioning around enemies during dangerous encounters with shifting terrain.</p><p><strong>Earth:</strong> Earth eidolons favor durability, grounded movement, stone, metal, and resilient battlefield control that helps allies withstand pressure from enemies in sustained defensive fights.</p><p><strong>Fire:</strong> Fire eidolons focus on heat, flame, persistent damage, bright destructive pressure, and aggressive offense that can punish clustered opponents before they scatter.</p>",
    );

    let labels = units
        .iter()
        .map(|unit| (unit.kind, unit.label.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec![
            (EmbeddingUnitKind::HeadingSection, "Elemental Core"),
            (EmbeddingUnitKind::TitledOption, "Air"),
            (EmbeddingUnitKind::TitledOption, "Earth"),
            (EmbeddingUnitKind::TitledOption, "Fire"),
        ]
    );
    let heading = units
        .iter()
        .find(|unit| unit.kind == EmbeddingUnitKind::HeadingSection)
        .expect("heading section unit");
    assert!(heading.body.contains("section overview"));
    assert!(!heading.body.contains("Air eidolons"));
}

#[test]
fn heading_sections_omit_nested_titled_option_lists() {
    let units = extract_structured_embedding_units(
        "Light Mortar Innovation",
        "<h3>Innovation</h3><p>Your light mortar is a field-tested prototype with modular chambers and defensive bracing. This overview should remain searchable for the innovation section while the individual modifications are searched as option children.</p><ul><li><strong>Contained Shrapnel</strong> Your innovation redirects explosive force into safer patterns, protecting allies while still pressuring enemies near the target area during complicated battlefield engagements.</li><li><strong>Enhanced Shrapnel</strong> Your innovation improves the fragment spread and makes the mortar more effective against enemies that rely on tight formations and defensive positioning.</li><li><strong>Spring-Loaded</strong> Your innovation prepares a faster reload sequence, helping you recover between attacks and keep pressure during extended encounters with multiple opponents.</li></ul>",
    );

    let heading = units
        .iter()
        .find(|unit| unit.kind == EmbeddingUnitKind::HeadingSection)
        .expect("heading section unit");
    assert!(heading.body.contains("field-tested prototype"));
    assert!(!heading.body.contains("Contained Shrapnel"));
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
fn activation_blocks_group_activate_through_effect() {
    let units = extract_structured_embedding_units(
        "Void Mirror",
        "<p><strong>Activate</strong> 1 hour (Interact)</p><p><strong>Research</strong> Accumulate 12 RP by making Occultism checks.</p><p><strong>Frequency</strong> once per month</p><p><strong>Effect</strong> The first activation ritual is known as \"Speak to the Void\" and allows the user to contact an intelligence in a distant part of the universe. The alien intelligence infuses the user's mind with answers, allowing Recall Knowledge with legendary proficiency, but failed checks deal mental damage.</p><p><strong>Activate</strong> 1 hour (Interact)</p><p><strong>Research</strong> Accumulate 12 RP by making more difficult Occultism checks over a longer downtime interval.</p><p><strong>Frequency</strong> once per year</p><p><strong>Effect</strong> The second activation ritual is known as \"Call from the Void\" and draws a creature across the universe with attitude determined by the user's Occultism result and Will DC. The creature can arrive helpful, indifferent, hostile, or with magical feedback depending on the outcome.</p>",
    );

    assert_eq!(units.len(), 2);
    assert_eq!(units[0].kind, EmbeddingUnitKind::ActivationBlock);
    assert_eq!(units[0].label, "Speak to the Void");
    assert_eq!(units[1].label, "Call from the Void");
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
