//! Terminal formatting of consolidated Rust-projected relationship facts.
use atlas_app_model::{
    RelatedCategoryView, RemasterInlinePartView, RemasterRelationshipsView,
    RemasterSourceContextView,
};
use std::fmt::Write;

pub(crate) fn lines(category: &RelatedCategoryView<RemasterRelationshipsView>) -> String {
    let mut out = String::new();
    match category {
        RelatedCategoryView::Content { content } => {
            for relationship in &content.relationships {
                let _ = write!(out, "{}", relationship.label);
                if !relationship.parts.is_empty() {
                    let _ = write!(out, ": ");
                }
                for part in &relationship.parts {
                    match part {
                        RemasterInlinePartView::Text { text } => {
                            let _ = write!(out, "{text}");
                        }
                        RemasterInlinePartView::Reference { label, navigation } => {
                            let _ = write!(out, "{label} ({})", navigation.record_key);
                        }
                    }
                }
                let _ = writeln!(out);
                for link in &relationship.explanation_links {
                    let _ = writeln!(out, "{} ({})", link.label, link.navigation.record_key);
                }
            }
            if content.truncated {
                let _ = writeln!(out, "More remaster information is available.");
            }
        }
        RelatedCategoryView::Empty => {}
        RelatedCategoryView::Unavailable { reason } => {
            let _ = writeln!(out, "Remaster information unavailable: {reason}");
        }
        RelatedCategoryView::Error { error } => {
            let _ = writeln!(
                out,
                "Remaster information could not be read: {}",
                error.message
            );
        }
    }
    out
}
pub(crate) fn source_context(category: &RelatedCategoryView<RemasterSourceContextView>) -> String {
    match category {
        RelatedCategoryView::Content { content } => {
            format!("{}\n{}", content.heading, lines(&content.relationships))
        }
        RelatedCategoryView::Empty => String::new(),
        RelatedCategoryView::Unavailable { reason } => {
            format!("Source-entry remaster information unavailable: {reason}\n")
        }
        RelatedCategoryView::Error { error } => format!(
            "Source-entry remaster information could not be read: {}\n",
            error.message
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_record_context_renders_relationship_and_useful_journal_destination() {
        let category=serde_json::from_value(serde_json::json!({"kind":"content","content":{"relationships":[{"id":"former","label":"Former name","parts":[{"kind":"text","text":"Magic Missile"}],"explanation_links":[{"label":"Remaster Changes — Spells","navigation":{"record_key":"journals:aaaaaaaaaaaaaaaa","embedded_path":[],"field":"/text/content","passage":null,"source_fingerprint":null}}],"truncated":false}],"truncated":false}})).unwrap();
        assert_eq!(
            lines(&category),
            "Former name: Magic Missile\nRemaster Changes — Spells (journals:aaaaaaaaaaaaaaaa)\n"
        );
    }
}
