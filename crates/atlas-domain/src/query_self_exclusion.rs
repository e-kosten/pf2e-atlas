//! Shared capability policy for counterfactual facet and relationship discovery.
use crate::{FacetUnavailableReason, QueryExpression, QueryFacetOutcome, QueryPredicate};

/// Field facets remove the complete root selection; record pickers remove one clause.
pub enum QuerySelfExclusion<'a> {
    RootField(&'a str),
    Clause(&'a str),
}
#[derive(Debug, Clone)]
pub struct SelfExcludedQuery {
    pub predicate: QueryPredicate,
    pub target: QueryPredicate,
    /// Retains correlation for counts within one authored collection member.
    pub member: Option<QueryPredicate>,
    pub scope: Option<String>,
}
impl QuerySelfExclusion<'_> {
    fn matches(&self, p: &QueryPredicate) -> bool {
        match self {
            Self::Clause(id) => p.clause_id.as_deref() == Some(id),
            Self::RootField(expected) => matches!(&p.expression,
                QueryExpression::Compare { field, .. } | QueryExpression::In { field, .. }
                | QueryExpression::SetMatch { field, .. } | QueryExpression::StateMatch { field, .. }
                if field == expected),
        }
    }
}
/// `None` means no addressed selection exists. Unsupported discovery does not
/// change the executable expression or invent Boolean influence semantics.
pub fn exclude_query_selection(
    predicate: &QueryPredicate,
    selector: QuerySelfExclusion<'_>,
) -> Option<QueryFacetOutcome<SelfExcludedQuery>> {
    struct Walk<'a> {
        selector: QuerySelfExclusion<'a>,
        target: Option<QueryPredicate>,
        member: Option<QueryPredicate>,
        scope: Option<String>,
        unavailable: bool,
    }
    impl Walk<'_> {
        fn visit(
            &mut self,
            p: &QueryPredicate,
            scope: Option<&str>,
            allowed: bool,
        ) -> QueryPredicate {
            if self.selector.matches(p) {
                self.unavailable |= !allowed;
                self.target = Some(p.clone());
                self.scope = scope.map(str::to_owned);
                return QueryPredicate::boolean(true);
            }
            let mut result = p.clone();
            result.expression = match &p.expression {
                QueryExpression::AllOf { children } => QueryExpression::AllOf {
                    children: children
                        .iter()
                        .map(|p| self.visit(p, scope, allowed))
                        .collect(),
                },
                QueryExpression::AnyOf { children } => QueryExpression::AnyOf {
                    children: children
                        .iter()
                        .map(|p| self.visit(p, scope, false))
                        .collect(),
                },
                QueryExpression::Not { predicate } if self.selector.matches(predicate) => {
                    // One directly negated selection is removable; compound NOT is not.
                    self.visit(predicate, scope, allowed && scope.is_none());
                    return QueryPredicate::boolean(true);
                }
                QueryExpression::Not { predicate } => QueryExpression::Not {
                    predicate: Box::new(self.visit(predicate, scope, false)),
                },
                QueryExpression::Exists {
                    collection,
                    scope_id,
                    predicate,
                } if matches!(self.selector, QuerySelfExclusion::Clause(_)) => {
                    let before = self.target.is_some();
                    let child = self.visit(predicate, Some(collection), allowed);
                    if !before && self.target.is_some() {
                        self.member = Some(child.clone());
                    }
                    QueryExpression::Exists {
                        collection: collection.clone(),
                        scope_id: scope_id.clone(),
                        predicate: Box::new(child),
                    }
                }
                expression => expression.clone(),
            };
            result
        }
    }
    let mut walk = Walk {
        selector,
        target: None,
        member: None,
        scope: None,
        unavailable: false,
    };
    let predicate = walk.visit(predicate, None, true);
    let target = walk.target?;
    Some(if walk.unavailable {
        QueryFacetOutcome::Unavailable {
            reason: FacetUnavailableReason::NonConjunctiveContext,
        }
    } else {
        QueryFacetOutcome::Available {
            value: SelfExcludedQuery {
                predicate,
                target,
                member: walk.member,
                scope: walk.scope,
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QueryCompare, QueryLiteral};
    fn field(id: &str, name: &str, value: &str) -> QueryPredicate {
        let mut p = QueryPredicate::new(QueryExpression::Compare {
            field: name.into(),
            op: QueryCompare::Eq,
            value: QueryLiteral::String(value.into()),
        });
        p.clause_id = Some(id.into());
        p
    }
    fn and(children: Vec<QueryPredicate>) -> QueryPredicate {
        QueryPredicate::new(QueryExpression::AllOf { children })
    }
    #[test]
    fn field_selection_and_one_clause_share_context_policy() {
        let creature = field("kind", "kind", "creature");
        let character = QueryPredicate::new(QueryExpression::Not {
            predicate: Box::new(field("exclude", "kind", "character")),
        });
        let level = field("level", "level", "5");
        let query = and(vec![creature, character, level.clone()]);
        let Some(QueryFacetOutcome::Available { value }) =
            exclude_query_selection(&query, QuerySelfExclusion::RootField("kind"))
        else {
            panic!("supported field selection")
        };
        assert_eq!(
            value.predicate,
            and(vec![
                QueryPredicate::boolean(true),
                QueryPredicate::boolean(true),
                level.clone()
            ])
        );
        let Some(QueryFacetOutcome::Available { value }) =
            exclude_query_selection(&query, QuerySelfExclusion::Clause("exclude"))
        else {
            panic!("supported clause")
        };
        assert_eq!(
            value.predicate,
            and(vec![
                field("kind", "kind", "creature"),
                QueryPredicate::boolean(true),
                level
            ])
        );
    }
    #[test]
    fn unsupported_paths_are_identical_for_fields_and_clauses() {
        for expression in [
            QueryExpression::AnyOf {
                children: vec![
                    field("target", "kind", "creature"),
                    field("other", "level", "5"),
                ],
            },
            QueryExpression::Not {
                predicate: Box::new(and(vec![
                    field("target", "kind", "creature"),
                    field("other", "level", "5"),
                ])),
            },
        ] {
            let query = QueryPredicate::new(expression);
            for selector in [
                QuerySelfExclusion::RootField("kind"),
                QuerySelfExclusion::Clause("target"),
            ] {
                assert!(matches!(
                    exclude_query_selection(&query, selector),
                    Some(QueryFacetOutcome::Unavailable {
                        reason: FacetUnavailableReason::NonConjunctiveContext
                    })
                ));
            }
        }
    }
    #[test]
    fn member_exclusion_preserves_correlation_and_root_field_stays_outside() {
        let query = QueryPredicate::new(QueryExpression::Exists {
            collection: "items".into(),
            scope_id: "item".into(),
            predicate: Box::new(and(vec![
                field("target", "kind", "equipment"),
                field("other", "name", "shield"),
            ])),
        });
        let Some(QueryFacetOutcome::Available { value }) =
            exclude_query_selection(&query, QuerySelfExclusion::Clause("target"))
        else {
            panic!("supported member")
        };
        assert_eq!(value.scope.as_deref(), Some("items"));
        assert_eq!(
            value.member,
            Some(and(vec![
                QueryPredicate::boolean(true),
                field("other", "name", "shield")
            ]))
        );
        assert!(exclude_query_selection(&query, QuerySelfExclusion::RootField("kind")).is_none());
    }
}
