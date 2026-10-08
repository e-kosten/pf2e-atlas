//! Lossless source admission, separate from strict declaration parsing and normalization.
use serde::Serialize;

use super::generated::{
    self, ActorSourcePF2e, ItemSourcePF2e, JournalEntrySource, MacroSource, RollTableSource,
    RuleSource,
};
use super::parse::{ParseResult, SourceContext, SourceDiagnostic};
use super::presence::SourceFieldRejection;
use super::value::{SourceValue, parse_source};

/// Original source plus its usable typed representation. No defaults or coercions.
/// An unsupported root retains raw source with `model: None`. Invalid fields in a
/// supported document are explicit `SourcePresence::Invalid` values in the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAdmission<T> {
    pub raw: SourceValue,
    pub model: Option<T>,
    pub diagnostics: Vec<SourceDiagnostic>,
}

pub(crate) fn source(context: &SourceContext, bytes: &[u8]) -> ParseResult<SourceValue> {
    let value = parse_source(bytes).map_err(|error| {
        context.message(
            &context.json_path,
            "complete JSON object",
            error.to_string(),
        )
    })?;
    if !matches!(value, SourceValue::Object(_)) {
        return Err(context.error(&context.json_path, "document object", &value));
    }
    Ok(value)
}

fn admit<T>(
    mut context: SourceContext,
    raw: SourceValue,
    parse: impl Fn(&SourceValue, &SourceContext, &str) -> ParseResult<T>,
) -> ParseResult<SourceAdmission<T>> {
    let mut diagnostics = Vec::new();
    // Re-run the unchanged parser with explicit invalid field states. Each
    // iteration marks a new field; finite input bounds the loop. Union identity,
    // collection ordering, other values, and source bytes are never rewritten.
    loop {
        match parse(&raw, &context, &context.json_path) {
            Ok(model) => {
                return Ok(SourceAdmission {
                    raw,
                    model: Some(model),
                    diagnostics,
                });
            }
            Err(mut error) => {
                let failure = error.field_failure.take();
                diagnostics.push(error.clone());
                if let Some(failure) = failure {
                    let (json_path, values) = *failure;
                    if !context.rejected_fields.contains_key(&json_path) {
                        context.rejected_fields.insert(
                            json_path.clone(),
                            Box::new(SourceFieldRejection {
                                json_path,
                                values,
                                diagnostic: error,
                            }),
                        );
                        continue;
                    }
                }
                return Ok(SourceAdmission {
                    raw,
                    model: None,
                    diagnostics,
                });
            }
        }
    }
}

macro_rules! admission {
    ($function:ident, $value_function:ident, $parser:ident, $model:ident) => {
        pub fn $function(
            context: SourceContext,
            bytes: &[u8],
        ) -> ParseResult<SourceAdmission<$model>> {
            let raw = source(&context, bytes)?;
            $value_function(context, raw)
        }

        pub(crate) fn $value_function(
            context: SourceContext,
            raw: SourceValue,
        ) -> ParseResult<SourceAdmission<$model>> {
            admit(context, raw, generated::$parser)
        }
    };
}

admission!(
    admit_actor_source_pf2e,
    admit_actor_value,
    parse_actor_source_pf2e,
    ActorSourcePF2e
);
admission!(
    admit_item_source_pf2e,
    admit_item_value,
    parse_item_source_pf2e,
    ItemSourcePF2e
);
admission!(
    admit_journal_entry_source,
    admit_journal_value,
    parse_journal_entry_source,
    JournalEntrySource
);
admission!(
    admit_macro_source,
    admit_macro_value,
    parse_macro_source,
    MacroSource
);
admission!(
    admit_roll_table_source,
    admit_roll_table_value,
    parse_roll_table_source,
    RollTableSource
);

/// Rules are interpreted as complete nodes. A rejected condition or mechanical
/// field makes the whole specific rule unavailable; its parent stays intact.
pub fn admit_rule_source(
    context: SourceContext,
    bytes: &[u8],
) -> ParseResult<SourceAdmission<RuleSource>> {
    let raw = source(&context, bytes)?;
    let (model, diagnostics) =
        match generated::parse_rule_source(&raw, &context, &context.json_path) {
            Ok(model) => (Some(model), Vec::new()),
            Err(mut error) => {
                error.field_failure = None;
                (None, vec![error])
            }
        };
    Ok(SourceAdmission {
        raw,
        model,
        diagnostics,
    })
}
