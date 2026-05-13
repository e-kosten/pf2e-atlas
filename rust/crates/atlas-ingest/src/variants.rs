use super::*;

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

pub(super) fn assign_variant_groups(
    records: &mut [LoadedRecord],
    index: &RecordReferenceIndex,
    diagnostics: &mut IngestDiagnostics,
) {
    let mut candidates_by_group = BTreeMap::<String, Vec<(usize, VariantCandidate)>>::new();
    let mut base_names_by_group = BTreeMap::<String, String>::new();
    let known_creature_base_names = known_creature_variant_base_names(records);
    for (index_in_records, record) in records.iter().enumerate() {
        let Some(candidate) = variant_candidate(record, index, &known_creature_base_names) else {
            continue;
        };
        let group_key = variant_group_key(record, &candidate.base_name);
        base_names_by_group.insert(group_key.clone(), candidate.base_name.clone());
        candidates_by_group
            .entry(group_key)
            .or_default()
            .push((index_in_records, candidate));
    }

    let mut assigned_indices = BTreeSet::new();
    for (group_key, mut members) in candidates_by_group {
        let Some(base_name) = base_names_by_group.get(&group_key) else {
            continue;
        };
        if let Some(base_index) = exact_base_index(records, &group_key, base_name)
            && !members
                .iter()
                .any(|(member_index, _candidate)| *member_index == base_index)
        {
            members.push((
                base_index,
                VariantCandidate {
                    base_name: base_name.clone(),
                    label: None,
                    axes: Vec::new(),
                    source: "composite",
                    diagnostic_source: VariantDiagnosticSource::ExactBase,
                    confidence: 0.62,
                },
            ));
        }
        members.sort_by_key(|(member_index, _candidate)| *member_index);
        members.dedup_by_key(|(member_index, _candidate)| *member_index);
        if members.len() < 2
            || !members.iter().any(|(_index, candidate)| {
                candidate
                    .label
                    .as_deref()
                    .is_some_and(|label| !label.is_empty())
            })
        {
            continue;
        }
        if members
            .iter()
            .any(|(member_index, _candidate)| assigned_indices.contains(member_index))
        {
            continue;
        }

        let axes = sorted_unique(
            members
                .iter()
                .flat_map(|(_index, candidate)| candidate.axes.clone())
                .collect(),
        );
        let source = if members
            .iter()
            .any(|(_index, candidate)| candidate.source == "composite")
        {
            "composite"
        } else {
            members[0].1.source
        };
        let confidence = members
            .iter()
            .map(|(_index, candidate)| candidate.confidence)
            .fold(0.0_f64, f64::max);

        for (member_index, candidate) in members {
            match candidate.diagnostic_source {
                VariantDiagnosticSource::Parenthetical => {
                    diagnostics.variant_parenthetical_records += 1;
                }
                VariantDiagnosticSource::Suffix => {
                    diagnostics.variant_suffix_records += 1;
                }
                VariantDiagnosticSource::CreatureBlurb => {
                    diagnostics.variant_creature_blurb_records += 1;
                }
                VariantDiagnosticSource::CreatureSuffix => {
                    diagnostics.variant_creature_suffix_records += 1;
                }
                VariantDiagnosticSource::ExactBase => {
                    diagnostics.variant_exact_base_records += 1;
                }
            }
            let record = &mut records[member_index];
            record.variant_group_key = Some(group_key.clone());
            record.variant_base_name = Some(base_name.clone());
            record.variant_label = candidate.label;
            record.variant_axes = axes.clone();
            record.variant_confidence = Some(confidence);
            record.variant_source = source.to_string();
            assigned_indices.insert(member_index);
        }
    }
}

fn variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
    known_creature_base_names: &BTreeSet<String>,
) -> Option<VariantCandidate> {
    match record.record_family {
        RecordFamily::Creature => {
            parse_creature_variant_candidate(record, index, known_creature_base_names)
                .or_else(|| parse_parenthetical_variant_candidate(&record.name))
        }
        RecordFamily::Equipment | RecordFamily::Spell => {
            parse_parenthetical_variant_candidate(&record.name)
                .or_else(|| parse_trailing_suffix_variant_candidate(&record.name))
        }
        _ => None,
    }
}

fn known_creature_variant_base_names(records: &[LoadedRecord]) -> BTreeSet<String> {
    records
        .iter()
        .filter(|record| record.record_family == RecordFamily::Creature)
        .filter_map(|record| parse_parenthetical_variant_candidate(&record.name))
        .map(|candidate| normalize_text(&candidate.base_name))
        .filter(|base_name| !base_name.is_empty())
        .collect()
}

fn parse_parenthetical_variant_candidate(name: &str) -> Option<VariantCandidate> {
    let mut remainder = name.trim().to_string();
    let mut labels = Vec::new();
    while let Some((base, label)) = split_trailing_parenthetical(&remainder) {
        if base.is_empty() || label.is_empty() {
            break;
        }
        remainder = base;
        labels.insert(0, label);
    }
    if remainder.is_empty() || labels.is_empty() {
        return None;
    }
    Some(VariantCandidate {
        base_name: remainder,
        label: Some(labels.join(", ")),
        axes: infer_variant_axes(&labels),
        source: "namePattern",
        diagnostic_source: VariantDiagnosticSource::Parenthetical,
        confidence: 0.6,
    })
}

fn split_trailing_parenthetical(value: &str) -> Option<(String, String)> {
    let value = value.trim();
    if !value.ends_with(')') {
        return None;
    }
    let open = value.rfind(" (")?;
    let base = value[..open].trim().to_string();
    let label = value[open + 2..value.len() - 1].trim().to_string();
    Some((base, label))
}

fn parse_trailing_suffix_variant_candidate(name: &str) -> Option<VariantCandidate> {
    let words = name.split_whitespace().collect::<Vec<_>>();
    let suffix = words.last()?;
    let normalized_suffix = normalize_text(suffix);
    let label = if is_grade_label(&normalized_suffix) {
        title_case_words(&normalized_suffix)
    } else if is_rank_label(&normalized_suffix) {
        normalize_rank_label(&normalized_suffix)
    } else {
        return None;
    };
    let base_name = words[..words.len() - 1].join(" ").trim().to_string();
    if base_name.is_empty() {
        return None;
    }
    let axes = infer_variant_axes(std::slice::from_ref(&label));
    Some(VariantCandidate {
        base_name,
        label: Some(label),
        axes,
        source: "namePattern",
        diagnostic_source: VariantDiagnosticSource::Suffix,
        confidence: 0.74,
    })
}

fn parse_creature_variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
    known_creature_base_names: &BTreeSet<String>,
) -> Option<VariantCandidate> {
    parse_creature_blurb_variant_candidate(record, index, known_creature_base_names)
        .or_else(|| parse_creature_suffix_variant_candidate(record, index))
}

fn parse_creature_blurb_variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
    known_creature_base_names: &BTreeSet<String>,
) -> Option<VariantCandidate> {
    let blurb = record.blurb_text.as_ref()?;
    let tokens = normalize_text(blurb)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tokens.len() < 2 || tokens.len() > 6 {
        return None;
    }
    let mut label_tokens = Vec::new();
    let mut cursor = 0;
    while let Some(token) = tokens.get(cursor) {
        if is_dragon_age_label(token) || is_specialization_label(token) || is_gender_label(token) {
            label_tokens.push(token.clone());
            cursor += 1;
        } else {
            break;
        }
    }
    if label_tokens.is_empty() {
        return None;
    }
    let base_tokens = tokens[cursor..].to_vec();
    if base_tokens.is_empty() || base_tokens.len() > 3 {
        return None;
    }
    for base_name in creature_base_name_candidates(&base_tokens) {
        let base_record = exact_creature_base_record(index, &base_name);
        if base_record.is_none() && !known_creature_base_names.contains(&normalize_text(&base_name))
        {
            continue;
        }
        if is_gender_only(&label_tokens)
            && base_record.is_some_and(|record| {
                record
                    .traits
                    .iter()
                    .any(|trait_value| trait_value == "humanoid")
            })
        {
            continue;
        }
        let cleaned_labels = label_tokens
            .iter()
            .map(|token| title_case_words(token))
            .collect::<Vec<_>>();
        let label = choose_creature_variant_label(record, &base_name, &cleaned_labels);
        return Some(VariantCandidate {
            base_name,
            label,
            axes: infer_variant_axes(&cleaned_labels),
            source: "composite",
            diagnostic_source: VariantDiagnosticSource::CreatureBlurb,
            confidence: 0.86,
        });
    }
    None
}

fn parse_creature_suffix_variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
) -> Option<VariantCandidate> {
    const ALLOWLIST: &[(&str, &str)] = &[
        ("ghost", "ghost"),
        ("ghoul", "ghoul"),
        ("wight", "wight"),
        ("wraith", "wraith"),
    ];
    let normalized_name = normalize_text(&record.name);
    for (base, required_trait) in ALLOWLIST {
        if normalized_name == *base || !normalized_name.ends_with(&format!(" {base}")) {
            continue;
        }
        if !record
            .traits
            .iter()
            .any(|trait_value| trait_value == required_trait)
        {
            continue;
        }
        let base_name = title_case_words(base);
        let Some(base_record) = exact_creature_base_record(index, &base_name) else {
            continue;
        };
        if !base_record
            .traits
            .iter()
            .any(|trait_value| trait_value == required_trait)
        {
            continue;
        }
        return Some(VariantCandidate {
            base_name,
            label: Some(record.name.clone()),
            axes: vec!["other".to_string()],
            source: "namePattern",
            diagnostic_source: VariantDiagnosticSource::CreatureSuffix,
            confidence: 0.68,
        });
    }
    None
}

fn exact_creature_base_record<'a>(
    index: &'a RecordReferenceIndex,
    base_name: &str,
) -> Option<&'a LoadedRecord> {
    let matches = index.by_name.get(&normalize_text(base_name))?;
    matches
        .iter()
        .filter_map(|key| record_by_key(index, key))
        .find(|record| record.record_family == RecordFamily::Creature)
}

fn exact_base_index(records: &[LoadedRecord], group_key: &str, base_name: &str) -> Option<usize> {
    records.iter().position(|record| {
        record.name == base_name && variant_group_key(record, base_name) == group_key
    })
}

fn variant_group_key(record: &LoadedRecord, base_name: &str) -> String {
    variant_group_key_for_parts(record.record_family, record.pack_name.as_str(), base_name)
}

fn variant_group_key_for_parts(
    record_family: RecordFamily,
    pack_name: &str,
    base_name: &str,
) -> String {
    if record_family == RecordFamily::Creature {
        format!("creature:family:{}", slugify_hyphen(base_name))
    } else {
        format!(
            "{}:{}:{}",
            record_family.as_str(),
            pack_name,
            slugify_hyphen(base_name)
        )
    }
}

fn creature_base_name_candidates(tokens: &[String]) -> Vec<String> {
    let mut candidates = Vec::new();
    let add = |values: &[String], candidates: &mut Vec<String>| {
        let candidate = title_case_words(&values.join(" "));
        if !candidate.is_empty() && !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    };
    add(tokens, &mut candidates);
    if let Some(last) = tokens.last()
        && let Some(singular) = singularize_creature_token(last)
    {
        let mut singular_tokens = tokens.to_vec();
        if let Some(last_token) = singular_tokens.last_mut() {
            *last_token = singular;
        }
        add(&singular_tokens, &mut candidates);
    }
    candidates
}

fn singularize_creature_token(token: &str) -> Option<String> {
    if token.len() <= 3 {
        return None;
    }
    if let Some(stem) = token.strip_suffix("ies") {
        return Some(format!("{stem}y"));
    }
    for suffix in ["xes", "ches", "shes", "sses", "zes"] {
        if token.ends_with(suffix) {
            return Some(token[..token.len() - 2].to_string());
        }
    }
    if token.ends_with('s') && !token.ends_with("ss") {
        return Some(token[..token.len() - 1].to_string());
    }
    None
}

fn choose_creature_variant_label(
    record: &LoadedRecord,
    base_name: &str,
    labels: &[String],
) -> Option<String> {
    let explicit = labels.join(", ");
    let normalized_name = normalize_text(&record.name);
    let normalized_base_name = normalize_text(base_name);
    if normalized_name.is_empty() || normalized_name == normalized_base_name {
        return (!explicit.is_empty()).then_some(explicit);
    }
    let generic_only = labels.iter().all(|label| {
        is_specialization_label(&normalize_text(label)) || is_gender_label(&normalize_text(label))
    });
    if generic_only {
        return Some(record.name.clone());
    }
    if normalized_name.contains(&normalized_base_name) && !explicit.is_empty() {
        return Some(explicit);
    }
    Some(record.name.clone())
}

fn infer_variant_axes(labels: &[String]) -> Vec<String> {
    let axes = labels
        .iter()
        .flat_map(|label| {
            let normalized = normalize_text(label);
            if is_rank_label(&normalized) {
                vec!["rank".to_string()]
            } else if is_grade_label(&normalized) {
                vec!["grade".to_string()]
            } else if is_damage_type_label(&normalized) {
                vec!["damageType".to_string()]
            } else if is_dragon_age_label(&normalized) {
                vec!["dragonAge".to_string()]
            } else if is_specialization_label(&normalized) {
                vec!["specialization".to_string()]
            } else if is_gender_label(&normalized) {
                Vec::new()
            } else {
                vec!["other".to_string()]
            }
        })
        .collect::<Vec<_>>();
    let axes = sorted_unique(axes);
    if axes.is_empty() {
        vec!["other".to_string()]
    } else {
        axes
    }
}

pub(super) fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn is_grade_label(value: &str) -> bool {
    matches!(
        value,
        "minor" | "lesser" | "moderate" | "greater" | "major" | "true"
    )
}

fn is_damage_type_label(value: &str) -> bool {
    matches!(
        value,
        "acid" | "cold" | "electricity" | "fire" | "poison" | "sonic" | "void" | "vitality"
    )
}

fn is_dragon_age_label(value: &str) -> bool {
    matches!(
        value,
        "wyrmling" | "hatchling" | "young" | "juvenile" | "adult" | "ancient" | "greatwyrm"
    )
}

fn is_specialization_label(value: &str) -> bool {
    matches!(value, "spellcaster" | "elite" | "weak" | "variant")
}

fn is_gender_label(value: &str) -> bool {
    matches!(value, "male" | "female")
}

fn is_gender_only(labels: &[String]) -> bool {
    !labels.is_empty() && labels.iter().all(|label| is_gender_label(label))
}

fn is_rank_label(value: &str) -> bool {
    let normalized = value.replace('-', " ");
    let mut parts = normalized.split_whitespace();
    let Some(ordinal) = parts.next() else {
        return false;
    };
    let Some(kind) = parts.next() else {
        return false;
    };
    is_ordinal(ordinal) && matches!(kind, "rank" | "level")
}

fn is_ordinal(value: &str) -> bool {
    ["st", "nd", "rd", "th"].iter().any(|suffix| {
        value
            .strip_suffix(suffix)
            .is_some_and(|prefix| prefix.parse::<u8>().is_ok())
    })
}

fn normalize_rank_label(value: &str) -> String {
    value
        .replace('-', " ")
        .split_whitespace()
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                part.to_string()
            } else {
                title_case_words(part)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slugify_hyphen(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for character in normalize_text(value).chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !output.is_empty() {
            output.push('-');
            last_was_separator = true;
        }
    }
    while output.ends_with('-') {
        output.pop();
    }
    output
}
