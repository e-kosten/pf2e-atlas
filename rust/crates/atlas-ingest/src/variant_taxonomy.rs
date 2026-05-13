use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use crate::normalize::normalize_text;
use crate::references::{reference_pack_and_locator, resolve_record_key};
use crate::{FolderDefinition, IngestDiagnostics, LoadedPack, LoadedRecord, RecordReferenceIndex};

pub(super) fn assign_taxonomy_families(
    records: &mut [LoadedRecord],
    packs: &[LoadedPack],
    index: &RecordReferenceIndex,
    diagnostics: &mut IngestDiagnostics,
) {
    let folder_families = load_folder_family_maps(packs);
    let glossary_families = build_glossary_family_map(records);

    for record in records {
        let mut families = BTreeSet::new();
        let mut assigned_from_folder = false;
        let mut assigned_from_glossary = false;
        if let Some(folder_id) = &record.folder_id
            && let Some(folder_family) =
                folder_families.get(&(record.pack_name.as_str().to_string(), folder_id.clone()))
            && should_keep_folder_family(record.pack_name.as_str(), folder_family)
        {
            families.insert(folder_family.clone());
            assigned_from_folder = true;
        }

        for candidate in &record.reference_candidates {
            let Some((pack_name, locator)) = reference_pack_and_locator(&candidate.raw_target)
            else {
                continue;
            };
            if pack_name != "bestiary-family-ability-glossary" {
                continue;
            }
            let Some(record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
                continue;
            };
            if let Some(family) = glossary_families.get(&record_key.to_string()) {
                families.insert(family.clone());
                assigned_from_glossary = true;
            }
        }

        record.taxonomy_families = families.into_iter().collect();
        diagnostics.taxonomy_folder_records += usize::from(assigned_from_folder);
        diagnostics.taxonomy_glossary_records += usize::from(assigned_from_glossary);
    }
}

fn load_folder_family_maps(packs: &[LoadedPack]) -> BTreeMap<(String, String), String> {
    let mut families = BTreeMap::new();
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
            if let Some(family) = resolve_folder_family(folder_id, &folders_by_id) {
                families.insert((pack.name.as_str().to_string(), folder_id.clone()), family);
            }
        }
    }
    families
}

fn resolve_folder_family(
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
            return normalize_family_name(folder.name.as_deref().unwrap_or_default());
        }
    }
    current.and_then(|folder| normalize_family_name(folder.name.as_deref().unwrap_or_default()))
}

fn should_keep_folder_family(pack_name: &str, family: &str) -> bool {
    const NPC_CORE_FAMILY_ALLOWLIST: &[&str] = &[
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
    pack_name == "pathfinder-npc-core" && NPC_CORE_FAMILY_ALLOWLIST.contains(&family)
}

fn build_glossary_family_map(records: &[LoadedRecord]) -> BTreeMap<String, String> {
    let mut families = BTreeMap::new();
    for record in records {
        if record.pack_name.as_str() != "bestiary-family-ability-glossary" {
            continue;
        }
        if let Some(family) = derive_glossary_family_from_source_path(&record.source_path) {
            families.insert(record.key.to_string(), family);
        }
    }
    families
}

fn derive_glossary_family_from_source_path(source_path: &str) -> Option<String> {
    let mut segments = source_path.split('/').filter(|segment| !segment.is_empty());
    while let Some(segment) = segments.next() {
        if segment == "bestiary-family-ability-glossary" {
            return segments.next().and_then(normalize_family_name);
        }
    }
    None
}

fn normalize_family_name(value: &str) -> Option<String> {
    let family = normalize_text(value).replace(' ', "-");
    (!family.is_empty()).then_some(family)
}
