//! Names come from the selected source kind; active numeric values are consumer context.
use super::{ItemSourceView, SourceNodeView};
use crate::source_content::FormulaContext;

pub fn source_formula_labels(node: SourceNodeView<'_>) -> FormulaContext {
    FormulaContext {
        item_is_spell: matches!(node, SourceNodeView::Item(ItemSourceView::SpellSource(_))),
        condition_name: if matches!(
            node,
            SourceNodeView::Item(ItemSourceView::ConditionSource(_))
        ) {
            node.name().value().cloned()
        } else {
            None
        },
        ..FormulaContext::default()
    }
}
