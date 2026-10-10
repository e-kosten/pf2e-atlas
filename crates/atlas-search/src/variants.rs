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
    ) -> Result<Option<VariantGroupResult>, SearchError> {
        let Some(seed) = self.index.read_summary(request.record_key)? else {
            return Ok(None);
        };
        let Some(name) = seed.name.as_value() else {
            return Ok(None);
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
        let mut variants = Vec::new();
        let mut evidence = Vec::new();
        let mut ambiguous = false;
        for record in self.summaries(&keys)? {
            let Some(name) = record.name.as_value() else {
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
            return Ok(None);
        };
        let mut qualifiers = std::collections::BTreeSet::new();
        for e in &evidence {
            if !qualifiers.insert(atlas_domain::normalize_record_name(&e.qualifier)) {
                ambiguous = true;
            }
        }
        Ok(Some(VariantGroupResult {
            seed,
            base_name: base,
            variants,
            evidence,
            ambiguous,
            truncated,
        }))
    }
}
