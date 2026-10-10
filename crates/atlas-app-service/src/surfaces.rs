//! Selected source presentation, using the same authored DTO for roots and children.
use crate::projection::{field_state, navigation_fingerprint};
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::*;
use atlas_domain::{QueryFieldState, RecordKey};
use atlas_record::source_content::{
    ContentAudience, ContentInteractionKind, ContentVisibilityRule, OwnedContentLocator,
    SourceContentLocator,
};
use atlas_record::source_record::{
    ActorIwrKind, ActorIwrType, ActorSave, ItemSourceView, SourceBackedRecord, SourceContentFormat,
    SourceContentSelection, SourceFieldView, SourceNodeView, SourceQueryView,
};

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
pub(crate) fn record_surface(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    summary: &RecordSummaryView,
    owners: &[OwnedContentLocator],
    fields: Vec<PreparedContentFieldView>,
    profile: RecordSurfaceProfileView,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> RecordSurfaceView {
    let fingerprint = retrieval.source_fingerprint();
    let mut sections = Vec::new();
    let query = SourceQueryView {
        source: node,
        pack_id: record.key().pack().as_str(),
        pack_label: summary.pack.as_deref().unwrap_or(""),
    };
    let actor = query.actor();
    let mut facts = vec![];
    for (key, label, value) in [
        ("actor.ac", "AC", actor.armor_class()),
        ("actor.hp.maximum", "Maximum HP", actor.hp_maximum()),
        ("actor.perception", "Perception", actor.perception()),
        ("hazard.hardness", "Hardness", actor.hazard_hardness()),
        ("hazard.stealth", "Stealth", actor.hazard_stealth()),
    ] {
        if let Some(n) = value.value() {
            facts.push(number_value(
                key,
                label,
                n.clone(),
                SurfaceValueDisplayView::StaticNumber,
            ));
        }
    }
    if !facts.is_empty() {
        sections.push(section(
            RecordSurfaceSectionKindView::Vitals,
            "Authored statistics",
            facts,
        ));
    }
    let saves = [
        (ActorSave::Fortitude, "Fortitude"),
        (ActorSave::Reflex, "Reflex"),
        (ActorSave::Will, "Will"),
    ]
    .into_iter()
    .filter_map(|(save, label)| {
        actor.save(save).value().map(|n| {
            number_value(
                &format!("save.{}", label.to_lowercase()),
                label,
                n.clone(),
                SurfaceValueDisplayView::SignedModifier,
            )
        })
    })
    .collect::<Vec<_>>();
    if !saves.is_empty() {
        sections.push(section(RecordSurfaceSectionKindView::Saves, "Saves", saves));
    }
    let defenses = authored_defenses(actor);
    if !defenses.is_empty() {
        let mut s = section(
            RecordSurfaceSectionKindView::Defenses,
            "Authored defenses",
            vec![],
        );
        s.notes = defenses;
        sections.push(s);
    }
    let mut movement = vec![];
    if let Some(n) = actor.land_speed().value() {
        movement.push(number_value(
            "speed.land",
            "Land speed",
            n.clone(),
            SurfaceValueDisplayView::Distance,
        ));
    }
    if let Some(speeds) = actor.speeds().value() {
        for speed in speeds.iter() {
            if let (Some(kind), Some(n)) = (speed.speed_type().value(), speed.value().value()) {
                let label = serde_json::to_value(kind)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "Movement".into());
                movement.push(number_value(
                    &format!("speed.{label}"),
                    &label,
                    n.clone(),
                    SurfaceValueDisplayView::Distance,
                ));
            }
        }
    }
    if !movement.is_empty() {
        sections.push(section(
            RecordSurfaceSectionKindView::Movement,
            "Movement",
            movement,
        ));
    }
    if owners.is_empty()
        && let Some(block) = crate::encounters::mechanics::record_stat_block(record, fingerprint)
    {
        for (prefix, kind, title) in [
            (
                "ability.",
                RecordSurfaceSectionKindView::Abilities,
                "Abilities",
            ),
            ("skill.", RecordSurfaceSectionKindView::Skills, "Skills"),
        ] {
            let values = block
                .values
                .iter()
                .filter(|v| v.target.starts_with(prefix))
                .map(|v| {
                    number_value(
                        &v.target,
                        &v.label,
                        v.base_value.clone(),
                        SurfaceValueDisplayView::SignedModifier,
                    )
                })
                .collect::<Vec<_>>();
            if !values.is_empty() {
                sections.push(section(kind, title, values));
            }
        }
        append_context(&mut sections, &block.unapplied_effects);
    }
    append_node_sections(&mut sections, node);
    for field in fields {
        let title = match field.role.as_str() {
            "description" => "Description",
            "routine" => "Routine",
            "disable" => "Disable",
            "reset" => "Reset",
            "stealth" => "Stealth",
            "tableresult" => "Result",
            _ => "Notes",
        };
        let mut s = section(RecordSurfaceSectionKindView::RichContent, title, vec![]);
        s.content = Some(field);
        sections.push(s);
    }
    append_owned_sections(record, node, owners, fingerprint, &mut sections);
    record_surface_header(record, node, summary, owners, profile, retrieval, sections)
}
fn authored_defenses(
    actor: atlas_record::source_record::ActorQueryView<'_>,
) -> Vec<SurfaceNoteView> {
    let mut defenses = vec![];
    for (kind, label) in [
        (ActorIwrKind::Immunity, "Immunity"),
        (ActorIwrKind::Weakness, "Weakness"),
        (ActorIwrKind::Resistance, "Resistance"),
    ] {
        if let Some(entries) = actor.iwr(kind).value() {
            for entry in entries.iter() {
                let Some(kind) = entry.iwr_type().value() else {
                    continue;
                };
                let text = match kind {
                    ActorIwrType::Immunity(v) => serde_json::to_value(v),
                    ActorIwrType::Weakness(v) => serde_json::to_value(v),
                    ActorIwrType::Resistance(v) => serde_json::to_value(v),
                }
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned));
                if let Some(text) = text {
                    defenses.push(SurfaceNoteView {
                        label: label.into(),
                        text: entry
                            .value()
                            .value()
                            .map(|v| format!("{text} {v}"))
                            .unwrap_or(text),
                        source: None,
                    });
                }
            }
        }
    }
    if let Some(complex) = actor.hazard_complexity().value() {
        defenses.push(SurfaceNoteView {
            label: "Complexity".into(),
            text: if complex { "Complex" } else { "Simple" }.into(),
            source: None,
        });
    }
    defenses
}
fn append_context(sections: &mut Vec<RecordSurfaceSectionView>, notes: &[UnappliedEffectView]) {
    if !notes.is_empty() {
        let mut s = section(
            RecordSurfaceSectionKindView::Notes,
            "Authored and runtime context",
            vec![],
        );
        s.notes = notes
            .iter()
            .map(|n| SurfaceNoteView {
                label: n.label.clone(),
                text: n.reason.clone(),
                source: Some(n.source.clone()),
            })
            .collect();
        sections.push(s);
    }
}
fn append_node_sections(sections: &mut Vec<RecordSurfaceSectionView>, node: SourceNodeView<'_>) {
    if let SourceNodeView::Item(item) = node {
        let mut values = item_facts(item);
        if let Some(n) = item.melee_attack().value() {
            values.push(number_value(
                "attack",
                "Attack",
                n.clone(),
                SurfaceValueDisplayView::SignedModifier,
            ));
        }
        if let Some(damage) = item.melee_damage().value() {
            for (id, d) in &damage.entries {
                if let Some(formula) = SourceFieldView::from(&d.damage).value() {
                    values.push(formula_value(id, "Damage", formula));
                }
            }
        }
        if let Some(damage) = item.spell_damage().value() {
            for (id, d) in &damage.entries {
                if let Some(formula) = SourceFieldView::from(&d.formula).value() {
                    values.push(formula_value(id, "Damage", formula));
                }
            }
        }
        if !values.is_empty() {
            sections.push(section(
                RecordSurfaceSectionKindView::Activities,
                "Authored activity",
                values,
            ));
        }
    }
    if let SourceNodeView::Table(table) = node
        && let Some(formula) = SourceFieldView::from(&table.formula).value()
    {
        sections.push(section(
            RecordSurfaceSectionKindView::Activities,
            "Roll table",
            vec![formula_value("table.formula", "Formula", formula)],
        ));
    }
    if let SourceNodeView::Result(_) = node {
        let values = table_result_facts(node);
        if !values.is_empty() {
            sections.push(section(
                RecordSurfaceSectionKindView::References,
                "Table result",
                values,
            ));
        }
    }
}
fn append_owned_sections(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    owners: &[OwnedContentLocator],
    fingerprint: &str,
    sections: &mut Vec<RecordSurfaceSectionView>,
) {
    if owners.is_empty() {
        let mut activities = Vec::new();
        let mut add = |owner: &OwnedContentLocator, child: SourceNodeView<'_>| {
            let owners = vec![owner.clone()];
            activities.push(SurfaceActivityView {
                navigation: RecordNavigationView {
                    record_key: record.key().to_string(),
                    source_fingerprint: navigation_fingerprint(&owners, fingerprint),
                    owners,
                    field: None,
                    passage: None,
                },
                key: format!("{}:{:?}", owner.collection, owner.identity),
                label: node_title(child).unwrap_or_else(|| child.document_kind().into()),
                kind: child
                    .source_type()
                    .value()
                    .unwrap_or(child.document_kind())
                    .into(),
                usage: match child {
                    SourceNodeView::Item(i) => item_usage(i),
                    _ => None,
                },
                values: table_result_facts(child),
                groups: vec![],
                notes: vec![],
            });
        };
        match node {
            SourceNodeView::Actor(_) => {
                record.visit_immediate_actor_items(|_, o, i| add(o, SourceNodeView::Item(i)));
            }
            SourceNodeView::Journal(_) => {
                record.visit_immediate_journal_pages(&mut add);
            }
            SourceNodeView::Table(_) => {
                record.visit_immediate_table_results(&mut add);
            }
            _ => {}
        }
        if !activities.is_empty() {
            let mut s = section(
                RecordSurfaceSectionKindView::Activities,
                "Embedded records",
                vec![],
            );
            s.activities = activities;
            sections.push(s);
        }
    }
}
fn record_surface_header(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    summary: &RecordSummaryView,
    owners: &[OwnedContentLocator],
    profile: RecordSurfaceProfileView,
    retrieval: &atlas_search::AtlasRetrievalService,
    sections: Vec<RecordSurfaceSectionView>,
) -> RecordSurfaceView {
    let query = SourceQueryView {
        source: node,
        pack_id: record.key().pack().as_str(),
        pack_label: summary.pack.as_deref().unwrap_or(""),
    };
    let actor = query.actor();
    let mut header = RecordSurfaceHeaderView {
        level_label: summary.level_label.clone(),
        kind_label: Some(summary.kind_label.clone()),
        rarity: summary.rarity.clone(),
        traits: summary
            .traits
            .iter()
            .map(|t| SurfaceBadgeView {
                kind: t.kind.clone(),
                label: t.label.clone(),
                value: t.value.clone(),
            })
            .collect(),
        publication: summary.publication.clone(),
        pack: summary.pack.clone(),
    };
    if !owners.is_empty() {
        header.level_label = match node {
            SourceNodeView::Item(i) => i
                .spell_rank()
                .value()
                .map(|v| format!("Rank {v}"))
                .or_else(|| i.item_level().value().map(|v| format!("Level {v}"))),
            _ => actor.level().value().map(|v| format!("Level {v}")),
        };
        header.kind_label = node.source_type().value().map(str::to_owned);
        header.rarity = query
            .rarity()
            .value()
            .and_then(|v| serde_json::to_value(v).ok())
            .and_then(|v| v.as_str().map(str::to_owned));
        header.traits = query
            .traits()
            .value()
            .into_iter()
            .flatten()
            .map(|v| SurfaceBadgeView {
                kind: "trait".into(),
                label: retrieval.trait_label(v).unwrap_or(v).into(),
                value: v.clone(),
            })
            .collect();
        header.publication = query.publication_title().value().map(str::to_owned);
    }
    RecordSurfaceView {
        record_key: record.key().to_string(),
        title: node_title(node).unwrap_or_else(|| summary.title.clone()),
        kind: node.source_type().value().unwrap_or(&summary.kind).into(),
        profile,
        header,
        sections,
    }
}
fn node_title(node: SourceNodeView<'_>) -> Option<String> {
    node.name().value().cloned().or_else(|| match node {
        SourceNodeView::Result(r) => SourceFieldView::from(&r.range)
            .value()
            .map(|(lo, hi)| format!("Result {lo}–{hi}")),
        _ => None,
    })
}
pub(crate) fn section(
    kind: RecordSurfaceSectionKindView,
    title: &str,
    values: Vec<SurfaceValueView>,
) -> RecordSurfaceSectionView {
    RecordSurfaceSectionView {
        kind,
        title: title.into(),
        values,
        groups: vec![],
        activities: vec![],
        notes: vec![],
        content: None,
        collapsed_by_default: false,
    }
}
pub(crate) fn number_value(
    key: &str,
    label: &str,
    n: serde_json::Number,
    display: SurfaceValueDisplayView,
) -> SurfaceValueView {
    SurfaceValueView {
        key: key.into(),
        label: label.into(),
        value: SurfaceScalarView::Number(n),
        base_value: None,
        adjusted: false,
        display,
        adjustments: vec![],
        suppressed_adjustments: vec![],
        notes: vec![],
    }
}

fn formula_value(key: &str, label: &str, value: &str) -> SurfaceValueView {
    SurfaceValueView {
        key: key.into(),
        label: label.into(),
        value: SurfaceScalarView::Formula(value.into()),
        base_value: None,
        adjusted: false,
        display: SurfaceValueDisplayView::Formula,
        adjustments: vec![],
        suppressed_adjustments: vec![],
        notes: vec![],
    }
}
fn text_value(key: &str, label: &str, value: &str) -> SurfaceValueView {
    let mut v = formula_value(key, label, value);
    v.value = SurfaceScalarView::Text(value.into());
    v.display = SurfaceValueDisplayView::Text;
    v
}
fn item_usage(item: ItemSourceView<'_>) -> Option<String> {
    item.spell_casting_time()
        .value()
        .or_else(|| item.physical_usage().value())
        .map(str::to_owned)
        .or_else(|| {
            item.action_type()
                .value()
                .and_then(|v| serde_json::to_value(v).ok())
                .and_then(|v| v.as_str().map(str::to_owned))
                .map(|kind| {
                    item.action_count()
                        .value()
                        .map(|n| format!("{n} {kind}"))
                        .unwrap_or(kind)
                })
        })
}
fn item_facts(item: ItemSourceView<'_>) -> Vec<SurfaceValueView> {
    let mut values = vec![];
    for (key, label, value) in [
        ("spell.casting", "Casting", item.spell_casting_time()),
        ("spell.range", "Range", item.spell_range()),
        ("spell.target", "Target", item.spell_target()),
        ("spell.duration", "Duration", item.spell_duration_text()),
        ("physical.usage", "Usage", item.physical_usage()),
    ] {
        if let Some(text) = value.value() {
            values.push(text_value(key, label, text));
        }
    }
    if let Some(n) = item.physical_bulk().value() {
        values.push(number_value(
            "physical.bulk",
            "Bulk",
            n.clone(),
            SurfaceValueDisplayView::StaticNumber,
        ));
    }
    if let Some(price) = item.physical_price().value() {
        if let Some(coins) = SourceFieldView::from(&price.value).value() {
            let text = [
                ("pp", &coins.pp),
                ("gp", &coins.gp),
                ("sp", &coins.sp),
                ("cp", &coins.cp),
            ]
            .into_iter()
            .filter_map(|(unit, n)| {
                SourceFieldView::from(n)
                    .value()
                    .map(|n| format!("{n} {unit}"))
            })
            .collect::<Vec<_>>()
            .join(", ");
            if !text.is_empty() {
                values.push(text_value("physical.price", "Price", &text));
            }
        }
        if let Some(n) = SourceFieldView::from(&price.per).value() {
            values.push(number_value(
                "physical.price.per",
                "Price per",
                n.clone(),
                SurfaceValueDisplayView::StaticNumber,
            ));
        }
    }
    values
}
fn table_result_facts(node: SourceNodeView<'_>) -> Vec<SurfaceValueView> {
    let SourceNodeView::Result(result) = node else {
        return vec![];
    };
    let mut values = vec![];
    if let Some((start, end)) = SourceFieldView::from(&result.range).value() {
        values.push(text_value(
            "table.result.range",
            "Range",
            &format!("{start}–{end}"),
        ));
    }
    if let Some(n) = SourceFieldView::from(&result.weight).value() {
        values.push(number_value(
            "table.result.weight",
            "Weight",
            n.clone(),
            SurfaceValueDisplayView::StaticNumber,
        ));
    }
    if let Some(kind) = SourceFieldView::from(&result.r#type)
        .value()
        .and_then(|v| serde_json::to_value(v).ok())
        .and_then(|v| v.as_str().map(str::to_owned))
    {
        values.push(text_value("table.result.type", "Result type", &kind));
    }
    values
}
pub(crate) fn encounter_participant_surface(
    p: &EncounterParticipantView,
    record: Option<&SourceBackedRecord>,
) -> Option<RecordSurfaceView> {
    let block = p.stat_block.as_ref()?;
    let mut sections = vec![];
    for (kind, title) in [
        (RecordSurfaceSectionKindView::Vitals, "HP"),
        (RecordSurfaceSectionKindView::Defenses, "Defenses"),
        (RecordSurfaceSectionKindView::Saves, "Saves"),
        (RecordSurfaceSectionKindView::Abilities, "Abilities"),
        (RecordSurfaceSectionKindView::Skills, "Skills"),
    ] {
        let values = block
            .values
            .iter()
            .filter(|v| stat_section(&v.target) == kind)
            .map(|v| {
                let mut value = number_value(
                    &v.target,
                    &v.label,
                    v.adjusted_value.clone(),
                    if matches!(v.target.as_str(), "ac" | "hp.max" | "hazard.hardness") {
                        SurfaceValueDisplayView::StaticNumber
                    } else {
                        SurfaceValueDisplayView::SignedModifier
                    },
                );
                value.base_value = Some(SurfaceScalarView::Number(v.base_value.clone()));
                value.adjusted = v.base_value != v.adjusted_value;
                value.adjustments = v
                    .modifiers
                    .iter()
                    .map(|m| SurfaceAdjustmentView {
                        label: m.label.clone(),
                        source: m.source.clone(),
                        delta: Some(SurfaceScalarView::Number(m.value.into())),
                        reason: Some(format!("{:?}", m.modifier_type).to_lowercase()),
                    })
                    .collect();
                value
            })
            .collect::<Vec<_>>();
        let mut s = section(kind, title, values);
        if kind == RecordSurfaceSectionKindView::Defenses
            && let Some(record) = record
        {
            s.notes = authored_defenses(
                SourceQueryView::new(record.source(), record.key().pack().as_str(), "").actor(),
            );
        }
        if !s.values.is_empty() || !s.notes.is_empty() {
            sections.push(s);
        }
    }
    if !block.speeds.is_empty() {
        let values = block
            .speeds
            .iter()
            .map(|s| {
                let mut v = number_value(
                    &format!("speed.{}", s.movement_type),
                    &s.label,
                    s.adjusted_value_feet.clone(),
                    SurfaceValueDisplayView::Distance,
                );
                v.base_value = Some(SurfaceScalarView::Number(s.base_value_feet.clone()));
                v.adjusted = s.base_value_feet != s.adjusted_value_feet;
                v.adjustments = s
                    .adjustments
                    .iter()
                    .map(|a| SurfaceAdjustmentView {
                        label: a.label.clone(),
                        source: a.source.clone(),
                        delta: Some(SurfaceScalarView::Number(a.value.into())),
                        reason: a.reason.clone(),
                    })
                    .collect();
                v.notes = s
                    .notes
                    .iter()
                    .map(|n| SurfaceNoteView {
                        label: n.label.clone(),
                        text: n.reason.clone(),
                        source: Some(n.source.clone()),
                    })
                    .collect();
                v
            })
            .collect();
        sections.push(section(
            RecordSurfaceSectionKindView::Movement,
            "Movement",
            values,
        ));
    }
    if let Some(budget) = &block.action_budget {
        let mut s = section(
            RecordSurfaceSectionKindView::Runtime,
            "Action budget",
            vec![
                number_value(
                    "actions",
                    "Actions",
                    budget.actions.adjusted_value.into(),
                    SurfaceValueDisplayView::StaticNumber,
                ),
                number_value(
                    "reactions",
                    "Reactions",
                    budget.reactions.adjusted_value.into(),
                    SurfaceValueDisplayView::StaticNumber,
                ),
            ],
        );
        s.notes = budget
            .notes
            .iter()
            .map(|n| SurfaceNoteView {
                label: n.label.clone(),
                text: n.reason.clone(),
                source: Some(n.source.clone()),
            })
            .collect();
        for count in [&budget.actions, &budget.reactions] {
            for segment in &count.segments {
                if segment.restricted {
                    s.notes.push(SurfaceNoteView {
                        label: segment.label.clone(),
                        text: segment
                            .reason
                            .clone()
                            .unwrap_or_else(|| format!("{} restricted", segment.value)),
                        source: None,
                    });
                }
            }
        }
        sections.push(s);
    }
    append_context(&mut sections, &block.unapplied_effects);
    if !block.activities.is_empty() {
        let mut s = section(
            RecordSurfaceSectionKindView::Activities,
            "Activities",
            vec![],
        );
        s.activities = block
            .activities
            .iter()
            .map(|a| SurfaceActivityView {
                key: a.activity_id.clone(),
                label: a.label.clone(),
                kind: format!("{:?}", a.kind).to_lowercase(),
                navigation: a.navigation.clone(),
                usage: record.and_then(|r| match r.node_at(&a.navigation.owners) {
                    Some(SourceNodeView::Item(i)) => item_usage(i),
                    _ => None,
                }),
                values: a
                    .rolls
                    .iter()
                    .map(|r| {
                        number_value(
                            &r.roll_id,
                            &r.label,
                            r.adjusted_value.clone(),
                            SurfaceValueDisplayView::SignedModifier,
                        )
                    })
                    .chain(
                        a.damage
                            .iter()
                            .map(|d| formula_value(&d.damage_id, "Damage", &d.formula)),
                    )
                    .collect(),
                groups: vec![],
                notes: a
                    .notes
                    .iter()
                    .map(|n| SurfaceNoteView {
                        label: n.label.clone(),
                        text: n.reason.clone(),
                        source: Some(n.source.clone()),
                    })
                    .collect(),
            })
            .collect();
        sections.push(s);
    }
    Some(RecordSurfaceView {
        record_key: p
            .record_key
            .clone()
            .unwrap_or_else(|| p.participant_key.clone()),
        title: p.display_name.clone(),
        kind: format!("{:?}", p.participant_kind).to_lowercase(),
        profile: RecordSurfaceProfileView::EncounterParticipant,
        header: RecordSurfaceHeaderView {
            level_label: block.adjusted_level.as_ref().map(ToString::to_string),
            kind_label: None,
            rarity: p.record.as_ref().and_then(|r| r.rarity.clone()),
            traits: p
                .record
                .as_ref()
                .map(|r| {
                    r.traits
                        .iter()
                        .map(|t| SurfaceBadgeView {
                            kind: t.kind.clone(),
                            label: t.label.clone(),
                            value: t.value.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            publication: None,
            pack: None,
        },
        sections,
    })
}
fn stat_section(target: &str) -> RecordSurfaceSectionKindView {
    if target == "hp.max" {
        RecordSurfaceSectionKindView::Vitals
    } else if target.starts_with("save.") {
        RecordSurfaceSectionKindView::Saves
    } else if target.starts_with("ability.") {
        RecordSurfaceSectionKindView::Abilities
    } else if target.starts_with("skill.") || target == "hazard.stealth" {
        RecordSurfaceSectionKindView::Skills
    } else {
        RecordSurfaceSectionKindView::Defenses
    }
}
