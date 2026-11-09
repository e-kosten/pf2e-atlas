use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::{RecordKey, SourceRecordSummary, normalize_record_name};
use atlas_record::source_record::{ItemSourceView, SourceFieldView, SourceNodeView};
use std::sync::LazyLock;

use super::catalog::{Catalog, Family, Fit, Member};
use super::{Compatibility, VariantGroupResult, VariantSuggestionEvidence, compatibility};

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

pub(super) fn verified_group(
    retrieval: &AtlasRetrievalService,
    seed: &SourceRecordSummary,
) -> Result<Option<VariantGroupResult>, SearchError> {
    let Some(family) = catalog()?
        .families
        .values()
        .find(|family| family.members.iter().any(|member| member.key == seed.key))
    else {
        return Ok(None);
    };

    let keys = family
        .members
        .iter()
        .map(|member| member.key.clone())
        .collect::<Vec<_>>();
    let batch = retrieval.index.read_summaries(&keys)?;
    if !batch.missing.is_empty() {
        return Ok(None);
    }
    let mut reviewed = Vec::with_capacity(keys.len());
    for (member, summary) in family.members.iter().zip(batch.records) {
        if !member.matches(retrieval, family, &summary)? {
            return Ok(None);
        }
        let evidence = VariantSuggestionEvidence {
            record: member.key.clone(),
            naming_convention: "reviewed_membership".into(),
            qualifier: member.display_label(),
            compatibility: "reviewed equipment family".into(),
        };
        reviewed.push((summary, evidence));
    }

    // A declared progression follows its authored order. Named options sort by level.
    if family.order_by_level {
        reviewed.sort_by(|(a, _), (b, _)| {
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
    let (variants, evidence) = reviewed.into_iter().unzip();
    Ok(Some(VariantGroupResult {
        seed: seed.clone(),
        base_name: family.name.clone(),
        variants,
        evidence,
        ambiguous: false,
        truncated: false,
        verified: true,
    }))
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

impl Member {
    fn matches(
        &self,
        retrieval: &AtlasRetrievalService,
        family: &Family,
        summary: &SourceRecordSummary,
    ) -> Result<bool, SearchError> {
        if summary.document_kind != "Item"
            || summary.source_type.as_value() != Some(&family.item_type)
            || !name_matches(summary, &self.name)
        {
            return Ok(false);
        }
        if family.category.is_none() && self.fit.is_none() {
            return Ok(true);
        }
        let Some(record) = retrieval.index.read_source_record(&self.key)? else {
            return Ok(false);
        };
        if let Some(fit) = &self.fit {
            let Some(SourceNodeView::Item(item @ ItemSourceView::AmmoSource(_))) =
                record.node_at(&[])
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

    fn display_label(&self) -> String {
        match &self.fit {
            Some(fit) => format!(
                "Fits {}",
                fit.weapons
                    .iter()
                    .map(|weapon| weapon.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None => self.attributes.join(", "),
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
                assert!(!member.display_label().is_empty());
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
