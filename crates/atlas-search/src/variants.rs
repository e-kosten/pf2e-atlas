mod axes;
#[cfg(test)]
#[path = "../build_axes.rs"]
mod build_axes;
mod catalog;
mod verified;

use atlas_domain::{RecordKey, SourceRecordSummary};
pub struct VariantGroupRequest<'a> {
    pub record_key: &'a RecordKey,
}
#[derive(Debug, Clone)]
pub struct VariantSuggestionEvidence {
    pub record: RecordKey,
    pub naming_convention: String,
    pub qualifier: String,
    pub compatibility: String,
}
#[derive(Debug, Clone)]
pub struct VariantGroupResult {
    pub seed: SourceRecordSummary,
    pub base_name: String,
    pub variants: Vec<SourceRecordSummary>,
    pub evidence: Vec<VariantSuggestionEvidence>,
    pub ambiguous: bool,
    pub truncated: bool,
    pub verified: bool,
}
/// Absence is only asserted after a complete compatibility check.
#[derive(Debug, Clone)]
pub enum VariantGroupOutcome {
    Missing,
    Empty,
    Unavailable { reason: String },
    Content(Box<VariantGroupResult>),
}
pub struct VariantExploration {
    pub variants: VariantGroupOutcome,
    pub family: Option<VariantFamilyContext>,
    pub equipment_family_id: Option<String>,
}

pub enum EquipmentFamilyOutcome {
    Missing,
    Unavailable { name: String },
    Content(EquipmentFamily),
}

pub struct EquipmentFamily {
    pub name: String,
    pub members: Vec<VariantFamilyChoice>,
    pub groupings: Vec<EquipmentFamilyGrouping>,
    pub overviews: Vec<VariantFamilyChoice>,
}

pub struct EquipmentFamilyGrouping {
    pub id: String,
    pub label: String,
    pub groups: Vec<VariantAxisSlice>,
}

pub enum VariantFamilyContext {
    Concrete {
        slices: Vec<VariantAxisSlice>,
        other_combinations: Vec<RecordKey>,
        general: Option<Box<SourceRecordSummary>>,
    },
    General {
        heading: String,
        members: Vec<VariantFamilyChoice>,
    },
}

pub struct VariantAxisSlice {
    pub heading: String,
    pub members: Vec<RecordKey>,
}

pub struct VariantFamilyChoice {
    pub summary: SourceRecordSummary,
    pub qualifier: String,
}

use crate::{AtlasRetrievalService, SearchError};
use atlas_record::source_record::{ItemSourceView, SourceBackedRecord, SourceNodeView};
fn naming(name: &str) -> (&str, &str, &str) {
    if let Some((base, qualifier)) = name.strip_suffix(')').and_then(|s| s.rsplit_once(" ("))
        && !base.is_empty()
        && !qualifier.trim().is_empty()
    {
        return (base, qualifier, "trailing_parenthetical");
    }
    for grade in [
        "Lesser", "Moderate", "Greater", "Major", "Minor", "Standard",
    ] {
        if let Some(base) = name.strip_prefix(grade).and_then(|s| s.strip_prefix(' '))
            && !base.is_empty()
        {
            return (base, grade, "leading_grade");
        }
    }
    (name, "", "unqualified")
}
#[derive(Debug, PartialEq, Eq)]
enum Compatibility<'a> {
    Family,
    Weapon(
        &'a atlas_foundry_model::generated::WeaponSystemSourceCategory,
        &'a atlas_foundry_model::generated::WeaponSystemSourceGroup,
    ),
    Armor(&'a atlas_foundry_model::generated::ArmorSystemSourceCategory),
    Consumable(&'a atlas_foundry_model::generated::ConsumableSystemSourceCategory),
}
fn compatibility(record: &SourceBackedRecord) -> Option<Compatibility<'_>> {
    match record.node_at(&[])? {
        SourceNodeView::Item(item) => match item {
            ItemSourceView::WeaponSource(_) => Some(Compatibility::Weapon(
                item.weapon_category().value()?,
                item.weapon_group().value()?,
            )),
            ItemSourceView::ArmorSource(_) => {
                Some(Compatibility::Armor(item.armor_category().value()?))
            }
            ItemSourceView::ConsumableSource(_) => Some(Compatibility::Consumable(
                item.consumable_category().value()?,
            )),
            _ => Some(Compatibility::Family),
        },
        _ => Some(Compatibility::Family),
    }
}
impl AtlasRetrievalService {
    pub fn variant_group(
        &self,
        request: VariantGroupRequest<'_>,
    ) -> Result<VariantGroupOutcome, SearchError> {
        Ok(self.read_variants(request, false)?.variants)
    }

    pub fn equipment_family(&self, id: &str) -> Result<EquipmentFamilyOutcome, SearchError> {
        verified::equipment_family(self, id)
    }

    pub fn variant_exploration(
        &self,
        request: VariantGroupRequest<'_>,
    ) -> Result<VariantExploration, SearchError> {
        self.read_variants(request, true)
    }

    fn read_variants(
        &self,
        request: VariantGroupRequest<'_>,
        include_family: bool,
    ) -> Result<VariantExploration, SearchError> {
        let Some(seed) = self.index.read_summary(request.record_key)? else {
            return Ok(VariantExploration {
                variants: VariantGroupOutcome::Missing,
                family: None,
                equipment_family_id: None,
            });
        };
        let verified = verified::verified_group(self, &seed, include_family)?;
        let variants = match verified.group {
            Some(group) => VariantGroupOutcome::Content(Box::new(group)),
            None if verified.context.is_some() => VariantGroupOutcome::Empty,
            None => self.possible_variant_group(seed)?,
        };
        Ok(VariantExploration {
            variants,
            family: verified.context,
            equipment_family_id: verified.family_id,
        })
    }

    fn possible_variant_group(
        &self,
        seed: SourceRecordSummary,
    ) -> Result<VariantGroupOutcome, SearchError> {
        let Some(name) = seed.name.as_value() else {
            return Ok(VariantGroupOutcome::Unavailable {
                reason: "The seed name is unavailable for variant verification".into(),
            });
        };
        let (base, _, _) = naming(name);
        let base = base.to_owned();
        let family = seed
            .source_type
            .as_value()
            .map(String::as_str)
            .unwrap_or(&seed.document_kind);
        let keys = self
            .index
            .variant_candidates(seed.key.pack().as_str(), family, &base, 256)?;
        let mut truncated = keys.len() == 256;
        let mut candidates = keys.into_iter().collect::<std::collections::BTreeSet<_>>();
        for grade in [
            "Lesser", "Moderate", "Greater", "Major", "Minor", "Standard",
        ] {
            let keys = self.index.variant_candidates(
                seed.key.pack().as_str(),
                family,
                &format!("{grade} {base}"),
                256,
            )?;
            truncated |= keys.len() == 256;
            candidates.extend(keys);
        }
        truncated |= candidates.len() > 256;
        candidates.remove(&seed.key);
        let mut keys = vec![seed.key.clone()];
        keys.extend(candidates.into_iter().take(255));
        let source = self.index.read_source_record(&seed.key)?.ok_or_else(|| {
            SearchError::artifact_contract_violation("variant seed source absent")
        })?;
        let compatible = compatibility(&source);
        if compatible.is_none() {
            return Ok(VariantGroupOutcome::Unavailable {
                reason: "The seed compatibility facts are unavailable".into(),
            });
        }
        let mut insufficient_evidence = false;
        let mut variants = Vec::new();
        let mut evidence = Vec::new();
        let mut ambiguous = false;
        for record in self.summaries(&keys)? {
            if verified::excluded(&seed.key, &record.key)? {
                continue;
            }
            let Some(name) = record.name.as_value() else {
                insufficient_evidence = true;
                continue;
            };
            let (candidate_base, qualifier, convention) = naming(name);
            if atlas_domain::normalize_record_name(candidate_base)
                != atlas_domain::normalize_record_name(&base)
            {
                continue;
            }
            let candidate;
            let candidate_compatibility = if record.key == seed.key {
                compatibility(&source)
            } else {
                candidate = self.index.read_source_record(&record.key)?.ok_or_else(|| {
                    SearchError::artifact_contract_violation("variant candidate source absent")
                })?;
                compatibility(&candidate)
            };
            if compatible.is_none() || candidate_compatibility.is_none() {
                ambiguous = true;
                insufficient_evidence = true;
                continue;
            }
            if compatible != candidate_compatibility {
                ambiguous = true;
                continue;
            }
            evidence.push(VariantSuggestionEvidence {
                record: record.key.clone(),
                naming_convention: convention.to_owned(),
                qualifier: qualifier.to_owned(),
                compatibility: if matches!(compatible, Some(Compatibility::Family)) {
                    "same pack and family"
                } else {
                    "same pack, family and known physical category/group"
                }
                .to_owned(),
            });
            variants.push(record);
        }
        if variants.len() < 2 {
            return Ok(if truncated || insufficient_evidence {
                VariantGroupOutcome::Unavailable {
                    reason: if truncated {
                        "The bounded variant check was incomplete"
                    } else {
                        "Candidate compatibility facts are unavailable"
                    }
                    .into(),
                }
            } else {
                VariantGroupOutcome::Empty
            });
        };
        let mut qualifiers = std::collections::BTreeSet::new();
        for e in &evidence {
            if !qualifiers.insert(atlas_domain::normalize_record_name(&e.qualifier)) {
                ambiguous = true;
            }
        }
        Ok(VariantGroupOutcome::Content(Box::new(VariantGroupResult {
            seed,
            base_name: base,
            variants,
            evidence,
            ambiguous,
            truncated,
            verified: false,
        })))
    }
}
