use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::{RecordKey, SourceRecordSummary, normalize_record_name};
use atlas_record::source_record::{ItemSourceView, SourceFieldView, SourceNodeView};
use std::sync::LazyLock;

use super::catalog::{Catalog, Family, Fit, GeneralRecord, Member};
use super::{
    Compatibility, VariantFamilyContext, VariantGroupResult, VariantSuggestionEvidence, axes,
    compatibility,
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

enum FamilySeed<'a> {
    Concrete(&'a Member),
    General(&'a GeneralRecord),
}

pub(super) fn verified_group(
    retrieval: &AtlasRetrievalService,
    seed: &SourceRecordSummary,
    include_family: bool,
) -> Result<(Option<VariantGroupResult>, Option<VariantFamilyContext>), SearchError> {
    let Some((family, selection)) = catalog()?.families.values().find_map(|family| {
        if let Some(member) = family.members.iter().find(|member| member.key == seed.key) {
            return Some((family, FamilySeed::Concrete(member)));
        }
        if include_family {
            return family
                .general_records
                .iter()
                .find(|general| general.key == seed.key)
                .map(|general| (family, FamilySeed::General(general)));
        }
        None
    }) else {
        return Ok((None, None));
    };

    if let FamilySeed::General(general) = selection
        && !record_matches(retrieval, family, seed, &general.name, None)?
    {
        return Ok((None, None));
    }
    let keys = family
        .members
        .iter()
        .map(|member| member.key.clone())
        .collect::<Vec<_>>();
    let batch = retrieval.index.read_summaries(&keys)?;
    if !batch.missing.is_empty() {
        return Ok((None, None));
    }
    let mut reviewed = Vec::with_capacity(keys.len());
    for (member, summary) in family.members.iter().zip(batch.records) {
        if !record_matches(
            retrieval,
            family,
            &summary,
            &member.name,
            member.fit.as_ref(),
        )? {
            return Ok((None, None));
        }
        let evidence = VariantSuggestionEvidence {
            record: member.key.clone(),
            naming_convention: "reviewed_membership".into(),
            qualifier: member.display_label(family),
            compatibility: "reviewed equipment family".into(),
        };
        reviewed.push((member, summary, evidence));
    }

    // A declared progression follows its authored order. Named options sort by level.
    if family.order_by_level {
        reviewed.sort_by(|(_, a, _), (_, b, _)| {
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
    if let FamilySeed::General(general) = selection {
        let members = reviewed
            .into_iter()
            .filter_map(|(member, summary, evidence)| {
                general
                    .covers(member)
                    .then_some(super::VariantFamilyChoice {
                        summary,
                        qualifier: evidence.qualifier,
                    })
            })
            .collect();
        return Ok((
            None,
            Some(VariantFamilyContext::General {
                heading: axes::general_heading(family, general),
                members,
            }),
        ));
    }
    let context = if include_family
        && !family.axes.is_empty()
        && let FamilySeed::Concrete(concrete_seed) = selection
    {
        let general = match family
            .general_records
            .iter()
            .find(|entry| entry.covers(concrete_seed))
        {
            Some(entry) => read_general(retrieval, family, entry)?,
            None => None,
        };
        Some(axes::concrete_context(
            family,
            concrete_seed,
            reviewed.iter().map(|(member, _, _)| *member),
            general,
        ))
    } else {
        None
    };
    let (variants, evidence) = reviewed
        .into_iter()
        .map(|(_, summary, evidence)| (summary, evidence))
        .unzip();
    Ok((
        Some(VariantGroupResult {
            seed: seed.clone(),
            base_name: family.name.clone(),
            variants,
            evidence,
            ambiguous: false,
            truncated: false,
            verified: true,
        }),
        context,
    ))
}

fn read_general(
    retrieval: &AtlasRetrievalService,
    family: &Family,
    entry: &GeneralRecord,
) -> Result<Option<Box<SourceRecordSummary>>, SearchError> {
    let Some(summary) = retrieval.index.read_summary(&entry.key)? else {
        return Ok(None);
    };
    Ok(record_matches(retrieval, family, &summary, &entry.name, None)?.then(|| Box::new(summary)))
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
    fit: Option<&Fit>,
) -> Result<bool, SearchError> {
    if summary.document_kind != "Item"
        || summary.source_type.as_value() != Some(&family.item_type)
        || !name_matches(summary, expected_name)
    {
        return Ok(false);
    }
    if family.category.is_none() && fit.is_none() {
        return Ok(true);
    }
    let Some(record) = retrieval.index.read_source_record(&summary.key)? else {
        return Ok(false);
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
    let category = family.category.as_deref();
    Ok(match compatibility(&record) {
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
                    "category": family.category,
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
        invalid(family, "invalid general record scope");

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
            let explored = explore(&retrieval, &cold);
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
            if name == Some(moderate.name.as_str()) {
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
        assert!(
            matches!(explored.variants, VariantGroupOutcome::Content(group) if !group.verified)
        );
        assert!(explore(&retrieval, &moderate.key).family.is_none());
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
