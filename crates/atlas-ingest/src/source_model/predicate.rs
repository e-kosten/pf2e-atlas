//! Persisted predicate shapes only. This module does not execute Foundry predicates.
use super::generated::{self, PredicateInput, PredicateStatement, PredicateStatements};
use super::parse::{SourceContext, SourceDiagnostic};
use super::value::parse_source;

pub fn parse_predicate_statement(
    context: SourceContext,
    bytes: &[u8],
) -> Result<PredicateStatement, SourceDiagnostic> {
    let value = parse_source(bytes).map_err(|error| {
        context.message(&context.json_path, "complete JSON value", error.to_string())
    })?;
    generated::parse_predicate_statement(&value, &context, &context.json_path)
}

pub fn parse_predicate_statements(
    context: SourceContext,
    bytes: &[u8],
) -> Result<PredicateStatements, SourceDiagnostic> {
    let value = parse_source(bytes).map_err(|error| {
        context.message(&context.json_path, "complete JSON value", error.to_string())
    })?;
    generated::parse_predicate_statements(&value, &context, &context.json_path)
}

/// ChoiceSet's compiler-extracted constructor input accepts a statement or array.
pub fn parse_predicate_input(
    context: SourceContext,
    bytes: &[u8],
) -> Result<PredicateInput, SourceDiagnostic> {
    let value = parse_source(bytes).map_err(|error| {
        context.message(&context.json_path, "complete JSON value", error.to_string())
    })?;
    generated::parse_predicate_input(&value, &context, &context.json_path)
}
