use super::identity::valid_source_id;
use super::{ItemSourceView, SourceFieldView, SourceNodeView};
use crate::source_content::{OwnedContentIdentity, OwnedContentLocator};
use atlas_foundry_model::{FoundryDocumentSource, ItemSourcePF2e, generated::*};
use std::collections::BTreeMap;

/// Visit authored preorder using borrowed bodies and one reusable owner chain.
/// Only active ancestor frames and sibling ID counts are retained. Unavailable
/// collections never yield a salvaged subset or prove a known empty collection.
pub(super) fn visit_source_nodes<'a>(
    source: &'a FoundryDocumentSource,
    mut visit: impl FnMut(&[OwnedContentLocator], SourceNodeView<'a>),
) {
    let root = SourceNodeView::from(source);
    visit(&[], root);
    let mut frames = Vec::new();
    if let Some(frame) = ChildFrame::from_node(root) {
        frames.push(frame);
    }
    let mut owners = Vec::new();
    while !frames.is_empty() {
        owners.truncate(frames.len() - 1);
        let Some(frame) = frames.last_mut() else {
            break;
        };
        let Some(node) = frame.children.node(frame.next_index) else {
            frames.pop();
            continue;
        };
        let index = frame.next_index;
        frame.next_index += 1;
        let identity = match node.id().value() {
            Some(id) if valid_source_id(id) && frame.ids[id.as_str()] == 1 => {
                OwnedContentIdentity::Stable(id.clone())
            }
            _ => OwnedContentIdentity::SnapshotLocal { index },
        };
        owners.push(OwnedContentLocator {
            collection: frame.collection.into(),
            identity,
        });
        visit(&owners, node);
        if let Some(frame) = ChildFrame::from_node(node) {
            frames.push(frame);
        }
    }
}

#[derive(Clone, Copy)]
enum BorrowedChildren<'a> {
    Items(&'a [ItemSourcePF2e]),
    PhysicalItems(&'a [PhysicalItemSource]),
    Spell(&'a SpellSource),
    Pages(&'a [SourceFromSchemaJournalEntrySchemaPagesEntrySourceFromSchema]),
    Results(&'a [SourceFromSchemaTableResultSchema]),
}

impl<'a> BorrowedChildren<'a> {
    fn node(self, index: usize) -> Option<SourceNodeView<'a>> {
        match self {
            Self::Items(items) => items
                .get(index)
                .map(|item| SourceNodeView::Item(item.into())),
            Self::PhysicalItems(items) => items
                .get(index)
                .map(|item| SourceNodeView::Item(item.into())),
            Self::Spell(spell) => {
                (index == 0).then_some(SourceNodeView::Item(ItemSourceView::SpellSource(spell)))
            }
            Self::Pages(pages) => pages.get(index).map(SourceNodeView::JournalPage),
            Self::Results(results) => results.get(index).map(SourceNodeView::Result),
        }
    }
}

struct ChildFrame<'a> {
    collection: &'static str,
    children: BorrowedChildren<'a>,
    next_index: usize,
    ids: BTreeMap<&'a str, usize>,
}

impl<'a> ChildFrame<'a> {
    fn from_node(node: SourceNodeView<'a>) -> Option<Self> {
        let (collection, children) = match node {
            SourceNodeView::Actor(_) => (
                "/items",
                BorrowedChildren::Items(node.actor_items().value()?),
            ),
            SourceNodeView::Item(ItemSourceView::ConsumableSource(source)) => (
                "/system/spell",
                BorrowedChildren::Spell(
                    SourceFieldView::from(&source.system)
                        .and_then(|system| (&system.spell).into())
                        .value()?,
                ),
            ),
            SourceNodeView::Item(item) => (
                "/system/subitems",
                BorrowedChildren::PhysicalItems(item.subitems().value()?),
            ),
            SourceNodeView::Journal(source) => (
                "/pages",
                BorrowedChildren::Pages(SourceFieldView::from(&source.pages).value()?),
            ),
            SourceNodeView::Table(source) => (
                "/results",
                BorrowedChildren::Results(SourceFieldView::from(&source.results).value()?),
            ),
            _ => return None,
        };
        let mut ids = BTreeMap::new();
        for node in (0..).map_while(|index| children.node(index)) {
            if let Some(id) = node.id().value().filter(|id| valid_source_id(id)) {
                *ids.entry(id.as_str()).or_insert(0usize) += 1;
            }
        }
        Some(Self {
            collection,
            children,
            next_index: 0,
            ids,
        })
    }
}
