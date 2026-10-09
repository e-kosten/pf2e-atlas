use super::nodes::item_fields;
use super::{
    FieldAvailability, OwnedNodeFact, SourceBackedRecord, SourceFieldView, SourceNodeView,
    SourceRecordEnrichment, source_nodes,
};
use crate::source_content::{
    ContentAudience, ContentReferenceResolver, ContentVisibilityRule, LocalizationResolver,
    PreparedSourceContent, SourceContentLocator, prepare_plain_source_content,
    prepare_source_content,
};
use atlas_domain::RecordKey;
use atlas_foundry_model::{ActorSourcePF2e, FoundryDocumentSource};
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
    Unavailable(FieldAvailability),
    FormatUnavailable(FieldAvailability),
    UnsupportedFormat,
    PreparationFailed { message: String },
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceContentOutcome {
    pub locator: SourceContentLocator,
    pub role: SourceContentRole,
    pub format: Option<SourceContentFormat>,
    pub visibility: ContentVisibilityRule,
    pub visibility_availability: FieldAvailability,
    pub status: SourceContentStatus,
}

struct ContentSelection<'a> {
    field: &'static str,
    role: SourceContentRole,
    format: SourceFieldView<'a, SourceContentFormat>,
    visibility: SourceFieldView<'a, ContentVisibilityRule>,
    text: SourceFieldView<'a, &'a String>,
}

/// Consume the admitted source, preserving it unchanged. The resolver contains
/// identities only; a preparation error affects one field, never the root.
pub fn enrich_source_record(
    key: RecordKey,
    source: FoundryDocumentSource,
    audience: ContentAudience,
    localization: Option<&dyn LocalizationResolver>,
    references: Option<&super::SourceReferenceIndex>,
) -> SourceBackedRecord {
    let (nodes, collections) = source_nodes(&key, &source);
    let mut enrichment = SourceRecordEnrichment {
        audience,
        collections,
        owned_nodes: Vec::new(),
        content: Vec::new(),
        relationships: Vec::new(),
    };
    for entry in nodes {
        if !entry.owners.is_empty() {
            enrichment.owned_nodes.push(OwnedNodeFact {
                owners: entry.owners.clone(),
                order: entry.order,
                document_kind: entry.source.document_kind().into(),
                source_type: entry.source.source_type().value().map(str::to_string),
            });
        }
        for selection in select_content(entry.source) {
            let locator = SourceContentLocator {
                record: key.clone(),
                owners: entry.owners.clone(),
                field: selection.field.into(),
            };
            let format = selection.format.value();
            let visibility = selection
                .visibility
                .value()
                .unwrap_or(ContentVisibilityRule::None);
            let visibility_availability = selection.visibility.availability();
            let status = match (selection.text, selection.format) {
                (SourceFieldView::Value(text), SourceFieldView::Value(format)) => match format {
                    SourceContentFormat::Plain => SourceContentStatus::Prepared(Box::new(
                        prepare_plain_source_content(locator.clone(), text, audience, visibility),
                    )),
                    SourceContentFormat::Html => match prepare_source_content(
                        locator.clone(),
                        text,
                        audience,
                        visibility,
                        localization,
                        references.map(|r| r as &dyn ContentReferenceResolver),
                    ) {
                        Ok(prepared) => SourceContentStatus::Prepared(Box::new(prepared)),
                        Err(error) => SourceContentStatus::PreparationFailed {
                            message: error.to_string(),
                        },
                    },
                    SourceContentFormat::Markdown | SourceContentFormat::Unsupported => {
                        SourceContentStatus::UnsupportedFormat
                    }
                },
                (text, _) if !matches!(text, SourceFieldView::Value(_)) => {
                    SourceContentStatus::Unavailable(text.availability())
                }
                (_, format) => SourceContentStatus::FormatUnavailable(format.availability()),
            };
            enrichment.content.push(SourceContentOutcome {
                locator,
                role: selection.role,
                format,
                visibility,
                visibility_availability,
                status,
            });
        }
        super::relationships::collect_relationships(
            &key,
            &entry,
            references,
            &mut enrichment.relationships,
        );
    }
    SourceBackedRecord {
        key,
        source,
        enrichment,
    }
}

// Evidence: PF2e 6.12.4 Item.description and Actor family sheet enrichHTML
// callsites; core journal text format 1/2 and RollTable HTMLField declarations.
// Unknown/additional/patch/rule strings are intentionally not traversed.
fn select_content(source: SourceNodeView<'_>) -> Vec<ContentSelection<'_>> {
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
