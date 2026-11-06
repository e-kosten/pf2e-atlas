use crate::IngestError;
use atlas_domain::{RecordKey, normalize_record_name};
use atlas_index::{SourceAliasInput, SourceRemasterPairInput};
use atlas_record::{
    remaster::*,
    source_content::ContentVisibilityRule,
    source_record::{
        FieldAvailability, SourceBackedRecord, SourceContentOutcome, SourceContentStatus,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn error(e: impl std::fmt::Display) -> IngestError {
    IngestError::SourceSelectionFailed(e.to_string())
}
fn insert_pair(
    pairs: &mut BTreeMap<RecordKey, SourceRemasterPairInput>,
    pair: SourceRemasterPairInput,
) -> Result<(), IngestError> {
    if let Some(previous) = pairs.get(&pair.legacy)
        && previous.remaster != pair.remaster
    {
        return Err(error(format!(
            "conflicting remaster counterparts for {}: {} and {}",
            pair.legacy, previous.remaster, pair.remaster
        )));
    }
    pairs.insert(pair.legacy.clone(), pair);
    Ok(())
}
type JournalVisitor<'a> =
    dyn FnMut(&SourceBackedRecord, &[SourceContentOutcome]) -> Result<(), IngestError> + 'a;
type JournalStream<'a> = dyn FnMut(&mut JournalVisitor<'_>) -> Result<(), IngestError> + 'a;
type PreparedRemaster = (
    Vec<SourceAliasInput>,
    Vec<SourceRemasterPairInput>,
    Vec<RecordRemasterContext>,
);
pub(crate) fn extract_prepared_aliases(
    names: &BTreeMap<RecordKey, String>,
    families: &BTreeMap<RecordKey, Option<String>>,
    journals: &mut JournalStream<'_>,
    root: &Path,
    capture: &crate::source_fingerprint::SourceCapture,
) -> Result<PreparedRemaster, IngestError> {
    let mut contexts = vec![];
    let audience = crate::build::default_audience();
    journals(&mut |record, outcomes| {
        for outcome in outcomes {
            let visible = match outcome.visibility {
                ContentVisibilityRule::All => true,
                ContentVisibilityRule::Gm => audience.include_gm,
                ContentVisibilityRule::Owner => audience.include_owner,
                ContentVisibilityRule::None => false,
            };
            if !visible || !matches!(outcome.visibility_availability, FieldAvailability::Value) {
                continue;
            }
            let SourceContentStatus::Prepared(content) = &outcome.status else {
                continue;
            };
            let Some(section) = record
                .node_at(&content.locator.owners)
                .and_then(|n| n.name().value().cloned())
            else {
                continue;
            };
            contexts.extend(journal_context(&section, content).map_err(error)?);
        }
        Ok(())
    })?;
    let mapping: Correspondences =
        serde_json::from_slice(ATLAS_CORRESPONDENCE_BYTES).map_err(error)?;
    let mut pairs = BTreeMap::new();
    for context in &contexts {
        if !context.owners.is_empty()
            || context.targets.len() != 1
            || !matches!(
                context.relation,
                RemasterRelation::RenamedTo | RemasterRelation::ReplacedBy
            )
        {
            continue;
        }
        let target = &context.targets[0];
        if let Some(new) = &target.record
            && target.owners.is_empty()
            && &context.record != new
            && families.get(&context.record).is_some_and(Option::is_some)
            && families.get(&context.record) == families.get(new)
        {
            insert_pair(
                &mut pairs,
                SourceRemasterPairInput {
                    legacy: context.record.clone(),
                    remaster: new.clone(),
                    provenance: context.journal.clone(),
                },
            )?;
        }
    }
    for entry in mapping.entries {
        if families.get(&entry.before).and_then(Option::as_deref)
            != Some(entry.source_type.as_str())
            || families.get(&entry.after).and_then(Option::as_deref)
                != Some(entry.source_type.as_str())
        {
            continue;
        }
        let journal = entry.journal_context.map(|j| j.locator).filter(|locator| {
            contexts.iter().any(|c| {
                c.record == entry.after
                    && c.owners.is_empty()
                    && c.journal.as_ref() == Some(locator)
                    && c.targets.iter().any(|t| {
                        t.record.as_ref() == Some(&entry.before)
                            || (t.record.is_none()
                                && normalize_record_name(&t.name)
                                    == normalize_record_name(&entry.before_label))
                    })
            })
        });
        let old = RemasterTarget {
            name: entry.before_label,
            record: Some(entry.before.clone()),
            owners: vec![],
        };
        if let Some(journal) = &journal {
            for context in &mut contexts {
                if context.record == entry.after
                    && context.owners.is_empty()
                    && context.journal.as_ref() == Some(journal)
                {
                    for target in &mut context.targets {
                        if target.record.is_none()
                            && normalize_record_name(&target.name)
                                == normalize_record_name(&old.name)
                        {
                            *target = old.clone();
                        }
                    }
                }
            }
        }
        insert_pair(
            &mut pairs,
            SourceRemasterPairInput {
                legacy: entry.before,
                remaster: entry.after,
                provenance: journal,
            },
        )?;
    }
    atlas_record::remaster::refine_context(&mut contexts);
    let mut aliases = BTreeSet::new();
    for context in &contexts {
        if !context.owners.is_empty()
            || !matches!(
                context.relation,
                RemasterRelation::FormerName
                    | RemasterRelation::Replaces
                    | RemasterRelation::MergedFrom
            )
        {
            continue;
        }
        for target in &context.targets {
            let name = if let Some(key) = &target.record {
                names.get(key).unwrap_or(&target.name)
            } else {
                &target.name
            };
            if !name.trim().is_empty() {
                aliases.insert((
                    context.record.clone(),
                    name.clone(),
                    format!(
                        "journal:{}",
                        serde_json::to_string(&context.journal).map_err(error)?
                    ),
                ));
            }
        }
    }
    for pair in pairs.values() {
        if let Some(name) = names.get(&pair.legacy) {
            aliases.insert((
                pair.remaster.clone(),
                name.clone(),
                match &pair.provenance {
                    Some(locator) => {
                        format!("journal:{}", serde_json::to_string(locator).map_err(error)?)
                    }
                    None => format!(
                        "atlas:crates/atlas-record/src/remaster/correspondences.json#{}",
                        pair.legacy
                    ),
                },
            ));
        }
    }
    // These authored comments define textual renames, never old-record identity.
    let directory = root.join("src/module/migration/migrations");
    {
        let paths = capture.paths(&directory);
        let mut by_name = BTreeMap::<String, Vec<RecordKey>>::new();
        for (key, name) in names {
            by_name
                .entry(normalize_record_name(name))
                .or_default()
                .push(key.clone());
        }
        for path in paths.into_iter().filter(|path| {
            path.parent() == Some(directory.as_path())
                && path.extension().is_some_and(|extension| extension == "ts")
        }) {
            let source = String::from_utf8(capture.read(&path)?)
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
                    contexts.push(RecordRemasterContext {
                        record: targets[0].clone(),
                        owners: vec![],
                        relation: RemasterRelation::FormerName,
                        targets: vec![RemasterTarget {
                            name: old.into(),
                            record: None,
                            owners: vec![],
                        }],
                        journal: None,
                        scope: "migration rename".into(),
                    });
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

    let mut unique = BTreeMap::new();
    for (record, alias, evidence) in aliases {
        if names
            .get(&record)
            .is_some_and(|name| normalize_record_name(name) != normalize_record_name(&alias))
        {
            unique
                .entry((record.clone(), alias.clone()))
                .or_insert(SourceAliasInput {
                    record,
                    alias,
                    evidence,
                });
        }
    }
    Ok((
        unique.into_values().collect(),
        pairs.into_values().collect(),
        contexts,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conflicting_explicit_counterparts_are_rejected_instead_of_last_wins() {
        let mut pairs = BTreeMap::new();
        let pair = SourceRemasterPairInput {
            legacy: "items:aaaaaaaaaaaaaaaa".parse().unwrap(),
            remaster: "items:bbbbbbbbbbbbbbbb".parse().unwrap(),
            provenance: None,
        };
        insert_pair(&mut pairs, pair.clone()).unwrap();
        insert_pair(&mut pairs, pair.clone()).unwrap();
        let conflict = SourceRemasterPairInput {
            remaster: "items:cccccccccccccccc".parse().unwrap(),
            ..pair.clone()
        };
        assert!(insert_pair(&mut pairs, conflict).is_err());
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[&pair.legacy], pair);
    }
}
