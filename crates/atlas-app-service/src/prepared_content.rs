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

pub(crate) struct RuntimeContentContext {
    pub actor_level: Option<i64>,
    pub badge_value: Option<i64>,
    pub binding: ContentRollEncounterContext,
}

/// One field namespace's values and the identities needed to recheck its dice.
pub(crate) struct PreparedContentContext<'a> {
    pub fingerprint: &'a str,
    pub formula: atlas_record::source_content::FormulaContext,
    pub spell_rank: Option<u8>,
    pub encounter: Option<ContentRollEncounterContext>,
}
impl<'a> PreparedContentContext<'a> {
    pub fn source(
        fingerprint: &'a str,
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
        Self {
            fingerprint,
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
                        locator: locator.into(),
                        role: format!("{:?}", s.role).to_lowercase(),
                        source_fingerprint: navigation_fingerprint(owners, context.fingerprint),
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
                        html: atlas_record::source_content::render_contextual_interactions(
                            html,
                            &cached.interactions,
                            &context.formula,
                        ),
                        controls: content_controls(&cached.interactions, Some((&locator, context))),
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
            source_fingerprint: navigation_fingerprint(owners, context.fingerprint),
            body,
        });
    }
    Ok(fields)
}

pub(crate) fn content_controls(
    interactions: &[atlas_record::source_content::ContentInteraction],
    source: Option<(&SourceContentLocator, &PreparedContentContext<'_>)>,
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
                            source: locator.clone().into(),
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
                            source: locator.clone().into(),
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
    let mut context = PreparedContentContext::source(
        r.source_fingerprint(),
        source,
        &view.selected.owners,
        view.selected.spell_rank,
    );
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
            *html = atlas_record::source_content::render_contextual_interactions(
                authored,
                &cached.interactions,
                &context.formula,
            );
            *controls = content_controls(&cached.interactions, Some((&cached.locator, &context)));
            inclusions.clear();
        }
    }
    crate::inline_content::compose_inline_content(
        r,
        view.presentation.content.iter_mut(),
        &content.relationships,
    )
}
