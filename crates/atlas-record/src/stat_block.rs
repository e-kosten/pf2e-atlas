use atlas_domain::{MetricDomain, RecordKey, RecordKind};

use crate::{AtlasRecord, MetricDefinitionMatch, MetricRow, MetricValue, definition_for, metrics};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatBlock {
    pub record_key: RecordKey,
    pub kind: RecordKind,
    pub title: String,
    pub level: Option<i64>,
    pub values: Vec<StatValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatValue {
    pub target: StatTarget,
    pub label: String,
    pub base_value: StatScalar,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StatTarget {
    ArmorClass,
    MaxHp,
    Perception,
    Save { save: SaveKind },
    Skill { slug: String },
    AbilityModifier { ability: AbilityKind },
}

impl StatTarget {
    pub fn id(&self) -> String {
        match self {
            Self::ArmorClass => "ac".to_string(),
            Self::MaxHp => "hp.max".to_string(),
            Self::Perception => "perception".to_string(),
            Self::Save { save } => format!("save.{}", save.as_str()),
            Self::Skill { slug } => format!("skill.{slug}"),
            Self::AbilityModifier { ability } => format!("ability.{}", ability.as_str()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveKind {
    Fortitude,
    Reflex,
    Will,
}

impl SaveKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fortitude => "fort",
            Self::Reflex => "ref",
            Self::Will => "will",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Fortitude => "Fortitude",
            Self::Reflex => "Reflex",
            Self::Will => "Will",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbilityKind {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl AbilityKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Strength => "str",
            Self::Dexterity => "dex",
            Self::Constitution => "con",
            Self::Intelligence => "int",
            Self::Wisdom => "wis",
            Self::Charisma => "cha",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Dexterity",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatScalar {
    Number(i64),
}

pub fn build_stat_block(record: &AtlasRecord) -> Option<StatBlock> {
    if record.classification.kind != RecordKind::Creature {
        return None;
    }
    let mut values = Vec::new();
    push_exact(
        &mut values,
        &record.mechanics.metrics,
        metrics::actor::ARMOR_CLASS,
        StatTarget::ArmorClass,
        "AC",
    );
    push_exact(
        &mut values,
        &record.mechanics.metrics,
        metrics::actor::HP_MAX,
        StatTarget::MaxHp,
        "Max HP",
    );
    push_exact(
        &mut values,
        &record.mechanics.metrics,
        metrics::actor::PERCEPTION_MOD,
        StatTarget::Perception,
        "Perception",
    );
    for save in [SaveKind::Fortitude, SaveKind::Reflex, SaveKind::Will] {
        push_key(
            &mut values,
            &record.mechanics.metrics,
            &metrics::actor::save::mod_key(save.as_str()),
            StatTarget::Save { save },
            save.label(),
        );
    }
    for ability in [
        AbilityKind::Strength,
        AbilityKind::Dexterity,
        AbilityKind::Constitution,
        AbilityKind::Intelligence,
        AbilityKind::Wisdom,
        AbilityKind::Charisma,
    ] {
        push_key(
            &mut values,
            &record.mechanics.metrics,
            &metrics::actor::ability::mod_key(ability.as_str()),
            StatTarget::AbilityModifier { ability },
            ability.label(),
        );
    }
    let mut skill_values = record
        .mechanics
        .metrics
        .iter()
        .filter_map(skill_value)
        .collect::<Vec<_>>();
    skill_values.sort_by(|left, right| left.label.cmp(&right.label));
    values.extend(skill_values);
    if values.is_empty() {
        return None;
    }
    Some(StatBlock {
        record_key: record.identity.key.clone(),
        kind: record.classification.kind,
        title: record.identity.name.clone(),
        level: record.classification.level,
        values,
    })
}

fn push_exact(
    values: &mut Vec<StatValue>,
    metrics: &[MetricRow],
    definition: crate::MetricDefinition,
    target: StatTarget,
    fallback_label: &str,
) {
    if let Some(key) = definition.exact_key() {
        push_key(values, metrics, key, target, fallback_label);
    }
}

fn push_key(
    values: &mut Vec<StatValue>,
    metrics: &[MetricRow],
    key: &str,
    target: StatTarget,
    fallback_label: &str,
) {
    let Some(metric) = metrics
        .iter()
        .find(|metric| metric.domain == MetricDomain::Actor && metric.key == key)
    else {
        return;
    };
    let Some(value) = metric_i64(metric) else {
        return;
    };
    values.push(StatValue {
        target,
        label: fallback_label.to_string(),
        base_value: StatScalar::Number(value),
    });
}

fn skill_value(metric: &MetricRow) -> Option<StatValue> {
    if metric.domain != MetricDomain::Actor {
        return None;
    }
    let MetricDefinitionMatch {
        definition,
        captures,
    } = definition_for(metric.domain, &metric.key)?;
    if *definition != metrics::actor::skill::MOD {
        return None;
    }
    let skill = captures.first()?;
    let value = metric_i64(metric)?;
    Some(StatValue {
        target: StatTarget::Skill {
            slug: skill.raw.clone(),
        },
        label: skill.label.clone(),
        base_value: StatScalar::Number(value),
    })
}

fn metric_i64(metric: &MetricRow) -> Option<i64> {
    let MetricValue::Number(value) = metric.value else {
        return None;
    };
    (value.fract() == 0.0).then_some(value as i64)
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, PublicationCategory, Rarity, RecordId, RecordKey, RecordKind};

    use super::*;
    use crate::{
        ActorMechanics, AtlasRecord, ContentSourceKind, FoundryDocumentMechanics,
        FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, RecordClassification,
        RecordContent, RecordIdentity, RecordMechanics, RecordProvenance, RecordPublication,
        RecordRequirements, RecordTaxonomy, RecordTiming, RecordVisibility, RichDocument, RichNode,
    };

    #[test]
    fn creature_stat_block_extracts_typed_numeric_targets() {
        let mut record = base_record(RecordKind::Creature);
        record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics {
            size: Some("medium".to_string()),
            languages: Vec::new(),
            speed_types: Vec::new(),
            senses: Vec::new(),
            immunities: Vec::new(),
            resistances: Vec::new(),
            weaknesses: Vec::new(),
            disable_text: None,
            disable_skills: Vec::new(),
            is_complex: false,
        });
        record.mechanics.metrics = vec![
            defined_metric(metrics::actor::ARMOR_CLASS, 19.0),
            defined_metric(metrics::actor::HP_MAX, 45.0),
            defined_metric(metrics::actor::PERCEPTION_MOD, 9.0),
            metric(&metrics::actor::save::mod_key("fort"), 12.0),
            metric(&metrics::actor::save::mod_key("ref"), 8.0),
            metric(&metrics::actor::save::mod_key("will"), 7.0),
            metric(&metrics::actor::ability::mod_key("str"), 4.0),
            metric(&metrics::actor::ability::mod_key("dex"), 2.0),
            metric(&metrics::actor::skill::mod_key("athletics"), 11.0),
            metric(&metrics::actor::skill::mod_key("stealth"), 8.0),
        ];

        let block = build_stat_block(&record).expect("creature should project stats");

        assert_eq!(block.record_key, record.identity.key);
        assert_eq!(block.kind, RecordKind::Creature);
        assert_eq!(block.level, Some(3));
        assert_stat(&block, StatTarget::ArmorClass, "AC", 19);
        assert_stat(&block, StatTarget::MaxHp, "Max HP", 45);
        assert_stat(&block, StatTarget::Perception, "Perception", 9);
        assert_stat(
            &block,
            StatTarget::Save {
                save: SaveKind::Fortitude,
            },
            "Fortitude",
            12,
        );
        assert_stat(
            &block,
            StatTarget::Save {
                save: SaveKind::Reflex,
            },
            "Reflex",
            8,
        );
        assert_stat(
            &block,
            StatTarget::AbilityModifier {
                ability: AbilityKind::Strength,
            },
            "Strength",
            4,
        );
        assert_stat(
            &block,
            StatTarget::Skill {
                slug: "athletics".to_string(),
            },
            "Athletics",
            11,
        );
        assert_stat(
            &block,
            StatTarget::Skill {
                slug: "stealth".to_string(),
            },
            "Stealth",
            8,
        );
    }

    #[test]
    fn non_creature_records_do_not_project_creature_stats() {
        let record = base_record(RecordKind::Spell);

        assert!(build_stat_block(&record).is_none());
    }

    fn assert_stat(block: &StatBlock, target: StatTarget, label: &str, value: i64) {
        let stat = block
            .values
            .iter()
            .find(|stat| stat.target == target)
            .unwrap_or_else(|| panic!("missing stat target {}", target.id()));
        assert_eq!(stat.label, label);
        assert_eq!(stat.base_value, StatScalar::Number(value));
    }

    fn base_record(kind: RecordKind) -> AtlasRecord {
        AtlasRecord {
            identity: RecordIdentity {
                key: RecordKey::new(
                    PackName::new("test-pack").expect("pack should parse"),
                    RecordId::new("TestRecord").expect("id should parse"),
                ),
                name: "Test Record".to_string(),
            },
            classification: RecordClassification {
                kind,
                level: Some(3),
                rarity: Some(Rarity::Uncommon),
                traits: Vec::new(),
                taxonomy: RecordTaxonomy::default(),
            },
            foundry: FoundryRecordInfo {
                pack_label: "Test Pack".to_string(),
                document_type: FoundryDocumentType::Item,
                record_type: FoundryRecordType::from_foundry(kind.as_str()),
                folder_id: None,
            },
            provenance: RecordProvenance {
                source_path: "packs/test-pack/TestRecord.json".to_string(),
                raw_json: Some("{}".to_string()),
            },
            publication: RecordPublication {
                title: Some("Player Core".to_string()),
                remaster: true,
                category: PublicationCategory::Core,
            },
            requirements: RecordRequirements::default(),
            timing: RecordTiming::default(),
            mechanics: RecordMechanics::default(),
            content: RecordContent {
                documents: vec![crate::RecordContentDocument {
                    source_kind: ContentSourceKind::Description,
                    label: None,
                    document: RichDocument {
                        nodes: vec![RichNode::Text {
                            text: "description".to_string(),
                        }],
                    },
                }],
            },
            variant: None,
            visibility: RecordVisibility::default(),
        }
    }

    fn defined_metric(definition: crate::MetricDefinition, value: f64) -> MetricRow {
        metric(definition.exact_key().expect("static key"), value)
    }

    fn metric(key: &str, value: f64) -> MetricRow {
        MetricRow {
            domain: MetricDomain::Actor,
            key: key.to_string(),
            value: MetricValue::Number(value),
        }
    }
}
