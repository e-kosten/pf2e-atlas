use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSheetResponse {
    pub character_key: String,
    pub character_revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub sheet: Option<CharacterSheetView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSheetView {
    pub name: String,
    pub level: i64,
    pub has_additional_details: bool,
    pub sections: Vec<SheetSectionView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SheetSectionKind {
    Stats,
    Progression,
    Magic,
    Inventory,
    Companions,
    Proficiencies,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSectionView {
    pub kind: SheetSectionKind,
    pub groups: Vec<SheetGroupView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetGroupView {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub kind: Option<SheetGroupKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub container_id: Option<String>,
    pub rows: Vec<SheetRowView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spell_ranks: Vec<SheetSpellRankView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SheetGroupKind {
    Skills,
    Proficiencies,
    Weapons,
    Armor,
    Gear,
    Container,
    Coins,
    FocusPool,
    Formulas,
    Advancement,
    SpecialAbilities,
    AbilityBoosts,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSpellRankView {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub slots: Option<i64>,
    pub known: Vec<SheetSpellView>,
    pub prepared: Vec<SheetSpellView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSpellView {
    pub label: String,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub ability_key: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRowView {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub kind: Option<SheetRowKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub summary_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub summary_help: Option<crate::ReferenceHelpView>,
    pub facts: Vec<SheetFactView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub ability_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub record_key: Option<RecordKey>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SheetFactView {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub display_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub role: Option<SheetFactRole>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SheetRowKind {
    ContainerMetadata,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SheetFactRole {
    Quantity,
    State,
    Category,
    Choice,
    AcquiredLevel,
    Primary,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportChangesView {
    pub had_sheet: bool,
    pub sections: Vec<ImportSectionChangesView>,
    pub removed_corrections: Vec<String>,
    pub retained_corrections: usize,
    pub additional_fields_changed: Vec<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportSectionChangesView {
    pub label: String,
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub removed: Vec<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDetailsView {
    pub fields: Vec<ImportFieldView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub previous_fields: Option<Vec<ImportFieldView>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportFieldView {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub display_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub scalar_kind: Option<ImportScalarKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub collection: Option<ImportCollectionKind>,
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub children: Vec<ImportFieldView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportCollectionKind {
    Object,
    Array,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportScalarKind {
    Text,
    Number,
    Boolean,
    SourceFragment,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewImportDetailsRequest {
    pub json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_character_revision: Option<i64>,
}
