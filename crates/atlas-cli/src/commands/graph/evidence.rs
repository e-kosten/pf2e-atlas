//! Present the direction and ownership of a single attributed reference.
use super::super::record::navigation::Locator;
use atlas_app_model::{RecordReferenceDestinationView, RecordRelationshipView, RecordSummaryView};
use serde_json::{Value, json};

pub(crate) fn identity(record: &RecordSummaryView) -> Value {
    let mut fields = json!({
        "record_key": record.record_key,
        "title": record.title,
        "kind_label": record.kind_label,
        "publication": record.publication,
    });
    for (key, value) in [
        ("kind", &record.kind),
        ("parent_kind", &record.parent_kind),
        ("journal_pack_kind", &record.journal_pack_kind),
        ("level_label", &record.level_label),
        ("unit_price_label", &record.unit_price_label),
    ] {
        if let Some(value) = value {
            fields[key] = json!(value);
        }
    }
    if record.journal_pack_kind.is_some()
        && let Some(label) = &record.pack
    {
        fields["pack"] = json!(label);
    }
    if let Some(level) = &record.level {
        fields["level"] = json!(level);
    }
    if let Some(basis) = record.level_basis {
        fields["level_basis"] = json!(basis);
    }
    if let Some(edition) = record.edition {
        fields["edition"] = json!(edition);
    }
    fields
}

pub(crate) fn occurrence(
    edge: &RecordRelationshipView,
    owner: Option<&RecordSummaryView>,
    generation: &str,
) -> Value {
    use RecordReferenceDestinationView::*;
    let destination = match &edge.destination {
        ResolvedRecord { target } | ResolvedEmbedded { target } => json!({
            "kind": if matches!(edge.destination, ResolvedRecord { .. }) { "resolved_record" } else { "resolved_embedded" },
            "target": Locator::new(generation, target).compact(),
        }),
        other => json!(other),
    };
    let mut result = json!({
        "kind":edge.kind, "source_title":edge.source_title, "owner":edge.source_navigation.record_key,
        "owner_title":owner.map(|r| r.title.as_str()), "field":edge.source.field,
        "source_navigation":Locator::new(generation, &edge.source_navigation).compact(),
        "target_title":edge.target_title, "destination":destination, "availability":edge.availability,
    });
    if !edge.audiences.is_empty() {
        result["audiences"] = json!(edge.audiences);
    }
    if edge.destination.record_target().is_none()
        && let Some(target) = &edge.authored_target
    {
        result["authored_target"] = json!(target);
    }
    if let Some(target) = &edge.contextual_target {
        result["contextual_target"] = Locator::new(generation, target).compact();
    }
    if let Some(fields) = result.as_object_mut() {
        fields.retain(|_, value| !value.is_null());
    }
    result
}

pub(crate) fn line(edge: &RecordRelationshipView, owner: Option<&RecordSummaryView>) -> String {
    use RecordReferenceDestinationView::*;
    let root = owner.map_or(edge.source_navigation.record_key.as_str(), |r| &r.title);
    let source = match edge.source_title.as_deref() {
        Some(title) if title != root => format!("{root} / {title}"),
        _ => root.to_owned(),
    };
    let kind = match edge.kind.as_str() {
        "CompendiumProvenance" => "compendium source",
        kind => kind,
    };
    let destination = match &edge.destination {
        ResolvedRecord { target } | ResolvedEmbedded { target } => edge
            .target_title
            .as_deref()
            .unwrap_or(&target.record_key)
            .to_owned(),
        ResolvedUrl { url } => url.clone(),
        UnverifiedUrl { url } => format!("{url} (unverified URL)"),
        Unresolved => format!(
            "{} (unresolved)",
            edge.authored_target.as_deref().unwrap_or("reference")
        ),
        Blocked => "blocked reference".into(),
    };
    let availability = edge
        .availability
        .filter(|state| *state != atlas_domain::QueryFieldState::Value)
        .map(|state| format!(", {}", state.as_str()))
        .unwrap_or_default();
    format!(
        "{source} → {destination} ({kind}, {}{availability})",
        edge.source.field
    )
}
