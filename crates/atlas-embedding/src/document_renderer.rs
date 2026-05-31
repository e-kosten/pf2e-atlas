use atlas_record::{
    ContentSourceKind, PresentationBlock, PresentationFact, PresentationRelationshipKind,
    PresentationSection, PresentationSectionKind, RecordPresentationDocument,
};

pub fn render_presentation_document_for_embedding(document: &RecordPresentationDocument) -> String {
    render_embedding_chunks_for_embedding(&render_presentation_document_embedding_chunks(document))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingInputChunk {
    pub section: EmbeddingInputSection,
    pub text: String,
    pub truncatable: bool,
    pub(crate) source_kind: Option<ContentSourceKind>,
    pub(crate) group_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EmbeddingInputSection {
    Identity,
    Traits,
    Classification,
    Summary,
    Description,
    Defense,
    Movement,
    Offense,
    Routine,
    Details,
    References,
    Aliases,
}

impl EmbeddingInputSection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Traits => "traits",
            Self::Classification => "classification",
            Self::Summary => "summary",
            Self::Description => "description",
            Self::Defense => "defense",
            Self::Movement => "movement",
            Self::Offense => "offense",
            Self::Routine => "routine",
            Self::Details => "details",
            Self::References => "references",
            Self::Aliases => "aliases",
        }
    }
}

impl EmbeddingInputChunk {
    pub fn line(section: EmbeddingInputSection, text: impl Into<String>) -> Self {
        Self {
            section,
            text: text.into(),
            truncatable: false,
            source_kind: None,
            group_key: None,
        }
    }

    pub fn truncatable_line(section: EmbeddingInputSection, text: impl Into<String>) -> Self {
        Self {
            section,
            text: text.into(),
            truncatable: true,
            source_kind: None,
            group_key: None,
        }
    }

    pub(crate) fn with_source_kind(mut self, source_kind: ContentSourceKind) -> Self {
        self.source_kind = Some(source_kind);
        self
    }

    pub(crate) fn with_group_key(mut self, group_key: impl Into<String>) -> Self {
        self.group_key = Some(group_key.into());
        self
    }
}

pub fn render_presentation_document_embedding_chunks(
    document: &RecordPresentationDocument,
) -> Vec<EmbeddingInputChunk> {
    let mut chunks = Vec::new();
    push_chunk(
        &mut chunks,
        EmbeddingInputChunk::line(
            EmbeddingInputSection::Identity,
            format!("Name: {}", document.title),
        ),
    );
    push_fact_chunks(
        &mut chunks,
        EmbeddingInputSection::Identity,
        &document.identity,
    );
    render_badges(&mut chunks, document);

    for kind in EMBEDDING_SECTION_ORDER {
        for section in document
            .sections
            .iter()
            .filter(|section| section.kind == *kind)
        {
            render_section(&mut chunks, section);
        }
    }

    chunks
}

pub fn render_embedding_chunks_for_embedding(chunks: &[EmbeddingInputChunk]) -> String {
    chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

const EMBEDDING_SECTION_ORDER: &[PresentationSectionKind] = &[
    PresentationSectionKind::Summary,
    PresentationSectionKind::Description,
    PresentationSectionKind::Defense,
    PresentationSectionKind::Movement,
    PresentationSectionKind::Offense,
    PresentationSectionKind::Routine,
    PresentationSectionKind::Details,
    PresentationSectionKind::References,
    PresentationSectionKind::Classification,
];

fn render_badges(chunks: &mut Vec<EmbeddingInputChunk>, document: &RecordPresentationDocument) {
    let traits = document
        .badges
        .iter()
        .filter(|badge| badge.label == "Trait")
        .map(|badge| badge.value.as_str())
        .collect::<Vec<_>>();
    if !traits.is_empty() {
        push_chunk(
            chunks,
            EmbeddingInputChunk::line(
                EmbeddingInputSection::Traits,
                format!("Traits: {}", traits.join(", ")),
            ),
        );
    }

    let classifications = document
        .badges
        .iter()
        .filter(|badge| badge.label != "Trait")
        .map(|badge| badge.value.as_str())
        .collect::<Vec<_>>();
    if !classifications.is_empty() {
        push_chunk(
            chunks,
            EmbeddingInputChunk::line(
                EmbeddingInputSection::Classification,
                format!("Classification: {}", classifications.join(", ")),
            ),
        );
    }
}

fn render_section(chunks: &mut Vec<EmbeddingInputChunk>, section: &PresentationSection) {
    if section.kind == PresentationSectionKind::Backlinks {
        return;
    }
    let embedding_section = embedding_section_kind(section.kind);
    for block in &section.blocks {
        match block {
            PresentationBlock::FactList(facts) => {
                push_fact_chunks(chunks, embedding_section, facts)
            }
            PresentationBlock::Prose(text) => {
                push_chunk(
                    chunks,
                    EmbeddingInputChunk::truncatable_line(
                        embedding_section,
                        format!("Description: {}", text.text),
                    ),
                );
            }
            PresentationBlock::Content(document) => {
                push_chunk(
                    chunks,
                    EmbeddingInputChunk::truncatable_line(
                        embedding_section,
                        format!("Description: {}", atlas_record::render_plain_text(document)),
                    ),
                );
            }
            PresentationBlock::Relationships(relationships) => {
                let references = relationships
                    .iter()
                    .filter(|relationship| {
                        relationship.kind == PresentationRelationshipKind::Reference
                    })
                    .map(|relationship| relationship.label.as_str())
                    .collect::<Vec<_>>();
                if !references.is_empty() {
                    push_chunk(
                        chunks,
                        EmbeddingInputChunk::truncatable_line(
                            EmbeddingInputSection::References,
                            format!("References: {}", references.join(", ")),
                        ),
                    );
                }
            }
        }
    }
}

fn push_fact_chunks(
    chunks: &mut Vec<EmbeddingInputChunk>,
    section: EmbeddingInputSection,
    facts: &[PresentationFact],
) {
    for fact in facts {
        if is_display_only_fact(fact) {
            continue;
        }
        push_chunk(
            chunks,
            EmbeddingInputChunk::line(section, format!("{}: {}", fact.label, fact.value)),
        );
    }
}

fn is_display_only_fact(fact: &PresentationFact) -> bool {
    fact.key == "prerequisites"
}

fn push_chunk(chunks: &mut Vec<EmbeddingInputChunk>, mut chunk: EmbeddingInputChunk) {
    let trimmed = chunk.text.trim();
    if trimmed.is_empty() {
        return;
    }
    if !chunks.iter().any(|existing| existing.text == trimmed) {
        chunk.text = trimmed.to_string();
        chunks.push(chunk);
    }
}

fn embedding_section_kind(kind: PresentationSectionKind) -> EmbeddingInputSection {
    match kind {
        PresentationSectionKind::Summary => EmbeddingInputSection::Summary,
        PresentationSectionKind::DescriptionPreview => EmbeddingInputSection::Description,
        PresentationSectionKind::Description => EmbeddingInputSection::Description,
        PresentationSectionKind::Defense => EmbeddingInputSection::Defense,
        PresentationSectionKind::Movement => EmbeddingInputSection::Movement,
        PresentationSectionKind::Offense => EmbeddingInputSection::Offense,
        PresentationSectionKind::Routine => EmbeddingInputSection::Routine,
        PresentationSectionKind::Details => EmbeddingInputSection::Details,
        PresentationSectionKind::References => EmbeddingInputSection::References,
        PresentationSectionKind::Backlinks => EmbeddingInputSection::References,
        PresentationSectionKind::Classification => EmbeddingInputSection::Classification,
    }
}
