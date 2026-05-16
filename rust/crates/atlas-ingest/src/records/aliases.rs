use std::collections::BTreeSet;
use std::path::Path;

use atlas_domain::{RecordKey, RemasterLinkSource};
use serde_json::Value;

mod html;
mod migrations;
mod remaster_journal;

use migrations::migration_rename_pairs_from_root;
use remaster_journal::extract_remaster_journal_changes;

#[cfg(test)]
use html::html_text;
#[cfg(test)]
use migrations::migration_rename_pairs;
#[cfg(test)]
use remaster_journal::expand_grouped_alias_text;

use crate::records::references::{record_by_key, reference_pack_and_locator, resolve_record_key};
use crate::records::{
    AliasSource, LoadedSourceRecord, NormalizedRecord, RecordAlias, RecordReferenceIndex,
    RemasterLink,
};
use crate::source::normalize::{normalize_text, pointer_bool, pointer_string};

pub(crate) fn resolve_record_aliases(
    records: &[LoadedSourceRecord],
    index: &RecordReferenceIndex,
    source_root: &Path,
) -> Vec<RecordAlias> {
    let mut aliases = Vec::new();
    for loaded in records {
        let record = &loaded.record;
        if record.foundry_document_type == "JournalEntry"
            && record.normalized_name == "remaster changes"
        {
            aliases.extend(extract_remaster_journal_aliases(record, index));
        }
        aliases.extend(extract_compendium_source_aliases(record, index));
    }

    aliases.extend(extract_migration_aliases(source_root, index));
    dedupe_record_aliases(aliases)
}

fn extract_remaster_journal_aliases(
    record: &NormalizedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RecordAlias> {
    let mut aliases = Vec::new();
    for change in extract_remaster_journal_changes(record, index) {
        add_record_alias(
            &mut aliases,
            &change.remaster_record_key,
            &change.legacy_name,
            AliasSource::RemasterJournal,
            &change.source_ref,
            index,
        );
    }
    aliases
}

fn extract_migration_aliases(source_root: &Path, index: &RecordReferenceIndex) -> Vec<RecordAlias> {
    let mut aliases = Vec::new();
    for (legacy_name, remaster_name) in migration_rename_pairs_from_root(source_root) {
        let Some(remaster_record_key) = resolve_record_key(None, &remaster_name, index) else {
            continue;
        };
        add_record_alias(
            &mut aliases,
            &remaster_record_key,
            &legacy_name,
            AliasSource::Migration,
            "src/module/migration/migrations",
            index,
        );
    }
    aliases
}

fn extract_compendium_source_aliases(
    record: &NormalizedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RecordAlias> {
    let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
        return Vec::new();
    };
    let Some(items) = raw.pointer("/items").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut aliases = Vec::new();
    for item in items {
        let Some(alias_text) = pointer_string(item, "/name") else {
            continue;
        };
        let Some(compendium_source) = pointer_string(item, "/_stats/compendiumSource") else {
            continue;
        };
        let Some((pack_name, locator)) = reference_pack_and_locator(&compendium_source) else {
            continue;
        };
        let Some(target_record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
            continue;
        };
        let Some(target_record) = record_by_key(index, &target_record_key) else {
            continue;
        };
        let embedded_remaster = pointer_bool(item, "/system/publication/remaster").unwrap_or(false);
        if embedded_remaster
            || !target_record.publication_remaster
            || should_ignore_compendium_alias(&alias_text, &target_record.name)
        {
            continue;
        }

        add_record_alias(
            &mut aliases,
            &target_record_key,
            &alias_text,
            AliasSource::CompendiumSource,
            &record.key.to_string(),
            index,
        );
    }
    aliases
}

fn add_record_alias(
    aliases: &mut Vec<RecordAlias>,
    canonical_record_key: &RecordKey,
    alias_text: &str,
    source: AliasSource,
    source_ref: &str,
    index: &RecordReferenceIndex,
) {
    let normalized_alias = normalize_text(alias_text);
    if normalized_alias.is_empty() {
        return;
    }
    let Some(canonical_record) = record_by_key(index, canonical_record_key) else {
        return;
    };
    if normalized_alias == canonical_record.normalized_name {
        return;
    }

    aliases.push(RecordAlias {
        canonical_record_key: canonical_record_key.clone(),
        alias_text: alias_text.trim().to_string(),
        normalized_alias,
        source,
        source_ref: source_ref.to_string(),
    });
}

fn should_ignore_compendium_alias(alias_text: &str, target_name: &str) -> bool {
    let normalized_alias = normalize_text(alias_text);
    let normalized_target = normalize_text(target_name);
    if normalized_alias.is_empty() || normalized_alias == normalized_target {
        return true;
    }
    if alias_text
        .chars()
        .any(|character| character.is_ascii_digit())
    {
        return true;
    }
    if contains_any_word(
        &normalized_alias,
        &[
            "feet",
            "foot",
            "mile",
            "miles",
            "precise",
            "imprecise",
            "status",
            "circumstance",
        ],
    ) {
        return true;
    }
    if alias_text.contains('(')
        || alias_text.contains(')')
        || target_name.trim_start().starts_with('(')
    {
        return true;
    }
    false
}

fn contains_any_word(value: &str, words: &[&str]) -> bool {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|word| words.contains(&word))
}

fn dedupe_record_aliases(aliases: Vec<RecordAlias>) -> Vec<RecordAlias> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for alias in aliases {
        let key = (
            alias.canonical_record_key.to_string(),
            alias.normalized_alias.clone(),
            alias.source.as_str(),
            alias.source_ref.clone(),
        );
        if seen.insert(key) {
            deduped.push(alias);
        }
    }
    deduped.sort_by(|left, right| {
        (
            left.canonical_record_key.to_string(),
            left.normalized_alias.as_str(),
            left.source.as_str(),
            left.source_ref.as_str(),
        )
            .cmp(&(
                right.canonical_record_key.to_string(),
                right.normalized_alias.as_str(),
                right.source.as_str(),
                right.source_ref.as_str(),
            ))
    });
    deduped
}

pub(crate) fn resolve_remaster_links(
    records: &[LoadedSourceRecord],
    index: &RecordReferenceIndex,
    source_root: &Path,
) -> Vec<RemasterLink> {
    let mut links = Vec::new();
    for loaded in records {
        let record = &loaded.record;
        if record.foundry_document_type != "JournalEntry"
            || record.normalized_name != "remaster changes"
        {
            continue;
        }

        links.extend(extract_remaster_journal_links(record, index));
    }

    links.extend(extract_migration_remaster_links(source_root, index));
    dedupe_remaster_links(links)
}

fn extract_remaster_journal_links(
    record: &NormalizedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RemasterLink> {
    let mut links = Vec::new();
    for change in extract_remaster_journal_changes(record, index) {
        add_remaster_link(
            &mut links,
            &change.remaster_record_key,
            &change.legacy_name,
            RemasterLinkSource::RemasterJournal,
            &change.source_ref,
            index,
        );
    }
    links
}

fn extract_migration_remaster_links(
    source_root: &Path,
    index: &RecordReferenceIndex,
) -> Vec<RemasterLink> {
    let mut links = Vec::new();
    for (legacy_name, remaster_name) in migration_rename_pairs_from_root(source_root) {
        let Some(remaster_record_key) = resolve_record_key(None, &remaster_name, index) else {
            continue;
        };
        add_remaster_link(
            &mut links,
            &remaster_record_key,
            &legacy_name,
            RemasterLinkSource::Migration,
            "src/module/migration/migrations",
            index,
        );
    }

    links
}

fn add_remaster_link(
    links: &mut Vec<RemasterLink>,
    remaster_record_key: &RecordKey,
    legacy_name: &str,
    source: RemasterLinkSource,
    source_ref: &str,
    index: &RecordReferenceIndex,
) {
    let Some(legacy_record_key) = resolve_record_key(None, legacy_name, index) else {
        return;
    };
    if legacy_record_key == *remaster_record_key {
        return;
    }

    let Some(remaster_record) = record_by_key(index, remaster_record_key) else {
        return;
    };
    let Some(legacy_record) = record_by_key(index, &legacy_record_key) else {
        return;
    };
    if !remaster_record.publication_remaster || legacy_record.publication_remaster {
        return;
    }

    links.push(RemasterLink {
        remaster_record_key: remaster_record_key.clone(),
        legacy_record_key,
        source,
        source_ref: source_ref.to_string(),
    });
}

fn dedupe_remaster_links(links: Vec<RemasterLink>) -> Vec<RemasterLink> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for link in links {
        let key = (
            link.remaster_record_key.to_string(),
            link.legacy_record_key.to_string(),
            link.source.as_str(),
            link.source_ref.clone(),
        );
        if seen.insert(key) {
            deduped.push(link);
        }
    }
    deduped.sort_by(|left, right| {
        (
            left.remaster_record_key.to_string(),
            left.legacy_record_key.to_string(),
            left.source.as_str(),
            left.source_ref.as_str(),
        )
            .cmp(&(
                right.remaster_record_key.to_string(),
                right.legacy_record_key.to_string(),
                right.source.as_str(),
                right.source_ref.as_str(),
            ))
    });
    deduped
}

#[cfg(test)]
mod tests;
