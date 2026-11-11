use crate::{AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{CreatureDisplayView, CreatureFamilyView, CreatureScenarioUseView};
use atlas_domain::SourceRecordSummary;
use atlas_search::{AtlasRetrievalService, CreatureFamilyRecords};

pub(crate) fn creature_display(record: &SourceRecordSummary) -> Option<CreatureDisplayView> {
    let display = atlas_search::creature_display(record)?;
    if display.label.is_none() && display.qualifier.is_none() && display.scenario_uses.is_empty() {
        return None;
    }
    let title = display
        .label
        .map(str::to_owned)
        .or_else(|| record.name.as_value().cloned())?;
    let uses: Vec<_> = display
        .scenario_uses
        .into_iter()
        .map(|use_| {
            let mut labels = Vec::new();
            if let Some([low, high]) = use_.party_levels {
                labels.push(format!("Party levels {low}–{high}"));
            }
            if let Some(points) = use_.minimum_challenge_points {
                labels.push(format!("{points}+ Challenge Points"));
            }
            if let Some(qualifier) = use_.qualifier {
                labels.push(qualifier.into());
            }
            let label = if labels.is_empty() {
                use_.publication.clone()
            } else {
                labels.join(" · ")
            };
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
    Some(CreatureDisplayView {
        title,
        qualifiers,
        uses: uses.into_iter().map(|(_, use_)| use_).collect(),
    })
}

pub(crate) fn family_view(
    retrieval: &AtlasRetrievalService,
    family: CreatureFamilyRecords,
) -> CreatureFamilyView {
    let mut versions = Vec::new();
    let mut named_creatures = Vec::new();
    let mut named_bases = std::collections::BTreeMap::new();
    for member in family.members {
        let mut record = crate::projection::localized_summary_view(&member.summary, retrieval);
        if let Some(qualifier) = member.qualifier
            && record.title != qualifier
            && !record
                .creature_display
                .as_ref()
                .is_some_and(|display| display.qualifiers.iter().any(|value| value == qualifier))
        {
            let display = record
                .creature_display
                .get_or_insert_with(|| CreatureDisplayView {
                    title: record.title.clone(),
                    qualifiers: Vec::new(),
                    uses: Vec::new(),
                });
            display.qualifiers.insert(0, qualifier.into());
        }
        if let Some(base) = member.named_base {
            named_bases.insert(record.record_key.clone(), base.to_string());
            named_creatures.push(record);
        } else {
            versions.push(record);
        }
    }
    CreatureFamilyView {
        id: family.id.into(),
        name: family.name.into(),
        versions,
        named_creatures,
        named_bases,
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
