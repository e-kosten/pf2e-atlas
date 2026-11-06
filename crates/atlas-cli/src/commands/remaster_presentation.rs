//! Terminal formatting of Rust-projected change information.
use atlas_app_model::{
    RelatedCategoryView, RemasterCitationDestinationView, RemasterEndpointResolutionView,
    RemasterRelationshipsView, RemasterSourceContextView,
};
use std::fmt::Write;

pub(crate) fn notices(category: &RelatedCategoryView<RemasterRelationshipsView>) -> String {
    let mut out = String::new();
    match category {
        RelatedCategoryView::Content { content } => {
            for notice in &content.notices {
                let _ = writeln!(out, "{}", notice.heading);
                if let Some(description) = &notice.description {
                    let _ = writeln!(out, "  {description}");
                }
                for link in &notice.links {
                    let _ = writeln!(out, "  {} — {}", link.label, link.navigation.record_key);
                }
                if let Some(attribution) = &notice.attribution {
                    let _ = writeln!(out, "  {attribution}");
                }
            }
            if content.truncated {
                let _ = writeln!(out, "More remaster information is available in the source.");
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
            format!("{}\n{}", content.heading, notices(&content.relationships))
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
        return match category {
            RelatedCategoryView::Empty => "No source-backed remaster relationships found.\n".into(),
            _ => notices(category),
        };
    };
    let mut out = String::new();
    for declaration in &content.declarations {
        let _ = writeln!(out, "{}", declaration.heading);
        if let Some(summary) = &declaration.summary {
            let _ = writeln!(out, "  {summary}");
        }
        if !declaration.context.is_empty() {
            let _ = writeln!(out, "  {}", declaration.context.join(" · "));
        }
        if let Some(status) = &declaration.authored_status {
            let _ = writeln!(out, "  Source status: {status}");
        }
        for (heading, sides) in [
            ("Before", &declaration.before),
            ("After", &declaration.after),
        ] {
            for side in sides {
                let _ = writeln!(out, "  {heading}: {}", side.text);
                for endpoint in &side.endpoints {
                    let label = &endpoint.label;
                    match &endpoint.resolution {
                        RemasterEndpointResolutionView::Current { record, navigation } => {
                            let edition = record.edition.map(|edition| match edition {
                                atlas_app_model::RecordEditionView::Legacy => "Legacy",
                                atlas_app_model::RecordEditionView::Remaster => "Remaster",
                            });
                            let _ = writeln!(
                                out,
                                "    {label}: {} — {}{}{}",
                                record.title,
                                navigation.record_key,
                                edition
                                    .map(|value| format!(" · {value}"))
                                    .unwrap_or_default(),
                                record
                                    .publication
                                    .as_ref()
                                    .map(|value| format!(" · {value}"))
                                    .unwrap_or_default(),
                            );
                        }
                        RemasterEndpointResolutionView::Absent {
                            record_key,
                            message,
                        } => {
                            let _ = writeln!(out, "    {label}: {record_key} — {message}");
                        }
                        RemasterEndpointResolutionView::Unresolved { message, .. } => {
                            let _ = writeln!(out, "    {label}: {message}");
                        }
                        RemasterEndpointResolutionView::Ambiguous {
                            message,
                            candidates,
                            truncated,
                        } => {
                            let _ = writeln!(out, "    {label}: {message}");
                            for candidate in candidates {
                                let _ = writeln!(
                                    out,
                                    "      {} — {}",
                                    candidate.record.title, candidate.navigation.record_key
                                );
                            }
                            if *truncated {
                                let _ = writeln!(out, "      Additional possible records omitted.");
                            }
                        }
                        RemasterEndpointResolutionView::Term { .. }
                        | RemasterEndpointResolutionView::SourceText => {}
                    }
                }
                if side.truncated {
                    let _ = writeln!(
                        out,
                        "    Additional source details omitted; inspect the source."
                    );
                }
            }
        }
        for citation in &declaration.citations {
            let _ = writeln!(
                out,
                "  Source: {} ({})",
                citation.attribution, citation.revision
            );
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
        if declaration.truncated || declaration.citations_truncated {
            let _ = writeln!(out, "  Additional evidence omitted; inspect the source.");
        }
        let _ = writeln!(out);
    }
    if content.truncated {
        let _ = writeln!(
            out,
            "Additional remaster information omitted; inspect the source."
        );
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
        let current:RecordSummaryView=serde_json::from_value(json!({"record_key":"items:bbbbbbbbbbbbbbbb","title":"Current entry","kind":"action","kind_label":"Action","traits":[],"edition":"remaster","publication":"Player Core"})).unwrap();
        let destination = navigation(&current.record_key);
        let absent = RemasterEndpointView {
            label: "Alpha".into(),
            context: None,
            resolution: RemasterEndpointResolutionView::Absent {
                record_key: "items:aaaaaaaaaaaaaaaa".into(),
                message: "Identified record is unavailable".into(),
            },
        };
        let unresolved = RemasterEndpointView {
            label: "Beta".into(),
            context: None,
            resolution: RemasterEndpointResolutionView::Unresolved {
                reference: None,
                message: "No historical identity is established".into(),
            },
        };
        let ambiguous = RemasterEndpointView {
            label: "Gamma".into(),
            context: None,
            resolution: RemasterEndpointResolutionView::Ambiguous {
                candidates: vec![RemasterEndpointCandidateView {
                    record: current.clone(),
                    navigation: destination.clone(),
                }],
                truncated: true,
                message: "Possible current records; none selected".into(),
            },
        };
        let current = RemasterEndpointView {
            label: "Name quoted by the source".into(),
            context: None,
            resolution: RemasterEndpointResolutionView::Current {
                record: current,
                navigation: destination.clone(),
            },
        };
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
                attribution: "Atlas-reviewed — Exact glossary correspondence".into(),
                revision: "atlas-review/v1".into(),
                statement: "Reviewed glossary context".into(),
                shortened: false,
                destination: RemasterCitationDestinationView::Attribution { reason: None },
            },
        ];
        RelatedCategoryView::Content {
            content: RemasterRelationshipsView {
                notices: vec![RemasterNoticeView {
                    declaration_id: "statement-1".into(),
                    heading: "Replacement options".into(),
                    description: Some("Source groups these options.".into()),
                    links: vec![RemasterNoticeLinkView {
                        label: "Current entry — Player Core".into(),
                        navigation: destination,
                    }],
                    attribution: Some("Atlas-reviewed correspondence".into()),
                }],
                declarations: vec![RemasterDeclarationView {
                    id: "statement-1".into(),
                    heading: "Replacement options".into(),
                    summary: Some("Source groups these options.".into()),
                    kinds: vec![
                        RemasterChangeKindView::Replaced,
                        RemasterChangeKindView::MechanicsChanged,
                    ],
                    authored_status: Some("Replaced, altered mechanics".into()),
                    context: vec!["Class Features".into(), "Fixture class".into()],
                    before: vec![RemasterSideView {
                        text: "Alpha + Beta + Gamma".into(),
                        terms: vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
                        endpoints: vec![absent, unresolved, ambiguous],
                        truncated: true,
                    }],
                    after: vec![RemasterSideView {
                        text: "Current option".into(),
                        terms: vec!["Current option".into()],
                        endpoints: vec![current],
                        truncated: false,
                    }],
                    citations,
                    citations_truncated: true,
                    truncated: true,
                }],
                truncated: true,
            },
        }
    }
    #[test]
    fn remaster_human_output_retains_grouped_context_exact_identity_and_all_source_destinations() {
        let category = fixture();
        let output = full(&category);
        for expected in [
            "Replacement options",
            "Class Features · Fixture class",
            "Source status: Replaced, altered mechanics",
            "Before: Alpha + Beta + Gamma",
            "Alpha: items:aaaaaaaaaaaaaaaa — Identified record is unavailable",
            "Beta: No historical identity is established",
            "Gamma: Possible current records; none selected",
            "Current entry — items:bbbbbbbbbbbbbbbb · Remaster · Player Core",
            "journals:cccccccccccccccc — /text/content",
            "https://github.com/foundryvtt/pf2e/blob/pinned-revision/src/migration.ts",
            "Atlas-reviewed — Exact glossary correspondence (atlas-review/v1)",
            "Additional evidence omitted",
            "Additional remaster information omitted",
        ] {
            assert!(output.contains(expected), "Missing {expected}: {output}");
        }
        let wire = serde_json::to_value(&category).unwrap();
        let decoded: RelatedCategoryView<RemasterRelationshipsView> =
            serde_json::from_value(wire).unwrap();
        assert_eq!(
            full(&decoded),
            output,
            "human and JSON formatting consume the same projected facts"
        );
        let main = notices(&category);
        assert!(main.contains("Current entry — Player Core — items:bbbbbbbbbbbbbbbb"));
        assert!(main.contains("Atlas-reviewed correspondence"));
    }
    #[test]
    fn remaster_source_error_keeps_known_source_context_and_local_recovery_text() {
        let source = RelatedCategoryView::Content {
            content: RemasterSourceContextView {
                heading: "Source entry: Original glossary entry".into(),
                source: RemasterEndpointView {
                    label: "Original glossary entry".into(),
                    context: None,
                    resolution: RemasterEndpointResolutionView::Absent {
                        record_key: "items:aaaaaaaaaaaaaaaa".into(),
                        message: "Unavailable".into(),
                    },
                },
                relationships: RelatedCategoryView::Error {
                    error: AppError::new(AppErrorCode::InternalError, "Evidence could not be read"),
                },
            },
        };
        let output = source_context(&source);
        assert!(output.starts_with("Source entry: Original glossary entry\n"));
        assert!(
            output.contains("Remaster information could not be read: Evidence could not be read")
        );
        assert!(!output.contains("No source-backed remaster relationships found"));
    }
}
