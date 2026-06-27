use atlas_app_model::{
    ActionBudgetView, DamageExpressionView, EncounterParticipantConditionView,
    EncounterParticipantView, MechanicActivityView, MovementSpeedView, RecordSurfaceHeaderView,
    RecordSurfaceProfileView, RecordSurfaceSectionKindView, RecordSurfaceSectionView,
    RecordSurfaceView, RuntimeAdjustmentView, RuntimeCountView, RuntimeEffectNoteView,
    StatBlockView, StatModifierView, StatValueView, SurfaceActivityView, SurfaceAdjustmentView,
    SurfaceBadgeView, SurfaceNoteView, SurfaceScalarView, SurfaceValueDisplayView,
    SurfaceValueGroupView, SurfaceValueView, UnappliedEffectView,
};
use atlas_record::{
    AtlasRecord, PresentationBlock, PresentationContent, PresentationSectionKind,
    RecordPresentationDocument, build_record_presentation_document,
};

use crate::encounters::record_stat_block;
use crate::projection::kind_label;

pub(crate) fn record_surface(
    record: &AtlasRecord,
    profile: RecordSurfaceProfileView,
) -> RecordSurfaceView {
    let presentation = build_record_presentation_document(record);
    let stat_block = record_stat_block(record);
    surface_from_parts(
        SurfaceParts {
            record_key: record.identity.key.to_string(),
            title: record.identity.name.clone(),
            kind: record.classification.kind.as_str().to_string(),
            profile,
            header: record_header(record, profile),
            description: description_content(&presentation),
            stat_block: stat_block.as_ref(),
            conditions: &[],
            participant_note: None,
            fallback_presentation: match profile {
                RecordSurfaceProfileView::RecordDetail => Some(presentation),
                RecordSurfaceProfileView::EncounterParticipant => Some(presentation),
                RecordSurfaceProfileView::SearchCompact => None,
            },
        },
        profile,
    )
}

pub(crate) fn encounter_participant_surface(
    participant: &EncounterParticipantView,
    record: Option<&AtlasRecord>,
    stat_block: Option<&StatBlockView>,
) -> Option<RecordSurfaceView> {
    let fallback_presentation = record.map(build_record_presentation_document);
    let header = record
        .map(|record| record_header(record, RecordSurfaceProfileView::EncounterParticipant))
        .unwrap_or_else(|| participant_header(participant));
    let description = fallback_presentation.as_ref().and_then(description_content);
    Some(surface_from_parts(
        SurfaceParts {
            record_key: participant
                .record_key
                .clone()
                .unwrap_or_else(|| participant.participant_key.clone()),
            title: participant.display_name.clone(),
            kind: format!("{:?}", participant.participant_kind).to_lowercase(),
            profile: RecordSurfaceProfileView::EncounterParticipant,
            header,
            description,
            stat_block,
            conditions: &participant.conditions,
            participant_note: participant.note.clone(),
            fallback_presentation,
        },
        RecordSurfaceProfileView::EncounterParticipant,
    ))
}

struct SurfaceParts<'a> {
    record_key: String,
    title: String,
    kind: String,
    profile: RecordSurfaceProfileView,
    header: RecordSurfaceHeaderView,
    description: Option<PresentationContent>,
    stat_block: Option<&'a StatBlockView>,
    conditions: &'a [EncounterParticipantConditionView],
    participant_note: Option<String>,
    fallback_presentation: Option<RecordPresentationDocument>,
}

fn surface_from_parts(
    parts: SurfaceParts<'_>,
    profile: RecordSurfaceProfileView,
) -> RecordSurfaceView {
    let mut sections = Vec::new();

    if profile == RecordSurfaceProfileView::SearchCompact {
        if let Some(description) = parts.description {
            sections.push(RecordSurfaceSectionView {
                kind: RecordSurfaceSectionKindView::Description,
                title: "Description".to_string(),
                values: Vec::new(),
                groups: Vec::new(),
                activities: Vec::new(),
                notes: Vec::new(),
                content: Some(description),
                collapsed_by_default: false,
            });
        }
        if let Some(stat_block) = parts.stat_block {
            sections.extend(search_compact_stat_sections(stat_block));
        }
        return RecordSurfaceView {
            record_key: parts.record_key,
            title: parts.title,
            kind: parts.kind,
            profile: parts.profile,
            header: parts.header,
            sections,
            fallback_presentation: None,
        };
    }

    if let Some(description) = parts.description {
        sections.push(RecordSurfaceSectionView {
            kind: RecordSurfaceSectionKindView::Description,
            title: "Description".to_string(),
            values: Vec::new(),
            groups: Vec::new(),
            activities: Vec::new(),
            notes: Vec::new(),
            content: Some(description),
            collapsed_by_default: profile == RecordSurfaceProfileView::EncounterParticipant,
        });
    }

    if let Some(stat_block) = parts.stat_block {
        sections.extend(stat_sections(stat_block, profile));
    }

    if !parts.conditions.is_empty() {
        sections.push(condition_section(parts.conditions));
    }

    if let Some(note) = parts
        .participant_note
        .filter(|note| !note.trim().is_empty())
    {
        sections.push(RecordSurfaceSectionView {
            kind: RecordSurfaceSectionKindView::Notes,
            title: "Notes".to_string(),
            values: Vec::new(),
            groups: Vec::new(),
            activities: Vec::new(),
            notes: vec![SurfaceNoteView {
                label: "Participant note".to_string(),
                text: note,
                source: None,
            }],
            content: None,
            collapsed_by_default: true,
        });
    }

    RecordSurfaceView {
        record_key: parts.record_key,
        title: parts.title,
        kind: parts.kind,
        profile: parts.profile,
        header: parts.header,
        sections,
        fallback_presentation: parts.fallback_presentation,
    }
}

fn search_compact_stat_sections(stat_block: &StatBlockView) -> Vec<RecordSurfaceSectionView> {
    let mut values = Vec::new();
    values.extend(values_for_targets(stat_block, &["ac", "hp.max"]));
    values.extend(
        stat_block
            .values
            .iter()
            .filter(|value| value.target.starts_with("save."))
            .map(surface_stat_value),
    );
    values.extend(stat_block.speeds.iter().map(surface_speed_value));
    if values.is_empty() {
        return Vec::new();
    }
    vec![RecordSurfaceSectionView {
        kind: RecordSurfaceSectionKindView::Identity,
        title: "Facts".to_string(),
        values,
        groups: Vec::new(),
        activities: Vec::new(),
        notes: Vec::new(),
        content: None,
        collapsed_by_default: false,
    }]
}

fn stat_sections(
    stat_block: &StatBlockView,
    profile: RecordSurfaceProfileView,
) -> Vec<RecordSurfaceSectionView> {
    let mut sections = Vec::new();

    if profile == RecordSurfaceProfileView::EncounterParticipant
        && (stat_block.action_budget.is_some() || !stat_block.speeds.is_empty())
    {
        sections.push(runtime_section(stat_block));
    }

    let vitals = values_for_targets(stat_block, &["hp.max"]);
    if !vitals.is_empty() {
        sections.push(value_section(
            RecordSurfaceSectionKindView::Vitals,
            "Vitals",
            vitals,
        ));
    }

    let defenses = values_for_targets(stat_block, &["ac"]);
    if !defenses.is_empty() {
        sections.push(value_section(
            RecordSurfaceSectionKindView::Defenses,
            "Defenses",
            defenses,
        ));
    }

    let saves = stat_block
        .values
        .iter()
        .filter(|value| value.target.starts_with("save."))
        .map(surface_stat_value)
        .collect::<Vec<_>>();
    if !saves.is_empty() {
        sections.push(value_section(
            RecordSurfaceSectionKindView::Saves,
            "Saves",
            saves,
        ));
    }

    let abilities = stat_block
        .values
        .iter()
        .filter(|value| value.target.starts_with("ability."))
        .map(surface_stat_value)
        .collect::<Vec<_>>();
    if !abilities.is_empty() && profile != RecordSurfaceProfileView::SearchCompact {
        sections.push(value_section(
            RecordSurfaceSectionKindView::Abilities,
            "Abilities",
            abilities,
        ));
    }

    let skill_values = stat_block
        .values
        .iter()
        .filter(|value| value.target.starts_with("skill."))
        .map(surface_stat_value)
        .collect::<Vec<_>>();
    if !skill_values.is_empty() && profile != RecordSurfaceProfileView::SearchCompact {
        sections.push(value_section(
            RecordSurfaceSectionKindView::Skills,
            "Skills",
            skill_values,
        ));
    }

    if !stat_block.speeds.is_empty() {
        sections.push(RecordSurfaceSectionView {
            kind: RecordSurfaceSectionKindView::Movement,
            title: "Movement".to_string(),
            values: stat_block.speeds.iter().map(surface_speed_value).collect(),
            groups: Vec::new(),
            activities: Vec::new(),
            notes: stat_block
                .speeds
                .iter()
                .flat_map(|speed| speed.notes.iter().map(runtime_note))
                .collect(),
            content: None,
            collapsed_by_default: false,
        });
    }

    if !stat_block.activities.is_empty() && profile != RecordSurfaceProfileView::SearchCompact {
        sections.push(RecordSurfaceSectionView {
            kind: RecordSurfaceSectionKindView::Activities,
            title: "Activities".to_string(),
            values: Vec::new(),
            groups: Vec::new(),
            activities: stat_block.activities.iter().map(surface_activity).collect(),
            notes: Vec::new(),
            content: None,
            collapsed_by_default: false,
        });
    }

    if !stat_block.unapplied_effects.is_empty() {
        sections.push(RecordSurfaceSectionView {
            kind: RecordSurfaceSectionKindView::References,
            title: "Effect Notes".to_string(),
            values: Vec::new(),
            groups: Vec::new(),
            activities: Vec::new(),
            notes: stat_block
                .unapplied_effects
                .iter()
                .map(unapplied_note)
                .collect(),
            content: None,
            collapsed_by_default: false,
        });
    }

    sections
}

fn runtime_section(stat_block: &StatBlockView) -> RecordSurfaceSectionView {
    let mut values = Vec::new();
    let mut notes = Vec::new();
    if let Some(action_budget) = &stat_block.action_budget {
        values.extend(action_values(action_budget));
        notes.extend(action_budget.notes.iter().map(runtime_note));
    }
    RecordSurfaceSectionView {
        kind: RecordSurfaceSectionKindView::Runtime,
        title: "Runtime".to_string(),
        values,
        groups: Vec::new(),
        activities: Vec::new(),
        notes,
        content: None,
        collapsed_by_default: false,
    }
}

fn value_section(
    kind: RecordSurfaceSectionKindView,
    title: &str,
    values: Vec<SurfaceValueView>,
) -> RecordSurfaceSectionView {
    RecordSurfaceSectionView {
        kind,
        title: title.to_string(),
        values,
        groups: Vec::new(),
        activities: Vec::new(),
        notes: Vec::new(),
        content: None,
        collapsed_by_default: false,
    }
}

fn values_for_targets(stat_block: &StatBlockView, targets: &[&str]) -> Vec<SurfaceValueView> {
    targets
        .iter()
        .filter_map(|target| {
            stat_block
                .values
                .iter()
                .find(|value| value.target == *target)
                .map(surface_stat_value)
        })
        .collect()
}

fn surface_stat_value(value: &StatValueView) -> SurfaceValueView {
    let display = if value.target == "ac" || value.target == "hp.max" {
        SurfaceValueDisplayView::StaticNumber
    } else {
        SurfaceValueDisplayView::SignedModifier
    };
    numeric_value(
        value.target.clone(),
        value.label.clone(),
        value.base_value,
        value.adjusted_value,
        display,
        value.modifiers.iter().map(stat_modifier).collect(),
        value
            .suppressed_modifiers
            .iter()
            .map(stat_modifier)
            .collect(),
    )
}

fn surface_speed_value(speed: &MovementSpeedView) -> SurfaceValueView {
    SurfaceValueView {
        key: format!("speed.{}", speed.movement_type),
        label: speed.label.clone(),
        value: SurfaceScalarView::DistanceFeet(speed.adjusted_value_feet),
        base_value: Some(SurfaceScalarView::DistanceFeet(speed.base_value_feet)),
        adjusted: speed.adjusted_value_feet != speed.base_value_feet,
        display: SurfaceValueDisplayView::Distance,
        adjustments: speed.adjustments.iter().map(runtime_adjustment).collect(),
        suppressed_adjustments: speed
            .suppressed_adjustments
            .iter()
            .map(runtime_adjustment)
            .collect(),
        notes: speed.notes.iter().map(runtime_note).collect(),
    }
}

fn action_values(action_budget: &ActionBudgetView) -> Vec<SurfaceValueView> {
    vec![
        runtime_count_value("actions", &action_budget.actions),
        runtime_count_value("reactions", &action_budget.reactions),
    ]
}

fn runtime_count_value(key: &str, count: &RuntimeCountView) -> SurfaceValueView {
    numeric_value(
        format!("runtime.{key}"),
        count.label.clone(),
        count.base_value,
        count.adjusted_value,
        SurfaceValueDisplayView::StaticNumber,
        count.adjustments.iter().map(runtime_adjustment).collect(),
        count
            .suppressed_adjustments
            .iter()
            .map(runtime_adjustment)
            .collect(),
    )
}

fn numeric_value(
    key: String,
    label: String,
    base_value: i64,
    adjusted_value: i64,
    display: SurfaceValueDisplayView,
    adjustments: Vec<SurfaceAdjustmentView>,
    suppressed_adjustments: Vec<SurfaceAdjustmentView>,
) -> SurfaceValueView {
    SurfaceValueView {
        key,
        label,
        value: SurfaceScalarView::Number(adjusted_value),
        base_value: Some(SurfaceScalarView::Number(base_value)),
        adjusted: adjusted_value != base_value,
        display,
        adjustments,
        suppressed_adjustments,
        notes: Vec::new(),
    }
}

fn surface_activity(activity: &MechanicActivityView) -> SurfaceActivityView {
    let mut values = activity
        .rolls
        .iter()
        .map(|roll| {
            numeric_value(
                format!("activity.{}.roll.{}", activity.activity_id, roll.roll_id),
                roll.label.clone(),
                roll.base_value,
                roll.adjusted_value,
                SurfaceValueDisplayView::SignedModifier,
                roll.modifiers.iter().map(stat_modifier).collect(),
                roll.suppressed_modifiers
                    .iter()
                    .map(stat_modifier)
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    values.extend(activity.damage.iter().map(|damage| {
        surface_damage_value(
            format!(
                "activity.{}.damage.{}",
                activity.activity_id, damage.damage_id
            ),
            damage,
        )
    }));
    SurfaceActivityView {
        key: activity.activity_id.clone(),
        label: activity.label.clone(),
        kind: format!("{:?}", activity.kind).to_lowercase(),
        usage: Some(format!("{:?}", activity.usage).to_lowercase()),
        values,
        groups: activity
            .modes
            .iter()
            .map(|mode| SurfaceValueGroupView {
                key: mode.mode_id.clone(),
                label: mode.label.clone(),
                values: mode
                    .damage
                    .iter()
                    .map(|damage| {
                        surface_damage_value(
                            format!(
                                "activity.{}.mode.{}.damage.{}",
                                activity.activity_id, mode.mode_id, damage.damage_id
                            ),
                            damage,
                        )
                    })
                    .collect(),
            })
            .collect(),
        notes: Vec::new(),
    }
}

fn surface_damage_value(key: String, damage: &DamageExpressionView) -> SurfaceValueView {
    let formula = damage
        .adjusted_formula
        .as_ref()
        .unwrap_or(&damage.formula)
        .clone();
    SurfaceValueView {
        key,
        label: damage
            .label
            .clone()
            .unwrap_or_else(|| match damage.effect_kind {
                atlas_app_model::DamageEffectKindView::Damage => "Damage".to_string(),
                atlas_app_model::DamageEffectKindView::Healing => "Healing".to_string(),
                atlas_app_model::DamageEffectKindView::DamageOrHealing => {
                    "Damage or healing".to_string()
                }
                atlas_app_model::DamageEffectKindView::Unknown => "Effect".to_string(),
            }),
        value: SurfaceScalarView::Formula(match &damage.damage_type {
            Some(damage_type) => format!("{formula} {damage_type}"),
            None => formula,
        }),
        base_value: Some(SurfaceScalarView::Formula(match &damage.damage_type {
            Some(damage_type) => format!("{} {}", damage.formula, damage_type),
            None => damage.formula.clone(),
        })),
        adjusted: damage.adjusted_formula.is_some(),
        display: SurfaceValueDisplayView::Formula,
        adjustments: damage.modifiers.iter().map(stat_modifier).collect(),
        suppressed_adjustments: Vec::new(),
        notes: if damage.effect_kind == atlas_app_model::DamageEffectKindView::DamageOrHealing {
            vec![SurfaceNoteView {
                label: "Mode".to_string(),
                text: "Can be used as damage or healing.".to_string(),
                source: None,
            }]
        } else {
            Vec::new()
        },
    }
}

fn condition_section(conditions: &[EncounterParticipantConditionView]) -> RecordSurfaceSectionView {
    RecordSurfaceSectionView {
        kind: RecordSurfaceSectionKindView::Conditions,
        title: "Conditions".to_string(),
        values: conditions
            .iter()
            .map(|condition| SurfaceValueView {
                key: condition
                    .condition_key
                    .clone()
                    .unwrap_or_else(|| format!("condition.{}", condition.condition_id)),
                label: condition.name.clone(),
                value: condition
                    .value
                    .map(SurfaceScalarView::Number)
                    .unwrap_or_else(|| SurfaceScalarView::Text("active".to_string())),
                base_value: None,
                adjusted: false,
                display: condition
                    .value
                    .map(|_| SurfaceValueDisplayView::StaticNumber)
                    .unwrap_or(SurfaceValueDisplayView::Text),
                adjustments: Vec::new(),
                suppressed_adjustments: Vec::new(),
                notes: condition
                    .note
                    .iter()
                    .map(|note| SurfaceNoteView {
                        label: "Note".to_string(),
                        text: note.clone(),
                        source: None,
                    })
                    .collect(),
            })
            .collect(),
        groups: Vec::new(),
        activities: Vec::new(),
        notes: Vec::new(),
        content: None,
        collapsed_by_default: false,
    }
}

fn stat_modifier(modifier: &StatModifierView) -> SurfaceAdjustmentView {
    SurfaceAdjustmentView {
        label: modifier.label.clone(),
        source: modifier.source.clone(),
        delta: Some(SurfaceScalarView::Number(modifier.value)),
        reason: Some(format!("{:?}", modifier.modifier_type).to_lowercase()),
    }
}

fn runtime_adjustment(adjustment: &RuntimeAdjustmentView) -> SurfaceAdjustmentView {
    SurfaceAdjustmentView {
        label: adjustment.label.clone(),
        source: adjustment.source.clone(),
        delta: Some(SurfaceScalarView::Number(adjustment.value)),
        reason: adjustment.reason.clone(),
    }
}

fn runtime_note(note: &RuntimeEffectNoteView) -> SurfaceNoteView {
    SurfaceNoteView {
        label: note.label.clone(),
        text: note.reason.clone(),
        source: Some(note.source.clone()),
    }
}

fn unapplied_note(note: &UnappliedEffectView) -> SurfaceNoteView {
    SurfaceNoteView {
        label: note.label.clone(),
        text: note.reason.clone(),
        source: Some(note.source.clone()),
    }
}

fn record_header(
    record: &AtlasRecord,
    profile: RecordSurfaceProfileView,
) -> RecordSurfaceHeaderView {
    let trait_limit = match profile {
        RecordSurfaceProfileView::SearchCompact => 4,
        RecordSurfaceProfileView::RecordDetail | RecordSurfaceProfileView::EncounterParticipant => {
            usize::MAX
        }
    };
    RecordSurfaceHeaderView {
        level_label: record.classification.level.map(|level| level.to_string()),
        kind_label: Some(kind_label(record.classification.kind.as_str())),
        rarity: record
            .classification
            .rarity
            .as_ref()
            .map(|rarity| rarity.as_str().to_string()),
        traits: record
            .classification
            .traits
            .iter()
            .take(trait_limit)
            .map(|value| SurfaceBadgeView {
                kind: "trait".to_string(),
                label: value.clone(),
                value: value.clone(),
            })
            .collect(),
        publication: record.publication.title.clone(),
        pack: Some(record.foundry.pack_label.clone()),
    }
}

fn participant_header(participant: &EncounterParticipantView) -> RecordSurfaceHeaderView {
    RecordSurfaceHeaderView {
        level_label: None,
        kind_label: Some(format!("{:?}", participant.participant_kind).to_lowercase()),
        rarity: None,
        traits: Vec::new(),
        publication: None,
        pack: None,
    }
}

fn description_content(presentation: &RecordPresentationDocument) -> Option<PresentationContent> {
    presentation
        .sections
        .iter()
        .find(|section| section.kind == PresentationSectionKind::Description)
        .and_then(|section| {
            section.blocks.iter().find_map(|block| match block {
                PresentationBlock::Content(content) => Some(content.clone()),
                PresentationBlock::Prose(text) => Some(PresentationContent::new(vec![
                    atlas_record::PresentationContentBlock::Paragraph {
                        spans: vec![atlas_record::PresentationInline::Text {
                            text: text.text.clone(),
                        }],
                    },
                ])),
                PresentationBlock::FactList(_) | PresentationBlock::Relationships(_) => None,
            })
        })
}
