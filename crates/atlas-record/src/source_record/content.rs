use super::nodes::item_fields;
use super::nodes::source_nodes;
use super::{FieldAvailability, SourceBackedRecord, SourceFieldView, SourceNodeView};
use crate::source_content::{
    ContentAudience, ContentReferenceResolver, ContentVisibilityRule, LocalizationResolver,
    PreparedSourceContent, SourceContentLocator, prepare_source_content,
};
use atlas_foundry_model::ActorSourcePF2e;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceContentRole {
    Name,
    Description,
    Notes,
    Routine,
    Disable,
    Reset,
    Stealth,
    Biography,
    Caption,
    TableResult,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceContentFormat {
    Html,
    Plain,
    Markdown,
    Unsupported,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceContentStatus {
    Prepared(Box<PreparedSourceContent>),
    FormatUnavailable {
        locator: SourceContentLocator,
        availability: FieldAvailability,
    },
    UnsupportedFormat {
        locator: SourceContentLocator,
        format: SourceContentFormat,
    },
    PreparationFailed {
        locator: SourceContentLocator,
        message: String,
    },
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceContentOutcome {
    pub role: SourceContentRole,
    pub visibility: ContentVisibilityRule,
    pub visibility_availability: FieldAvailability,
    pub status: SourceContentStatus,
}

impl SourceContentOutcome {
    /// Each result retains its field address once, including failed preparation.
    pub fn locator(&self) -> &SourceContentLocator {
        match &self.status {
            SourceContentStatus::Prepared(prepared) => &prepared.locator,
            SourceContentStatus::FormatUnavailable { locator, .. }
            | SourceContentStatus::UnsupportedFormat { locator, .. }
            | SourceContentStatus::PreparationFailed { locator, .. } => locator,
        }
    }
}

pub(super) struct ContentSelection<'a> {
    pub field: &'static str,
    pub role: SourceContentRole,
    pub format: SourceFieldView<'a, SourceContentFormat>,
    pub visibility: SourceFieldView<'a, ContentVisibilityRule>,
    pub text: SourceFieldView<'a, &'a String>,
}

/// Prepare the complete declared prose selection without changing its source.
/// Every selected present rich field yields a result, including empty/hidden text
/// and unsupported formats. Source-unavailable fields have no output row; their
/// presence and admission diagnostics remain in the DTO. Plain fields are borrowed
/// by text_sources instead. This is not an audience-safe application response.
///
/// Results are specific to this source snapshot and the supplied audience,
/// localization and reference context. Callers retain those identities when
/// storing/reusing results; a RecordKey alone cannot identify a prepared cache.
pub fn prepare_record_content(
    record: &SourceBackedRecord,
    audience: ContentAudience,
    localization: Option<&dyn LocalizationResolver>,
    references: Option<&super::SourceReferenceIndex>,
) -> Vec<SourceContentOutcome> {
    let mut output = Vec::new();
    for entry in source_nodes(&record.source) {
        for selection in select_content(entry.source) {
            let SourceFieldView::Value(text) = selection.text else {
                continue;
            };
            if matches!(
                selection.format,
                SourceFieldView::Value(SourceContentFormat::Plain)
            ) {
                continue;
            }
            let locator = SourceContentLocator {
                record: record.key.clone(),
                owners: entry.owners.clone(),
                field: selection.field.into(),
            };
            let visibility = selection
                .visibility
                .value()
                .unwrap_or(ContentVisibilityRule::None);
            let visibility_availability = selection.visibility.availability();
            let status = match selection.format {
                SourceFieldView::Value(SourceContentFormat::Html) => match prepare_source_content(
                    locator.clone(),
                    text,
                    audience,
                    visibility,
                    localization,
                    references.map(|r| r as &dyn ContentReferenceResolver),
                ) {
                    Ok(prepared) => SourceContentStatus::Prepared(Box::new(prepared)),
                    Err(error) => SourceContentStatus::PreparationFailed {
                        locator,
                        message: error.to_string(),
                    },
                },
                SourceFieldView::Value(format) => {
                    SourceContentStatus::UnsupportedFormat { locator, format }
                }
                format => SourceContentStatus::FormatUnavailable {
                    locator,
                    availability: format.availability(),
                },
            };
            output.push(SourceContentOutcome {
                role: selection.role,
                visibility,
                visibility_availability,
                status,
            });
        }
    }
    output
}

// Evidence: PF2e 6.12.4 Item.description and Actor family sheet enrichHTML
// callsites; core journal text format 1/2 and RollTable HTMLField declarations.
// Unknown/additional/patch/rule strings are intentionally not traversed.
pub(super) fn select_content(source: SourceNodeView<'_>) -> Vec<ContentSelection<'_>> {
    use ContentVisibilityRule::{All, Gm, Owner};
    use SourceContentFormat::{Html, Plain};
    use SourceContentRole::*;
    let mut out = Vec::new();
    let mut add = |field, role, format, visibility, text| {
        out.push(ContentSelection {
            field,
            role,
            format: SourceFieldView::Value(format),
            visibility: SourceFieldView::Value(visibility),
            text,
        })
    };
    if !matches!(source, SourceNodeView::Result(_)) {
        add("/name", Name, Plain, All, source.name());
    }
    match source {
        SourceNodeView::Item(item) => {
            let description = item_fields!(
                item,
                s,
                SourceFieldView::from(&s.system).and_then(|s| (&s.description).into())
            );
            add(
                "/system/description/value",
                Description,
                Html,
                All,
                description.and_then(|d| (&d.value).into()),
            );
            add(
                "/system/description/gm",
                Description,
                Html,
                Gm,
                description.and_then(|d| (&d.gm).into()),
            );
        }
        SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
            let details = SourceFieldView::from(&s.system).and_then(|s| (&s.details).into());
            add(
                "/system/details/publicNotes",
                Notes,
                Html,
                All,
                details.and_then(|d| (&d.public_notes).into()),
            );
            add(
                "/system/details/privateNotes",
                Notes,
                Html,
                Gm,
                details.and_then(|d| (&d.private_notes).into()),
            );
            add(
                "/system/details/blurb",
                Notes,
                Plain,
                All,
                details.and_then(|d| (&d.blurb).into()),
            );
        }
        SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
            let system = SourceFieldView::from(&s.system);
            let details = system.and_then(|s| (&s.details).into());
            add(
                "/system/details/description",
                Description,
                Html,
                All,
                details.and_then(|d| (&d.description).into()),
            );
            add(
                "/system/details/disable",
                Disable,
                Html,
                All,
                details.and_then(|d| (&d.disable).into()),
            );
            add(
                "/system/details/routine",
                Routine,
                Html,
                All,
                details.and_then(|d| (&d.routine).into()),
            );
            add(
                "/system/details/reset",
                Reset,
                Html,
                All,
                details.and_then(|d| (&d.reset).into()),
            );
            add(
                "/system/attributes/stealth/details",
                Stealth,
                Html,
                All,
                system
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|s| (&s.stealth).into())
                    .and_then(|s| (&s.details).into()),
            );
        }
        SourceNodeView::Actor(ActorSourcePF2e::ArmySource(s)) => add(
            "/system/details/description",
            Description,
            Html,
            All,
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.description).into()),
        ),
        SourceNodeView::Actor(ActorSourcePF2e::LootSource(s)) => add(
            "/system/details/description",
            Description,
            Html,
            All,
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.description).into()),
        ),
        // Vehicle description uses the HTML editor helper in the pinned vehicle
        // description template. Party description is an explicit HTMLField.
        SourceNodeView::Actor(ActorSourcePF2e::VehicleSource(s)) => add(
            "/system/details/description",
            Description,
            Html,
            All,
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.description).into()),
        ),
        SourceNodeView::Actor(ActorSourcePF2e::PartySource(s)) => add(
            "/system/details/description",
            Description,
            Html,
            All,
            SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.description).into()),
        ),
        SourceNodeView::Actor(ActorSourcePF2e::CharacterSource(s)) => {
            let biography = SourceFieldView::from(&s.system)
                .and_then(|s| (&s.details).into())
                .and_then(|s| (&s.biography).into());
            let visibility = biography.and_then(|s| (&s.visibility).into());
            // Visibility flags govern authored sections; unavailable flags do
            // not silently make private biography text public.
            for (field, text, visible) in [
                (
                    "/system/details/biography/appearance",
                    biography.and_then(|s| (&s.appearance).into()),
                    visibility
                        .and_then(|s| (&s.appearance).into())
                        .map(|v| if *v { All } else { Owner }),
                ),
                (
                    "/system/details/biography/backstory",
                    biography.and_then(|s| (&s.backstory).into()),
                    visibility
                        .and_then(|s| (&s.backstory).into())
                        .map(|v| if *v { All } else { Owner }),
                ),
                (
                    "/system/details/biography/campaignNotes",
                    biography.and_then(|s| (&s.campaign_notes).into()),
                    visibility
                        .and_then(|s| (&s.campaign).into())
                        .map(|v| if *v { All } else { Owner }),
                ),
                (
                    "/system/details/biography/allies",
                    biography.and_then(|s| (&s.allies).into()),
                    visibility
                        .and_then(|s| (&s.campaign).into())
                        .map(|v| if *v { All } else { Owner }),
                ),
                (
                    "/system/details/biography/enemies",
                    biography.and_then(|s| (&s.enemies).into()),
                    visibility
                        .and_then(|s| (&s.campaign).into())
                        .map(|v| if *v { All } else { Owner }),
                ),
                (
                    "/system/details/biography/organizations",
                    biography.and_then(|s| (&s.organizations).into()),
                    visibility
                        .and_then(|s| (&s.campaign).into())
                        .map(|v| if *v { All } else { Owner }),
                ),
            ] {
                out.push(ContentSelection {
                    field,
                    role: Biography,
                    format: SourceFieldView::Value(Html),
                    visibility: visible,
                    text,
                });
            }
            for (field, text, visible) in [
                (
                    "/system/details/biography/birthPlace",
                    biography.and_then(|s| (&s.birth_place).into()),
                    visibility.and_then(|s| (&s.backstory).into()),
                ),
                (
                    "/system/details/biography/attitude",
                    biography.and_then(|s| (&s.attitude).into()),
                    visibility.and_then(|s| (&s.personality).into()),
                ),
                (
                    "/system/details/biography/beliefs",
                    biography.and_then(|s| (&s.beliefs).into()),
                    visibility.and_then(|s| (&s.personality).into()),
                ),
                (
                    "/system/details/biography/likes",
                    biography.and_then(|s| (&s.likes).into()),
                    visibility.and_then(|s| (&s.personality).into()),
                ),
                (
                    "/system/details/biography/dislikes",
                    biography.and_then(|s| (&s.dislikes).into()),
                    visibility.and_then(|s| (&s.personality).into()),
                ),
                (
                    "/system/details/biography/catchphrases",
                    biography.and_then(|s| (&s.catchphrases).into()),
                    visibility.and_then(|s| (&s.personality).into()),
                ),
            ] {
                out.push(ContentSelection {
                    field,
                    role: Biography,
                    format: SourceFieldView::Value(Plain),
                    visibility: visible.map(|v| if *v { All } else { Owner }),
                    text,
                });
            }
        }
        SourceNodeView::JournalPage(s) => {
            let text = SourceFieldView::from(&s.text);
            let format = SourceFieldView::from(&s.r#type).and_then(|kind| {
                if kind == "text" {
                    text.and_then(|t| (&t.format).into())
                        .map(|n| match n.as_f64() {
                            Some(1.0) => Html,
                            Some(2.0) => SourceContentFormat::Markdown,
                            _ => SourceContentFormat::Unsupported,
                        })
                } else {
                    SourceFieldView::NotApplicable
                }
            });
            // Only authored HTML text pages are interpreted. Markdown remains
            // explicit unsupported content, including an empty authored body.
            if !matches!(format, SourceFieldView::NotApplicable) {
                let markdown = matches!(
                    format,
                    SourceFieldView::Value(SourceContentFormat::Markdown)
                );
                out.push(ContentSelection {
                    field: if markdown {
                        "/text/markdown"
                    } else {
                        "/text/content"
                    },
                    role: Description,
                    format,
                    visibility: SourceFieldView::Value(All),
                    text: if markdown {
                        text.and_then(|t| (&t.markdown).into())
                    } else {
                        text.and_then(|t| (&t.content).into())
                    },
                });
            }
            out.push(ContentSelection {
                field: "/image/caption",
                role: Caption,
                format: SourceFieldView::Value(Plain),
                visibility: SourceFieldView::Value(All),
                text: SourceFieldView::from(&s.image).and_then(|i| (&i.caption).into()),
            });
        }
        SourceNodeView::Table(s) => add(
            "/description",
            Description,
            Html,
            All,
            (&s.description).into(),
        ),
        SourceNodeView::Result(s) => add("/text", TableResult, Html, All, (&s.text).into()),
        SourceNodeView::Actor(ActorSourcePF2e::FamiliarSource(_))
        | SourceNodeView::Journal(_)
        | SourceNodeView::Macro(_) => {}
    }
    out
}
