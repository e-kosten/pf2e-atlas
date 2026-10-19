//! Authored cost display facts shared by spells, activities and owned links.
use super::*;
use atlas_foundry_model::generated::AbilitySystemSourceActionTypeSourceFromSchemaValue as ActionType;
use atlas_record::source_record::spell_casting_form;

pub(super) fn casting_cost(value: &str) -> ActionCostView {
    let (glyph, label) = match spell_casting_form(value) {
        "one_action" => (Some(ActionGlyphView::OneAction), "1 action"),
        "two_actions" => (Some(ActionGlyphView::TwoActions), "2 actions"),
        "three_actions" => (Some(ActionGlyphView::ThreeActions), "3 actions"),
        "free" => (Some(ActionGlyphView::FreeAction), "free action"),
        "reaction" => (Some(ActionGlyphView::Reaction), "reaction"),
        "one_or_two" => (None, "1 or 2 actions"),
        "one_to_three" => (None, "1 to 3 actions"),
        "two_or_three" => (None, "2 or 3 actions"),
        _ => (None, value),
    };
    ActionCostView {
        glyph,
        label: label.into(),
    }
}

pub(super) fn item_action_cost(item: ItemSourceView<'_>) -> FactView<ActionCostView> {
    if item.family() == "spell" {
        return fact(item.spell_casting_time().map(casting_cost));
    }
    fact(item.action_type().and_then(|kind| match kind {
        ActionType::Action => item.action_count().map(|count| match count.as_f64() {
            Some(1.0) => casting_cost("1"),
            Some(2.0) => casting_cost("2"),
            Some(3.0) => casting_cost("3"),
            _ => ActionCostView {
                glyph: None,
                label: format!("{count} actions"),
            },
        }),
        ActionType::Free => SourceFieldView::Value(ActionCostView {
            glyph: Some(ActionGlyphView::FreeAction),
            label: "free action".into(),
        }),
        ActionType::Reaction => SourceFieldView::Value(ActionCostView {
            glyph: Some(ActionGlyphView::Reaction),
            label: "reaction".into(),
        }),
        ActionType::Passive => SourceFieldView::Value(ActionCostView {
            glyph: None,
            label: "passive".into(),
        }),
    }))
}
