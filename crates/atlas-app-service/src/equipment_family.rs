use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppErrorCode, EquipmentFamilyGroupingView, EquipmentFamilyView, VariantAxisSliceView,
    VariantFamilyChoiceView,
};
use atlas_search::{
    AtlasRetrievalService, EquipmentFamilyOutcome, VariantAxisSlice, VariantFamilyChoice,
};

impl AtlasAppService {
    pub fn equipment_family(&self, id: String) -> AppServiceResult<EquipmentFamilyView> {
        self.with_retrieval(
            &ReadExecutionContext::uncancellable(),
            move |retrieval| match retrieval.equipment_family(&id)? {
                EquipmentFamilyOutcome::Missing => Err(AppServiceError::new(
                    AppErrorCode::EquipmentFamilyNotFound,
                    "Equipment family not found",
                )),
                EquipmentFamilyOutcome::Unavailable { name } => {
                    Ok(EquipmentFamilyView::Unavailable { name })
                }
                EquipmentFamilyOutcome::Content(family) => Ok(EquipmentFamilyView::Content {
                    name: family.name,
                    all_variants_by_default: family.all_variants_by_default,
                    members: family
                        .members
                        .into_iter()
                        .map(|choice| family_choice(retrieval, choice))
                        .collect(),
                    groupings: family
                        .groupings
                        .into_iter()
                        .map(|grouping| EquipmentFamilyGroupingView {
                            id: grouping.id,
                            label: grouping.label,
                            groups: grouping.groups.into_iter().map(family_slice).collect(),
                        })
                        .collect(),
                    overviews: family
                        .overviews
                        .into_iter()
                        .map(|choice| family_choice(retrieval, choice))
                        .collect(),
                }),
            },
        )
    }
}

pub(crate) fn family_choice(
    retrieval: &AtlasRetrievalService,
    choice: VariantFamilyChoice,
) -> VariantFamilyChoiceView {
    VariantFamilyChoiceView {
        record: crate::projection::localized_summary_view(&choice.summary, retrieval),
        qualifier: choice.qualifier,
    }
}

pub(crate) fn family_slice(slice: VariantAxisSlice) -> VariantAxisSliceView {
    VariantAxisSliceView {
        heading: slice.heading,
        members: slice.members.iter().map(ToString::to_string).collect(),
    }
}
