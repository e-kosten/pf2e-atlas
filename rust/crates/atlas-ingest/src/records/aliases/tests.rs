use super::*;

#[test]
fn expands_grouped_alias_text_by_target_count() {
    assert_eq!(
        expand_grouped_alias_text("Produce Flame (Melee, Ranged)", 2),
        Some(vec![
            "Produce Flame (Melee)".to_string(),
            "Produce Flame (Ranged)".to_string(),
        ])
    );
    assert_eq!(
        expand_grouped_alias_text("Produce Flame (Melee, Ranged)", 1),
        None
    );
}

#[test]
fn extracts_migration_rename_pairs_from_source_comments() {
    let source = r#"
            // Rename all uses and mentions of "Flat-Footed" to "Off-Guard".
            // Rename all uses and mentions of "Produce Flame" to "Ignition".
        "#;

    assert_eq!(
        migration_rename_pairs(source),
        vec![
            ("Flat-Footed".to_string(), "Off-Guard".to_string()),
            ("Produce Flame".to_string(), "Ignition".to_string()),
        ]
    );
}

#[test]
fn html_text_keeps_uuid_display_text() {
    assert_eq!(
        html_text(r#"<td>@UUID[Compendium.pf2e.foo.Item.abc]{Off-Guard}</td>"#),
        "Off-Guard"
    );
}
