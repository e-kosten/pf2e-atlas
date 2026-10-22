//! Exact source references within the shared query expression.
use crate::{
    OwnedContentLocator, QueryError, QueryExpression, QueryLimits, QueryPredicate, RecordKey,
    SourcePassageAddress,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum QueryRelationshipDirection {
    Incoming,
    Outgoing,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QueryRecordSelection {
    pub record_key: RecordKey,
    #[serde(default, rename = "embedded_path")]
    pub owners: Vec<OwnedContentLocator>,
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub passage: Option<SourcePassageAddress>,
    #[serde(default)]
    pub source_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spell_rank: Option<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QueryRelationship {
    pub direction: QueryRelationshipDirection,
    pub selection: QueryRecordSelection,
    pub include_contents: bool,
}
fn error(message: &str) -> QueryError {
    QueryError {
        code: "relationship_clause".into(),
        message: message.into(),
        field: None,
        clause_id: None,
        source_span: None,
    }
}
fn visit<'a>(
    p: &'a QueryPredicate,
    depth: usize,
    nodes: &mut usize,
    f: &mut impl FnMut(&'a QueryPredicate),
) -> Result<(), QueryError> {
    *nodes += 1;
    if depth > QueryLimits::default().depth || *nodes > QueryLimits::default().nodes {
        return Err(error("Query exceeds node or depth bounds"));
    }
    f(p);
    match &p.expression {
        QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
            for c in children {
                visit(c, depth + 1, nodes, f)?;
            }
        }
        QueryExpression::Not { predicate } | QueryExpression::Exists { predicate, .. } => {
            visit(predicate, depth + 1, nodes, f)?
        }
        _ => {}
    }
    Ok(())
}
/// Fill missing relationship IDs deterministically without replacing caller-owned IDs.
pub fn assign_relationship_clause_ids(p: &mut QueryPredicate) -> Result<(), QueryError> {
    let mut used = BTreeSet::new();
    visit(p, 0, &mut 0, &mut |n| {
        if let Some(id) = &n.clause_id {
            used.insert(id.clone());
        }
    })?;
    fn assign(p: &mut QueryPredicate, used: &mut BTreeSet<String>, next: &mut usize) {
        if matches!(p.expression, QueryExpression::Relationship { .. }) && p.clause_id.is_none() {
            loop {
                let id = format!("relationship-{next}");
                *next += 1;
                if used.insert(id.clone()) {
                    p.clause_id = Some(id);
                    break;
                }
            }
        }
        match &mut p.expression {
            QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
                for c in children {
                    assign(c, used, next);
                }
            }
            QueryExpression::Not { predicate } | QueryExpression::Exists { predicate, .. } => {
                assign(predicate, used, next)
            }
            _ => {}
        }
    }
    assign(p, &mut used, &mut 0);
    Ok(())
}
/// Select one structurally positive link clause. Actual matching proof is checked by retrieval.
pub fn select_relationship_clause_id(
    p: &QueryPredicate,
    requested: Option<&str>,
) -> Result<String, QueryError> {
    let mut leaves = Vec::new();
    fn collect<'a>(p: &'a QueryPredicate, positive: bool, leaves: &mut Vec<(&'a str, bool)>) {
        match &p.expression {
            QueryExpression::Relationship { .. } => {
                if let Some(id) = &p.clause_id {
                    leaves.push((id, positive));
                }
            }
            QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
                for c in children {
                    collect(c, positive, leaves);
                }
            }
            QueryExpression::Not { predicate } => collect(predicate, !positive, leaves),
            _ => {}
        }
    }
    visit(p, 0, &mut 0, &mut |_| {})?;
    collect(p, true, &mut leaves);
    if let Some(id) = requested {
        return match leaves.iter().find(|(candidate, _)| *candidate == id) {
            Some((_, true)) => Ok(id.into()),
            _ => Err(error("Choose a positive link condition from this filter")),
        };
    }
    let positive = leaves
        .into_iter()
        .filter(|(_, positive)| *positive)
        .collect::<Vec<_>>();
    if positive.len() != 1 {
        return Err(error("Choose one positive link condition"));
    }
    Ok(positive[0].0.into())
}

/// Bounded source-order presentation inventory, never an independent eligibility scope.
pub fn relationship_clauses(
    p: &QueryPredicate,
) -> Result<Vec<(&str, &QueryRelationship)>, QueryError> {
    let mut clauses = Vec::new();
    let mut invalid = false;
    let mut ids = BTreeSet::new();
    visit(p, 0, &mut 0, &mut |node| {
        if let QueryExpression::Relationship { relationship } = &node.expression {
            if let Some(id) = &node.clause_id {
                if id.is_empty() || id.len() > 128 || !ids.insert(id.as_str()) {
                    invalid = true;
                }
                clauses.push((id.as_str(), relationship));
            } else {
                invalid = true;
            }
        }
    })?;
    if invalid {
        return Err(error(
            "Link conditions require distinct nonempty clause IDs",
        ));
    }
    if clauses.len() > QueryLimits::default().relationship_leaves {
        return Err(error("Use at most 16 link conditions"));
    }
    Ok(clauses)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leaf(id: Option<&str>) -> QueryPredicate {
        QueryPredicate {
            clause_id: id.map(str::to_owned),
            expression: QueryExpression::Relationship {
                relationship: QueryRelationship {
                    direction: QueryRelationshipDirection::Incoming,
                    selection: QueryRecordSelection {
                        record_key: RecordKey::parse("items:aaaaaaaaaaaaaaaa").unwrap(),
                        owners: vec![],
                        field: None,
                        passage: None,
                        source_fingerprint: None,
                        spell_rank: None,
                    },
                    include_contents: true,
                },
            },
        }
    }
    #[test]
    fn allocation_preserves_provided_ids_and_avoids_collisions_across_branches() {
        let mut predicate = QueryPredicate::new(QueryExpression::AnyOf {
            children: vec![
                leaf(Some("relationship-0")),
                leaf(None),
                QueryPredicate::new(QueryExpression::Not {
                    predicate: Box::new(leaf(None)),
                }),
            ],
        });
        assign_relationship_clause_ids(&mut predicate).unwrap();
        assert_eq!(
            relationship_clauses(&predicate)
                .unwrap()
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            ["relationship-0", "relationship-1", "relationship-2"]
        );
        assert!(select_relationship_clause_id(&predicate, None).is_err());
        assert_eq!(
            select_relationship_clause_id(&predicate, Some("relationship-1")).unwrap(),
            "relationship-1"
        );
        assert!(select_relationship_clause_id(&predicate, Some("relationship-2")).is_err());
        let original = predicate.clone();
        assign_relationship_clause_ids(&mut predicate).unwrap();
        assert_eq!(predicate, original);
    }
    #[test]
    fn repeated_relationships_require_distinct_clause_identity_and_obey_leaf_budget() {
        let duplicate = QueryPredicate::new(QueryExpression::AllOf {
            children: vec![leaf(Some("same")), leaf(Some("same"))],
        });
        assert!(relationship_clauses(&duplicate).is_err());
        let mut predicate = QueryPredicate::new(QueryExpression::AllOf {
            children: (0..17).map(|_| leaf(None)).collect(),
        });
        assign_relationship_clause_ids(&mut predicate).unwrap();
        assert!(relationship_clauses(&predicate).is_err());
    }
}
