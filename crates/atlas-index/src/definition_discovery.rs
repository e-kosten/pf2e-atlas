//! Artifact-scoped glossary choices, independent of matching-record vocabulary.
use crate::{
    IndexError, ReferenceAvailability, ReferenceCoverage, ReferenceSummary, SqliteIndexReader,
};
use atlas_domain::{
    DefinitionKey, QueryDefinitionReferenceTarget, QueryRecordSelection, RecordKey,
    SourceRecordSummary,
};
use rusqlite::{OptionalExtension, params};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct DefinitionFilterOptionsRequest {
    pub text: Option<String>,
    pub offset: usize,
    pub limit: usize,
    pub selected: Vec<QueryDefinitionReferenceTarget>,
}
#[derive(Debug, Clone)]
pub enum DefinitionFilterOption {
    Definition { summary: Box<ReferenceSummary> },
    Record { summary: Box<SourceRecordSummary> },
}
impl DefinitionFilterOption {
    pub fn target(&self) -> QueryDefinitionReferenceTarget {
        match self {
            Self::Definition { summary } => QueryDefinitionReferenceTarget::Definition {
                key: summary.key.clone(),
            },
            Self::Record { summary } => QueryDefinitionReferenceTarget::Record {
                selection: Box::new(QueryRecordSelection {
                    record_key: summary.key.clone(),
                    owners: vec![],
                    field: None,
                    passage: None,
                    source_fingerprint: None,
                    spell_rank: None,
                }),
            },
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionFilterSelectionAvailability {
    Available,
    Missing,
    NotInCatalog,
    SourceInapplicable,
}
#[derive(Debug, Clone)]
pub struct DefinitionFilterSelection {
    pub target: QueryDefinitionReferenceTarget,
    pub option: Option<DefinitionFilterOption>,
    pub availability: DefinitionFilterSelectionAvailability,
}
#[derive(Debug, Clone)]
pub struct DefinitionFilterOptionsPage {
    pub options: Vec<DefinitionFilterOption>,
    pub selected: Vec<DefinitionFilterSelection>,
    pub total: u64,
}

const OPTIONS: &str = "WITH expected AS (SELECT json_extract(value,'$[0]') AS key,json_extract(value,'$[1]') AS family FROM json_each(?1)), options AS (SELECT 'definition' AS origin,e.key AS identity,json_extract(e.definition_json,'$.label') AS label FROM reference_entries e WHERE json_extract(e.definition_json,'$.availability')<>'source_inapplicable' UNION ALL SELECT 'record' AS origin,r.key AS identity,r.name AS label FROM records r JOIN expected ON expected.key=r.key WHERE r.document_kind='Item' AND r.source_type_state='value' AND r.source_type=expected.family AND r.name_state='value')";

impl SqliteIndexReader {
    pub fn definition_filter_options(
        &self,
        request: DefinitionFilterOptionsRequest,
    ) -> Result<DefinitionFilterOptionsPage, IndexError> {
        if self.reference_coverage() != ReferenceCoverage::Complete {
            return Err(IndexError::Unavailable(
                "glossary filter choices require explicit atlas setup".into(),
            ));
        }
        if !(1..=100).contains(&request.limit)
            || request.offset > 100_000
            || request.selected.len() > 100
            || request
                .text
                .as_ref()
                .is_some_and(|text| text.len() > atlas_domain::QueryLimits::default().source_bytes)
            || serde_json::to_vec(&request.selected)?.len()
                > atlas_domain::QueryLimits::default().source_bytes
        {
            return Err(IndexError::Invalid("glossary discovery requires bounded text, limit 1..100, offset and at most 100 selected identities".into()));
        }
        for target in &request.selected {
            crate::validate_query(&atlas_domain::QueryPredicate {
                clause_id: Some("selected-definition".into()),
                expression: atlas_domain::QueryExpression::DefinitionReference {
                    reference: atlas_domain::QueryDefinitionReference {
                        target: target.clone(),
                        include_contents: true,
                    },
                },
            })?;
        }
        let expected: Vec<_> = atlas_foundry_model::generated::foundry_glossary_records()
            .iter()
            .filter(|(key, _)| {
                key.parse::<RecordKey>().is_ok_and(|key| {
                    crate::reference_pack_applies(
                        self.reference_preparation(),
                        "pf2e",
                        key.pack().as_str(),
                    )
                })
            })
            .collect();
        let expected = serde_json::to_string(&expected)?;
        let text = request.text.as_deref().unwrap_or("");
        let filter =
            "(?2='' OR instr(lower(label),lower(?2))>0 OR instr(lower(identity),lower(?2))>0)";
        self.count_statement();
        let total = self.native()?.query_row(
            &format!("{OPTIONS} SELECT count(*) FROM options WHERE {filter}"),
            params![expected, text],
            |row| row.get(0),
        )?;
        self.count_statement();
        let mut statement = self.native()?.prepare(&format!("{OPTIONS} SELECT origin,identity FROM options WHERE {filter} ORDER BY label COLLATE NOCASE,origin,identity LIMIT ?3 OFFSET ?4"))?;
        let identities = statement
            .query_map(
                params![expected, text, request.limit as i64, request.offset as i64],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        let mut definition_keys = Vec::new();
        let mut record_keys = Vec::new();
        for (origin, identity) in &identities {
            if origin == "definition" {
                definition_keys.push(identity.parse::<DefinitionKey>().map_err(|e| {
                    IndexError::Invalid(format!("invalid glossary definition identity: {e}"))
                })?);
            } else {
                record_keys.push(identity.parse::<RecordKey>().map_err(|e| {
                    IndexError::Invalid(format!("invalid glossary record identity: {e}"))
                })?);
            }
        }
        let mut definitions: std::collections::BTreeMap<_, _> = self
            .reference_summaries(&definition_keys)?
            .entries
            .into_iter()
            .map(|summary| (summary.key.to_string(), summary))
            .collect();
        let mut records: std::collections::BTreeMap<_, _> = self
            .read_summaries(&record_keys)?
            .records
            .into_iter()
            .map(|summary| (summary.key.to_string(), summary))
            .collect();
        let mut options = Vec::with_capacity(identities.len());
        for (origin, identity) in identities {
            let option = if origin == "definition" {
                definitions
                    .remove(&identity)
                    .map(|summary| DefinitionFilterOption::Definition {
                        summary: Box::new(summary),
                    })
            } else {
                records
                    .remove(&identity)
                    .map(|summary| DefinitionFilterOption::Record {
                        summary: Box::new(summary),
                    })
            };
            options.push(option.ok_or_else(|| {
                IndexError::Invalid("selected glossary choice is unavailable".into())
            })?);
        }
        let mut seen = BTreeSet::new();
        let mut selected = Vec::new();
        for target in request.selected {
            if !seen.insert(serde_json::to_string(&target)?) {
                continue;
            }
            let (option, availability) = match &target {
                QueryDefinitionReferenceTarget::Definition { key } => match self
                    .reference_summary(key)?
                {
                    Some(summary)
                        if summary.availability == ReferenceAvailability::SourceInapplicable =>
                    {
                        (
                            None,
                            DefinitionFilterSelectionAvailability::SourceInapplicable,
                        )
                    }
                    Some(summary) => (
                        Some(DefinitionFilterOption::Definition {
                            summary: Box::new(summary),
                        }),
                        DefinitionFilterSelectionAvailability::Available,
                    ),
                    None => (None, DefinitionFilterSelectionAvailability::Missing),
                },
                QueryDefinitionReferenceTarget::Record { selection } => {
                    let Some(summary) = self.read_summary(&selection.record_key)? else {
                        selected.push(DefinitionFilterSelection {
                            target,
                            option: None,
                            availability: DefinitionFilterSelectionAvailability::Missing,
                        });
                        continue;
                    };
                    let catalogued = if selection.owners.is_empty()
                        && selection.field.is_none()
                        && selection.passage.is_none()
                        && selection.spell_rank.is_none()
                    {
                        self.count_statement();
                        self.native()?.query_row(&format!("{OPTIONS} SELECT 1 FROM options WHERE origin='record' AND identity=?2"), params![expected,selection.record_key.to_string()], |_|Ok(true)).optional()?.unwrap_or(false)
                    } else {
                        false
                    };
                    if catalogued {
                        (
                            Some(DefinitionFilterOption::Record {
                                summary: Box::new(summary),
                            }),
                            DefinitionFilterSelectionAvailability::Available,
                        )
                    } else {
                        (
                            Some(DefinitionFilterOption::Record {
                                summary: Box::new(summary),
                            }),
                            DefinitionFilterSelectionAvailability::NotInCatalog,
                        )
                    }
                }
            };
            selected.push(DefinitionFilterSelection {
                target,
                option,
                availability,
            });
        }
        Ok(DefinitionFilterOptionsPage {
            options,
            selected,
            total,
        })
    }
}
