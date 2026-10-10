//! Aliases from explicit Foundry remaster journal and migration declarations.
//! A name rename is alias evidence; only two explicit root references form a pair.
use crate::IngestError;
use atlas_domain::{RecordKey, normalize_record_name};
use atlas_index::{IndexBuildInput, SourceAliasInput, SourceRemasterPairInput};
use atlas_record::{
    source_content::{ContentReferenceResolution, ContentReferenceTarget, prepared_plain_text},
    source_record::SourceContentStatus,
};
use scraper::{ElementRef, Html, Selector};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub(crate) fn extract_source_aliases(
    input: &mut IndexBuildInput,
    root: &Path,
) -> Result<(), IngestError> {
    let selector = |s: &str| {
        Selector::parse(s)
            .map_err(|e| IngestError::SourceSelectionFailed(format!("alias selector: {e}")))
    };
    let reference_selector = selector("[data-atlas-reference]")?;
    let row_selector = selector("tr")?;
    let cell_selector = selector("td")?;
    let list_selector = selector("li")?;
    let names = input
        .records
        .iter()
        .filter(|record| {
            !matches!(
                record.record.source(),
                atlas_foundry_model::FoundryDocumentSource::Macro(_)
            )
        })
        .filter_map(|record| {
            record.record.node_at(&[]).and_then(|node| {
                node.name()
                    .value()
                    .map(|name| (record.record.key().clone(), name.clone()))
            })
        })
        .collect::<BTreeMap<_, _>>();
    let families = input
        .records
        .iter()
        .filter_map(|record| {
            record.record.node_at(&[]).map(|node| {
                (
                    record.record.key().clone(),
                    node.source_type().value().map(str::to_owned),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let mut aliases = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    for record in &input.records {
        if !matches!(
            record.record.source(),
            atlas_foundry_model::FoundryDocumentSource::JournalEntry(_)
        ) || names.get(record.record.key()).map(String::as_str) != Some("Remaster Changes")
        {
            continue;
        }
        for outcome in &record.content {
            let SourceContentStatus::Prepared(content) = &outcome.status else {
                continue;
            };
            let html = Html::parse_fragment(&content.html);
            let targets = |cell: ElementRef<'_>| -> Vec<RecordKey> {
                cell.select(&reference_selector)
                    .filter_map(|marker| {
                        marker
                            .attr("data-atlas-reference")?
                            .parse::<usize>()
                            .ok()
                            .and_then(|ordinal| content.references.get(ordinal))
                    })
                    .filter_map(|reference| match &reference.resolution {
                        ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
                            key,
                        }) => Some(key.clone()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            };
            let evidence = format!(
                "journal:{}:{}:{}",
                record.record.key(),
                serde_json::to_string(&content.locator.owners)
                    .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?,
                content.locator.field
            );
            for row in html.select(&row_selector) {
                let cells = row.select(&cell_selector).collect::<Vec<_>>();
                if cells.len() < 2 {
                    continue;
                }
                if cells.len() >= 4
                    && !matches!(
                        plain(cells[2])?.to_lowercase().as_str(),
                        "renamed" | "merged" | "replaced"
                    )
                {
                    continue;
                }
                let Some(last) = cells.last() else {
                    continue;
                };
                let new = targets(*last);
                if new.len() != 1 {
                    continue;
                }
                let old = targets(cells[0]);
                let alias = if old.len() == 1 {
                    names.get(&old[0]).cloned().unwrap_or_default()
                } else if old.is_empty() {
                    plain(cells[0])?
                } else {
                    continue;
                };
                if !alias.trim().is_empty() {
                    aliases.insert((new[0].clone(), alias, evidence.clone()));
                }
                if old.len() == 1
                    && old[0] != new[0]
                    && families.get(&old[0]).is_some_and(Option::is_some)
                    && families.get(&old[0]) == families.get(&new[0])
                {
                    pairs.insert((old[0].clone(), new[0].clone(), evidence.clone()));
                }
            }
            for item in html.select(&list_selector) {
                let target = targets(item);
                if target.len() != 1 {
                    continue;
                }
                let text = plain(item)?;
                let old = [
                    " are merged into ",
                    " is merged into ",
                    " are now ",
                    " is now ",
                ]
                .iter()
                .find_map(|delimiter| text.split_once(delimiter).map(|(old, _)| old));
                if let Some(old) = old {
                    for name in old
                        .replace(" and ", ", ")
                        .split(',')
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                    {
                        aliases.insert((target[0].clone(), name.into(), evidence.clone()));
                    }
                }
            }
        }
    }
    // These authored comments define textual renames, never old-record identity.
    let directory = root.join("src/module/migration/migrations");
    if directory.is_dir() {
        let mut paths = fs::read_dir(&directory)
            .map_err(|e| IngestError::SourceUnavailable(e.to_string()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
        paths.sort();
        let mut by_name = BTreeMap::<String, Vec<RecordKey>>::new();
        for (key, name) in &names {
            by_name
                .entry(normalize_record_name(name))
                .or_default()
                .push(key.clone());
        }
        for path in paths
            .into_iter()
            .filter(|path| path.extension().is_some_and(|extension| extension == "ts"))
        {
            let source = fs::read_to_string(&path)
                .map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
            for line in source.lines() {
                let Some((_, suffix)) = line.split_once("Rename all uses and mentions of \"")
                else {
                    continue;
                };
                let Some((old, suffix)) = suffix.split_once('"') else {
                    continue;
                };
                let Some((_, suffix)) = suffix.split_once(" to \"") else {
                    continue;
                };
                let Some((new, _)) = suffix.split_once('"') else {
                    continue;
                };
                if let Some(targets) = by_name.get(&normalize_record_name(new))
                    && targets.len() == 1
                    && !old.trim().is_empty()
                {
                    aliases.insert((
                        targets[0].clone(),
                        old.into(),
                        format!(
                            "migration:{}",
                            crate::source::discovery::relative_source_path(root, &path)
                        ),
                    ));
                }
            }
        }
    }
    input.aliases = aliases
        .into_iter()
        .filter(|(key, alias, _)| {
            names
                .get(key)
                .is_some_and(|name| normalize_record_name(name) != normalize_record_name(alias))
        })
        .map(|(record, alias, evidence)| SourceAliasInput {
            record,
            alias,
            evidence,
        })
        .collect();
    input.remaster_pairs = pairs
        .into_iter()
        .map(|(legacy, remaster, evidence)| SourceRemasterPairInput {
            legacy,
            remaster,
            evidence,
        })
        .collect();
    let mut by_root = BTreeMap::<_, Vec<&str>>::new();
    for alias in &input.aliases {
        by_root.entry(&alias.record).or_default().push(&alias.alias);
    }
    for unit in &mut input.lexical_units {
        if matches!(unit.kind, atlas_index::SourceLexicalUnitKind::RootName)
            && let Some(aliases) = by_root.get(&unit.record)
        {
            unit.alias_terms = aliases.join(" ");
        }
    }
    Ok(())
}
fn plain(element: ElementRef<'_>) -> Result<String, IngestError> {
    prepared_plain_text(&element.html())
        .map_err(|error| IngestError::SourceSelectionFailed(error.to_string()))
}
