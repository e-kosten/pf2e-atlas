use crate::{AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    CreatureDisplayView, CreatureFamilyGroupView, CreatureFamilyGroupingView, CreatureFamilyView,
    CreatureScenarioUseView, VariantFamilyView,
};
use atlas_domain::SourceRecordSummary;
use atlas_search::{
    AtlasRetrievalService, CreatureDisplay, CreatureFamilyRecords, CreatureScenarioUse,
};
use std::collections::BTreeMap;

pub(crate) fn creature_display(
    record: &SourceRecordSummary,
    display: CreatureDisplay,
) -> Option<CreatureDisplayView> {
    if display.label.is_none() && display.qualifier.is_none() && display.scenario_uses.is_empty() {
        return None;
    }
    let uses: Vec<_> = display
        .scenario_uses
        .into_iter()
        .map(|use_| {
            let label = scenario_use_label(&use_, true).unwrap_or_else(|| use_.publication.clone());
            (
                use_.publication,
                CreatureScenarioUseView {
                    id: use_.id.into(),
                    label,
                },
            )
        })
        .collect();
    let mut qualifiers: Vec<_> = display
        .label
        .as_ref()
        .and(display.qualifier)
        .map(str::to_owned)
        .into_iter()
        .collect();
    match uses.as_slice() {
        [(publication, single)] if publication != &single.label => {
            qualifiers.push(format!("{publication} · {}", single.label));
        }
        [(_, single)] => qualifiers.push(single.label.clone()),
        [_, _, ..] => qualifiers.push("Multiple scenario uses".into()),
        [] => (),
    }
    let title = display.label.or_else(|| record.name.as_value().cloned())?;
    Some(CreatureDisplayView {
        title,
        qualifiers,
        uses: uses.into_iter().map(|(_, use_)| use_).collect(),
    })
}

fn party_levels_label([low, high]: [u8; 2]) -> String {
    format!("Party levels {low}–{high}")
}

fn scenario_use_label(use_: &CreatureScenarioUse, include_range: bool) -> Option<String> {
    let mut labels = Vec::new();
    if include_range && let Some(range) = use_.party_levels {
        labels.push(party_levels_label(range));
    }
    if let Some(points) = use_.minimum_challenge_points {
        labels.push(format!("{points}+ Challenge Points"));
    }
    if let Some(qualifier) = use_.qualifier {
        labels.push(qualifier.into());
    }
    (!labels.is_empty()).then(|| labels.join(" · "))
}

fn party_level_grouping<'a>(
    records: impl IntoIterator<Item = (&'a str, Option<atlas_search::CreatureDisplay>)>,
) -> Option<CreatureFamilyGroupingView> {
    let mut by_range = BTreeMap::<_, CreatureFamilyGroupView>::new();
    for (key, display) in records {
        let mut ranges = BTreeMap::<_, Vec<String>>::new();
        if let Some(display) = &display {
            for use_ in &display.scenario_uses {
                let labels = ranges.entry(use_.party_levels).or_default();
                let context = scenario_use_label(use_, false).map_or_else(
                    || use_.publication.clone(),
                    |label| format!("{} · {label}", use_.publication),
                );
                if !labels.contains(&context) {
                    labels.push(context);
                }
            }
        }
        if ranges.is_empty() {
            ranges.insert(None, Vec::new());
        }
        for (range, mut labels) in ranges {
            let group = by_range
                .entry(range)
                .or_insert_with(|| CreatureFamilyGroupView {
                    heading: range
                        .map_or_else(|| "Party range unspecified".into(), party_levels_label),
                    members: Vec::new(),
                    qualifiers: BTreeMap::new(),
                });
            group.members.push(key.into());
            if !labels.is_empty() {
                if let Some(qualifier) = display.as_ref().and_then(|display| display.qualifier) {
                    labels.insert(0, qualifier.into());
                }
                group.qualifiers.insert(key.into(), labels);
            }
        }
    }
    let unspecified = by_range.remove(&None);
    if by_range.len() < 2 {
        return None;
    }
    Some(CreatureFamilyGroupingView {
        id: "scenario:party-levels".into(),
        label: "Party levels".into(),
        groups: by_range.into_values().chain(unspecified).collect(),
    })
}

pub(crate) fn family_view(
    retrieval: &AtlasRetrievalService,
    family: CreatureFamilyRecords,
) -> CreatureFamilyView {
    let mut versions = Vec::new();
    let mut named_creatures = Vec::new();
    let mut named_bases = BTreeMap::new();
    let mut scenario_displays = Vec::new();
    for member in family.members {
        let mut record = crate::projection::localized_summary_view(&member.summary, retrieval);
        if let Some(qualifier) = member.qualifier
            && record.title != qualifier
            && !record
                .creature_display
                .as_ref()
                .is_some_and(|display| display.qualifiers.iter().any(|value| value == &qualifier))
        {
            let display = record
                .creature_display
                .get_or_insert_with(|| CreatureDisplayView {
                    title: record.title.clone(),
                    qualifiers: Vec::new(),
                    uses: Vec::new(),
                });
            display.qualifiers.insert(0, qualifier);
        }
        if let Some(base) = member.named_base {
            named_bases.insert(record.record_key.clone(), base.to_string());
            named_creatures.push(record);
        } else {
            scenario_displays.push(atlas_search::creature_display(&member.summary, || None));
            versions.push(record);
        }
    }
    let groupings = family
        .groupings
        .into_iter()
        .map(|grouping| CreatureFamilyGroupingView {
            id: grouping.id,
            label: grouping.label,
            groups: grouping
                .groups
                .into_iter()
                .map(|group| CreatureFamilyGroupView {
                    heading: group.heading,
                    members: group.members.iter().map(ToString::to_string).collect(),
                    qualifiers: BTreeMap::new(),
                })
                .collect(),
        })
        .chain(party_level_grouping(
            versions
                .iter()
                .map(|record| record.record_key.as_str())
                .zip(scenario_displays),
        ))
        .collect();
    let version_context = family
        .version_slices
        .map(|context| VariantFamilyView::Concrete {
            slices: context
                .slices
                .into_iter()
                .map(crate::equipment_family::family_slice)
                .collect(),
            other_combinations: context
                .other_combinations
                .iter()
                .map(ToString::to_string)
                .collect(),
            general: None,
        });
    CreatureFamilyView {
        id: family.id.into(),
        name: family.name.into(),
        versions,
        named_creatures,
        named_bases,
        groupings,
        all_variants_by_default: family.all_variants_by_default,
        version_context,
    }
}

impl AtlasAppService {
    pub fn creature_family(&self, id: String) -> AppServiceResult<Option<CreatureFamilyView>> {
        self.with_retrieval(&ReadExecutionContext::uncancellable(), move |retrieval| {
            Ok(retrieval
                .creature_family(&id)?
                .map(|family| family_view(retrieval, family)))
        })
    }
}

pub(crate) fn overrides(
    retrieval: &AtlasRetrievalService,
    record: &SourceRecordSummary,
    display: Option<&CreatureDisplayView>,
) -> AppServiceResult<Vec<atlas_app_model::CreatureOverrideView>> {
    use atlas_app_model::{CreatureOverrideView, EncounterParticipantVariantView as Variant};
    use atlas_search::CreatureAdjustment;

    let mut overrides = Vec::new();
    for use_ in display.into_iter().flat_map(|display| &display.uses) {
        for (adjustment, variant) in [
            (CreatureAdjustment::Weak, Variant::Weak),
            (CreatureAdjustment::Elite, Variant::Elite),
        ] {
            if let Some(target) =
                retrieval.creature_adjustment_override(record, adjustment, Some(&use_.id))?
            {
                overrides.push(CreatureOverrideView {
                    use_id: use_.id.clone(),
                    adjustment: variant,
                    record_key: target.to_string(),
                });
            }
        }
    }
    Ok(overrides)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn party_level_groups_repeat_actors_only_across_distinct_uses() {
        let use_ = |party_levels, minimum_challenge_points| CreatureScenarioUse {
            id: "use",
            publication: "Cavern".into(),
            party_levels,
            minimum_challenge_points,
            qualifier: None,
        };
        let group = party_level_grouping([
            (
                "multi",
                Some(CreatureDisplay {
                    label: Some("Eltha".into()),
                    qualifier: None,
                    scenario_uses: vec![
                        use_(Some([11, 12]), Some(28)),
                        use_(Some([9, 10]), None),
                        use_(Some([11, 12]), Some(28)),
                        use_(Some([11, 12]), Some(30)),
                        use_(None, None),
                    ],
                }),
            ),
            ("unknown", None),
        ])
        .unwrap();
        let groups = group.groups;
        assert_eq!(
            groups
                .iter()
                .map(|group| group.heading.as_str())
                .collect::<Vec<_>>(),
            [
                "Party levels 9–10",
                "Party levels 11–12",
                "Party range unspecified"
            ]
        );
        assert_eq!(groups[0].members, ["multi"]);
        assert_eq!(groups[1].members, ["multi"]);
        assert_eq!(
            groups[1].qualifiers["multi"],
            [
                "Cavern · 28+ Challenge Points",
                "Cavern · 30+ Challenge Points"
            ]
        );
        assert_eq!(groups[2].members, ["multi", "unknown"]);
    }
}
