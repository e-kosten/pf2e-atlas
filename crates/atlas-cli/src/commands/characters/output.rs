use super::super::saved_workspace::{SelectionOptions, SheetDetail};
use atlas_app_model::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fmt::Write};

pub(crate) fn character(view: &CharacterContextView, options: &SelectionOptions) -> String {
    let c = &view.character;
    let level = c
        .level
        .map_or_else(|| "unknown level".into(), |l| format!("level {l}"));
    let mut text = format!(
        "{} · {level}\n{}  revision {}\n",
        c.name, c.character_key, c.revision
    );
    match view.source_state {
        CharacterSourceState::NoImport => text.push_str("No Pathbuilder import.\n"),
        CharacterSourceState::ReferenceOnly => text.push_str(
            "Rules reference only; re-import from Pathbuilder for equipment and sheet details.\n",
        ),
        CharacterSourceState::Available => {
            if let Some(imported) = &view.imported_identity
                && (imported.name != c.name || Some(imported.level) != c.level)
            {
                let _ = writeln!(
                    text,
                    "Imported build: {} · level {}",
                    imported.name, imported.level
                );
            }
        }
    }
    for party in &c.parties {
        let _ = writeln!(text, "Party: {} ({})", party.name, party.party_key);
    }
    let show_unavailable = options.detail == SheetDetail::Full
        || !options.sections.is_empty()
        || view.source_state == CharacterSourceState::Available;
    let visible = view
        .sections
        .iter()
        .filter(|(kind, _)| show_unavailable || **kind == CharacterSection::Abilities);
    sections(&mut text, visible, options.detail);
    text
}

fn sections<'a>(
    text: &mut String,
    sections: impl IntoIterator<Item = (&'a CharacterSection, &'a CharacterSectionView)>,
    detail: SheetDetail,
) {
    for (kind, section) in sections {
        let _ = writeln!(text, "\n{}", section_label(*kind));
        if let Some(abilities) = &section.abilities {
            let _ = writeln!(text, "{} of {} abilities", abilities.shown, abilities.total);
            for entry in &abilities.entries {
                let a = &entry.ability;
                let choice = a
                    .choice
                    .as_ref()
                    .map(|v| format!(" ({v})"))
                    .unwrap_or_default();
                let state = match entry.link_state {
                    AbilityLinkState::Automatic => "automatic",
                    AbilityLinkState::Manual => "manual",
                    AbilityLinkState::Unlinked => "unlinked",
                    AbilityLinkState::Ambiguous => "ambiguous",
                    AbilityLinkState::Unavailable => "rule unavailable",
                };
                let _ = writeln!(
                    text,
                    "  {}{choice} · {state}\n    {}",
                    a.label, a.ability_key
                );
                if let Some(key) = &a.record_key {
                    let _ = writeln!(text, "    {key}");
                }
            }
            continue;
        }
        if *kind == CharacterSection::Source {
            if let Some(source) = &section.source_json {
                text.push_str(source);
                if !source.ends_with('\n') {
                    text.push('\n');
                }
            } else {
                text.push_str("  Original export unavailable; re-import from Pathbuilder.\n");
            }
            continue;
        }
        let has_rows = section
            .presentation
            .iter()
            .any(|g| !g.rows.is_empty() || !g.spell_ranks.is_empty());
        if detail == SheetDetail::Full || !has_rows {
            for (key, fact) in &section.fields {
                fact_text(text, key, fact, 2);
            }
            for link in &section.record_links {
                let _ = writeln!(text, "  Rule: {}\n    {}", link.title, link.record_key);
            }
        } else {
            for (key, fact) in &section.fields {
                match fact {
                    ImportedFactView::Missing
                    | ImportedFactView::Null
                    | ImportedFactView::Unrecognized => fact_text(text, key, fact, 2),
                    ImportedFactView::Value(ImportedFactValue::List(values))
                        if values.is_empty() =>
                    {
                        fact_text(text, key, fact, 2)
                    }
                    _ => {}
                }
            }
            for group in &section.presentation {
                if group.rows.is_empty() && group.spell_ranks.is_empty() {
                    continue;
                }
                let _ = writeln!(text, "  {}", group.label);
                for row in &group.rows {
                    let summary = row
                        .summary_value
                        .as_ref()
                        .map(|v| format!(": {v}"))
                        .unwrap_or_default();
                    let _ = writeln!(text, "    {}{summary}", row.label);
                    for fact in &row.facts {
                        let _ = writeln!(text, "      {}: {}", fact.label, fact.value);
                    }
                    if let Some(key) = &row.record_key {
                        let _ = writeln!(text, "      {key}");
                    }
                }
                for rank in &group.spell_ranks {
                    let _ = writeln!(
                        text,
                        "    {} · slots {}",
                        rank.label,
                        rank.slots
                            .map_or_else(|| "unknown".into(), |n| n.to_string())
                    );
                    for (label, list) in [("Known", &rank.known), ("Prepared", &rank.prepared)] {
                        for spell in list {
                            let _ =
                                writeln!(text, "      {label}: {} ×{}", spell.label, spell.count);
                        }
                    }
                }
            }
        }
    }
}

/// Standard JSON keeps imported scalars typed. An absent object member means the
/// export omitted it. A missing list element needs a tag to preserve its position.
fn compact_fields(fields: &BTreeMap<String, ImportedFactView>) -> Value {
    Value::Object(
        fields
            .iter()
            .filter(|(_, value)| !matches!(value, ImportedFactView::Missing))
            .map(|(key, value)| (key.clone(), compact_fact(value)))
            .collect(),
    )
}

fn compact_fact(fact: &ImportedFactView) -> Value {
    match fact {
        ImportedFactView::Missing => json!({"state": "missing"}),
        ImportedFactView::Null => Value::Null,
        ImportedFactView::Unrecognized => json!({"state": "unrecognized"}),
        ImportedFactView::Value(value) => match value {
            ImportedFactValue::Text(value) => json!(value),
            ImportedFactValue::Integer(value) => json!(value),
            ImportedFactValue::Boolean(value) => json!(value),
            ImportedFactValue::List(entries) => {
                Value::Array(entries.iter().map(compact_fact).collect())
            }
            ImportedFactValue::Object(fields) => compact_fields(fields),
        },
    }
}

fn equipment_group(group: &SheetGroupView) -> Value {
    let items: Vec<_> = group
        .rows
        .iter()
        .filter(|row| row.kind != Some(SheetRowKind::ContainerMetadata))
        .map(|row| {
            let mut item = json!({"name": row.label});
            if let Some(key) = &row.record_key {
                item["record_key"] = json!(key);
            }
            for fact in &row.facts {
                match fact.role {
                    Some(SheetFactRole::Quantity) => item["quantity_label"] = json!(fact.value),
                    Some(SheetFactRole::State) => item["state_label"] = json!(fact.value),
                    _ => {}
                }
            }
            item
        })
        .collect();
    let mut result = json!({"label": group.label, "items": items});
    if let Some(id) = &group.container_id {
        result["container_id"] = json!(id);
    }
    result
}

pub(crate) fn compact_character(view: &CharacterContextView) -> Value {
    let sections = view
        .sections
        .iter()
        .map(|(kind, section)| {
            let compact = match kind {
                CharacterSection::Source => json!({"source_json": section.source_json}),
                CharacterSection::Abilities => json!(section.abilities),
                _ => {
                    let mut result = json!({"fields": compact_fields(&section.fields)});
                    if *kind == CharacterSection::Equipment {
                        let groups: Vec<_> = section
                            .presentation
                            .iter()
                            .filter(|group| !group.rows.is_empty())
                            .map(equipment_group)
                            .collect();
                        if !groups.is_empty() {
                            result["groups"] = json!(groups);
                        }
                    } else if !section.record_links.is_empty() {
                        result["record_links"] = json!(section.record_links);
                    }
                    result
                }
            };
            (kind, compact)
        })
        .collect::<BTreeMap<_, _>>();
    json!({"character":view.character,"source_state":view.source_state,"imported_identity":view.imported_identity,"selected_sections":view.selected_sections,"sections":sections})
}

fn fact_text(text: &mut String, key: &str, fact: &ImportedFactView, indent: usize) {
    let key = key.replace('_', " ");
    match fact {
        ImportedFactView::Missing => {
            let _ = writeln!(text, "{:indent$}{key}: not supplied", "");
        }
        ImportedFactView::Null => {
            let _ = writeln!(text, "{:indent$}{key}: null", "");
        }
        ImportedFactView::Unrecognized => {
            let _ = writeln!(text, "{:indent$}{key}: unrecognized export value", "");
        }
        ImportedFactView::Value(value) => match value {
            ImportedFactValue::Text(value) => {
                let _ = writeln!(text, "{:indent$}{key}: {value}", "");
            }
            ImportedFactValue::Integer(value) => {
                let _ = writeln!(text, "{:indent$}{key}: {value}", "");
            }
            ImportedFactValue::Boolean(value) => {
                let _ = writeln!(text, "{:indent$}{key}: {value}", "");
            }
            ImportedFactValue::List(entries) => {
                let unit = if entries.len() == 1 {
                    "entry"
                } else {
                    "entries"
                };
                let _ = writeln!(text, "{:indent$}{key}: {} {unit}", "", entries.len());
                for (i, entry) in entries.iter().enumerate() {
                    fact_text(text, &(i + 1).to_string(), entry, indent + 2);
                }
            }
            ImportedFactValue::Object(fields) => {
                let _ = writeln!(text, "{:indent$}{key}:", "");
                for (key, field) in fields {
                    fact_text(text, key, field, indent + 2);
                }
            }
        },
    }
}

fn section_label(section: CharacterSection) -> &'static str {
    match section {
        CharacterSection::Identity => "Imported identity",
        CharacterSection::Statistics => "Statistics from imported build",
        CharacterSection::Proficiencies => "Proficiencies",
        CharacterSection::CharacterOptions => "Character options",
        CharacterSection::Abilities => "Ability rules",
        CharacterSection::Spellcasting => "Spellcasting",
        CharacterSection::Equipment => "Equipment",
        CharacterSection::Money => "Money",
        CharacterSection::Crafting => "Crafting",
        CharacterSection::Companions => "Companions",
        CharacterSection::Source => "Original Pathbuilder export",
    }
}

pub(crate) fn preview(view: &CharacterImportPreviewView, detail: SheetDetail) -> String {
    let mut text = format!(
        "Proposed import: {} · level {}\nSource SHA-256: {}\nArtifact generation: {}\n",
        view.name, view.level, view.source_sha256, view.artifact_generation
    );
    if let Some(current) = &view.current {
        let level = current
            .character
            .level
            .map_or_else(|| "unknown".into(), |level| level.to_string());
        let _ = writeln!(
            text,
            "Current: {} · level {level} · revision {}",
            current.character.name, current.character.revision
        );
        text.push_str(
            "Refresh replaces the saved name and level, imported source, and rule reference.\n",
        );
    }
    sections(&mut text, &view.proposed, detail);
    if let Some(changes) = &view.changes {
        text.push_str("\nChanges from saved import\n");
        for section in &changes.sections {
            let _ = writeln!(text, "  {}", section.label);
            for (label, rows) in [
                ("Added", &section.added),
                ("Changed", &section.changed),
                ("Removed", &section.removed),
            ] {
                for row in rows {
                    let _ = writeln!(text, "    {label}: {row}");
                }
            }
        }
        let _ = writeln!(
            text,
            "  {} manual corrections retained",
            changes.retained_corrections
        );
        for removed in &changes.removed_corrections {
            let _ = writeln!(text, "  Correction removed: {removed}");
        }
    }
    text
}
