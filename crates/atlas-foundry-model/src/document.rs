use serde::{Deserialize, Serialize};

use crate::source_model::admission::{
    admit_actor_value, admit_item_value, admit_journal_value, admit_macro_value,
    admit_roll_table_value, source,
};
use crate::{
    ActorSourcePF2e, ItemSourcePF2e, JournalEntrySource, MacroSource, RollTableSource,
    SourceAdmission, SourceContext, SourceDiagnostic,
};

/// Authored document bodies, without Foundry preparation or Atlas enrichment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "$variant", content = "$value", deny_unknown_fields)]
pub enum FoundryDocumentSource {
    Actor(Box<ActorSourcePF2e>),
    Item(Box<ItemSourcePF2e>),
    JournalEntry(Box<JournalEntrySource>),
    Macro(Box<MacroSource>),
    RollTable(Box<RollTableSource>),
}

/// Parse authored JSON once, then apply the unchanged family admission policy.
pub fn admit_document_source(
    document_type: &str,
    context: SourceContext,
    bytes: &[u8],
) -> Result<SourceAdmission<FoundryDocumentSource>, SourceDiagnostic> {
    fn wrap<T>(
        admission: SourceAdmission<T>,
        constructor: impl FnOnce(Box<T>) -> FoundryDocumentSource,
    ) -> SourceAdmission<FoundryDocumentSource> {
        SourceAdmission {
            raw: admission.raw,
            model: admission.model.map(Box::new).map(constructor),
            diagnostics: admission.diagnostics,
        }
    }
    let raw = source(&context, bytes)?;
    Ok(match document_type {
        "Actor" => wrap(
            admit_actor_value(context, raw)?,
            FoundryDocumentSource::Actor,
        ),
        "Item" => wrap(admit_item_value(context, raw)?, FoundryDocumentSource::Item),
        "JournalEntry" => wrap(
            admit_journal_value(context, raw)?,
            FoundryDocumentSource::JournalEntry,
        ),
        "Macro" => wrap(
            admit_macro_value(context, raw)?,
            FoundryDocumentSource::Macro,
        ),
        "RollTable" => wrap(
            admit_roll_table_value(context, raw)?,
            FoundryDocumentSource::RollTable,
        ),
        _ => SourceAdmission {
            diagnostics: vec![context.message(
                "$",
                "manifest document type Actor | Item | JournalEntry | Macro | RollTable",
                document_type,
            )],
            raw,
            model: None,
        },
    })
}
