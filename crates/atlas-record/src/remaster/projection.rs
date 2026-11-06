use super::*;
use atlas_domain::{OwnedContentLocator, RecordKey, normalize_record_name};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemasterSideKind {
    Before,
    After,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RemasterBindingResolution {
    Current {
        record_key: RecordKey,
        owners: Vec<OwnedContentLocator>,
    },
    MissingExplicitKey {
        record_key: RecordKey,
    },
    Unresolved,
    Ambiguous {
        candidates: Vec<(RecordKey, Vec<OwnedContentLocator>)>,
        truncated: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemasterBinding {
    pub declaration: String,
    pub side: RemasterSideKind,
    pub group: usize,
    pub reference: usize,
    pub resolution: RemasterBindingResolution,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemasterAlias {
    pub record: RecordKey,
    pub alias: String,
    pub declarations: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemasterPreference {
    pub predecessor: RecordKey,
    pub successor: RecordKey,
    pub declarations: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemasterProjection {
    pub aliases: Vec<RemasterAlias>,
    pub preferences: Vec<RemasterPreference>,
    pub diagnostics: Vec<RemasterDerivationDiagnostic>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemasterDerivationReason {
    AdapterConditions,
    UnsupportedDisposition,
    MissingExplicitPredecessor,
    GroupedEndpoints,
    EndpointUnavailable,
    SameIdentity,
    FamilyMismatch,
    ConflictingDisposition,
    Cycle,
}
impl RemasterDerivationReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AdapterConditions => "adapter_conditions",
            Self::UnsupportedDisposition => "unsupported_disposition",
            Self::MissingExplicitPredecessor => "missing_explicit_predecessor",
            Self::GroupedEndpoints => "grouped_endpoints",
            Self::EndpointUnavailable => "endpoint_unavailable",
            Self::SameIdentity => "same_identity",
            Self::FamilyMismatch => "family_mismatch",
            Self::ConflictingDisposition => "conflicting_disposition",
            Self::Cycle => "cycle",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemasterDerivationDiagnostic {
    pub declaration: String,
    pub reason: RemasterDerivationReason,
}

#[derive(Debug, Clone)]
pub struct RemasterRecordFacts {
    pub name: Option<String>,
    pub source_type: Option<String>,
}
fn roots<'a>(
    bindings: &'a [RemasterBinding],
    id: &str,
    side: RemasterSideKind,
) -> Vec<&'a RecordKey> {
    bindings
        .iter()
        .filter(|b| b.declaration == id && b.side == side)
        .filter_map(|b| match &b.resolution {
            RemasterBindingResolution::Current { record_key, owners } if owners.is_empty() => {
                Some(record_key)
            }
            _ => None,
        })
        .collect()
}
/// A retained statement can stay navigable after current conditions withdraw
/// every derived lookup/discovery permission from its source adapter.
fn adapter_approved(
    declaration: &RemasterDeclaration,
    bindings: &[RemasterBinding],
    records: &BTreeMap<RecordKey, RemasterRecordFacts>,
    sources: &[RemasterSourceInput],
) -> bool {
    let before = roots(bindings, &declaration.id, RemasterSideKind::Before);
    let after = roots(bindings, &declaration.id, RemasterSideKind::After);
    declaration.citations.iter().all(|citation| match citation {
        RemasterCitation::Journal { .. } => true,
        RemasterCitation::CapturedFile { input_key, .. } => sources
            .iter()
            .find(|s| &s.key == input_key)
            .is_some_and(|source| match source.kind {
                RemasterInputKind::Migration850 => {
                    after.len() == 1
                        && records
                            .get(after[0])
                            .is_some_and(|r| r.source_type.as_deref() == Some("condition"))
                }
                RemasterInputKind::Migration887 => {
                    after.len() == 1
                        && records
                            .get(after[0])
                            .is_some_and(|r| r.source_type.as_deref() == Some("spell"))
                        && before.iter().all(|key| {
                            records
                                .get(*key)
                                .is_some_and(|r| r.source_type.as_deref() == Some("spell"))
                        })
                }
                RemasterInputKind::AtlasCorrespondence => {
                    before.len() == 1
                        && after.len() == 1
                        && declaration
                            .context
                            .source_type
                            .as_ref()
                            .is_some_and(|expected| {
                                before.iter().chain(&after).all(|key| {
                                    records
                                        .get(*key)
                                        .is_some_and(|r| r.source_type.as_ref() == Some(expected))
                                })
                            })
                        && (declaration.context.namespace != "creature_ability"
                            || before
                                .iter()
                                .chain(&after)
                                .all(|key| key.pack().as_str() == "bestiary-ability-glossary-srd"))
                }
            }),
    })
}
/// Source relations grant lookup and discovery different, explicit permissions.
pub fn derive_remaster_projection(
    declarations: &[RemasterDeclaration],
    bindings: &[RemasterBinding],
    records: &BTreeMap<RecordKey, RemasterRecordFacts>,
    sources: &[RemasterSourceInput],
) -> RemasterProjection {
    let mut aliases = BTreeMap::<(RecordKey, String), BTreeSet<String>>::new();
    let mut preferences = BTreeMap::<(RecordKey, RecordKey), BTreeSet<String>>::new();
    let mut unsafe_predecessors = BTreeSet::new();
    let mut diagnostics = BTreeSet::<(String, RemasterDerivationReason)>::new();
    for declaration in declarations {
        let approved = adapter_approved(declaration, bindings, records, sources);
        let kinds = &declaration.kinds;
        let after_refs = declaration
            .after
            .iter()
            .map(|s| s.references.len())
            .sum::<usize>();
        let after = roots(bindings, &declaration.id, RemasterSideKind::After);
        if approved
            && after_refs == 1
            && after.len() == 1
            && kinds.iter().any(|k| {
                matches!(
                    k,
                    RemasterChangeKind::Renamed
                        | RemasterChangeKind::Merged
                        | RemasterChangeKind::Replaced
                )
            })
            && !kinds.iter().any(|k| {
                matches!(
                    k,
                    RemasterChangeKind::Removed
                        | RemasterChangeKind::Unclassified
                        | RemasterChangeKind::Discussion
                )
            })
            && let Some(record) = records.get(after[0])
        {
            for term in declaration.before.iter().flat_map(|s| &s.terms) {
                if !term.trim().is_empty()
                    && record.name.as_ref().is_none_or(|name| {
                        normalize_record_name(name) != normalize_record_name(term)
                    })
                {
                    aliases
                        .entry((after[0].clone(), term.clone()))
                        .or_default()
                        .insert(declaration.id.clone());
                }
            }
        }
        let before_refs = declaration
            .before
            .iter()
            .map(|s| s.references.len())
            .sum::<usize>();
        let before = roots(bindings, &declaration.id, RemasterSideKind::Before);
        if !kinds.contains(&RemasterChangeKind::Discussion) {
            let reason = if !approved {
                Some(RemasterDerivationReason::AdapterConditions)
            } else if kinds.iter().any(|k| {
                matches!(
                    k,
                    RemasterChangeKind::Merged
                        | RemasterChangeKind::Removed
                        | RemasterChangeKind::Unclassified
                )
            }) {
                Some(RemasterDerivationReason::UnsupportedDisposition)
            } else if before_refs == 0 {
                Some(RemasterDerivationReason::MissingExplicitPredecessor)
            } else if before_refs != 1 || after_refs != 1 {
                Some(RemasterDerivationReason::GroupedEndpoints)
            } else if before.len() != 1 || after.len() != 1 {
                Some(RemasterDerivationReason::EndpointUnavailable)
            } else if before[0] == after[0] {
                Some(RemasterDerivationReason::SameIdentity)
            } else {
                None
            };
            if let Some(reason) = reason {
                diagnostics.insert((declaration.id.clone(), reason));
            }
        }
        // A grouped or contradictory authored disposition cannot be made safe by
        // another single-output declaration. Keep its information, veto preference.
        if !kinds.contains(&RemasterChangeKind::Discussion)
            && before.len() == 1
            && before_refs == 1
            && (after_refs != 1
                || after.len() != 1
                || kinds.iter().any(|k| {
                    matches!(
                        k,
                        RemasterChangeKind::Merged
                            | RemasterChangeKind::Removed
                            | RemasterChangeKind::Unclassified
                    )
                }))
        {
            unsafe_predecessors.insert(before[0].clone());
        }
        if !approved
            || before_refs != 1
            || after_refs != 1
            || before.len() != 1
            || after.len() != 1
            || before[0] == after[0]
            || kinds.iter().any(|k| {
                matches!(
                    k,
                    RemasterChangeKind::Merged
                        | RemasterChangeKind::Removed
                        | RemasterChangeKind::Unclassified
                        | RemasterChangeKind::Discussion
                )
            })
            || !kinds.iter().any(|k| {
                matches!(
                    k,
                    RemasterChangeKind::Renamed | RemasterChangeKind::Replaced
                )
            })
        {
            continue;
        }
        let (Some(old), Some(new)) = (records.get(before[0]), records.get(after[0])) else {
            continue;
        };
        if old.source_type.is_none()
            || old.source_type != new.source_type
            || declaration
                .context
                .source_type
                .as_ref()
                .is_some_and(|family| old.source_type.as_ref() != Some(family))
        {
            diagnostics.insert((
                declaration.id.clone(),
                RemasterDerivationReason::FamilyMismatch,
            ));
            continue;
        }
        if declaration.context.namespace == "creature_ability"
            && (before[0].pack().as_str() != "bestiary-ability-glossary-srd"
                || after[0].pack().as_str() != "bestiary-ability-glossary-srd")
        {
            continue;
        }
        preferences
            .entry((before[0].clone(), after[0].clone()))
            .or_default()
            .insert(declaration.id.clone());
    }
    let mut successors = BTreeMap::<RecordKey, BTreeSet<RecordKey>>::new();
    for (old, new) in preferences.keys() {
        successors
            .entry(old.clone())
            .or_default()
            .insert(new.clone());
    }
    preferences.retain(|(old, _), support| {
        let keep =
            !unsafe_predecessors.contains(old) && successors.get(old).is_some_and(|s| s.len() == 1);
        if !keep {
            for id in support.iter() {
                diagnostics.insert((id.clone(), RemasterDerivationReason::ConflictingDisposition));
            }
        }
        keep
    });
    let graph = preferences
        .keys()
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut cyclic = BTreeSet::new();
    for start in graph.keys() {
        let mut path = Vec::new();
        let mut node = start;
        while let Some(next) = graph.get(node) {
            if let Some(position) = path.iter().position(|seen| seen == node) {
                cyclic.extend(path[position..].iter().cloned());
                break;
            }
            path.push(node.clone());
            node = next;
        }
    }
    for ((old, _), support) in &preferences {
        if cyclic.contains(old) {
            for id in support {
                diagnostics.insert((id.clone(), RemasterDerivationReason::Cycle));
            }
        }
    }
    RemasterProjection {
        diagnostics: diagnostics
            .into_iter()
            .map(|(declaration, reason)| RemasterDerivationDiagnostic {
                declaration,
                reason,
            })
            .collect(),
        aliases: aliases
            .into_iter()
            .map(|((record, alias), sources)| RemasterAlias {
                record,
                alias,
                declarations: sources.into_iter().collect(),
            })
            .collect(),
        preferences: preferences
            .into_iter()
            .filter(|((old, _), _)| !cyclic.contains(old))
            .map(|((predecessor, successor), sources)| RemasterPreference {
                predecessor,
                successor,
                declarations: sources.into_iter().collect(),
            })
            .collect(),
    }
}

#[derive(Debug, Clone)]
pub struct RemasterSelectedField {
    pub locator: crate::source_content::SourceContentLocator,
    pub authored_sha256: String,
    pub references: Vec<crate::source_content::ContentReferenceOccurrence>,
}

pub fn bind_remaster_declarations(
    declarations: &[RemasterDeclaration],
    records: &BTreeMap<RecordKey, RemasterRecordFacts>,
    fields: &[RemasterSelectedField],
) -> Result<Vec<RemasterBinding>, RemasterError> {
    use crate::source_content::{ContentReferenceResolution, ContentReferenceTarget};
    let mut result = vec![];
    for declaration in declarations {
        for (side, groups) in [
            (RemasterSideKind::Before, &declaration.before),
            (RemasterSideKind::After, &declaration.after),
        ] {
            for (group, value) in groups.iter().enumerate() {
                for (reference, value) in value.references.iter().enumerate() {
                    let resolution = if let Some(ordinal) = value.occurrence {
                        let (locator, hash) = declaration
                            .citations
                            .iter()
                            .find_map(|c| match c {
                                RemasterCitation::Journal {
                                    locator,
                                    authored_sha256,
                                    ..
                                } => Some((locator, authored_sha256)),
                                _ => None,
                            })
                            .ok_or_else(|| {
                                RemasterError::Invalid(
                                    "journal reference has no selected field".into(),
                                )
                            })?;
                        let field = fields
                            .iter()
                            .find(|f| &f.locator == locator && &f.authored_sha256 == hash)
                            .ok_or_else(|| {
                                RemasterError::Invalid("remaster field is not selected".into())
                            })?;
                        let selected = field
                            .references
                            .iter()
                            .find(|r| {
                                r.ordinal == ordinal
                                    && r.visible
                                    && r.authored_target == value.target
                            })
                            .ok_or_else(|| {
                                RemasterError::Invalid(
                                    "remaster reference attribution differs".into(),
                                )
                            })?;
                        match &selected.resolution {
                            ContentReferenceResolution::Resolved(
                                ContentReferenceTarget::Record { key },
                            ) if records.contains_key(key) => RemasterBindingResolution::Current {
                                record_key: key.clone(),
                                owners: vec![],
                            },
                            ContentReferenceResolution::Resolved(
                                ContentReferenceTarget::OwnedNode { key, owners },
                            ) if records.contains_key(key) => RemasterBindingResolution::Current {
                                record_key: key.clone(),
                                owners: owners.clone(),
                            },
                            _ => crate::source_record::SourceReferenceIndex::explicit_root_key(
                                &selected.authored_target,
                            )
                            .filter(|key| !records.contains_key(key))
                            .map(|record_key| RemasterBindingResolution::MissingExplicitKey {
                                record_key,
                            })
                            .unwrap_or(RemasterBindingResolution::Unresolved),
                        }
                    } else if let Some(key) = &value.explicit_key {
                        if records.contains_key(key) {
                            RemasterBindingResolution::Current {
                                record_key: key.clone(),
                                owners: vec![],
                            }
                        } else {
                            RemasterBindingResolution::MissingExplicitKey {
                                record_key: key.clone(),
                            }
                        }
                    } else {
                        RemasterBindingResolution::Unresolved
                    };
                    result.push(RemasterBinding {
                        declaration: declaration.id.clone(),
                        side,
                        group,
                        reference,
                        resolution,
                    });
                }
            }
        }
    }
    Ok(result)
}
