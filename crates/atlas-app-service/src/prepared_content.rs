//! Selected source presentation, using the same authored DTO for roots and children.
use crate::projection::{content_location, field_state, navigation_fingerprint};
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::*;
use atlas_domain::OwnedContentLocator;
use atlas_domain::{QueryFieldState, RecordKey};
use atlas_record::source_content::{
    ContentAudience, ContentInteractionKind, ContentVisibilityRule, SourceContentLocator,
};
use atlas_record::source_record::{SourceContentFormat, SourceContentSelection, SourceFieldView};

pub(crate) struct RuntimeContentContext {
    pub actor_level: Option<i64>,
    pub badge_value: Option<i64>,
    pub binding: ContentRollEncounterContext,
}

/// One field namespace's values and the identities needed to recheck its dice.
pub(crate) struct PreparedContentContext<'a> {
    pub fingerprint: &'a str,
    pub retrieval: &'a atlas_search::AtlasRetrievalService,
    pub item_traits: Vec<String>,
    pub formula: atlas_record::source_content::FormulaContext,
    pub spell_rank: Option<u8>,
    pub encounter: Option<ContentRollEncounterContext>,
}
impl<'a> PreparedContentContext<'a> {
    pub fn source(
        retrieval: &'a atlas_search::AtlasRetrievalService,
        record: &atlas_record::source_record::SourceBackedRecord,
        owners: &[OwnedContentLocator],
        rank: Option<u8>,
    ) -> Self {
        use atlas_record::source_record::{ActorQueryView, ItemSourceView, SourceNodeView};
        let node = record.node_at(owners);
        let mut formula = node
            .map(atlas_record::source_record::source_formula_labels)
            .unwrap_or_default();
        formula.item_rank = rank;
        if let Some(SourceNodeView::Item(item)) = node {
            formula.item_level = item.item_level().value().and_then(|n| n.as_i64())
        }
        // Only the actor itself and its immediate owned items have this parent.
        if (owners.is_empty() || owners.len() == 1 && owners[0].collection == "/items")
            && let Some(root @ SourceNodeView::Actor(_)) = record.node_at(&[])
        {
            let actor = ActorQueryView { source: root };
            let level = actor.level().value().and_then(|n| n.as_i64());
            formula.actor_level = if let Some(variant) =
                crate::encounters::mechanics::initial_variant(record).0
            {
                level.map(|level| crate::encounters::mechanics::adjusted_npc_level(level, variant))
            } else if matches!(
                root,
                SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(_))
            ) {
                None
            } else {
                level
            };
        }
        if formula.item_is_spell
            && formula.item_rank.is_none()
            && let Some(SourceNodeView::Item(ItemSourceView::SpellSource(spell))) = node
        {
            let ranks = crate::presentation::spell_context::ranks(record, spell);
            if let [rank] = ranks.as_slice() {
                formula.item_rank = Some(*rank)
            }
        }
        let mut item_traits = match node {
            Some(SourceNodeView::Item(item)) => item.traits().value().unwrap_or(&[]).to_vec(),
            _ => vec![],
        };
        if let Some(SourceNodeView::Item(ItemSourceView::SpellSource(spell))) = node
            && let Some(rank) = formula.item_rank
            && let Ok(effective) = crate::presentation::spell_rank::heighten(spell, rank)
        {
            item_traits = ItemSourceView::SpellSource(&effective)
                .traits()
                .value()
                .unwrap_or(&[])
                .to_vec();
        }
        Self {
            fingerprint: retrieval.source_fingerprint(),
            retrieval,
            item_traits,
            spell_rank: formula.item_rank,
            formula,
            encounter: None,
        }
    }
}

pub(crate) fn prepared_fields(
    key: &RecordKey,
    owners: &[OwnedContentLocator],
    selections: &[SourceContentSelection<'_>],
    cache: &[atlas_search::SourcePreparedContent],
    bounded: &[SourceContentLocator],
    audience: ContentAudience,
    context: &PreparedContentContext<'_>,
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
                        routine_header: None,
                        locator: content_location(locator),
                        role: format!("{:?}", s.role).to_lowercase(),
                        source_fingerprint: navigation_fingerprint(owners, context.fingerprint),
                        section_rendering: None,
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
                        html: render_field_content(html, &cached.interactions, context),
                        controls: content_controls(
                            &cached.interactions,
                            Some((&locator, context)),
                            context.retrieval,
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
        // Section ordinals bind canonical prepared content, before contextual
        // substitutions change the displayed text. Render both copies with the
        // same field context after deriving those annotations.
        let section_rendering = match cache
            .iter()
            .find(|cached| cached.locator == locator)
            .and_then(|cached| cached.html.as_ref().map(|html| (cached, html)))
        {
            Some((cached, html)) if matches!(body, PreparedFieldBodyView::Html { .. }) => {
                contextual_section_rendering(html, &cached.interactions, &context.formula)?
            }
            _ => section_rendering(&body)?,
        };
        fields.push(PreparedContentFieldView {
            routine_header: None,
            section_rendering,
            locator: content_location(locator),
            role: format!("{:?}", s.role).to_lowercase(),
            source_fingerprint: navigation_fingerprint(owners, context.fingerprint),
            body,
        });
    }
    for field in &mut fields {
        if let (Some(rendering), Some(cached)) = (
            &mut field.section_rendering,
            cache.iter().find(|cached| {
                cached.locator.field == field.locator.field && cached.locator.owners == owners
            }),
        ) {
            rendering.html = render_field_content(&rendering.html, &cached.interactions, context);
        }
        project_routine_header(field, context.retrieval);
    }
    Ok(fields)
}

pub(crate) fn content_controls(
    interactions: &[atlas_record::source_content::ContentInteraction],
    source: Option<(&SourceContentLocator, &PreparedContentContext<'_>)>,
    retrieval: &atlas_search::AtlasRetrievalService,
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
                        details: check_details(
                            options,
                            source
                                .map(|(_, ctx)| ctx.item_traits.as_slice())
                                .unwrap_or(&[]),
                            retrieval,
                            matches!(
                                i.label,
                                atlas_record::source_content::ContentInteractionLabel::Check { .. }
                            ),
                        ),
                    }
                }
                ContentInteractionKind::Damage { formula, options } => {
                    let parts = source
                        .and_then(|(_, ctx)| {
                            atlas_record::source_content::resolved_contextual_damage(
                                formula,
                                &ctx.formula,
                            )
                        })
                        .or_else(|| atlas_record::source_content::literal_damage(formula));
                    let roll = source.and_then(|(locator, context)| {
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
                            source: content_location(locator.clone()),
                            source_fingerprint: context.fingerprint.to_owned(),
                            spell_rank: context.spell_rank,
                            encounter: context.encounter.clone(),
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
                    let literal = source
                        .and_then(|(_, ctx)| {
                            atlas_record::source_content::resolved_contextual_roll(
                                command,
                                arguments,
                                &ctx.formula,
                            )
                        })
                        .or_else(|| {
                            atlas_record::source_content::literal_content_roll(command, arguments)
                        });
                    let roll = source.and_then(|(locator, context)| {
                        literal.as_ref().filter(|roll| roll.has_dice)?;
                        Some(RollContentRequest {
                            content_interpretation_version:
                                atlas_record::source_content::CONTENT_INTERPRETATION_VERSION.into(),
                            source: content_location(locator.clone()),
                            source_fingerprint: context.fingerprint.to_owned(),
                            spell_rank: context.spell_rank,
                            encounter: context.encounter.clone(),
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

/// One bounded rendering copy per selected field. Selection policy remains in
/// atlas-record; the original body is never rewritten to carry view metadata.
pub(crate) fn contextual_section_rendering(
    html: &str,
    interactions: &[atlas_record::source_content::ContentInteraction],
    formula: &atlas_record::source_content::FormulaContext,
) -> AppServiceResult<Option<PreparedSectionRenderingView>> {
    let mut rendering = section_rendering(&PreparedFieldBodyView::Html {
        html: html.to_owned(),
        controls: vec![],
        inclusions: vec![],
    })?;
    if let Some(rendering) = &mut rendering {
        rendering.html = atlas_record::source_content::render_contextual_interactions(
            &rendering.html,
            interactions,
            formula,
        );
    }
    Ok(rendering)
}

fn section_rendering(
    body: &PreparedFieldBodyView,
) -> AppServiceResult<Option<PreparedSectionRenderingView>> {
    use atlas_record::source_record::{
        SOURCE_CONTENT_SELECTION_VERSION, section_annotated_prepared_html,
    };
    let html = match body {
        PreparedFieldBodyView::Html { html, .. } => {
            section_annotated_prepared_html(html).map_err(|error| {
                AppServiceError::new(AppErrorCode::ArtifactIncompatible, error.to_string())
            })?
        }
        PreparedFieldBodyView::Plain { text } => format!(
            "<span data-atlas-source-section=\"0\">{}</span>",
            atlas_record::source_content::escape_content_text(text)
        ),
        _ => return Ok(None),
    };
    Ok(Some(PreparedSectionRenderingView {
        html,
        selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
    }))
}

/// Rebuild the host namespace from its checked prepared row before composing
/// inclusions. A participant's values never reach an included foreign record.
pub(crate) fn resolve_runtime_content(
    r: &atlas_search::AtlasRetrievalService,
    source: &atlas_record::source_record::SourceBackedRecord,
    view: &mut RecordDetailView,
    runtime: RuntimeContentContext,
) -> AppServiceResult<()> {
    let needs_context = view
        .presentation
        .content
        .iter()
        .any(|field| match &field.body {
            PreparedFieldBodyView::Html { controls, .. } => {
                controls.iter().any(|control| match &control.control {
                    ContentControlKindView::Damage { formula, .. } => formula.contains('@'),
                    ContentControlKindView::Command { arguments, .. } => arguments.contains('@'),
                    ContentControlKindView::Check { options, .. } => {
                        options.values().any(|v| v.contains('@'))
                    }
                    ContentControlKindView::Template { .. } => false,
                })
            }
            _ => false,
        });
    if !needs_context {
        return Ok(());
    }
    let mut context =
        PreparedContentContext::source(r, source, &view.selected.owners, view.selected.spell_rank);
    if context.formula.item_is_spell {
        context.formula.item_rank = view.selected.spell_rank;
        context.spell_rank = view.selected.spell_rank;
    }
    context.formula.actor_level = runtime.actor_level;
    context.formula.badge_value = runtime.badge_value;
    context.encounter = Some(runtime.binding);
    let locators = view
        .presentation
        .content
        .iter()
        .filter(|f| matches!(f.body, PreparedFieldBodyView::Html { .. }))
        .map(|f| SourceContentLocator {
            record: f.locator.record.clone(),
            owners: f.locator.owners.clone(),
            field: f.locator.field.clone(),
        })
        .collect::<Vec<_>>();
    let content = r.read_attributed_content(&locators)?;
    for field in &mut view.presentation.content {
        if let PreparedFieldBodyView::Html {
            html,
            controls,
            inclusions,
        } = &mut field.body
            && let Some(cached) = content.content.fields.iter().find(|c| {
                c.locator.record == field.locator.record
                    && c.locator.owners == field.locator.owners
                    && c.locator.field == field.locator.field
            })
            && let Some(authored) = &cached.html
        {
            let marked;
            let authored = if let Some(address) = view.selected.passage.as_ref()
                && view.selected.field.as_deref() == Some(field.locator.field.as_str())
            {
                marked = atlas_record::source_record::mark_source_html_passage(authored, address)
                    .map_err(|e| {
                    AppServiceError::invalid_request(format!("stale or invalid passage: {e}"))
                })?;
                marked.as_str()
            } else {
                authored
            };
            *html = render_field_content(authored, &cached.interactions, &context);
            field.section_rendering =
                contextual_section_rendering(authored, &cached.interactions, &context.formula)?;
            *controls =
                content_controls(&cached.interactions, Some((&cached.locator, &context)), r);
            inclusions.clear();
            if let Some(rendering) = &mut field.section_rendering {
                rendering.html =
                    render_field_content(&rendering.html, &cached.interactions, &context);
            }
            project_routine_header(field, r);
        }
    }
    crate::inline_content::compose_inline_content(
        r,
        view.presentation.content.iter_mut(),
        &content.relationships,
    )
}

#[cfg(test)]
mod section_rendering_tests {
    use super::*;

    #[test]
    fn contextual_copy_retains_source_section_and_passage_markers() {
        use atlas_record::source_content::{ContentInteraction, ContentInteractionLabel};
        let canonical = "<h2>Breath</h2><p data-atlas-passage-target=\"true\">Deals <span data-atlas-interaction=\"0\">unresolved damage</span>.</p>";
        let interactions = [ContentInteraction {
            ordinal: 0,
            path: "/system/description/value".into(),
            kind: ContentInteractionKind::Damage {
                formula: "@actor.level[fire]".into(),
                options: Default::default(),
            },
            label: ContentInteractionLabel::Damage,
        }];
        let rendering = contextual_section_rendering(
            canonical,
            &interactions,
            &atlas_record::source_content::FormulaContext {
                actor_level: Some(7),
                ..Default::default()
            },
        )
        .unwrap()
        .unwrap();
        assert!(rendering.html.contains("7 fire damage"));
        assert!(rendering.html.contains("data-atlas-source-section=\"0\""));
        assert!(
            rendering
                .html
                .contains("data-atlas-passage-target=\"true\"")
        );
        assert!(canonical.contains("unresolved damage"));
    }

    #[test]
    fn plain_rendering_escapes_original_text_and_unavailable_has_no_metadata() {
        let body = PreparedFieldBodyView::Plain {
            text: "<rules> & facts".into(),
        };
        let rendering = section_rendering(&body).unwrap().unwrap();
        assert_eq!(
            rendering.html,
            "<span data-atlas-source-section=\"0\">&lt;rules&gt; &amp; facts</span>"
        );
        assert_eq!(
            body,
            PreparedFieldBodyView::Plain {
                text: "<rules> & facts".into()
            }
        );
        assert!(
            section_rendering(&PreparedFieldBodyView::Bounded {
                reason: "bounded".into()
            })
            .unwrap()
            .is_none()
        );
    }
}

fn trait_badges(
    traits: &[String],
    retrieval: &atlas_search::AtlasRetrievalService,
) -> Vec<RecordBadgeView> {
    traits
        .iter()
        .map(|value| RecordBadgeView {
            kind: "trait".into(),
            value: value.clone(),
            label: retrieval.trait_label(value).unwrap_or(value).to_owned(),
            trait_key: retrieval
                .trait_label(value)
                .and_then(|_| atlas_domain::TraitKey::new(value.clone()).ok())
                .map(|key| key.to_string()),
        })
        .collect()
}
fn check_details(
    options: &std::collections::BTreeMap<String, String>,
    item_traits: &[String],
    retrieval: &atlas_search::AtlasRetrievalService,
    generated_check_label: bool,
) -> Option<CheckDetailsView> {
    let traits = atlas_record::source_content::check_traits(options, item_traits);
    (!traits.is_empty() && (!generated_check_label || traits.iter().any(|t| t != "secret"))).then(
        || CheckDetailsView {
            traits: trait_badges(&traits, retrieval),
        },
    )
}

pub(crate) fn project_routine_header(
    field: &mut PreparedContentFieldView,
    retrieval: &atlas_search::AtlasRetrievalService,
) {
    if field.role != "routine" {
        return;
    }
    let PreparedFieldBodyView::Html { html, .. } = &mut field.body else {
        return;
    };
    let Some(header) = atlas_record::source_content::routine_header(html) else {
        return;
    };
    if let Some(rendering) = &mut field.section_rendering {
        // Derive ordinals first, then remove exactly the same supported prefix.
        let Some(section) = atlas_record::source_content::routine_header(&rendering.html) else {
            return;
        };
        rendering.html = section.html;
    }
    *html = header.html;
    field.routine_header = Some(RoutineHeaderView {
        actions: header.actions,
        traits: trait_badges(&header.traits, retrieval),
    });
}

/// Inherited traits affect generated check labels (notably secret), while
/// control options retain their authored values and custom labels remain intact.
pub(crate) fn render_field_content(
    html: &str,
    interactions: &[atlas_record::source_content::ContentInteraction],
    context: &PreparedContentContext<'_>,
) -> String {
    let mut effective = interactions.to_vec();
    for interaction in &mut effective {
        if let ContentInteractionKind::Check { options, .. } = &mut interaction.kind {
            let traits = atlas_record::source_content::check_traits(options, &context.item_traits);
            options.remove("overrideTraits");
            options.insert("traits".into(), traits.join(","));
        }
    }
    atlas_record::source_content::render_contextual_interactions(html, &effective, &context.formula)
}
