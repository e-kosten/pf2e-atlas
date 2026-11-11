use crate::{VariantAxisSliceView, VariantFamilyChoiceView};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EquipmentFamilyView {
    Unavailable {
        name: String,
    },
    Content {
        name: String,
        members: Vec<VariantFamilyChoiceView>,
        groupings: Vec<EquipmentFamilyGroupingView>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        #[cfg_attr(feature = "openapi", schema(required = false))]
        all_variants_by_default: bool,
        overviews: Vec<VariantFamilyChoiceView>,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquipmentFamilyGroupingView {
    pub id: String,
    pub label: String,
    pub groups: Vec<VariantAxisSliceView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquipmentFamilyReferenceView {
    pub id: String,
    pub name: String,
}
