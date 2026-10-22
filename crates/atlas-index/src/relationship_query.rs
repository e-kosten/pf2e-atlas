//! Relationship leaves share the query compiler's parameters and exact owner semantics.
use crate::query::compiler::parameter;
use crate::{IndexError, SourceStoredRelationship, SqliteIndexReader};
use atlas_domain::{QueryRelationship, QueryRelationshipDirection, RecordKey};
use rusqlite::{params_from_iter, types::Value};

pub struct RelationshipOccurrencePageRequest<'a> {
    pub relationship: &'a QueryRelationship,
    pub result: &'a RecordKey,
    pub offset: usize,
    pub limit: usize,
}
pub struct RelationshipOccurrencePage {
    pub occurrences: Vec<SourceStoredRelationship>,
    pub total: u64,
}
fn occurrence_scope(
    relationship: &QueryRelationship,
    source_alias: &str,
    params: &mut Vec<Value>,
) -> Result<String, IndexError> {
    let selection = &relationship.selection;
    let key = parameter(params, Value::Text(selection.record_key.to_string()));
    let owners = serde_json::to_string(&selection.owners)?;
    let owner = parameter(params, Value::Text(owners.clone()));
    let mut sql = match relationship.direction {
        QueryRelationshipDirection::Incoming => {
            format!("t.key={key} AND COALESCE(o.target_owners_json,'[]')={owner}")
        }
        QueryRelationshipDirection::Outgoing => {
            if relationship.include_contents {
                let prefix = parameter(params, Value::Text(owners));
                format!(
                    "{source_alias}.key={key} AND json_array_length(o.owners_json)>=json_array_length({owner}) AND NOT EXISTS(SELECT 1 FROM json_each({prefix}) selected WHERE json_extract(o.owners_json,'$['||selected.key||']')<>selected.value)"
                )
            } else {
                format!("{source_alias}.key={key} AND o.owners_json={owner}")
            }
        }
    };
    match relationship.direction {
        QueryRelationshipDirection::Incoming => {
            if !relationship.include_contents {
                sql.push_str(" AND o.owners_json='[]'");
            }
        }
        QueryRelationshipDirection::Outgoing => {
            if let Some(field) = &selection.field {
                let value = parameter(params, Value::Text(field.clone()));
                sql.push_str(&format!(" AND o.field_path={value}"));
            }
        }
    }
    Ok(sql)
}
pub(crate) fn compile_relationship(
    relationship: &QueryRelationship,
    params: &mut Vec<Value>,
) -> Result<String, IndexError> {
    let scope = occurrence_scope(relationship, "s", params)?;
    let candidate = match relationship.direction {
        QueryRelationshipDirection::Incoming => "o.record_id",
        QueryRelationshipDirection::Outgoing => "o.target_record_id",
    };
    Ok(format!(
        "r.record_id IN (SELECT {candidate} FROM relationship_occurrences o JOIN records s ON s.record_id=o.record_id JOIN records t ON t.record_id=o.target_record_id WHERE s.document_kind<>'Macro' AND t.document_kind<>'Macro' AND o.resolution IN ('resolved_record','resolved_owned') AND {scope})"
    ))
}
impl SqliteIndexReader {
    pub fn relationship_occurrence_page(
        &self,
        request: RelationshipOccurrencePageRequest<'_>,
    ) -> Result<RelationshipOccurrencePage, IndexError> {
        if !(1..=100).contains(&request.limit) {
            return Err(IndexError::Invalid(
                "relationship evidence page must contain 1..=100 occurrences".into(),
            ));
        }
        let mut params = Vec::new();
        let scope = occurrence_scope(request.relationship, "r", &mut params)?;
        let result = parameter(&mut params, Value::Text(request.result.to_string()));
        let candidate = match request.relationship.direction {
            QueryRelationshipDirection::Incoming => format!("r.key={result}"),
            QueryRelationshipDirection::Outgoing => format!("t.key={result}"),
        };
        let where_sql = format!(
            "r.document_kind<>'Macro' AND t.document_kind<>'Macro' AND o.resolution IN ('resolved_record','resolved_owned') AND {scope} AND {candidate}"
        );
        let joins = "FROM relationship_occurrences o JOIN records r ON r.record_id=o.record_id JOIN records t ON t.record_id=o.target_record_id";
        self.count_statement();
        let total = self.native()?.query_row(
            &format!("SELECT count(*) {joins} WHERE {where_sql}"),
            params_from_iter(params.iter()),
            |r| r.get(0),
        )?;
        let limit = parameter(&mut params, Value::Integer(request.limit as i64));
        let offset = parameter(
            &mut params,
            Value::Integer(
                i64::try_from(request.offset).map_err(|e| IndexError::Invalid(e.to_string()))?,
            ),
        );
        self.count_statement();
        let mut statement = self.native()?.prepare(&format!(
            "{} WHERE {where_sql} ORDER BY o.occurrence_id LIMIT {limit} OFFSET {offset}",
            crate::read_content::RELATIONSHIP_SELECT
        ))?;
        let occurrences = statement
            .query_map(
                params_from_iter(params.iter()),
                crate::read_content::relationship_row,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RelationshipOccurrencePage { occurrences, total })
    }
    /// Positive link leaves participating in successful Boolean proofs, using the compiled SQL truth.
    pub fn relationship_proofs(
        &self,
        query: &crate::ValidatedQuery,
        keys: &[RecordKey],
    ) -> Result<Vec<(RecordKey, String)>, IndexError> {
        let mut compiled = crate::query::compiler::compile_predicate(query)?;
        if compiled.proofs.is_empty() || keys.is_empty() {
            return Ok(vec![]);
        }
        let requested = parameter(
            &mut compiled.parameters,
            Value::Text(serde_json::to_string(
                &keys.iter().map(ToString::to_string).collect::<Vec<_>>(),
            )?),
        );
        let columns = compiled
            .proofs
            .iter()
            .map(|(_, proof)| format!("({proof}) IS TRUE"))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT r.key,{columns} FROM records r JOIN json_each({requested}) requested ON requested.value=r.key WHERE ({}) IS TRUE",
            compiled.expression
        );
        self.count_statement();
        let mut statement = self.native()?.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter(compiled.parameters.iter()), |row| {
                let key = row.get::<_, String>(0)?;
                let ids = compiled
                    .proofs
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (id, _))| match row.get::<_, bool>(i + 1) {
                        Ok(true) => Some(Ok(id.clone())),
                        Ok(false) => None,
                        Err(e) => Some(Err(e)),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((key, ids))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut proofs = Vec::new();
        for (key, ids) in rows {
            let key = crate::read_units::parse_key(key)?;
            for id in ids {
                proofs.push((key.clone(), id));
            }
        }
        Ok(proofs)
    }
}
