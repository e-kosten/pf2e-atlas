//! Selected source presentation, using the same authored DTO for roots and children.
use crate::projection::{field_state, navigation_fingerprint};
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::*;
use atlas_domain::{QueryFieldState, RecordKey};
use atlas_record::source_content::{
    ContentAudience, ContentInteractionKind, ContentVisibilityRule, OwnedContentLocator,
    SourceContentLocator,
};
use atlas_record::source_record::{SourceContentFormat, SourceContentSelection, SourceFieldView};

pub(crate) fn prepared_fields(
    key: &RecordKey,
    owners: &[OwnedContentLocator],
    selections: &[SourceContentSelection<'_>],
    cache: &[atlas_search::SourcePreparedContent],
    audience: ContentAudience,
    fingerprint: &str,
) -> AppServiceResult<Vec<PreparedContentFieldView>> {
    let mut fields = Vec::new();
    for s in selections {
        let locator = SourceContentLocator {
            record: key.clone(),
            owners: owners.to_vec(),
            field: s.field.into(),
        };
        let visible = match s.visibility.value() {
            Some(ContentVisibilityRule::All) => true,
            Some(ContentVisibilityRule::Gm) => audience.include_gm,
            Some(ContentVisibilityRule::Owner) => audience.include_owner,
            _ => false,
        };
        if !visible {
            continue;
        }
        let body = match (s.text, s.format) {
            (SourceFieldView::Value(text), SourceFieldView::Value(SourceContentFormat::Plain)) => {
                PreparedFieldBodyView::Plain { text: text.clone() }
            }
            (SourceFieldView::Value(_), SourceFieldView::Value(SourceContentFormat::Html)) => {
                let cached = cache.iter().find(|c| c.locator == locator).ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::ArtifactIncompatible,
                        "selected rich field has no prepared outcome",
                    )
                })?;
                match &cached.html {
                    Some(html) => PreparedFieldBodyView::Html {
                        html: html.clone(),
                        controls: cached
                            .interactions
                            .iter()
                            .map(|i| ContentControlView {
                                ordinal: i.ordinal,
                                control: match &i.kind {
                                    ContentInteractionKind::Check { statistic, options } => {
                                        ContentControlKindView::Check {
                                            statistic: statistic.clone(),
                                            options: options.clone(),
                                        }
                                    }
                                    ContentInteractionKind::Damage { formula, options } => {
                                        ContentControlKindView::Damage {
                                            formula: formula.clone(),
                                            options: options.clone(),
                                        }
                                    }
                                    ContentInteractionKind::Command {
                                        command,
                                        arguments,
                                        options,
                                    } => ContentControlKindView::Command {
                                        command: command.clone(),
                                        arguments: arguments.clone(),
                                        options: options.clone(),
                                    },
                                    ContentInteractionKind::Template { shape, options } => {
                                        ContentControlKindView::Template {
                                            shape: shape.clone(),
                                            options: options.clone(),
                                        }
                                    }
                                },
                            })
                            .collect(),
                    },
                    None if cached.outcome == "empty" => PreparedFieldBodyView::Html {
                        html: String::new(),
                        controls: vec![],
                    },
                    None => PreparedFieldBodyView::Unavailable {
                        state: QueryFieldState::Invalid,
                    },
                }
            }
            (text, _) if !matches!(text, SourceFieldView::Value(_)) => {
                PreparedFieldBodyView::Unavailable {
                    state: field_state(text.availability()),
                }
            }
            (_, format) => PreparedFieldBodyView::Unavailable {
                state: if matches!(format, SourceFieldView::Value(_)) {
                    QueryFieldState::NotApplicable
                } else {
                    field_state(format.availability())
                },
            },
        };
        fields.push(PreparedContentFieldView {
            locator,
            role: format!("{:?}", s.role).to_lowercase(),
            source_fingerprint: navigation_fingerprint(owners, fingerprint),
            body,
        });
    }
    Ok(fields)
}
