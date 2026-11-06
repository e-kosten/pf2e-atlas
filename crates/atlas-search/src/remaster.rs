//! Bounded source-backed change information, independent of record identity lookup.
use atlas_domain::{OwnedContentLocator, RecordKey, SourceRecordSummary};
pub use atlas_record::remaster::{RemasterChangeKind, RemasterContext, RemasterOrigin};
use atlas_record::source_content::SourceContentLocator;

pub const DEFAULT_REMASTER_INFORMATION_LIMIT: usize = 20;
pub const MAX_REMASTER_INFORMATION_LIMIT: usize = 100;

pub struct RecordRemasterInformationRequest<'a> {
    pub record_key: &'a RecordKey,
    pub owners: &'a [OwnedContentLocator],
    pub limit: usize,
}
pub struct RemasterTermInformationRequest<'a> {
    pub term: &'a str,
    pub limit: usize,
}
#[derive(Debug, Clone)]
pub struct RemasterInformation {
    pub declarations: Vec<RemasterDeclarationResult>,
    pub truncated: bool,
}
#[derive(Debug, Clone)]
pub struct RecordRemasterInformationResult {
    pub record_key: RecordKey,
    pub seed: Option<SourceRecordSummary>,
    pub information: RemasterInformation,
}
#[derive(Debug, Clone)]
pub struct RemasterTermInformationResult {
    pub term: String,
    pub information: RemasterInformation,
}
#[derive(Debug, Clone)]
pub struct RemasterDeclarationResult {
    pub id: String,
    pub kinds: Vec<RemasterChangeKind>,
    pub authored_status: Option<String>,
    pub context: RemasterContext,
    pub before: Vec<RemasterSideResult>,
    pub after: Vec<RemasterSideResult>,
    pub citations: Vec<RemasterCitationResult>,
}
#[derive(Debug, Clone)]
pub struct RemasterSideResult {
    pub text: String,
    pub terms: Vec<String>,
    pub endpoints: Vec<RemasterEndpointResult>,
}
#[derive(Debug, Clone)]
pub struct RemasterEndpointResult {
    pub label: String,
    pub reference: Option<String>,
    pub resolution: RemasterEndpointResolution,
}
#[derive(Debug, Clone)]
pub enum RemasterEndpointResolution {
    Current {
        record: Box<SourceRecordSummary>,
        owners: Vec<OwnedContentLocator>,
    },
    MissingExplicitKey {
        record_key: RecordKey,
    },
    Unresolved {
        reference: Option<String>,
    },
    Ambiguous {
        candidates: Vec<RemasterEndpointCandidate>,
        truncated: bool,
    },
    Term,
    SourceText,
}
#[derive(Debug, Clone)]
pub struct RemasterEndpointCandidate {
    pub record: SourceRecordSummary,
    pub owners: Vec<OwnedContentLocator>,
}
#[derive(Debug, Clone)]
pub struct RemasterCitationResult {
    pub origin: RemasterOrigin,
    pub attribution: String,
    pub revision: String,
    pub statement: String,
    pub source: RemasterCitationSource,
}
#[derive(Debug, Clone)]
pub enum RemasterCitationSource {
    Journal { locator: SourceContentLocator },
    CapturedFile { path: String },
}
#[derive(Debug, Clone)]
pub struct RemasterLookupSupport {
    pub declarations: Vec<RemasterDeclarationResult>,
    pub truncated: bool,
}
#[derive(Debug, Clone)]
pub struct RemasterAliasMatch {
    pub alias: String,
    pub support: RemasterLookupSupport,
}

use crate::{AtlasRetrievalService, SearchError};
use atlas_record::remaster::{RemasterBindingResolution, RemasterSideKind};
use std::collections::{BTreeMap, BTreeSet};

impl AtlasRetrievalService {
    pub fn record_remaster_information(
        &self,
        request: RecordRemasterInformationRequest<'_>,
    ) -> Result<RecordRemasterInformationResult, SearchError> {
        check_limit(request.limit)?;
        let seed = self.index.read_summary(request.record_key)?;
        // Root-key information also describes explicitly named missing records. An
        // owned selection, however, must still exist at its exact checked address.
        if !request.owners.is_empty() {
            if seed.is_none() {
                return Err(SearchError::invalid_search_options(
                    "An unavailable record cannot supply an owned selection",
                ));
            }
            let source = self
                .index
                .read_source_record(request.record_key)?
                .ok_or_else(|| {
                    SearchError::artifact_contract_violation("record summary has no checked source")
                })?;
            if source.node_at(request.owners).is_none() {
                return Err(SearchError::invalid_search_options(
                    "The selected entry is unavailable. Open the record again.",
                ));
            }
        }
        let information = self.index.remaster_record_information(
            request.record_key,
            request.owners,
            request.limit,
        )?;
        let summaries = self.remaster_summaries(std::iter::once(&information))?;
        Ok(RecordRemasterInformationResult {
            record_key: request.record_key.clone(),
            seed,
            information: project_information(information, &summaries)?,
        })
    }

    pub fn remaster_term_information(
        &self,
        request: RemasterTermInformationRequest<'_>,
    ) -> Result<RemasterTermInformationResult, SearchError> {
        check_limit(request.limit)?;
        if request.term.trim().is_empty() || request.term.chars().count() > 256 {
            return Err(SearchError::invalid_search_options(
                "Remaster terms must contain 1..=256 characters",
            ));
        }
        let information = self
            .index
            .remaster_term_information(request.term, request.limit)?;
        let summaries = self.remaster_summaries(std::iter::once(&information))?;
        Ok(RemasterTermInformationResult {
            term: atlas_domain::normalize_record_name(request.term),
            information: project_information(information, &summaries)?,
        })
    }

    /// Hydrate only aliases actually delivered as identity or lexical witnesses.
    /// Source/support reads and current summaries are batched across the page.
    pub(crate) fn remaster_alias_support(
        &self,
        aliases: &[(RecordKey, String)],
    ) -> Result<BTreeMap<(RecordKey, String), RemasterLookupSupport>, SearchError> {
        if aliases.is_empty() {
            return Ok(BTreeMap::new());
        }
        let aliases = aliases
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let mut information = BTreeMap::new();
        for batch in aliases.chunks(256) {
            information.extend(self.index.remaster_alias_information(batch, 20)?);
        }
        if aliases.iter().any(|alias| {
            information
                .get(alias)
                .is_none_or(|support| support.declarations.is_empty())
        }) {
            return Err(SearchError::artifact_contract_violation(
                "matched remaster alias has no supporting declaration",
            ));
        }
        let summaries = self.remaster_summaries(information.values())?;
        information
            .into_iter()
            .map(|(key, information)| {
                let information = project_information(information, &summaries)?;
                Ok((
                    key,
                    RemasterLookupSupport {
                        declarations: information.declarations,
                        truncated: information.truncated,
                    },
                ))
            })
            .collect()
    }

    fn remaster_summaries<'a>(
        &self,
        information: impl Iterator<Item = &'a atlas_index::SourceRemasterInformation>,
    ) -> Result<BTreeMap<RecordKey, SourceRecordSummary>, SearchError> {
        let keys = information
            .flat_map(|information| &information.declarations)
            .flat_map(|declaration| &declaration.bindings)
            .flat_map(|binding| match &binding.resolution {
                RemasterBindingResolution::Current { record_key, .. } => vec![record_key.clone()],
                RemasterBindingResolution::Ambiguous { candidates, .. } => {
                    candidates.iter().map(|(key, _)| key.clone()).collect()
                }
                _ => vec![],
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        Ok(self
            .summaries(&keys)?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect())
    }
}

fn check_limit(limit: usize) -> Result<(), SearchError> {
    if !(1..=MAX_REMASTER_INFORMATION_LIMIT).contains(&limit) {
        return Err(SearchError::invalid_search_options(
            "Remaster information limit must be between 1 and 100",
        ));
    }
    Ok(())
}

fn project_information(
    information: atlas_index::SourceRemasterInformation,
    summaries: &BTreeMap<RecordKey, SourceRecordSummary>,
) -> Result<RemasterInformation, SearchError> {
    let declarations = information
        .declarations
        .into_iter()
        .map(|source| {
            let declaration = source.declaration;
            let side = |side_kind, values| {
                project_sides(
                    values,
                    side_kind,
                    &source.bindings,
                    &declaration.context,
                    &declaration.kinds,
                    summaries,
                )
            };
            let before = side(RemasterSideKind::Before, declaration.before)?;
            let after = side(RemasterSideKind::After, declaration.after)?;
            let citations = source
                .citations
                .into_iter()
                .map(|citation| {
                    let source = match (citation.locator, citation.path) {
                        (Some(locator), None) => RemasterCitationSource::Journal { locator },
                        (None, Some(path)) => RemasterCitationSource::CapturedFile { path },
                        _ => {
                            return Err(SearchError::artifact_contract_violation(
                                "remaster citation must have one source destination",
                            ));
                        }
                    };
                    Ok(RemasterCitationResult {
                        origin: citation.origin,
                        attribution: citation.attribution,
                        revision: citation.revision,
                        statement: citation.statement,
                        source,
                    })
                })
                .collect::<Result<Vec<_>, SearchError>>()?;
            Ok(RemasterDeclarationResult {
                id: declaration.id,
                kinds: declaration.kinds,
                authored_status: declaration.authored_status,
                context: declaration.context,
                before,
                after,
                citations,
            })
        })
        .collect::<Result<Vec<_>, SearchError>>()?;
    Ok(RemasterInformation {
        declarations,
        truncated: information.truncated,
    })
}

fn project_sides(
    values: Vec<atlas_record::remaster::RemasterSide>,
    side: RemasterSideKind,
    bindings: &[atlas_record::remaster::RemasterBinding],
    context: &RemasterContext,
    kinds: &[RemasterChangeKind],
    summaries: &BTreeMap<RecordKey, SourceRecordSummary>,
) -> Result<Vec<RemasterSideResult>, SearchError> {
    values
        .into_iter()
        .enumerate()
        .map(|(group, value)| {
            let mut endpoints = Vec::new();
            for (reference, authored) in value.references.into_iter().enumerate() {
                let binding = bindings
                    .iter()
                    .find(|binding| {
                        binding.side == side
                            && binding.group == group
                            && binding.reference == reference
                    })
                    .ok_or_else(|| {
                        SearchError::artifact_contract_violation(
                            "remaster reference has no binding",
                        )
                    })?;
                let resolution = match &binding.resolution {
                    RemasterBindingResolution::Current { record_key, owners } => {
                        RemasterEndpointResolution::Current {
                            record: Box::new(summary(summaries, record_key)?),
                            owners: owners.clone(),
                        }
                    }
                    RemasterBindingResolution::MissingExplicitKey { record_key } => {
                        RemasterEndpointResolution::MissingExplicitKey {
                            record_key: record_key.clone(),
                        }
                    }
                    RemasterBindingResolution::Unresolved => {
                        RemasterEndpointResolution::Unresolved {
                            reference: Some(authored.target.clone()),
                        }
                    }
                    RemasterBindingResolution::Ambiguous {
                        candidates,
                        truncated,
                    } => RemasterEndpointResolution::Ambiguous {
                        candidates: candidates
                            .iter()
                            .take(10)
                            .map(|(key, owners)| {
                                Ok(RemasterEndpointCandidate {
                                    record: summary(summaries, key)?,
                                    owners: owners.clone(),
                                })
                            })
                            .collect::<Result<Vec<_>, SearchError>>()?,
                        truncated: *truncated || candidates.len() > 10,
                    },
                };
                let label = if authored.label.is_empty() {
                    match &resolution {
                        RemasterEndpointResolution::Current { record, .. } => record
                            .name
                            .as_value()
                            .cloned()
                            .unwrap_or_else(|| authored.target.clone()),
                        _ => authored.target.clone(),
                    }
                } else {
                    authored.label
                };
                endpoints.push(RemasterEndpointResult {
                    label,
                    reference: Some(authored.target),
                    resolution,
                });
            }
            if endpoints.is_empty() {
                for term in &value.terms {
                    let resolution = if kinds.contains(&RemasterChangeKind::Discussion) {
                        RemasterEndpointResolution::SourceText
                    } else if matches!(
                        context.namespace.as_str(),
                        "language" | "domain" | "pantheon" | "material"
                    ) {
                        RemasterEndpointResolution::Term
                    } else {
                        RemasterEndpointResolution::Unresolved { reference: None }
                    };
                    endpoints.push(RemasterEndpointResult {
                        label: term.clone(),
                        reference: None,
                        resolution,
                    });
                }
            }
            Ok(RemasterSideResult {
                text: value.text,
                terms: value.terms,
                endpoints,
            })
        })
        .collect()
}
fn summary(
    summaries: &BTreeMap<RecordKey, SourceRecordSummary>,
    key: &RecordKey,
) -> Result<SourceRecordSummary, SearchError> {
    summaries.get(key).cloned().ok_or_else(|| {
        SearchError::artifact_contract_violation("remaster current endpoint summary absent")
    })
}
