#![deny(unsafe_code)]

use atlas_domain::{
    MetricDomain, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey,
    RemasterLinkSource, TimeKind, TimeUnit,
};

pub mod content;
mod json_projection;
pub mod metrics;
pub mod presentation;
mod presentation_format;
mod presentation_recipe;
#[cfg(test)]
mod presentation_recipe_tests;
pub mod reference_policy;

pub use content::{
    ContentBlock, ContentDefinitionItem, ContentDocument, ContentFtsField, ContentInline,
    ContentReference, ContentReferenceLocator, ContentSectionNode, ContentSectionOrigin,
    ContentSourceKind, ContentVisibility, RecordFtsProjection, SupplementalContentDocument,
    build_content_section_tree, build_record_fts_projection, iter_content_references,
    render_markdown_like, render_plain_text, visit_content_references_mut,
};
pub use json_projection::{
    RecordBlockJson, RecordJson, RecordJsonOptions, RecordSectionJson, record_json,
};
pub use metrics::{
    MetricCapture, MetricDefinition, MetricDefinitionMatch, MetricDisplayLabel, MetricGroup,
    MetricKeyPattern, MetricKeySegment, MetricLabelTemplate, MetricVariableVocabulary,
    PatternMetricDefinition, StaticMetricDefinition, all_definitions, definition_for, is_known_key,
    label_for_row,
};
pub use presentation::{
    PresentationBadge, PresentationBadgeKind, PresentationBlock, PresentationFact,
    PresentationRelationship, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, PresentationText, RecordPresentationDocument,
};
pub use presentation_recipe::build_record_presentation_document;
pub use reference_policy::{
    DEFAULT_EXCLUDED_SOURCE_KINDS, ReferenceEdgeFacts, ReferenceGraphMode, ReferenceGraphPolicy,
    ReferenceVisibilityPolicy, reference_edge_matches_mode, reference_graph_policy,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedRecord {
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
    pub description: Option<ContentDocument>,
    pub blurb: Option<ContentDocument>,
    pub supplemental_content: Vec<SupplementalContentDocument>,
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
    pub is_default_visible: bool,
    pub raw_json: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersistedRecord {
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
    pub description: Option<ContentDocument>,
    pub blurb: Option<ContentDocument>,
    pub supplemental_content: Vec<SupplementalContentDocument>,
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
    pub is_default_visible: bool,
    pub raw_json: String,
}

impl From<&PersistedRecord> for NormalizedRecord {
    fn from(record: &PersistedRecord) -> Self {
        Self {
            key: record.key.clone(),
            id: record.id.clone(),
            name: record.name.clone(),
            normalized_name: record.normalized_name.clone(),
            record_family: record.record_family,
            pack_name: record.pack_name.clone(),
            pack_label: record.pack_label.clone(),
            foundry_document_type: record.foundry_document_type.clone(),
            foundry_record_type: record.foundry_record_type.clone(),
            level: record.level,
            rarity: record.rarity.clone(),
            traits: record.traits.clone(),
            system_category: record.system_category.clone(),
            system_group: record.system_group.clone(),
            system_base_item: record.system_base_item.clone(),
            system_usage: record.system_usage.clone(),
            system_price_json: record.system_price_json.clone(),
            system_actions_value: record.system_actions_value,
            system_time_value: record.system_time_value.clone(),
            system_duration_value: record.system_duration_value.clone(),
            price_cp: record.price_cp,
            activation_time: record.activation_time.clone(),
            duration: record.duration.clone(),
            metrics: record.metrics.clone(),
            actor_data: record.actor_data.clone(),
            item_data: record.item_data.clone(),
            spell_data: record.spell_data.clone(),
            publication_title: record.publication_title.clone(),
            publication_remaster: record.publication_remaster,
            description: record.description.clone(),
            blurb: record.blurb.clone(),
            supplemental_content: record.supplemental_content.clone(),
            publication_family: record.publication_family,
            folder_id: record.folder_id.clone(),
            taxonomy_families: record.taxonomy_families.clone(),
            variant_group_key: record.variant_group_key.clone(),
            variant_base_name: record.variant_base_name.clone(),
            variant_label: record.variant_label.clone(),
            variant_axes: record.variant_axes.clone(),
            variant_confidence: record.variant_confidence,
            variant_source: record.variant_source.clone(),
            source_path: record.source_path.clone(),
            is_default_visible: record.is_default_visible,
            raw_json: record.raw_json.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PersistedRecordSet {
    pub records: Vec<PersistedRecord>,
    pub reference_edges: Vec<ReferenceEdge>,
    pub aliases: Vec<RecordAlias>,
    pub remaster_links: Vec<RemasterLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceEdge {
    pub from_record_key: RecordKey,
    pub to_record_key: RecordKey,
    pub display_text: Option<String>,
    pub reference_text: String,
    pub source_kind: ContentSourceKind,
    pub visibility: ContentVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AliasSource {
    RemasterJournal,
    Migration,
    CompendiumSource,
}

impl AliasSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RemasterJournal => "remaster_journal",
            Self::Migration => "migration",
            Self::CompendiumSource => "compendium_source",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "remaster_journal" => Some(Self::RemasterJournal),
            "migration" => Some(Self::Migration),
            "compendium_source" => Some(Self::CompendiumSource),
            _ => None,
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
