//! Incoming definition references share ordinary eligibility and occurrence owners.
use crate::query::compiler::parameter;
use crate::{DefinitionReferenceInput, IndexError, SourceStoredRelationship, SqliteIndexReader};
use atlas_domain::{QueryDefinitionReference, QueryDefinitionReferenceTarget, RecordKey};
use rusqlite::{params_from_iter, types::Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum DefinitionReferenceOccurrence {
    Record {
        occurrence: Box<SourceStoredRelationship>,
    },
    Definition {
        occurrence: Box<DefinitionReferenceInput>,
    },
}
pub struct DefinitionReferenceOccurrencePageRequest<'a> {
    pub reference: &'a QueryDefinitionReference,
    pub result: &'a RecordKey,
    pub offset: usize,
    pub limit: usize,
}
pub struct DefinitionReferenceOccurrencePage {
    pub occurrences: Vec<DefinitionReferenceOccurrence>,
    pub total: u64,
}

struct OccurrenceScopes {
    definition: String,
    record: String,
}
fn scopes(
    reference: &QueryDefinitionReference,
    parameters: &mut Vec<Value>,
) -> Result<OccurrenceScopes, IndexError> {
    let (definition, record) = match &reference.target {
        QueryDefinitionReferenceTarget::Record { selection } => {
            let key = parameter(parameters, Value::Text(selection.record_key.to_string()));
            let owners = parameter(
                parameters,
                Value::Text(serde_json::to_string(&selection.owners)?),
            );
            (
                format!(
                    "o.target_record_key={key} AND json_extract(o.evidence,'$.target.owners')={owners}"
                ),
                format!("t.key={key} AND COALESCE(o.target_owners_json,'[]')={owners}"),
            )
        }
        QueryDefinitionReferenceTarget::Definition { key } => {
            let key = parameter(parameters, Value::Text(key.to_string()));
            // Definition identity and its current checked record backing are
            // separate. Only this fixed metadata path selects that destination.
            let mapped = format!(
                "SELECT json_extract(e.definition_json,'$.backing.key') FROM reference_entries e WHERE e.key={key} AND json_extract(e.definition_json,'$.backing.kind')='record'"
            );
            (
                format!(
                    "(o.target_definition_key={key} OR (o.target_record_key=({mapped}) AND json_extract(o.evidence,'$.target.owners')='[]'))"
                ),
                format!("t.key=({mapped}) AND COALESCE(o.target_owners_json,'[]')='[]'"),
            )
        }
    };
    let source_definition = if reference.include_contents {
        ""
    } else {
        " AND json_extract(o.evidence,'$.source.owners')='[]'"
    };
    let source_record = if reference.include_contents {
        ""
    } else {
        " AND o.owners_json='[]'"
    };
    Ok(OccurrenceScopes {
        definition: format!(
            "s.document_kind<>'Macro' AND (t.document_kind IS NULL OR t.document_kind<>'Macro') AND o.visible=1 AND ({definition}){source_definition}"
        ),
        record: format!(
            "s.document_kind<>'Macro' AND t.document_kind<>'Macro' AND o.resolution IN ('resolved_record','resolved_owned') AND ({record}){source_record}"
        ),
    })
}
const DEFINITION_FROM: &str = "FROM definition_reference_occurrences o JOIN records s ON s.key=o.source_record_key LEFT JOIN records t ON t.key=o.target_record_key";
const RECORD_FROM: &str = "FROM relationship_occurrences o JOIN records s ON s.record_id=o.record_id JOIN records t ON t.record_id=o.target_record_id";

pub(crate) fn compile_definition_reference(
    reference: &QueryDefinitionReference,
    parameters: &mut Vec<Value>,
) -> Result<String, IndexError> {
    let scope = scopes(reference, parameters)?;
    Ok(format!(
        "r.record_id IN (SELECT s.record_id {DEFINITION_FROM} WHERE {} UNION SELECT s.record_id {RECORD_FROM} WHERE {})",
        scope.definition, scope.record
    ))
}

impl SqliteIndexReader {
    pub fn definition_reference_occurrence_page(
        &self,
        request: DefinitionReferenceOccurrencePageRequest<'_>,
    ) -> Result<DefinitionReferenceOccurrencePage, IndexError> {
        if !(1..=100).contains(&request.limit) || request.offset > 100_000 {
            return Err(IndexError::Invalid(
                "definition evidence page requires limit 1..100 and bounded offset".into(),
            ));
        }
        let mut parameters = Vec::new();
        let scope = scopes(request.reference, &mut parameters)?;
        let result = parameter(&mut parameters, Value::Text(request.result.to_string()));
        let union = format!(
            "SELECT 'definition' AS origin,o.occurrence_id {DEFINITION_FROM} WHERE {} AND s.key={result} UNION ALL SELECT 'record' AS origin,CAST(o.occurrence_id AS TEXT) {RECORD_FROM} WHERE {} AND s.key={result}",
            scope.definition, scope.record
        );
        self.count_statement();
        let total = self.native()?.query_row(
            &format!("SELECT count(*) FROM ({union})"),
            params_from_iter(parameters.iter()),
            |row| row.get(0),
        )?;
        let limit = parameter(&mut parameters, Value::Integer(request.limit as i64));
        let offset = parameter(&mut parameters, Value::Integer(request.offset as i64));
        self.count_statement();
        let mut statement = self.native()?.prepare(&format!("SELECT origin,occurrence_id FROM ({union}) ORDER BY origin,occurrence_id LIMIT {limit} OFFSET {offset}"))?;
        let selected = statement
            .query_map(params_from_iter(parameters.iter()), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut occurrences = BTreeMap::new();
        let definitions: Vec<&str> = selected
            .iter()
            .filter(|(origin, _)| origin == "definition")
            .map(|(_, id)| id.as_str())
            .collect();
        if !definitions.is_empty() {
            self.count_statement();
            let mut statement = self.native()?.prepare("SELECT occurrence_id,evidence FROM definition_reference_occurrences WHERE occurrence_id IN (SELECT value FROM json_each(?1))")?;
            let rows = statement.query_map([serde_json::to_string(&definitions)?], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (id, json) = row?;
                let occurrence: DefinitionReferenceInput = serde_json::from_str(&json)?;
                if crate::references::reference_id(&occurrence)? != id || !occurrence.visible {
                    return Err(IndexError::Invalid(
                        "invalid selected definition evidence".into(),
                    ));
                }
                occurrences.insert(
                    ("definition".to_owned(), id),
                    DefinitionReferenceOccurrence::Definition {
                        occurrence: Box::new(occurrence),
                    },
                );
            }
        }
        let records: Vec<i64> = selected
            .iter()
            .filter(|(origin, _)| origin == "record")
            .map(|(_, id)| {
                id.parse::<i64>().map_err(|_| {
                    IndexError::Invalid("invalid selected relationship identifier".into())
                })
            })
            .collect::<Result<_, _>>()?;
        for (id, occurrence) in self.relationship_occurrences_by_id(&records)? {
            occurrences.insert(
                ("record".to_owned(), id.to_string()),
                DefinitionReferenceOccurrence::Record {
                    occurrence: Box::new(occurrence),
                },
            );
        }
        let occurrences = selected
            .into_iter()
            .map(|identity| {
                occurrences.remove(&identity).ok_or_else(|| {
                    IndexError::Invalid("selected definition occurrence disappeared".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DefinitionReferenceOccurrencePage { occurrences, total })
    }
}
