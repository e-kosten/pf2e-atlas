//! Reviewed source-copy declarations. Hash drift requires another source review.
use crate::IngestError;
use atlas_domain::{RecordKey, normalize_record_name};
use atlas_index::{
    PublicationSpool, SourceAliasInput, SourceArtifactDiagnostic, VerifiedDuplicateInput,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const CATALOG: &str = include_str!("../verified-duplicates/creatures.json");
pub(crate) const RECIPE: &str = "reviewed-source-duplicates/v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedRecord {
    key: RecordKey,
    name: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DuplicateReview {
    canonical: ExpectedRecord,
    duplicate: ExpectedRecord,
    remaster: bool,
    source_note: String,
}

pub(crate) struct SourceFacts {
    pub(crate) name: String,
    pub(crate) sha256: String,
    pub(crate) remaster: Option<bool>,
    pub(crate) npc: bool,
}

impl DuplicateReview {
    pub(crate) fn keys(&self) -> [&RecordKey; 2] {
        [&self.duplicate.key, &self.canonical.key]
    }

    fn matches(&self, record: &ExpectedRecord, facts: &SourceFacts) -> bool {
        facts.npc
            && facts.remaster == Some(self.remaster)
            && facts.name == record.name
            && facts.sha256 == record.sha256
    }
}

pub(crate) fn catalog() -> Result<Vec<DuplicateReview>, IngestError> {
    let reviews: Vec<DuplicateReview> = serde_json::from_str(CATALOG).map_err(|err| {
        IngestError::SourceSelectionFailed(format!("verified duplicate catalog: {err}"))
    })?;
    let mut copies = BTreeSet::new();
    for review in &reviews {
        if !copies.insert(&review.duplicate.key)
            || review.source_note.trim().is_empty()
            || [&review.duplicate, &review.canonical].iter().any(|record| {
                record.name.trim().is_empty()
                    || record.sha256.len() != 64
                    || !record.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            })
        {
            return Err(IngestError::SourceSelectionFailed(
                "invalid verified duplicate declaration".into(),
            ));
        }
    }
    if reviews
        .iter()
        .any(|review| copies.contains(&review.canonical.key))
    {
        return Err(IngestError::SourceSelectionFailed(
            "verified duplicates must point directly to a canonical record".into(),
        ));
    }
    Ok(reviews)
}

pub(crate) fn prepare(
    reviews: Vec<DuplicateReview>,
    facts: &BTreeMap<RecordKey, SourceFacts>,
    active: &BTreeSet<RecordKey>,
    spool: &mut PublicationSpool,
    aliases: &mut Vec<SourceAliasInput>,
) -> Result<(), atlas_index::IndexError> {
    let mut pairs = Vec::new();
    for review in reviews {
        let available = |expected: &ExpectedRecord| {
            active.contains(&expected.key)
                && facts
                    .get(&expected.key)
                    .is_some_and(|facts| review.matches(expected, facts))
        };
        if !available(&review.duplicate) || !available(&review.canonical) {
            let message = format!(
                "Verified duplicate {} → {} is inactive. A source record is missing or differs from its reviewed identity, edition, or bytes.",
                review.duplicate.key, review.canonical.key,
            );
            tracing::warn!("{message}");
            if let Some(key) = review.keys().into_iter().find(|key| active.contains(*key)) {
                spool.duplicate_diagnostics.push((
                    key.clone(),
                    SourceArtifactDiagnostic {
                        source_diagnostic: None,
                        owners: None,
                        field: None,
                        stage: "duplicate_review".into(),
                        code: "inactive_verified_duplicate".into(),
                        message,
                    },
                ));
            }
            continue;
        }
        if normalize_record_name(&review.duplicate.name)
            != normalize_record_name(&review.canonical.name)
            && !aliases.iter().any(|alias| {
                alias.record == review.canonical.key && alias.alias == review.duplicate.name
            })
        {
            aliases.push(SourceAliasInput {
                record: review.canonical.key.clone(),
                alias: review.duplicate.name,
                evidence: atlas_index::VERIFIED_DUPLICATE_ALIAS_EVIDENCE.into(),
            });
        }
        pairs.push(VerifiedDuplicateInput {
            duplicate: review.duplicate.key,
            canonical: review.canonical.key,
            duplicate_sha256: review.duplicate.sha256,
            canonical_sha256: review.canonical.sha256,
        });
    }
    spool.set_verified_duplicates(pairs)
}
