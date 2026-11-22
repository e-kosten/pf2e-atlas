mod details;
mod inventory;
mod magic;
mod progression;
mod source_details;
mod stats;

use crate::{
    AppServiceResult, AtlasAppService,
    pathbuilder_sheet::{ImportedField, PathbuilderSheet, read_sheet},
};
use atlas_app_model::{
    CharacterSheetResponse, CharacterSheetView, SheetFactRole, SheetFactView, SheetGroupKind,
    SheetGroupView, SheetRowKind, SheetRowView, SheetSectionKind, SheetSectionView,
};
use atlas_domain::{RecordKey, SourceRecordSummary, normalize_record_name};
use atlas_local_state::{PcAbilityKind, PcReference};
use atlas_search::{AtlasRetrievalService, ResolveRecordRequest};
use std::{collections::HashMap, fmt::Display};

pub(super) use details::compare_imports;

/// The same sheet presenters, selected by semantic category rather than visible labels.
pub(crate) fn context_groups(
    sheet: &PathbuilderSheet,
    reference: Option<&PcReference>,
    retrieval: Option<&AtlasRetrievalService>,
    selected: &[atlas_app_model::CharacterSection],
) -> AppServiceResult<
    std::collections::BTreeMap<atlas_app_model::CharacterSection, Vec<SheetGroupView>>,
> {
    use atlas_app_model::CharacterSection as S;
    let mut result: std::collections::BTreeMap<_, Vec<_>> = std::collections::BTreeMap::new();
    let keys: AbilityKeys = reference
        .into_iter()
        .flat_map(|r| &r.abilities)
        .map(|a| {
            (
                ability_identity(a.kind, &a.label, a.choice.as_deref()),
                a.key.as_str(),
            )
        })
        .collect();
    if selected
        .iter()
        .any(|s| matches!(s, S::Identity | S::Statistics))
    {
        for (section, group) in stats::categorized(sheet, retrieval)? {
            if selected.contains(&section) {
                result.entry(section).or_default().push(group);
            }
        }
    }
    if selected
        .iter()
        .any(|s| matches!(s, S::Equipment | S::Money | S::Crafting))
    {
        for (section, group) in inventory::categorized(sheet, retrieval)? {
            if selected.contains(&section) {
                result.entry(section).or_default().push(group);
            }
        }
    }
    for section in selected {
        let groups = match section {
            S::Proficiencies => stats::proficiencies(sheet),
            S::CharacterOptions => progression::groups(sheet, &keys),
            S::Spellcasting => magic::groups(sheet, &keys),
            S::Companions => companions(sheet),
            _ => continue,
        };
        result.insert(*section, groups);
    }
    Ok(result)
}

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
                        project_sheet(
                            source.json(),
                            &read_sheet(source.json())?,
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
    source: &str,
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
        has_additional_details: source_details::has_source_details(source)?,
        sections: vec![
            section(SheetSectionKind::Stats, stats::groups(sheet, retrieval)?),
            section(
                SheetSectionKind::Progression,
                progression::groups(sheet, &abilities),
            ),
            section(SheetSectionKind::Magic, magic::groups(sheet, &abilities)),
            section(
                SheetSectionKind::Inventory,
                inventory::groups(sheet, retrieval)?,
            ),
            section(SheetSectionKind::Companions, companions(sheet)),
            section(SheetSectionKind::Proficiencies, stats::proficiencies(sheet)),
        ],
    })
}

fn unique_pf2e_record(
    retrieval: &AtlasRetrievalService,
    query: &str,
    eligible: impl Fn(&SourceRecordSummary) -> bool,
) -> AppServiceResult<Option<RecordKey>> {
    let mut matches = retrieval
        .resolve_record(ResolveRecordRequest {
            query,
            filter: None,
        })?
        .into_iter()
        .filter(|result| retrieval.is_pf2e_record(&result.record.key) && eligible(&result.record));
    let first = matches.next();
    Ok(first
        .filter(|_| matches.next().is_none())
        .map(|result| result.record.key))
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
            .filter(|group| !group.rows.is_empty() || !group.spell_ranks.is_empty())
            .collect(),
    }
}

fn group(label: impl Into<String>, rows: Vec<SheetRowView>) -> SheetGroupView {
    SheetGroupView {
        label: label.into(),
        kind: None,
        container_id: None,
        rows,
        spell_ranks: Vec::new(),
    }
}

fn typed_group(
    kind: SheetGroupKind,
    label: impl Into<String>,
    rows: Vec<SheetRowView>,
) -> SheetGroupView {
    let mut group = group(label, rows);
    group.kind = Some(kind);
    group
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
        kind: None,
        summary_value: None,
        summary_help: None,
        facts,
        ability_key: None,
        record_key: None,
    }
}

fn fact_value(label: impl Into<String>, value: impl Display) -> SheetFactView {
    SheetFactView {
        label: label.into(),
        value: value.to_string(),
        display_value: None,
        role: None,
    }
}

fn role_fact(role: SheetFactRole, label: impl Into<String>, value: impl Display) -> SheetFactView {
    let mut fact = fact_value(label, value);
    fact.role = Some(role);
    fact
}

fn add_role_facts<const N: usize>(
    facts: &mut Vec<SheetFactView>,
    entries: [(SheetFactRole, &str, Option<String>); N],
) {
    facts.extend(entries.into_iter().filter_map(|(role, label, value)| {
        value
            .filter(|text| !text.is_empty())
            .map(|value| role_fact(role, label, value))
    }));
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

fn companions(sheet: &PathbuilderSheet) -> Vec<SheetGroupView> {
    let rows = values(&sheet.pets).filter_map(|companion| {
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
        facts.retain(|fact| !fact.value.trim().is_empty());
        let name = value(&companion.name).filter(|name| !name.trim().is_empty());
        if name.is_none() && facts.is_empty() {
            return None;
        }
        Some(row(
            name.map(String::as_str).unwrap_or("Unnamed companion"),
            facts,
        ))
    });
    vec![group("Companions", rows.collect())]
}
