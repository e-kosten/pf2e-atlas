use super::{ContentBlock, ContentDocument, ContentInline, ContentReference};

pub type ContentReferenceIter<'a> = Box<dyn Iterator<Item = &'a ContentReference> + 'a>;

pub fn iter_content_references(document: &ContentDocument) -> ContentReferenceIter<'_> {
    Box::new(document.blocks.iter().flat_map(iter_block_references))
}

pub fn visit_content_references_mut(
    document: &mut ContentDocument,
    mut visitor: impl FnMut(&mut ContentReference),
) {
    for block in &mut document.blocks {
        visit_block_references_mut(block, &mut visitor);
    }
}

fn iter_block_references(block: &ContentBlock) -> ContentReferenceIter<'_> {
    match block {
        ContentBlock::Heading { content, .. } | ContentBlock::Paragraph { content } => {
            Box::new(iter_inline_references(content))
        }
        ContentBlock::List { items, .. } => Box::new(
            items
                .iter()
                .flat_map(|item| item.iter().flat_map(iter_block_references)),
        ),
        ContentBlock::Table {
            caption,
            headers,
            rows,
        } => Box::new(
            caption
                .iter()
                .flat_map(|caption| iter_inline_references(caption))
                .chain(headers.iter().flat_map(|cell| iter_inline_references(cell)))
                .chain(
                    rows.iter()
                        .flat_map(|row| row.iter().flat_map(|cell| iter_inline_references(cell))),
                ),
        ),
        ContentBlock::Callout { title, blocks } | ContentBlock::RuleBlock { title, blocks } => {
            Box::new(
                title
                    .iter()
                    .flat_map(|title| iter_inline_references(title))
                    .chain(blocks.iter().flat_map(iter_block_references)),
            )
        }
        ContentBlock::DefinitionList { items } => Box::new(items.iter().flat_map(|item| {
            iter_inline_references(&item.term)
                .chain(item.definition.iter().flat_map(iter_block_references))
        })),
        ContentBlock::Separator => Box::new(std::iter::empty()),
    }
}

fn iter_inline_references(
    inlines: &[ContentInline],
) -> impl Iterator<Item = &ContentReference> + '_ {
    inlines.iter().flat_map(iter_inline_reference)
}

fn iter_inline_reference(inline: &ContentInline) -> ContentReferenceIter<'_> {
    match inline {
        ContentInline::Reference { reference } => Box::new(
            std::iter::once(reference).chain(
                reference
                    .label
                    .iter()
                    .flat_map(|label| iter_inline_references(label)),
            ),
        ),
        ContentInline::Strong { content } | ContentInline::Emphasis { content } => {
            Box::new(iter_inline_references(content))
        }
        ContentInline::Text { .. }
        | ContentInline::Code { .. }
        | ContentInline::Break
        | ContentInline::Roll { .. }
        | ContentInline::Template { .. }
        | ContentInline::Macro { .. }
        | ContentInline::ActionGlyph { .. }
        | ContentInline::Icon { .. } => Box::new(std::iter::empty()),
    }
}

fn visit_block_references_mut(
    block: &mut ContentBlock,
    visitor: &mut impl FnMut(&mut ContentReference),
) {
    match block {
        ContentBlock::Heading { content, .. } | ContentBlock::Paragraph { content } => {
            visit_inline_references_mut(content, visitor);
        }
        ContentBlock::List { items, .. } => {
            for item in items {
                for block in item {
                    visit_block_references_mut(block, visitor);
                }
            }
        }
        ContentBlock::Table {
            caption,
            headers,
            rows,
        } => {
            if let Some(caption) = caption {
                visit_inline_references_mut(caption, visitor);
            }
            for header in headers {
                visit_inline_references_mut(header, visitor);
            }
            for row in rows {
                for cell in row {
                    visit_inline_references_mut(cell, visitor);
                }
            }
        }
        ContentBlock::Callout { title, blocks } | ContentBlock::RuleBlock { title, blocks } => {
            if let Some(title) = title {
                visit_inline_references_mut(title, visitor);
            }
            for block in blocks {
                visit_block_references_mut(block, visitor);
            }
        }
        ContentBlock::DefinitionList { items } => {
            for item in items {
                visit_inline_references_mut(&mut item.term, visitor);
                for block in &mut item.definition {
                    visit_block_references_mut(block, visitor);
                }
            }
        }
        ContentBlock::Separator => {}
    }
}

fn visit_inline_references_mut(
    inlines: &mut [ContentInline],
    visitor: &mut impl FnMut(&mut ContentReference),
) {
    for inline in inlines {
        match inline {
            ContentInline::Reference { reference } => {
                visitor(reference);
                if let Some(label) = &mut reference.label {
                    visit_inline_references_mut(label, visitor);
                }
            }
            ContentInline::Strong { content } | ContentInline::Emphasis { content } => {
                visit_inline_references_mut(content, visitor);
            }
            ContentInline::Text { .. }
            | ContentInline::Code { .. }
            | ContentInline::Break
            | ContentInline::Roll { .. }
            | ContentInline::Template { .. }
            | ContentInline::Macro { .. }
            | ContentInline::ActionGlyph { .. }
            | ContentInline::Icon { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{ContentReferenceLocator, ContentSourceKind, ContentVisibility};

    #[test]
    fn finds_references_inside_nested_blocks_and_inline_labels() {
        let document = ContentDocument::new(vec![ContentBlock::DefinitionList {
            items: vec![super::super::ContentDefinitionItem {
                term: vec![ContentInline::Reference {
                    reference: ContentReference {
                        label: None,
                        locator: ContentReferenceLocator::Unknown {
                            raw: "term".to_string(),
                        },
                        resolved_key: None,
                    },
                }],
                definition: vec![ContentBlock::Paragraph {
                    content: vec![ContentInline::Reference {
                        reference: ContentReference {
                            label: Some(vec![ContentInline::Reference {
                                reference: ContentReference {
                                    label: None,
                                    locator: ContentReferenceLocator::Unknown {
                                        raw: "label".to_string(),
                                    },
                                    resolved_key: None,
                                },
                            }]),
                            locator: ContentReferenceLocator::Unknown {
                                raw: "body".to_string(),
                            },
                            resolved_key: None,
                        },
                    }],
                }],
            }],
        }]);

        let raw_values: Vec<String> = iter_content_references(&document)
            .map(|reference| match &reference.locator {
                ContentReferenceLocator::Unknown { raw } => raw.clone(),
                _ => unreachable!("test only uses unknown locators"),
            })
            .collect();

        assert_eq!(raw_values, vec!["term", "body", "label"]);
        assert_eq!(
            ContentSourceKind::PublicNotes.default_visibility(),
            ContentVisibility::Public
        );
    }
}
