use crate::VariantAxisSlice;
use crate::family_axes::{Axis, Coordinates};
use atlas_domain::RecordKey;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct FamilyGrouping {
    pub id: String,
    pub label: String,
    pub groups: Vec<VariantAxisSlice>,
}

pub(crate) fn labels(axes: &[Axis], values: &Coordinates) -> String {
    axes.iter()
        .filter_map(|axis| {
            axis.options
                .iter()
                .find(|option| values.get(&axis.id) == Some(&option.id))
        })
        .map(|option| option.label.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn groupings(
    axes: &[Axis],
    members: &[(&RecordKey, &Coordinates)],
) -> Vec<FamilyGrouping> {
    axes.iter()
        .map(|axis| FamilyGrouping {
            id: axis.id.clone(),
            label: axis.label.clone(),
            groups: axis
                .options
                .iter()
                .filter_map(|option| {
                    let members: Vec<_> = members
                        .iter()
                        .filter(|(_, values)| values.get(&axis.id) == Some(&option.id))
                        .map(|(key, _)| (*key).clone())
                        .collect();
                    (!members.is_empty()).then(|| VariantAxisSlice {
                        heading: option.label.clone(),
                        members,
                    })
                })
                .collect(),
        })
        .collect()
}

/// Partition verified siblings by exactly one changed coordinate.
pub(crate) fn siblings(
    axes: &[Axis],
    selected: &Coordinates,
    members: &[(&RecordKey, &Coordinates)],
) -> (Vec<VariantAxisSlice>, Vec<RecordKey>) {
    let mut displayed = BTreeSet::new();
    let slices = axes
        .iter()
        .filter_map(|axis| {
            let mut siblings = Vec::new();
            for option in &axis.options {
                if selected.get(&axis.id) == Some(&option.id) {
                    continue;
                }
                if let Some((key, _)) = members.iter().find(|(_, values)| {
                    values.get(&axis.id) == Some(&option.id)
                        && selected
                            .iter()
                            .all(|(id, value)| id == &axis.id || values.get(id) == Some(value))
                }) {
                    displayed.insert(*key);
                    siblings.push((*key).clone());
                }
            }
            (!siblings.is_empty()).then(|| VariantAxisSlice {
                heading: axis
                    .alternatives_label
                    .clone()
                    .unwrap_or_else(|| format!("Other {}", axis.label.to_lowercase())),
                members: siblings,
            })
        })
        .collect();
    let others = members
        .iter()
        .filter(|(key, _)| !displayed.contains(key))
        .map(|(key, _)| (*key).clone())
        .collect();
    (slices, others)
}
