mod creature;

use std::ops::Deref;

use atlas_domain::{DetailLevel, RecordKind};
use serde::Serialize;

pub use creature::{
    CreatureActionJson, CreatureActivityModeJson, CreatureArmorClassJson, CreatureDamageJson,
    CreatureDefensesJson, CreatureHitPointsJson, CreatureIwrJson, CreatureMovementJson,
    CreatureMovementModeJson, CreatureResourceJson, CreatureRollJson, CreatureSaveJson,
    CreatureSavesJson, CreatureSenseJson, CreatureSkillJson, CreatureSpellJson,
    CreatureSpellcastingEntryJson, CreatureSpellcastingJson, CreatureStrikeJson,
};

use crate::{
    AtlasRecord, PresentationBlock, PresentationContent, PresentationFact,
    PresentationRelationship, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, build_record_presentation_document, render_plain_text,
};

const DESCRIPTION_PREVIEW_WORDS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordJsonOptions {
    pub detail: DetailLevel,
    pub include_source_json: bool,
}

/// The one durable CLI/agent record presentation contract.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecordJson {
    #[serde(flatten)]
    pub base: RecordJsonBase,
    #[serde(flatten)]
    pub presentation: RecordPresentationJson,
}

impl Deref for RecordJson {
    type Target = RecordJsonBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl RecordJson {
    pub fn generic_sections(&self) -> &[RecordSectionJson] {
        match &self.presentation {
            RecordPresentationJson::Creature { .. } => &[],
            RecordPresentationJson::Unmigrated { sections, .. } => sections,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecordJsonBase {
    pub key: String,
    pub name: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<RecordSourceJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supplementary_sections: Vec<RecordSectionJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "presentation_type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // The serialized variants intentionally stay flat.
pub enum RecordPresentationJson {
    Creature {
        #[serde(skip_serializing_if = "Option::is_none")]
        defenses: Option<CreatureDefensesJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        perception: Option<CreaturePerceptionJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        languages: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        skills: Option<Vec<CreatureSkillJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        movement: Option<CreatureMovementJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resources: Option<Vec<CreatureResourceJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strikes: Option<Vec<CreatureStrikeJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<CreatureActionJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        spellcasting: Option<CreatureSpellcastingJson>,
    },
    Unmigrated {
        migration: UnmigratedRegistryJson,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        sections: Vec<RecordSectionJson>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreaturePerceptionJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<i64>,
    pub senses: Vec<CreatureSenseJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnmigratedRegistryJson {
    pub family: &'static str,
    pub plan_id: &'static str,
    pub acceptance_checkpoint: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordSourceJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<RecordPackJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_remaster: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foundry: Option<FoundrySourceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordPackJson {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FoundrySourceJson {
    pub document_type: String,
    pub record_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordSectionJson {
    pub kind: &'static str,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<RecordBlockJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordBlockJson {
    FactList {
        facts: Vec<RecordFactJson>,
    },
    Prose {
        text: String,
    },
    Content {
        content: PresentationContent,
    },
    Relationships {
        relationships: Vec<RecordRelationshipJson>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordFactJson {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordRelationshipJson {
    pub kind: &'static str,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_key: Option<String>,
}

pub fn record_json(record: &AtlasRecord, options: RecordJsonOptions) -> RecordJson {
    let document = build_record_presentation_document(record);
    let detailed_sections = sections_for_detail(record, &document.sections, options.detail);
    let presentation = if record.classification.kind == RecordKind::Creature {
        creature::creature_presentation(record, options.detail)
    } else {
        RecordPresentationJson::Unmigrated {
            migration: unmigrated_registry(record),
            sections: generic_sections(&detailed_sections),
        }
    };

    RecordJson {
        base: RecordJsonBase {
            key: record.identity.key.to_string(),
            name: record.identity.name.clone(),
            kind: record.classification.kind.as_str(),
            level: record.classification.level,
            rarity: record
                .classification
                .rarity
                .map(|rarity| rarity.as_str().to_string()),
            traits: record.classification.traits.clone(),
            source: source_json(record, options.detail),
            supplementary_sections: supplementary_sections(&detailed_sections),
            source_json: options
                .include_source_json
                .then(|| record.provenance.raw_json.clone())
                .flatten(),
        },
        presentation,
    }
}

fn source_json(record: &AtlasRecord, detail: DetailLevel) -> Option<RecordSourceJson> {
    let full = detail == DetailLevel::Full;
    Some(RecordSourceJson {
        publication_title: record.publication.title.clone(),
        pack: Some(RecordPackJson {
            name: record.identity.pack().to_string(),
            label: record.foundry.pack_label.clone(),
        }),
        category: full.then(|| record.publication.category.as_str()),
        publication_remaster: full.then_some(record.publication.remaster),
        source_path: full.then(|| record.provenance.source_path.clone()),
        foundry: full.then(|| FoundrySourceJson {
            document_type: record.foundry.document_type.as_str().to_string(),
            record_type: record.foundry.record_type.as_str().to_string(),
        }),
    })
}

fn unmigrated_registry(record: &AtlasRecord) -> UnmigratedRegistryJson {
    use crate::FoundryRecordType;

    let (family, plan_id) = match record.classification.kind {
        // This private registry is called only from the non-creature branch in
        // `record_json`; retain an explicit entry so the match stays total.
        RecordKind::Creature => ("creature", "D2"),
        RecordKind::Hazard => ("hazard", "H1"),
        RecordKind::Spell => ("spell_or_ritual", "H2"),
        RecordKind::Equipment => match record.foundry.record_type {
            FoundryRecordType::Weapon | FoundryRecordType::Ammo => ("weapon_or_ammunition", "H3"),
            FoundryRecordType::Armor | FoundryRecordType::Shield => ("armor_or_shield", "H4"),
            FoundryRecordType::Consumable => ("consumable", "H5"),
            _ => ("remaining_physical_item", "H6"),
        },
        RecordKind::Feat
        | RecordKind::Affliction
        | RecordKind::Rule
        | RecordKind::CharacterOption => ("rules_content", "H7"),
        RecordKind::Lore | RecordKind::CampaignFeature => ("journal_or_table_content", "H8"),
        RecordKind::Character | RecordKind::Companion | RecordKind::Army | RecordKind::Vehicle => {
            ("actor_family", "H9")
        }
        RecordKind::Tooling => ("generated_container_or_embedded_context", "H10"),
    };
    UnmigratedRegistryJson {
        family,
        plan_id,
        acceptance_checkpoint: "H12",
    }
}

fn sections_for_detail(
    record: &AtlasRecord,
    sections: &[PresentationSection],
    detail: DetailLevel,
) -> Vec<RecordSectionJson> {
    let mut projected = Vec::new();
    if let Some(summary) = sections
        .iter()
        .find(|section| section.kind == PresentationSectionKind::Summary)
        .and_then(section_json)
    {
        projected.push(summary);
    }
    match detail {
        DetailLevel::Summary => {}
        DetailLevel::Preview => {
            projected.extend(
                sections
                    .iter()
                    .filter(|section| {
                        !matches!(
                            section.kind,
                            PresentationSectionKind::Summary
                                | PresentationSectionKind::Description
                                | PresentationSectionKind::Details
                        )
                    })
                    .filter_map(section_json),
            );
            if let Some(preview) = description_preview_section(record) {
                projected.push(preview);
            }
        }
        DetailLevel::Description => {
            if let Some(description) = sections
                .iter()
                .find(|section| section.kind == PresentationSectionKind::Description)
                .and_then(section_json)
            {
                projected.push(description);
            }
        }
        DetailLevel::Standard | DetailLevel::Full => {
            projected.extend(
                sections
                    .iter()
                    .filter(|section| section.kind != PresentationSectionKind::Summary)
                    .filter_map(section_json),
            );
        }
    }
    projected
}

fn description_preview_section(record: &AtlasRecord) -> Option<RecordSectionJson> {
    let description = record.content.description()?;
    let preview = truncate_words(&render_plain_text(description), DESCRIPTION_PREVIEW_WORDS)?;
    Some(RecordSectionJson {
        kind: PresentationSectionKind::DescriptionPreview.as_str(),
        title: PresentationSectionKind::DescriptionPreview
            .default_title()
            .to_string(),
        blocks: vec![RecordBlockJson::Prose { text: preview }],
    })
}

fn truncate_words(text: &str, max_words: usize) -> Option<String> {
    let mut words = text.split_whitespace();
    let mut preview = Vec::new();
    for _ in 0..max_words {
        if let Some(word) = words.next() {
            preview.push(word);
        } else {
            break;
        }
    }
    if preview.is_empty() {
        return None;
    }
    let mut output = preview.join(" ");
    if words.next().is_some() {
        output.push_str("...");
    }
    Some(output)
}

fn supplementary_sections(sections: &[RecordSectionJson]) -> Vec<RecordSectionJson> {
    sections
        .iter()
        .filter_map(|section| section_with_blocks(section, false))
        .collect()
}

fn generic_sections(sections: &[RecordSectionJson]) -> Vec<RecordSectionJson> {
    sections
        .iter()
        .filter_map(|section| section_with_blocks(section, true))
        .collect()
}

fn section_with_blocks(section: &RecordSectionJson, facts: bool) -> Option<RecordSectionJson> {
    let blocks = section
        .blocks
        .iter()
        .filter(|block| matches!(block, RecordBlockJson::FactList { .. }) == facts)
        .cloned()
        .collect::<Vec<_>>();
    (!blocks.is_empty()).then(|| RecordSectionJson {
        kind: section.kind,
        title: section.title.clone(),
        blocks,
    })
}

fn section_json(section: &PresentationSection) -> Option<RecordSectionJson> {
    let blocks = section
        .blocks
        .iter()
        .filter_map(block_json)
        .collect::<Vec<_>>();
    (!blocks.is_empty()).then(|| RecordSectionJson {
        kind: section.kind.as_str(),
        title: section.title.clone(),
        blocks,
    })
}

fn block_json(block: &PresentationBlock) -> Option<RecordBlockJson> {
    match block {
        PresentationBlock::FactList(facts) => {
            let facts = facts.iter().map(fact_json).collect::<Vec<_>>();
            (!facts.is_empty()).then_some(RecordBlockJson::FactList { facts })
        }
        PresentationBlock::Prose(text) => {
            (!text.text.trim().is_empty()).then(|| RecordBlockJson::Prose {
                text: text.text.clone(),
            })
        }
        PresentationBlock::Content(content) => {
            (!content.is_empty()).then_some(RecordBlockJson::Content {
                content: content.clone(),
            })
        }
        PresentationBlock::Relationships(relationships) => {
            let relationships = relationships
                .iter()
                .map(relationship_json)
                .collect::<Vec<_>>();
            (!relationships.is_empty()).then_some(RecordBlockJson::Relationships { relationships })
        }
    }
}

fn fact_json(fact: &PresentationFact) -> RecordFactJson {
    RecordFactJson {
        key: fact.key.clone(),
        label: fact.label.clone(),
        value: fact.value.clone(),
    }
}

fn relationship_json(relationship: &PresentationRelationship) -> RecordRelationshipJson {
    RecordRelationshipJson {
        kind: match relationship.kind {
            PresentationRelationshipKind::Reference => "reference",
            PresentationRelationshipKind::Backlink => "backlink",
        },
        label: relationship.label.clone(),
        record_key: relationship.record_key.as_ref().map(ToString::to_string),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use atlas_domain::{MetricDomain, PackName, PublicationCategory, RecordId, RecordKey};

    use super::*;
    use crate::{
        ActivityRoll, ActivityRollSurface, ActorMechanics, ContentSourceKind, DamageEffectKind,
        DamageExpression, FoundryDocumentMechanics, FoundryDocumentType, FoundryRecordInfo,
        FoundryRecordType, MechanicActivity, MechanicActivityKind, MechanicActivityUsage,
        MetricRow, MetricValue, RecordClassification, RecordContent, RecordContentDocument,
        RecordIdentity, RecordMechanics, RecordProvenance, RecordPublication, RecordRequirements,
        RecordTaxonomy, RecordTiming, RecordVisibility, RichDocument, RichNode,
        SpellcastingEntryMechanics, SpellcastingPreparation,
    };

    #[test]
    fn non_creature_is_registry_bound_and_raw_is_an_independent_opt_in() {
        let record = fixture_record();
        let without_raw = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        );
        let with_raw = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: true,
            },
        );

        assert!(without_raw.source_json.is_none());
        assert!(with_raw.source_json.is_some());
        assert!(matches!(
            without_raw.presentation,
            RecordPresentationJson::Unmigrated {
                migration: UnmigratedRegistryJson { plan_id: "H7", .. },
                ..
            }
        ));
    }

    #[test]
    fn creature_exposes_direct_typed_scan_fields_without_generic_mechanics() {
        let json = record_json(
            &fixture_creature_record(),
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        );
        let RecordPresentationJson::Creature {
            defenses,
            perception,
            languages,
            movement,
            ..
        } = &json.presentation
        else {
            panic!("creature presentation")
        };

        let defenses = defenses.as_ref().expect("standard defenses");
        assert_eq!(defenses.ac.as_ref().expect("ac").value, 25);
        assert_eq!(defenses.hp.as_ref().expect("hp").maximum, Some(80));
        assert_eq!(
            defenses.saves.fortitude.as_ref().expect("fortitude").value,
            14
        );
        assert_eq!(perception.as_ref().expect("perception").modifier, Some(12));
        assert_eq!(
            languages.as_deref(),
            Some(["Common".to_string()].as_slice())
        );
        assert_eq!(
            movement.as_ref().expect("standard movement").modes[0].value_feet,
            25
        );
        assert!(json.generic_sections().is_empty());
        assert!(json.supplementary_sections.iter().all(|section| {
            section
                .blocks
                .iter()
                .all(|block| !matches!(block, RecordBlockJson::FactList { .. }))
        }));
    }

    #[test]
    fn creature_detail_levels_keep_the_tagged_schema_with_purposeful_hydration() {
        let record = fixture_creature_record();
        let summary = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Summary,
                include_source_json: false,
            },
        );
        let preview = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        );
        let description = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Description,
                include_source_json: false,
            },
        );
        let standard = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        );

        let summary_value = serde_json::to_value(&summary).expect("summary json");
        let description_value = serde_json::to_value(&description).expect("description json");
        for field in [
            "defenses",
            "perception",
            "languages",
            "skills",
            "movement",
            "resources",
            "strikes",
            "actions",
            "spellcasting",
        ] {
            assert!(
                summary_value.get(field).is_none(),
                "summary omitted {field}"
            );
            assert!(
                description_value.get(field).is_none(),
                "description omitted {field}"
            );
        }
        let RecordPresentationJson::Creature { defenses, .. } = preview.presentation else {
            panic!("preview creature")
        };
        assert_eq!(
            defenses
                .expect("preview defenses")
                .ac
                .expect("preview ac")
                .value,
            25
        );
        let preview_value = serde_json::to_value(record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        ))
        .expect("preview json");
        let preview_strike = &preview_value["strikes"][0];
        assert!(preview_strike.get("rolls").is_none());
        assert!(preview_strike.get("damage").is_none());
        assert!(preview_strike.get("modes").is_none());
        let standard_value = serde_json::to_value(standard).expect("standard json");
        let standard_strike = &standard_value["strikes"][0];
        assert!(standard_strike["rolls"].is_array());
        assert!(standard_strike["damage"].is_array());
        assert!(standard_strike["modes"].is_array());
        assert!(standard_value["actions"][0]["rolls"].is_array());
        assert_eq!(standard_value["skills"][0]["slug"], "theater_lore");
        assert_eq!(standard_value["spellcasting"]["entries"][0]["order"], 0);
        assert_eq!(standard_value["spellcasting"]["spells"][0]["order"], 2);
        assert!(
            description
                .supplementary_sections
                .iter()
                .any(|section| section.kind == "description")
        );
    }

    fn fixture_record() -> AtlasRecord {
        AtlasRecord {
            identity: RecordIdentity {
                key: RecordKey::new(
                    PackName::new("actions".to_string()).expect("pack"),
                    RecordId::new("treat-wounds".to_string()).expect("id"),
                ),
                name: "Treat Wounds".to_string(),
            },
            classification: RecordClassification {
                kind: RecordKind::Rule,
                level: None,
                rarity: None,
                traits: vec!["healing".to_string()],
                taxonomy: RecordTaxonomy::default(),
            },
            foundry: FoundryRecordInfo {
                pack_label: "Actions".to_string(),
                document_type: FoundryDocumentType::Item,
                record_type: FoundryRecordType::Action,
                folder_id: None,
            },
            provenance: RecordProvenance {
                source_path: "packs/actions/treat-wounds.json".to_string(),
                raw_json: Some("{\"name\":\"Treat Wounds\"}".to_string()),
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
                documents: vec![RecordContentDocument {
                    source_kind: ContentSourceKind::Description,
                    label: None,
                    document: RichDocument::new(vec![RichNode::HtmlElement {
                        tag: "p".to_string(),
                        attributes: BTreeMap::new(),
                        children: vec![RichNode::Text {
                            text: "You spend 10 minutes treating one injured creature.".to_string(),
                        }],
                    }]),
                }],
            },
            variant: None,
            visibility: RecordVisibility::default(),
        }
    }

    fn fixture_creature_record() -> AtlasRecord {
        let mut record = fixture_record();
        record.identity.key = RecordKey::new(
            PackName::new("creatures".to_string()).expect("pack"),
            RecordId::new("test-guardian".to_string()).expect("id"),
        );
        record.identity.name = "Test Guardian".to_string();
        record.classification.kind = RecordKind::Creature;
        record.classification.level = Some(5);
        record.foundry.document_type = FoundryDocumentType::Actor;
        record.foundry.record_type = FoundryRecordType::Npc;
        record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics {
            size: Some("med".to_string()),
            languages: vec!["Common".to_string()],
            speed_types: vec!["land".to_string()],
            senses: vec!["Darkvision".to_string()],
            immunities: vec!["sleep".to_string()],
            resistances: vec!["mental".to_string()],
            weaknesses: vec!["cold-iron".to_string()],
            ..ActorMechanics::default()
        });
        record.mechanics.metrics = vec![
            metric("perception.mod", 12.0),
            metric("ac.value", 25.0),
            metric("hp.max", 80.0),
            metric("save.fort.mod", 14.0),
            metric("save.ref.mod", 11.0),
            metric("save.will.mod", 12.0),
            metric("speed.land.value", 25.0),
            metric("skill.theater_lore.mod", 15.0),
        ];
        record.mechanics.spellcasting_entries = vec![SpellcastingEntryMechanics {
            entry_id: "occult-innate".to_string(),
            label: "Occult Innate Spells".to_string(),
            preparation: SpellcastingPreparation::Innate,
            spell_attack: Some(16),
            spell_dc: Some(26),
        }];
        record.mechanics.activities = vec![
            MechanicActivity {
                activity_id: "jaws".to_string(),
                label: "Jaws".to_string(),
                kind: MechanicActivityKind::Strike,
                traits: vec!["magical".to_string()],
                compendium_source: None,
                usage: MechanicActivityUsage::Unlimited,
                rolls: vec![ActivityRoll {
                    roll_id: "attack".to_string(),
                    label: "Attack".to_string(),
                    base_value: 17,
                    surface: ActivityRollSurface::AttackRoll,
                    ability: None,
                }],
                damage: vec![DamageExpression {
                    damage_id: "piercing".to_string(),
                    label: None,
                    formula: "2d8+8".to_string(),
                    damage_type: Some("piercing".to_string()),
                    effect_kind: DamageEffectKind::Damage,
                    ability: None,
                }],
                modes: Vec::new(),
            },
            MechanicActivity {
                activity_id: "change-shape".to_string(),
                label: "Change Shape".to_string(),
                kind: MechanicActivityKind::Other,
                traits: vec!["polymorph".to_string()],
                compendium_source: None,
                usage: MechanicActivityUsage::Unlimited,
                rolls: Vec::new(),
                damage: Vec::new(),
                modes: Vec::new(),
            },
            MechanicActivity {
                activity_id: "magic-missile".to_string(),
                label: "Magic Missile".to_string(),
                kind: MechanicActivityKind::Spell,
                traits: vec!["force".to_string()],
                compendium_source: Some("Compendium.pf2e.spells.Item.magic".to_string()),
                usage: MechanicActivityUsage::Unlimited,
                rolls: Vec::new(),
                damage: vec![DamageExpression {
                    damage_id: "force".to_string(),
                    label: None,
                    formula: "1d4+1".to_string(),
                    damage_type: Some("force".to_string()),
                    effect_kind: DamageEffectKind::Damage,
                    ability: None,
                }],
                modes: Vec::new(),
            },
        ];
        record
    }

    fn metric(key: &str, value: f64) -> MetricRow {
        MetricRow {
            domain: MetricDomain::Actor,
            key: key.to_string(),
            value: MetricValue::Number(value),
        }
    }
}
