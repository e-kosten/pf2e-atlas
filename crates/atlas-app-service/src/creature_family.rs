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
        .iter()
        .map(|use_| CreatureScenarioUseView {
            id: use_.id.into(),
            label: scenario_use_label(use_, true)
                .or_else(|| record.publication_title.as_value().cloned())
                .unwrap_or_else(|| use_.publication.clone()),
        })
        .collect();
    let qualifiers = display
        .label
        .as_ref()
        .and(display.qualifier)
        .map(str::to_owned)
        .into_iter()
        .collect();
    let title = display.label.or_else(|| record.name.as_value().cloned())?;
    Some(CreatureDisplayView {
        title,
        qualifiers,
        scenario_context: scenario_context(&display.scenario_uses, &uses),
        uses,
    })
}

fn party_levels_label(range: [u8; 2]) -> String {
    format!("Party levels {}", party_range_label(range))
}

fn party_range_label([low, high]: [u8; 2]) -> String {
    format!("{low}–{high}")
}

fn scenario_context(
    uses: &[CreatureScenarioUse],
    labels: &[CreatureScenarioUseView],
) -> Option<String> {
    if labels.is_empty() {
        return None;
    }
    if uses.len() > 1 && uses.iter().all(|use_| use_.party_levels.is_some()) {
        let ranges = uses.iter().filter_map(|use_| {
            let range = party_range_label(use_.party_levels?);
            Some(match scenario_use_label(use_, false) {
                Some(conditions) => format!("{range} ({conditions})"),
                None => range,
            })
        });
        return Some(format!(
            "Party levels {}",
            ranges.collect::<Vec<_>>().join(" · ")
        ));
    }
    Some(
        labels
            .iter()
            .map(|use_| use_.label.as_str())
            .collect::<Vec<_>>()
            .join(" · "),
    )
}

fn scenario_use_label(use_: &CreatureScenarioUse, include_range: bool) -> Option<String> {
    let mut labels = Vec::new();
    let range = use_.party_levels.filter(|_| include_range);
    if let Some(range) = range {
        let label = party_levels_label(range);
        labels.push(match use_.qualifier {
            Some(qualifier) => format!("{label} ({qualifier})"),
            None => label,
        });
    }
    if let Some(points) = use_.minimum_challenge_points {
        labels.push(format!("{points}+ Challenge Points"));
    }
    if range.is_none()
        && let Some(qualifier) = use_.qualifier
    {
        labels.push(qualifier.into());
    }
    (!labels.is_empty()).then(|| labels.join(" · "))
}

fn party_level_grouping<'a>(
    records: impl IntoIterator<Item = (&'a str, Option<atlas_search::CreatureDisplay>)>,
) -> Option<CreatureFamilyGroupingView> {
    let mut by_range = BTreeMap::<_, CreatureFamilyGroupView>::new();
    let mut general = Vec::new();
    let mut unavailable = Vec::new();
    for (key, display) in records {
        if display
            .as_ref()
            .is_some_and(|display| display.scenario_uses_unavailable)
        {
            unavailable.push(key.into());
            continue;
        }
        let Some(display) = display.filter(|display| !display.scenario_uses.is_empty()) else {
            general.push(key.into());
            continue;
        };
        let mut ranges = BTreeMap::<_, Vec<String>>::new();
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
                if let Some(qualifier) = display.qualifier {
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
    let general = (!general.is_empty()).then_some(CreatureFamilyGroupView {
        heading: "General stat blocks".into(),
        members: general,
        qualifiers: BTreeMap::new(),
    });
    let unavailable = (!unavailable.is_empty()).then_some(CreatureFamilyGroupView {
        heading: "Scenario context unavailable".into(),
        members: unavailable,
        qualifiers: BTreeMap::new(),
    });
    Some(CreatureFamilyGroupingView {
        id: "scenario:party-levels".into(),
        label: "Party levels".into(),
        groups: general
            .into_iter()
            .chain(by_range.into_values())
            .chain(unspecified)
            .chain(unavailable)
            .collect(),
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
                    scenario_context: None,
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
    fn scenario_context_labels_keep_each_qualifier_with_its_party_range() {
        let use_ = |party_levels, qualifier, minimum_challenge_points| CreatureScenarioUse {
            id: "use",
            publication: "Cavern".into(),
            party_levels,
            minimum_challenge_points,
            qualifier,
        };
        assert_eq!(
            scenario_use_label(&use_(Some([9, 10]), Some("Advanced"), None), true).as_deref(),
            Some("Party levels 9–10 (Advanced)"),
        );
        assert_eq!(
            scenario_use_label(&use_(Some([11, 12]), None, Some(28)), true).as_deref(),
            Some("Party levels 11–12 · 28+ Challenge Points"),
        );
        assert_eq!(scenario_use_label(&use_(None, None, None), true), None);
        let uses = [
            use_(Some([9, 10]), Some("Advanced"), None),
            use_(Some([11, 12]), None, Some(28)),
        ];
        let labels = uses.each_ref().map(|use_| CreatureScenarioUseView {
            id: use_.id.into(),
            label: scenario_use_label(use_, true).unwrap(),
        });
        assert_eq!(
            scenario_context(&uses, &labels).as_deref(),
            Some("Party levels 9–10 (Advanced) · 11–12 (28+ Challenge Points)"),
        );
    }

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
                    scenario_uses_unavailable: false,
                    scenario_uses: vec![
                        use_(Some([11, 12]), Some(28)),
                        use_(Some([9, 10]), None),
                        use_(Some([11, 12]), Some(28)),
                        use_(Some([11, 12]), Some(30)),
                        use_(None, None),
                    ],
                }),
            ),
            ("general", None),
            (
                "unavailable",
                Some(CreatureDisplay {
                    label: None,
                    qualifier: None,
                    scenario_uses: vec![],
                    scenario_uses_unavailable: true,
                }),
            ),
        ])
        .unwrap();
        let groups = group.groups;
        assert_eq!(
            groups
                .iter()
                .map(|group| group.heading.as_str())
                .collect::<Vec<_>>(),
            [
                "General stat blocks",
                "Party levels 9–10",
                "Party levels 11–12",
                "Party range unspecified",
                "Scenario context unavailable",
            ]
        );
        assert_eq!(groups[0].members, ["general"]);
        assert_eq!(groups[1].members, ["multi"]);
        assert_eq!(groups[2].members, ["multi"]);
        assert_eq!(
            groups[2].qualifiers["multi"],
            [
                "Cavern · 28+ Challenge Points",
                "Cavern · 30+ Challenge Points"
            ]
        );
        assert_eq!(groups[3].members, ["multi"]);
        assert_eq!(groups[4].members, ["unavailable"]);
    }
}
