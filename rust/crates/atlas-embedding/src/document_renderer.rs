use atlas_record::{
    PresentationBlock, PresentationFact, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, RecordPresentationDocument,
};

pub fn render_presentation_document_for_embedding(document: &RecordPresentationDocument) -> String {
    let mut lines = Vec::new();
    push_line(&mut lines, &format!("Name: {}", document.title));
    push_fact_lines(&mut lines, &document.identity);
    render_badges(&mut lines, document);

    for kind in EMBEDDING_SECTION_ORDER {
        for section in document
            .sections
            .iter()
            .filter(|section| section.kind == *kind)
        {
            render_section(&mut lines, section);
        }
    }

    lines.join("\n")
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

fn render_badges(lines: &mut Vec<String>, document: &RecordPresentationDocument) {
    let traits = document
        .badges
        .iter()
        .filter(|badge| badge.label == "Trait")
        .map(|badge| badge.value.as_str())
        .collect::<Vec<_>>();
    if !traits.is_empty() {
        push_line(lines, &format!("Traits: {}", traits.join(", ")));
    }

    let classifications = document
        .badges
        .iter()
        .filter(|badge| badge.label != "Trait")
        .map(|badge| badge.value.as_str())
        .collect::<Vec<_>>();
    if !classifications.is_empty() {
        push_line(
            lines,
            &format!("Classification: {}", classifications.join(", ")),
        );
    }
}

fn render_section(lines: &mut Vec<String>, section: &PresentationSection) {
    if section.kind == PresentationSectionKind::Backlinks {
        return;
    }
    for block in &section.blocks {
        match block {
            PresentationBlock::FactList(facts) => push_fact_lines(lines, facts),
            PresentationBlock::Prose(text) => {
                push_line(lines, &format!("Description: {}", text.text));
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
                    push_line(lines, &format!("References: {}", references.join(", ")));
                }
            }
        }
    }
}

fn push_fact_lines(lines: &mut Vec<String>, facts: &[PresentationFact]) {
    for fact in facts {
        push_line(lines, &format!("{}: {}", fact.label, fact.value));
    }
}

fn push_line(lines: &mut Vec<String>, line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }
    if !lines.iter().any(|existing| existing == trimmed) {
        lines.push(trimmed.to_string());
    }
}
