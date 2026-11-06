//! Exact occurrence reconstruction from retained checked sources for deep audit.
use crate::{
    DefinitionReferenceInput, DefinitionReferenceSource, DefinitionReferenceTarget, IndexError,
    ReferenceBacking, ReferenceCoverage, ReferenceEntry, SourceArtifactBuildContext,
};
use atlas_domain::DefinitionKey;
use atlas_record::source_content::{
    ContentReferenceResolution, ContentReferenceTarget, ContentVisibilityRule,
    DefinitionLocalizationMap,
};
use rusqlite::Connection;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn visit_entries(
    c: &Connection,
    visitor: &mut dyn FnMut(ReferenceEntry) -> Result<(), IndexError>,
) -> Result<(), IndexError> {
    let mut statement = c.prepare(
        "SELECT key,definition_json,definition_hash FROM reference_entries ORDER BY key",
    )?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let json: String = row.get(1)?;
        let hash: String = row.get(2)?;
        if crate::codec::sha256(json.as_bytes()) != hash {
            return Err(IndexError::Invalid(
                "reference definition checksum mismatch".into(),
            ));
        }
        let entry: ReferenceEntry = serde_json::from_str(&json)?;
        if entry.key.to_string() != key {
            return Err(IndexError::Invalid(
                "reference definition identity mismatch".into(),
            ));
        }
        crate::references::validate_entry(c, &entry)?;
        visitor(entry)?;
    }
    Ok(())
}

pub(crate) fn prepared_record_references(entry: &ReferenceEntry) -> Vec<DefinitionReferenceInput> {
    let ReferenceBacking::Prepared { content, .. } = &entry.backing else {
        return vec![];
    };
    content
        .references
        .iter()
        .filter_map(|occurrence| {
            let ContentReferenceResolution::Resolved(target) = &occurrence.resolution else {
                return None;
            };
            let (key, owners) = match target {
                ContentReferenceTarget::Record { key } => (key.clone(), vec![]),
                ContentReferenceTarget::OwnedNode { key, owners } => (key.clone(), owners.clone()),
                ContentReferenceTarget::Url { .. } => return None,
            };
            let label = if owners.is_empty() {
                None
            } else {
                occurrence.label.target_name.clone()
            };
            Some(DefinitionReferenceInput {
                source: DefinitionReferenceSource::Definition {
                    key: entry.key.clone(),
                },
                target: DefinitionReferenceTarget::Record { key, owners },
                occurrence_path: occurrence.path.clone(),
                evidence: format!(
                    "{}:{}",
                    entry.provenance.description_key.as_deref().unwrap_or(""),
                    occurrence.authored_target
                ),
                label,
                visible: occurrence.visible,
            })
        })
        .collect()
}

struct ReferenceCatalog {
    keys: DefinitionLocalizationMap,
    targets: BTreeMap<DefinitionKey, DefinitionReferenceTarget>,
}
impl ReferenceCatalog {
    fn read(c: &Connection) -> Result<Self, IndexError> {
        let mut catalog = Self {
            keys: Default::default(),
            targets: Default::default(),
        };
        visit_entries(c, &mut |entry| {
            if entry.availability == crate::ReferenceAvailability::SourceInapplicable
                || (entry.provenance.record_mapping.is_some()
                    && matches!(&entry.backing, ReferenceBacking::None))
            {
                return Ok(());
            }
            let target = match &entry.backing {
                ReferenceBacking::Record { key } => DefinitionReferenceTarget::Record {
                    key: key.clone(),
                    owners: vec![],
                },
                _ => DefinitionReferenceTarget::Definition {
                    key: entry.key.clone(),
                },
            };
            catalog.targets.insert(entry.key.clone(), target);
            for key in [entry.provenance.label_key, entry.provenance.description_key]
                .into_iter()
                .flatten()
            {
                catalog
                    .keys
                    .entry(key)
                    .or_default()
                    .insert(entry.key.clone());
            }
            Ok(())
        })?;
        for keys in catalog.keys.values_mut() {
            if keys.len() > 1 {
                let targets = keys
                    .iter()
                    .map(|key| serde_json::to_string(&catalog.targets[key]))
                    .collect::<Result<BTreeSet<_>, _>>()?;
                let selected = keys
                    .first()
                    .cloned()
                    .ok_or_else(|| IndexError::Invalid("empty localization mapping".into()))?;
                // Distinct properties can share a real definition destination.
                // Never collapse standalone identities sharing only their prose.
                if targets.len() == 1
                    && matches!(
                        catalog.targets[&selected],
                        DefinitionReferenceTarget::Record { .. }
                    )
                {
                    keys.clear();
                    keys.insert(selected);
                }
            }
        }
        Ok(catalog)
    }
}

/// Reconstruct through the record-owned authored parser, never expanded prose.
pub(crate) fn validate(
    c: &Connection,
    context: &SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    if let Some(preparation) = &context.reference_preparation {
        if preparation.recipe.is_empty() || !crate::codec::checked_hash(&preparation.input_hash) {
            return Err(IndexError::Invalid(
                "invalid reference preparation identity".into(),
            ));
        }
    }
    let keys = c
        .prepare("SELECT key FROM reference_entries ORDER BY key")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    crate::references::validate_inventory(context.reference_preparation.as_ref(), keys)?;
    let catalog = ReferenceCatalog::read(c)?;
    let complete = context
        .reference_preparation
        .as_ref()
        .is_some_and(|p| p.coverage == ReferenceCoverage::Complete);
    let mut expected = BTreeSet::new();
    visit_entries(c, &mut |entry| {
        for reference in prepared_record_references(&entry) {
            expected.insert(crate::references::reference_id(&reference)?);
        }
        if complete {
            if let ReferenceBacking::Prepared {
                authored_markup, ..
            } = &entry.backing
            {
                for occurrence in
                    atlas_record::source_content::definition_body_localization_references(
                        authored_markup,
                        &catalog.keys,
                        context.audience,
                        ContentVisibilityRule::All,
                    )
                {
                    let target = catalog
                        .targets
                        .get(&occurrence.key)
                        .ok_or_else(|| {
                            IndexError::Invalid(
                                "definition body occurrence target is absent".into(),
                            )
                        })?
                        .clone();
                    let reference = DefinitionReferenceInput {
                        source: DefinitionReferenceSource::Definition {
                            key: entry.key.clone(),
                        },
                        target,
                        occurrence_path: occurrence.path,
                        evidence: occurrence.localization_key,
                        label: None,
                        visible: occurrence.visible,
                    };
                    expected.insert(crate::references::reference_id(&reference)?);
                }
            }
        }
        Ok(())
    })?;
    crate::visit_persisted_inputs(c, context, &mut |source| {
        if complete {
            for occurrence in atlas_record::source_record::definition_record_localization_references(
                &source.record,
                context.audience,
                &catalog.keys,
            ) {
                let target = catalog
                    .targets
                    .get(&occurrence.key)
                    .ok_or_else(|| {
                        IndexError::Invalid("record occurrence target is absent".into())
                    })?
                    .clone();
                let label = if occurrence.locator.owners.is_empty() {
                    None
                } else {
                    source
                        .record
                        .node_at(&occurrence.locator.owners)
                        .and_then(|node| node.name().value().cloned())
                };
                let reference = DefinitionReferenceInput {
                    source: DefinitionReferenceSource::Record {
                        key: occurrence.locator.record,
                        owners: occurrence.locator.owners,
                        field: Some(occurrence.locator.field),
                    },
                    target,
                    occurrence_path: occurrence.path,
                    evidence: occurrence.localization_key,
                    label,
                    visible: occurrence.visible,
                };
                expected.insert(crate::references::reference_id(&reference)?);
            }
        } else {
            // Retained trait-only coverage does not claim complete new glossary
            // associations; its existing visible trait surface remains checked.
            for occurrence in
                crate::traits::record_reference_inputs(&source.record, context.audience)?
            {
                expected.insert(crate::references::reference_id(&occurrence)?);
            }
        }
        Ok(())
    })?;
    let actual = actual_occurrences(c)?;
    if actual != expected {
        return Err(IndexError::Invalid(
            "definition reference evidence differs from checked source".into(),
        ));
    }
    let mut notices = c.prepare("SELECT id,sha256,text,source_url FROM reference_notices")?;
    for row in notices.query_map([], |r| {
        Ok(crate::ReferenceNotice {
            id: r.get(0)?,
            sha256: r.get(1)?,
            text: r.get(2)?,
            source_url: r.get(3)?,
        })
    })? {
        crate::references::validate_notice(&row?)?;
    }
    Ok(())
}

fn actual_occurrences(c: &Connection) -> Result<BTreeSet<String>, IndexError> {
    let mut actual = BTreeSet::new();
    let mut statement=c.prepare("SELECT occurrence_id,source_record_key,source_definition_key,target_record_key,target_definition_key,visible,evidence FROM definition_reference_occurrences")?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let json: String = row.get(6)?;
        let reference: DefinitionReferenceInput = serde_json::from_str(&json)?;
        let source_record: Option<String> = row.get(1)?;
        let source_definition: Option<String> = row.get(2)?;
        let target_record: Option<String> = row.get(3)?;
        let target_definition: Option<String> = row.get(4)?;
        let visible: bool = row.get(5)?;
        let source_ok = match &reference.source {
            DefinitionReferenceSource::Record { key, .. } => {
                source_record == Some(key.to_string()) && source_definition.is_none()
            }
            DefinitionReferenceSource::Definition { key } => {
                source_definition == Some(key.to_string()) && source_record.is_none()
            }
        };
        let target_ok = match &reference.target {
            DefinitionReferenceTarget::Record { key, .. } => {
                target_record == Some(key.to_string()) && target_definition.is_none()
            }
            DefinitionReferenceTarget::Definition { key } => {
                target_definition == Some(key.to_string()) && target_record.is_none()
            }
        };
        if !source_ok
            || !target_ok
            || reference.visible != visible
            || crate::references::reference_id(&reference)? != id
            || serde_json::to_string(&reference)? != json
        {
            return Err(IndexError::Invalid(
                "definition occurrence identity/projection mismatch".into(),
            ));
        }
        actual.insert(id);
    }
    Ok(actual)
}
