use crate::family_axis_projection as projection;
use atlas_domain::SourceRecordSummary;

use super::catalog::{Family, GeneralRecord, Member};
use super::{VariantAxisSlice, VariantFamilyContext};

pub(super) fn qualifier(family: &Family, member: &Member) -> String {
    projection::labels(&family.axes, &member.coordinates)
}

pub(super) fn general_heading(family: &Family, general: &GeneralRecord) -> String {
    if general.scope.is_empty() {
        return "Variants".into();
    }
    let mut free_axes = family
        .axes
        .iter()
        .filter(|axis| !general.scope.contains_key(&axis.id));
    match (free_axes.next(), free_axes.next()) {
        (Some(axis), None) => axis.label.clone(),
        _ => "Specific forms".into(),
    }
}

pub(super) fn concrete_context<'a>(
    family: &Family,
    seed: &Member,
    ordered: impl Iterator<Item = &'a Member>,
    general: Option<Box<SourceRecordSummary>>,
) -> VariantFamilyContext {
    let siblings = ordered
        .filter(|member| member.key != seed.key)
        .collect::<Vec<_>>();
    if family.axes.is_empty() {
        return VariantFamilyContext::Concrete {
            slices: vec![VariantAxisSlice {
                heading: "Variants".into(),
                members: siblings
                    .into_iter()
                    .map(|member| member.key.clone())
                    .collect(),
            }],
            other_combinations: Vec::new(),
            general,
        };
    }
    let members = siblings
        .iter()
        .map(|member| (&member.key, &member.coordinates))
        .collect::<Vec<_>>();
    let (slices, other_combinations) =
        projection::siblings(&family.axes, &seed.coordinates, &members);
    VariantFamilyContext::Concrete {
        slices,
        other_combinations,
        general,
    }
}
