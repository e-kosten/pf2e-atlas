use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordFamily;

mod labels;

use labels::{
    infer_variant_axes, is_dragon_age_label, is_gender_label, is_gender_only, is_grade_label,
    is_rank_label, is_specialization_label, normalize_rank_label, slugify_hyphen, title_case_words,
};

use crate::normalize::normalize_text;
use crate::references::record_by_key;
use crate::{
    IngestDiagnostics, LoadedRecord, RecordReferenceIndex, VariantCandidate,
    VariantDiagnosticSource,
};

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

pub(super) fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_parenthetical_variant_labels_inside_out() {
        let candidate = parse_parenthetical_variant_candidate("Ignition (Melee) (Heightened)")
            .expect("parenthetical variant should parse");

        assert_eq!(candidate.base_name, "Ignition");
        assert_eq!(candidate.label.as_deref(), Some("Melee, Heightened"));
        assert_eq!(candidate.axes, vec!["other"]);
    }

    #[test]
    fn parses_grade_and_rank_suffixes() {
        let grade = parse_trailing_suffix_variant_candidate("Healing Potion Greater")
            .expect("grade suffix should parse");
        assert_eq!(grade.base_name, "Healing Potion");
        assert_eq!(grade.label.as_deref(), Some("Greater"));
        assert_eq!(grade.axes, vec!["grade"]);

        let rank = parse_trailing_suffix_variant_candidate("Summon Construct 4th-rank")
            .expect("rank suffix should parse");
        assert_eq!(rank.base_name, "Summon Construct");
        assert_eq!(rank.label.as_deref(), Some("4th Rank"));
        assert_eq!(rank.axes, vec!["rank"]);
    }

    #[test]
    fn creature_group_keys_ignore_pack_name() {
        assert_eq!(
            variant_group_key_for_parts(RecordFamily::Creature, "any-pack", "Young Red Dragon"),
            "creature:family:young-red-dragon"
        );
        assert_eq!(
            variant_group_key_for_parts(RecordFamily::Spell, "spells", "Ignition"),
            "spell:spells:ignition"
        );
    }
}
