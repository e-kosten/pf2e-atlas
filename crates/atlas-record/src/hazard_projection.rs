use std::collections::BTreeMap;

use atlas_domain::{MetricDomain, Rarity, RecordKind};
use serde::Serialize;

use crate::{
    ContentOwner, FactValue, FoundryNode, HazardActionCapability, HazardActionCount,
    HazardActionType, HazardCapability, HazardComplexity, HazardDefenses, HazardEmitsSound,
    HazardEntity, HazardEntityFamily, HazardEntityId, HazardEntityOccurrence, HazardFact,
    HazardFrequency, HazardFrequencyInterval, HazardItemCommon, HazardItemLineage, HazardIwr,
    HazardRecord, HazardRuleElement, HazardRuleType, HazardSaveKind, HazardSourceAttackMode,
    HazardSourceValue, HazardStrikeCapability, HazardStrikeDamage, MetricValue,
    OwnedRichContentDocument, PresentationBadge, PresentationBadgeKind, PresentationBlock,
    PresentationFact, PresentationSection, PresentationSectionKind, RecordPresentationDocument,
    RichDocument, RichNode, metrics, project_presentation_content,
};

pub const HAZARD_CONVENIENCE_RULE_ID: &str = "pf2e-hazard-conveniences";
pub const HAZARD_CONVENIENCE_RULE_VERSION: u32 = 1;
pub const PF2E_HAZARD_ATTACK_MODE_RULE_ID: &str = "pf2e-hazard-strike-trait-mode";
pub const PF2E_HAZARD_ATTACK_MODE_RULE_VERSION: u32 = 1;
pub const PF2E_STRIKE_ACTION_COST_RULE_ID: &str = "pf2e-strike-action-cost";
pub const PF2E_STRIKE_ACTION_COST_RULE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HazardAttackMode {
    Melee,
    Ranged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardAttackModeProjection {
    pub mode: HazardAttackMode,
    pub rule_id: &'static str,
    pub rule_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardStrikeActionCostProjection {
    pub cost: HazardActionCount,
    pub rule_id: &'static str,
    pub rule_version: u32,
    pub basis: HazardEntityFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HazardAttackEffect {
    NoMultipleAttackPenalty,
    IndependentLimbs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardAttackEffectsProjection {
    pub effects: Vec<HazardAttackEffect>,
    pub has_unmodeled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HazardHasHealthConsistency {
    Consistent,
    Conflict {
        source_has_health: bool,
        derived_has_health: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HazardWeaponTypeConsistency {
    Consistent,
    Conflict {
        source_weapon_type: HazardSourceAttackMode,
        derived_mode: HazardAttackMode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "field", rename_all = "snake_case")]
pub enum HazardSourceMetadataFact {
    TokenName {
        value: HazardFact<String>,
    },
    HasHealth {
        value: HazardFact<bool>,
    },
    TemporaryMaximum {
        value: HazardFact<i64>,
    },
    SaveDetail {
        save: HazardSaveKind,
        value: HazardFact<String>,
    },
    ItemRarity {
        entity_id: HazardEntityId,
        value: HazardFact<Rarity>,
    },
    ItemLineage {
        entity_id: HazardEntityId,
        value: HazardFact<HazardItemLineage>,
    },
    StrikeAttack {
        entity_id: HazardEntityId,
        value: HazardFact<i64>,
    },
    StrikeWeaponType {
        entity_id: HazardEntityId,
        value: HazardFact<HazardSourceAttackMode>,
    },
    StrikeAttackEffectsCustom {
        entity_id: HazardEntityId,
        value: HazardFact<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardSourceMetadataField {
    TokenName,
    HasHealth,
    TemporaryMaximum,
    SaveDetail(HazardSaveKind),
    ItemRarity,
    ItemLineage,
    StrikeAttack,
    StrikeWeaponType,
    StrikeAttackEffectsCustom,
}

impl HazardSourceMetadataField {
    pub const fn key(self) -> &'static str {
        match self {
            Self::TokenName => "provenance.token.name",
            Self::HasHealth => "defenses.source_metadata.has_health",
            Self::TemporaryMaximum => "defenses.hit_points.source_metadata.temporary_maximum",
            Self::SaveDetail(HazardSaveKind::Fortitude) => {
                "defenses.saves.source_metadata.fortitude_detail"
            }
            Self::SaveDetail(HazardSaveKind::Reflex) => {
                "defenses.saves.source_metadata.reflex_detail"
            }
            Self::SaveDetail(HazardSaveKind::Will) => "defenses.saves.source_metadata.will_detail",
            Self::ItemRarity => "activity.source_metadata.rarity",
            Self::ItemLineage => "activity.source_metadata.lineage",
            Self::StrikeAttack => "activity.strike.source_metadata.attack",
            Self::StrikeWeaponType => "activity.strike.source_metadata.weapon_type",
            Self::StrikeAttackEffectsCustom => {
                "activity.strike.source_metadata.attack_effects_custom"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardSourceMetadataIssueKind {
    Malformed,
    HasHealthConflict,
    NonZeroTemporaryMaximum,
    NonEmptySaveDetail,
    WeaponTypeConflict,
    NonEmptyAttackEffectsCustom,
}

impl HazardSourceMetadataIssueKind {
    pub const fn message(self) -> &'static str {
        match self {
            Self::Malformed => {
                "This hazard source metadata value is malformed and is retained in provenance."
            }
            Self::HasHealthConflict => {
                "Stored health metadata conflicts with typed maximum hit points."
            }
            Self::NonZeroTemporaryMaximum => {
                "A nonzero temporary-maximum HP source value is retained for reassessment."
            }
            Self::NonEmptySaveDetail => {
                "Hazard save detail is present but is not modeled as gameplay."
            }
            Self::WeaponTypeConflict => {
                "Stored attack-mode metadata conflicts with the trait-derived strike mode."
            }
            Self::NonEmptyAttackEffectsCustom => {
                "Custom attack-effect text is present but is not modeled as gameplay."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardSourceMetadataIssue {
    pub kind: HazardSourceMetadataIssueKind,
    pub field: HazardSourceMetadataField,
    pub entity_id: Option<HazardEntityId>,
}

impl HazardSourceMetadataIssue {
    pub const fn field_key(&self) -> &'static str {
        self.field.key()
    }

    pub fn component_id(&self) -> Option<&str> {
        self.entity_id.as_ref().map(HazardEntityId::as_str)
    }

    pub const fn message(&self) -> &'static str {
        self.kind.message()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardSourceMetadataProjection {
    pub facts: Vec<HazardSourceMetadataFact>,
    pub issues: Vec<HazardSourceMetadataIssue>,
}

pub fn project_hazard_source_metadata(hazard: &HazardRecord) -> HazardSourceMetadataProjection {
    let mut facts = Vec::new();
    let mut issues = Vec::new();

    let token_name = token_name_fact(hazard);
    push_malformed_issue(
        &mut issues,
        HazardSourceMetadataField::TokenName,
        None,
        &token_name,
    );
    facts.push(HazardSourceMetadataFact::TokenName { value: token_name });

    if let Some(defenses) = hazard.defenses.typed() {
        push_malformed_issue(
            &mut issues,
            HazardSourceMetadataField::HasHealth,
            None,
            &defenses.source_metadata.has_health,
        );
        facts.push(HazardSourceMetadataFact::HasHealth {
            value: defenses.source_metadata.has_health.clone(),
        });
        if matches!(
            project_hazard_has_health_consistency(hazard),
            Some(HazardHasHealthConsistency::Conflict { .. })
        ) {
            issues.push(HazardSourceMetadataIssue {
                kind: HazardSourceMetadataIssueKind::HasHealthConflict,
                field: HazardSourceMetadataField::HasHealth,
                entity_id: None,
            });
        }

        if let Some(hit_points) = defenses.hit_points.typed() {
            let temporary_maximum = &hit_points.source_metadata.temporary_maximum;
            push_malformed_issue(
                &mut issues,
                HazardSourceMetadataField::TemporaryMaximum,
                None,
                temporary_maximum,
            );
            if temporary_maximum.typed().is_some_and(|value| *value != 0) {
                issues.push(HazardSourceMetadataIssue {
                    kind: HazardSourceMetadataIssueKind::NonZeroTemporaryMaximum,
                    field: HazardSourceMetadataField::TemporaryMaximum,
                    entity_id: None,
                });
            }
            facts.push(HazardSourceMetadataFact::TemporaryMaximum {
                value: temporary_maximum.clone(),
            });
        }

        if let Some(saves) = defenses.saves.typed() {
            for (save, value) in [
                (
                    HazardSaveKind::Fortitude,
                    &saves.source_metadata.fortitude_detail,
                ),
                (HazardSaveKind::Reflex, &saves.source_metadata.reflex_detail),
                (HazardSaveKind::Will, &saves.source_metadata.will_detail),
            ] {
                let field = HazardSourceMetadataField::SaveDetail(save);
                push_malformed_issue(&mut issues, field, None, value);
                if value.typed().is_some_and(|value| !value.is_empty()) {
                    issues.push(HazardSourceMetadataIssue {
                        kind: HazardSourceMetadataIssueKind::NonEmptySaveDetail,
                        field,
                        entity_id: None,
                    });
                }
                facts.push(HazardSourceMetadataFact::SaveDetail {
                    save,
                    value: value.clone(),
                });
            }
        }
    }

    if let Some(embedded) = hazard.embedded_entities.typed() {
        for entity in &embedded.entities {
            let common = entity_common(entity);
            push_malformed_issue(
                &mut issues,
                HazardSourceMetadataField::ItemRarity,
                Some(&entity.id),
                &common.rarity,
            );
            facts.push(HazardSourceMetadataFact::ItemRarity {
                entity_id: entity.id.clone(),
                value: common.rarity.clone(),
            });

            push_malformed_issue(
                &mut issues,
                HazardSourceMetadataField::ItemLineage,
                Some(&entity.id),
                &common.lineage,
            );
            if let Some(lineage) = common.lineage.typed() {
                push_malformed_issue(
                    &mut issues,
                    HazardSourceMetadataField::ItemLineage,
                    Some(&entity.id),
                    &lineage.compendium_source,
                );
            }
            facts.push(HazardSourceMetadataFact::ItemLineage {
                entity_id: entity.id.clone(),
                value: common.lineage.clone(),
            });

            if let HazardCapability::Strike(strike) = &entity.capability {
                for (field, malformed) in [
                    (
                        HazardSourceMetadataField::StrikeAttack,
                        is_malformed(&strike.source_metadata.attack),
                    ),
                    (
                        HazardSourceMetadataField::StrikeWeaponType,
                        is_malformed(&strike.source_metadata.weapon_type),
                    ),
                    (
                        HazardSourceMetadataField::StrikeAttackEffectsCustom,
                        is_malformed(&strike.source_metadata.attack_effects_custom),
                    ),
                ] {
                    if malformed {
                        issues.push(HazardSourceMetadataIssue {
                            kind: HazardSourceMetadataIssueKind::Malformed,
                            field,
                            entity_id: Some(entity.id.clone()),
                        });
                    }
                }
                if matches!(
                    project_hazard_weapon_type_consistency(entity),
                    Some(HazardWeaponTypeConsistency::Conflict { .. })
                ) {
                    issues.push(HazardSourceMetadataIssue {
                        kind: HazardSourceMetadataIssueKind::WeaponTypeConflict,
                        field: HazardSourceMetadataField::StrikeWeaponType,
                        entity_id: Some(entity.id.clone()),
                    });
                }
                if strike
                    .source_metadata
                    .attack_effects_custom
                    .typed()
                    .is_some_and(|value| !value.is_empty())
                {
                    issues.push(HazardSourceMetadataIssue {
                        kind: HazardSourceMetadataIssueKind::NonEmptyAttackEffectsCustom,
                        field: HazardSourceMetadataField::StrikeAttackEffectsCustom,
                        entity_id: Some(entity.id.clone()),
                    });
                }
                facts.extend([
                    HazardSourceMetadataFact::StrikeAttack {
                        entity_id: entity.id.clone(),
                        value: strike.source_metadata.attack.clone(),
                    },
                    HazardSourceMetadataFact::StrikeWeaponType {
                        entity_id: entity.id.clone(),
                        value: strike.source_metadata.weapon_type.clone(),
                    },
                    HazardSourceMetadataFact::StrikeAttackEffectsCustom {
                        entity_id: entity.id.clone(),
                        value: strike.source_metadata.attack_effects_custom.clone(),
                    },
                ]);
            }
        }
    }

    HazardSourceMetadataProjection { facts, issues }
}

fn token_name_fact(hazard: &HazardRecord) -> HazardFact<String> {
    match &hazard.provenance.token.value {
        FactValue::Missing => HazardFact::source(
            FactValue::Missing,
            hazard
                .provenance
                .token
                .provenance
                .relative_source_path
                .clone(),
        ),
        FactValue::Null => HazardFact::source(
            FactValue::Null,
            hazard
                .provenance
                .token
                .provenance
                .relative_source_path
                .clone(),
        ),
        FactValue::Value(HazardSourceValue::Typed(token)) => token.name.clone(),
        FactValue::Value(HazardSourceValue::Unsupported(value)) => HazardFact::source(
            FactValue::Value(HazardSourceValue::Unsupported(value.clone())),
            hazard
                .provenance
                .token
                .provenance
                .relative_source_path
                .clone(),
        ),
    }
}

fn entity_common(entity: &HazardEntity) -> &HazardItemCommon {
    match &entity.capability {
        HazardCapability::Action(value) => &value.common,
        HazardCapability::Strike(value) => &value.common,
        HazardCapability::Condition(value) => &value.common,
        HazardCapability::Effect(value) => &value.common,
        HazardCapability::UnsupportedChild(value) => &value.common,
    }
}

fn is_malformed<T>(fact: &HazardFact<T>) -> bool {
    matches!(
        fact.value,
        FactValue::Value(HazardSourceValue::Unsupported(_))
    )
}

fn push_malformed_issue<T>(
    issues: &mut Vec<HazardSourceMetadataIssue>,
    field: HazardSourceMetadataField,
    entity_id: Option<&HazardEntityId>,
    fact: &HazardFact<T>,
) {
    if is_malformed(fact) {
        issues.push(HazardSourceMetadataIssue {
            kind: HazardSourceMetadataIssueKind::Malformed,
            field,
            entity_id: entity_id.cloned(),
        });
    }
}

pub fn project_hazard_has_health_consistency(
    hazard: &HazardRecord,
) -> Option<HazardHasHealthConsistency> {
    let defenses = hazard.defenses.typed()?;
    let source_has_health = *defenses.source_metadata.has_health.typed()?;
    let maximum = *defenses.hit_points.typed()?.maximum.typed()?;
    let derived_has_health = maximum > 0;
    Some(if source_has_health == derived_has_health {
        HazardHasHealthConsistency::Consistent
    } else {
        HazardHasHealthConsistency::Conflict {
            source_has_health,
            derived_has_health,
        }
    })
}

pub fn project_hazard_attack_mode(entity: &HazardEntity) -> Option<HazardAttackModeProjection> {
    let HazardCapability::Strike(strike) = &entity.capability else {
        return None;
    };
    let traits = strike.common.traits.typed()?;
    let mode = if traits
        .iter()
        .any(|value| value.as_str().starts_with("range-"))
    {
        HazardAttackMode::Ranged
    } else {
        HazardAttackMode::Melee
    };
    Some(HazardAttackModeProjection {
        mode,
        rule_id: PF2E_HAZARD_ATTACK_MODE_RULE_ID,
        rule_version: PF2E_HAZARD_ATTACK_MODE_RULE_VERSION,
    })
}

pub fn project_hazard_weapon_type_consistency(
    entity: &HazardEntity,
) -> Option<HazardWeaponTypeConsistency> {
    let HazardCapability::Strike(strike) = &entity.capability else {
        return None;
    };
    let source_weapon_type = *strike.source_metadata.weapon_type.typed()?;
    let derived_mode = project_hazard_attack_mode(entity)?.mode;
    let consistent = matches!(
        (source_weapon_type, derived_mode),
        (HazardSourceAttackMode::Melee, HazardAttackMode::Melee)
            | (HazardSourceAttackMode::Ranged, HazardAttackMode::Ranged)
    );
    Some(if consistent {
        HazardWeaponTypeConsistency::Consistent
    } else {
        HazardWeaponTypeConsistency::Conflict {
            source_weapon_type,
            derived_mode,
        }
    })
}

pub fn project_hazard_strike_action_cost(
    entity: &HazardEntity,
) -> Option<HazardStrikeActionCostProjection> {
    matches!(
        (&entity.family, &entity.capability),
        (HazardEntityFamily::Strike, HazardCapability::Strike(_))
    )
    .then_some(HazardStrikeActionCostProjection {
        cost: HazardActionCount::One,
        rule_id: PF2E_STRIKE_ACTION_COST_RULE_ID,
        rule_version: PF2E_STRIKE_ACTION_COST_RULE_VERSION,
        basis: HazardEntityFamily::Strike,
    })
}

pub fn project_hazard_attack_effects(
    entity: &HazardEntity,
) -> Option<HazardAttackEffectsProjection> {
    let HazardCapability::Strike(strike) = &entity.capability else {
        return None;
    };
    let authored = strike.attack_effects.typed()?;
    let mut effects = Vec::new();
    let mut has_unmodeled = false;
    for value in authored {
        let effect = match value.as_str() {
            "no-map" => HazardAttackEffect::NoMultipleAttackPenalty,
            "independent-limbs" => HazardAttackEffect::IndependentLimbs,
            _ => {
                has_unmodeled = true;
                continue;
            }
        };
        if !effects.contains(&effect) {
            effects.push(effect);
        }
    }
    Some(HazardAttackEffectsProjection {
        effects,
        has_unmodeled,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardConvenienceProjection {
    pub rule_id: &'static str,
    pub rule_version: u32,
    pub detection_dc: Option<i64>,
    pub broken_threshold: Option<i64>,
    pub initiative_suggestion: Option<HazardInitiativeSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardInitiativeSuggestion {
    pub statistic: HazardInitiativeStatistic,
    pub modifier: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HazardInitiativeStatistic {
    Stealth,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HazardFactProjection {
    pub conveniences: HazardConvenienceProjection,
    pub metrics: Vec<crate::MetricRow>,
    pub mechanic_terms: Vec<String>,
}

pub fn project_hazard_conveniences(hazard: &HazardRecord) -> HazardConvenienceProjection {
    let stealth_modifier = hazard
        .detection
        .typed()
        .and_then(|detection| detection.stealth_modifier.typed())
        .copied();
    let detection_dc = stealth_modifier.and_then(|modifier| modifier.checked_add(10));
    let maximum_hit_points = hazard
        .defenses
        .typed()
        .and_then(|defenses| defenses.hit_points.typed())
        .and_then(|hit_points| hit_points.maximum.typed())
        .copied();
    let broken_threshold = maximum_hit_points
        .filter(|maximum| *maximum >= 0)
        .map(|maximum| maximum / 2);
    let is_complex = matches!(hazard.complexity.typed(), Some(HazardComplexity::Complex));
    let initiative_suggestion = is_complex
        .then_some(stealth_modifier)
        .flatten()
        .map(|modifier| HazardInitiativeSuggestion {
            statistic: HazardInitiativeStatistic::Stealth,
            modifier,
        });

    HazardConvenienceProjection {
        rule_id: HAZARD_CONVENIENCE_RULE_ID,
        rule_version: HAZARD_CONVENIENCE_RULE_VERSION,
        detection_dc,
        broken_threshold,
        initiative_suggestion,
    }
}

pub fn project_hazard_facts(hazard: &HazardRecord) -> HazardFactProjection {
    let conveniences = project_hazard_conveniences(hazard);
    let mut metrics = Vec::new();
    let mut mechanic_terms = Vec::new();

    if let Some(level) = hazard.level.typed() {
        mechanic_terms.push(format!("level {level}"));
    }
    if let Some(complexity) = hazard.complexity.typed() {
        mechanic_terms.push(match complexity {
            HazardComplexity::Simple => "simple hazard".to_string(),
            HazardComplexity::Complex => "complex hazard".to_string(),
        });
    }
    if let Some(rarity) = hazard.rarity.typed() {
        mechanic_terms.push(rarity.as_str().to_string());
    }
    if let Some(size) = hazard.size.typed() {
        mechanic_terms.push(size.as_source().to_string());
    }
    if let Some(traits) = hazard.traits.typed() {
        mechanic_terms.extend(traits.iter().map(|value| value.as_str().to_string()));
    }

    if let Some(detection) = hazard.detection.typed() {
        push_defined_number(
            &mut metrics,
            metrics::actor::STEALTH_MOD,
            detection.stealth_modifier.typed().copied(),
        );
    }
    if let Some(defenses) = hazard.defenses.typed() {
        project_defenses(defenses, &mut metrics, &mut mechanic_terms);
    }

    if let Some(lifecycle) = hazard.lifecycle.typed()
        && let Some(disable) = lifecycle.disable.typed()
    {
        project_structured_disable_checks(disable, &mut metrics);
    }

    if let Some(embedded) = hazard.embedded_entities.typed() {
        let mut occurrences = embedded.occurrences.iter().collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| occurrence.authored_order);
        for occurrence in occurrences {
            if let Some(entity) = embedded
                .entities
                .iter()
                .find(|entity| entity.id == occurrence.entity_id)
            {
                project_embedded_terms(entity, &mut mechanic_terms);
            }
        }
    }

    HazardFactProjection {
        conveniences,
        metrics,
        mechanic_terms,
    }
}

pub fn build_hazard_presentation_document(
    hazard: &HazardRecord,
    include_content: impl Fn(&OwnedRichContentDocument) -> bool,
) -> RecordPresentationDocument {
    let projection = project_hazard_facts(hazard);
    let conveniences = &projection.conveniences;
    let mut summary = Vec::new();
    let mut defense = Vec::new();
    let mut offense_facts = Vec::new();
    let mut offense_content = Vec::new();
    let mut routine_blocks = Vec::new();
    let mut description_blocks = Vec::new();
    let mut detail_facts = Vec::new();
    let mut detail_blocks = Vec::new();

    push_fact(
        &mut summary,
        "level",
        "Level",
        hazard.level.typed().copied(),
    );
    push_text_fact(
        &mut summary,
        "complexity",
        "Complexity",
        hazard.complexity.typed().map(|value| match value {
            HazardComplexity::Simple => "Simple",
            HazardComplexity::Complex => "Complex",
        }),
    );
    if let Some(detection) = hazard.detection.typed() {
        push_modifier_fact(
            &mut summary,
            "stealth.modifier",
            "Stealth",
            detection.stealth_modifier.typed().copied(),
        );
        push_fact(
            &mut summary,
            "stealth.dc",
            "Detection DC",
            conveniences.detection_dc,
        );
        push_labeled_document(&mut detail_blocks, "Detection", &detection.details);
    }
    push_text_fact(
        &mut detail_facts,
        "size",
        "Size",
        hazard.size.typed().map(|size| size.as_source()),
    );
    if let Some(publication) = hazard.publication.typed() {
        push_text_fact(
            &mut detail_facts,
            "publication.title",
            "Publication",
            publication.title.typed().map(String::as_str),
        );
    }
    push_text_fact(
        &mut detail_facts,
        "emits_sound",
        "Emits Sound",
        hazard.emits_sound.typed().map(emits_sound_label),
    );

    if let Some(defenses) = hazard.defenses.typed() {
        push_fact(
            &mut defense,
            "armor_class",
            "Armor Class",
            defenses.armor_class.typed().copied(),
        );
        push_fact(
            &mut defense,
            "hardness",
            "Hardness",
            defenses.hardness.typed().copied(),
        );
        if let Some(hit_points) = defenses.hit_points.typed() {
            push_fact(
                &mut defense,
                "hit_points.current",
                "Hit Points",
                hit_points.current.typed().copied(),
            );
            push_fact(
                &mut defense,
                "hit_points.maximum",
                "Maximum Hit Points",
                hit_points.maximum.typed().copied(),
            );
            push_fact(
                &mut defense,
                "hit_points.temporary",
                "Temporary Hit Points",
                hit_points.temporary.typed().copied(),
            );
            push_fact(
                &mut defense,
                "hit_points.broken_threshold",
                "Broken Threshold",
                conveniences.broken_threshold,
            );
            push_labeled_document(&mut detail_blocks, "Hit Points", &hit_points.details);
        }
        if let Some(saves) = defenses.saves.typed() {
            push_modifier_fact(
                &mut defense,
                "save.fortitude",
                "Fortitude",
                saves.fortitude.typed().copied(),
            );
            push_modifier_fact(
                &mut defense,
                "save.reflex",
                "Reflex",
                saves.reflex.typed().copied(),
            );
            push_modifier_fact(
                &mut defense,
                "save.will",
                "Will",
                saves.will.typed().copied(),
            );
        }
        push_iwr_fact(
            &mut defense,
            "immunities",
            "Immunities",
            &defenses.immunities,
        );
        push_iwr_fact(
            &mut defense,
            "weaknesses",
            "Weaknesses",
            &defenses.weaknesses,
        );
        push_iwr_fact(
            &mut defense,
            "resistances",
            "Resistances",
            &defenses.resistances,
        );
    }

    if let Some(embedded) = hazard.embedded_entities.typed() {
        let mut occurrences = embedded.occurrences.iter().collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| occurrence.authored_order);
        for occurrence in occurrences {
            let Some(entity) = embedded
                .entities
                .iter()
                .find(|entity| entity.id == occurrence.entity_id)
            else {
                continue;
            };
            push_embedded_presentation(&mut offense_facts, occurrence, entity);
        }
    }

    if let Some(lifecycle) = hazard.lifecycle.typed() {
        push_document(&mut description_blocks, &lifecycle.description);
        push_labeled_document(&mut routine_blocks, "Disable", &lifecycle.disable);
        push_labeled_document(&mut routine_blocks, "Routine", &lifecycle.routine);
        push_labeled_document(&mut routine_blocks, "Reset", &lifecycle.reset);
    }

    let mut owned = hazard.content.documents.iter().collect::<Vec<_>>();
    owned.sort_by_key(|document| (document.authored_order, document.id.content_key.as_str()));
    for document in owned
        .into_iter()
        .filter(|document| include_content(document))
    {
        let mut content = project_presentation_content(&document.document);
        if let Some(label) = document
            .label
            .as_deref()
            .filter(|label| !label.trim().is_empty())
        {
            content.blocks.insert(
                0,
                crate::PresentationContentBlock::Heading {
                    level: 3,
                    text: label.to_string(),
                },
            );
        }
        let block = PresentationBlock::Content(content);
        match document.owner {
            ContentOwner::HazardEntity(_) | ContentOwner::HazardOccurrence(_) => {
                // Occurrence-owned prose stays with the activity family rather than general lore.
                offense_content.push(block);
            }
            ContentOwner::Record(_)
                if matches!(
                    document.source_kind,
                    crate::ContentSourceKind::Description
                        | crate::ContentSourceKind::Disable
                        | crate::ContentSourceKind::Routine
                        | crate::ContentSourceKind::Reset
                        | crate::ContentSourceKind::StealthDetails
                ) => {}
            ContentOwner::Record(_) => description_blocks.push(block),
            ContentOwner::CreatureEntity(_) | ContentOwner::CreatureOccurrence(_) => {}
        }
    }

    let mut sections = Vec::new();
    push_fact_section(&mut sections, PresentationSectionKind::Summary, summary);
    push_fact_section(&mut sections, PresentationSectionKind::Defense, defense);
    if !offense_facts.is_empty() || !offense_content.is_empty() {
        let mut blocks = Vec::new();
        if !offense_facts.is_empty() {
            blocks.push(PresentationBlock::FactList(offense_facts));
        }
        blocks.extend(offense_content);
        sections.push(PresentationSection::new(
            PresentationSectionKind::Offense,
            blocks,
        ));
    }
    if !routine_blocks.is_empty() {
        sections.push(PresentationSection::new(
            PresentationSectionKind::Routine,
            routine_blocks,
        ));
    }
    if !description_blocks.is_empty() {
        sections.push(PresentationSection::new(
            PresentationSectionKind::Description,
            description_blocks,
        ));
    }
    if !detail_facts.is_empty() || !detail_blocks.is_empty() {
        let mut blocks = Vec::new();
        if !detail_facts.is_empty() {
            blocks.push(PresentationBlock::FactList(detail_facts));
        }
        blocks.extend(detail_blocks);
        sections.push(PresentationSection::new(
            PresentationSectionKind::Details,
            blocks,
        ));
    }

    let mut badges = Vec::new();
    if let Some(rarity) = hazard.rarity.typed() {
        badges.push(PresentationBadge {
            kind: PresentationBadgeKind::Classification,
            label: "Rarity".to_string(),
            value: rarity.as_str().to_string(),
        });
    }
    if let Some(traits) = hazard.traits.typed() {
        badges.extend(traits.iter().map(|value| PresentationBadge {
            kind: PresentationBadgeKind::Trait,
            label: "Trait".to_string(),
            value: value.as_str().to_string(),
        }));
    }

    RecordPresentationDocument {
        record_key: hazard.identity.record_key.clone(),
        kind: RecordKind::Hazard,
        title: hazard.identity.name.clone(),
        identity: Vec::new(),
        badges,
        sections,
    }
}

fn project_defenses(
    defenses: &HazardDefenses,
    metrics_out: &mut Vec<crate::MetricRow>,
    terms: &mut Vec<String>,
) {
    push_defined_number(
        metrics_out,
        metrics::actor::ARMOR_CLASS,
        defenses.armor_class.typed().copied(),
    );
    push_defined_number(
        metrics_out,
        metrics::actor::HARDNESS,
        defenses.hardness.typed().copied(),
    );
    if let Some(hit_points) = defenses.hit_points.typed() {
        push_defined_number(
            metrics_out,
            metrics::actor::HP_VALUE,
            hit_points.current.typed().copied(),
        );
        push_defined_number(
            metrics_out,
            metrics::actor::HP_MAX,
            hit_points.maximum.typed().copied(),
        );
    }
    if let Some(saves) = defenses.saves.typed() {
        for (slug, value) in [
            ("fort", saves.fortitude.typed().copied()),
            ("ref", saves.reflex.typed().copied()),
            ("will", saves.will.typed().copied()),
        ] {
            if let Some(value) = value {
                metrics_out.push(crate::MetricRow {
                    domain: MetricDomain::Actor,
                    key: metrics::actor::save::mod_key(slug),
                    value: MetricValue::Number(value as f64),
                });
            }
        }
    }
    for (family, entries) in [
        ("immunity", defenses.immunities.typed()),
        ("weakness", defenses.weaknesses.typed()),
        ("resistance", defenses.resistances.typed()),
    ]
    .into_iter()
    .filter_map(|(family, entries)| entries.map(|entries| (family, entries)))
    {
        for entry in entries {
            let iwr_type = entry.iwr_type.typed();
            if let Some(iwr_type) = iwr_type {
                terms.push(iwr_type.clone());
            }
            if let Some(value) = entry.value.typed() {
                terms.push(match iwr_type {
                    Some(iwr_type) => format!("{family} {iwr_type} {value}"),
                    None => format!("{family} {value}"),
                });
            }
            if let Some(values) = entry.exceptions.typed() {
                terms.extend(values.iter().map(|value| format!("except {value}")));
            }
            if let Some(values) = entry.double_vs.typed() {
                terms.extend(values.iter().map(|value| format!("double vs {value}")));
            }
        }
    }
}

fn project_embedded_terms(entity: &HazardEntity, terms: &mut Vec<String>) {
    terms.push(entity.label.clone());
    match &entity.capability {
        HazardCapability::Action(action) => {
            terms.push("hazard action".to_string());
            project_common_terms(&action.common, terms);
            if let Some(value) = action.action_type.typed() {
                terms.push(action_type_label(value).to_ascii_lowercase());
            }
            if let Some(value) = action.actions.typed() {
                terms.push(format!("{} actions", value.value()));
            }
            if let Some(value) = action.category.typed() {
                terms.push(format!(
                    "{} action",
                    action_category_label(value).to_ascii_lowercase()
                ));
            }
            if let Some(value) = action.death_note.typed() {
                terms.push(format!("death note {value}"));
            }
            if let Some(value) = action.frequency.typed() {
                project_frequency_terms(value, terms);
            }
            if let Some(value) = action.self_effect.typed() {
                if let Some(target) = value.target_uuid.typed() {
                    terms.push(target.clone());
                }
                if let Some(label) = value.label.typed() {
                    terms.push(label.clone());
                }
            }
        }
        HazardCapability::Strike(strike) => {
            terms.push("hazard strike".to_string());
            project_common_terms(&strike.common, terms);
            if let Some(value) = strike.bonus.typed() {
                terms.push(format!("strike bonus {value:+}"));
            }
            if let Some(values) = strike.attack_effects.typed() {
                terms.extend(values.iter().cloned());
            }
            if let Some(values) = strike.damage_rolls.typed() {
                for damage in values {
                    project_damage_terms(damage, terms);
                }
            }
        }
        HazardCapability::Condition(condition) => {
            terms.push("hazard condition".to_string());
            project_common_terms(&condition.common, terms);
        }
        HazardCapability::Effect(effect) => {
            terms.push("hazard effect".to_string());
            project_common_terms(&effect.common, terms);
        }
        HazardCapability::UnsupportedChild(child) => {
            terms.push(format!("unsupported child {}", child.child_type));
            project_common_terms(&child.common, terms);
        }
    }
}

fn project_common_terms(common: &HazardItemCommon, terms: &mut Vec<String>) {
    if let Some(values) = common.traits.typed() {
        terms.extend(values.iter().map(|value| value.as_str().to_string()));
    }
    if let Some(rules) = common.rules.typed() {
        for rule in rules {
            project_rule_terms(rule, terms);
        }
    }
}

fn project_rule_terms(rule: &HazardRuleElement, terms: &mut Vec<String>) {
    match rule {
        HazardRuleElement::Immunity(rule) => {
            terms.push("immunity rule".to_string());
            if let Some(mode) = rule.mode.typed() {
                terms.push(format!("immunity mode {mode:?}").to_ascii_lowercase());
            }
            if let Some(types) = rule.immunity_types.typed() {
                match types {
                    HazardRuleType::Single(value) => terms.push(value.clone()),
                    HazardRuleType::Multiple(values) => terms.extend(values.iter().cloned()),
                }
            }
        }
        HazardRuleElement::ActiveEffectLike(rule) => {
            terms.push("active effect like rule".to_string());
            if let Some(mode) = rule.mode.typed() {
                terms.push(format!(
                    "active effect mode {}",
                    hazard_rule_mode_label(*mode)
                ));
            }
            if let Some(path) = rule.path.typed() {
                terms.push(path.clone());
            }
            if let Some(value) = rule.value.typed() {
                terms.push(format!("active effect value {value}"));
            }
        }
        HazardRuleElement::Aura(rule) => {
            terms.push("aura rule".to_string());
            if let Some(radius) = rule.radius.typed() {
                terms.push(format!("aura radius {radius}"));
            }
            if let Some(slug) = rule.slug.typed() {
                terms.push(slug.clone());
            }
            if let Some(traits) = rule.traits.typed() {
                terms.extend(traits.iter().map(|value| value.as_str().to_string()));
            }
        }
        HazardRuleElement::DamageDice(rule) => {
            terms.push("damage dice rule".to_string());
            for value in [
                rule.die_size.typed(),
                rule.damage_type.typed(),
                rule.selector.typed(),
            ]
            .into_iter()
            .flatten()
            {
                terms.push(value.clone());
            }
            if let Some(value) = rule.dice_number.typed() {
                terms.push(format!("damage dice number {value}"));
            }
            if let Some(value) = rule.critical.typed() {
                terms.push(format!("damage dice critical {value}"));
            }
        }
        HazardRuleElement::FlatModifier(rule) => {
            terms.push("flat modifier rule".to_string());
            for value in [rule.damage_type.typed(), rule.selector.typed()]
                .into_iter()
                .flatten()
            {
                terms.push(value.clone());
            }
            if let Some(value) = rule.value.typed() {
                terms.push(format!("flat modifier {value}"));
            }
            if let Some(value) = rule.critical.typed() {
                terms.push(format!("flat modifier critical {value}"));
            }
        }
        HazardRuleElement::Note(rule) => {
            terms.push("note rule".to_string());
            if let Some(values) = rule.outcomes.typed() {
                terms.extend(values.iter().cloned());
            }
            for value in [
                rule.selector.typed(),
                rule.title.typed(),
                rule.visibility.typed(),
            ]
            .into_iter()
            .flatten()
            {
                terms.push(value.clone());
            }
            if let Some(document) = rule.text.typed() {
                let text = crate::render_plain_text(document);
                if !text.is_empty() {
                    terms.push(text);
                }
            }
        }
        HazardRuleElement::Unsupported(_) => {}
    }
}

fn project_frequency_terms(frequency: &HazardFrequency, terms: &mut Vec<String>) {
    if let Some(value) = frequency.value.typed() {
        terms.push(format!("frequency value {value}"));
    }
    if let Some(value) = frequency.maximum.typed() {
        terms.push(format!("frequency maximum {value}"));
    }
    if let Some(value) = frequency.per.typed() {
        terms.push(format!("frequency per {}", frequency_interval_label(value)));
    }
}

fn project_damage_terms(damage: &HazardStrikeDamage, terms: &mut Vec<String>) {
    terms.push(format!("damage key {}", damage.source_key));
    if let Some(value) = damage.damage.typed() {
        terms.push(value.clone());
    }
    if let Some(value) = damage.damage_type.typed() {
        terms.push(value.clone());
    }
    if let Some(value) = damage.category.typed() {
        terms.push(damage_category_label(value).to_ascii_lowercase());
    }
}

fn push_embedded_presentation(
    output: &mut Vec<PresentationFact>,
    occurrence: &HazardEntityOccurrence,
    entity: &HazardEntity,
) {
    let prefix = format!("activity.{}", occurrence.id.as_str());
    let kind = match &entity.capability {
        HazardCapability::Action(action) => action
            .action_type
            .typed()
            .map(action_type_label)
            .unwrap_or("Action"),
        HazardCapability::Strike(_) => "Strike",
        HazardCapability::Condition(_) => "Condition",
        HazardCapability::Effect(_) => "Effect",
        HazardCapability::UnsupportedChild(_) => "Unsupported child",
    };
    push_owned_text_fact(output, &prefix, &entity.label, kind);

    match &entity.capability {
        HazardCapability::Action(action) => push_action_presentation(output, &prefix, action),
        HazardCapability::Strike(strike) => push_strike_presentation(output, &prefix, strike),
        HazardCapability::Condition(condition) => {
            push_common_presentation(output, &prefix, &condition.common)
        }
        HazardCapability::Effect(effect) => {
            push_common_presentation(output, &prefix, &effect.common)
        }
        HazardCapability::UnsupportedChild(child) => {
            push_owned_text_fact(
                output,
                &format!("{prefix}.child_type"),
                "Child Type",
                &child.child_type,
            );
            push_common_presentation(output, &prefix, &child.common);
        }
    }
}

fn push_action_presentation(
    output: &mut Vec<PresentationFact>,
    prefix: &str,
    action: &HazardActionCapability,
) {
    push_common_presentation(output, prefix, &action.common);
    if let Some(value) = action.actions.typed() {
        push_owned_text_fact(
            output,
            &format!("{prefix}.actions"),
            "Actions",
            &value.value().to_string(),
        );
    }
    if let Some(value) = action.category.typed() {
        push_owned_text_fact(
            output,
            &format!("{prefix}.category"),
            "Category",
            action_category_label(value),
        );
    }
    if let Some(value) = action.death_note.typed() {
        push_owned_text_fact(
            output,
            &format!("{prefix}.death_note"),
            "Death Note",
            if *value { "yes" } else { "no" },
        );
    }
    if let Some(frequency) = action.frequency.typed() {
        if let Some(value) = frequency.value.typed() {
            push_owned_text_fact(
                output,
                &format!("{prefix}.frequency.value"),
                "Frequency Value",
                &value.to_string(),
            );
        }
        if let Some(value) = frequency.maximum.typed() {
            push_owned_text_fact(
                output,
                &format!("{prefix}.frequency.maximum"),
                "Frequency Maximum",
                &value.to_string(),
            );
        }
        if let Some(value) = frequency.per.typed() {
            push_owned_text_fact(
                output,
                &format!("{prefix}.frequency.per"),
                "Frequency Period",
                frequency_interval_label(value),
            );
        }
    }
    if let Some(self_effect) = action.self_effect.typed() {
        if let Some(value) = self_effect.target_uuid.typed() {
            push_owned_text_fact(
                output,
                &format!("{prefix}.self_effect.target"),
                "Self Effect Target",
                value,
            );
        }
        if let Some(value) = self_effect.label.typed() {
            push_owned_text_fact(
                output,
                &format!("{prefix}.self_effect.label"),
                "Self Effect",
                value,
            );
        }
    }
}

fn push_strike_presentation(
    output: &mut Vec<PresentationFact>,
    prefix: &str,
    strike: &HazardStrikeCapability,
) {
    push_common_presentation(output, prefix, &strike.common);
    if let Some(value) = strike.bonus.typed() {
        push_owned_text_fact(
            output,
            &format!("{prefix}.bonus"),
            "Strike Bonus",
            &format!("{value:+}"),
        );
    }
    if let Some(values) = strike.attack_effects.typed()
        && !values.is_empty()
    {
        push_owned_text_fact(
            output,
            &format!("{prefix}.attack_effects"),
            "Attack Effects",
            &values.join(", "),
        );
    }
    if let Some(values) = strike.damage_rolls.typed() {
        for damage in values {
            let mut parts = Vec::new();
            if let Some(value) = damage.damage.typed() {
                parts.push(value.clone());
            }
            if let Some(value) = damage.damage_type.typed() {
                parts.push(value.clone());
            }
            if let Some(value) = damage.category.typed() {
                parts.push(damage_category_label(value).to_string());
            }
            if !parts.is_empty() {
                push_owned_text_fact(
                    output,
                    &format!("{prefix}.damage.{}", damage.source_key),
                    &format!("Damage {}", damage.source_key),
                    &parts.join(" "),
                );
            }
        }
    }
}

fn push_common_presentation(
    output: &mut Vec<PresentationFact>,
    prefix: &str,
    common: &HazardItemCommon,
) {
    if let Some(values) = common.traits.typed()
        && !values.is_empty()
    {
        push_owned_text_fact(
            output,
            &format!("{prefix}.traits"),
            "Traits",
            &values
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    if let Some(rules) = common.rules.typed() {
        for rule in rules {
            push_rule_presentation(output, prefix, rule);
        }
    }
}

fn push_rule_presentation(
    output: &mut Vec<PresentationFact>,
    prefix: &str,
    rule: &HazardRuleElement,
) {
    let (ordinal, value) = match rule {
        HazardRuleElement::Immunity(rule) => {
            let types = rule
                .immunity_types
                .typed()
                .map(|types| match types {
                    HazardRuleType::Single(value) => value.clone(),
                    HazardRuleType::Multiple(values) => values.join(", "),
                })
                .unwrap_or_default();
            let mode = rule
                .mode
                .typed()
                .map(|mode| hazard_rule_mode_label(*mode))
                .unwrap_or_default();
            (
                rule.authored_order,
                format!("Immunity {mode} {types}").trim().to_string(),
            )
        }
        HazardRuleElement::ActiveEffectLike(rule) => (
            rule.authored_order,
            format!(
                "ActiveEffectLike {} {} {}",
                rule.mode
                    .typed()
                    .map(|mode| hazard_rule_mode_label(*mode))
                    .unwrap_or_default(),
                rule.path.typed().cloned().unwrap_or_default(),
                rule.value
                    .typed()
                    .map(ToString::to_string)
                    .unwrap_or_default()
            )
            .trim()
            .to_string(),
        ),
        HazardRuleElement::Aura(rule) => {
            let core = match (rule.slug.typed(), rule.radius.typed()) {
                (Some(slug), Some(radius)) => format!("Aura {slug} {radius}"),
                (Some(slug), None) => format!("Aura {slug}"),
                (None, Some(radius)) => format!("Aura {radius}"),
                (None, None) => "Aura".to_string(),
            };
            let traits = rule
                .traits
                .typed()
                .map(|traits| {
                    traits
                        .iter()
                        .map(|value| value.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            (
                rule.authored_order,
                format!("{core} {traits}").trim().to_string(),
            )
        }
        HazardRuleElement::DamageDice(rule) => (
            rule.authored_order,
            format!(
                "DamageDice {}{} {} critical={} selector={}",
                rule.dice_number
                    .typed()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                rule.die_size.typed().cloned().unwrap_or_default(),
                rule.damage_type.typed().cloned().unwrap_or_default(),
                rule.critical
                    .typed()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                rule.selector.typed().cloned().unwrap_or_default()
            )
            .trim()
            .to_string(),
        ),
        HazardRuleElement::FlatModifier(rule) => (
            rule.authored_order,
            format!(
                "FlatModifier {} critical={} damageType={} selector={}",
                rule.value
                    .typed()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                rule.critical
                    .typed()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                rule.damage_type.typed().cloned().unwrap_or_default(),
                rule.selector.typed().cloned().unwrap_or_default()
            )
            .trim()
            .to_string(),
        ),
        HazardRuleElement::Note(rule) => (
            rule.authored_order,
            rule.title
                .typed()
                .cloned()
                .unwrap_or_else(|| "Note".to_string()),
        ),
        HazardRuleElement::Unsupported(rule) => (
            rule.authored_order,
            "Unsupported rule element (source retained)".to_string(),
        ),
    };
    push_owned_text_fact(output, &format!("{prefix}.rule.{ordinal}"), "Rule", &value);
}

fn hazard_rule_mode_label(mode: crate::HazardRuleMode) -> &'static str {
    match mode {
        crate::HazardRuleMode::Add => "add",
        crate::HazardRuleMode::Remove => "remove",
        crate::HazardRuleMode::Override => "override",
    }
}

fn push_owned_text_fact(output: &mut Vec<PresentationFact>, key: &str, label: &str, value: &str) {
    output.push(PresentationFact {
        key: key.to_string(),
        label: label.to_string(),
        value: value.to_string(),
    });
}

fn project_structured_disable_checks(
    document: &RichDocument,
    metrics_out: &mut Vec<crate::MetricRow>,
) {
    let mut values = Vec::new();
    let mut values_by_statistic = BTreeMap::<String, (i64, i64)>::new();
    visit_nodes(&document.nodes, &mut |node| {
        let FoundryNode::Check {
            statistic, options, ..
        } = node
        else {
            return;
        };
        if let Some(value) = options
            .get("dc")
            .and_then(|value| value.parse::<i64>().ok())
        {
            values.push(value);
            if let Some(statistic) = statistic {
                let statistic = metrics::normalize_metric_key_segment(statistic);
                if !statistic.is_empty() {
                    values_by_statistic
                        .entry(statistic)
                        .and_modify(|(minimum, maximum)| {
                            *minimum = (*minimum).min(value);
                            *maximum = (*maximum).max(value);
                        })
                        .or_insert((value, value));
                }
            }
        }
    });
    if let Some(value) = values.iter().min().copied() {
        push_defined_number(metrics_out, metrics::actor::disable::DC_MIN, Some(value));
    }
    if let Some(value) = values.iter().max().copied() {
        push_defined_number(metrics_out, metrics::actor::disable::DC_MAX, Some(value));
    }
    for (statistic, (minimum, maximum)) in values_by_statistic {
        push_number(
            metrics_out,
            metrics::actor::disable::skill_dc_min_key(&statistic),
            minimum,
        );
        push_number(
            metrics_out,
            metrics::actor::disable::skill_dc_max_key(&statistic),
            maximum,
        );
    }
}

fn visit_nodes(nodes: &[RichNode], visit: &mut impl FnMut(&FoundryNode)) {
    for node in nodes {
        match node {
            RichNode::Foundry { node } => visit(node),
            RichNode::HtmlElement { children, .. } => visit_nodes(children, visit),
            RichNode::FoundryLink { link } => {
                if let Some(label) = link.label.as_deref() {
                    visit_nodes(label, visit);
                }
            }
            RichNode::Text { .. } => {}
        }
    }
}

fn push_defined_number(
    output: &mut Vec<crate::MetricRow>,
    definition: metrics::MetricDefinition,
    value: Option<i64>,
) {
    if let (Some(key), Some(value)) = (definition.exact_key(), value) {
        output.push(crate::MetricRow {
            domain: MetricDomain::Actor,
            key: key.to_string(),
            value: MetricValue::Number(value as f64),
        });
    }
}

fn push_number(output: &mut Vec<crate::MetricRow>, key: String, value: i64) {
    output.push(crate::MetricRow {
        domain: MetricDomain::Actor,
        key,
        value: MetricValue::Number(value as f64),
    });
}

fn push_fact(output: &mut Vec<PresentationFact>, key: &str, label: &str, value: Option<i64>) {
    if let Some(value) = value {
        output.push(PresentationFact {
            key: key.to_string(),
            label: label.to_string(),
            value: value.to_string(),
        });
    }
}

fn push_modifier_fact(
    output: &mut Vec<PresentationFact>,
    key: &str,
    label: &str,
    value: Option<i64>,
) {
    if let Some(value) = value {
        output.push(PresentationFact {
            key: key.to_string(),
            label: label.to_string(),
            value: if value >= 0 {
                format!("+{value}")
            } else {
                value.to_string()
            },
        });
    }
}

fn push_text_fact(output: &mut Vec<PresentationFact>, key: &str, label: &str, value: Option<&str>) {
    if let Some(value) = value {
        output.push(PresentationFact {
            key: key.to_string(),
            label: label.to_string(),
            value: value.to_string(),
        });
    }
}

fn push_iwr_fact(
    output: &mut Vec<PresentationFact>,
    key: &str,
    label: &str,
    entries: &HazardFact<Vec<HazardIwr>>,
) {
    let Some(entries) = entries.typed() else {
        return;
    };
    let values = entries
        .iter()
        .filter_map(|entry| {
            let mut rendered = match (entry.iwr_type.typed(), entry.value.typed()) {
                (Some(iwr_type), Some(value)) => format!("{iwr_type} {value}"),
                (Some(iwr_type), None) => iwr_type.clone(),
                (None, Some(value)) => value.to_string(),
                (None, None) => String::new(),
            };
            if let Some(exceptions) = entry.exceptions.typed()
                && !exceptions.is_empty()
            {
                rendered.push_str(" (except ");
                rendered.push_str(&exceptions.join(", "));
                rendered.push(')');
            }
            if let Some(double_vs) = entry.double_vs.typed()
                && !double_vs.is_empty()
            {
                rendered.push_str(" (double vs ");
                rendered.push_str(&double_vs.join(", "));
                rendered.push(')');
            }
            (!rendered.is_empty()).then_some(rendered)
        })
        .collect::<Vec<_>>();
    if !values.is_empty() {
        push_text_fact(output, key, label, Some(&values.join(", ")));
    }
}

fn push_document(output: &mut Vec<PresentationBlock>, fact: &HazardFact<RichDocument>) {
    if let Some(document) = fact.typed() {
        let content = project_presentation_content(document);
        if !content.is_empty() {
            output.push(PresentationBlock::Content(content));
        }
    }
}

fn push_labeled_document(
    output: &mut Vec<PresentationBlock>,
    label: &str,
    fact: &HazardFact<RichDocument>,
) {
    let Some(document) = fact.typed() else {
        return;
    };
    let mut content = project_presentation_content(document);
    if content.is_empty() {
        return;
    }
    content.blocks.insert(
        0,
        crate::PresentationContentBlock::Heading {
            level: 3,
            text: label.to_string(),
        },
    );
    output.push(PresentationBlock::Content(content));
}

fn push_fact_section(
    sections: &mut Vec<PresentationSection>,
    kind: PresentationSectionKind,
    facts: Vec<PresentationFact>,
) {
    if !facts.is_empty() {
        sections.push(PresentationSection::new(
            kind,
            vec![PresentationBlock::FactList(facts)],
        ));
    }
}

fn action_type_label(value: &HazardActionType) -> &'static str {
    match value {
        HazardActionType::Action => "Action",
        HazardActionType::Reaction => "Reaction",
        HazardActionType::Free => "Free action",
        HazardActionType::Passive => "Passive",
    }
}

fn action_category_label(value: &crate::HazardActionCategory) -> &'static str {
    match value {
        crate::HazardActionCategory::Interaction => "Interaction",
        crate::HazardActionCategory::Defensive => "Defensive",
        crate::HazardActionCategory::Offensive => "Offensive",
        crate::HazardActionCategory::Familiar => "Familiar",
    }
}

fn frequency_interval_label(value: &HazardFrequencyInterval) -> &'static str {
    match value {
        HazardFrequencyInterval::Turn => "turn",
        HazardFrequencyInterval::Round => "round",
        HazardFrequencyInterval::OneMinute => "1 minute",
        HazardFrequencyInterval::TenMinutes => "10 minutes",
        HazardFrequencyInterval::OneHour => "1 hour",
        HazardFrequencyInterval::TwentyFourHours => "24 hours",
        HazardFrequencyInterval::Day => "day",
        HazardFrequencyInterval::Week => "week",
        HazardFrequencyInterval::Month => "month",
        HazardFrequencyInterval::Year => "year",
    }
}

fn damage_category_label(value: &crate::HazardDamageCategory) -> &'static str {
    match value {
        crate::HazardDamageCategory::Persistent => "persistent",
        crate::HazardDamageCategory::Precision => "precision",
        crate::HazardDamageCategory::Splash => "splash",
    }
}

fn emits_sound_label(value: &HazardEmitsSound) -> &str {
    match value {
        HazardEmitsSound::Boolean(true) => "yes",
        HazardEmitsSound::Boolean(false) => "no",
        HazardEmitsSound::Named(value) => value,
    }
}
