//! The editor maps the single backend catalog to conventional controls.
use atlas_app_model::{
    FilterControlView, FilterEditorFieldView, FilterEditorGroupView, FilterEditorView,
    FilterFieldPlacement,
};
use atlas_domain::{QueryCapability, QueryFieldType};
use std::collections::BTreeMap;
pub(crate) fn filter_editor_view(
    catalog: QueryCapability,
    selected: &[String],
) -> FilterEditorView {
    let mut groups = BTreeMap::<String, FilterEditorGroupView>::new();
    for definition in catalog.fields {
        let id = definition
            .scope
            .clone()
            .unwrap_or_else(|| definition.path.split('.').next().unwrap_or("Common").into());
        let control = match definition.field_type {
            QueryFieldType::String => FilterControlView::Text,
            QueryFieldType::Number => FilterControlView::Numeric,
            QueryFieldType::Boolean => FilterControlView::Boolean,
            QueryFieldType::Set => FilterControlView::Set,
            QueryFieldType::Collection => FilterControlView::Collection,
        };
        let placement = if selected.contains(&definition.id)
            || ["traits", "rarity", "level", "spell.rank"].contains(&definition.path.as_str())
        {
            FilterFieldPlacement::InitiallyVisible
        } else {
            FilterFieldPlacement::Addable
        };
        groups
            .entry(id.clone())
            .or_insert_with(|| FilterEditorGroupView {
                id: id.clone(),
                label: crate::projection::kind_label(&id),
                fields: vec![],
            })
            .fields
            .push(FilterEditorFieldView {
                definition,
                control,
                placement,
            });
    }
    FilterEditorView {
        catalog_version: catalog.version,
        limits: catalog.limits,
        groups: groups.into_values().collect(),
    }
}
