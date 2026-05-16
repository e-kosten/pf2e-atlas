use super::render::render_inlines_plain;
use super::{ContentBlock, ContentDocument, ContentInline};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSectionNode {
    pub title: Option<String>,
    pub level: u8,
    pub blocks: Vec<ContentBlock>,
    pub children: Vec<ContentSectionNode>,
}

pub fn build_content_section_tree(document: &ContentDocument) -> ContentSectionNode {
    let flat_sections = collect_flat_sections(document);
    let (children, _) = build_children(&flat_sections, 0, 0);
    let mut root = ContentSectionNode {
        title: None,
        level: 0,
        blocks: leading_blocks(document),
        children,
    };
    extract_synthetic_sections(&mut root);
    root
}

fn collect_flat_sections(document: &ContentDocument) -> Vec<FlatSection> {
    let mut sections = Vec::new();
    let mut current: Option<FlatSection> = None;

    for block in &document.blocks {
        if let ContentBlock::Heading { level, content } = block {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            current = Some(FlatSection {
                title: render_inlines_plain(content),
                level: *level,
                heading: block.clone(),
                blocks: Vec::new(),
            });
        } else if let Some(section) = &mut current {
            section.blocks.push(block.clone());
        }
    }

    if let Some(section) = current {
        sections.push(section);
    }

    sections
}

fn leading_blocks(document: &ContentDocument) -> Vec<ContentBlock> {
    document
        .blocks
        .iter()
        .take_while(|block| !matches!(block, ContentBlock::Heading { .. }))
        .cloned()
        .collect()
}

fn build_children(
    sections: &[FlatSection],
    mut index: usize,
    parent_level: u8,
) -> (Vec<ContentSectionNode>, usize) {
    let mut nodes = Vec::new();

    while index < sections.len() {
        let section = &sections[index];
        if section.level <= parent_level {
            break;
        }

        let mut node = ContentSectionNode {
            title: Some(section.title.clone()),
            level: section.level,
            blocks: std::iter::once(section.heading.clone())
                .chain(section.blocks.clone())
                .collect(),
            children: Vec::new(),
        };
        index += 1;

        let (children, next_index) = build_children(sections, index, section.level);
        node.children = children;
        index = next_index;
        nodes.push(node);
    }

    (nodes, index)
}

fn extract_synthetic_sections(node: &mut ContentSectionNode) {
    for child in &mut node.children {
        extract_synthetic_sections(child);
    }

    let mut retained_blocks = Vec::new();
    let mut synthetic_children = Vec::new();
    for block in std::mem::take(&mut node.blocks) {
        if retained_blocks.is_empty() && matches!(block, ContentBlock::Heading { .. }) {
            retained_blocks.push(block);
            continue;
        }

        if let Some(title) = synthetic_section_title(&block) {
            synthetic_children.push(ContentSectionNode {
                title: Some(title),
                level: node.level.saturating_add(1),
                blocks: vec![block],
                children: Vec::new(),
            });
        } else {
            retained_blocks.push(block);
        }
    }

    synthetic_children.extend(std::mem::take(&mut node.children));
    node.blocks = retained_blocks;
    node.children = synthetic_children;
}

fn synthetic_section_title(block: &ContentBlock) -> Option<String> {
    match block {
        ContentBlock::Paragraph { content } => strong_lead_title(content),
        ContentBlock::Table {
            caption: Some(caption),
            ..
        } => non_empty_title(render_inlines_plain(caption)),
        _ => None,
    }
}

fn strong_lead_title(content: &[ContentInline]) -> Option<String> {
    let Some(ContentInline::Strong { content }) = content.first() else {
        return None;
    };

    non_empty_title(
        render_inlines_plain(content)
            .trim_end_matches(':')
            .to_string(),
    )
}

fn non_empty_title(title: String) -> Option<String> {
    let title = title.trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlatSection {
    title: String,
    level: u8,
    heading: ContentBlock,
    blocks: Vec<ContentBlock>,
}

#[cfg(test)]
mod tests {
    use crate::content::{ContentInline, render_plain_text};

    use super::*;

    #[test]
    fn builds_tree_from_heading_stack_and_keeps_leading_blocks() {
        let document = ContentDocument::new(vec![
            ContentBlock::Paragraph {
                content: vec![ContentInline::Text {
                    text: "Lead text.".to_string(),
                }],
            },
            heading(2, "Actions"),
            ContentBlock::Paragraph {
                content: vec![ContentInline::Text {
                    text: "Action text.".to_string(),
                }],
            },
            heading(3, "One Action"),
            ContentBlock::Paragraph {
                content: vec![ContentInline::Text {
                    text: "Strike.".to_string(),
                }],
            },
            heading(2, "Aftermath"),
        ]);

        let tree = build_content_section_tree(&document);

        assert_eq!(
            render_plain_text(&ContentDocument::new(tree.blocks)),
            "Lead text."
        );
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].title.as_deref(), Some("Actions"));
        assert_eq!(
            tree.children[0].children[0].title.as_deref(),
            Some("One Action")
        );
        assert_eq!(tree.children[1].title.as_deref(), Some("Aftermath"));
    }

    #[test]
    fn promotes_strong_leads_and_table_captions_to_synthetic_sections() {
        let document = ContentDocument::new(vec![
            heading(2, "Outcomes"),
            ContentBlock::Paragraph {
                content: vec![
                    ContentInline::Strong {
                        content: vec![ContentInline::Text {
                            text: "Critical Success".to_string(),
                        }],
                    },
                    ContentInline::Text {
                        text: " You win.".to_string(),
                    },
                ],
            },
            ContentBlock::Table {
                caption: Some(vec![ContentInline::Text {
                    text: "Treasure by Level".to_string(),
                }]),
                headers: vec![vec![ContentInline::Text {
                    text: "Level".to_string(),
                }]],
                rows: vec![vec![vec![ContentInline::Text {
                    text: "1".to_string(),
                }]]],
            },
        ]);

        let tree = build_content_section_tree(&document);
        let outcomes = &tree.children[0];

        assert_eq!(outcomes.title.as_deref(), Some("Outcomes"));
        assert_eq!(outcomes.children.len(), 2);
        assert_eq!(
            outcomes.children[0].title.as_deref(),
            Some("Critical Success")
        );
        assert_eq!(
            outcomes.children[1].title.as_deref(),
            Some("Treasure by Level")
        );
    }

    fn heading(level: u8, title: &str) -> ContentBlock {
        ContentBlock::Heading {
            level,
            content: vec![ContentInline::Text {
                text: title.to_string(),
            }],
        }
    }
}
