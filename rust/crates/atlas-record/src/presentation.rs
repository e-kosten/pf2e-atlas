use atlas_domain::{RecordFamily, RecordKey};

use crate::ContentDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordPresentationDocument {
    pub record_key: RecordKey,
    pub record_family: RecordFamily,
    pub title: String,
    pub identity: Vec<PresentationFact>,
    pub badges: Vec<PresentationBadge>,
    pub sections: Vec<PresentationSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentationBlock {
    FactList(Vec<PresentationFact>),
    Prose(PresentationText),
    Content(ContentDocument),
    Relationships(Vec<PresentationRelationship>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationFact {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationText {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationBadge {
    pub kind: PresentationBadgeKind,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PresentationBadgeKind {
    Trait,
    Classification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationRelationship {
    pub kind: PresentationRelationshipKind,
    pub label: String,
    pub record_key: Option<RecordKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            record_family: RecordFamily::Spell,
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

        assert_eq!(document.record_family, RecordFamily::Spell);
        assert_eq!(document.sections[0].kind, PresentationSectionKind::Summary);
        assert_eq!(document.sections[1].title, "Description");
        assert!(matches!(
            &document.sections[2].blocks[0],
            PresentationBlock::Relationships(relationships)
                if relationships[0].label == "vitality"
        ));
    }
}
