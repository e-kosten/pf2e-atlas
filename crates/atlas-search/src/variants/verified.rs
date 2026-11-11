use crate::{AtlasRetrievalService, SearchError, SourceRecordDetail};
use atlas_domain::{RecordKey, SourceRecordSummary, normalize_record_name};
use atlas_record::source_record::{
    ItemSourceView, SourceBackedRecord, SourceFieldView, SourceNodeView,
};
use std::sync::LazyLock;

use super::catalog::{Catalog, Family, Fit, GeneralRecord, Member, valid_family_id};
use super::{
    Compatibility, EquipmentFamily, EquipmentFamilyGrouping, EquipmentFamilyOutcome,
    EquipmentFamilyReference, VariantAxisSlice, VariantFamilyChoice, VariantFamilyContext,
    VariantGroupResult, VariantSuggestionEvidence, axes, compatibility,
};

static CATALOG: LazyLock<Result<Catalog, serde_json::Error>> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/equipment_families.json"
    )))
});

fn catalog() -> Result<&'static Catalog, SearchError> {
    CATALOG.as_ref().map_err(|error| {
        SearchError::artifact_contract_violation(format!(
            "invalid equipment family catalog: {error}"
        ))
    })
}

pub(super) fn excluded(a: &RecordKey, b: &RecordKey) -> Result<bool, SearchError> {
    Ok(catalog()?.exclusions.iter().any(|pair| {
        (pair.members.contains(a) && pair.others.contains(b))
            || (pair.members.contains(b) && pair.others.contains(a))
    }))
}

#[derive(Clone, Copy)]
enum FamilySeed<'a> {
    Concrete(&'a Member),
    General(&'a GeneralRecord),
}

#[derive(Default)]
pub(super) struct VerifiedSelection {
    pub(super) group: Option<VariantGroupResult>,
    pub(super) context: Option<VariantFamilyContext>,
}

pub(super) fn verified_group(
    retrieval: &AtlasRetrievalService,
    seed: &SourceRecordSummary,
    include_family: bool,
) -> Result<VerifiedSelection, SearchError> {
    let Some(selected) = catalog_selection(&seed.key, include_family)? else {
        return Ok(VerifiedSelection::default());
    };
    let family = selected.family;
    let selection = selected.role;
    if !overview_matches(retrieval, &selected, seed, None)? {
        return Ok(VerifiedSelection::default());
    }
    let Some(reviewed) = reviewed_members(retrieval, family, None)? else {
        return Ok(VerifiedSelection::default());
    };
    if let FamilySeed::General(general) = selection {
        let members = reviewed
            .into_iter()
            .filter(|(member, _)| general.covers(member))
            .map(|(member, summary)| member_choice(family, member, summary))
            .collect();
        return Ok(VerifiedSelection {
            group: None,
            context: Some(VariantFamilyContext::General {
                heading: axes::general_heading(family, general),
                members,
            }),
        });
    }
    let context = if include_family && let FamilySeed::Concrete(concrete_seed) = selection {
        let general = match family
            .general_records
            .iter()
            .find(|entry| entry.covers(concrete_seed))
        {
            Some(entry) => read_general(retrieval, family, entry)?,
            None => None,
        };
        (!family.axes.is_empty() || general.is_some()).then(|| {
            axes::concrete_context(
                family,
                concrete_seed,
                reviewed.iter().map(|(member, _)| *member),
                general,
            )
        })
    } else {
        None
    };
    let (variants, evidence) = reviewed
        .into_iter()
        .map(|(member, summary)| {
            let evidence = VariantSuggestionEvidence {
                record: member.key.clone(),
                naming_convention: "reviewed_membership".into(),
                qualifier: member.display_label(family),
                compatibility: "reviewed equipment family".into(),
            };
            (summary, evidence)
        })
        .unzip();
    Ok(VerifiedSelection {
        group: Some(VariantGroupResult {
            seed: seed.clone(),
            base_name: family.name.clone(),
            variants,
            evidence,
            ambiguous: false,
            truncated: false,
            verified: true,
        }),
        context,
    })
}

struct CatalogSelection {
    id: &'static str,
    family: &'static Family,
    role: FamilySeed<'static>,
}

fn catalog_selection(
    key: &RecordKey,
    include_general: bool,
) -> Result<Option<CatalogSelection>, SearchError> {
    if key.pack().as_str() != "equipment-srd" {
        return Ok(None);
    }
    Ok(catalog()?.families.iter().find_map(|(id, family)| {
        let role = if let Some(member) = family.members.iter().find(|member| &member.key == key) {
            FamilySeed::Concrete(member)
        } else if include_general {
            FamilySeed::General(
                family
                    .general_records
                    .iter()
                    .find(|general| &general.key == key)?,
            )
        } else {
            return None;
        };
        Some(CatalogSelection { id, family, role })
    }))
}

fn overview_matches(
    retrieval: &AtlasRetrievalService,
    selected: &CatalogSelection,
    seed: &SourceRecordSummary,
    source: Option<&SourceBackedRecord>,
) -> Result<bool, SearchError> {
    match selected.role {
        FamilySeed::General(general) => record_matches(
            retrieval,
            selected.family,
            seed,
            &general.name,
            selected.family.category.as_deref(),
            None,
            source,
        ),
        FamilySeed::Concrete(_) => Ok(true),
    }
}

pub(super) fn family_reference(
    retrieval: &AtlasRetrievalService,
    seed: &SourceRecordDetail,
) -> Result<Option<EquipmentFamilyReference>, SearchError> {
    let Some(selected) = catalog_selection(&seed.summary.key, true)? else {
        return Ok(None);
    };
    if !overview_matches(retrieval, &selected, &seed.summary, Some(&seed.source))?
        || reviewed_members(retrieval, selected.family, Some(seed))?.is_none()
    {
        return Ok(None);
    }
    Ok(Some(EquipmentFamilyReference {
        id: selected.id.to_owned(),
        name: selected.family.name.clone(),
    }))
}

fn reviewed_members<'a>(
    retrieval: &AtlasRetrievalService,
    family: &'a Family,
    loaded: Option<&SourceRecordDetail>,
) -> Result<Option<Vec<(&'a Member, SourceRecordSummary)>>, SearchError> {
    let loaded_for = |member: &Member| loaded.filter(|detail| detail.summary.key == member.key);
    let keys = family
        .members
        .iter()
        .filter(|member| loaded_for(member).is_none())
        .map(|member| member.key.clone())
        .collect::<Vec<_>>();
    let batch = retrieval.index.read_summaries(&keys)?;
    if !batch.missing.is_empty() {
        return Ok(None);
    }
    let mut reviewed = Vec::with_capacity(family.members.len());
    let mut summaries = batch.records.into_iter();
    for member in &family.members {
        let loaded = loaded_for(member);
        let summary = if let Some(detail) = loaded {
            detail.summary.clone()
        } else if let Some(summary) = summaries.next() {
            summary
        } else {
            return Ok(None);
        };
        if !record_matches(
            retrieval,
            family,
            &summary,
            &member.name,
            member.category.as_deref().or(family.category.as_deref()),
            member.fit.as_ref(),
            loaded.map(|detail| &detail.source),
        )? {
            return Ok(None);
        }
        reviewed.push((member, summary));
    }

    // A declared progression follows its authored order. Named options sort by level.
    if family.order_by_level {
        reviewed.sort_by(|(_, a), (_, b)| {
            let level = |record: &SourceRecordSummary| {
                record
                    .level
                    .as_value()
                    .and_then(serde_json::Number::as_f64)
                    .unwrap_or(f64::INFINITY)
            };
            let name = |record: &SourceRecordSummary| {
                record
                    .name
                    .as_value()
                    .map_or_else(String::new, |name| normalize_record_name(name))
            };
            level(a)
                .total_cmp(&level(b))
                .then_with(|| name(a).cmp(&name(b)))
                .then_with(|| a.key.cmp(&b.key))
        });
    }
    Ok(Some(reviewed))
}

fn member_choice(
    family: &Family,
    member: &Member,
    summary: SourceRecordSummary,
) -> VariantFamilyChoice {
    VariantFamilyChoice {
        summary,
        qualifier: member.display_label(family),
    }
}

pub(super) fn equipment_family(
    retrieval: &AtlasRetrievalService,
    id: &str,
) -> Result<EquipmentFamilyOutcome, SearchError> {
    if !valid_family_id(id) {
        return Ok(EquipmentFamilyOutcome::Missing);
    }
    let Some(family) = catalog()?.families.get(id) else {
        return Ok(EquipmentFamilyOutcome::Missing);
    };
    let Some(reviewed) = reviewed_members(retrieval, family, None)? else {
        return Ok(EquipmentFamilyOutcome::Unavailable {
            name: family.name.clone(),
        });
    };
    let groupings = family
        .axes
        .iter()
        .map(|axis| EquipmentFamilyGrouping {
            id: axis.id.clone(),
            label: axis.label.clone(),
            groups: axis
                .options
                .iter()
                .filter_map(|option| {
                    let members: Vec<_> = reviewed
                        .iter()
                        .filter(|(member, _)| member.coordinates[&axis.id] == option.id)
                        .map(|(member, _)| member.key.clone())
                        .collect();
                    (!members.is_empty()).then(|| VariantAxisSlice {
                        heading: option.label.clone(),
                        members,
                    })
                })
                .collect(),
        })
        .collect();
    let mut overviews = Vec::new();
    for entry in &family.general_records {
        if let Some(summary) = read_general(retrieval, family, entry)? {
            let qualifier = axes::labels(family, &entry.scope);
            overviews.push(VariantFamilyChoice {
                summary: *summary,
                qualifier,
            });
        }
    }
    Ok(EquipmentFamilyOutcome::Content(EquipmentFamily {
        name: family.name.clone(),
        members: reviewed
            .into_iter()
            .map(|(member, summary)| member_choice(family, member, summary))
            .collect(),
        groupings,
        overviews,
    }))
}

fn read_general(
    retrieval: &AtlasRetrievalService,
    family: &Family,
    entry: &GeneralRecord,
) -> Result<Option<Box<SourceRecordSummary>>, SearchError> {
    let Some(summary) = retrieval.index.read_summary(&entry.key)? else {
        return Ok(None);
    };
    Ok(record_matches(
        retrieval,
        family,
        &summary,
        &entry.name,
        family.category.as_deref(),
        None,
        None,
    )?
    .then(|| Box::new(summary)))
}

fn token_matches(value: impl serde::Serialize, expected: Option<&str>) -> bool {
    serde_json::to_value(value)
        .ok()
        .as_ref()
        .and_then(serde_json::Value::as_str)
        == expected
}

fn name_matches(summary: &SourceRecordSummary, expected: &str) -> bool {
    summary
        .name
        .as_value()
        .is_some_and(|name| normalize_record_name(name) == normalize_record_name(expected))
}

fn record_matches(
    retrieval: &AtlasRetrievalService,
    family: &Family,
    summary: &SourceRecordSummary,
    expected_name: &str,
    category: Option<&str>,
    fit: Option<&Fit>,
    source: Option<&SourceBackedRecord>,
) -> Result<bool, SearchError> {
    if summary.document_kind != "Item"
        || summary.source_type.as_value() != Some(&family.item_type)
        || !name_matches(summary, expected_name)
    {
        return Ok(false);
    }
    if category.is_none() && fit.is_none() {
        return Ok(true);
    }
    let fetched;
    let record = if let Some(source) = source {
        source
    } else {
        fetched = retrieval.index.read_source_record(&summary.key)?;
        let Some(record) = &fetched else {
            return Ok(false);
        };
        record
    };
    if let Some(fit) = fit {
        let Some(SourceNodeView::Item(item @ ItemSourceView::AmmoSource(_))) = record.node_at(&[])
        else {
            return Ok(false);
        };
        return Ok(item
            .physical_base_type()
            .value()
            .is_some_and(|base| token_matches(base, Some(&fit.ammo_type)))
            && fit.matches(retrieval)?);
    }
    Ok(match compatibility(record) {
        Some(Compatibility::Weapon(value, group)) => {
            token_matches(value, category) && token_matches(group, family.group.as_deref())
        }
        Some(Compatibility::Armor(value)) => token_matches(value, category),
        Some(Compatibility::Consumable(value)) => token_matches(value, category),
        _ => false,
    })
}

impl Member {
    fn display_label(&self, family: &Family) -> String {
        match &self.fit {
            Some(fit) => format!(
                "Fits {}",
                fit.weapons
                    .iter()
                    .map(|weapon| weapon.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None if family.axes.is_empty() => self.attributes.join(", "),
            None => axes::qualifier(family, self),
        }
    }
}

impl Fit {
    fn matches(&self, retrieval: &AtlasRetrievalService) -> Result<bool, SearchError> {
        for weapon in &self.weapons {
            let Some(summary) = retrieval.index.read_summary(&weapon.key)? else {
                return Ok(false);
            };
            if !name_matches(&summary, &weapon.name) {
                return Ok(false);
            }
            let Some(record) = retrieval.index.read_source_record(&weapon.key)? else {
                return Ok(false);
            };
            let Some(SourceNodeView::Item(ItemSourceView::WeaponSource(source))) =
                record.node_at(&[])
            else {
                return Ok(false);
            };
            let base_type = SourceFieldView::from(&source.system)
                .and_then(|system| (&system.ammo).into())
                .and_then(|ammo| (&ammo.base_type).into());
            if !base_type
                .value()
                .is_some_and(|base| token_matches(base, Some(&self.ammo_type)))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{VariantGroupOutcome, VariantGroupRequest, test_support};
    use serde_json::json;
    use std::collections::BTreeSet;

    #[test]
    fn catalog_has_disjoint_family_members_and_supported_attributes() {
        let mut keys = BTreeSet::new();
        for family in catalog().unwrap().families.values() {
            assert!(!family.source_note.trim().is_empty());
            assert!(family.members.len() >= 2);
            for member in &family.members {
                assert_eq!(member.key.pack().as_str(), "equipment-srd");
                assert!(keys.insert(&member.key), "duplicate {}", member.key);
                assert!(!member.display_label(family).is_empty());
                if let Some(fit) = &member.fit {
                    assert!(!fit.weapons.is_empty());
                }
            }
        }
    }

    fn fixture(
        family: &Family,
        member: &Member,
        name: &str,
        level: Option<i32>,
    ) -> atlas_index::SourceArtifactRecordInput {
        test_support::record(
            member.key.pack().as_str(),
            "Item",
            json!({
                "_id": member.key.id().as_str(),
                "type": family.item_type,
                "name": name,
                "system": {
                    "category": member.category.as_ref().or(family.category.as_ref()),
                    "level": level.map(|value| json!({"value": value})),
                    "traits": {"value": [], "rarity": "common"}
                }
            }),
        )
    }

    fn variants(retrieval: &AtlasRetrievalService, key: &RecordKey) -> VariantGroupOutcome {
        retrieval
            .variant_group(VariantGroupRequest { record_key: key })
            .unwrap()
    }

    fn reference(
        retrieval: &AtlasRetrievalService,
        key: &RecordKey,
    ) -> Option<EquipmentFamilyReference> {
        let detail = retrieval
            .get_record(crate::GetRecordRequest {
                record_key: key,
                selected_content: &[],
            })
            .unwrap()?;
        retrieval
            .verified_equipment_family_reference(&detail)
            .unwrap()
    }

    #[test]
    fn reviewed_family_checks_the_exact_members_source_category() {
        let family = &catalog().unwrap().families["surging-serum"];
        let minor = family
            .members
            .iter()
            .find(|member| member.name == "Surging Serum (Minor)")
            .unwrap();
        let lesser = &family.members[1];
        for (category, verified) in [("other", true), ("elixir", false)] {
            let (retrieval, _artifact) = test_support::open_source_fixture(
                family
                    .members
                    .iter()
                    .map(|member| {
                        if member.key == minor.key {
                            return test_support::record(
                                "equipment-srd",
                                "Item",
                                json!({
                                    "_id": minor.key.id().as_str(),
                                    "type": "consumable",
                                    "name": minor.name,
                                    "system": {"category": category, "level": {"value": 2}}
                                }),
                            );
                        }
                        fixture(family, member, &member.name, Some(4))
                    })
                    .collect(),
                false,
            )
            .unwrap();
            let page = retrieval.equipment_family("surging-serum").unwrap();
            assert!(match page {
                EquipmentFamilyOutcome::Content(_) => verified,
                EquipmentFamilyOutcome::Unavailable { .. } => !verified,
                EquipmentFamilyOutcome::Missing => false,
            });
            for seed in [minor, lesser] {
                let VariantGroupOutcome::Content(group) = variants(&retrieval, &seed.key) else {
                    panic!("expected variants for {}", seed.name);
                };
                assert_eq!(group.verified, verified);
                assert_eq!(reference(&retrieval, &seed.key).is_some(), verified);
            }
        }
    }

    #[test]
    fn axes_catalog_rejects_ambiguous_or_invalid_declarations() {
        let family = &catalog().unwrap().families["potion-of-resistance"];
        let input = serde_json::to_string(family).unwrap();
        let parse = || serde_json::from_str::<Family>(&input).unwrap();
        let invalid = |family: Family, reason: &str| {
            let error = crate::variants::build_axes::validate(&family).unwrap_err();
            assert!(error.contains(reason), "{error}");
        };
        assert!(crate::variants::build_axes::validate(&parse()).is_ok());

        let mut family = parse();
        family.axes[1].id = family.axes[0].id.clone();
        invalid(family, "duplicate axis");

        let mut family = parse();
        family.axes[0].options[1].id = family.axes[0].options[0].id.clone();
        invalid(family, "duplicate axis or options");

        let mut family = parse();
        family.members[0].coordinates.remove("grade");
        invalid(family, "axis coordinates");

        let mut family = parse();
        family.members[0]
            .coordinates
            .insert("grade".into(), "major".into());
        invalid(family, "axis coordinates");

        let mut family = parse();
        family.members[1].coordinates = family.members[0].coordinates.clone();
        invalid(family, "duplicate axis coordinates");

        let mut family = parse();
        family.general_records[0].scope.clear();
        invalid(family, "overlapping general record scopes");

        let mut family = parse();
        family.general_records.truncate(1);
        family.general_records[0].scope.clear();
        assert!(crate::variants::build_axes::validate(&family).is_ok());
        assert_eq!(
            axes::general_heading(&family, &family.general_records[0]),
            "Variants"
        );

        let ring = &catalog().unwrap().families["specialists-ring"];
        let mut ring: Family = serde_json::from_value(serde_json::to_value(ring).unwrap()).unwrap();
        assert!(crate::variants::build_axes::validate(&ring).is_ok());
        let mut duplicate = serde_json::to_value(&ring.general_records[0]).unwrap();
        duplicate["key"] = json!("equipment-srd:xxxxxxxxxxxxxxxx");
        ring.general_records
            .push(serde_json::from_value(duplicate.clone()).unwrap());
        invalid(ring, "overlapping general record scopes");

        duplicate.as_object_mut().unwrap().remove("scope");
        let error = serde_json::from_value::<GeneralRecord>(duplicate)
            .err()
            .unwrap();
        assert!(
            error.to_string().contains("missing field `scope`"),
            "{error}"
        );

        let mut family = parse();
        family.general_records[0].scope = family.members[0].coordinates.clone();
        invalid(family, "invalid general record scope");

        let mut family = parse();
        family.general_records[0].scope = family.general_records[1].scope.clone();
        invalid(family, "overlapping general record scopes");

        let mut family = parse();
        family
            .members
            .retain(|member| member.coordinates["grade"] != "lesser");
        invalid(family, "covers no concrete members");
    }

    #[test]
    fn whole_family_overview_links_exact_records_and_keeps_flat_variants_visible() {
        let family = &catalog().unwrap().families["specialists-ring"];
        let overview = &family.general_records[0];
        let seed = &family.members[0];
        let expected: Vec<_> = family
            .members
            .iter()
            .map(|member| member.key.clone())
            .collect();
        assert_eq!(expected.len(), 8);
        let explore = |retrieval: &AtlasRetrievalService, record_key: &RecordKey| {
            retrieval
                .variant_exploration(VariantGroupRequest { record_key })
                .unwrap()
        };
        for overview_name in [Some(overview.name.as_str()), Some("Renamed ring"), None] {
            let mut records: Vec<_> = family
                .members
                .iter()
                .map(|member| fixture(family, member, &member.name, Some(10)))
                .collect();
            records.extend(overview_name.map(|name| {
                test_support::record(
                    "equipment-srd",
                    "Item",
                    json!({
                        "_id": overview.key.id().as_str(),
                        "type": family.item_type,
                        "name": name,
                        "system": {}
                    }),
                )
            }));
            let (retrieval, _artifact) = test_support::open_source_fixture(records, false).unwrap();
            let EquipmentFamilyOutcome::Content(page) =
                retrieval.equipment_family("specialists-ring").unwrap()
            else {
                panic!("expected the verified ring family");
            };
            let member_keys = |members: &[VariantFamilyChoice]| {
                members
                    .iter()
                    .map(|member| member.summary.key.clone())
                    .collect::<Vec<_>>()
            };
            assert_eq!(member_keys(&page.members), expected);
            assert!(page.groupings.is_empty());
            assert_eq!(
                reference(&retrieval, &seed.key).unwrap().id,
                "specialists-ring"
            );
            let concrete = explore(&retrieval, &seed.key);
            assert!(matches!(
                concrete.variants,
                VariantGroupOutcome::Content(group) if group.verified && group.variants.len() == 8
            ));
            let generic = explore(&retrieval, &overview.key);
            if overview_name == Some(overview.name.as_str()) {
                assert_eq!(page.overviews.len(), 1);
                assert_eq!(page.overviews[0].summary.key, overview.key);
                assert!(page.overviews[0].qualifier.is_empty());
                assert_eq!(
                    reference(&retrieval, &overview.key).unwrap().id,
                    "specialists-ring"
                );
                assert!(matches!(generic.variants, VariantGroupOutcome::Empty));
                let Some(VariantFamilyContext::General { heading, members }) = generic.family
                else {
                    panic!("expected the overview's concrete variants");
                };
                assert_eq!(heading, "Variants");
                assert_eq!(member_keys(&members), expected);
                let Some(VariantFamilyContext::Concrete {
                    slices,
                    other_combinations,
                    general,
                }) = concrete.family
                else {
                    panic!("expected the visible sibling slice and overview link");
                };
                assert_eq!(slices.len(), 1);
                assert_eq!(slices[0].heading, "Variants");
                assert_eq!(slices[0].members, expected[1..]);
                assert!(other_combinations.is_empty());
                assert_eq!(general.unwrap().key, overview.key);
            } else {
                assert!(page.overviews.is_empty());
                assert!(reference(&retrieval, &overview.key).is_none());
                assert!(generic.family.is_none());
                assert!(concrete.family.is_none());
            }
        }
    }

    #[test]
    fn resistance_axes_preserve_exact_members_and_verify_scoped_links_in_both_directions() {
        let family = &catalog().unwrap().families["potion-of-resistance"];
        let key = |id: &str| RecordKey::parse(&format!("equipment-srd:{id}")).unwrap();
        let cold = key("evdDpgFpiFZt9UyA");
        let moderate = &family.general_records[1];
        let build = |general_name: Option<&str>, drop_concrete: bool| {
            let mut records: Vec<_> = family
                .members
                .iter()
                .filter(|member| !drop_concrete || member.key != key("m99u0zXVbyPdK8Mf"))
                .map(|member| fixture(family, member, &member.name, Some(6)))
                .collect();
            records.extend(general_name.map(|name| {
                test_support::record(
                    "equipment-srd",
                    "Item",
                    json!({
                        "_id": moderate.key.id().as_str(),
                        "type": family.item_type,
                        "name": name,
                        "system": {"category": family.category}
                    }),
                )
            }));
            test_support::open_source_fixture(records, false).unwrap()
        };
        let explore = |retrieval: &AtlasRetrievalService, record_key: &RecordKey| {
            retrieval
                .variant_exploration(VariantGroupRequest { record_key })
                .unwrap()
        };
        for name in [Some(moderate.name.as_str()), Some("Renamed potion"), None] {
            let (retrieval, _artifact) = build(name, false);
            let EquipmentFamilyOutcome::Content(page) =
                retrieval.equipment_family("potion-of-resistance").unwrap()
            else {
                panic!("expected verified family page");
            };
            assert_eq!(page.members.len(), 15);
            assert_eq!(page.groupings[0].id, "damage-type");
            assert_eq!(
                page.groupings[0].groups[1].members,
                [
                    key("mRs24OwmGPjoIDvO"),
                    cold.clone(),
                    key("VZmUrseCZs9xMaxt"),
                ]
            );
            assert_eq!(page.groupings[1].id, "grade");
            assert_eq!(page.groupings[1].groups[1].members.len(), 5);
            let valid_overview = name == Some(moderate.name.as_str());
            assert_eq!(page.overviews.len(), usize::from(valid_overview));
            if let Some(overview) = page.overviews.first() {
                assert_eq!(overview.summary.key, moderate.key);
                assert_eq!(overview.qualifier, "Moderate");
            }
            let explored = explore(&retrieval, &cold);
            assert_eq!(
                reference(&retrieval, &cold)
                    .map(|value| value.name)
                    .as_deref(),
                Some("Potion of Resistance")
            );
            let VariantGroupOutcome::Content(flat) = explored.variants else {
                panic!("expected flat variants")
            };
            assert!(flat.verified);
            assert_eq!(flat.variants.len(), 15);
            let Some(VariantFamilyContext::Concrete {
                slices,
                other_combinations,
                general,
            }) = explored.family
            else {
                panic!("expected concrete context");
            };
            assert_eq!(slices[0].heading, "Other damage types");
            assert_eq!(
                slices[0].members,
                [
                    key("yG6Za8FaG3hpXUGh"),
                    key("MHoghQhxbvZRyyaB"),
                    key("m99u0zXVbyPdK8Mf"),
                    key("kbjspHV7kwKKULmd"),
                ]
            );
            assert_eq!(slices[1].heading, "Other grades");
            assert_eq!(
                slices[1].members,
                [key("mRs24OwmGPjoIDvO"), key("VZmUrseCZs9xMaxt")]
            );
            assert_eq!(other_combinations.len(), 8);
            assert_eq!(other_combinations[0], key("u1gZaIZHnEfZDc1o"));
            let reverse = explore(&retrieval, &moderate.key);
            assert_eq!(
                reference(&retrieval, &moderate.key)
                    .map(|value| value.id)
                    .as_deref(),
                valid_overview.then_some("potion-of-resistance")
            );
            if valid_overview {
                assert_eq!(general.unwrap().key, moderate.key);
                assert!(matches!(reverse.variants, VariantGroupOutcome::Empty));
                let Some(VariantFamilyContext::General { heading, members }) = reverse.family
                else {
                    panic!("expected general context");
                };
                assert_eq!(heading, "Damage types");
                assert_eq!(members.len(), 5);
                assert_eq!(members[1].summary.key, cold);
                assert_eq!(members[3].summary.key, key("m99u0zXVbyPdK8Mf"));
                assert_eq!(members[3].qualifier, "Fire, Moderate");
            } else {
                assert!(general.is_none());
                assert!(reverse.family.is_none());
            }
        }
        let (retrieval, _artifact) = build(Some(&moderate.name), true);
        let explored = explore(&retrieval, &cold);
        assert!(explored.family.is_none());
        assert!(reference(&retrieval, &cold).is_none());
        assert!(reference(&retrieval, &moderate.key).is_none());
        assert!(matches!(
            retrieval.equipment_family("potion-of-resistance").unwrap(),
            EquipmentFamilyOutcome::Unavailable { .. }
        ));
        assert!(
            matches!(explored.variants, VariantGroupOutcome::Content(group) if !group.verified)
        );
        assert!(explore(&retrieval, &moderate.key).family.is_none());
    }

    #[test]
    fn family_reference_reuses_loaded_source_and_skips_catalog_misses() {
        let family = &catalog().unwrap().families["alloy-orb"];
        let seed = &family.members[0];
        let other_key = RecordKey::parse("equipment-srd:xxxxxxxxxxxxxxxx").unwrap();
        let mut records: Vec<_> = family
            .members
            .iter()
            .map(|member| fixture(family, member, &member.name, Some(3)))
            .collect();
        records.push(test_support::record("equipment-srd", "Item", json!({
            "_id": other_key.id().as_str(), "type": "equipment", "name": "Unrelated item", "system": {}
        })));
        let (retrieval, _artifact) = test_support::open_source_fixture(records, false).unwrap();
        for (key, expected_id, summary_batches, body_decodes) in [
            (&seed.key, Some("alloy-orb"), 1, 4),
            (&other_key, None, 0, 0),
        ] {
            let detail = retrieval
                .get_record(crate::GetRecordRequest {
                    record_key: key,
                    selected_content: &[],
                })
                .unwrap()
                .unwrap();
            retrieval.reset_read_metrics();
            let found = retrieval
                .verified_equipment_family_reference(&detail)
                .unwrap();
            assert_eq!(found.as_ref().map(|value| value.id.as_str()), expected_id);
            let reads = retrieval.read_metrics();
            assert_eq!(reads.summary_batches, summary_batches);
            assert_eq!(reads.source_body_decodes, body_decodes);
            if expected_id.is_none() {
                assert_eq!(reads.sql_statements, 0);
            }
        }
    }

    #[test]
    fn sparse_alloy_orb_has_no_material_option_at_low_grade() {
        let family = &catalog().unwrap().families["alloy-orb"];
        let seed = &family.members[0];
        let VariantFamilyContext::Concrete {
            slices,
            other_combinations,
            ..
        } = axes::concrete_context(family, seed, family.members.iter(), None)
        else {
            unreachable!()
        };
        assert_eq!(slices.len(), 1);
        assert_eq!(slices[0].heading, "Other material grades");
        assert_eq!(
            slices[0].members,
            [&family.members[1], &family.members[3]].map(|m| m.key.clone())
        );
        assert_eq!(
            other_combinations,
            [&family.members[2], &family.members[4]].map(|m| m.key.clone())
        );
    }

    #[test]
    fn reviewed_membership_is_seed_independent_and_downgrades_as_a_group() {
        let family = &catalog().unwrap().families["wondrous-figurine"];
        let fixture_level = |member: &Member| match member.name.as_str() {
            "Wondrous Figurine (Onyx Dog)" => Some(10),
            "Wondrous Figurine (Golden Lions)" => Some(2),
            "Wondrous Figurine, Stuffed Fox" => None,
            _ => Some(10),
        };
        let (complete, _artifact) = test_support::open_source_fixture(
            family
                .members
                .iter()
                .map(|member| fixture(family, member, &member.name, fixture_level(member)))
                .collect(),
            false,
        )
        .unwrap();
        let mut ordered = family.members.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|member| (fixture_level(member).unwrap_or(i32::MAX), &member.name));
        let keys = ordered.iter().map(|member| &member.key).collect::<Vec<_>>();
        let EquipmentFamilyOutcome::Content(page) =
            complete.equipment_family("wondrous-figurine").unwrap()
        else {
            panic!("expected verified family page");
        };
        assert!(page.groupings.is_empty());
        assert_eq!(
            page.members
                .iter()
                .map(|member| &member.summary.key)
                .collect::<Vec<_>>(),
            keys
        );
        assert_eq!(
            reference(&complete, keys[0])
                .map(|value| value.id)
                .as_deref(),
            Some("wondrous-figurine")
        );

        for member in &family.members {
            let VariantGroupOutcome::Content(group) = variants(&complete, &member.key) else {
                panic!("expected reviewed siblings for {}", member.name);
            };
            assert!(group.verified);
            assert_eq!(
                group
                    .variants
                    .iter()
                    .map(|summary| &summary.key)
                    .collect::<Vec<_>>(),
                keys
            );
        }

        for include_renamed in [false, true] {
            let (stale, _artifact) = test_support::open_source_fixture(
                family
                    .members
                    .iter()
                    .filter_map(|member| {
                        if member.name.contains("Stuffed Fox") {
                            return include_renamed
                                .then(|| fixture(family, member, "Some Other Figurine", Some(0)));
                        }
                        Some(fixture(family, member, &member.name, fixture_level(member)))
                    })
                    .collect(),
                false,
            )
            .unwrap();
            for member in family
                .members
                .iter()
                .filter(|member| !member.name.contains("Stuffed Fox"))
            {
                let VariantGroupOutcome::Content(group) = variants(&stale, &member.key) else {
                    panic!("expected heuristic siblings for {}", member.name);
                };
                assert!(!group.verified);
                assert_eq!(group.variants.len(), 9);
            }
        }
    }

    #[test]
    fn kit_refill_exclusions_survive_a_missing_verified_member() {
        let catalog = catalog().unwrap();
        let family = &catalog.families["disguise-kit"];
        let kit = &family.members[0];
        let refills = &catalog.exclusions[0].others;
        let (retrieval, _artifact) = test_support::open_source_fixture(
            vec![
                fixture(family, kit, &kit.name, Some(0)),
                test_support::record(
                    "equipment-srd",
                    "Item",
                    json!({
                        "_id": refills[0].id().as_str(),
                        "type": "equipment",
                        "name": "Disguise Kit (Refill)",
                        "system": {}
                    }),
                ),
            ],
            false,
        )
        .unwrap();
        assert!(matches!(
            variants(&retrieval, &kit.key),
            VariantGroupOutcome::Empty
        ));
        assert!(matches!(
            variants(&retrieval, &refills[0]),
            VariantGroupOutcome::Empty
        ));
    }
}
