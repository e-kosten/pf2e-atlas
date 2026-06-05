use super::{FoundryLink, FoundryNode, RichDocument, RichNode};

pub type FoundryLinkIter<'a> = Box<dyn Iterator<Item = &'a FoundryLink> + 'a>;

pub fn iter_foundry_links(document: &RichDocument) -> FoundryLinkIter<'_> {
    Box::new(document.nodes.iter().flat_map(iter_node_links))
}

pub fn visit_foundry_links_mut(
    document: &mut RichDocument,
    mut visitor: impl FnMut(&mut FoundryLink),
) {
    for node in &mut document.nodes {
        visit_node_links_mut(node, &mut visitor);
    }
}

fn iter_node_links(node: &RichNode) -> FoundryLinkIter<'_> {
    match node {
        RichNode::Text { .. } => Box::new(std::iter::empty()),
        RichNode::HtmlElement { children, .. } => {
            Box::new(children.iter().flat_map(iter_node_links))
        }
        RichNode::FoundryLink { link } => Box::new(
            std::iter::once(link).chain(
                link.label
                    .iter()
                    .flat_map(|label| label.iter().flat_map(iter_node_links)),
            ),
        ),
        RichNode::Foundry { node } => iter_foundry_node_links(node),
    }
}

fn iter_foundry_node_links(node: &FoundryNode) -> FoundryLinkIter<'_> {
    match node {
        FoundryNode::Check { label, .. }
        | FoundryNode::Damage { label, .. }
        | FoundryNode::InlineCommand { label, .. }
        | FoundryNode::Template { label, .. }
        | FoundryNode::Trait { label, .. }
        | FoundryNode::UnknownFoundry { label, .. } => Box::new(
            label
                .iter()
                .flat_map(|label| label.iter().flat_map(iter_node_links)),
        ),
        FoundryNode::Localize { value, .. } => Box::new(
            value
                .iter()
                .flat_map(|value| value.iter().flat_map(iter_node_links)),
        ),
        FoundryNode::ActionGlyph { .. } => Box::new(std::iter::empty()),
    }
}

fn visit_node_links_mut(node: &mut RichNode, visitor: &mut impl FnMut(&mut FoundryLink)) {
    match node {
        RichNode::Text { .. } => {}
        RichNode::HtmlElement { children, .. } => {
            for child in children {
                visit_node_links_mut(child, visitor);
            }
        }
        RichNode::FoundryLink { link } => {
            visitor(link);
            if let Some(label) = &mut link.label {
                for node in label {
                    visit_node_links_mut(node, visitor);
                }
            }
        }
        RichNode::Foundry { node } => visit_foundry_node_links_mut(node, visitor),
    }
}

fn visit_foundry_node_links_mut(
    node: &mut FoundryNode,
    visitor: &mut impl FnMut(&mut FoundryLink),
) {
    match node {
        FoundryNode::Check { label, .. }
        | FoundryNode::Damage { label, .. }
        | FoundryNode::InlineCommand { label, .. }
        | FoundryNode::Template { label, .. }
        | FoundryNode::Trait { label, .. }
        | FoundryNode::UnknownFoundry { label, .. } => {
            if let Some(label) = label {
                for node in label {
                    visit_node_links_mut(node, visitor);
                }
            }
        }
        FoundryNode::Localize { value, .. } => {
            if let Some(value) = value {
                for node in value {
                    visit_node_links_mut(node, visitor);
                }
            }
        }
        FoundryNode::ActionGlyph { .. } => {}
    }
}
