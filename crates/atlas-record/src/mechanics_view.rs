use atlas_domain::{MetricDomain, RecordKey, RecordKind};

use crate::{
    AtlasRecord, MechanicActivity, MetricDefinitionMatch, MetricRow, MetricValue, definition_for,
    metrics,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanicsView {
    pub record_key: RecordKey,
    pub kind: RecordKind,
    pub title: String,
    pub level: Option<i64>,
    pub values: Vec<MechanicValue>,
    pub speeds: Vec<MovementSpeed>,
    pub activities: Vec<MechanicActivity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementSpeed {
    pub movement_type: String,
    pub label: String,
    pub value_feet: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanicValue {
    pub target: MechanicTarget,
    pub label: String,
    pub base_value: MechanicScalar,
    pub facets: MechanicFacets,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicTarget {
    ArmorClass,
    MaxHp,
    Perception,
    Save { save: SaveKind },
    Skill { slug: String },
    AbilityModifier { ability: AbilityKind },
}

impl MechanicTarget {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanicFacets {
    pub surface: MechanicSurface,
    pub statistic: Option<MechanicStatistic>,
    pub ability: Option<AbilityKind>,
}

impl MechanicFacets {
    pub const fn armor_class() -> Self {
        Self {
            surface: MechanicSurface::ArmorClass,
            statistic: None,
            ability: Some(AbilityKind::Dexterity),
        }
    }

    pub const fn hit_points() -> Self {
        Self {
            surface: MechanicSurface::HitPoints,
            statistic: None,
            ability: None,
        }
    }

    pub const fn perception() -> Self {
        Self {
            surface: MechanicSurface::Check,
            statistic: Some(MechanicStatistic::Perception),
            ability: Some(AbilityKind::Wisdom),
        }
    }

    pub const fn saving_throw(save: SaveKind) -> Self {
        let ability = match save {
            SaveKind::Fortitude => AbilityKind::Constitution,
            SaveKind::Reflex => AbilityKind::Dexterity,
            SaveKind::Will => AbilityKind::Wisdom,
        };
        Self {
            surface: MechanicSurface::SavingThrow,
            statistic: Some(MechanicStatistic::Save(save)),
            ability: Some(ability),
        }
    }

    pub fn skill(slug: &str) -> Self {
        Self {
            surface: MechanicSurface::Check,
            statistic: None,
            ability: skill_ability(slug),
        }
    }

    pub const fn ability_modifier(ability: AbilityKind) -> Self {
        Self {
            surface: MechanicSurface::RawModifier,
            statistic: Some(MechanicStatistic::Ability(ability)),
            ability: Some(ability),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechanicSurface {
    RawModifier,
    Check,
    Dc,
    ArmorClass,
    SavingThrow,
    AttackRoll,
    Damage,
    HitPoints,
}

impl MechanicSurface {
    pub const fn is_check_or_dc(self) -> bool {
        matches!(
            self,
            Self::Check | Self::Dc | Self::ArmorClass | Self::SavingThrow | Self::AttackRoll
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicStatistic {
    Ability(AbilityKind),
    Perception,
    Save(SaveKind),
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
pub enum MechanicScalar {
    Number(i64),
}

pub fn build_mechanics_view(record: &AtlasRecord) -> Option<MechanicsView> {
    if record.classification.kind != RecordKind::Creature {
        return None;
    }
    let mut values = Vec::new();
    push_exact(
        &mut values,
        &record.mechanics.metrics,
        metrics::actor::ARMOR_CLASS,
        MechanicTarget::ArmorClass,
        "AC",
        MechanicFacets::armor_class(),
    );
    push_exact(
        &mut values,
        &record.mechanics.metrics,
        metrics::actor::HP_MAX,
        MechanicTarget::MaxHp,
        "Max HP",
        MechanicFacets::hit_points(),
    );
    push_exact(
        &mut values,
        &record.mechanics.metrics,
        metrics::actor::PERCEPTION_MOD,
        MechanicTarget::Perception,
        "Perception",
        MechanicFacets::perception(),
    );
    for save in [SaveKind::Fortitude, SaveKind::Reflex, SaveKind::Will] {
        push_key(
            &mut values,
            &record.mechanics.metrics,
            &metrics::actor::save::mod_key(save.as_str()),
            MechanicTarget::Save { save },
            save.label(),
            MechanicFacets::saving_throw(save),
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
            MechanicTarget::AbilityModifier { ability },
            ability.label(),
            MechanicFacets::ability_modifier(ability),
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
    let speeds = speed_values(&record.mechanics.metrics);
    if values.is_empty() && speeds.is_empty() {
        return None;
    }
    Some(MechanicsView {
        record_key: record.identity.key.clone(),
        kind: record.classification.kind,
        title: record.identity.name.clone(),
        level: record.classification.level,
        values,
        speeds,
        activities: record.mechanics.activities.clone(),
    })
}

fn push_exact(
    values: &mut Vec<MechanicValue>,
    metrics: &[MetricRow],
    definition: crate::MetricDefinition,
    target: MechanicTarget,
    fallback_label: &str,
    facets: MechanicFacets,
) {
    if let Some(key) = definition.exact_key() {
        push_key(values, metrics, key, target, fallback_label, facets);
    }
}

fn push_key(
    values: &mut Vec<MechanicValue>,
    metrics: &[MetricRow],
    key: &str,
    target: MechanicTarget,
    fallback_label: &str,
    facets: MechanicFacets,
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
    values.push(MechanicValue {
        target,
        label: fallback_label.to_string(),
        base_value: MechanicScalar::Number(value),
        facets,
    });
}

fn skill_value(metric: &MetricRow) -> Option<MechanicValue> {
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
    Some(MechanicValue {
        target: MechanicTarget::Skill {
            slug: skill.raw.clone(),
        },
        label: skill.label.clone(),
        base_value: MechanicScalar::Number(value),
        facets: MechanicFacets::skill(&skill.raw),
    })
}

fn speed_values(metrics: &[MetricRow]) -> Vec<MovementSpeed> {
    let mut speeds = metrics.iter().filter_map(speed_value).collect::<Vec<_>>();
    speeds.sort_by(|left, right| {
        speed_sort_key(&left.movement_type).cmp(&speed_sort_key(&right.movement_type))
    });
    speeds
}

fn speed_value(metric: &MetricRow) -> Option<MovementSpeed> {
    if metric.domain != MetricDomain::Actor {
        return None;
    }
    let MetricDefinitionMatch {
        definition,
        captures,
    } = definition_for(metric.domain, &metric.key)?;
    if *definition != metrics::actor::speed::VALUE {
        return None;
    }
    let movement = captures.first()?;
    let value_feet = metric_i64(metric)?;
    if value_feet <= 0 {
        return None;
    }
    Some(MovementSpeed {
        movement_type: movement.raw.clone(),
        label: format!("{} Speed", movement.label),
        value_feet,
    })
}

fn speed_sort_key(movement_type: &str) -> (u8, &str) {
    if movement_type == "land" {
        (0, movement_type)
    } else {
        (1, movement_type)
    }
}

fn skill_ability(slug: &str) -> Option<AbilityKind> {
    match slug {
        "acr" | "acrobatics" | "ste" | "stealth" | "thi" | "thievery" => {
            Some(AbilityKind::Dexterity)
        }
        "ath" | "athletics" => Some(AbilityKind::Strength),
        "arc" | "arcana" | "cra" | "crafting" | "occ" | "occultism" | "soc" | "society" => {
            Some(AbilityKind::Intelligence)
        }
        "med" | "medicine" | "nat" | "nature" | "rel" | "religion" | "sur" | "survival" => {
            Some(AbilityKind::Wisdom)
        }
        "dec" | "deception" | "dip" | "diplomacy" | "itm" | "intimidation" | "prf"
        | "performance" => Some(AbilityKind::Charisma),
        _ => None,
    }
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
    fn creature_mechanics_view_extracts_typed_numeric_targets() {
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
            metric(&metrics::actor::speed::value_key("fly"), 40.0),
            metric(&metrics::actor::speed::value_key("land"), 25.0),
        ];

        let view = build_mechanics_view(&record).expect("creature should project mechanics");

        assert_eq!(view.record_key, record.identity.key);
        assert_eq!(view.kind, RecordKind::Creature);
        assert_eq!(view.level, Some(3));
        assert_value(
            &view,
            MechanicTarget::ArmorClass,
            "AC",
            19,
            MechanicSurface::ArmorClass,
            Some(AbilityKind::Dexterity),
        );
        assert_value(
            &view,
            MechanicTarget::MaxHp,
            "Max HP",
            45,
            MechanicSurface::HitPoints,
            None,
        );
        assert_value(
            &view,
            MechanicTarget::Perception,
            "Perception",
            9,
            MechanicSurface::Check,
            Some(AbilityKind::Wisdom),
        );
        assert_value(
            &view,
            MechanicTarget::Save {
                save: SaveKind::Fortitude,
            },
            "Fortitude",
            12,
            MechanicSurface::SavingThrow,
            Some(AbilityKind::Constitution),
        );
        assert_value(
            &view,
            MechanicTarget::Save {
                save: SaveKind::Reflex,
            },
            "Reflex",
            8,
            MechanicSurface::SavingThrow,
            Some(AbilityKind::Dexterity),
        );
        assert_value(
            &view,
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Strength,
            },
            "Strength",
            4,
            MechanicSurface::RawModifier,
            Some(AbilityKind::Strength),
        );
        assert_value(
            &view,
            MechanicTarget::Skill {
                slug: "athletics".to_string(),
            },
            "Athletics",
            11,
            MechanicSurface::Check,
            Some(AbilityKind::Strength),
        );
        assert_value(
            &view,
            MechanicTarget::Skill {
                slug: "stealth".to_string(),
            },
            "Stealth",
            8,
            MechanicSurface::Check,
            Some(AbilityKind::Dexterity),
        );
        assert_eq!(
            view.speeds,
            vec![
                MovementSpeed {
                    movement_type: "land".to_string(),
                    label: "Land Speed".to_string(),
                    value_feet: 25,
                },
                MovementSpeed {
                    movement_type: "fly".to_string(),
                    label: "Fly Speed".to_string(),
                    value_feet: 40,
                },
            ]
        );
    }

    #[test]
    fn non_creature_records_do_not_project_creature_stats() {
        let record = base_record(RecordKind::Spell);

        assert!(build_mechanics_view(&record).is_none());
    }

    #[test]
    fn creature_mechanics_view_does_not_project_absent_zero_speeds() {
        let mut record = base_record(RecordKind::Creature);
        record.mechanics.metrics = vec![
            defined_metric(metrics::actor::ARMOR_CLASS, 19.0),
            metric(&metrics::actor::speed::value_key("land"), 0.0),
            metric(&metrics::actor::speed::value_key("fly"), 40.0),
        ];

        let view = build_mechanics_view(&record).expect("creature should project mechanics");

        assert_eq!(
            view.speeds,
            vec![MovementSpeed {
                movement_type: "fly".to_string(),
                label: "Fly Speed".to_string(),
                value_feet: 40,
            }]
        );
    }

    #[test]
    fn creature_mechanics_view_projects_land_only_speed() {
        let mut record = base_record(RecordKind::Creature);
        record.mechanics.metrics = vec![
            defined_metric(metrics::actor::ARMOR_CLASS, 19.0),
            metric(&metrics::actor::speed::value_key("land"), 25.0),
        ];

        let view = build_mechanics_view(&record).expect("creature should project mechanics");

        assert_eq!(
            view.speeds,
            vec![MovementSpeed {
                movement_type: "land".to_string(),
                label: "Land Speed".to_string(),
                value_feet: 25,
            }]
        );
    }

    #[test]
    fn creature_mechanics_view_projects_additional_movement_types_once() {
        let mut record = base_record(RecordKind::Creature);
        record.mechanics.metrics = vec![
            defined_metric(metrics::actor::ARMOR_CLASS, 19.0),
            metric(&metrics::actor::speed::value_key("swim"), 20.0),
            metric(&metrics::actor::speed::value_key("climb"), 15.0),
            metric(&metrics::actor::speed::value_key("fly"), 40.0),
        ];

        let view = build_mechanics_view(&record).expect("creature should project mechanics");

        assert_eq!(
            view.speeds,
            vec![
                MovementSpeed {
                    movement_type: "climb".to_string(),
                    label: "Climb Speed".to_string(),
                    value_feet: 15,
                },
                MovementSpeed {
                    movement_type: "fly".to_string(),
                    label: "Fly Speed".to_string(),
                    value_feet: 40,
                },
                MovementSpeed {
                    movement_type: "swim".to_string(),
                    label: "Swim Speed".to_string(),
                    value_feet: 20,
                },
            ]
        );
    }

    fn assert_value(
        view: &MechanicsView,
        target: MechanicTarget,
        label: &str,
        value: i64,
        surface: MechanicSurface,
        ability: Option<AbilityKind>,
    ) {
        let stat = view
            .values
            .iter()
            .find(|stat| stat.target == target)
            .unwrap_or_else(|| panic!("missing stat target {}", target.id()));
        assert_eq!(stat.label, label);
        assert_eq!(stat.base_value, MechanicScalar::Number(value));
        assert_eq!(stat.facets.surface, surface);
        assert_eq!(stat.facets.ability, ability);
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
