use super::render::render_nodes_plain_text;
use super::{RichDocument, RichNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSectionNode {
    pub title: Option<String>,
    pub level: u8,
    pub origin: ContentSectionOrigin,
    pub source_nodes: Vec<RichNode>,
    pub nodes: Vec<RichNode>,
    pub children: Vec<ContentSectionNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentSectionOrigin {
    Root,
    ExplicitHeading,
    StrongLeadParagraph,
    TableCaption,
}

pub fn build_content_section_tree(document: &RichDocument) -> ContentSectionNode {
    let flat_sections = collect_flat_sections(document);
    let (children, _) = build_children(&flat_sections, 0, 0);
    let root_nodes = leading_nodes(document);
    let mut root = ContentSectionNode {
        title: None,
        level: 0,
        origin: ContentSectionOrigin::Root,
        source_nodes: root_nodes.clone(),
        nodes: root_nodes,
        children,
    };
    extract_synthetic_sections(&mut root);
    root
}

fn collect_flat_sections(document: &RichDocument) -> Vec<FlatSection> {
    let mut sections = Vec::new();
    let mut current: Option<FlatSection> = None;

    for node in &document.nodes {
        if let Some((level, title)) = heading_node(node) {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            current = Some(FlatSection {
                title,
                level,
                heading: node.clone(),
                nodes: Vec::new(),
            });
        } else if let Some(section) = &mut current {
            section.nodes.push(node.clone());
        }
    }

    if let Some(section) = current {
        sections.push(section);
    }

    sections
}

fn leading_nodes(document: &RichDocument) -> Vec<RichNode> {
    document
        .nodes
        .iter()
        .take_while(|node| heading_node(node).is_none())
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

        let rich_nodes: Vec<_> = std::iter::once(section.heading.clone())
            .chain(section.nodes.clone())
            .collect();
        let mut node = ContentSectionNode {
            title: Some(section.title.clone()),
            level: section.level,
            origin: ContentSectionOrigin::ExplicitHeading,
            source_nodes: rich_nodes.clone(),
            nodes: rich_nodes,
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

    let mut retained_nodes = Vec::new();
    let mut synthetic_children = Vec::new();
    for rich_node in std::mem::take(&mut node.nodes) {
        if retained_nodes.is_empty() && heading_node(&rich_node).is_some() {
            retained_nodes.push(rich_node);
            continue;
        }

        if let Some((origin, title)) = synthetic_section_title(&rich_node) {
            synthetic_children.push(ContentSectionNode {
                title: Some(title),
                level: node.level.saturating_add(1),
                origin,
                source_nodes: vec![rich_node.clone()],
                nodes: vec![rich_node],
                children: Vec::new(),
            });
        } else {
            retained_nodes.push(rich_node);
        }
    }

    synthetic_children.extend(std::mem::take(&mut node.children));
    node.nodes = retained_nodes;
    node.children = synthetic_children;
}

fn synthetic_section_title(node: &RichNode) -> Option<(ContentSectionOrigin, String)> {
    match node {
        RichNode::HtmlElement { tag, children, .. } if tag == "p" => strong_lead_title(children)
            .map(|title| (ContentSectionOrigin::StrongLeadParagraph, title)),
        RichNode::HtmlElement { tag, children, .. } if tag == "table" => {
            table_caption(children).map(|title| (ContentSectionOrigin::TableCaption, title))
        }
        _ => None,
    }
}

fn strong_lead_title(children: &[RichNode]) -> Option<String> {
    let Some(RichNode::HtmlElement { tag, children, .. }) = children.first() else {
        return None;
    };
    if tag != "strong" && tag != "b" {
        return None;
    }

    non_empty_title(
        render_nodes_plain_text(children)
            .trim_end_matches(':')
            .to_string(),
    )
}

fn table_caption(children: &[RichNode]) -> Option<String> {
    children.iter().find_map(|node| match node {
        RichNode::HtmlElement { tag, children, .. } if tag == "caption" => {
            non_empty_title(render_nodes_plain_text(children))
        }
        _ => None,
    })
}

fn heading_node(node: &RichNode) -> Option<(u8, String)> {
    let RichNode::HtmlElement { tag, children, .. } = node else {
        return None;
    };
    let level = heading_level(tag)?;
    non_empty_title(render_nodes_plain_text(children)).map(|title| (level, title))
}

fn heading_level(tag: &str) -> Option<u8> {
    let level = tag.strip_prefix('h')?.parse::<u8>().ok()?;
    (1..=6).contains(&level).then_some(level)
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
    heading: RichNode,
    nodes: Vec<RichNode>,
}
