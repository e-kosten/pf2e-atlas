use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::SourceRecordSummary;

use super::catalog::{Family, GeneralRecord, Member};
use super::{VariantAxisSlice, VariantFamilyContext};

pub(super) fn qualifier(family: &Family, member: &Member) -> String {
    labels(family, &member.coordinates)
}

pub(super) fn labels(family: &Family, values: &BTreeMap<String, String>) -> String {
    family
        .axes
        .iter()
        .flat_map(|axis| {
            axis.options
                .iter()
                .find(|option| values.get(&axis.id) == Some(&option.id))
        })
        .map(|option| option.label.as_str())
        .collect::<Vec<_>>()
        .join(", ")
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
    let mut displayed = BTreeSet::new();
    let mut slices = Vec::new();
    for axis in &family.axes {
        let mut members = Vec::new();
        for option in &axis.options {
            if option.id == seed.coordinates[&axis.id] {
                continue;
            }
            if let Some(member) = siblings.iter().find(|member| {
                member.coordinates[&axis.id] == option.id
                    && seed
                        .coordinates
                        .iter()
                        .all(|(id, value)| id == &axis.id || member.coordinates[id] == *value)
            }) {
                displayed.insert(&member.key);
                members.push(member.key.clone());
            }
        }
        if !members.is_empty() {
            slices.push(VariantAxisSlice {
                heading: format!("Other {}", axis.label.to_lowercase()),
                members,
            });
        }
    }
    let other_combinations = siblings
        .iter()
        .filter(|member| !displayed.contains(&member.key))
        .map(|member| member.key.clone())
        .collect();
    VariantFamilyContext::Concrete {
        slices,
        other_combinations,
        general,
    }
}
