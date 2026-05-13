use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use atlas_domain::{RecordKey, RemasterLinkSource};
use serde_json::Value;

use crate::normalize::{normalize_text, pointer_bool, pointer_string};
use crate::references::{
    extract_reference_candidates_from_text, record_by_key, reference_pack_and_locator,
    resolve_record_key,
};
use crate::{AliasSource, LoadedRecord, RecordAlias, RecordReferenceIndex, RemasterLink};

pub(super) fn resolve_record_aliases(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
    source_root: &Path,
) -> Vec<RecordAlias> {
    let mut aliases = Vec::new();
    for record in records {
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
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RecordAlias> {
    let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
        return Vec::new();
    };
    let Some(pages) = raw.pointer("/pages").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut aliases = Vec::new();
    for page in pages {
        let page_name = pointer_string(page, "/name").unwrap_or_else(|| "journal-page".to_string());
        let Some(content) = pointer_string(page, "/text/content") else {
            continue;
        };

        if page_name == "Remaster Changes" {
            for list_item in html_elements(&content, "li") {
                let targets = resolve_journal_targets(&list_item, index);
                if targets.len() != 1 {
                    continue;
                }
                let plain = html_text(&list_item);
                let old_segment = split_remaster_intro_alias_segment(&plain);
                for alias_text in split_alias_list_text(&old_segment) {
                    add_record_alias(
                        &mut aliases,
                        &targets[0],
                        &alias_text,
                        AliasSource::RemasterJournal,
                        &format!("journal:{page_name}"),
                        index,
                    );
                }
            }
        }

        for row in html_elements(&content, "tr") {
            let cells = html_cells(&row);
            if cells.len() < 2 {
                continue;
            }
            let status_cell = if cells.len() >= 4 {
                &cells[2]
            } else {
                "Renamed"
            };
            let status = normalize_text(&html_text(status_cell));
            if !matches!(status.as_str(), "renamed" | "merged" | "replaced") {
                continue;
            }

            let old_cell = &cells[0];
            let new_cell = cells.last().expect("row should have at least two cells");
            let targets = resolve_journal_targets(new_cell, index);
            if targets.is_empty() {
                continue;
            }

            if targets.len() == 1 {
                let Some(old_name) = resolve_alias_source_name(old_cell, index) else {
                    continue;
                };
                add_record_alias(
                    &mut aliases,
                    &targets[0],
                    &old_name,
                    AliasSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
                continue;
            }

            let old_text = html_text(old_cell);
            let Some(grouped_aliases) = expand_grouped_alias_text(&old_text, targets.len()) else {
                continue;
            };
            for (alias_text, target) in grouped_aliases.iter().zip(targets.iter()) {
                add_record_alias(
                    &mut aliases,
                    target,
                    alias_text,
                    AliasSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
            }
        }
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
    record: &LoadedRecord,
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

pub(super) fn resolve_remaster_links(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
    source_root: &Path,
) -> Vec<RemasterLink> {
    let mut links = Vec::new();
    for record in records {
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
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RemasterLink> {
    let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
        return Vec::new();
    };
    let Some(pages) = raw.pointer("/pages").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut links = Vec::new();
    for page in pages {
        let page_name = pointer_string(page, "/name").unwrap_or_else(|| "journal-page".to_string());
        let Some(content) = pointer_string(page, "/text/content") else {
            continue;
        };

        if page_name == "Remaster Changes" {
            for list_item in html_elements(&content, "li") {
                let targets = resolve_journal_targets(&list_item, index);
                if targets.len() != 1 {
                    continue;
                }
                let plain = html_text(&list_item);
                let old_segment = split_remaster_intro_alias_segment(&plain);
                for alias_text in split_alias_list_text(&old_segment) {
                    add_remaster_link(
                        &mut links,
                        &targets[0],
                        &alias_text,
                        RemasterLinkSource::RemasterJournal,
                        &format!("journal:{page_name}"),
                        index,
                    );
                }
            }
        }

        for row in html_elements(&content, "tr") {
            let cells = html_cells(&row);
            if cells.len() < 2 {
                continue;
            }
            let status_cell = if cells.len() >= 4 {
                &cells[2]
            } else {
                "Renamed"
            };
            let status = normalize_text(&html_text(status_cell));
            if !matches!(status.as_str(), "renamed" | "merged" | "replaced") {
                continue;
            }

            let old_cell = &cells[0];
            let new_cell = cells.last().expect("row should have at least two cells");
            let targets = resolve_journal_targets(new_cell, index);
            if targets.is_empty() {
                continue;
            }

            if targets.len() == 1 {
                let Some(old_name) = resolve_alias_source_name(old_cell, index) else {
                    continue;
                };
                add_remaster_link(
                    &mut links,
                    &targets[0],
                    &old_name,
                    RemasterLinkSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
                continue;
            }

            let old_text = html_text(old_cell);
            let Some(grouped_aliases) = expand_grouped_alias_text(&old_text, targets.len()) else {
                continue;
            };
            for (alias_text, target) in grouped_aliases.iter().zip(targets.iter()) {
                add_remaster_link(
                    &mut links,
                    target,
                    alias_text,
                    RemasterLinkSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
            }
        }
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

fn migration_rename_pairs_from_root(source_root: &Path) -> Vec<(String, String)> {
    let migration_root = source_root.join("src/module/migration/migrations");
    let Ok(entries) = fs::read_dir(migration_root) else {
        return Vec::new();
    };

    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ts"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut pairs = Vec::new();
    for path in paths {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        pairs.extend(migration_rename_pairs(&source));
    }
    pairs
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

fn resolve_journal_targets(cell_html: &str, index: &RecordReferenceIndex) -> Vec<RecordKey> {
    let candidates = extract_reference_candidates_from_text(cell_html);
    if candidates.is_empty() {
        let plain = html_text(cell_html);
        return resolve_record_key(None, &plain, index)
            .into_iter()
            .collect();
    }

    let mut targets = Vec::new();
    for candidate in candidates {
        let Some((pack_name, locator)) = reference_pack_and_locator(&candidate.raw_target) else {
            continue;
        };
        let Some(record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
            continue;
        };
        if record_by_key(index, &record_key)
            .is_some_and(|record| record.foundry_document_type != "JournalEntry")
        {
            targets.push(record_key);
        }
    }
    targets
}

fn resolve_alias_source_name(cell_html: &str, index: &RecordReferenceIndex) -> Option<String> {
    let direct_text = html_text(cell_html);
    if !direct_text.is_empty() && !cell_html.contains("@UUID[") {
        return Some(direct_text);
    }

    let candidate = extract_reference_candidates_from_text(cell_html)
        .into_iter()
        .next()?;
    let (pack_name, locator) = reference_pack_and_locator(&candidate.raw_target)?;
    let record_key = resolve_record_key(Some(&pack_name), &locator, index)?;
    record_by_key(index, &record_key).map(|record| record.name.clone())
}

fn split_remaster_intro_alias_segment(plain_text: &str) -> String {
    for delimiter in [
        " are merged into ",
        " is merged into ",
        " are now ",
        " is now ",
    ] {
        if let Some((segment, _)) = plain_text.split_once(delimiter) {
            return segment.trim().to_string();
        }
    }
    plain_text.trim().to_string()
}

fn split_alias_list_text(value: &str) -> Vec<String> {
    value
        .replace(" and ", ", ")
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn expand_grouped_alias_text(alias_text: &str, expected_count: usize) -> Option<Vec<String>> {
    let open = alias_text.rfind('(')?;
    let close = alias_text.rfind(')')?;
    if close <= open {
        return None;
    }
    let base_name = alias_text[..open].trim();
    let variants = split_alias_list_text(&alias_text[open + 1..close]);
    if base_name.is_empty() || variants.len() != expected_count {
        return None;
    }
    Some(
        variants
            .into_iter()
            .map(|variant| format!("{base_name} ({variant})"))
            .collect(),
    )
}

fn migration_rename_pairs(source: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("Rename all uses and mentions of \"") {
        rest = &rest[start + "Rename all uses and mentions of \"".len()..];
        let Some(old_end) = rest.find('"') else {
            break;
        };
        let legacy_name = rest[..old_end].trim().to_string();
        rest = &rest[old_end + 1..];
        let Some(to_start) = rest.find(" to \"") else {
            continue;
        };
        rest = &rest[to_start + " to \"".len()..];
        let Some(new_end) = rest.find('"') else {
            break;
        };
        let remaster_name = rest[..new_end].trim().to_string();
        rest = &rest[new_end + 1..];
        if !legacy_name.is_empty() && !remaster_name.is_empty() {
            pairs.push((legacy_name, remaster_name));
        }
    }
    pairs
}

fn html_cells(row_html: &str) -> Vec<String> {
    let mut cells = html_elements(row_html, "td");
    cells.extend(html_elements(row_html, "th"));
    cells
}

fn html_elements(html: &str, tag_name: &str) -> Vec<String> {
    let lower = html.to_lowercase();
    let open_prefix = format!("<{tag_name}");
    let close_tag = format!("</{tag_name}>");
    let mut elements = Vec::new();
    let mut offset = 0;
    while let Some(open_relative) = lower[offset..].find(&open_prefix) {
        let open = offset + open_relative;
        let Some(open_end_relative) = lower[open..].find('>') else {
            break;
        };
        let content_start = open + open_end_relative + 1;
        let Some(close_relative) = lower[content_start..].find(&close_tag) else {
            break;
        };
        let close = content_start + close_relative;
        elements.push(html[content_start..close].to_string());
        offset = close + close_tag.len();
    }
    elements
}

fn html_text(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            '@' if !in_tag && chars.peek().is_some_and(|next| *next == 'U') => {
                for next in chars.by_ref() {
                    if next == ']' {
                        break;
                    }
                }
                if chars.peek().is_some_and(|next| *next == '{') {
                    let _ = chars.next();
                    for display in chars.by_ref() {
                        if display == '}' {
                            break;
                        }
                        output.push(display);
                    }
                }
                output.push(' ');
            }
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests;
