//! CEL's maintained parser owns grammar and macro expansion. Atlas accepts a
//! bounded AST subset and compiles it; the library interpreter is not executed.
use super::validation::{ValidatedQuery, error, validate_query};
use atlas_domain::{
    QueryCompare, QueryError, QueryExpression, QueryFieldState, QueryLimits, QueryLiteral,
    QueryPredicate, QuerySetMatch,
};
use cel::{
    common::ast::{Expr, IdedExpr, LiteralValue, operators},
    parser::Parser,
};

fn path(e: &IdedExpr) -> Option<String> {
    match &e.expr {
        Expr::Ident(s) => Some(s.clone()),
        Expr::Select(s) if !s.test => Some(format!("{}.{}", path(&s.operand)?, s.field)),
        _ => None,
    }
}
fn relative(e: &IdedExpr, var: Option<&str>) -> Result<String, QueryError> {
    let p = path(e).ok_or_else(|| {
        error(
            "unsupported_ast",
            "Expected a selected catalog field; use atlas filters fields",
            None,
        )
    })?;
    match var {
        Some(v) => p
            .strip_prefix(&format!("{v}."))
            .map(str::to_owned)
            .ok_or_else(|| {
                error(
                    "mixed_scope",
                    "Child predicates may only reference their own scope variable",
                    Some(&p),
                )
            }),
        None => Ok(p),
    }
}
fn literal(e: &IdedExpr) -> Result<QueryLiteral, QueryError> {
    match &e.expr {
        Expr::Literal(LiteralValue::String(s)) => Ok(QueryLiteral::String(s.inner().to_owned())),
        Expr::Literal(LiteralValue::Boolean(v)) => Ok(QueryLiteral::Boolean(*v.inner())),
        Expr::Literal(LiteralValue::Int(v)) => Ok(QueryLiteral::Number((*v.inner()).into())),
        Expr::Literal(LiteralValue::UInt(v)) => Ok(QueryLiteral::Number((*v.inner()).into())),
        Expr::Literal(LiteralValue::Double(v)) => serde_json::Number::from_f64(*v.inner())
            .map(QueryLiteral::Number)
            .ok_or_else(|| error("numeric_domain", "Number must be finite", None)),
        Expr::Call(c) if c.func_name == operators::NEGATE && c.args.len() == 1 => {
            match literal(&c.args[0])? {
                QueryLiteral::Number(n) => {
                    if let Some(i) = n.as_i64().and_then(i64::checked_neg) {
                        Ok(QueryLiteral::Number(i.into()))
                    } else if n.is_f64() {
                        n.as_f64()
                            .and_then(|v| serde_json::Number::from_f64(-v))
                            .map(QueryLiteral::Number)
                            .ok_or_else(|| error("numeric_domain", "Number must be finite", None))
                    } else {
                        Err(error(
                            "numeric_domain",
                            "Negated integer is outside the lossless i64 domain",
                            None,
                        ))
                    }
                }
                _ => Err(error(
                    "literal_type",
                    "Unary minus accepts only numeric literals",
                    None,
                )),
            }
        }
        _ => Err(error(
            "literal_type",
            "Use a Boolean, string or numeric literal; field comparisons and null tests are unsupported. Use availability[\"field\"] for field state",
            None,
        )),
    }
}
fn state_field(e: &IdedExpr, var: Option<&str>) -> Option<String> {
    let Expr::Call(c) = &e.expr else { return None };
    if c.func_name != operators::INDEX || c.args.len() != 2 {
        return None;
    }
    let expected = var
        .map(|v| format!("{v}.availability"))
        .unwrap_or_else(|| "availability".to_owned());
    if path(&c.args[0])? != expected {
        return None;
    }
    match literal(&c.args[1]).ok()? {
        QueryLiteral::String(v) => Some(v),
        _ => None,
    }
}
fn state(value: QueryLiteral) -> Result<QueryFieldState, QueryError> {
    match value {
        QueryLiteral::String(s) => match s.as_str() {
            "value" => Ok(QueryFieldState::Value),
            "missing" => Ok(QueryFieldState::Missing),
            "null" => Ok(QueryFieldState::Null),
            "invalid" => Ok(QueryFieldState::Invalid),
            "not_applicable" => Ok(QueryFieldState::NotApplicable),
            _ => Err(error(
                "invalid_state",
                "State must be value/missing/null/invalid/not_applicable",
                None,
            )),
        },
        _ => Err(error(
            "literal_type",
            "Availability state must be a string",
            None,
        )),
    }
}
fn lower(
    e: &IdedExpr,
    var: Option<&str>,
    depth: usize,
    nodes: &mut usize,
) -> Result<QueryPredicate, QueryError> {
    *nodes += 1;
    if depth > QueryLimits::default().depth || *nodes > QueryLimits::default().nodes {
        return Err(error(
            "query_limit",
            "CEL AST exceeds node/depth bounds",
            None,
        ));
    }
    let expression = match &e.expr {
        Expr::Literal(LiteralValue::Boolean(v)) => {
            QueryExpression::BooleanConstant { value: *v.inner() }
        }
        Expr::Call(c) if c.target.is_none() => match c.func_name.as_str() {
            operators::LOGICAL_AND | operators::LOGICAL_OR if c.args.len() == 2 => {
                let children = c
                    .args
                    .iter()
                    .map(|c| lower(c, var, depth + 1, nodes))
                    .collect::<Result<Vec<_>, _>>()?;
                if c.func_name == operators::LOGICAL_AND {
                    QueryExpression::AllOf { children }
                } else {
                    QueryExpression::AnyOf { children }
                }
            }
            operators::LOGICAL_NOT if c.args.len() == 1 => QueryExpression::Not {
                predicate: Box::new(lower(&c.args[0], var, depth + 1, nodes)?),
            },
            operators::EQUALS
            | operators::NOT_EQUALS
            | operators::LESS
            | operators::LESS_EQUALS
            | operators::GREATER
            | operators::GREATER_EQUALS
                if c.args.len() == 2 =>
            {
                let (field, value, reversed) =
                    if path(&c.args[0]).is_some() || state_field(&c.args[0], var).is_some() {
                        (&c.args[0], literal(&c.args[1])?, false)
                    } else {
                        (&c.args[1], literal(&c.args[0])?, true)
                    };
                let op = match c.func_name.as_str() {
                    operators::EQUALS => QueryCompare::Eq,
                    operators::NOT_EQUALS => QueryCompare::Neq,
                    operators::LESS => {
                        if reversed {
                            QueryCompare::Gt
                        } else {
                            QueryCompare::Lt
                        }
                    }
                    operators::LESS_EQUALS => {
                        if reversed {
                            QueryCompare::Gte
                        } else {
                            QueryCompare::Lte
                        }
                    }
                    operators::GREATER => {
                        if reversed {
                            QueryCompare::Lt
                        } else {
                            QueryCompare::Gt
                        }
                    }
                    _ => {
                        if reversed {
                            QueryCompare::Lte
                        } else {
                            QueryCompare::Gte
                        }
                    }
                };
                if let Some(field) = state_field(field, var) {
                    if !matches!(op, QueryCompare::Eq | QueryCompare::Neq) {
                        return Err(error(
                            "operator_type",
                            "Availability supports == and !=",
                            Some(&field),
                        ));
                    }
                    let state = QueryPredicate::new(QueryExpression::StateMatch {
                        field,
                        state: state(value)?,
                    });
                    if op == QueryCompare::Neq {
                        QueryExpression::Not {
                            predicate: Box::new(state),
                        }
                    } else {
                        state.expression
                    }
                } else {
                    QueryExpression::Compare {
                        field: relative(field, var)?,
                        op,
                        value,
                    }
                }
            }
            operators::IN if c.args.len() == 2 => match &c.args[1].expr {
                Expr::List(list) if list.optional_indices.is_empty() => QueryExpression::In {
                    field: relative(&c.args[0], var)?,
                    values: list
                        .elements
                        .iter()
                        .map(literal)
                        .collect::<Result<Vec<_>, _>>()?,
                },
                _ => match literal(&c.args[0])? {
                    QueryLiteral::String(value) => QueryExpression::SetMatch {
                        field: relative(&c.args[1], var)?,
                        op: QuerySetMatch::Includes,
                        values: vec![value],
                    },
                    _ => {
                        return Err(error(
                            "literal_type",
                            "Set membership requires a literal identifier string",
                            None,
                        ));
                    }
                },
            },
            _ => {
                return Err(error(
                    "unsupported_ast",
                    format!(
                        "Unsupported CEL operation {}; use selected comparisons, membership, Boolean groups, exists or availability literal map",
                        c.func_name
                    ),
                    None,
                ));
            }
        },
        Expr::Comprehension(c) => {
            if var.is_some() {
                return Err(error(
                    "nested_exists",
                    "Nested collection scopes are unsupported",
                    None,
                ));
            }
            // Accept exactly the maintained parser's exists expansion. Other
            // standard macros expand differently and remain explicitly rejected.
            let Expr::Call(step) = &c.loop_step.expr else {
                return Err(error("unsupported_macro", "Only exists is supported", None));
            };
            let init_false =
                matches!(&c.accu_init.expr,Expr::Literal(LiteralValue::Boolean(v)) if !*v.inner());
            let result_accu = matches!(&c.result.expr,Expr::Ident(v) if *v==c.accu_var);
            let step_accu =
                matches!(step.args.first().map(|v|&v.expr),Some(Expr::Ident(v)) if *v==c.accu_var);
            let condition = match &c.loop_cond.expr {
                Expr::Call(call)
                    if call.func_name == operators::NOT_STRICTLY_FALSE && call.args.len() == 1 =>
                {
                    match &call.args[0].expr {
                        Expr::Call(neg)
                            if neg.func_name == operators::LOGICAL_NOT && neg.args.len() == 1 =>
                        {
                            matches!(&neg.args[0].expr,Expr::Ident(v) if *v==c.accu_var)
                        }
                        _ => false,
                    }
                }
                _ => false,
            };
            if !init_false
                || !result_accu
                || !step_accu
                || !condition
                || step.func_name != operators::LOGICAL_OR
                || step.args.len() != 2
                || c.iter_var2.is_some()
            {
                return Err(error(
                    "unsupported_macro",
                    "Only one-variable exists is supported",
                    None,
                ));
            }
            QueryExpression::Exists {
                collection: relative(&c.iter_range, None)?,
                scope_id: format!("cel-scope-{}", e.id),
                predicate: Box::new(lower(&step.args[1], Some(&c.iter_var), depth + 1, nodes)?),
            }
        }
        _ => {
            return Err(error(
                "unsupported_ast",
                "Unsupported CEL syntax; use atlas filters fields for the selected field/operator catalog",
                None,
            ));
        }
    };
    Ok(QueryPredicate {
        clause_id: Some(format!("cel-clause-{}", e.id)),
        expression,
    })
}
/// A resource guard, not a lexer/grammar: only count nesting and uninterrupted
/// selection/negation chains outside CEL strings/comments. The maintained parser
/// still decides whether any admitted text is legal CEL.
fn source_guard(source: &str) -> Result<(), QueryError> {
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    let mut selects = 0usize;
    let mut negations = 0usize;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if byte == b'\'' || byte == b'"' {
            let raw = i > 0
                && matches!(bytes[i - 1], b'r' | b'R')
                && (i < 2 || !bytes[i - 2].is_ascii_alphanumeric() && bytes[i - 2] != b'_');
            let width = if bytes.get(i + 1) == Some(&byte) && bytes.get(i + 2) == Some(&byte) {
                3
            } else {
                1
            };
            i += width;
            while i < bytes.len() {
                if !raw && bytes[i] == b'\\' {
                    i = (i + 2).min(bytes.len());
                    continue;
                }
                if (0..width).all(|n| bytes.get(i + n) == Some(&byte)) {
                    i += width;
                    break;
                }
                i += 1;
            }
            selects = 0;
            negations = 0;
            continue;
        }
        match byte {
            b'(' | b'[' | b'{' => {
                depth += 1;
                selects = 0;
                negations = 0;
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                selects = 0;
                negations = 0;
            }
            b'.' => selects += 1,
            b'!' if bytes.get(i + 1) != Some(&b'=') => negations += 1,
            b'-' => negations += 1,
            b if b.is_ascii_whitespace() || b.is_ascii_alphanumeric() || b == b'_' => {}
            _ => {
                selects = 0;
                negations = 0;
            }
        }
        if depth > QueryLimits::default().depth
            || selects > QueryLimits::default().depth
            || negations > QueryLimits::default().depth
        {
            return Err(error(
                "query_limit",
                "CEL nesting/selection/negation chain exceeds 32",
                None,
            ));
        }
        i += 1;
    }
    Ok(())
}
pub fn parse_where(source: &str) -> Result<ValidatedQuery, QueryError> {
    if source.len() > QueryLimits::default().source_bytes {
        return Err(error("query_limit", "CEL source exceeds 16KiB", None));
    }
    source_guard(source)?;
    let ast = Parser::new()
        .max_recursion_depth(64)
        .error_recovery_limit(8)
        .parse(source)
        .map_err(|e| {
            let mut error = error("cel_syntax", e.to_string(), None);
            error.source_span = Some((0, source.len()));
            error
        })?;
    let lowered = lower(&ast, None, 0, &mut 0).and_then(|p| validate_query(&p));
    lowered.map_err(|mut e| {
        e.source_span = Some((0, source.len()));
        e
    })
}
