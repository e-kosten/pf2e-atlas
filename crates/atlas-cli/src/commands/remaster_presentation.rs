//! Terminal formatting of consolidated Rust-projected relationship facts.
use atlas_app_model::{
    RelatedCategoryView, RemasterCitationDestinationView, RemasterCitationOriginView,
    RemasterInlinePartView, RemasterRelationshipsView, RemasterSourceContextView,
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
            }
            if content.truncated {
                let _ = writeln!(
                    out,
                    "Additional relationship details are available in Sources."
                );
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
pub(crate) fn full(category: &RelatedCategoryView<RemasterRelationshipsView>) -> String {
    let RelatedCategoryView::Content { content } = category else {
        return if matches!(category, RelatedCategoryView::Empty) {
            "No source-backed remaster relationships found.\n".into()
        } else {
            lines(category)
        };
    };
    let mut out = lines(category);
    if !content.declarations.is_empty() {
        let _ = writeln!(out, "Sources:");
    }
    for declaration in &content.declarations {
        for origin in declaration.origins.iter().filter(|origin| {
            !declaration
                .citations
                .iter()
                .any(|citation| citation.origin == **origin)
        }) {
            let _ = writeln!(
                out,
                "  {} (additional details omitted)",
                match origin {
                    RemasterCitationOriginView::Foundry => "Foundry source",
                    RemasterCitationOriginView::AtlasReviewed => "Atlas mapping",
                }
            );
        }
        for citation in &declaration.citations {
            let _ = writeln!(out, "  {} ({})", citation.attribution, citation.revision);
            let _ = writeln!(out, "    {}", citation.statement);
            match &citation.destination {
                RemasterCitationDestinationView::Internal { navigation } => {
                    let _ = writeln!(
                        out,
                        "    {}{}",
                        navigation.record_key,
                        navigation
                            .field
                            .as_ref()
                            .map(|field| format!(" — {field}"))
                            .unwrap_or_default()
                    );
                }
                RemasterCitationDestinationView::External { url } => {
                    let _ = writeln!(out, "    {url}");
                }
                RemasterCitationDestinationView::Attribution {
                    reason: Some(reason),
                } => {
                    let _ = writeln!(out, "    {reason}");
                }
                RemasterCitationDestinationView::Attribution { reason: None } => {}
            }
        }
        if declaration.citations_truncated {
            let _ = writeln!(out, "  Additional evidence omitted; inspect the source.");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::*;
    use serde_json::json;
    fn navigation(key: &str) -> RecordNavigationView {
        RecordNavigationView {
            record_key: key.into(),
            spell_rank: None,
            owners: vec![],
            field: None,
            passage: None,
            source_fingerprint: None,
        }
    }
    fn fixture() -> RelatedCategoryView<RemasterRelationshipsView> {
        let destination = navigation("items:bbbbbbbbbbbbbbbb");
        let mut source = navigation("journals:cccccccccccccccc");
        source.field = Some("/text/content".into());
        let citations = vec![
            RemasterCitationView {
                origin: RemasterCitationOriginView::Foundry,
                attribution: "Remaster Changes — Class Features".into(),
                revision: "upstream-revision".into(),
                statement: "Journal declaration".into(),
                shortened: false,
                destination: RemasterCitationDestinationView::Internal { navigation: source },
            },
            RemasterCitationView {
                origin: RemasterCitationOriginView::Foundry,
                attribution: "Migration 887".into(),
                revision: "pinned-revision".into(),
                statement: "Explicit replacement IDs".into(),
                shortened: false,
                destination: RemasterCitationDestinationView::External {
                    url: "https://github.com/foundryvtt/pf2e/blob/pinned-revision/src/migration.ts"
                        .into(),
                },
            },
            RemasterCitationView {
                origin: RemasterCitationOriginView::AtlasReviewed,
                attribution: "Atlas mapping".into(),
                revision: "atlas-review/v1".into(),
                statement: "Reviewed glossary context".into(),
                shortened: false,
                destination: RemasterCitationDestinationView::Attribution { reason: None },
            },
        ];
        RelatedCategoryView::Content {
            content: RemasterRelationshipsView {
                relationships: vec![RemasterRelationshipView {
                    id: "relationship-1".into(),
                    label: "Replaces".into(),
                    parts: vec![
                        RemasterInlinePartView::Text {
                            text: "Alpha, Beta, and ".into(),
                        },
                        RemasterInlinePartView::Reference {
                            label: "Current entry".into(),
                            navigation: Box::new(destination),
                        },
                    ],
                    declaration_ids: vec!["statement-1".into()],
                    explanation_links: vec![],
                    truncated: false,
                }],
                declarations: vec![RemasterDeclarationView {
                    id: "statement-1".into(),
                    kinds: vec![
                        RemasterChangeKindView::Replaced,
                        RemasterChangeKindView::MechanicsChanged,
                    ],
                    authored_status: Some("Replaced, altered mechanics".into()),
                    context: vec!["Class Features".into()],
                    before: vec![],
                    after: vec![],
                    origins: vec![
                        RemasterCitationOriginView::Foundry,
                        RemasterCitationOriginView::AtlasReviewed,
                    ],
                    citations,
                    citations_truncated: true,
                    truncated: true,
                }],
                truncated: true,
            },
        }
    }
    #[test]
    fn remaster_human_and_json_share_one_consolidated_line_and_separate_sources() {
        let category = fixture();
        let primary = lines(&category);
        assert!(
            primary
                .starts_with("Replaces: Alpha, Beta, and Current entry (items:bbbbbbbbbbbbbbbb)\n")
        );
        assert_eq!(primary.matches("Current entry").count(), 1);
        assert!(!primary.contains("Atlas"));
        assert!(!primary.contains("Replaced, altered mechanics"));
        let output = full(&category);
        for expected in [
            "Sources:",
            "Journal declaration",
            "journals:cccccccccccccccc — /text/content",
            "https://github.com/foundryvtt/pf2e/blob/pinned-revision/src/migration.ts",
            "Atlas mapping (atlas-review/v1)",
            "Additional evidence omitted",
        ] {
            assert!(output.contains(expected), "Missing {expected}: {output}");
        }
        let decoded: RelatedCategoryView<RemasterRelationshipsView> =
            serde_json::from_value(serde_json::to_value(&category).unwrap()).unwrap();
        assert_eq!(full(&decoded), output);
    }
    #[test]
    fn remaster_source_error_and_origin_survive_without_primary_provenance_badges() {
        let mut category = fixture();
        let RelatedCategoryView::Content { content } = &mut category else {
            panic!("fixture");
        };
        content.declarations[0].citations.clear();
        assert!(!lines(&category).contains("Atlas"));
        assert!(full(&category).contains("Atlas mapping (additional details omitted)"));
        let source=RelatedCategoryView::Content{content:RemasterSourceContextView{heading:"Source entry: Original glossary entry".into(),source:serde_json::from_value(json!({"label":"Original glossary entry","context":null,"resolution":{"kind":"absent","record_key":"items:aaaaaaaaaaaaaaaa","message":"Unavailable"}})).unwrap(),relationships:RelatedCategoryView::Error{error:AppError::new(AppErrorCode::InternalError,"Evidence could not be read")}}};
        let output = source_context(&source);
        assert!(output.starts_with("Source entry: Original glossary entry\n"));
        assert!(
            output.contains("Remaster information could not be read: Evidence could not be read")
        );
    }
    #[test]
    fn remaster_partial_citation_budget_retains_each_missing_source_origin() {
        let mut category = fixture();
        let RelatedCategoryView::Content { content } = &mut category else {
            panic!("fixture");
        };
        content.declarations[0]
            .citations
            .retain(|citation| citation.origin == RemasterCitationOriginView::Foundry);
        let output = full(&category);
        assert!(output.contains("Journal declaration"));
        assert!(output.contains("Atlas mapping (additional details omitted)"));
        assert_eq!(output.matches("Atlas mapping").count(), 1);
        assert!(!lines(&category).contains("Atlas"));
    }
}
