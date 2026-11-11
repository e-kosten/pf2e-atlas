use crate::creature_catalog::{CreatureCatalog, CreatureFamily, Member, ScenarioUse};
use crate::family_axes::Coordinates;
use crate::family_axis_projection;
use crate::{
    AtlasRetrievalService, FamilyGrouping, GetRecordsRequest, SearchError, VariantAxisSlice,
};
use atlas_domain::{RecordKey, RecordKind, SourceRecordSummary};
use atlas_foundry_model::{ActorSourcePF2e, FoundryDocumentSource};
use atlas_record::source_record::SourceBackedRecord;
use std::sync::OnceLock;

pub use crate::creature_catalog::CreatureAdjustment;

#[derive(Debug)]
pub struct CreatureFamilyRecords {
    pub id: &'static str,
    pub name: &'static str,
    pub members: Vec<CreatureFamilyRecord>,
    pub groupings: Vec<FamilyGrouping>,
    pub all_variants_by_default: bool,
    pub version_slices: Option<CreatureVersionSlices>,
}

#[derive(Debug)]
pub struct CreatureVersionSlices {
    pub slices: Vec<VariantAxisSlice>,
    pub other_combinations: Vec<RecordKey>,
}

#[derive(Debug)]
pub struct CreatureFamilyRecord {
    pub summary: SourceRecordSummary,
    pub named_base: Option<&'static RecordKey>,
    pub qualifier: Option<String>,
    coordinates: &'static Coordinates,
}

pub struct CreatureDisplay {
    pub label: Option<String>,
    pub qualifier: Option<&'static str>,
    pub scenario_uses: Vec<CreatureScenarioUse>,
}

pub struct CreatureScenarioUse {
    pub id: &'static str,
    pub publication: String,
    pub party_levels: Option<[u8; 2]>,
    pub minimum_challenge_points: Option<u16>,
    pub qualifier: Option<&'static str>,
}

fn catalog() -> Option<&'static CreatureCatalog> {
    static CATALOG: OnceLock<Option<CreatureCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            match serde_json::from_str(include_str!(concat!(
                env!("OUT_DIR"),
                "/creature_families.json"
            ))) {
                Ok(catalog) => Some(catalog),
                Err(error) => {
                    eprintln!("invalid bundled creature families: {error}");
                    None
                }
            }
        })
        .as_ref()
}

fn member_for(key: &RecordKey) -> Option<(&'static str, &'static CreatureFamily, &'static Member)> {
    catalog()?.iter().find_map(|(id, family)| {
        let member = family.members.iter().find(|member| member.key == *key)?;
        Some((id.as_str(), family, member))
    })
}

fn matches(member: &Member, summary: &SourceRecordSummary) -> bool {
    member.key == summary.key
        && summary.record_kind.as_value() == Some(&RecordKind::Creature)
        && summary.name.as_value() == Some(&member.name)
}

pub fn creature_display(
    summary: &SourceRecordSummary,
    load_token_name: impl FnOnce() -> Option<String>,
) -> Option<CreatureDisplay> {
    let (_, _, member) = member_for(&summary.key)?;
    if !matches(member, summary) {
        return None;
    }
    let scenario_uses = member
        .uses
        .iter()
        .filter_map(|use_| {
            let title = summary.publication_title.as_value()?;
            let short_title = title
                .split_once(": ")
                .map_or(title.as_str(), |(_, title)| title);
            Some(CreatureScenarioUse {
                id: &use_.id,
                publication: short_title.into(),
                party_levels: use_.party_levels,
                minimum_challenge_points: use_.minimum_challenge_points,
                qualifier: use_.qualifier.as_deref(),
            })
        })
        .collect();
    let label = member.label.clone().or_else(|| {
        if member.uses.is_empty() {
            return None;
        }
        load_token_name().or_else(|| summary.name.as_value().cloned())
    });
    Some(CreatureDisplay {
        label,
        qualifier: member.qualifier.as_deref(),
        scenario_uses,
    })
}

pub fn creature_token_name(source: &FoundryDocumentSource) -> Option<String> {
    let FoundryDocumentSource::Actor(actor) = source else {
        return None;
    };
    let ActorSourcePF2e::NPCSource(npc) = actor.as_ref() else {
        return None;
    };
    npc.prototype_token
        .as_value()?
        .name
        .as_value()
        .filter(|name| !name.trim().is_empty())
        .cloned()
}

fn select_use<'a>(member: &'a Member, selected: Option<&str>) -> Option<&'a ScenarioUse> {
    match selected {
        Some(id) => member.uses.iter().find(|use_| use_.id == id),
        None => match member.uses.as_slice() {
            [use_] if use_.minimum_challenge_points.is_none() => Some(use_),
            _ => None,
        },
    }
}

impl AtlasRetrievalService {
    pub fn creature_display(
        &self,
        summary: &SourceRecordSummary,
        source: Option<&SourceBackedRecord>,
    ) -> Option<CreatureDisplay> {
        creature_display(summary, || match source {
            Some(source) => creature_token_name(source.source()),
            None => {
                let source = self.index.read_source_record(&summary.key).ok()??;
                creature_token_name(source.source())
            }
        })
    }

    pub fn creature_family_for(
        &self,
        summary: &SourceRecordSummary,
    ) -> Result<Option<CreatureFamilyRecords>, SearchError> {
        let Some((id, family, member)) = member_for(&summary.key) else {
            return Ok(None);
        };
        if !matches(member, summary) {
            return Ok(None);
        }
        Ok(self
            .read_creature_family(id, family, Some(&summary.key))?
            .filter(|family| {
                family
                    .members
                    .iter()
                    .any(|member| member.summary.key == summary.key)
            }))
    }

    pub fn creature_family(&self, id: &str) -> Result<Option<CreatureFamilyRecords>, SearchError> {
        let Some((id, family)) = catalog().and_then(|catalog| catalog.get_key_value(id)) else {
            return Ok(None);
        };
        self.read_creature_family(id, family, None)
    }

    fn read_creature_family(
        &self,
        id: &'static str,
        family: &'static CreatureFamily,
        seed: Option<&RecordKey>,
    ) -> Result<Option<CreatureFamilyRecords>, SearchError> {
        let keys: Vec<_> = family
            .members
            .iter()
            .map(|member| member.key.clone())
            .collect();
        let batch = self.get_records(GetRecordsRequest { record_keys: &keys })?;
        let mut members: Vec<_> = batch
            .records
            .into_iter()
            .filter_map(|summary| {
                let member = family
                    .members
                    .iter()
                    .find(|member| matches(member, &summary))?;
                Some(CreatureFamilyRecord {
                    summary,
                    named_base: member.named_base.as_ref(),
                    qualifier: if member.coordinates.is_empty() {
                        member.qualifier.clone()
                    } else {
                        Some(family_axis_projection::labels(
                            &family.axes,
                            &member.coordinates,
                        ))
                    },
                    coordinates: &member.coordinates,
                })
            })
            .collect();
        let verified_bases: std::collections::BTreeSet<_> = members
            .iter()
            .filter(|member| member.named_base.is_none())
            .map(|member| member.summary.key.clone())
            .collect();
        members.retain(|member| {
            member
                .named_base
                .is_none_or(|base| verified_bases.contains(base))
        });
        if members.len() < 2 || members.iter().all(|member| member.named_base.is_some()) {
            return Ok(None);
        }
        let concrete = members
            .iter()
            .filter(|member| member.named_base.is_none())
            .map(|member| (&member.summary.key, member.coordinates))
            .collect::<Vec<_>>();
        let groupings = family_axis_projection::groupings(&family.axes, &concrete);
        let version_slices = seed.filter(|_| concrete.len() > 1).and_then(|seed| {
            let index = concrete.iter().position(|(key, _)| *key == seed)?;
            let mut siblings = concrete;
            let (_, coordinates) = siblings.remove(index);
            let (slices, other_combinations) = if family.axes.is_empty() {
                (
                    vec![VariantAxisSlice {
                        heading: "Variants".into(),
                        members: siblings.into_iter().map(|(key, _)| key.clone()).collect(),
                    }],
                    Vec::new(),
                )
            } else {
                family_axis_projection::siblings(&family.axes, coordinates, &siblings)
            };
            Some(CreatureVersionSlices {
                slices,
                other_combinations,
            })
        });
        Ok(Some(CreatureFamilyRecords {
            id,
            name: &family.name,
            members,
            groupings,
            all_variants_by_default: family.all_variants_by_default,
            version_slices,
        }))
    }

    pub fn creature_adjustment_override(
        &self,
        base: &SourceRecordSummary,
        adjustment: CreatureAdjustment,
        scenario_use: Option<&str>,
    ) -> Result<Option<RecordKey>, SearchError> {
        let Some((_, family, member)) = member_for(&base.key) else {
            return Ok(None);
        };
        // The scenario choice is only presented when the source identifies the adventure.
        if !matches(member, base) || base.publication_title.as_value().is_none() {
            return Ok(None);
        }
        let Some(scope) = select_use(member, scenario_use) else {
            return Ok(None);
        };
        let Some(entry) = family.overrides.iter().find(|entry| {
            entry.base == base.key
                && entry.adjustment == adjustment
                && entry.scenario_use == scope.id
        }) else {
            return Ok(None);
        };
        let batch = self.get_records(GetRecordsRequest {
            record_keys: std::slice::from_ref(&entry.target),
        })?;
        let Some((_, _, target)) = member_for(&entry.target) else {
            return Ok(None);
        };
        Ok(batch
            .records
            .iter()
            .any(|summary| matches(target, summary))
            .then(|| entry.target.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_adventure_title_keeps_the_scenario_override_unavailable() {
        use crate::test_support::{open_source_fixture, record};
        use serde_json::json;

        let base = RecordKey::parse("pfs-season-1-bestiary:7PXPcv8x3Wnc730i").unwrap();
        let (retrieval, _fixture) = open_source_fixture(
            vec![
                record(
                    base.pack().as_str(),
                    "Actor",
                    json!({
                        "_id": base.id().as_str(), "type": "npc", "name": "Prefect (Fist) (3-4)",
                        "system": {"details": {"level": {"value": 3}}}
                    }),
                ),
                record(
                    base.pack().as_str(),
                    "Actor",
                    json!({
                        "_id": "UyJUEp0asfGDBvYa", "type": "npc", "name": "Elite Prefect (Fist) (3-4)",
                        "system": {"details": {
                            "level": {"value": 4},
                            "publication": {"title": "Pathfinder Society Quest #9: Wayfinder Origins"}
                        }}
                    }),
                ),
            ],
            false,
        )
        .unwrap();
        let summary = retrieval
            .get_records(GetRecordsRequest {
                record_keys: &[base],
            })
            .unwrap()
            .records
            .pop()
            .unwrap();
        assert!(
            creature_display(&summary, || None)
                .unwrap()
                .scenario_uses
                .is_empty()
        );
        assert!(retrieval.creature_family_for(&summary).unwrap().is_some());
        assert_eq!(
            retrieval
                .creature_adjustment_override(&summary, CreatureAdjustment::Elite, None)
                .unwrap(),
            None,
        );
    }

    #[test]
    fn a_sole_challenge_point_use_requires_explicit_selection() {
        let (_, _, actor) =
            member_for(&RecordKey::parse("pfs-season-3-bestiary:pYR3Ovkd9eKQQ5qE").unwrap())
                .unwrap();

        assert!(select_use(actor, None).is_none());
        assert_eq!(
            select_use(actor, Some("party-11-12-cp-28"))
                .unwrap()
                .minimum_challenge_points,
            Some(28),
        );
    }

    #[test]
    fn adamantine_dragon_groups_and_slices_only_verified_records() {
        use crate::test_support::{open_source_fixture, record};
        use serde_json::json;

        let family = &catalog().unwrap()["adamantine-dragon"];
        let seed = &family.members[3].key;
        let build = |renamed: bool| {
            let records = family
                .members
                .iter()
                .enumerate()
                .map(|(index, member)| {
                    let name = if renamed && index == 4 {
                        "Renamed dragon"
                    } else {
                        &member.name
                    };
                    record(
                        member.key.pack().as_str(),
                        "Actor",
                        json!({
                            "_id": member.key.id().as_str(), "type": "npc", "name": name,
                            "system": {"details": {"level": {"value": 13}}}
                        }),
                    )
                })
                .collect();
            open_source_fixture(records, false).unwrap()
        };
        let (retrieval, _fixture) = build(false);
        let group = retrieval
            .creature_family("adamantine-dragon")
            .unwrap()
            .unwrap();
        assert!(group.all_variants_by_default);
        let expected = |indexes: &[usize]| {
            indexes
                .iter()
                .map(|i| family.members[*i].key.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            group
                .groupings
                .iter()
                .map(|grouping| grouping.label.as_str())
                .collect::<Vec<_>>(),
            ["Age", "Form"]
        );
        assert_eq!(group.groupings[0].groups[1].members, expected(&[2, 3]));
        assert_eq!(group.groupings[1].groups[1].members, expected(&[1, 3, 5]));
        assert_eq!(
            group
                .members
                .iter()
                .map(|member| member.summary.key.clone())
                .collect::<Vec<_>>(),
            expected(&[0, 1, 2, 3, 4, 5])
        );
        let check_slices = |retrieval: &AtlasRetrievalService, others: &[usize]| {
            let summary = retrieval
                .get_records(GetRecordsRequest {
                    record_keys: std::slice::from_ref(seed),
                })
                .unwrap()
                .records
                .pop()
                .unwrap();
            let slices = retrieval
                .creature_family_for(&summary)
                .unwrap()
                .unwrap()
                .version_slices
                .unwrap();
            assert_eq!(slices.slices[0].heading, "Other ages");
            assert_eq!(slices.slices[0].members, expected(&[1, 5]));
            assert_eq!(slices.slices[1].heading, "Other forms");
            assert_eq!(slices.slices[1].members, expected(&[2]));
            assert_eq!(slices.other_combinations, expected(others));
        };
        check_slices(&retrieval, &[0, 4]);
        let (partial, _fixture) = build(true);
        check_slices(&partial, &[0]);
    }
}
