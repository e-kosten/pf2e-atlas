use atlas_domain::{RecordKey, RecordKind};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RecordPresentationDocument {
    #[ts(type = "string")]
    pub record_key: RecordKey,
    #[ts(type = "string")]
    pub kind: RecordKind,
    pub title: String,
    pub identity: Vec<PresentationFact>,
    pub badges: Vec<PresentationBadge>,
    pub sections: Vec<PresentationSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum PresentationSectionKind {
    Summary,
    DescriptionPreview,
    Description,
    Defense,
    Movement,
    Offense,
    Routine,
    Details,
    References,
    Backlinks,
    Classification,
}

impl PresentationSectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::DescriptionPreview => "description_preview",
            Self::Description => "description",
            Self::Defense => "defense",
            Self::Movement => "movement",
            Self::Offense => "offense",
            Self::Routine => "routine",
            Self::Details => "details",
            Self::References => "references",
            Self::Backlinks => "backlinks",
            Self::Classification => "classification",
        }
    }

    pub const fn default_title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::DescriptionPreview => "Description Preview",
            Self::Description => "Description",
            Self::Defense => "Defense",
            Self::Movement => "Movement",
            Self::Offense => "Offense",
            Self::Routine => "Routine",
            Self::Details => "Details",
            Self::References => "References",
            Self::Backlinks => "Referenced By",
            Self::Classification => "Classification",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PresentationSection {
    pub kind: PresentationSectionKind,
    pub title: String,
    pub blocks: Vec<PresentationBlock>,
}

impl PresentationSection {
    pub fn new(kind: PresentationSectionKind, blocks: Vec<PresentationBlock>) -> Self {
        Self {
            kind,
            title: kind.default_title().to_string(),
            blocks,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", content = "content", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum PresentationBlock {
    FactList(Vec<PresentationFact>),
    Prose(PresentationText),
    Content(PresentationContent),
    Relationships(Vec<PresentationRelationship>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, TS)]
pub struct PresentationContent {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<PresentationContentBlock>,
}

impl PresentationContent {
    pub fn new(blocks: Vec<PresentationContentBlock>) -> Self {
        Self { blocks }
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum PresentationContentBlock {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        spans: Vec<PresentationInline>,
    },
    List {
        ordered: bool,
        items: Vec<PresentationListItem>,
    },
    Table {
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
        rows: Vec<PresentationTableRow>,
    },
    Rule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PresentationListItem {
    pub blocks: Vec<PresentationContentBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PresentationTableRow {
    pub cells: Vec<PresentationContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum PresentationInline {
    Text {
        text: String,
    },
    Strong {
        spans: Vec<PresentationInline>,
    },
    Emphasis {
        spans: Vec<PresentationInline>,
    },
    Code {
        text: String,
    },
    Reference {
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "string")]
        record_key: Option<RecordKey>,
        embedded: bool,
    },
    LineBreak,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PresentationFact {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PresentationText {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PresentationBadge {
    pub kind: PresentationBadgeKind,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum PresentationBadgeKind {
    Trait,
    Classification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PresentationRelationship {
    pub kind: PresentationRelationshipKind,
    pub label: String,
    #[ts(type = "string | null")]
    pub record_key: Option<RecordKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum PresentationRelationshipKind {
    Reference,
    Backlink,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_kinds_have_stable_ids_and_default_titles() {
        assert_eq!(PresentationSectionKind::Summary.as_str(), "summary");
        assert_eq!(
            PresentationSectionKind::DescriptionPreview.as_str(),
            "description_preview"
        );
        assert_eq!(PresentationSectionKind::Description.as_str(), "description");
        assert_eq!(PresentationSectionKind::Defense.as_str(), "defense");
        assert_eq!(PresentationSectionKind::Movement.as_str(), "movement");
        assert_eq!(PresentationSectionKind::Offense.as_str(), "offense");
        assert_eq!(PresentationSectionKind::Routine.as_str(), "routine");
        assert_eq!(PresentationSectionKind::Details.as_str(), "details");
        assert_eq!(PresentationSectionKind::References.as_str(), "references");
        assert_eq!(PresentationSectionKind::Backlinks.as_str(), "backlinks");
        assert_eq!(
            PresentationSectionKind::Classification.as_str(),
            "classification"
        );

        assert_eq!(
            PresentationSectionKind::Backlinks.default_title(),
            "Referenced By"
        );
    }

    #[test]
    fn document_preserves_sectioned_renderer_neutral_content() {
        let document = RecordPresentationDocument {
            record_key: RecordKey::parse("spells:Heal").expect("record key should parse"),
            kind: RecordKind::Spell,
            title: "Heal".to_string(),
            identity: vec![PresentationFact {
                key: "level".to_string(),
                label: "Rank".to_string(),
                value: "1".to_string(),
            }],
            badges: vec![PresentationBadge {
                kind: PresentationBadgeKind::Trait,
                label: "Trait".to_string(),
                value: "healing".to_string(),
            }],
            sections: vec![
                PresentationSection::new(
                    PresentationSectionKind::Summary,
                    vec![PresentationBlock::FactList(vec![PresentationFact {
                        key: "traditions".to_string(),
                        label: "Traditions".to_string(),
                        value: "divine, primal".to_string(),
                    }])],
                ),
                PresentationSection::new(
                    PresentationSectionKind::Description,
                    vec![PresentationBlock::Prose(PresentationText {
                        text: "A healing spell restores vitality.".to_string(),
                    })],
                ),
                PresentationSection::new(
                    PresentationSectionKind::References,
                    vec![PresentationBlock::Relationships(vec![
                        PresentationRelationship {
                            kind: PresentationRelationshipKind::Reference,
                            label: "vitality".to_string(),
                            record_key: Some(
                                RecordKey::parse("rules:Vitality")
                                    .expect("record key should parse"),
                            ),
                        },
                    ])],
                ),
            ],
        };

        assert_eq!(document.kind, RecordKind::Spell);
        assert_eq!(document.sections[0].kind, PresentationSectionKind::Summary);
        assert_eq!(document.sections[1].title, "Description");
        assert!(matches!(
            &document.sections[2].blocks[0],
            PresentationBlock::Relationships(relationships)
                if relationships[0].label == "vitality"
        ));
    }

    #[test]
    fn document_serializes_with_stable_presentation_contract() {
        let document = RecordPresentationDocument {
            record_key: RecordKey::parse("spells:Heal").expect("record key should parse"),
            kind: RecordKind::Spell,
            title: "Heal".to_string(),
            identity: vec![PresentationFact {
                key: "level".to_string(),
                label: "Rank".to_string(),
                value: "1".to_string(),
            }],
            badges: vec![PresentationBadge {
                kind: PresentationBadgeKind::Trait,
                label: "Trait".to_string(),
                value: "healing".to_string(),
            }],
            sections: vec![PresentationSection::new(
                PresentationSectionKind::Description,
                vec![
                    PresentationBlock::FactList(vec![PresentationFact {
                        key: "traditions".to_string(),
                        label: "Traditions".to_string(),
                        value: "divine".to_string(),
                    }]),
                    PresentationBlock::Content(PresentationContent::new(vec![
                        PresentationContentBlock::Paragraph {
                            spans: vec![PresentationInline::Reference {
                                label: "Treat Wounds".to_string(),
                                record_key: Some(
                                    RecordKey::parse("actions:TreatWounds")
                                        .expect("record key should parse"),
                                ),
                                embedded: false,
                            }],
                        },
                    ])),
                    PresentationBlock::Relationships(vec![PresentationRelationship {
                        kind: PresentationRelationshipKind::Reference,
                        label: "Treat Wounds".to_string(),
                        record_key: Some(
                            RecordKey::parse("actions:TreatWounds")
                                .expect("record key should parse"),
                        ),
                    }]),
                ],
            )],
        };

        let value = serde_json::to_value(document).expect("document should serialize");

        assert_eq!(value["record_key"], "spells:Heal");
        assert_eq!(value["kind"], "spell");
        assert_eq!(value["badges"][0]["kind"], "trait");
        assert_eq!(value["sections"][0]["kind"], "description");
        assert_eq!(value["sections"][0]["blocks"][0]["kind"], "fact_list");
        assert_eq!(
            value["sections"][0]["blocks"][0]["content"][0]["key"],
            "traditions"
        );
        assert_eq!(
            value["sections"][0]["blocks"][1]["content"]["blocks"][0]["kind"],
            "paragraph"
        );
        assert_eq!(
            value["sections"][0]["blocks"][1]["content"]["blocks"][0]["spans"][0]["record_key"],
            "actions:TreatWounds"
        );
        assert_eq!(
            value["sections"][0]["blocks"][2]["content"][0]["kind"],
            "reference"
        );
    }
}
