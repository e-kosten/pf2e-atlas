//! Full relationship eligibility is independent of occurrence response budgets.
use crate::{IndexError, SourceRelationshipDirection, SourceStoredRelationship, SqliteIndexReader};
use atlas_domain::RecordKey;
use atlas_record::source_content::OwnedContentLocator;
use rusqlite::{params_from_iter, types::Value};

#[derive(Debug, Clone)]
pub struct RelationshipConstraint {
    pub direction: SourceRelationshipDirection,
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub include_contents: bool,
}
pub struct RelationshipOccurrencePageRequest<'a> {
    pub constraint: &'a RelationshipConstraint,
    pub result: &'a RecordKey,
    pub offset: usize,
    pub limit: usize,
}
pub struct RelationshipOccurrencePage {
    pub occurrences: Vec<SourceStoredRelationship>,
    pub total: u64,
}

/// Predicate over an occurrence alias `o`; exact target ownership is never broadened.
fn occurrence_scope(
    constraint: &RelationshipConstraint,
    source_alias: &str,
) -> Result<(String, Vec<Value>), IndexError> {
    let owners = serde_json::to_string(&constraint.owners)?;
    let mut values = vec![Value::Text(constraint.record.to_string())];
    let mut sql = match constraint.direction {
        SourceRelationshipDirection::Incoming => {
            values.push(Value::Text(owners.clone()));
            "t.key=? AND COALESCE(o.target_owners_json,'[]')=?".to_owned()
        }
        SourceRelationshipDirection::Outgoing => {
            values.push(Value::Text(owners.clone()));
            if constraint.include_contents {
                // JSON array element equality preserves exact source-address components.
                format!(
                    "{source_alias}.key=? AND json_array_length(o.owners_json)>=json_array_length(?) AND NOT EXISTS(SELECT 1 FROM json_each(?) selected WHERE json_extract(o.owners_json,'$['||selected.key||']')<>selected.value)"
                )
            } else {
                format!("{source_alias}.key=? AND o.owners_json=?")
            }
        }
    };
    match constraint.direction {
        SourceRelationshipDirection::Incoming => {
            if constraint.field.is_some() {
                return Err(IndexError::Invalid(
                    "incoming references do not select a destination field".into(),
                ));
            }
            if !constraint.include_contents {
                sql.push_str(" AND o.owners_json='[]'");
            }
        }
        SourceRelationshipDirection::Outgoing => {
            if constraint.include_contents {
                values.push(Value::Text(owners.clone()));
            }
            if let Some(field) = &constraint.field {
                sql.push_str(" AND o.field_path=?");
                values.push(Value::Text(field.clone()));
            }
        }
    }
    Ok((sql, values))
}

pub(crate) fn eligibility(
    constraint: &RelationshipConstraint,
) -> Result<(String, Vec<Value>), IndexError> {
    let (scope, values) = occurrence_scope(constraint, "s")?;
    let candidate = match constraint.direction {
        SourceRelationshipDirection::Incoming => "o.record_id",
        SourceRelationshipDirection::Outgoing => "o.target_record_id",
    };
    Ok((
        format!(
            "r.record_id IN (SELECT {candidate} FROM relationship_occurrences o JOIN records s ON s.record_id=o.record_id JOIN records t ON t.record_id=o.target_record_id WHERE s.document_kind<>'Macro' AND t.document_kind<>'Macro' AND o.resolution IN ('resolved_record','resolved_owned') AND {scope})"
        ),
        values,
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
        let (scope, mut values) = occurrence_scope(request.constraint, "r")?;
        let candidate = match request.constraint.direction {
            SourceRelationshipDirection::Incoming => "r.key=?",
            SourceRelationshipDirection::Outgoing => "t.key=?",
        };
        values.push(Value::Text(request.result.to_string()));
        let where_sql = format!(
            "r.document_kind<>'Macro' AND t.document_kind<>'Macro' AND o.resolution IN ('resolved_record','resolved_owned') AND {scope} AND {candidate}"
        );
        let joins = "FROM relationship_occurrences o JOIN records r ON r.record_id=o.record_id JOIN records t ON t.record_id=o.target_record_id";
        let c = self.native()?;
        self.count_statement();
        let total: u64 = c.query_row(
            &format!("SELECT count(*) {joins} WHERE {where_sql}"),
            params_from_iter(values.iter()),
            |row| row.get(0),
        )?;
        values.push(Value::Integer(request.limit as i64));
        values.push(Value::Integer(
            i64::try_from(request.offset).map_err(|e| IndexError::Invalid(e.to_string()))?,
        ));
        // Reuse the checked occurrence decoder and its exact provenance fields.
        let select = crate::read_content::RELATIONSHIP_SELECT;
        self.count_statement();
        let mut statement = c.prepare(&format!(
            "{select} WHERE {where_sql} ORDER BY o.occurrence_id LIMIT ? OFFSET ?"
        ))?;
        let occurrences = statement
            .query_map(
                params_from_iter(values.iter()),
                crate::read_content::relationship_row,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RelationshipOccurrencePage { occurrences, total })
    }
}
