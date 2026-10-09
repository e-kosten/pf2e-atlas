#![deny(unsafe_code)]
//! Generated Foundry authored models, admission and typed snapshots.
//! Filesystem discovery and loading remain in `atlas-ingest`.

mod document;
mod snapshot;
mod source_model;

pub use document::{FoundryDocumentSource, admit_document_source};
pub use snapshot::{
    SNAPSHOT_VERSION, SOURCE_CONTRACT_ID, SnapshotError, decode_snapshot, encode_snapshot,
};
pub use source_model::value::parse_source as parse_source_value;
pub use source_model::{
    ActorSourcePF2e, Coins, EquipmentFields, EquipmentSourceSlice, EquippedData,
    EquippedDataCarryType, ItemDescriptionSource, ItemGranterSource, ItemSourceFlagsPF2e,
    ItemSourcePF2e, ItemSourceSlice, ItemTraits, JournalEntrySource, MacroSource, PartialPrice,
    PhysicalEquipmentFields, PhysicalEquipmentFieldsUsage, PhysicalItemHPSource, PredicateInput,
    PredicateStatement, PredicateStatements, PublicationData, RollTableSource, RuleSource,
    SourceAdmission, SourceContext, SourceDiagnostic, SourceFieldRejection, SourceMap,
    SourceObject, SourcePresence, SourceValue, admit_actor_source_pf2e, admit_item_source_pf2e,
    admit_journal_entry_source, admit_macro_source, admit_roll_table_source, admit_rule_source,
    generated, is_numeric_key, parse_actor_source_pf2e, parse_equipment_source_slice,
    parse_item_source_pf2e, parse_item_source_slice, parse_journal_entry_source,
    parse_macro_source, parse_physical_equipment_fields, parse_predicate_input,
    parse_predicate_statement, parse_predicate_statements, parse_roll_table_source,
    parse_rule_source,
};
