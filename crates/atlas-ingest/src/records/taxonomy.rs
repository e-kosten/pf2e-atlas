use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use atlas_record::{ContentReferenceLocator, iter_content_references};

use crate::diagnostics::FolderDefinition;
use crate::diagnostics::IngestDiagnostics;
use crate::records::references::{reference_pack_and_locator, resolve_record_key};
use crate::records::{LoadedSourceRecord, RecordReferenceIndex};
use crate::source::LoadedPack;
use crate::source::normalize::normalize_text;

pub(crate) fn assign_inferred_taxonomy_groups(
    records: &mut [LoadedSourceRecord],
    packs: &[LoadedPack],
    index: &RecordReferenceIndex,
    diagnostics: &mut IngestDiagnostics,
) {
    let folder_groups = load_folder_group_maps(packs);
    let glossary_groups = build_glossary_group_map(records);

    for loaded in records {
        let glossary_targets = glossary_reference_targets(loaded);
        let record = &mut loaded.record;
        let mut groups = BTreeSet::new();
        let mut assigned_from_folder = false;
        let mut assigned_from_glossary = false;
        if let Some(folder_id) = &record.foundry.folder_id
            && let Some(folder_group) = folder_groups.get(&(
                record.identity.pack().as_str().to_string(),
                folder_id.clone(),
            ))
            && should_keep_folder_group(record.identity.pack().as_str(), folder_group)
        {
            groups.insert(folder_group.clone());
            assigned_from_folder = true;
        }

        for (pack_name, locator) in glossary_targets {
            if pack_name != "bestiary-family-ability-glossary" {
                continue;
            }
            let Some(record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
                continue;
            };
            if let Some(group) = glossary_groups.get(&record_key.to_string()) {
                groups.insert(group.clone());
                assigned_from_glossary = true;
            }
        }

        record.classification.taxonomy.inferred_groups = groups.into_iter().collect();
        diagnostics.taxonomy_folder_records += usize::from(assigned_from_folder);
        diagnostics.taxonomy_glossary_records += usize::from(assigned_from_glossary);
    }
}

fn glossary_reference_targets(loaded: &LoadedSourceRecord) -> Vec<(String, String)> {
    let mut targets = Vec::new();
    let record = &loaded.record;
    if let Some(document) = record.content.description() {
        collect_glossary_targets(document, &mut targets);
    }
    if let Some(document) = record.content.blurb() {
        collect_glossary_targets(document, &mut targets);
    }
    for supplemental in record.content.reference_documents() {
        collect_glossary_targets(&supplemental.document, &mut targets);
    }
    targets
}

fn collect_glossary_targets(
    document: &atlas_record::ContentDocument,
    targets: &mut Vec<(String, String)>,
) {
    for reference in iter_content_references(document) {
        match &reference.locator {
            ContentReferenceLocator::FoundryUuid { raw_target }
            | ContentReferenceLocator::Compendium { raw_target } => {
                if let Some(target) = reference_pack_and_locator(raw_target) {
                    targets.push(target);
                }
            }
            ContentReferenceLocator::PackAndLocator { pack_name, locator } => {
                targets.push((pack_name.clone(), locator.clone()));
            }
            ContentReferenceLocator::Unknown { .. } => {}
        }
    }
}

fn load_folder_group_maps(packs: &[LoadedPack]) -> BTreeMap<(String, String), String> {
    let mut groups = BTreeMap::new();
    for pack in packs {
        let path = pack.resolved_path.join("_folders.json");
        let Ok(serialized) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(raw_folders) = serde_json::from_str::<Vec<FolderDefinition>>(&serialized) else {
            continue;
        };
        let folders_by_id = raw_folders
            .into_iter()
            .filter_map(|folder| folder.id.clone().map(|id| (id, folder)))
            .collect::<BTreeMap<_, _>>();
        for folder_id in folders_by_id.keys() {
            if let Some(group) = resolve_folder_group(folder_id, &folders_by_id) {
                groups.insert((pack.name.as_str().to_string(), folder_id.clone()), group);
            }
        }
    }
    groups
}

fn resolve_folder_group(
    folder_id: &str,
    folders_by_id: &BTreeMap<String, FolderDefinition>,
) -> Option<String> {
    let mut visited = BTreeSet::new();
    let mut current_id = Some(folder_id.to_string());
    let mut current = None;
    while let Some(id) = current_id {
        if !visited.insert(id.clone()) {
            return None;
        }
        let folder = folders_by_id.get(&id)?;
        current = Some(folder);
        if let Some(parent_id) = folder
            .folder
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            current_id = Some(parent_id.clone());
        } else {
            return normalize_group_name(folder.name.as_deref().unwrap_or_default());
        }
    }
    current.and_then(|folder| normalize_group_name(folder.name.as_deref().unwrap_or_default()))
}

fn should_keep_folder_group(pack_name: &str, group: &str) -> bool {
    const NPC_CORE_GROUP_ALLOWLIST: &[&str] = &[
        "ancestry-npcs",
        "artisan",
        "courtier",
        "criminal",
        "devotee",
        "downtrodden",
        "engineer",
        "explorer",
        "healer",
        "laborer",
        "martial-artist",
        "maverick",
        "mercenary",
        "military",
        "mystic",
        "official",
        "performer",
        "primalist",
        "scholar",
        "seafarer",
        "villain",
    ];
    pack_name == "pathfinder-npc-core" && NPC_CORE_GROUP_ALLOWLIST.contains(&group)
}

fn build_glossary_group_map(records: &[LoadedSourceRecord]) -> BTreeMap<String, String> {
    let mut groups = BTreeMap::new();
    for loaded in records {
        let record = &loaded.record;
        if record.identity.pack().as_str() != "bestiary-family-ability-glossary" {
            continue;
        }
        if let Some(group) = derive_glossary_group_from_source_path(&record.provenance.source_path)
        {
            groups.insert(record.identity.key.to_string(), group);
        }
    }
    groups
}

fn derive_glossary_group_from_source_path(source_path: &str) -> Option<String> {
    let mut segments = source_path.split('/').filter(|segment| !segment.is_empty());
    while let Some(segment) = segments.next() {
        if segment == "bestiary-family-ability-glossary" {
            return segments.next().and_then(normalize_group_name);
        }
    }
    None
}

fn normalize_group_name(value: &str) -> Option<String> {
    let group = normalize_text(value).replace(' ', "-");
    (!group.is_empty()).then_some(group)
}
