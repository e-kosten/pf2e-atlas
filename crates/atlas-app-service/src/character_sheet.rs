mod details;
mod inventory;
mod magic;
mod stats;

use crate::{
    AppServiceResult, AtlasAppService,
    pathbuilder_sheet::{ImportedField, PathbuilderSheet, read_sheet},
};
use atlas_app_model::{
    CharacterSheetResponse, CharacterSheetView, SheetFactView, SheetGroupView, SheetRowView,
    SheetSectionKind, SheetSectionView,
};
use atlas_domain::normalize_record_name;
use atlas_local_state::{PcAbilityKind, PcReference};
use atlas_search::AtlasRetrievalService;
use std::{collections::HashMap, fmt::Display};

pub(super) use details::compare_imports;

impl AtlasAppService {
    pub fn character_sheet(&self, key: &str) -> AppServiceResult<CharacterSheetResponse> {
        let key = key.to_owned();
        self.with_saved_detail(
            |runtime, state| runtime.reference_read(state, None),
            move |state| {
                let characters = state.local_state()?.characters();
                let character = characters.get(&key)?;
                let source = characters.pathbuilder_source(&key)?;
                let needs_catalog = source.is_some();
                Ok(((character, source), needs_catalog))
            },
            |(character, source), reference| {
                let sheet = source
                    .map(|source| {
                        let parsed = read_sheet(source.json())?;
                        project_sheet(
                            &parsed,
                            character.reference.as_ref(),
                            reference.ok().flatten().map(|session| session.retrieval()),
                        )
                    })
                    .transpose()?;
                Ok(CharacterSheetResponse {
                    character_key: character.character_key,
                    character_revision: character.revision,
                    sheet,
                })
            },
        )
    }
}

pub(super) fn project_sheet(
    sheet: &PathbuilderSheet,
    reference: Option<&PcReference>,
    retrieval: Option<&AtlasRetrievalService>,
) -> AppServiceResult<CharacterSheetView> {
    let abilities: AbilityKeys = reference
        .into_iter()
        .flat_map(|reference| &reference.abilities)
        .map(|ability| {
            (
                ability_identity(ability.kind, &ability.label, ability.choice.as_deref()),
                ability.key.as_str(),
            )
        })
        .collect();
    Ok(CharacterSheetView {
        name: sheet.name.trim().to_owned(),
        level: sheet.level,
        sections: vec![
            section(SheetSectionKind::Stats, stats::groups(sheet)),
            section(
                SheetSectionKind::Progression,
                progression(sheet, &abilities),
            ),
            section(SheetSectionKind::Magic, magic::groups(sheet, &abilities)),
            section(
                SheetSectionKind::Inventory,
                inventory::groups(sheet, retrieval)?,
            ),
            section(SheetSectionKind::Companions, companions(sheet)),
        ],
    })
}

type AbilityKeys<'a> = HashMap<(PcAbilityKind, String, String), &'a str>;

fn ability_identity(
    kind: PcAbilityKind,
    name: &str,
    choice: Option<&str>,
) -> (PcAbilityKind, String, String) {
    (
        kind,
        normalize_record_name(name),
        choice.map(normalize_record_name).unwrap_or_default(),
    )
}

fn ability_row(
    abilities: &AbilityKeys<'_>,
    kind: PcAbilityKind,
    name: &str,
    choice: Option<&str>,
    facts: Vec<SheetFactView>,
) -> SheetRowView {
    let mut row = row(name, facts);
    row.ability_key = abilities
        .get(&ability_identity(kind, name, choice))
        .map(|key| (*key).to_owned());
    row
}

fn section(kind: SheetSectionKind, groups: Vec<SheetGroupView>) -> SheetSectionView {
    SheetSectionView {
        kind,
        groups: groups
            .into_iter()
            .filter(|group| !group.rows.is_empty())
            .collect(),
    }
}

fn group(label: impl Into<String>, rows: Vec<SheetRowView>) -> SheetGroupView {
    SheetGroupView {
        label: label.into(),
        container_id: None,
        rows,
    }
}

fn summary(label: &str, facts: Vec<SheetFactView>) -> SheetGroupView {
    let rows = facts
        .into_iter()
        .map(|fact| compact_row(fact.label, Some(fact.value), Vec::new()));
    group(label, rows.collect())
}

fn compact_row(
    label: impl Into<String>,
    summary_value: Option<String>,
    facts: Vec<SheetFactView>,
) -> SheetRowView {
    let mut row = row(label, facts);
    row.summary_value = summary_value;
    row
}

fn row(label: impl Into<String>, facts: Vec<SheetFactView>) -> SheetRowView {
    SheetRowView {
        label: label.into(),
        summary_value: None,
        facts,
        ability_key: None,
        record_key: None,
    }
}

fn fact_value(label: impl Into<String>, value: impl Display) -> SheetFactView {
    SheetFactView {
        label: label.into(),
        value: value.to_string(),
    }
}

fn value<T>(field: &ImportedField<T>) -> Option<&T> {
    match field {
        ImportedField::Value(value) => Some(value),
        _ => None,
    }
}

fn values<T>(field: &ImportedField<Vec<ImportedField<T>>>) -> impl Iterator<Item = &T> {
    value(field).into_iter().flatten().filter_map(value)
}

fn name<'a>(field: &'a ImportedField<String>, fallback: &'a str) -> &'a str {
    value(field)
        .map(String::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback)
}

fn text_list(list: &ImportedField<Vec<ImportedField<String>>>) -> Option<String> {
    let names: Vec<_> = values(list).map(String::as_str).collect();
    (!names.is_empty()).then(|| names.join(", "))
}

fn add_facts<const N: usize>(facts: &mut Vec<SheetFactView>, entries: [(&str, Option<String>); N]) {
    facts.extend(entries.into_iter().filter_map(|(label, value)| {
        value
            .filter(|text| !text.is_empty())
            .map(|value| fact_value(label, value))
    }));
}

fn shown(field: &ImportedField<impl Display>) -> Option<String> {
    value(field).map(ToString::to_string)
}

fn source_label(field: &ImportedField<String>, label: fn(&str) -> &str) -> Option<String> {
    value(field).map(|value| label(value).to_owned())
}

fn title_case(field: &ImportedField<String>) -> Option<String> {
    value(field).map(|value| sentence_case(value))
}

fn sentence_case(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

fn feat_selection(label: &str) -> &str {
    match label {
        "childChoice" => "Child choice",
        "parentChoice" => "Parent choice",
        "standardChoice" => "Standard choice",
        label => label,
    }
}

fn yes_no(field: &ImportedField<bool>) -> Option<String> {
    value(field).map(|flag| if *flag { "Yes" } else { "No" }.into())
}

fn signed(field: &ImportedField<i64>) -> Option<String> {
    value(field).map(|number| format!("{number:+}"))
}

fn rank(rank: i64) -> String {
    match rank {
        0 => "Untrained".into(),
        2 => "Trained".into(),
        4 => "Expert".into(),
        6 => "Master".into(),
        8 => "Legendary".into(),
        other => format!("Imported rank {other}"),
    }
}

fn progression(sheet: &PathbuilderSheet, keys: &AbilityKeys<'_>) -> Vec<SheetGroupView> {
    let feats = values(&sheet.feats).map(|feat| {
        let mut facts = vec![
            fact_value("Type", &feat.category),
            fact_value("Acquired at level", feat.level),
        ];
        add_facts(
            &mut facts,
            [
                ("Choice", feat.choice.clone()),
                ("Slot", shown(&feat.slot)),
                (
                    "Selection",
                    source_label(&feat.selection_kind, feat_selection),
                ),
                ("Selection detail", shown(&feat.selection_detail)),
            ],
        );
        let kind = if feat.category == "Heritage" {
            PcAbilityKind::Heritage
        } else {
            PcAbilityKind::Feat
        };
        ability_row(keys, kind, &feat.name, feat.choice.as_deref(), facts)
    });
    let specials = values(&sheet.specials)
        .map(|name| ability_row(keys, PcAbilityKind::Special, name, None, Vec::new()));
    let mut groups = vec![
        group("Feats", feats.collect()),
        group("Special abilities", specials.collect()),
    ];
    if let Some(boosts) = value(&sheet.abilities).and_then(|abilities| value(&abilities.breakdown))
    {
        let mut facts = Vec::new();
        add_facts(
            &mut facts,
            [
                ("Free ancestry boosts", text_list(&boosts.ancestry_free)),
                ("Ancestry boosts", text_list(&boosts.ancestry_boosts)),
                ("Ancestry flaws", text_list(&boosts.ancestry_flaws)),
                ("Background boosts", text_list(&boosts.background_boosts)),
                ("Class boosts", text_list(&boosts.class_boosts)),
            ],
        );
        for (level, boosts) in value(&boosts.map_levelled_boosts).into_iter().flatten() {
            if let Some(boosts) = text_list(boosts) {
                facts.push(fact_value(format!("Level {level} boosts"), boosts));
            }
        }
        groups.push(summary("Ability boosts", facts));
    }
    groups
}

fn companions(sheet: &PathbuilderSheet) -> Vec<SheetGroupView> {
    let rows = values(&sheet.pets).map(|companion| {
        let mut facts = Vec::new();
        add_facts(
            &mut facts,
            [
                ("Type", shown(&companion.kind)),
                ("Animal", shown(&companion.animal)),
                ("Mature", yes_no(&companion.mature)),
                ("Incredible", yes_no(&companion.incredible)),
                ("Incredible type", shown(&companion.incredible_type)),
                ("Specializations", text_list(&companion.specializations)),
                ("Armor", shown(&companion.armor)),
            ],
        );
        row(name(&companion.name, "Unnamed companion"), facts)
    });
    vec![group("Companions", rows.collect())]
}
