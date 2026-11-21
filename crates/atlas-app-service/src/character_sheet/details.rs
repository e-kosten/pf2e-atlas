use super::*;
use crate::{AppServiceError, pathbuilder};
use atlas_app_model::{
    ImportChangesView, ImportCollectionKind, ImportDetailsView, ImportFieldView,
    ImportSectionChangesView,
};
use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor},
};
use serde_json::value::RawValue;
use std::collections::{BTreeMap, HashSet};

impl AtlasAppService {
    pub fn character_import_details(
        &self,
        key: &str,
        expected_revision: i64,
    ) -> AppServiceResult<ImportDetailsView> {
        self.with_state(|session| {
            let characters = session.local_state()?.characters();
            characters.get_at_revision(key, expected_revision)?;
            let fields = characters
                .pathbuilder_source(key)?
                .map(|source| source_fields(source.json()))
                .transpose()?
                .unwrap_or_default();
            Ok(ImportDetailsView {
                fields,
                previous_fields: None,
            })
        })
    }

    pub fn preview_import_details(
        &self,
        json: &str,
        character_key: Option<&str>,
        expected_revision: Option<i64>,
    ) -> AppServiceResult<ImportDetailsView> {
        pathbuilder::preview(json)?;
        let fields = source_fields(json)?;
        let previous_fields = if let Some(key) = character_key {
            self.with_state(|session| {
                let characters = session.local_state()?.characters();
                let revision = expected_revision.ok_or_else(|| {
                    AppServiceError::invalid_request(
                        "Review the import again to see the previous build.",
                    )
                })?;
                characters.get_at_revision(key, revision)?;
                characters
                    .pathbuilder_source(key)?
                    .map(|source| source_fields(source.json()))
                    .transpose()
            })?
        } else {
            None
        };
        Ok(ImportDetailsView {
            fields,
            previous_fields,
        })
    }
}

struct ObjectFields(Vec<(String, Box<RawValue>)>);

impl<'de> Deserialize<'de> for ObjectFields {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldsVisitor;

        impl<'de> Visitor<'de> for FieldsVisitor {
            type Value = ObjectFields;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a Pathbuilder source object")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut fields = Vec::new();
                while let Some(field) = map.next_entry()? {
                    fields.push(field);
                }
                Ok(ObjectFields(fields))
            }
        }
        deserializer.deserialize_map(FieldsVisitor)
    }
}

fn source_fields(source: &str) -> AppServiceResult<Vec<ImportFieldView>> {
    object_fields(source).map_err(|_| {
        AppServiceError::invalid_request("Could not read the Pathbuilder import details.")
    })
}

fn object_fields(source: &str) -> serde_json::Result<Vec<ImportFieldView>> {
    serde_json::from_str::<ObjectFields>(source)?
        .0
        .into_iter()
        .map(|(label, raw)| source_field(label, raw.get()))
        .collect()
}

fn source_field(label: String, raw: &str) -> serde_json::Result<ImportFieldView> {
    let (value, collection, children) = if raw.starts_with('{') {
        (
            None,
            Some(ImportCollectionKind::Object),
            object_fields(raw)?,
        )
    } else if raw.starts_with('[') {
        let items = serde_json::from_str::<Vec<Box<RawValue>>>(raw)?;
        let children = items
            .into_iter()
            .enumerate()
            .map(|(index, item)| source_field(format!("Item {}", index + 1), item.get()))
            .collect::<serde_json::Result<Vec<_>>>()?;
        (None, Some(ImportCollectionKind::Array), children)
    } else {
        (Some(raw.to_owned()), None, Vec::new())
    };
    Ok(ImportFieldView {
        label,
        value,
        collection,
        children,
    })
}

pub(crate) fn compare_imports(
    incoming_json: &str,
    incoming: &CharacterSheetView,
    reference: &PcReference,
    previous_json: Option<&str>,
    previous_reference: Option<&PcReference>,
) -> AppServiceResult<ImportChangesView> {
    let before = previous_json
        .map(|raw| project_sheet(&read_sheet(raw)?, previous_reference, None))
        .transpose()?;
    let mut sections = Vec::new();
    if let Some(before) = &before {
        for (previous, incoming) in before.sections.iter().zip(&incoming.sections) {
            let label = match incoming.kind {
                SheetSectionKind::Stats => "Stats",
                SheetSectionKind::Progression => "Progression",
                SheetSectionKind::Magic => "Magic",
                SheetSectionKind::Inventory => "Inventory",
                SheetSectionKind::Companions => "Companions",
            };
            insert_changes(&mut sections, label, compare_section(previous, incoming));
        }
    }
    if let Some(previous) = previous_reference {
        let prior = previous
            .abilities
            .iter()
            .map(|ability| ability.key.as_str())
            .collect::<HashSet<_>>();
        let next = reference
            .abilities
            .iter()
            .map(|ability| ability.key.as_str())
            .collect::<HashSet<_>>();
        let describe = |ability: &atlas_local_state::PcAbility| {
            ability.choice.as_ref().map_or_else(
                || ability.label.clone(),
                |choice| format!("{} ({choice})", ability.label),
            )
        };
        let removed = previous
            .abilities
            .iter()
            .filter(|ability| !next.contains(ability.key.as_str()))
            .map(describe)
            .collect();
        let added = reference
            .abilities
            .iter()
            .filter(|ability| !prior.contains(ability.key.as_str()))
            .map(describe)
            .collect();
        insert_changes(
            &mut sections,
            "Rules reference",
            ImportSectionChangesView {
                added,
                removed,
                ..Default::default()
            },
        );
    }
    let mut removed_corrections = Vec::new();
    let mut retained_corrections = 0;
    let current: HashSet<_> = reference
        .abilities
        .iter()
        .map(|ability| ability.key.as_str())
        .collect();
    for ability in previous_reference
        .into_iter()
        .flat_map(|reference| &reference.abilities)
    {
        if ability.manual.is_some() {
            if current.contains(ability.key.as_str()) {
                retained_corrections += 1;
            } else {
                removed_corrections.push(ability.choice.as_ref().map_or_else(
                    || ability.label.clone(),
                    |choice| format!("{} ({choice})", ability.label),
                ));
            }
        }
    }
    Ok(ImportChangesView {
        had_sheet: before.is_some(),
        sections,
        removed_corrections,
        retained_corrections,
        additional_fields_changed: previous_json
            .map(|previous| changed_source_fields(previous, incoming_json))
            .transpose()?
            .unwrap_or_default(),
    })
}

fn insert_changes(
    sections: &mut Vec<ImportSectionChangesView>,
    label: &str,
    mut changes: ImportSectionChangesView,
) {
    if !(changes.added.is_empty() && changes.changed.is_empty() && changes.removed.is_empty()) {
        changes.label = label.to_owned();
        sections.push(changes);
    }
}

fn compare_section(
    previous: &SheetSectionView,
    incoming: &SheetSectionView,
) -> ImportSectionChangesView {
    type RowFacts<'a> = (Option<&'a str>, &'a [SheetFactView]);
    type Rows<'a> = BTreeMap<(Option<&'a str>, &'a str, &'a str), (&'a str, Vec<RowFacts<'a>>)>;
    fn rows(section: &SheetSectionView) -> Rows<'_> {
        let mut rows = Rows::new();
        for group in &section.groups {
            for row in &group.rows {
                let container = group.container_id.as_deref();
                let group_key = if container.is_some() {
                    ""
                } else {
                    &group.label
                };
                rows.entry((container, group_key, &row.label))
                    .or_insert_with(|| (&group.label, Vec::new()))
                    .1
                    .push((row.summary_value.as_deref(), &row.facts));
            }
        }
        rows
    }
    let mut before = rows(previous);
    let mut changes = ImportSectionChangesView::default();
    let describe = |group: &str, label: &str| format!("{group} · {label}");
    for ((container, group_key, label), (group, incoming)) in rows(incoming) {
        let (old_group, previous) = before
            .remove(&(container, group_key, label))
            .unwrap_or((group, Vec::new()));
        let mut unchanged = HashMap::new();
        for facts in &previous {
            *unchanged.entry(*facts).or_insert(0usize) += 1;
        }
        let mut matched = 0;
        for facts in &incoming {
            if let Some(count) = unchanged.get_mut(facts).filter(|count| **count > 0) {
                *count -= 1;
                matched += 1;
            }
        }
        let old_count = previous.len() - matched;
        let new_count = incoming.len() - matched;
        let changed = old_count.min(new_count);
        changes
            .changed
            .extend((0..changed).map(|_| describe(group, label)));
        changes
            .removed
            .extend((changed..old_count).map(|_| describe(old_group, label)));
        changes
            .added
            .extend((changed..new_count).map(|_| describe(group, label)));
    }
    for ((_, _, label), (group, rows)) in before {
        changes
            .removed
            .extend(rows.into_iter().map(|_| describe(group, label)));
    }
    changes
}

fn changed_source_fields(previous: &str, incoming: &str) -> AppServiceResult<Vec<String>> {
    fn collect(fields: Vec<ImportFieldView>) -> BTreeMap<String, Vec<ImportFieldView>> {
        let mut by_label = BTreeMap::<String, Vec<ImportFieldView>>::new();
        for mut field in fields {
            if field.label == "build" && field.collection == Some(ImportCollectionKind::Object) {
                for mut child in field.children {
                    canonicalize(&mut child);
                    by_label
                        .entry(format!("Build · {}", child.label))
                        .or_default()
                        .push(child);
                }
            } else {
                canonicalize(&mut field);
                by_label.entry(field.label.clone()).or_default().push(field);
            }
        }
        by_label
    }
    fn canonicalize(field: &mut ImportFieldView) {
        for child in &mut field.children {
            canonicalize(child);
        }
        if field.collection == Some(ImportCollectionKind::Object) {
            field.children.sort_by(|a, b| a.label.cmp(&b.label));
        }
    }
    let mut previous = collect(source_fields(previous)?);
    let incoming = collect(source_fields(incoming)?);
    let mut changed = Vec::new();
    for (label, incoming) in incoming {
        if previous.remove(&label).as_ref() != Some(&incoming) {
            changed.push(label);
        }
    }
    changed.extend(previous.into_keys());
    changed.sort();
    Ok(changed)
}
