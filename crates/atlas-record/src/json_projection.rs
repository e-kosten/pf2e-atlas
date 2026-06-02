use atlas_domain::DetailLevel;
use serde::Serialize;

use crate::{
    AtlasRecord, PresentationBlock, PresentationFact, PresentationRelationship,
    PresentationRelationshipKind, PresentationSection, PresentationSectionKind,
    build_record_presentation_document, render_markdown_like, render_plain_text,
};

const DESCRIPTION_PREVIEW_WORDS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordJsonOptions {
    pub detail: DetailLevel,
    pub include_source_json: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecordJson {
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
    pub sections: Vec<RecordSectionJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_json: Option<String>,
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
        format: &'static str,
        text: String,
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
    let presentation = build_record_presentation_document(record);
    let mut sections = sections_for_detail(record, &presentation.sections, options.detail);
    sections.retain(|section| !section.blocks.is_empty());
    RecordJson {
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
        sections,
        source_json: options
            .include_source_json
            .then(|| record.provenance.raw_json.clone())
            .flatten(),
    }
}

fn source_json(record: &AtlasRecord, detail: DetailLevel) -> Option<RecordSourceJson> {
    let full = detail == DetailLevel::Full;
    let source = RecordSourceJson {
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
    };

    if source.publication_title.is_none()
        && source.pack.is_none()
        && source.category.is_none()
        && source.publication_remaster.is_none()
        && source.source_path.is_none()
        && source.foundry.is_none()
    {
        None
    } else {
        Some(source)
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

fn section_json(section: &PresentationSection) -> Option<RecordSectionJson> {
    let blocks: Vec<_> = section.blocks.iter().filter_map(block_json).collect();
    (!blocks.is_empty()).then(|| RecordSectionJson {
        kind: section.kind.as_str(),
        title: section.title.clone(),
        blocks,
    })
}

fn block_json(block: &PresentationBlock) -> Option<RecordBlockJson> {
    match block {
        PresentationBlock::FactList(facts) => {
            let facts: Vec<_> = facts.iter().map(fact_json).collect();
            (!facts.is_empty()).then_some(RecordBlockJson::FactList { facts })
        }
        PresentationBlock::Prose(text) => {
            (!text.text.trim().is_empty()).then(|| RecordBlockJson::Prose {
                text: text.text.clone(),
            })
        }
        PresentationBlock::Content(document) => {
            let text = render_markdown_like(document);
            (!text.trim().is_empty()).then_some(RecordBlockJson::Content {
                format: "markdown",
                text,
            })
        }
        PresentationBlock::Relationships(relationships) => {
            let relationships: Vec<_> = relationships.iter().map(relationship_json).collect();
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
    use atlas_domain::{
        MetricDomain, PackName, PublicationCategory, RecordId, RecordKey, RecordKind, TimeKind,
        TimeUnit,
    };

    use super::*;
    use crate::{
        ActivationTimeSourceField, ActorMechanics, ContentBlock, ContentDocument,
        ContentSourceKind, DurationTimeSourceField, FoundryDocumentMechanics, FoundryDocumentType,
        FoundryRecordInfo, FoundryRecordType, MetricRow, MetricValue, RecordActivationTiming,
        RecordClassification, RecordContent, RecordContentDocument, RecordDurationTiming,
        RecordIdentity, RecordMechanics, RecordProvenance, RecordPublication, RecordRequirements,
        RecordTaxonomy, RecordTiming, RecordVisibility,
    };

    #[test]
    fn summary_record_json_uses_stable_identity_shape() {
        let record = fixture_record();
        let json = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Summary,
                include_source_json: false,
            },
        );

        assert_eq!(json.key, "actions:treat-wounds");
        assert_eq!(json.name, "Treat Wounds");
        assert_eq!(json.kind, "rule");
        assert_eq!(json.traits, vec!["exploration", "healing", "manipulate"]);
        assert!(json.source_json.is_none());
        assert_eq!(
            json.source.expect("source").pack.expect("pack").name,
            "actions"
        );
        assert_eq!(json.sections[0].kind, "summary");
        assert_eq!(json.sections.len(), 1);
    }

    #[test]
    fn preview_record_json_uses_truncated_description() {
        let record = fixture_record();
        let json = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        );

        assert_eq!(json.sections[0].kind, "summary");
        assert_eq!(json.sections[1].kind, "description_preview");
    }

    #[test]
    fn preview_record_json_includes_family_fact_sections_without_full_detail_sections() {
        let record = fixture_creature_record();
        let json = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        );
        let section_kinds = json
            .sections
            .iter()
            .map(|section| section.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            section_kinds,
            vec![
                "summary",
                "defense",
                "movement",
                "offense",
                "description_preview"
            ]
        );
        assert!(!section_kinds.contains(&"description"));
        assert!(!section_kinds.contains(&"details"));
        assert!(
            json.sections
                .iter()
                .find(|section| section.kind == "defense")
                .expect("defense section")
                .blocks
                .iter()
                .any(|block| match block {
                    RecordBlockJson::FactList { facts } => facts
                        .iter()
                        .any(|fact| fact.key == "ac" && fact.value == "25"),
                    _ => false,
                })
        );
    }

    #[test]
    fn description_record_json_uses_full_description_without_details() {
        let record = fixture_record();
        let json = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Description,
                include_source_json: false,
            },
        );

        assert_eq!(json.sections[0].kind, "summary");
        assert!(
            json.sections
                .iter()
                .any(|section| section.kind == "description")
        );
        assert!(
            !json
                .sections
                .iter()
                .any(|section| section.kind == "details")
        );
    }

    #[test]
    fn full_record_json_hydrates_source_and_raw_json_when_requested() {
        let record = fixture_record();
        let json = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Full,
                include_source_json: true,
            },
        );
        let source = json.source.expect("source");

        assert_eq!(source.category, Some("core"));
        assert_eq!(source.publication_remaster, Some(true));
        assert_eq!(
            source.foundry.expect("foundry").document_type,
            "Item".to_string()
        );
        assert_eq!(
            json.source_json,
            Some("{\"name\":\"Treat Wounds\"}".to_string())
        );
        assert!(
            json.sections
                .iter()
                .any(|section| section.kind == "description")
        );
        assert!(
            !json
                .sections
                .iter()
                .any(|section| section.kind == "description_preview")
        );
    }

    #[test]
    fn standard_record_json_uses_full_description_without_preview() {
        let record = fixture_record();
        let json = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        );

        assert!(
            json.sections
                .iter()
                .any(|section| section.kind == "description")
        );
        assert!(
            !json
                .sections
                .iter()
                .any(|section| section.kind == "description_preview")
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
                traits: vec![
                    "exploration".to_string(),
                    "healing".to_string(),
                    "manipulate".to_string(),
                ],
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
            timing: RecordTiming {
                activation: Some(RecordActivationTiming {
                    time: crate::NormalizedTime {
                        kind: TimeKind::Actions,
                        actions: Some(1),
                        duration_value: None,
                        duration_unit: None,
                        text: "1".to_string(),
                    },
                    source_field: ActivationTimeSourceField::ActionsValue,
                }),
                duration: Some(RecordDurationTiming {
                    time: crate::NormalizedTime {
                        kind: TimeKind::Duration,
                        actions: None,
                        duration_value: Some(10),
                        duration_unit: Some(TimeUnit::Minute),
                        text: "10 minutes".to_string(),
                    },
                    source_field: DurationTimeSourceField::DurationValue,
                }),
            },
            mechanics: RecordMechanics::default(),
            content: RecordContent {
                documents: vec![RecordContentDocument {
                    source_kind: ContentSourceKind::Description,
                    label: None,
                    document: ContentDocument {
                        blocks: vec![ContentBlock::Paragraph {
                            content: vec![crate::ContentInline::Text {
                                text: "You spend 10 minutes treating one injured living creature."
                                    .to_string(),
                            }],
                        }],
                    },
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
        record.foundry.pack_label = "Creatures".to_string();
        record.foundry.document_type = FoundryDocumentType::Actor;
        record.foundry.record_type = FoundryRecordType::Npc;
        record.classification.level = Some(5);
        record.classification.traits = vec!["human".to_string(), "humanoid".to_string()];
        record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics {
            size: Some("med".to_string()),
            languages: vec!["Common".to_string()],
            speed_types: vec!["land".to_string()],
            senses: vec!["Darkvision".to_string()],
            immunities: Vec::new(),
            resistances: Vec::new(),
            weaknesses: Vec::new(),
            disable_text: None,
            disable_skills: Vec::new(),
            is_complex: false,
        });
        record.mechanics.metrics = vec![
            metric("perception.mod", 12.0),
            metric("ac.value", 25.0),
            metric("hp.value", 80.0),
            metric("save.fort.mod", 14.0),
            metric("save.ref.mod", 11.0),
            metric("save.will.mod", 12.0),
            metric("speed.land.value", 25.0),
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
