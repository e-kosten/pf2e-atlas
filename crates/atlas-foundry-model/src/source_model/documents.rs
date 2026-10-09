//! Callable authored source parsers. No defaults, repairs, or pipeline adoption.
use super::generated::{
    self, ActorSourcePF2e, ItemSourcePF2e, JournalEntrySource, MacroSource, RollTableSource,
    RuleSource,
};
use super::parse::{SourceContext, SourceDiagnostic};
use super::value::parse_source;

macro_rules! source_parser {
    ($function:ident, $model:ident) => {
        pub fn $function(context: SourceContext, bytes: &[u8]) -> Result<$model, SourceDiagnostic> {
            let value = parse_source(bytes).map_err(|error| {
                context.message(&context.json_path, "complete JSON value", error.to_string())
            })?;
            generated::$function(&value, &context, &context.json_path)
        }
    };
}

source_parser!(parse_actor_source_pf2e, ActorSourcePF2e);
source_parser!(parse_item_source_pf2e, ItemSourcePF2e);
source_parser!(parse_journal_entry_source, JournalEntrySource);
source_parser!(parse_macro_source, MacroSource);
source_parser!(parse_roll_table_source, RollTableSource);
source_parser!(parse_rule_source, RuleSource);
