use atlas_domain::{
    MetricDomain, PackName, PublicationCategory, Rarity, RecordId, RecordKey, RecordKind,
    RemasterLinkSource, TimeKind, TimeUnit, normalize_record_name,
};

use crate::content::{
    ContentSourceKind, ContentVisibility, RecordContentDocument, ReferenceRelationKind,
    RichDocument,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AtlasRecord {
    pub identity: RecordIdentity,
    pub classification: RecordClassification,
    pub foundry: FoundryRecordInfo,
    pub provenance: RecordProvenance,
    pub publication: RecordPublication,
    pub requirements: RecordRequirements,
    pub timing: RecordTiming,
    pub mechanics: RecordMechanics,
    pub content: RecordContent,
    pub variant: Option<RecordVariantMembership>,
    pub visibility: RecordVisibility,
}

impl AtlasRecord {
    pub fn new(
        identity: RecordIdentity,
        classification: RecordClassification,
        foundry: FoundryRecordInfo,
        provenance: RecordProvenance,
    ) -> Self {
        Self {
            identity,
            classification,
            foundry,
            provenance,
            publication: RecordPublication::default(),
            requirements: RecordRequirements::default(),
            timing: RecordTiming::default(),
            mechanics: RecordMechanics::default(),
            content: RecordContent::default(),
            variant: None,
            visibility: RecordVisibility::visible(RecordVisibilityReason::SourceRecord),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordIdentity {
    pub key: RecordKey,
    pub name: String,
}

impl RecordIdentity {
    pub fn new(key: RecordKey, name: impl Into<String>) -> Self {
        Self {
            key,
            name: name.into(),
        }
    }

    pub fn id(&self) -> &RecordId {
        self.key.id()
    }

    pub fn pack(&self) -> &PackName {
        self.key.pack()
    }

    pub fn normalized_name(&self) -> String {
        normalize_record_name(&self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordClassification {
    pub kind: RecordKind,
    pub level: Option<i64>,
    pub rarity: Option<Rarity>,
    pub traits: Vec<String>,
    pub taxonomy: RecordTaxonomy,
}

impl RecordClassification {
    pub fn new(kind: RecordKind) -> Self {
        Self {
            kind,
            level: None,
            rarity: None,
            traits: Vec::new(),
            taxonomy: RecordTaxonomy::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordTaxonomy {
    pub inferred_groups: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundryRecordInfo {
    pub pack_label: String,
    pub document_type: FoundryDocumentType,
    pub record_type: FoundryRecordType,
    pub folder_id: Option<String>,
}

impl FoundryRecordInfo {
    pub fn new(
        pack_label: impl Into<String>,
        document_type: FoundryDocumentType,
        record_type: FoundryRecordType,
    ) -> Self {
        Self {
            pack_label: pack_label.into(),
            document_type,
            record_type,
            folder_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoundryDocumentType {
    Actor,
    Item,
    JournalEntry,
    JournalEntryPage,
    Macro,
    RollTable,
    Other(String),
}

impl FoundryDocumentType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Actor => "Actor",
            Self::Item => "Item",
            Self::JournalEntry => "JournalEntry",
            Self::JournalEntryPage => "JournalEntryPage",
            Self::Macro => "Macro",
            Self::RollTable => "RollTable",
            Self::Other(value) => value,
        }
    }

    pub fn from_foundry(value: &str) -> Self {
        match value {
            "Actor" => Self::Actor,
            "Item" => Self::Item,
            "JournalEntry" => Self::JournalEntry,
            "JournalEntryPage" => Self::JournalEntryPage,
            "Macro" => Self::Macro,
            "RollTable" => Self::RollTable,
            other => Self::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoundryRecordType {
    Action,
    Affliction,
    AfflictionInstance,
    Ammo,
    Ancestry,
    Army,
    Armor,
    Backpack,
    Background,
    CampaignFeature,
    Character,
    Class,
    Condition,
    Consumable,
    Deity,
    Effect,
    Equipment,
    Familiar,
    Feat,
    Hazard,
    Heritage,
    Kit,
    Npc,
    Script,
    Shield,
    Spell,
    Treasure,
    Vehicle,
    Weapon,
    Other(String),
}

impl FoundryRecordType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Action => "action",
            Self::Affliction => "affliction",
            Self::AfflictionInstance => "affliction-instance",
            Self::Ammo => "ammo",
            Self::Ancestry => "ancestry",
            Self::Army => "army",
            Self::Armor => "armor",
            Self::Backpack => "backpack",
            Self::Background => "background",
            Self::CampaignFeature => "campaignFeature",
            Self::Character => "character",
            Self::Class => "class",
            Self::Condition => "condition",
            Self::Consumable => "consumable",
            Self::Deity => "deity",
            Self::Effect => "effect",
            Self::Equipment => "equipment",
            Self::Familiar => "familiar",
            Self::Feat => "feat",
            Self::Hazard => "hazard",
            Self::Heritage => "heritage",
            Self::Kit => "kit",
            Self::Npc => "npc",
            Self::Script => "script",
            Self::Shield => "shield",
            Self::Spell => "spell",
            Self::Treasure => "treasure",
            Self::Vehicle => "vehicle",
            Self::Weapon => "weapon",
            Self::Other(value) => value,
        }
    }

    pub fn from_foundry(value: &str) -> Self {
        match value {
            "action" => Self::Action,
            "affliction" => Self::Affliction,
            "affliction-instance" => Self::AfflictionInstance,
            "ammo" => Self::Ammo,
            "ancestry" => Self::Ancestry,
            "army" => Self::Army,
            "armor" => Self::Armor,
            "backpack" => Self::Backpack,
            "background" => Self::Background,
            "campaignFeature" => Self::CampaignFeature,
            "character" => Self::Character,
            "class" => Self::Class,
            "condition" => Self::Condition,
            "consumable" => Self::Consumable,
            "deity" => Self::Deity,
            "effect" => Self::Effect,
            "equipment" => Self::Equipment,
            "familiar" => Self::Familiar,
            "feat" => Self::Feat,
            "hazard" => Self::Hazard,
            "heritage" => Self::Heritage,
            "kit" => Self::Kit,
            "npc" => Self::Npc,
            "script" => Self::Script,
            "shield" => Self::Shield,
            "spell" => Self::Spell,
            "treasure" => Self::Treasure,
            "vehicle" => Self::Vehicle,
            "weapon" => Self::Weapon,
            other => Self::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenance {
    pub source_path: String,
    pub raw_json: Option<String>,
}

impl RecordProvenance {
    pub fn new(source_path: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            raw_json: None,
        }
    }

    pub fn with_raw_json(mut self, raw_json: impl Into<String>) -> Self {
        self.raw_json = Some(raw_json.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordPublication {
    pub title: Option<String>,
    pub remaster: bool,
    pub category: PublicationCategory,
}

impl Default for RecordPublication {
    fn default() -> Self {
        Self {
            title: None,
            remaster: false,
            category: PublicationCategory::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordRequirements {
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecordTiming {
    pub activation: Option<RecordActivationTiming>,
    pub duration: Option<RecordDurationTiming>,
}

impl RecordTiming {
    pub fn activation_time(&self) -> Option<&NormalizedTime> {
        self.activation.as_ref().map(|activation| &activation.time)
    }

    pub fn activation_actions_value(&self) -> Option<i64> {
        self.activation.as_ref().and_then(|activation| {
            (activation.source_field == ActivationTimeSourceField::ActionsValue)
                .then_some(activation.time.actions)
                .flatten()
        })
    }

    pub fn activation_time_value(&self) -> Option<&str> {
        self.activation.as_ref().and_then(|activation| {
            (activation.source_field == ActivationTimeSourceField::TimeValue)
                .then_some(activation.time.text.as_str())
        })
    }

    pub fn duration_time(&self) -> Option<&NormalizedTime> {
        self.duration.as_ref().map(|duration| &duration.time)
    }

    pub fn duration_value_text(&self) -> Option<&str> {
        self.duration
            .as_ref()
            .map(|duration| duration.time.text.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordActivationTiming {
    pub time: NormalizedTime,
    pub source_field: ActivationTimeSourceField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActivationTimeSourceField {
    ActionsValue,
    TimeValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordDurationTiming {
    pub time: NormalizedTime,
    pub source_field: DurationTimeSourceField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DurationTimeSourceField {
    DurationValue,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecordMechanics {
    pub metrics: Vec<MetricRow>,
    pub document: FoundryDocumentMechanics,
}

impl RecordMechanics {
    pub fn actor(&self) -> Option<&ActorMechanics> {
        match &self.document {
            FoundryDocumentMechanics::Actor(actor) => Some(actor),
            FoundryDocumentMechanics::Item(_) | FoundryDocumentMechanics::None => None,
        }
    }

    pub fn item(&self) -> Option<&ItemMechanics> {
        match &self.document {
            FoundryDocumentMechanics::Item(item) => Some(item),
            FoundryDocumentMechanics::Actor(_) | FoundryDocumentMechanics::None => None,
        }
    }

    pub fn spell(&self) -> Option<&SpellMechanics> {
        self.item().and_then(ItemMechanics::spell)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FoundryDocumentMechanics {
    Actor(ActorMechanics),
    Item(ItemMechanics),
    #[default]
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemTypeMechanics {
    Spell(SpellMechanics),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecordContent {
    pub documents: Vec<RecordContentDocument>,
}

impl RecordContent {
    pub fn description(&self) -> Option<&RichDocument> {
        self.document(ContentSourceKind::Description)
    }

    pub fn blurb(&self) -> Option<&RichDocument> {
        self.document(ContentSourceKind::Blurb)
    }

    pub fn primary_body(&self) -> Option<&RichDocument> {
        self.description().or_else(|| self.blurb())
    }

    pub fn document(&self, source_kind: ContentSourceKind) -> Option<&RichDocument> {
        self.documents
            .iter()
            .find(|content| content.source_kind == source_kind)
            .map(|content| &content.document)
    }

    pub fn searchable_documents(&self) -> impl Iterator<Item = &RecordContentDocument> {
        self.documents
            .iter()
            .filter(|content| content.contributes_to_search())
    }

    pub fn reference_occurrence_documents(&self) -> impl Iterator<Item = &RecordContentDocument> {
        self.documents
            .iter()
            .filter(|content| content.contributes_to_reference_occurrences())
    }

    pub fn default_backlink_documents(&self) -> impl Iterator<Item = &RecordContentDocument> {
        self.documents
            .iter()
            .filter(|content| content.contributes_to_default_backlinks())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordVariantMembership {
    pub group_key: String,
    pub base_name: String,
    pub label: Option<String>,
    pub axes: Vec<String>,
    pub confidence: Option<f64>,
    pub source: VariantSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariantSource {
    None,
    Parenthetical,
    NamePattern,
    CreatureBlurb,
    CreatureSuffix,
    ExactBase,
}

impl VariantSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Parenthetical => "parenthetical",
            Self::NamePattern => "name_pattern",
            Self::CreatureBlurb => "creature_blurb",
            Self::CreatureSuffix => "creature_suffix",
            Self::ExactBase => "exact_base",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "parenthetical" => Some(Self::Parenthetical),
            "name_pattern" => Some(Self::NamePattern),
            "creature_blurb" => Some(Self::CreatureBlurb),
            "creature_suffix" => Some(Self::CreatureSuffix),
            "exact_base" => Some(Self::ExactBase),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordVisibility {
    pub default_retrieval: DefaultRetrievalVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefaultRetrievalVisibility {
    Visible { reason: RecordVisibilityReason },
    Hidden { reason: RecordVisibilityReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecordVisibilityReason {
    SourceRecord,
    GeneratedCanonical,
    GeneratedInstance,
}

impl RecordVisibility {
    pub const fn visible(reason: RecordVisibilityReason) -> Self {
        Self {
            default_retrieval: DefaultRetrievalVisibility::Visible { reason },
        }
    }

    pub const fn hidden(reason: RecordVisibilityReason) -> Self {
        Self {
            default_retrieval: DefaultRetrievalVisibility::Hidden { reason },
        }
    }

    pub const fn visible_by_default(&self) -> bool {
        matches!(
            self.default_retrieval,
            DefaultRetrievalVisibility::Visible { .. }
        )
    }

    pub const fn reason(&self) -> RecordVisibilityReason {
        match self.default_retrieval {
            DefaultRetrievalVisibility::Visible { reason }
            | DefaultRetrievalVisibility::Hidden { reason } => reason,
        }
    }
}

impl Default for RecordVisibility {
    fn default() -> Self {
        Self::visible(RecordVisibilityReason::SourceRecord)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AtlasRecordSet {
    pub records: Vec<AtlasRecord>,
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
    pub relation_kind: ReferenceRelationKind,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActorMechanics {
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemMechanics {
    pub foundry_type: Option<ItemTypeMechanics>,
    pub category: Option<String>,
    pub base_item: Option<String>,
    pub group: Option<String>,
    pub usage: Option<String>,
    pub price_json: Option<String>,
    pub price_cp: Option<i64>,
    pub bulk_value: Option<f64>,
    pub hands_requirement: Option<String>,
    pub damage_types: Vec<String>,
}

impl ItemMechanics {
    pub fn spell(&self) -> Option<&SpellMechanics> {
        match &self.foundry_type {
            Some(ItemTypeMechanics::Spell(spell)) => Some(spell),
            None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpellMechanics {
    pub traditions: Vec<String>,
    pub kinds: Vec<String>,
    pub range: Option<SpellRange>,
    pub target: Option<SpellTarget>,
    pub area: Option<SpellArea>,
    pub defense: Option<SpellDefense>,
    pub sustained: bool,
    pub damage_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpellRange {
    pub text: String,
    pub distance: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpellTarget {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpellArea {
    pub kind: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpellDefense {
    pub save: Option<String>,
    pub basic: bool,
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
