//! Read the visible prepared markers using their checked host or inclusion bindings.
use super::navigation::Locator;
use atlas_app_model::{
    PreparedContentFieldView, PreparedFieldBodyView, RecordDetailView,
    RecordReferenceDestinationView, RecordRelationshipView,
};
use scraper::{ElementRef, Html, Selector};
use serde_json::{Value, json};

pub(super) struct Link<'a> {
    label: String,
    binding: Option<&'a RecordRelationshipView>,
}

pub(super) fn visible<'a>(
    field: &'a PreparedContentFieldView,
    relationships: &'a [RecordRelationshipView],
) -> Result<Vec<Link<'a>>, String> {
    let PreparedFieldBodyView::Html {
        html, inclusions, ..
    } = &field.body
    else {
        return Ok(vec![]);
    };
    let html = Html::parse_fragment(html);
    let markers = Selector::parse("[data-atlas-reference]").map_err(|e| e.to_string())?;
    let mut links = Vec::new();
    for marker in html.select(&markers) {
        let ordinal = marker
            .attr("data-atlas-reference")
            .and_then(|v| v.parse::<usize>().ok());
        let namespace = std::iter::once(marker)
            .chain(marker.ancestors().filter_map(ElementRef::wrap))
            .find_map(|e| e.attr("data-atlas-inclusion"));
        let binding = ordinal.and_then(|ordinal| match namespace {
            Some(value) => inclusions
                .iter()
                .find(|i| Some(i.namespace) == value.parse().ok())
                .and_then(|i| i.references.iter().find(|r| r.ordinal == ordinal)),
            None => relationships
                .iter()
                .find(|r| r.source == field.locator && r.ordinal == ordinal),
        });
        let label = marker.text().collect::<String>();
        links.push(Link { label, binding });
    }
    Ok(links)
}

impl Link<'_> {
    pub(super) fn json(&self, generation: &str) -> Value {
        let Some(rel) = self.binding else {
            return json!({"label": self.label, "state": "binding_unavailable"});
        };
        let mut destination = json!(rel.destination);
        if let Some(nav) = rel.destination.record_target() {
            destination["target"] = json!(Locator::new(generation, nav));
        }
        let mut link = json!({
            "label": self.label, "target": destination, "audiences": rel.audiences,
            "availability": rel.availability, "authored_target": rel.authored_target
        });
        if let Some(target) = &rel.contextual_target {
            link["contextual_target"] = json!(Locator::new(generation, target));
        }
        link
    }

    fn text(&self, generation: &str) -> String {
        let Some(rel) = self.binding else {
            return format!("{} · reference unavailable", self.label);
        };
        let target = match &rel.destination {
            RecordReferenceDestinationView::ResolvedRecord { target }
                if target.owners.is_empty()
                    && target.field.is_none()
                    && target.passage.is_none()
                    && target.spell_rank.is_none() =>
            {
                target.record_key.clone()
            }
            RecordReferenceDestinationView::ResolvedRecord { target }
            | RecordReferenceDestinationView::ResolvedEmbedded { target } => {
                json!(Locator::new(generation, target)).to_string()
            }
            RecordReferenceDestinationView::ResolvedUrl { url } => url.clone(),
            RecordReferenceDestinationView::UnverifiedUrl { url } => format!("{url} (unverified)"),
            RecordReferenceDestinationView::Unresolved => format!(
                "{} (unresolved)",
                rel.authored_target.as_deref().unwrap_or("Target")
            ),
            RecordReferenceDestinationView::Blocked => "Blocked target".to_owned(),
        };
        let audience = rel.audiences.join(", ");
        let availability = rel.availability.map(|state| state.as_str()).unwrap_or("");
        let notes = [audience.as_str(), availability]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        if notes.is_empty() {
            format!("{} · {target}", self.label)
        } else {
            format!("{} · {target} ({notes})", self.label)
        }
    }
}

pub(super) fn append(out: &mut String, record: &RecordDetailView) -> Result<(), String> {
    let mut heading = false;
    let inline = match &record.presentation.body {
        atlas_app_model::RecordBodyView::Journal(journal) => journal.inline_page.as_ref(),
        _ => None,
    };
    let host = (&record.presentation.content, &record.relationships);
    for (content, relationships) in
        std::iter::once(host).chain(inline.map(|page| (&page.content, &page.relationships)))
    {
        for field in content {
            for link in visible(field, relationships)? {
                if !heading {
                    out.push_str("\nReferences\n");
                    heading = true;
                }
                out.push_str(&link.text(&record.artifact_generation));
                out.push('\n');
            }
        }
    }
    Ok(())
}
