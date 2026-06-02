use std::collections::BTreeMap;

use serde::Deserialize;

pub(crate) const DERIVED_AFFLICTIONS_PACK_NAME: &str = "derived-afflictions";
pub(crate) const DERIVED_AFFLICTIONS_PACK_LABEL: &str = "Derived Afflictions";
pub(crate) const DERIVED_AFFLICTION_INSTANCES_PACK_NAME: &str = "derived-affliction-instances";
pub(crate) const DERIVED_AFFLICTION_INSTANCES_PACK_LABEL: &str = "Derived Affliction Instances";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IngestDiagnostics {
    pub taxonomy_folder_records: usize,
    pub taxonomy_glossary_records: usize,
    pub variant_parenthetical_records: usize,
    pub variant_suffix_records: usize,
    pub variant_creature_blurb_records: usize,
    pub variant_creature_suffix_records: usize,
    pub variant_exact_base_records: usize,
    pub generated_affliction_canonical_records: usize,
    pub generated_affliction_instance_records: usize,
    pub generated_affliction_reference_edges: usize,
    pub dropped_inline_macros: BTreeMap<String, DroppedInlineMacroDiagnostic>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DroppedInlineMacroDiagnostic {
    pub count: usize,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantCandidate {
    pub(crate) base_name: String,
    pub(crate) label: Option<String>,
    pub(crate) axes: Vec<String>,
    pub(crate) source: VariantSource,
    pub(crate) diagnostic_source: VariantDiagnosticSource,
    pub(crate) confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VariantDiagnosticSource {
    Parenthetical,
    Suffix,
    CreatureBlurb,
    CreatureSuffix,
    ExactBase,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FolderDefinition {
    #[serde(rename = "_id")]
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) folder: Option<String>,
}
use atlas_record::VariantSource;
