mod creature;

use std::ops::Deref;

use atlas_domain::{DetailLevel, RecordKind};
use serde::Serialize;

pub use creature::{
    CreatureActionCostJson, CreatureActionJson, CreatureArmorClassJson, CreatureDamageJson,
    CreatureDefensesJson, CreatureFrequencyJson, CreatureHitPointsJson, CreatureIwrJson,
    CreatureMovementJson, CreatureMovementModeJson, CreatureOccurrenceContextJson,
    CreaturePreparedSpellJson, CreatureResourceJson, CreatureRollJson, CreatureSaveJson,
    CreatureSavesJson, CreatureSenseJson, CreatureSkillJson, CreatureSkillVariantJson,
    CreatureSpellJson, CreatureSpellSlotJson, CreatureSpellcastingEntryJson,
    CreatureSpellcastingJson, CreatureStrikeJson, CreatureUseLimitJson,
};

use crate::{
    AtlasRecord, PresentationBlock, PresentationContent, PresentationFact,
    PresentationRelationship, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, RecordBody, RetrievedRecord, build_record_presentation_document,
    render_plain_text,
};

const DESCRIPTION_PREVIEW_WORDS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordJsonOptions {
    pub detail: DetailLevel,
    pub include_source_json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordJsonError {
    MissingCreatureBody { record_key: String },
    UnexpectedCreatureBody { record_key: String },
}

impl std::fmt::Display for RecordJsonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCreatureBody { record_key } => write!(
                formatter,
                "retrieved creature record `{record_key}` is missing its canonical creature body"
            ),
            Self::UnexpectedCreatureBody { record_key } => write!(
                formatter,
                "retrieved non-creature record `{record_key}` has an unexpected canonical creature body"
            ),
        }
    }
}

impl std::error::Error for RecordJsonError {}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_vision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub senses: Option<Vec<CreatureSenseJson>>,
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

pub fn record_json(
    retrieved: &RetrievedRecord,
    options: RecordJsonOptions,
) -> Result<RecordJson, RecordJsonError> {
    let record = &retrieved.record;
    let document = build_record_presentation_document(record);
    let detailed_sections = sections_for_detail(record, &document.sections, options.detail);
    let presentation = match (record.classification.kind, &retrieved.body) {
        (RecordKind::Creature, Some(RecordBody::Creature(creature))) => {
            creature::creature_presentation(creature, options.detail)
        }
        (RecordKind::Creature, None) => {
            return Err(RecordJsonError::MissingCreatureBody {
                record_key: record.identity.key.to_string(),
            });
        }
        (_, Some(RecordBody::Creature(_))) => {
            return Err(RecordJsonError::UnexpectedCreatureBody {
                record_key: record.identity.key.to_string(),
            });
        }
        (_, None) => RecordPresentationJson::Unmigrated {
            migration: unmigrated_registry(record),
            sections: generic_sections(&detailed_sections),
        },
    };

    Ok(RecordJson {
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
    })
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

    use atlas_domain::{PackName, PublicationCategory, RecordId, RecordKey};

    use super::*;
    use crate::{
        ContentSourceKind, FactValue, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordContent, RecordContentDocument, RecordIdentity,
        RecordMechanics, RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy,
        RecordTiming, RecordVisibility, RichDocument, RichNode,
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
        )
        .expect("non-creature projection");
        let with_raw = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: true,
            },
        )
        .expect("non-creature projection with raw source");

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
    fn record_json_fails_closed_on_carrier_body_mismatches() {
        let mut missing = fixture_creature_record();
        missing.body = None;
        assert!(matches!(
            record_json(
                &missing,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                }
            ),
            Err(RecordJsonError::MissingCreatureBody { .. })
        ));

        let mut unexpected = fixture_record();
        unexpected.body = fixture_creature_record().body;
        assert!(matches!(
            record_json(
                &unexpected,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                }
            ),
            Err(RecordJsonError::UnexpectedCreatureBody { .. })
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
        )
        .expect("creature projection");
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
        assert_eq!(defenses.ac.as_ref().expect("ac").value, Some(25));
        assert_eq!(defenses.hp.as_ref().expect("hp").maximum, Some(80));
        assert_eq!(
            defenses
                .saves
                .as_ref()
                .expect("saves")
                .fortitude
                .as_ref()
                .expect("fortitude")
                .value,
            Some(14)
        );
        assert_eq!(perception.as_ref().expect("perception").modifier, Some(12));
        assert_eq!(
            languages.as_deref(),
            Some(["common".to_string()].as_slice())
        );
        assert_eq!(
            movement.as_ref().expect("standard movement").modes[0].value_feet,
            Some(25)
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
        )
        .expect("summary projection");
        let preview = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        )
        .expect("preview projection");
        let description = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Description,
                include_source_json: false,
            },
        )
        .expect("description projection");
        let standard = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        )
        .expect("standard projection");

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
            Some(25)
        );
        let preview_value = serde_json::to_value(
            record_json(
                &record,
                RecordJsonOptions {
                    detail: DetailLevel::Preview,
                    include_source_json: false,
                },
            )
            .expect("preview projection"),
        )
        .expect("preview json");
        let preview_strike = &preview_value["strikes"][0];
        assert!(preview_strike.get("rolls").is_none());
        assert!(preview_strike.get("damage").is_none());
        assert!(preview_strike.get("modes").is_none());
        let standard_value = serde_json::to_value(standard).expect("standard json");
        let standard_strike = &standard_value["strikes"][0];
        assert!(standard_strike["rolls"].is_array());
        assert!(standard_strike["damage"].is_array());
        assert!(standard_value["actions"][0]["rolls"].is_array());
        assert_eq!(standard_value["skills"][0]["slug"], "lore");
        assert_eq!(standard_value["skills"][0]["note"], "stage performances");
        assert_eq!(standard_value["resources"][0]["maximum"], 1);
        assert_eq!(standard_value["resources"][0]["serialized_value"], 1);
        assert_eq!(standard_value["defenses"]["resistances"][0]["value"], 10);
        assert_eq!(standard_value["defenses"]["weaknesses"][0]["value"], 10);
        assert_eq!(
            standard_value["actions"][0]["action_cost"]["kind"],
            "actions"
        );
        assert_eq!(standard_value["actions"][0]["action_cost"]["actions"], 1);
        assert_eq!(standard_value["spellcasting"]["entries"][0]["order"], 0);
        assert_eq!(
            standard_value["spellcasting"]["entries"][0]["slots"][0]["maximum"],
            1
        );
        assert_eq!(standard_value["spellcasting"]["spells"][0]["order"], 2);
        assert_eq!(
            standard_value["spellcasting"]["spells"][0]["context"]["rank"],
            1
        );
        assert_eq!(
            standard_value["spellcasting"]["spells"][0]["context"]["uses"]["maximum"],
            1
        );
        assert_eq!(
            standard_value["spellcasting"]["spells"][0]["parent_entry_id"],
            "occult-innate"
        );
        assert!(
            description
                .supplementary_sections
                .iter()
                .any(|section| section.kind == "description")
        );
    }

    fn fixture_record() -> RetrievedRecord {
        RetrievedRecord {
            record: fixture_base_record(),
            body: None,
        }
    }

    fn fixture_base_record() -> AtlasRecord {
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

    fn fixture_creature_record() -> RetrievedRecord {
        let mut record = fixture_base_record();
        record.identity.key = RecordKey::new(
            PackName::new("creatures".to_string()).expect("pack"),
            RecordId::new("test-guardian".to_string()).expect("id"),
        );
        record.identity.name = "Test Guardian".to_string();
        record.classification.kind = RecordKind::Creature;
        record.classification.level = Some(5);
        record.foundry.document_type = FoundryDocumentType::Actor;
        record.foundry.record_type = FoundryRecordType::Npc;
        let owner = record.identity.key.clone();
        let entry_id = crate::CreatureOccurrenceId::new("occult-innate").expect("entry id");
        let occurrence = |id: &str,
                          order: u32,
                          family: crate::CreatureEntityFamily,
                          label: &str,
                          parent: crate::CreatureOccurrenceParent,
                          context: crate::CreatureOccurrenceContext,
                          capability: crate::CreatureCapability| {
            crate::CreatureEntityOccurrence {
                id: crate::CreatureOccurrenceId::new(id).expect("occurrence id"),
                identity_stability: crate::OccurrenceIdentityStability::StableNestedSourceId,
                owner: owner.clone(),
                target: crate::CreatureEntityTarget::ActorOwned(
                    crate::CreatureEntityId::new(id).expect("entity id"),
                ),
                family,
                authored_order: order,
                source_sort: FactValue::Value(order.into()),
                source_folder: FactValue::Missing,
                source_identity: crate::CreatureEntitySourceIdentity {
                    nested_source_id: FactValue::Missing,
                    stable_source_locator: FactValue::Missing,
                    source_locators: Vec::new(),
                },
                parent,
                context: crate::CreatureOccurrenceContext {
                    contextual_label: FactValue::Value(label.to_string()),
                    ..context
                },
                capability,
                deltas: Vec::new(),
            }
        };
        let occurrences = vec![
            occurrence(
                "occult-innate",
                0,
                crate::CreatureEntityFamily::SpellcastingEntry,
                "Occult Innate Spells",
                crate::CreatureOccurrenceParent::Creature,
                crate::CreatureOccurrenceContext::default(),
                crate::CreatureCapability::SpellcastingEntry(
                    crate::CreatureSpellcastingEntryCapability {
                        preparation: FactValue::Value(crate::CreatureSpellPreparation::Innate),
                        tradition: FactValue::Value("occult".to_string()),
                        attack: FactValue::Value(16),
                        dc: FactValue::Value(26),
                        slots: FactValue::Value(vec![crate::CreatureSpellSlot {
                            rank: 1,
                            maximum: FactValue::Value(crate::CreatureSourceScalar::Value(1)),
                            serialized_value: FactValue::Value(crate::CreatureSourceScalar::Value(
                                1,
                            )),
                            prepared: FactValue::Value(Vec::new()),
                        }]),
                        unsupported_notes: Vec::new(),
                    },
                ),
            ),
            occurrence(
                "jaws",
                1,
                crate::CreatureEntityFamily::Strike,
                "Jaws",
                crate::CreatureOccurrenceParent::Creature,
                crate::CreatureOccurrenceContext::default(),
                crate::CreatureCapability::Strike(crate::CreatureStrikeCapability {
                    traits: FactValue::Value(vec!["magical".to_string()]),
                    attack_effects: FactValue::Value(Vec::new()),
                    rolls: vec![crate::CreatureRoll {
                        id: "attack".to_string(),
                        label: "Attack".to_string(),
                        kind: crate::CreatureRollKind::Attack,
                        value: FactValue::Value(17),
                        ability: FactValue::Missing,
                    }],
                    damage: FactValue::Value(vec![crate::CreatureDamage {
                        id: "piercing".to_string(),
                        formula: FactValue::Value("2d8+8".to_string()),
                        damage_type: FactValue::Value("piercing".to_string()),
                        category: FactValue::Missing,
                        kinds: FactValue::Value(vec![crate::CreatureDamageKind::Damage]),
                        apply_modifier: FactValue::Missing,
                    }]),
                    action_cost: crate::CreatureActionCost::Actions(1),
                    unsupported_notes: Vec::new(),
                }),
            ),
            occurrence(
                "magic-missile",
                2,
                crate::CreatureEntityFamily::Spell,
                "Magic Missile",
                crate::CreatureOccurrenceParent::SpellcastingEntry(entry_id),
                crate::CreatureOccurrenceContext {
                    rank: FactValue::Value(1),
                    uses: FactValue::Value(crate::CreatureUseLimit {
                        maximum: FactValue::Value(1),
                        serialized_value: FactValue::Value(1),
                    }),
                    ..crate::CreatureOccurrenceContext::default()
                },
                crate::CreatureCapability::Spell(crate::CreatureSpellCapability {
                    traits: FactValue::Value(vec!["force".to_string()]),
                    base_rank: FactValue::Value(1),
                    signature: FactValue::Value(false),
                    traditions: FactValue::Value(vec!["occult".to_string()]),
                    requirements: FactValue::Missing,
                    cost: FactValue::Missing,
                    counteraction: FactValue::Missing,
                    ritual: FactValue::Missing,
                    target: FactValue::Value("one creature".to_string()),
                    area: FactValue::Missing,
                    range: FactValue::Value("120 feet".to_string()),
                    time: FactValue::Missing,
                    duration: FactValue::Missing,
                    defense: FactValue::Missing,
                    damage: FactValue::Value(Vec::new()),
                    action_cost: crate::CreatureActionCost::Actions(1),
                    unsupported_notes: Vec::new(),
                }),
            ),
            occurrence(
                "change-shape",
                3,
                crate::CreatureEntityFamily::Action,
                "Change Shape",
                crate::CreatureOccurrenceParent::Creature,
                crate::CreatureOccurrenceContext::default(),
                crate::CreatureCapability::Action(crate::CreatureActionCapability {
                    category: FactValue::Value("offensive".to_string()),
                    traits: FactValue::Value(vec!["polymorph".to_string()]),
                    action_cost: crate::CreatureActionCost::Actions(1),
                    frequency: FactValue::Missing,
                    self_effect: FactValue::Missing,
                    self_effect_label: FactValue::Missing,
                    requirements: FactValue::Missing,
                    cost: FactValue::Missing,
                    rolls: Vec::new(),
                    damage: FactValue::Value(Vec::new()),
                    unsupported_notes: Vec::new(),
                }),
            ),
        ];
        let entities = ["occult-innate", "jaws", "magic-missile", "change-shape"]
            .into_iter()
            .map(|id| crate::CreatureEntity {
                id: crate::CreatureEntityId::new(id).expect("entity id"),
                family: match id {
                    "occult-innate" => crate::CreatureEntityFamily::SpellcastingEntry,
                    "jaws" => crate::CreatureEntityFamily::Strike,
                    "magic-missile" => crate::CreatureEntityFamily::Spell,
                    _ => crate::CreatureEntityFamily::Action,
                },
                label: id.to_string(),
                source_identity: crate::CreatureEntitySourceIdentity {
                    nested_source_id: FactValue::Missing,
                    stable_source_locator: FactValue::Missing,
                    source_locators: Vec::new(),
                },
            })
            .collect();
        let body = crate::CreatureRecord {
            identity: crate::CreatureIdentity {
                record_key: owner,
                source_id: crate::CreatureSourceId::new("test-guardian").expect("source id"),
                name: "Test Guardian".to_string(),
                family: crate::CreatureFamily::Npc,
            },
            level: missing(crate::CreatureSourceField::Level),
            rarity: missing(crate::CreatureSourceField::Rarity),
            traits: missing(crate::CreatureSourceField::Traits),
            size: missing(crate::CreatureSourceField::Size),
            publication: missing(crate::CreatureSourceField::Publication),
            adjustment: missing(crate::CreatureSourceField::Adjustment),
            source_alliance: missing(crate::CreatureSourceField::SourceAlliance),
            perception: crate::CreatureFact::source(
                FactValue::Value(crate::CreaturePerception {
                    modifier: FactValue::Value(12),
                    details: FactValue::Missing,
                    has_vision: FactValue::Value(true),
                    senses: FactValue::Value(Vec::new()),
                }),
                crate::CreatureSourceField::Perception,
            ),
            initiative: missing(crate::CreatureSourceField::Initiative),
            languages: crate::CreatureFact::source(
                FactValue::Value(crate::CreatureLanguages {
                    values: FactValue::Value(vec![crate::Language::new("common").expect("lang")]),
                    details: FactValue::Missing,
                }),
                crate::CreatureSourceField::Languages,
            ),
            skills: crate::CreatureFact::source(
                FactValue::Value(vec![crate::CreatureSkill {
                    id: crate::CreatureComponentId::new("theater-lore").expect("skill id"),
                    authored_order: 0,
                    kind: crate::CreatureSkillKind::Lore,
                    label: "Theater Lore".to_string(),
                    modifier: FactValue::Value(15),
                    note: FactValue::Value(crate::CreatureNote::new("stage performances")),
                    variants: FactValue::Value(Vec::new()),
                    source_item_id: FactValue::Missing,
                }]),
                crate::CreatureSourceField::Skills,
            ),
            legacy_abilities: missing(crate::CreatureSourceField::LegacyAbilities),
            defenses: crate::CreatureFact::source(
                FactValue::Value(fixture_defenses()),
                crate::CreatureSourceField::Defenses,
            ),
            movement: crate::CreatureFact::source(
                FactValue::Value(vec![crate::CreatureSpeed {
                    id: crate::CreatureComponentId::new("land").expect("speed id"),
                    authored_order: 0,
                    mode: crate::CreatureMovementMode::Land,
                    value: FactValue::Value(25),
                    label: FactValue::Value("Land Speed".to_string()),
                    details: FactValue::Missing,
                }]),
                crate::CreatureSourceField::Movement,
            ),
            resources: crate::CreatureFact::source(
                FactValue::Value(vec![crate::CreatureResource {
                    id: crate::CreatureComponentId::new("focus").expect("resource id"),
                    authored_order: 0,
                    kind: crate::CreatureResourceKind::new("focus").expect("resource kind"),
                    label: "Focus Points".to_string(),
                    maximum: FactValue::Value(crate::CreatureResourceAmount::Integer(1)),
                    serialized_value: FactValue::Value(crate::CreatureResourceAmount::Integer(1)),
                    source_drift: FactValue::Value(Vec::new()),
                    current_policy: crate::ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
                }]),
                crate::CreatureSourceField::Resources,
            ),
            embedded_entities: crate::CreatureFact::source(
                FactValue::Value(crate::CreatureEmbeddedEntities {
                    entities,
                    occurrences,
                    relationships: Vec::new(),
                    actor_spellcasting: FactValue::Missing,
                }),
                crate::CreatureSourceField::EmbeddedEntities,
            ),
            content: crate::OwnedRichContent::default(),
            provenance: crate::CreatureProvenance {
                source_path: "packs/creatures/test-guardian.json".to_string(),
                source_contract_version: "test".to_string(),
                source_system_version: "test".to_string(),
                source_upstream_commit: "test".to_string(),
            },
        };
        RetrievedRecord {
            record,
            body: Some(RecordBody::Creature(body)),
        }
    }

    fn fixture_defenses() -> crate::CreatureDefenses {
        let save = |id: &str, kind, value| crate::CreatureSave {
            id: crate::CreatureComponentId::new(id).expect("save id"),
            kind,
            value: FactValue::Value(value),
            details: FactValue::Missing,
        };
        let iwr = |id: &str, kind, iwr_type: &str, value| crate::CreatureIwr {
            id: crate::CreatureComponentId::new(id).expect("iwr id"),
            authored_order: 0,
            kind,
            iwr_type: crate::IwrType::new(iwr_type).expect("iwr type"),
            value: FactValue::Value(value),
            exceptions: FactValue::Value(Vec::new()),
            double_vs: FactValue::Value(Vec::new()),
            apply_once: FactValue::Missing,
        };
        crate::CreatureDefenses {
            armor_class: FactValue::Value(crate::CreatureArmorClass {
                value: FactValue::Value(25),
                details: FactValue::Missing,
            }),
            hit_points: FactValue::Value(crate::CreatureHitPoints {
                value: FactValue::Value(crate::CreatureNumber::Integer(80)),
                maximum: FactValue::Value(80),
                temporary: FactValue::Missing,
                temporary_maximum: FactValue::Missing,
                details: FactValue::Missing,
            }),
            hardness: FactValue::Missing,
            shield: FactValue::Missing,
            saves: FactValue::Value(crate::CreatureSaves {
                fortitude: FactValue::Value(save(
                    "fortitude",
                    crate::CreatureSaveKind::Fortitude,
                    14,
                )),
                reflex: FactValue::Value(save("reflex", crate::CreatureSaveKind::Reflex, 11)),
                will: FactValue::Value(save("will", crate::CreatureSaveKind::Will, 12)),
            }),
            all_saves_note: FactValue::Missing,
            immunities: FactValue::Value(Vec::new()),
            resistances: FactValue::Value(vec![iwr(
                "mental",
                crate::CreatureIwrKind::Resistance,
                "mental",
                10,
            )]),
            weaknesses: FactValue::Value(vec![iwr(
                "cold-iron",
                crate::CreatureIwrKind::Weakness,
                "cold-iron",
                10,
            )]),
        }
    }

    fn missing<T>(field: crate::CreatureSourceField) -> crate::CreatureFact<T> {
        crate::CreatureFact::source(FactValue::Missing, field)
    }
}
