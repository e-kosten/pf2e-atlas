//! Selected source presentation, using the same authored DTO for roots and children.
use crate::projection::{field_state, navigation_fingerprint};
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::*;
use atlas_domain::OwnedContentLocator;
use atlas_domain::{QueryFieldState, RecordKey};
use atlas_record::source_content::{
    ContentAudience, ContentInteractionKind, ContentVisibilityRule, SourceContentLocator,
};
use atlas_record::source_record::{SourceContentFormat, SourceContentSelection, SourceFieldView};

pub(crate) fn prepared_fields(
    key: &RecordKey,
    owners: &[OwnedContentLocator],
    selections: &[SourceContentSelection<'_>],
    cache: &[atlas_search::SourcePreparedContent],
    bounded: &[SourceContentLocator],
    audience: ContentAudience,
    fingerprint: &str,
) -> AppServiceResult<Vec<PreparedContentFieldView>> {
    let mut fields = Vec::new();
    for s in selections {
        let locator = SourceContentLocator {
            record: key.clone(),
            owners: owners.to_vec(),
            field: s.field.to_string(),
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
                if bounded.contains(&locator) {
                    fields.push(PreparedContentFieldView {
                        locator: locator.into(),
                        role: format!("{:?}", s.role).to_lowercase(),
                        source_fingerprint: navigation_fingerprint(owners, fingerprint),
                        body: PreparedFieldBodyView::Bounded {
                            reason: "This field exceeds the reference-link reading limit.".into(),
                        },
                    });
                    continue;
                }
                let cached = cache.iter().find(|c| c.locator == locator).ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::ArtifactIncompatible,
                        "selected rich field has no prepared outcome",
                    )
                })?;
                match &cached.html {
                    Some(html) => PreparedFieldBodyView::Html {
                        inclusions: vec![],
                        html: html.clone(),
                        controls: content_controls(
                            &cached.interactions,
                            Some((&locator, fingerprint)),
                        ),
                    },
                    None if cached.outcome == "empty" => PreparedFieldBodyView::Html {
                        inclusions: vec![],
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
            locator: locator.into(),
            role: format!("{:?}", s.role).to_lowercase(),
            source_fingerprint: navigation_fingerprint(owners, fingerprint),
            body,
        });
    }
    Ok(fields)
}

pub(crate) fn content_controls(
    interactions: &[atlas_record::source_content::ContentInteraction],
    source: Option<(&SourceContentLocator, &str)>,
) -> Vec<ContentControlView> {
    interactions
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
                    let parts = atlas_record::source_content::literal_damage(formula);
                    let roll = source.and_then(|(locator, fingerprint)| {
                        let parts = parts.as_ref()?;
                        (parts.iter().any(|p| p.has_dice)
                            && options.keys().all(|key| {
                                matches!(
                                    key.as_str(),
                                    "name"
                                        | "shortLabel"
                                        | "immutable"
                                        | "overrideTraits"
                                        | "traits"
                                        | "domains"
                                        | "options"
                                )
                            })
                            && options
                                .values()
                                .all(|value| !value.contains('@') && !value.contains("resolve(")))
                        .then(|| RollContentRequest {
                            content_interpretation_version:
                                atlas_record::source_content::CONTENT_INTERPRETATION_VERSION.into(),
                            source: locator.clone().into(),
                            source_fingerprint: fingerprint.to_owned(),
                            ordinal: i.ordinal,
                            authored_formula: formula.clone(),
                            authored_command: None,
                        })
                    });
                    ContentControlKindView::Damage {
                        formula: formula.clone(),
                        options: options.clone(),
                        literal: parts.map(super::content_roll_presentation::damage_presentation),
                        roll,
                    }
                }
                ContentInteractionKind::Command {
                    command,
                    arguments,
                    options,
                } => {
                    let literal =
                        atlas_record::source_content::literal_content_roll(command, arguments);
                    let roll = source.and_then(|(locator, fingerprint)| {
                        literal.as_ref().filter(|roll| roll.has_dice)?;
                        Some(RollContentRequest {
                            content_interpretation_version:
                                atlas_record::source_content::CONTENT_INTERPRETATION_VERSION.into(),
                            source: locator.clone().into(),
                            source_fingerprint: fingerprint.to_owned(),
                            ordinal: i.ordinal,
                            authored_formula: arguments.clone(),
                            authored_command: Some(command.clone()),
                        })
                    });
                    ContentControlKindView::Command {
                        command: command.clone(),
                        arguments: arguments.clone(),
                        options: options.clone(),
                        literal: literal
                            .map(super::content_roll_presentation::command_presentation),
                        roll,
                    }
                }
                ContentInteractionKind::Template { shape, options } => {
                    ContentControlKindView::Template {
                        shape: shape.clone(),
                        options: options.clone(),
                    }
                }
            },
        })
        .collect()
}
