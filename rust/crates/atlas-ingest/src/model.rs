use std::collections::BTreeMap;
use std::path::PathBuf;

use atlas_domain::{
    MetricDomain, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey,
    RemasterLinkSource, TextStatus, TimeKind, TimeUnit,
};
use serde::Deserialize;
use serde_json::Value;

pub(crate) const DERIVED_AFFLICTIONS_PACK_NAME: &str = "derived-afflictions";
pub(crate) const DERIVED_AFFLICTIONS_PACK_LABEL: &str = "Derived Afflictions";
pub(crate) const DERIVED_AFFLICTION_INSTANCES_PACK_NAME: &str = "derived-affliction-instances";
pub(crate) const DERIVED_AFFLICTION_INSTANCES_PACK_LABEL: &str = "Derived Affliction Instances";

#[derive(Debug, Clone)]
pub struct BuildArtifactOptions {
    pub source_root: PathBuf,
    pub output_path: PathBuf,
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactReport {
    pub output_path: PathBuf,
    pub pack_count: usize,
    pub record_count: usize,
    pub source_signature: String,
    pub diagnostics: IngestDiagnostics,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLoad {
    pub manifest_path: PathBuf,
    pub source_signature: String,
    pub packs: Vec<LoadedPack>,
    pub records: Vec<LoadedRecord>,
    pub references: Vec<ReferenceEdge>,
    pub aliases: Vec<RecordAlias>,
    pub remaster_links: Vec<RemasterLink>,
    pub diagnostics: IngestDiagnostics,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedRecord {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPack {
    pub name: PackName,
    pub label: String,
    pub document_type: String,
    pub declared_path: String,
    pub resolved_path: PathBuf,
    pub record_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedRecord {
    pub key: RecordKey,
    pub id: RecordId,
    pub name: String,
    pub normalized_name: String,
    pub record_family: RecordFamily,
    pub pack_name: PackName,
    pub pack_label: String,
    pub foundry_document_type: String,
    pub foundry_record_type: String,
    pub level: Option<i64>,
    pub rarity: Option<String>,
    pub traits: Vec<String>,
    pub system_category: Option<String>,
    pub system_group: Option<String>,
    pub system_base_item: Option<String>,
    pub system_usage: Option<String>,
    pub system_price_json: Option<String>,
    pub system_actions_value: Option<i64>,
    pub system_time_value: Option<String>,
    pub system_duration_value: Option<String>,
    pub price_cp: Option<i64>,
    pub activation_time: Option<NormalizedTime>,
    pub duration: Option<NormalizedTime>,
    pub metrics: Vec<MetricRow>,
    pub actor_data: Option<ActorSideData>,
    pub item_data: Option<ItemSideData>,
    pub spell_data: Option<SpellSideData>,
    pub publication_title: Option<String>,
    pub publication_remaster: bool,
    pub description_text: Option<String>,
    pub blurb_text: Option<String>,
    pub publication_family: PublicationFamily,
    pub folder_id: Option<String>,
    pub taxonomy_families: Vec<String>,
    pub variant_group_key: Option<String>,
    pub variant_base_name: Option<String>,
    pub variant_label: Option<String>,
    pub variant_axes: Vec<String>,
    pub variant_confidence: Option<f64>,
    pub variant_source: String,
    pub source_path: String,
    pub text_status: TextStatus,
    pub is_default_visible: bool,
    pub search_text_projection: String,
    pub reference_candidates: Vec<ReferenceCandidate>,
    pub raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCandidate {
    pub raw_target: String,
    pub display_text: Option<String>,
    pub reference_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceEdge {
    pub from_record_key: RecordKey,
    pub to_record_key: RecordKey,
    pub display_text: Option<String>,
    pub reference_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AliasSource {
    RemasterJournal,
    Migration,
    CompendiumSource,
}

impl AliasSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::RemasterJournal => "remaster_journal",
            Self::Migration => "migration",
            Self::CompendiumSource => "compendium_source",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordAlias {
    pub canonical_record_key: RecordKey,
    pub alias_text: String,
    pub normalized_alias: String,
    pub source: AliasSource,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemasterLink {
    pub remaster_record_key: RecordKey,
    pub legacy_record_key: RecordKey,
    pub source: RemasterLinkSource,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSideData {
    pub size: Option<String>,
    pub languages: Vec<String>,
    pub speed_types: Vec<String>,
    pub senses: Vec<String>,
    pub immunities: Vec<String>,
    pub resistances: Vec<String>,
    pub weaknesses: Vec<String>,
    pub disable_text: Option<String>,
    pub disable_skills: Vec<String>,
    pub is_complex: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemSideData {
    pub system_category: Option<String>,
    pub system_base_item: Option<String>,
    pub system_group: Option<String>,
    pub system_usage: Option<String>,
    pub price_cp: Option<i64>,
    pub bulk_value: Option<f64>,
    pub hands_requirement: Option<String>,
    pub damage_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpellSideData {
    pub traditions: Vec<String>,
    pub spell_kinds: Vec<String>,
    pub range_text: Option<String>,
    pub range_value: Option<f64>,
    pub target_text: Option<String>,
    pub area_type: Option<String>,
    pub area_value: Option<f64>,
    pub save_type: Option<String>,
    pub sustained: bool,
    pub basic_save: bool,
    pub damage_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricRow {
    pub domain: MetricDomain,
    pub key: String,
    pub value: MetricValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Number(f64),
    Text(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTime {
    pub kind: TimeKind,
    pub actions: Option<i64>,
    pub duration_value: Option<i64>,
    pub duration_unit: Option<TimeUnit>,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RecordReferenceIndex {
    pub(crate) by_key: BTreeMap<String, LoadedRecord>,
    pub(crate) by_pack_id: BTreeMap<(String, String), RecordKey>,
    pub(crate) by_pack_name: BTreeMap<(String, String), Vec<RecordKey>>,
    pub(crate) by_name: BTreeMap<String, Vec<RecordKey>>,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantCandidate {
    pub(crate) base_name: String,
    pub(crate) label: Option<String>,
    pub(crate) axes: Vec<String>,
    pub(crate) source: &'static str,
    pub(crate) diagnostic_source: VariantDiagnosticSource,
    pub(crate) confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AfflictionFamily {
    Curse,
    Disease,
    Poison,
}

#[derive(Debug, Clone)]
pub(crate) struct AfflictionOccurrence {
    pub(crate) host_record: LoadedRecord,
    pub(crate) source_record: Option<LoadedRecord>,
    pub(crate) source_raw: Option<Value>,
    pub(crate) child_raw: Value,
    pub(crate) family: AfflictionFamily,
    pub(crate) name: String,
    pub(crate) slug: Option<String>,
    pub(crate) traits: Vec<String>,
    pub(crate) linked_names: Vec<String>,
    pub(crate) source_path: String,
    pub(crate) occurrence_ref: String,
    pub(crate) candidate_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratedAfflictionBuild {
    pub(crate) records: Vec<LoadedRecord>,
    pub(crate) references: Vec<ReferenceEdge>,
}

pub(crate) struct DerivedAfflictionRecordInput {
    pub(crate) key: RecordKey,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) record_type: &'static str,
    pub(crate) family: AfflictionFamily,
    pub(crate) traits: Vec<String>,
    pub(crate) description_text: Option<String>,
    pub(crate) blurb_text: Option<String>,
    pub(crate) level: Option<i64>,
    pub(crate) rarity: Option<String>,
    pub(crate) publication_title: Option<String>,
    pub(crate) publication_remaster: bool,
    pub(crate) publication_family: PublicationFamily,
    pub(crate) source_path: String,
    pub(crate) is_default_visible: bool,
    pub(crate) search_text_projection: String,
    pub(crate) raw: Value,
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

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) packs: Vec<ManifestPack>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManifestPack {
    pub(crate) name: String,
    pub(crate) label: String,
    #[serde(rename = "type")]
    pub(crate) document_type: String,
    pub(crate) path: String,
}

#[derive(Debug)]
pub(crate) struct ParsedManifest {
    pub(crate) manifest: Manifest,
    pub(crate) content_hash: String,
}
