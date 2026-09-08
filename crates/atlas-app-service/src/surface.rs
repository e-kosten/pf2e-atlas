use std::collections::{BTreeMap, BTreeSet};

use crate::encounters::spells::unavailable_spell_cast;
use crate::projection::kind_label;
use crate::retrieval::VerifiedRemasterLookup;
use atlas_app_model::{
    CreatureSurfaceAbilitiesView, CreatureSurfaceActionCostView, CreatureSurfaceActivityTypeView,
    CreatureSurfaceActivityView, CreatureSurfaceAdjustmentValueView, CreatureSurfaceAdjustmentView,
    CreatureSurfaceAwarenessView, CreatureSurfaceContentBlockView,
    CreatureSurfaceContentInlineView, CreatureSurfaceContentListItemView,
    CreatureSurfaceContentProvenanceView, CreatureSurfaceContentRoleView,
    CreatureSurfaceContentTableRowView, CreatureSurfaceContentView, CreatureSurfaceDamageView,
    CreatureSurfaceDefensesView, CreatureSurfaceDomainUnavailableView,
    CreatureSurfaceEquipmentView, CreatureSurfaceFactOwnerView, CreatureSurfaceFactProvenanceView,
    CreatureSurfaceFrequencyView, CreatureSurfaceInitiativeView,
    CreatureSurfaceIntegerPresenceView, CreatureSurfaceIwrView, CreatureSurfaceLoreView,
    CreatureSurfaceMovementView, CreatureSurfaceOccurrenceIdentityStabilityView,
    CreatureSurfaceOccurrenceProvenanceView, CreatureSurfaceProvenanceView,
    CreatureSurfaceResourceView, CreatureSurfaceRitualsView, CreatureSurfaceRollView,
    CreatureSurfaceSaveView, CreatureSurfaceSavesView, CreatureSurfaceSelfEffectView,
    CreatureSurfaceSenseView, CreatureSurfaceShieldView, CreatureSurfaceSizeValueView,
    CreatureSurfaceSizeView, CreatureSurfaceSkillPredicateView,
    CreatureSurfaceSkillSourceEntryView, CreatureSurfaceSkillVariantView, CreatureSurfaceSkillView,
    CreatureSurfaceSourceFieldView, CreatureSurfaceSourceLocatorView,
    CreatureSurfaceSpellOccurrenceContextView, CreatureSurfaceSpellSlotView,
    CreatureSurfaceSpellView, CreatureSurfaceSpellcastingView, CreatureSurfaceUnavailableCauseView,
    CreatureSurfaceUnavailableDomainsView, CreatureSurfaceUnavailableFieldView,
    CreatureSurfaceUnavailableStateView, CreatureSurfaceUnmodeledSkillReasonView,
    CreatureSurfaceUnmodeledSkillView, CreatureSurfaceUsesView, CreatureSurfaceView,
    CreatureSurfaceVitalsView, EncounterRuntimeActivityKindView, EncounterRuntimeActivityView,
    EncounterRuntimeAutomationLimitationCodeView, EncounterRuntimeAutomationLimitationTargetView,
    EncounterRuntimeAutomationLimitationView, EncounterRuntimeSpellView,
    EncounterRuntimeSpellcastingView, EncounterRuntimeView,
    RecordSurfaceEditionCounterpartRoleView, RecordSurfaceEditionCounterpartView,
    RecordSurfaceEditionStatusView, RecordSurfaceEditionView, RecordSurfaceMetadataView,
    RecordSurfacePresentationView, RecordSurfaceProfileView, RecordSurfaceSourceView,
    RecordSurfaceView, RuntimeFactProvenanceView, RuntimeFactSourceView,
    SurfaceUnavailableReasonView, SurfaceUnavailableView,
};
use atlas_record::{
    ContentRole, CreatureActionCost, CreatureAdjustment, CreatureCapability,
    CreatureContentPlacement, CreatureDamage, CreatureDefenses, CreatureEmbeddedEntities,
    CreatureEntityOccurrence, CreatureEntityTarget, CreatureIwr, CreatureMovementMode,
    CreatureOccurrenceParent, CreaturePredicate, CreatureRecord, CreatureResourceAmount,
    CreatureRoll, CreatureRollKind, CreatureSize, CreatureSourceScalar, CreatureSpellPreparation,
    CreatureUnmodeledSkillReason, CreatureUseLimit, FactValue, PresentationContent,
    PresentationContentBlock, PresentationInline, RecordBody, RetrievedRecord, SenseAcuity,
    format_creature_frequency, place_creature_content_for_families, project_record_surface_content,
    render_plain_text,
};

const SEARCH_TEASER_WORDS: usize = 50;

pub(crate) fn record_surface(
    retrieved: &RetrievedRecord,
    profile: RecordSurfaceProfileView,
    mut encounter: Option<EncounterRuntimeView>,
    spell_selection: Option<(atlas_record::SpellFormId, u8)>,
    remaster_lookup: &VerifiedRemasterLookup,
) -> RecordSurfaceView {
    let metadata = record_metadata(retrieved, remaster_lookup);
    let mut selected_spell_issues = Vec::new();
    let mut content_issues = Vec::new();
    let presentation = match (&retrieved.record.classification.kind, &retrieved.body) {
        (atlas_domain::RecordKind::Creature, Some(RecordBody::Creature(creature))) => {
            let content_placement = activity_content_placement(creature);
            if let Some(runtime) = encounter.as_mut() {
                compose_encounter_payload(creature, &content_placement, runtime);
            }
            RecordSurfacePresentationView::Creature {
                body: Box::new(creature_surface_with_placement(
                    creature,
                    profile,
                    &content_placement,
                    (profile == RecordSurfaceProfileView::SearchCompact)
                        .then(|| search_teaser(retrieved))
                        .flatten(),
                )),
            }
        }
        (atlas_domain::RecordKind::Creature, _) => unavailable_presentation(
            &metadata.kind,
            SurfaceUnavailableReasonView::RecordUnavailable,
            "Canonical creature data is unavailable, so the surface fails closed.",
        ),
        (atlas_domain::RecordKind::Hazard, Some(RecordBody::Hazard(hazard))) => {
            RecordSurfacePresentationView::Hazard {
                body: Box::new(crate::hazard_surface::hazard_surface(
                    hazard,
                    profile,
                    (profile == RecordSurfaceProfileView::SearchCompact)
                        .then(|| search_teaser(retrieved))
                        .flatten(),
                )),
            }
        }
        (atlas_domain::RecordKind::Hazard, _) => unavailable_presentation(
            &metadata.kind,
            SurfaceUnavailableReasonView::RecordUnavailable,
            "Canonical hazard data is unavailable, so the surface fails closed.",
        ),
        (atlas_domain::RecordKind::Spell, Some(RecordBody::Spell(spell))) => {
            let (body, issues, projected_content_issues) = spell_surface(spell, spell_selection);
            content_issues = projected_content_issues;
            selected_spell_issues = issues;
            RecordSurfacePresentationView::Spell {
                body: Box::new(body),
            }
        }
        (atlas_domain::RecordKind::Spell, _) => unavailable_presentation(
            &metadata.kind,
            SurfaceUnavailableReasonView::RecordUnavailable,
            "Canonical spell data is unavailable, so the surface fails closed.",
        ),
        _ => unavailable_presentation(
            &metadata.kind,
            SurfaceUnavailableReasonView::RecordFamilyNotMigrated,
            "This record family does not yet have an approved typed app surface.",
        ),
    };
    let mut issues = crate::record_policy::record_surface_issues(
        retrieved,
        &presentation,
        &selected_spell_issues,
    );
    if !content_issues.is_empty() {
        issues.get_or_insert_with(Vec::new).extend(content_issues);
    }
    RecordSurfaceView {
        metadata,
        profile,
        presentation,
        issues,
        references: None,
        encounter,
    }
}

pub(crate) fn unavailable_participant_surface(
    record_key: Option<String>,
    title: String,
    kind: String,
    profile: RecordSurfaceProfileView,
    reason: SurfaceUnavailableReasonView,
    encounter: EncounterRuntimeView,
) -> RecordSurfaceView {
    let metadata = RecordSurfaceMetadataView {
        record_key,
        title,
        kind_label: kind_label(&kind),
        kind: kind.clone(),
        level: None,
        rarity: None,
        traits: Vec::new(),
        edition: None,
        source: None,
    };
    RecordSurfaceView {
        metadata,
        profile,
        presentation: unavailable_presentation(
            &kind,
            reason,
            "No canonical record body is available for this participant.",
        ),
        issues: Some(vec![atlas_app_model::RecordSurfaceIssueView {
            fact_id: None,
            code: atlas_app_model::RecordSurfaceIssueCodeView::Unavailable,
            placement: atlas_app_model::RecordSurfaceIssuePlacementView::Record,
            subject: None,
            fact_label: None,
            message: "This participant does not have an available typed record presentation."
                .to_string(),
        }]),
        references: None,
        encounter: Some(encounter),
    }
}

fn unavailable_presentation(
    requested_kind: &str,
    reason: SurfaceUnavailableReasonView,
    message: &str,
) -> RecordSurfacePresentationView {
    RecordSurfacePresentationView::Unavailable {
        unavailable: SurfaceUnavailableView {
            reason,
            requested_kind: requested_kind.to_string(),
            message: message.to_string(),
        },
    }
}

fn search_teaser(retrieved: &RetrievedRecord) -> Option<String> {
    let text = retrieved
        .record
        .content
        .primary_body()
        .map(render_plain_text)?;
    teaser_from_text(&text)
}

fn teaser_from_text(text: &str) -> Option<String> {
    let mut words = text.split_whitespace();
    let teaser = words
        .by_ref()
        .take(SEARCH_TEASER_WORDS)
        .collect::<Vec<_>>()
        .join(" ");
    (!teaser.is_empty()).then_some(teaser)
}

fn record_metadata(
    retrieved: &RetrievedRecord,
    remaster_lookup: &VerifiedRemasterLookup,
) -> RecordSurfaceMetadataView {
    let record = &retrieved.record;
    let spell_classification_owned_by_body =
        record.classification.kind == atlas_domain::RecordKind::Spell;
    let canonical_source_versions = retrieved.body.as_ref().map(|body| match body {
        RecordBody::Creature(creature) => (
            &creature.provenance.source_contract_version,
            &creature.provenance.source_system_version,
            &creature.provenance.source_upstream_commit,
        ),
        RecordBody::Hazard(hazard) => (
            &hazard.provenance.source_contract_version,
            &hazard.provenance.source_system_version,
            &hazard.provenance.source_upstream_commit,
        ),
        RecordBody::Spell(spell) => (
            &spell.definition.provenance.source_contract_version,
            &spell.definition.provenance.source_system_version,
            &spell.definition.provenance.source_upstream_commit,
        ),
    });
    RecordSurfaceMetadataView {
        record_key: Some(record.identity.key.to_string()),
        title: record.identity.name.clone(),
        kind: record.classification.kind.as_str().to_string(),
        kind_label: kind_label(record.classification.kind.as_str()),
        level: (!spell_classification_owned_by_body)
            .then_some(record.classification.level)
            .flatten(),
        rarity: record
            .classification
            .rarity
            .map(|rarity| rarity.as_str().to_string()),
        traits: if spell_classification_owned_by_body {
            Vec::new()
        } else {
            record.classification.traits.clone()
        },
        edition: Some(record_edition(retrieved, remaster_lookup)),
        source: Some(RecordSurfaceSourceView {
            publication_title: record.publication.title.clone(),
            pack_label: record.foundry.pack_label.clone(),
            document_type: record.foundry.document_type.as_str().to_string(),
            record_type: record.foundry.record_type.as_str().to_string(),
            source_path: Some(record.provenance.source_path.clone()),
            source_contract_version: canonical_source_versions
                .map(|(contract, _, _)| contract.clone()),
            source_system_version: canonical_source_versions.map(|(_, system, _)| system.clone()),
            source_upstream_commit: canonical_source_versions
                .map(|(_, _, upstream)| upstream.clone()),
        }),
    }
}

fn record_edition(
    retrieved: &RetrievedRecord,
    remaster_lookup: &VerifiedRemasterLookup,
) -> RecordSurfaceEditionView {
    let status = if retrieved.record.publication.remaster {
        RecordSurfaceEditionStatusView::Remaster
    } else {
        RecordSurfaceEditionStatusView::Legacy
    };
    let links = remaster_lookup.links();
    let counterparts = (links.seed.record.identity.key == retrieved.record.identity.key
        && links.seed.record.publication.remaster == retrieved.record.publication.remaster)
        .then_some(links)
        .into_iter()
        .flat_map(|links| &links.links)
        .filter_map(|link| verified_edition_counterpart(retrieved, link))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    RecordSurfaceEditionView {
        status,
        counterparts,
    }
}

fn verified_edition_counterpart(
    seed: &RetrievedRecord,
    link: &atlas_search::RemasterLinkResult,
) -> Option<RecordSurfaceEditionCounterpartView> {
    let seed_key = &seed.record.identity.key;
    let (seed_side, counterpart, role, expected_seed_remaster, expected_target_remaster) =
        if seed.record.publication.remaster {
            (
                &link.remaster_record,
                &link.legacy_record,
                RecordSurfaceEditionCounterpartRoleView::LegacyCounterpart,
                true,
                false,
            )
        } else {
            (
                &link.legacy_record,
                &link.remaster_record,
                RecordSurfaceEditionCounterpartRoleView::RemasteredCounterpart,
                false,
                true,
            )
        };
    if seed_side.record.identity.key != *seed_key
        || seed_side.record.publication.remaster != expected_seed_remaster
        || counterpart.record.identity.key == *seed_key
        || counterpart.record.publication.remaster != expected_target_remaster
    {
        return None;
    }
    Some(RecordSurfaceEditionCounterpartView {
        role,
        record_key: counterpart.record.identity.key.to_string(),
        title: counterpart.record.identity.name.clone(),
    })
}

fn spell_surface(
    spell: &atlas_record::SpellRecord,
    selection: Option<(atlas_record::SpellFormId, u8)>,
) -> (
    atlas_app_model::SpellSurfaceView,
    Vec<atlas_record::SpellPresentationIssue>,
    Vec<atlas_app_model::RecordSurfaceIssueView>,
) {
    let (forms, form_catalog_unavailable, mut issues) = spell_forms(spell);
    let base_rank = spell_base_rank(spell).unwrap_or(0);
    let (form_id, cast_rank) = selection.unwrap_or_else(|| {
        (
            atlas_record::SpellFormId::base(&spell.identity.record_key),
            base_rank,
        )
    });
    let (effective_form, effective_issues, content_context) =
        spell_effective_form(spell, form_id, cast_rank);
    issues.extend(effective_issues);
    let mut content_issues = Vec::new();
    let mut content = spell
        .definition
        .content
        .documents
        .iter()
        .filter_map(|document| {
            let projection = atlas_record::project_record_surface_content_with_context(
                &document.document,
                &content_context,
            );
            content_issues.extend(
                projection
                    .issues
                    .into_iter()
                    .map(|issue| spell_content_issue(document, issue)),
            );
            content_view_from_projection(document, projection.content)
        })
        .collect::<Vec<_>>();
    content.sort_by_key(|document| document.authored_order);
    (
        atlas_app_model::SpellSurfaceView {
            family: if spell_known(&spell.definition.ritual).is_some() {
                atlas_app_model::SpellFamilyView::Ritual
            } else {
                atlas_app_model::SpellFamilyView::Spell
            },
            forms,
            effective_form,
            form_catalog_unavailable,
            content,
        },
        issues,
        content_issues,
    )
}

fn spell_content_issue(
    document: &atlas_record::OwnedRichContentDocument,
    issue: atlas_record::RecordSurfaceContentIssue,
) -> atlas_app_model::RecordSurfaceIssueView {
    use atlas_app_model::{
        RecordSurfaceIssueCodeView as Code, RecordSurfaceIssuePlacementView as Placement,
    };
    let (code, message) = match issue.kind {
        atlas_record::RecordSurfaceContentIssueKind::MissingContext => (
            Code::Unavailable,
            "This description damage needs an available spell form and cast rank.",
        ),
        atlas_record::RecordSurfaceContentIssueKind::UnsupportedDamage => (
            Code::Unsupported,
            "This description damage formula cannot be displayed for the selected cast rank.",
        ),
    };
    atlas_app_model::RecordSurfaceIssueView {
        fact_id: Some(format!(
            "{}:damage:{}",
            document.id.content_key.as_str(),
            issue.inline_index
        )),
        code,
        placement: Placement::Record,
        subject: None,
        fact_label: Some("Description damage".to_string()),
        message: message.to_string(),
    }
}

fn spell_fact<T, U>(
    fact: &atlas_record::SpellFact<T>,
    map: impl FnOnce(&T) -> U,
) -> atlas_app_model::SpellFactView<U> {
    match fact {
        FactValue::Missing => atlas_app_model::SpellFactView::Missing,
        FactValue::Null => atlas_app_model::SpellFactView::Null,
        FactValue::Value(atlas_record::SpellSourceValue::Known(value)) => {
            atlas_app_model::SpellFactView::Known(map(value))
        }
        FactValue::Value(atlas_record::SpellSourceValue::Unsupported(_)) => {
            atlas_app_model::SpellFactView::Unsupported
        }
    }
}

fn spell_source_value<T, U>(
    value: &atlas_record::SpellSourceValue<T>,
    map: impl FnOnce(&T) -> U,
) -> atlas_app_model::SpellSourceValueView<U> {
    match value {
        atlas_record::SpellSourceValue::Known(value) => {
            atlas_app_model::SpellSourceValueView::Known(map(value))
        }
        atlas_record::SpellSourceValue::Unsupported(_) => {
            atlas_app_model::SpellSourceValueView::Unsupported
        }
    }
}

fn spell_classification(
    value: &atlas_record::SpellClassification,
) -> atlas_app_model::SpellClassificationView {
    atlas_app_model::SpellClassificationView {
        rank: spell_fact(&value.rank, |value| *value),
        traits: spell_fact(&value.traits, |values| {
            values
                .iter()
                .map(|value| value.as_str().to_string())
                .collect()
        }),
        traditions: spell_fact(&value.traditions, |values| {
            values
                .iter()
                .map(|value| value.as_str().to_string())
                .collect()
        }),
    }
}

fn spell_casting(value: &atlas_record::SpellCasting) -> atlas_app_model::SpellCastingView {
    atlas_app_model::SpellCastingView {
        action_cost: atlas_record::project_spell_action_cost(value).and_then(|cost| match cost {
            CreatureActionCost::Actions(count) => {
                Some(CreatureSurfaceActionCostView::Actions { count })
            }
            CreatureActionCost::Reaction => Some(CreatureSurfaceActionCostView::Reaction),
            CreatureActionCost::FreeAction => Some(CreatureSurfaceActionCostView::FreeAction),
            CreatureActionCost::Passive
            | CreatureActionCost::Time(_)
            | CreatureActionCost::Unsupported(_) => None,
        }),
        time: spell_fact(&value.time, Clone::clone),
        cost: spell_fact(&value.cost, Clone::clone),
        requirements: spell_fact(&value.requirements, Clone::clone),
        counteraction: spell_fact(&value.counteraction, |value| *value),
    }
}

fn spell_targeting(value: &atlas_record::SpellTargeting) -> atlas_app_model::SpellTargetingView {
    atlas_app_model::SpellTargetingView {
        target: spell_fact(&value.target, Clone::clone),
        range: spell_fact(&value.range, spell_range),
        area: spell_fact(&value.area, spell_area),
    }
}

fn spell_range(value: &atlas_record::SpellRangeValue) -> atlas_app_model::SpellRangeView {
    atlas_app_model::SpellRangeView {
        authored_text: value.authored_text.clone(),
    }
}

fn spell_area(value: &atlas_record::SpellAreaValue) -> atlas_app_model::SpellAreaView {
    atlas_app_model::SpellAreaView {
        value: spell_fact(&value.value, |value| *value),
        area_type: spell_fact(&value.area_type, |value| value.as_str().to_string()),
        legacy_area_type: spell_fact(&value.legacy_area_type, |value| {
            value.value.as_str().to_string()
        }),
        details: spell_fact(&value.details, Clone::clone),
    }
}

fn spell_defense(value: &atlas_record::SpellDefenseValue) -> atlas_app_model::SpellDefenseView {
    atlas_app_model::SpellDefenseView {
        passive: spell_fact(&value.passive, |value| value.as_str().to_string()),
        save: spell_fact(&value.save, spell_save),
    }
}

fn spell_save(value: &atlas_record::SpellSave) -> atlas_app_model::SpellSaveView {
    atlas_app_model::SpellSaveView {
        statistic: spell_fact(&value.statistic, |value| value.as_str().to_string()),
        basic: spell_fact(&value.basic, |value| *value),
    }
}

fn spell_damage(
    member: &atlas_record::SpellOrderedMember<atlas_record::SpellDamage>,
) -> atlas_app_model::SpellDamageView {
    let value = &member.value;
    atlas_app_model::SpellDamageView {
        label: spell_damage_label(value),
        formula: spell_fact(&value.formula, Clone::clone),
        damage_type: spell_fact(&value.damage_type, Clone::clone),
        category: spell_fact(&value.category, Clone::clone),
        kinds: spell_fact(&value.kinds, Clone::clone),
        materials: spell_fact(&value.materials, Clone::clone),
        apply_modifier: spell_fact(&value.apply_mod, |value| *value),
    }
}

fn spell_duration(value: &atlas_record::SpellDuration) -> atlas_app_model::SpellDurationView {
    atlas_app_model::SpellDurationView {
        value: spell_fact(&value.value, Clone::clone),
        sustained: spell_fact(&value.sustained, |value| *value),
    }
}

fn spell_heightening(
    value: &atlas_record::SpellHeightening,
    base_damage: Option<&[atlas_record::SpellOrderedMember<atlas_record::SpellDamage>]>,
) -> atlas_app_model::SpellHeighteningView {
    match value {
        atlas_record::SpellHeightening::Interval(value) => {
            atlas_app_model::SpellHeighteningView::Interval {
                interval: spell_fact(&value.interval, |value| *value),
                area: spell_fact(&value.area, |value| *value),
                damage: spell_fact(&value.damage, |members| {
                    members
                        .iter()
                        .map(|member| atlas_app_model::SpellHeighteningDamageView {
                            label: unique_spell_damage_member(base_damage, &member.key)
                                .map(|member| spell_damage_label(&member.value))
                                .unwrap_or_else(|| "Effect".to_string()),
                            value: member.value.clone(),
                        })
                        .collect()
                }),
            }
        }
        atlas_record::SpellHeightening::Fixed(layers) => {
            atlas_app_model::SpellHeighteningView::Fixed {
                layers: layers
                    .iter()
                    .map(|layer| atlas_app_model::SpellFixedHeighteningView {
                        rank: spell_source_value(&layer.rank, |value| *value),
                        changes: spell_fixed_heightening_changes(&layer.patch, base_damage),
                    })
                    .collect(),
            }
        }
    }
}

fn spell_fixed_heightening_changes(
    patch: &atlas_record::SpellPatch,
    base_damage: Option<&[atlas_record::SpellOrderedMember<atlas_record::SpellDamage>]>,
) -> Vec<atlas_app_model::SpellFixedHeighteningChangeView> {
    use atlas_app_model::SpellFixedHeighteningChangeView as Change;

    let mut changes = Vec::new();
    for (present, change) in [
        (
            spell_known(&patch.classification).is_some(),
            Change::Classification,
        ),
        (spell_known(&patch.casting).is_some(), Change::Casting),
        (spell_known(&patch.targeting).is_some(), Change::Targeting),
        (spell_known(&patch.defense).is_some(), Change::Defense),
        (spell_known(&patch.duration).is_some(), Change::Duration),
        (
            spell_known(&patch.heightening).is_some(),
            Change::Heightening,
        ),
        (spell_known(&patch.rules).is_some(), Change::Rules),
    ] {
        if present {
            changes.push(change);
        }
    }
    if let Some(damage) = spell_known(&patch.damage) {
        changes.extend(damage.members.iter().map(|member| {
            let base = unique_spell_damage_member(base_damage, &member.key);
            match &member.operation {
                atlas_record::SpellKeyedPatchOperation::Merge(value) => Change::Effect {
                    operation: atlas_app_model::SpellEffectChangeOperationView::Merge,
                    label: spell_damage_patch_label(value, base.map(|member| &member.value)),
                    value: Some(spell_effect_change(value)),
                },
                atlas_record::SpellKeyedPatchOperation::Delete => Change::Effect {
                    operation: atlas_app_model::SpellEffectChangeOperationView::Delete,
                    label: base
                        .map(|member| spell_damage_label(&member.value))
                        .unwrap_or_else(|| "Effect".to_string()),
                    value: None,
                },
                atlas_record::SpellKeyedPatchOperation::Unsupported(_) => Change::Effect {
                    operation: atlas_app_model::SpellEffectChangeOperationView::Unsupported,
                    label: "Effect".to_string(),
                    value: None,
                },
            }
        }));
    }
    changes
}

fn unique_spell_damage_member<'a>(
    damage: Option<&'a [atlas_record::SpellOrderedMember<atlas_record::SpellDamage>]>,
    key: &str,
) -> Option<&'a atlas_record::SpellOrderedMember<atlas_record::SpellDamage>> {
    let mut matches = damage?.iter().filter(|member| member.key == key);
    let member = matches.next()?;
    matches.next().is_none().then_some(member)
}

fn spell_damage_patch_label(
    patch: &atlas_record::SpellDamagePatch,
    base: Option<&atlas_record::SpellDamage>,
) -> String {
    if spell_known(&patch.category).is_some_and(|value| !value.trim().is_empty())
        || spell_known(&patch.kinds).is_some_and(|values| !values.is_empty())
    {
        return spell_damage_semantic_label(&patch.category, &patch.kinds);
    }
    if matches!(
        patch.category,
        FactValue::Value(atlas_record::SpellSourceValue::Unsupported(_))
    ) || matches!(
        patch.kinds,
        FactValue::Value(atlas_record::SpellSourceValue::Unsupported(_))
    ) {
        return "Effect".to_string();
    }
    base.map(spell_damage_label)
        .unwrap_or_else(|| "Effect".to_string())
}

fn spell_damage_label(value: &atlas_record::SpellDamage) -> String {
    spell_damage_semantic_label(&value.category, &value.kinds)
}

fn spell_damage_semantic_label(
    category: &atlas_record::SpellFact<String>,
    kinds: &atlas_record::SpellFact<Vec<String>>,
) -> String {
    if let Some(category) = spell_known(category).filter(|value| !value.trim().is_empty()) {
        let mut label = category.clone();
        if let Some(first) = label.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        return format!("{label} damage");
    }
    let kinds = spell_known(kinds).map(Vec::as_slice).unwrap_or_default();
    match (
        kinds.iter().any(|kind| kind == "damage"),
        kinds.iter().any(|kind| kind == "healing"),
    ) {
        (true, true) => "Damage or healing",
        (false, true) => "Healing",
        (true, false) => "Damage",
        (false, false) => "Effect",
    }
    .to_string()
}

fn spell_effect_change(
    value: &atlas_record::SpellDamagePatch,
) -> atlas_app_model::SpellEffectChangeView {
    atlas_app_model::SpellEffectChangeView {
        formula: spell_fact(&value.formula, Clone::clone),
        damage_type: spell_fact(&value.damage_type, Clone::clone),
        category: spell_fact(&value.category, Clone::clone),
        kinds: spell_fact(&value.kinds, Clone::clone),
        materials: spell_fact(&value.materials, Clone::clone),
        apply_modifier: spell_fact(&value.apply_mod, |value| *value),
    }
}

fn spell_ritual(value: &atlas_record::SpellRitual) -> atlas_app_model::SpellRitualView {
    atlas_app_model::SpellRitualView {
        primary_check: spell_fact(&value.primary_check, Clone::clone),
        secondary_casters: spell_fact(&value.secondary_casters, |value| *value),
        secondary_checks: spell_fact(&value.secondary_checks, Clone::clone),
    }
}

fn spell_rules(values: &[atlas_record::SpellRuleElement]) -> Vec<atlas_app_model::SpellRuleView> {
    values
        .iter()
        .map(|value| atlas_app_model::SpellRuleView {
            order: value.authored_order,
            rule: match &value.rule {
                atlas_record::SpellRule::DamageDice(rule) => {
                    atlas_app_model::SpellRuleDetailView::DamageDice(
                        atlas_app_model::SpellDamageDiceRuleView {
                            selector: spell_fact(&rule.selector, Clone::clone),
                            predicate: spell_rule_predicates(&rule.predicate),
                            dice_number: spell_fact(&rule.dice_number, Clone::clone),
                            die_size: spell_fact(&rule.die_size, Clone::clone),
                            damage_type: spell_fact(&rule.damage_type, Clone::clone),
                            hide_if_disabled: spell_fact(&rule.hide_if_disabled, |value| *value),
                        },
                    )
                }
                atlas_record::SpellRule::EphemeralEffect(rule) => {
                    atlas_app_model::SpellRuleDetailView::EphemeralEffect(
                        atlas_app_model::SpellEphemeralEffectRuleView {
                            predicate: spell_rule_predicates(&rule.predicate),
                            selectors: spell_fact(&rule.selectors, Clone::clone),
                            uuid: spell_fact(&rule.uuid, Clone::clone),
                        },
                    )
                }
                atlas_record::SpellRule::DamageAlteration(rule) => {
                    atlas_app_model::SpellRuleDetailView::DamageAlteration(
                        atlas_app_model::SpellDamageAlterationRuleView {
                            mode: spell_fact(&rule.mode, Clone::clone),
                            predicate: spell_rule_predicates(&rule.predicate),
                            property: spell_fact(&rule.property, Clone::clone),
                            selectors: spell_fact(&rule.selectors, Clone::clone),
                            slug: spell_fact(&rule.slug, Clone::clone),
                            value: spell_fact(&rule.value, Clone::clone),
                        },
                    )
                }
                atlas_record::SpellRule::RollOption(rule) => {
                    atlas_app_model::SpellRuleDetailView::RollOption(
                        atlas_app_model::SpellRollOptionRuleView {
                            domain: spell_fact(&rule.domain, Clone::clone),
                            label: spell_fact(&rule.label, Clone::clone),
                            option: spell_fact(&rule.option, Clone::clone),
                            placement: spell_fact(&rule.placement, Clone::clone),
                            predicate: spell_rule_predicates(&rule.predicate),
                            suboptions: spell_fact(&rule.suboptions, |suboptions| {
                                suboptions
                                    .iter()
                                    .map(|suboption| atlas_app_model::SpellRuleSuboptionView {
                                        label: spell_fact(&suboption.label, Clone::clone),
                                        value: spell_fact(&suboption.value, Clone::clone),
                                    })
                                    .collect()
                            }),
                            toggleable: spell_fact(&rule.toggleable, |value| *value),
                        },
                    )
                }
                atlas_record::SpellRule::ItemAlteration(rule) => {
                    atlas_app_model::SpellRuleDetailView::ItemAlteration(
                        atlas_app_model::SpellItemAlterationRuleView {
                            item_id: spell_fact(&rule.item_id, Clone::clone),
                            mode: spell_fact(&rule.mode, Clone::clone),
                            predicate: spell_rule_predicates(&rule.predicate),
                            property: spell_fact(&rule.property, Clone::clone),
                            value: spell_fact(&rule.value, Clone::clone),
                        },
                    )
                }
                atlas_record::SpellRule::Unsupported(_) => {
                    atlas_app_model::SpellRuleDetailView::Unsupported
                }
            },
        })
        .collect()
}

fn spell_rule_predicates(
    fact: &atlas_record::SpellFact<Vec<atlas_record::SpellRulePredicate>>,
) -> atlas_app_model::SpellFactView<Vec<atlas_app_model::SpellRulePredicateView>> {
    spell_fact(fact, |values| {
        values
            .iter()
            .map(|value| match value {
                atlas_record::SpellRulePredicate::Term(value) => {
                    atlas_app_model::SpellRulePredicateView::Term(value.clone())
                }
                atlas_record::SpellRulePredicate::Or(values) => {
                    atlas_app_model::SpellRulePredicateView::Or(values.clone())
                }
                atlas_record::SpellRulePredicate::Unsupported(_) => {
                    atlas_app_model::SpellRulePredicateView::Unsupported
                }
            })
            .collect()
    })
}

fn spell_forms(
    spell: &atlas_record::SpellRecord,
) -> (
    Vec<atlas_app_model::SpellFormView>,
    Option<atlas_app_model::SpellFormCatalogUnavailableReasonView>,
    Vec<atlas_record::SpellPresentationIssue>,
) {
    let cast_rank = spell_base_rank(spell).unwrap_or(0);
    let base_id = atlas_record::SpellFormId::base(&spell.identity.record_key);
    let mut forms = vec![spell_form(base_id, "Base".to_string(), 0, cast_rank, false)];
    let mut issues = Vec::new();
    match spell.ordered_overlays() {
        Ok(overlays) => {
            for (index, overlay) in overlays.into_iter().enumerate() {
                let form_id = overlay.form_id(&spell.identity.record_key);
                let context = atlas_record::SpellFormContext {
                    cast_rank,
                    overlay_id: Some(overlay.overlay_id.clone()),
                };
                let resolved = spell.resolve_form(form_id.clone(), context);
                let label = resolved.as_ref().ok().and_then(|resolved| {
                    spell_known(&overlay.name)
                        .filter(|name| !name.trim().is_empty())
                        .cloned()
                        .or_else(|| atlas_record::project_resolved_spell_form_label(resolved))
                });
                if let Some(label) = label {
                    forms.push(spell_form(
                        form_id,
                        label,
                        u32::try_from(index + 1).unwrap_or(u32::MAX),
                        cast_rank,
                        true,
                    ));
                } else {
                    issues.push(atlas_record::SpellPresentationIssue {
                        field: atlas_record::SpellPresentationIssueField::FormCatalog,
                        kind: atlas_record::FactIssueKind::Unavailable,
                    });
                }
            }
            (forms, None, issues)
        }
        Err(error) => (
            forms,
            Some(match error.reason {
                atlas_record::SpellOverlayOrderFailure::UnsupportedRoot => {
                    atlas_app_model::SpellFormCatalogUnavailableReasonView::UnsupportedOverlayRoot
                }
                atlas_record::SpellOverlayOrderFailure::UnavailableSort => {
                    atlas_app_model::SpellFormCatalogUnavailableReasonView::UnavailableOverlaySort
                }
            }),
            issues,
        ),
    }
}

fn spell_base_rank(spell: &atlas_record::SpellRecord) -> Option<u8> {
    let classification = spell_known(&spell.definition.classification)?;
    spell_known(&classification.rank).copied()
}

fn spell_known<T>(fact: &atlas_record::SpellFact<T>) -> Option<&T> {
    match fact {
        FactValue::Value(atlas_record::SpellSourceValue::Known(value)) => Some(value),
        FactValue::Missing
        | FactValue::Null
        | FactValue::Value(atlas_record::SpellSourceValue::Unsupported(_)) => None,
    }
}

fn spell_form(
    form_id: atlas_record::SpellFormId,
    label: String,
    order: u32,
    cast_rank: u8,
    overlay: bool,
) -> atlas_app_model::SpellFormView {
    atlas_app_model::SpellFormView {
        id: form_id.as_str().to_string(),
        label,
        order,
        minimum_cast_rank: cast_rank,
        kind: if overlay {
            atlas_app_model::SpellFormKindView::Overlay
        } else {
            atlas_app_model::SpellFormKindView::Base
        },
    }
}

fn spell_effective_form(
    spell: &atlas_record::SpellRecord,
    form_id: atlas_record::SpellFormId,
    cast_rank: u8,
) -> (
    atlas_app_model::SpellEffectiveFormView,
    Vec<atlas_record::SpellPresentationIssue>,
    atlas_record::RecordSurfaceContentContext,
) {
    let base_id = atlas_record::SpellFormId::base(&spell.identity.record_key);
    let overlay_id = (form_id != base_id)
        .then(|| {
            spell
                .ordered_overlays()
                .ok()?
                .into_iter()
                .find(|overlay| overlay.form_id(&spell.identity.record_key) == form_id)
                .map(|overlay| overlay.overlay_id.clone())
        })
        .flatten();
    let result = spell.resolve_form(
        form_id.clone(),
        atlas_record::SpellFormContext {
            cast_rank,
            overlay_id,
        },
    );
    let content_context = result.as_ref().map_or(
        atlas_record::RecordSurfaceContentContext::UnavailableSpellSelection,
        |resolved| atlas_record::RecordSurfaceContentContext::Spell {
            form_id: resolved.form_id.clone(),
            cast_rank: resolved.context.cast_rank,
        },
    );
    let (form_id, cast_rank) = match &content_context {
        atlas_record::RecordSurfaceContentContext::Spell { form_id, cast_rank } => {
            (form_id.clone(), *cast_rank)
        }
        atlas_record::RecordSurfaceContentContext::UnavailableSpellSelection => {
            (form_id, cast_rank)
        }
    };
    let issues = atlas_record::project_spell_form_result_presentation_issues(&result);
    let base_damage = spell_known(&spell.definition.damage).map(Vec::as_slice);
    let result = spell_form_result_view(result, &spell.definition.ritual, base_damage);
    (
        atlas_app_model::SpellEffectiveFormView {
            id: form_id.as_str().to_string(),
            cast_rank,
            result,
        },
        issues,
        content_context,
    )
}

fn spell_form_result_view(
    result: Result<atlas_record::ResolvedSpellForm, atlas_record::SpellFormSelectionError>,
    ritual: &atlas_record::SpellFact<atlas_record::SpellRitual>,
    base_damage: Option<&[atlas_record::SpellOrderedMember<atlas_record::SpellDamage>]>,
) -> atlas_app_model::SpellFormResultView {
    match result {
        Ok(definition) => atlas_app_model::SpellFormResultView::Available {
            definition: Box::new(spell_resolved_definition(definition, ritual, base_damage)),
        },
        Err(error) => atlas_app_model::SpellFormResultView::Unavailable {
            reason: spell_selection_unavailable(error),
        },
    }
}

fn spell_resolved_definition(
    value: atlas_record::ResolvedSpellForm,
    ritual: &atlas_record::SpellFact<atlas_record::SpellRitual>,
    base_damage: Option<&[atlas_record::SpellOrderedMember<atlas_record::SpellDamage>]>,
) -> atlas_app_model::SpellResolvedDefinitionView {
    let heightening = spell_resolved_field(value.heightening, |heightening| {
        spell_heightening(heightening, base_damage)
    });
    atlas_app_model::SpellResolvedDefinitionView {
        applied_fixed_ranks: value.applied_fixed_ranks,
        classification: spell_resolved_field(value.classification, spell_classification),
        casting: spell_resolved_field(value.casting, spell_casting),
        targeting: spell_resolved_field(value.targeting, spell_targeting),
        defense: spell_resolved_field(value.defense, spell_defense),
        damage: spell_resolved_field(value.damage, |members| {
            members.iter().map(spell_damage).collect()
        }),
        duration: spell_resolved_field(value.duration, spell_duration),
        heightening,
        ritual: spell_fact(ritual, spell_ritual),
        rules: spell_resolved_field(value.rules, |values| spell_rules(values)),
    }
}

fn spell_resolved_field<T, U>(
    value: atlas_record::SpellResolvedField<atlas_record::SpellFact<T>>,
    map: impl FnOnce(&T) -> U,
) -> atlas_app_model::SpellResolvedFieldView<U> {
    match value {
        atlas_record::SpellResolvedField::Available(value) => {
            atlas_app_model::SpellResolvedFieldView::Available {
                value: spell_fact(&value, map),
            }
        }
        atlas_record::SpellResolvedField::Unavailable(unavailable) => {
            atlas_app_model::SpellResolvedFieldView::Unavailable {
                unavailable: atlas_app_model::SpellFormFieldUnavailableView {
                    field: spell_form_field(unavailable.field),
                    source: match unavailable.source {
                        atlas_record::SpellFormPatchSource::Base => {
                            atlas_app_model::SpellFormPatchSourceView::Base
                        }
                        atlas_record::SpellFormPatchSource::Overlay => {
                            atlas_app_model::SpellFormPatchSourceView::Overlay
                        }
                        atlas_record::SpellFormPatchSource::FixedHeightening => {
                            atlas_app_model::SpellFormPatchSourceView::FixedHeightening
                        }
                        atlas_record::SpellFormPatchSource::IntervalHeightening => {
                            atlas_app_model::SpellFormPatchSourceView::IntervalHeightening
                        }
                    },
                    reason: match unavailable.reason {
                        atlas_record::SpellFormUnavailableReason::UnsupportedPatch => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::UnsupportedPatch
                        }
                        atlas_record::SpellFormUnavailableReason::DuplicateKey(_) => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::DuplicateKey
                        }
                        atlas_record::SpellFormUnavailableReason::IncompatibleHeightening => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::IncompatibleHeightening
                        }
                        atlas_record::SpellFormUnavailableReason::UnknownFixedRank(_) => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::UnknownFixedRank
                        }
                        atlas_record::SpellFormUnavailableReason::FixedRankKeyMismatch { .. } => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::FixedRankKeyMismatch
                        }
                        atlas_record::SpellFormUnavailableReason::InvalidInterval => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::InvalidInterval
                        }
                        atlas_record::SpellFormUnavailableReason::IntervalMemberUnavailable => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::IntervalMemberUnavailable
                        }
                        atlas_record::SpellFormUnavailableReason::IntervalFormulaUnsupported => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::IntervalFormulaUnsupported
                        }
                        atlas_record::SpellFormUnavailableReason::IntervalOverflow => {
                            atlas_app_model::SpellFormFieldUnavailableReasonView::IntervalOverflow
                        }
                    },
                },
            }
        }
    }
}

fn spell_form_field(value: atlas_record::SpellFormField) -> atlas_app_model::SpellFormFieldView {
    match value {
        atlas_record::SpellFormField::Classification => {
            atlas_app_model::SpellFormFieldView::Classification
        }
        atlas_record::SpellFormField::Casting => atlas_app_model::SpellFormFieldView::Casting,
        atlas_record::SpellFormField::Targeting => atlas_app_model::SpellFormFieldView::Targeting,
        atlas_record::SpellFormField::Defense => atlas_app_model::SpellFormFieldView::Defense,
        atlas_record::SpellFormField::Damage => atlas_app_model::SpellFormFieldView::Damage,
        atlas_record::SpellFormField::Duration => atlas_app_model::SpellFormFieldView::Duration,
        atlas_record::SpellFormField::Heightening => {
            atlas_app_model::SpellFormFieldView::Heightening
        }
        atlas_record::SpellFormField::Rules => atlas_app_model::SpellFormFieldView::Rules,
    }
}

fn spell_selection_unavailable(
    error: atlas_record::SpellFormSelectionError,
) -> atlas_app_model::SpellFormSelectionUnavailableReasonView {
    match error {
        atlas_record::SpellFormSelectionError::BaseRankUnavailable => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::BaseRankUnavailable
        }
        atlas_record::SpellFormSelectionError::CastRankBelowBase {
            base_rank,
            cast_rank,
        } => atlas_app_model::SpellFormSelectionUnavailableReasonView::CastRankBelowBase {
            base_rank,
            cast_rank,
        },
        atlas_record::SpellFormSelectionError::UnknownOverlay(_) => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::UnknownOverlay
        }
        atlas_record::SpellFormSelectionError::OverlayRootUnavailable => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::OverlayRootUnavailable
        }
        atlas_record::SpellFormSelectionError::DuplicateOverlay(_) => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::DuplicateOverlay
        }
        atlas_record::SpellFormSelectionError::OverlayIdentityUnavailable(_) => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::OverlayIdentityUnavailable
        }
        atlas_record::SpellFormSelectionError::OverlayIdentityMismatch { .. } => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::OverlayIdentityMismatch
        }
        atlas_record::SpellFormSelectionError::UnsupportedOverlayType(_) => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::UnsupportedOverlayType
        }
        atlas_record::SpellFormSelectionError::FormIdMismatch => {
            atlas_app_model::SpellFormSelectionUnavailableReasonView::FormIdMismatch
        }
    }
}

#[cfg(test)]
fn creature_surface(
    creature: &CreatureRecord,
    profile: RecordSurfaceProfileView,
) -> CreatureSurfaceView {
    let activity_content = activity_content_placement(creature);
    creature_surface_with_placement(creature, profile, &activity_content, None)
}

fn creature_surface_with_placement(
    creature: &CreatureRecord,
    profile: RecordSurfaceProfileView,
    activity_content: &CreatureContentPlacement,
    teaser: Option<String>,
) -> CreatureSurfaceView {
    let detail = profile == RecordSurfaceProfileView::RecordDetail;
    let encounter = profile == RecordSurfaceProfileView::EncounterParticipant;
    let compact = profile == RecordSurfaceProfileView::SearchCompact;
    let mut unavailable = SurfaceUnavailableDomains::default();
    let defenses = match &creature.defenses.value {
        FactValue::Value(defenses) => Some(defenses),
        FactValue::Missing => {
            unavailable
                .add_selected_defenses(profile, CreatureSurfaceUnavailableStateView::Missing);
            None
        }
        FactValue::Null => {
            unavailable.add_selected_defenses(profile, CreatureSurfaceUnavailableStateView::Null);
            None
        }
    };

    let vitals = if encounter {
        None
    } else {
        defenses.and_then(|defenses| vitals(defenses, &mut unavailable))
    };
    let defenses_view = if detail {
        defenses.map(|defenses| defenses_view(defenses, &mut unavailable))
    } else if compact {
        defenses.map(|defenses| compact_defenses_view(defenses, &mut unavailable))
    } else if encounter {
        defenses.and_then(|defenses| encounter_defenses_view(defenses, &mut unavailable))
    } else {
        None
    };
    let saves = if detail {
        defenses.and_then(|defenses| saves_view(defenses, &mut unavailable))
    } else if compact {
        defenses.and_then(|defenses| compact_saves_view(defenses, &mut unavailable))
    } else if encounter {
        defenses.and_then(|defenses| encounter_saves_view(defenses, &mut unavailable))
    } else {
        None
    };
    let awareness = if detail {
        awareness(creature, &mut unavailable, true)
    } else if compact {
        compact_awareness(creature, &mut unavailable)
    } else if encounter {
        awareness(creature, &mut unavailable, false)
    } else {
        None
    };
    let abilities = detail
        .then(|| abilities(creature, &mut unavailable))
        .flatten();
    let (skills, unmodeled_skills) = if detail {
        skills(creature, &mut unavailable)
    } else {
        (None, None)
    };
    let movement = detail
        .then(|| movement(creature, &mut unavailable))
        .flatten();
    let resources = detail
        .then(|| resources(creature, &mut unavailable))
        .flatten();
    let rituals = detail
        .then(|| rituals(creature, &mut unavailable))
        .flatten();
    let equipment = detail
        .then(|| equipment(creature, &mut unavailable))
        .flatten();
    let lore = detail.then(|| lore(creature, &mut unavailable)).flatten();
    let (spellcasting, standalone_spells) = if detail {
        spellcasting(creature, activity_content, &mut unavailable)
    } else {
        (None, None)
    };
    let activities = if detail {
        activities(creature, activity_content, &mut unavailable)
    } else {
        if encounter {
            register_encounter_activity_trait_unavailability(creature, &mut unavailable);
        }
        None
    };
    CreatureSurfaceView {
        teaser,
        size: detail.then(|| size(creature, &mut unavailable)).flatten(),
        adjustment: detail
            .then(|| adjustment(creature, &mut unavailable))
            .flatten(),
        initiative: detail
            .then(|| initiative(creature, &mut unavailable))
            .flatten(),
        vitals,
        defenses: defenses_view,
        saves,
        awareness,
        abilities,
        skills,
        unmodeled_skills,
        movement,
        resources,
        rituals,
        equipment,
        lore,
        spellcasting,
        standalone_spells,
        activities,
        content: (detail || encounter)
            .then(|| content(creature, activity_content))
            .flatten(),
        relationships: None,
        unavailable_domains: unavailable.into_view(),
        provenance: Some(CreatureSurfaceProvenanceView {
            source_path: creature.provenance.source_path.clone(),
            source_contract_version: creature.provenance.source_contract_version.clone(),
            source_system_version: creature.provenance.source_system_version.clone(),
            source_upstream_commit: creature.provenance.source_upstream_commit.clone(),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SurfaceDomain {
    Classification,
    Initiative,
    Vitals,
    Defenses,
    Saves,
    Awareness,
    Abilities,
    Skills,
    Movement,
    Resources,
    Equipment,
    Lore,
    Spellcasting,
    Activities,
}

#[derive(Default)]
struct SurfaceUnavailableDomains {
    causes: BTreeMap<SurfaceDomain, Vec<CreatureSurfaceUnavailableCauseView>>,
}

impl SurfaceUnavailableDomains {
    fn add(
        &mut self,
        domain: SurfaceDomain,
        state: CreatureSurfaceUnavailableStateView,
        field: CreatureSurfaceUnavailableFieldView,
        source_field: CreatureSurfaceSourceFieldView,
        component_id: Option<String>,
    ) {
        let cause = CreatureSurfaceUnavailableCauseView {
            state,
            field,
            component_id,
            provenance: fact_provenance(source_field),
            unmodeled_skill: None,
            message: match state {
                CreatureSurfaceUnavailableStateView::Missing
                | CreatureSurfaceUnavailableStateView::Null => "A value is required here.",
                CreatureSurfaceUnavailableStateView::Unsupported => {
                    "The authored value is not supported."
                }
            }
            .to_string(),
        };
        let causes = self.causes.entry(domain).or_default();
        if !causes.contains(&cause) {
            causes.push(cause);
            causes.sort();
        }
    }

    fn add_unmodeled_skill(
        &mut self,
        component_id: String,
        value: CreatureSurfaceUnmodeledSkillView,
    ) {
        let cause = CreatureSurfaceUnavailableCauseView {
            state: CreatureSurfaceUnavailableStateView::Unsupported,
            field: CreatureSurfaceUnavailableFieldView::UnmodeledSkill,
            component_id: Some(component_id),
            provenance: fact_provenance(CreatureSurfaceSourceFieldView::Skills),
            unmodeled_skill: Some(value),
            message: "The source supplied an unrecognized skill key.".to_string(),
        };
        let causes = self.causes.entry(SurfaceDomain::Skills).or_default();
        if !causes.contains(&cause) {
            causes.push(cause);
            causes.sort();
        }
    }

    fn add_selected_defenses(
        &mut self,
        profile: RecordSurfaceProfileView,
        state: CreatureSurfaceUnavailableStateView,
    ) {
        if profile != RecordSurfaceProfileView::EncounterParticipant {
            self.add(
                SurfaceDomain::Vitals,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
            self.add(
                SurfaceDomain::Defenses,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
        } else {
            self.add(
                SurfaceDomain::Defenses,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
            self.add(
                SurfaceDomain::Saves,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
        }
        if profile != RecordSurfaceProfileView::EncounterParticipant {
            self.add(
                SurfaceDomain::Saves,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
        }
    }

    fn into_view(mut self) -> Option<CreatureSurfaceUnavailableDomainsView> {
        if self.causes.is_empty() {
            return None;
        }
        let mut take = |domain| {
            self.causes
                .remove(&domain)
                .map(|causes| CreatureSurfaceDomainUnavailableView { causes })
        };
        Some(CreatureSurfaceUnavailableDomainsView {
            classification: take(SurfaceDomain::Classification),
            initiative: take(SurfaceDomain::Initiative),
            vitals: take(SurfaceDomain::Vitals),
            defenses: take(SurfaceDomain::Defenses),
            saves: take(SurfaceDomain::Saves),
            awareness: take(SurfaceDomain::Awareness),
            abilities: take(SurfaceDomain::Abilities),
            skills: take(SurfaceDomain::Skills),
            movement: take(SurfaceDomain::Movement),
            resources: take(SurfaceDomain::Resources),
            equipment: take(SurfaceDomain::Equipment),
            lore: take(SurfaceDomain::Lore),
            spellcasting: take(SurfaceDomain::Spellcasting),
            activities: take(SurfaceDomain::Activities),
            relationships: None,
        })
    }
}

fn optional_fact<T>(value: &FactValue<T>) -> Option<&T> {
    value.as_value()
}

fn fact_provenance(field: CreatureSurfaceSourceFieldView) -> CreatureSurfaceFactProvenanceView {
    CreatureSurfaceFactProvenanceView {
        owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
        field,
    }
}

fn required_fact<'a, T>(
    value: &'a FactValue<T>,
    unavailable: &mut SurfaceUnavailableDomains,
    domain: SurfaceDomain,
    field: CreatureSurfaceUnavailableFieldView,
    source_field: CreatureSurfaceSourceFieldView,
    component_id: Option<String>,
) -> Option<&'a T> {
    match value {
        FactValue::Value(value) => Some(value),
        FactValue::Missing => {
            unavailable.add(
                domain,
                CreatureSurfaceUnavailableStateView::Missing,
                field,
                source_field,
                component_id,
            );
            None
        }
        FactValue::Null => {
            unavailable.add(
                domain,
                CreatureSurfaceUnavailableStateView::Null,
                field,
                source_field,
                component_id,
            );
            None
        }
    }
}

fn unsupported(
    unavailable: &mut SurfaceUnavailableDomains,
    domain: SurfaceDomain,
    field: CreatureSurfaceUnavailableFieldView,
    source_field: CreatureSurfaceSourceFieldView,
    component_id: Option<String>,
) {
    unavailable.add(
        domain,
        CreatureSurfaceUnavailableStateView::Unsupported,
        field,
        source_field,
        component_id,
    );
}

fn size(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceSizeView> {
    required_fact(
        &creature.size.value,
        unavailable,
        SurfaceDomain::Classification,
        CreatureSurfaceUnavailableFieldView::Size,
        CreatureSurfaceSourceFieldView::Size,
        None,
    )
    .map(|size| CreatureSurfaceSizeView {
        value: match size {
            CreatureSize::Tiny => CreatureSurfaceSizeValueView::Tiny,
            CreatureSize::Small => CreatureSurfaceSizeValueView::Small,
            CreatureSize::Medium => CreatureSurfaceSizeValueView::Medium,
            CreatureSize::Large => CreatureSurfaceSizeValueView::Large,
            CreatureSize::Huge => CreatureSurfaceSizeValueView::Huge,
            CreatureSize::Gargantuan => CreatureSurfaceSizeValueView::Gargantuan,
        },
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Size),
    })
}

fn adjustment(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceAdjustmentView> {
    match optional_fact(&creature.adjustment.value) {
        Some(CreatureAdjustment::Elite) => Some(CreatureSurfaceAdjustmentView {
            value: CreatureSurfaceAdjustmentValueView::Elite,
            provenance: fact_provenance(CreatureSurfaceSourceFieldView::Adjustment),
        }),
        Some(CreatureAdjustment::Weak) => Some(CreatureSurfaceAdjustmentView {
            value: CreatureSurfaceAdjustmentValueView::Weak,
            provenance: fact_provenance(CreatureSurfaceSourceFieldView::Adjustment),
        }),
        Some(CreatureAdjustment::Unsupported(_)) => {
            unsupported(
                unavailable,
                SurfaceDomain::Classification,
                CreatureSurfaceUnavailableFieldView::Adjustment,
                CreatureSurfaceSourceFieldView::Adjustment,
                None,
            );
            None
        }
        None => None,
    }
}

fn initiative(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceInitiativeView> {
    let initiative = optional_fact(&creature.initiative.value)?;
    match optional_fact(&initiative.statistic) {
        Some(atlas_record::CreatureInitiativeStatistic::Named(statistic)) => {
            Some(CreatureSurfaceInitiativeView {
                statistic: statistic.as_str().to_string(),
                provenance: fact_provenance(CreatureSurfaceSourceFieldView::Initiative),
            })
        }
        Some(atlas_record::CreatureInitiativeStatistic::Unsupported(_)) => {
            unsupported(
                unavailable,
                SurfaceDomain::Initiative,
                CreatureSurfaceUnavailableFieldView::InitiativeStatistic,
                CreatureSurfaceSourceFieldView::Initiative,
                None,
            );
            None
        }
        None => None,
    }
}

fn vitals(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceVitalsView> {
    let hp = required_fact(
        &defenses.hit_points,
        unavailable,
        SurfaceDomain::Vitals,
        CreatureSurfaceUnavailableFieldView::HitPoints,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )?;
    let hit_points = required_fact(
        &hp.maximum,
        unavailable,
        SurfaceDomain::Vitals,
        CreatureSurfaceUnavailableFieldView::HitPoints,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )
    .copied();
    Some(CreatureSurfaceVitalsView {
        hit_points,
        details: note(&hp.details),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    })
}

fn defenses_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> CreatureSurfaceDefensesView {
    let armor_class = required_fact(
        &defenses.armor_class,
        unavailable,
        SurfaceDomain::Defenses,
        CreatureSurfaceUnavailableFieldView::ArmorClass,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    );
    CreatureSurfaceDefensesView {
        armor_class: armor_class.and_then(|value| {
            required_fact(
                &value.value,
                unavailable,
                SurfaceDomain::Defenses,
                CreatureSurfaceUnavailableFieldView::ArmorClass,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            )
            .copied()
        }),
        armor_class_details: armor_class.and_then(|value| note(&value.details)),
        hardness: integer(&defenses.hardness),
        shield: shield(&defenses.shield, unavailable),
        immunities: iwr(&defenses.immunities, unavailable),
        resistances: iwr(&defenses.resistances, unavailable),
        weaknesses: iwr(&defenses.weaknesses, unavailable),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    }
}

fn compact_defenses_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> CreatureSurfaceDefensesView {
    let armor_class = required_fact(
        &defenses.armor_class,
        unavailable,
        SurfaceDomain::Defenses,
        CreatureSurfaceUnavailableFieldView::ArmorClass,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    );
    CreatureSurfaceDefensesView {
        armor_class: armor_class.and_then(|value| {
            required_fact(
                &value.value,
                unavailable,
                SurfaceDomain::Defenses,
                CreatureSurfaceUnavailableFieldView::ArmorClass,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            )
            .copied()
        }),
        armor_class_details: None,
        hardness: None,
        shield: None,
        immunities: Vec::new(),
        resistances: Vec::new(),
        weaknesses: Vec::new(),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    }
}

fn shield(
    value: &FactValue<atlas_record::CreatureShield>,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceShieldView> {
    let shield = optional_fact(value)?;
    let armor_class_bonus = required_fact(
        &shield.armor_class_bonus,
        unavailable,
        SurfaceDomain::Defenses,
        CreatureSurfaceUnavailableFieldView::ShieldArmorClassBonus,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )
    .copied();
    let broken_threshold = required_fact(
        &shield.broken_threshold,
        unavailable,
        SurfaceDomain::Defenses,
        CreatureSurfaceUnavailableFieldView::ShieldBrokenThreshold,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )
    .copied();
    let hardness = required_fact(
        &shield.hardness,
        unavailable,
        SurfaceDomain::Defenses,
        CreatureSurfaceUnavailableFieldView::ShieldHardness,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )
    .copied();
    let maximum_hit_points = required_fact(
        &shield.maximum_hit_points,
        unavailable,
        SurfaceDomain::Defenses,
        CreatureSurfaceUnavailableFieldView::ShieldMaximumHitPoints,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )
    .copied();
    Some(CreatureSurfaceShieldView {
        armor_class_bonus,
        broken_threshold,
        hardness,
        maximum_hit_points,
    })
}

fn encounter_defenses_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceDefensesView> {
    let view = CreatureSurfaceDefensesView {
        armor_class: None,
        armor_class_details: defenses
            .armor_class
            .as_value()
            .and_then(|armor_class| note(&armor_class.details)),
        hardness: integer(&defenses.hardness),
        shield: None,
        immunities: iwr(&defenses.immunities, unavailable),
        resistances: iwr(&defenses.resistances, unavailable),
        weaknesses: iwr(&defenses.weaknesses, unavailable),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    };
    (view.armor_class_details.is_some()
        || view.hardness.is_some()
        || !view.immunities.is_empty()
        || !view.resistances.is_empty()
        || !view.weaknesses.is_empty())
    .then_some(view)
}

fn iwr(
    values: &FactValue<Vec<CreatureIwr>>,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Vec<CreatureSurfaceIwrView> {
    let Some(values) = optional_fact(values) else {
        return Vec::new();
    };
    let mut projected = values
        .iter()
        .map(|value| {
            let component_id = value.id.as_str().to_string();
            let amount = match value.kind {
                atlas_record::CreatureIwrKind::Immunity => integer(&value.value),
                atlas_record::CreatureIwrKind::Resistance
                | atlas_record::CreatureIwrKind::Weakness => required_fact(
                    &value.value,
                    unavailable,
                    SurfaceDomain::Defenses,
                    CreatureSurfaceUnavailableFieldView::IwrAmount,
                    CreatureSurfaceSourceFieldView::Defenses,
                    Some(component_id.clone()),
                )
                .copied(),
            };
            let exceptions = optional_fact(&value.exceptions)
                .map(|values| {
                    values
                        .iter()
                        .map(|value| value.as_str().to_string())
                        .collect()
                })
                .unwrap_or_default();
            let double_vs = optional_fact(&value.double_vs)
                .map(|values| {
                    values
                        .iter()
                        .map(|value| value.as_str().to_string())
                        .collect()
                })
                .unwrap_or_default();
            CreatureSurfaceIwrView {
                component_id,
                authored_order: value.authored_order,
                kind: value.iwr_type.as_str().to_string(),
                amount,
                exceptions,
                double_vs,
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    projected
}

fn saves_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceSavesView> {
    let saves = required_fact(
        &defenses.saves,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )?;
    Some(CreatureSurfaceSavesView {
        fortitude: save_fact(&saves.fortitude, unavailable, "fortitude"),
        reflex: save_fact(&saves.reflex, unavailable, "reflex"),
        will: save_fact(&saves.will, unavailable, "will"),
        all_saves_note: note(&defenses.all_saves_note),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    })
}

fn compact_saves_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceSavesView> {
    let saves = required_fact(
        &defenses.saves,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )?;
    Some(CreatureSurfaceSavesView {
        fortitude: compact_save_fact(&saves.fortitude, unavailable, "fortitude"),
        reflex: compact_save_fact(&saves.reflex, unavailable, "reflex"),
        will: compact_save_fact(&saves.will, unavailable, "will"),
        all_saves_note: None,
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    })
}

fn compact_save_fact(
    value: &FactValue<atlas_record::CreatureSave>,
    unavailable: &mut SurfaceUnavailableDomains,
    destination: &str,
) -> Option<CreatureSurfaceSaveView> {
    let value = required_fact(
        value,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        Some(destination.to_string()),
    )?;
    let component_id = value.id.as_str().to_string();
    let modifier = required_fact(
        &value.value,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        Some(component_id.clone()),
    )
    .copied();
    Some(CreatureSurfaceSaveView {
        component_id,
        modifier,
        details: None,
    })
}

fn encounter_saves_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceSavesView> {
    let saves = required_fact(
        &defenses.saves,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    );
    let fortitude =
        saves.and_then(|saves| save_context_fact(&saves.fortitude, unavailable, "fortitude"));
    let reflex = saves.and_then(|saves| save_context_fact(&saves.reflex, unavailable, "reflex"));
    let will = saves.and_then(|saves| save_context_fact(&saves.will, unavailable, "will"));
    let all_saves_note = note(&defenses.all_saves_note);
    (fortitude.is_some() || reflex.is_some() || will.is_some() || all_saves_note.is_some())
        .then_some(CreatureSurfaceSavesView {
            fortitude,
            reflex,
            will,
            all_saves_note,
            provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
        })
}

fn save_context_fact(
    value: &FactValue<atlas_record::CreatureSave>,
    unavailable: &mut SurfaceUnavailableDomains,
    destination: &str,
) -> Option<CreatureSurfaceSaveView> {
    let value = required_fact(
        value,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        Some(destination.to_string()),
    )?;
    let details = note(&value.details)?;
    Some(CreatureSurfaceSaveView {
        component_id: value.id.as_str().to_string(),
        modifier: None,
        details: Some(details),
    })
}

fn save_fact(
    value: &FactValue<atlas_record::CreatureSave>,
    unavailable: &mut SurfaceUnavailableDomains,
    destination: &str,
) -> Option<CreatureSurfaceSaveView> {
    let value = required_fact(
        value,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        Some(destination.to_string()),
    )?;
    let component_id = value.id.as_str().to_string();
    let modifier = required_fact(
        &value.value,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        Some(component_id.clone()),
    )
    .copied();
    Some(CreatureSurfaceSaveView {
        component_id,
        modifier,
        details: note(&value.details),
    })
}

fn awareness(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
    include_perception_modifier: bool,
) -> Option<CreatureSurfaceAwarenessView> {
    let languages = optional_fact(&creature.languages.value);
    let language_values = languages.and_then(|value| optional_fact(&value.values));
    let perception = required_fact(
        &creature.perception.value,
        unavailable,
        SurfaceDomain::Awareness,
        CreatureSurfaceUnavailableFieldView::Perception,
        CreatureSurfaceSourceFieldView::Perception,
        None,
    );
    let mut senses = perception
        .and_then(|value| optional_fact(&value.senses))
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    let component_id = value.id.as_str().to_string();
                    let acuity = match sense_acuity(value, unavailable, &component_id) {
                        Some(SenseAcuity::Precise) => Some("precise".to_string()),
                        Some(SenseAcuity::Imprecise) => Some("imprecise".to_string()),
                        Some(SenseAcuity::Vague) => Some("vague".to_string()),
                        Some(SenseAcuity::Unsupported(_)) => {
                            unsupported(
                                unavailable,
                                SurfaceDomain::Awareness,
                                CreatureSurfaceUnavailableFieldView::SenseAcuity,
                                CreatureSurfaceSourceFieldView::Perception,
                                Some(component_id.clone()),
                            );
                            None
                        }
                        None => None,
                    };
                    CreatureSurfaceSenseView {
                        component_id,
                        authored_order: value.authored_order,
                        kind: value.sense_type.as_str().to_string(),
                        acuity,
                        range_feet: integer(&value.range),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    senses.sort_by_key(|value| value.authored_order);
    let view = CreatureSurfaceAwarenessView {
        perception: include_perception_modifier
            .then(|| {
                perception.and_then(|value| {
                    required_fact(
                        &value.modifier,
                        unavailable,
                        SurfaceDomain::Awareness,
                        CreatureSurfaceUnavailableFieldView::Perception,
                        CreatureSurfaceSourceFieldView::Perception,
                        None,
                    )
                    .copied()
                })
            })
            .flatten(),
        details: perception.and_then(|value| note(&value.details)),
        has_vision: perception.and_then(|value| boolean(&value.has_vision)),
        senses,
        languages: language_values
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            })
            .unwrap_or_default(),
        language_details: languages.and_then(|value| note(&value.details)),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Perception),
    };
    if include_perception_modifier
        || view.details.is_some()
        || view.has_vision.is_some()
        || !view.senses.is_empty()
        || !view.languages.is_empty()
        || view.language_details.is_some()
    {
        Some(view)
    } else {
        None
    }
}

fn sense_acuity<'a>(
    value: &'a atlas_record::CreatureSense,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<&'a SenseAcuity> {
    if matches!(value.acuity, FactValue::Missing)
        && matches!(
            value.sense_type.as_str(),
            "low-light-vision" | "darkvision" | "greater-darkvision" | "see-invisibility"
        )
    {
        return None;
    }
    required_fact(
        &value.acuity,
        unavailable,
        SurfaceDomain::Awareness,
        CreatureSurfaceUnavailableFieldView::SenseAcuity,
        CreatureSurfaceSourceFieldView::Perception,
        Some(component_id.to_string()),
    )
}

fn compact_awareness(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceAwarenessView> {
    let perception = required_fact(
        &creature.perception.value,
        unavailable,
        SurfaceDomain::Awareness,
        CreatureSurfaceUnavailableFieldView::Perception,
        CreatureSurfaceSourceFieldView::Perception,
        None,
    )?;
    let modifier = required_fact(
        &perception.modifier,
        unavailable,
        SurfaceDomain::Awareness,
        CreatureSurfaceUnavailableFieldView::Perception,
        CreatureSurfaceSourceFieldView::Perception,
        None,
    )
    .copied();
    Some(CreatureSurfaceAwarenessView {
        perception: modifier,
        details: None,
        has_vision: None,
        senses: Vec::new(),
        languages: Vec::new(),
        language_details: None,
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Perception),
    })
}

fn abilities(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceAbilitiesView> {
    let abilities = required_fact(
        &creature.legacy_abilities.value,
        unavailable,
        SurfaceDomain::Abilities,
        CreatureSurfaceUnavailableFieldView::LegacyAbilities,
        CreatureSurfaceSourceFieldView::LegacyAbilities,
        None,
    )?;
    Some(CreatureSurfaceAbilitiesView {
        strength: ability_value(&abilities.strength, unavailable, "strength"),
        dexterity: ability_value(&abilities.dexterity, unavailable, "dexterity"),
        constitution: ability_value(&abilities.constitution, unavailable, "constitution"),
        intelligence: ability_value(&abilities.intelligence, unavailable, "intelligence"),
        wisdom: ability_value(&abilities.wisdom, unavailable, "wisdom"),
        charisma: ability_value(&abilities.charisma, unavailable, "charisma"),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::LegacyAbilities),
    })
}

fn ability_value(
    value: &FactValue<i64>,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<i64> {
    required_fact(
        value,
        unavailable,
        SurfaceDomain::Abilities,
        CreatureSurfaceUnavailableFieldView::LegacyAbilities,
        CreatureSurfaceSourceFieldView::LegacyAbilities,
        Some(component_id.to_string()),
    )
    .copied()
}

fn skills(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> (
    Option<Vec<CreatureSurfaceSkillView>>,
    Option<Vec<CreatureSurfaceUnmodeledSkillView>>,
) {
    let Some(values) = required_fact(
        &creature.skills.value,
        unavailable,
        SurfaceDomain::Skills,
        CreatureSurfaceUnavailableFieldView::Skills,
        CreatureSurfaceSourceFieldView::Skills,
        None,
    ) else {
        return (None, None);
    };
    let mut projected = Vec::new();
    let mut projected_unmodeled = Vec::new();
    for value in values {
        let component_id = value.id.as_str().to_string();
        if value.kind == atlas_record::CreatureSkillKind::Unmodeled {
            match &value.unmodeled {
                FactValue::Value(unmodeled) => {
                    let detail = CreatureSurfaceUnmodeledSkillView {
                        component_id: component_id.clone(),
                        authored_order: value.authored_order,
                        source_entries: skill_source_entries(&value.source_entries),
                        source_item_id: optional_fact(&value.source_item_id)
                            .map(|value| value.as_str().to_string()),
                        authored_key: unmodeled.authored_key.clone(),
                        base: integer_presence(&unmodeled.base),
                        reason: match unmodeled.reason {
                            CreatureUnmodeledSkillReason::UnknownAuthoredKey => {
                                CreatureSurfaceUnmodeledSkillReasonView::UnknownAuthoredKey
                            }
                        },
                    };
                    unavailable.add_unmodeled_skill(component_id, detail.clone());
                    projected_unmodeled.push(detail);
                }
                FactValue::Missing => unavailable.add(
                    SurfaceDomain::Skills,
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::UnmodeledSkill,
                    CreatureSurfaceSourceFieldView::Skills,
                    Some(component_id),
                ),
                FactValue::Null => unavailable.add(
                    SurfaceDomain::Skills,
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::UnmodeledSkill,
                    CreatureSurfaceSourceFieldView::Skills,
                    Some(component_id),
                ),
            }
            continue;
        }
        let variants = optional_fact(&value.variants).and_then(|variants| {
            let mut variants = variants
                .iter()
                .map(|variant| skill_variant(variant, unavailable, &component_id))
                .collect::<Vec<_>>();
            variants.sort_by_key(|variant| variant.authored_order);
            non_empty(variants)
        });
        projected.push(CreatureSurfaceSkillView {
            component_id: component_id.clone(),
            authored_order: value.authored_order,
            kind: value.kind.source_slug().to_string(),
            label: value.label.clone(),
            source_entries: skill_source_entries(&value.source_entries),
            modifier: required_fact(
                &value.modifier,
                unavailable,
                SurfaceDomain::Skills,
                CreatureSurfaceUnavailableFieldView::SkillModifier,
                CreatureSurfaceSourceFieldView::Skills,
                Some(component_id),
            )
            .copied(),
            note: note(&value.note),
            variants,
        });
    }
    projected.sort_by_key(|value| value.authored_order);
    projected_unmodeled.sort_by_key(|value| value.authored_order);
    (non_empty(projected), non_empty(projected_unmodeled))
}

fn skill_source_entries(
    values: &[atlas_record::CreatureSkillSourceEntry],
) -> Option<Vec<CreatureSurfaceSkillSourceEntryView>> {
    non_empty(
        values
            .iter()
            .enumerate()
            .map(|(index, entry)| CreatureSurfaceSkillSourceEntryView {
                authored_order: index as u32,
                authored_key: entry.authored_key.clone(),
                modifier: integer_presence(&entry.modifier),
            })
            .collect(),
    )
}

fn integer_presence(value: &FactValue<i64>) -> CreatureSurfaceIntegerPresenceView {
    match value {
        FactValue::Missing => CreatureSurfaceIntegerPresenceView::Missing,
        FactValue::Null => CreatureSurfaceIntegerPresenceView::Null,
        FactValue::Value(value) => CreatureSurfaceIntegerPresenceView::Value { value: *value },
    }
}

fn skill_variant(
    value: &atlas_record::CreatureSkillVariant,
    unavailable: &mut SurfaceUnavailableDomains,
    skill_id: &str,
) -> CreatureSurfaceSkillVariantView {
    let component_id = format!("{skill_id}/{}", value.id.as_str());
    let predicates = optional_fact(&value.predicate).and_then(|predicates| {
        let projected = predicates
            .iter()
            .filter_map(|predicate| match predicate {
                CreaturePredicate::Term(term) => Some(CreatureSurfaceSkillPredicateView::Term {
                    term: term.as_str().to_string(),
                }),
                CreaturePredicate::Not(term) => Some(CreatureSurfaceSkillPredicateView::Not {
                    term: term.as_str().to_string(),
                }),
                CreaturePredicate::Any(terms) => Some(CreatureSurfaceSkillPredicateView::Any {
                    terms: terms.iter().map(|term| term.as_str().to_string()).collect(),
                }),
                CreaturePredicate::AtLeast { term, minimum } => {
                    Some(CreatureSurfaceSkillPredicateView::AtLeast {
                        term: term.as_str().to_string(),
                        minimum: *minimum,
                    })
                }
                CreaturePredicate::Unsupported(_) => {
                    unsupported(
                        unavailable,
                        SurfaceDomain::Skills,
                        CreatureSurfaceUnavailableFieldView::SkillVariantPredicate,
                        CreatureSurfaceSourceFieldView::Skills,
                        Some(component_id.clone()),
                    );
                    None
                }
            })
            .collect::<Vec<_>>();
        non_empty(projected)
    });
    CreatureSurfaceSkillVariantView {
        component_id: value.id.as_str().to_string(),
        authored_order: value.authored_order,
        modifier: required_fact(
            &value.modifier,
            unavailable,
            SurfaceDomain::Skills,
            CreatureSurfaceUnavailableFieldView::SkillVariantModifier,
            CreatureSurfaceSourceFieldView::Skills,
            Some(component_id),
        )
        .copied(),
        label: text(&value.label),
        predicates,
    }
}

fn movement(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceMovementView>> {
    let values = required_fact(
        &creature.movement.value,
        unavailable,
        SurfaceDomain::Movement,
        CreatureSurfaceUnavailableFieldView::Movement,
        CreatureSurfaceSourceFieldView::Movement,
        None,
    )?;
    let mut projected = values
        .iter()
        .filter_map(|value| {
            let component_id = value.id.as_str().to_string();
            let speed_feet = required_fact(
                &value.value,
                unavailable,
                SurfaceDomain::Movement,
                CreatureSurfaceUnavailableFieldView::MovementSpeed,
                CreatureSurfaceSourceFieldView::Movement,
                Some(component_id.clone()),
            )
            .copied();
            let mode = match &value.mode {
                CreatureMovementMode::Land => "land",
                CreatureMovementMode::Burrow => "burrow",
                CreatureMovementMode::Climb => "climb",
                CreatureMovementMode::Fly => "fly",
                CreatureMovementMode::Swim => "swim",
                CreatureMovementMode::Unsupported(_) => {
                    unsupported(
                        unavailable,
                        SurfaceDomain::Movement,
                        CreatureSurfaceUnavailableFieldView::MovementMode,
                        CreatureSurfaceSourceFieldView::Movement,
                        Some(component_id.clone()),
                    );
                    return None;
                }
            };
            Some(CreatureSurfaceMovementView {
                component_id: component_id.clone(),
                authored_order: value.authored_order,
                mode: mode.to_string(),
                label: text(&value.label),
                speed_feet,
                details: note(&value.details),
            })
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn resources(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceResourceView>> {
    let values = required_fact(
        &creature.resources.value,
        unavailable,
        SurfaceDomain::Resources,
        CreatureSurfaceUnavailableFieldView::Resources,
        CreatureSurfaceSourceFieldView::Resources,
        None,
    )?;
    let mut projected = values
        .iter()
        .map(|value| {
            let component_id = value.id.as_str().to_string();
            let maximum = match required_fact(
                &value.maximum,
                unavailable,
                SurfaceDomain::Resources,
                CreatureSurfaceUnavailableFieldView::ResourceMaximum,
                CreatureSurfaceSourceFieldView::Resources,
                Some(component_id.clone()),
            ) {
                Some(CreatureResourceAmount::Integer(value)) => Some(*value),
                Some(CreatureResourceAmount::Unsupported(_)) => {
                    unsupported(
                        unavailable,
                        SurfaceDomain::Resources,
                        CreatureSurfaceUnavailableFieldView::ResourceMaximum,
                        CreatureSurfaceSourceFieldView::Resources,
                        Some(component_id.clone()),
                    );
                    None
                }
                None => None,
            };
            CreatureSurfaceResourceView {
                component_id,
                authored_order: value.authored_order,
                kind: value.kind.as_str().to_string(),
                label: value.label.clone(),
                maximum,
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn rituals(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceRitualsView> {
    let embedded = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Spellcasting,
        CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )?;
    let context = optional_fact(&embedded.actor_spellcasting)?;
    let difficulty_class = match required_fact(
        &context.rituals_dc,
        unavailable,
        SurfaceDomain::Spellcasting,
        CreatureSurfaceUnavailableFieldView::RitualDifficultyClass,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )? {
        CreatureSourceScalar::Value(value) => *value,
        CreatureSourceScalar::Unsupported(_) => {
            unsupported(
                unavailable,
                SurfaceDomain::Spellcasting,
                CreatureSurfaceUnavailableFieldView::RitualDifficultyClass,
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
                None,
            );
            return None;
        }
    };
    Some(CreatureSurfaceRitualsView {
        difficulty_class,
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::EmbeddedEntities),
    })
}

fn equipment(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceEquipmentView>> {
    let embedded = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Equipment,
        CreatureSurfaceUnavailableFieldView::Equipment,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )?;
    let mut projected = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| {
            let CreatureCapability::Equipment(capability) = &occurrence.capability else {
                return None;
            };
            let occurrence_id = occurrence.id.as_str().to_string();
            Some(CreatureSurfaceEquipmentView {
                occurrence_id: occurrence_id.clone(),
                authored_order: occurrence.authored_order,
                provenance: occurrence_provenance(occurrence),
                label: occurrence_label(occurrence, embedded),
                traits: optional_fact(&capability.traits)
                    .cloned()
                    .and_then(non_empty),
                level: optional_fact(&capability.level).copied(),
                usage: optional_fact(&capability.usage).cloned(),
                quantity: optional_fact(&capability.quantity).copied(),
                uses: optional_fact(&capability.uses).and_then(|uses| {
                    uses_view(
                        uses,
                        unavailable,
                        SurfaceDomain::Equipment,
                        CreatureSurfaceUnavailableFieldView::EquipmentUsesMaximum,
                        &occurrence_id,
                    )
                }),
            })
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn lore(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceLoreView>> {
    let embedded = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Lore,
        CreatureSurfaceUnavailableFieldView::Lore,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )?;
    let mut projected = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| {
            let CreatureCapability::Lore(capability) = &occurrence.capability else {
                return None;
            };
            let occurrence_id = occurrence.id.as_str().to_string();
            Some(CreatureSurfaceLoreView {
                occurrence_id: occurrence_id.clone(),
                authored_order: occurrence.authored_order,
                provenance: occurrence_provenance(occurrence),
                label: occurrence_label(occurrence, embedded),
                modifier: required_fact(
                    &capability.modifier,
                    unavailable,
                    SurfaceDomain::Lore,
                    CreatureSurfaceUnavailableFieldView::LoreModifier,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(occurrence_id),
                )
                .copied(),
            })
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn occurrence_provenance(
    occurrence: &CreatureEntityOccurrence,
) -> CreatureSurfaceOccurrenceProvenanceView {
    CreatureSurfaceOccurrenceProvenanceView {
        identity_stability: match occurrence.identity_stability {
            atlas_record::OccurrenceIdentityStability::StableNestedSourceId => {
                CreatureSurfaceOccurrenceIdentityStabilityView::StableNestedSourceId
            }
            atlas_record::OccurrenceIdentityStability::UnstableOwnerFamilyOrdinal => {
                CreatureSurfaceOccurrenceIdentityStabilityView::UnstableOwnerFamilyOrdinal
            }
        },
        nested_source_id: optional_fact(&occurrence.source_identity.nested_source_id)
            .map(|value| value.as_str().to_string()),
        stable_source_locator: optional_fact(&occurrence.source_identity.stable_source_locator)
            .map(|value| value.as_str().to_string()),
        source_locators: non_empty(
            occurrence
                .source_identity
                .source_locators
                .iter()
                .map(|value| CreatureSurfaceSourceLocatorView {
                    locator: value.locator.as_str().to_string(),
                    precedence: value.precedence,
                })
                .collect(),
        ),
    }
}

fn uses_view(
    value: &CreatureUseLimit,
    unavailable: &mut SurfaceUnavailableDomains,
    domain: SurfaceDomain,
    field: CreatureSurfaceUnavailableFieldView,
    component_id: &str,
) -> Option<CreatureSurfaceUsesView> {
    let maximum = required_fact(
        &value.maximum,
        unavailable,
        domain,
        field,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(component_id.to_string()),
    )
    .copied()?;
    Some(CreatureSurfaceUsesView {
        maximum: Some(maximum),
    })
}

fn activities(
    creature: &CreatureRecord,
    activity_content: &CreatureContentPlacement,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceActivityView>> {
    let embedded = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )?;
    let mut projected = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| {
            let component_id = occurrence.id.as_str();
            if activity_content.failure(&occurrence.id).is_some() {
                unsupported(
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id.to_string()),
                );
                return None;
            }
            let content = non_empty(
                activity_content
                    .documents_for_occurrence(creature, &occurrence.id)
                    .filter_map(content_view)
                    .collect(),
            );
            match &occurrence.capability {
                CreatureCapability::Strike(capability) => Some(activity(
                    occurrence,
                    embedded,
                    ActivityProjection {
                        activity_type: CreatureSurfaceActivityTypeView::Strike,
                        traits: &capability.traits,
                        action_cost: &capability.action_cost,
                        attack_effects: Some(&capability.attack_effects),
                        category: None,
                        frequency: None,
                        requirements: None,
                        cost: None,
                        self_effect: None,
                        self_effect_label: None,
                        rolls: &capability.rolls,
                        damage: ActivityDamageProjection::RequiredStrike(&capability.damage),
                    },
                    content,
                    unavailable,
                )),
                CreatureCapability::Action(capability) => Some(activity(
                    occurrence,
                    embedded,
                    ActivityProjection {
                        activity_type: CreatureSurfaceActivityTypeView::Action,
                        traits: &capability.traits,
                        action_cost: &capability.action_cost,
                        attack_effects: None,
                        category: Some(&capability.category),
                        frequency: Some(&capability.frequency),
                        requirements: Some(&capability.requirements),
                        cost: Some(&capability.cost),
                        self_effect: Some(&capability.self_effect),
                        self_effect_label: Some(&capability.self_effect_label),
                        rolls: &capability.rolls,
                        damage: ActivityDamageProjection::OptionalAction(&capability.damage),
                    },
                    content,
                    unavailable,
                )),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn register_encounter_activity_trait_unavailability(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) {
    let Some(embedded) = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    ) else {
        return;
    };
    for occurrence in &embedded.occurrences {
        let Some(traits) = activity_traits(occurrence) else {
            continue;
        };
        required_fact(
            traits,
            unavailable,
            SurfaceDomain::Activities,
            CreatureSurfaceUnavailableFieldView::ActivityTraits,
            CreatureSurfaceSourceFieldView::EmbeddedEntities,
            Some(occurrence.id.as_str().to_string()),
        );
    }
}

fn activity_traits(occurrence: &CreatureEntityOccurrence) -> Option<&FactValue<Vec<String>>> {
    match &occurrence.capability {
        CreatureCapability::Strike(capability) => Some(&capability.traits),
        CreatureCapability::Action(capability) => Some(&capability.traits),
        _ => None,
    }
}

struct ActivityProjection<'a> {
    activity_type: CreatureSurfaceActivityTypeView,
    traits: &'a FactValue<Vec<String>>,
    action_cost: &'a CreatureActionCost,
    attack_effects: Option<&'a FactValue<Vec<String>>>,
    category: Option<&'a FactValue<String>>,
    frequency: Option<&'a FactValue<atlas_record::CreatureFrequency>>,
    requirements: Option<&'a FactValue<String>>,
    cost: Option<&'a FactValue<String>>,
    self_effect: Option<&'a FactValue<String>>,
    self_effect_label: Option<&'a FactValue<String>>,
    rolls: &'a [CreatureRoll],
    damage: ActivityDamageProjection<'a>,
}

enum ActivityDamageProjection<'a> {
    RequiredStrike(&'a FactValue<Vec<CreatureDamage>>),
    OptionalAction(&'a FactValue<Vec<CreatureDamage>>),
}

fn activity(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
    projection: ActivityProjection<'_>,
    content: Option<Vec<CreatureSurfaceContentView>>,
    unavailable: &mut SurfaceUnavailableDomains,
) -> CreatureSurfaceActivityView {
    let component_id = occurrence.id.as_str().to_string();
    let traits = required_fact(
        projection.traits,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::ActivityTraits,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(component_id.clone()),
    )
    .cloned()
    .unwrap_or_default();
    CreatureSurfaceActivityView {
        occurrence_id: component_id.clone(),
        authored_order: occurrence.authored_order,
        provenance: occurrence_provenance(occurrence),
        activity_type: projection.activity_type,
        label: occurrence_label(occurrence, embedded),
        traits,
        action_cost: action_cost(projection.action_cost, unavailable, &component_id),
        attack_effects: projection
            .attack_effects
            .and_then(optional_fact)
            .cloned()
            .and_then(non_empty),
        category: projection.category.and_then(optional_fact).cloned(),
        frequency: projection
            .frequency
            .and_then(optional_fact)
            .and_then(|frequency| frequency_view(frequency, unavailable, &component_id)),
        requirements: projection.requirements.and_then(optional_fact).cloned(),
        cost: projection.cost.and_then(optional_fact).cloned(),
        uses: optional_fact(&occurrence.context.uses).and_then(|uses| {
            uses_view(
                uses,
                unavailable,
                SurfaceDomain::Activities,
                CreatureSurfaceUnavailableFieldView::ActionUsesMaximum,
                &component_id,
            )
        }),
        self_effect: {
            let value = projection.self_effect.and_then(optional_fact).cloned();
            let label = projection
                .self_effect_label
                .and_then(optional_fact)
                .cloned();
            (value.is_some() || label.is_some())
                .then_some(CreatureSurfaceSelfEffectView { value, label })
        },
        rolls: rolls(projection.rolls, unavailable, &component_id),
        damage: damage(projection.damage, unavailable, &component_id),
        content,
    }
}

fn frequency_view(
    value: &atlas_record::CreatureFrequency,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<CreatureSurfaceFrequencyView> {
    let maximum = required_fact(
        &value.maximum,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::ActionFrequencyMaximum,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(component_id.to_string()),
    )
    .copied();
    let period = required_fact(
        &value.period,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::ActionFrequencyPeriod,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(component_id.to_string()),
    )
    .cloned();
    let display_period = period.as_deref().and_then(|period| {
        match atlas_record::CreatureFrequencyPeriod::from_source_token(period) {
            Some(period) => Some(period),
            None => {
                unsupported(
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::ActionFrequencyPeriod,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id.to_string()),
                );
                None
            }
        }
    });
    let display = format_creature_frequency(maximum, display_period);
    Some(CreatureSurfaceFrequencyView {
        maximum,
        period,
        display,
    })
}

fn spellcasting(
    creature: &CreatureRecord,
    content_placement: &CreatureContentPlacement,
    unavailable: &mut SurfaceUnavailableDomains,
) -> (
    Option<Vec<CreatureSurfaceSpellcastingView>>,
    Option<Vec<CreatureSurfaceSpellView>>,
) {
    let Some(embedded) = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Spellcasting,
        CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    ) else {
        return (None, None);
    };
    let mut projected = embedded
        .occurrences
        .iter()
        .filter_map(|entry| {
            let CreatureCapability::SpellcastingEntry(capability) = &entry.capability else {
                return None;
            };
            let mut spells = embedded
                .occurrences
                .iter()
                .filter_map(|spell| {
                    let CreatureCapability::Spell(capability) = &spell.capability else {
                        return None;
                    };
                    let CreatureOccurrenceParent::SpellcastingEntry(parent) = &spell.parent else {
                        return None;
                    };
                    if parent != &entry.id {
                        return None;
                    }
                    surface_spell_view(
                        spell,
                        capability,
                        creature,
                        embedded,
                        content_placement,
                        unavailable,
                    )
                })
                .collect::<Vec<_>>();
            spells.sort_by_key(|value| value.authored_order);
            let component_id = entry.id.as_str().to_string();
            let preparation = required_fact(
                &capability.preparation,
                unavailable,
                SurfaceDomain::Spellcasting,
                CreatureSurfaceUnavailableFieldView::SpellPreparation,
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
                Some(component_id.clone()),
            )
            .and_then(|value| preparation(value, unavailable, &component_id));
            Some(CreatureSurfaceSpellcastingView {
                occurrence_id: component_id,
                authored_order: entry.authored_order,
                provenance: occurrence_provenance(entry),
                label: occurrence_label(entry, embedded),
                preparation,
                tradition: required_fact(
                    &capability.tradition,
                    unavailable,
                    SurfaceDomain::Spellcasting,
                    CreatureSurfaceUnavailableFieldView::SpellTradition,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(entry.id.as_str().to_string()),
                )
                .cloned(),
                attack_modifier: required_fact(
                    &capability.attack,
                    unavailable,
                    SurfaceDomain::Spellcasting,
                    CreatureSurfaceUnavailableFieldView::SpellAttack,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(entry.id.as_str().to_string()),
                )
                .copied(),
                difficulty_class: required_fact(
                    &capability.dc,
                    unavailable,
                    SurfaceDomain::Spellcasting,
                    CreatureSurfaceUnavailableFieldView::SpellDifficultyClass,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(entry.id.as_str().to_string()),
                )
                .copied(),
                slots: spell_slots(&capability.slots, unavailable, entry.id.as_str()),
                spells,
            })
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    let mut standalone_spells = embedded
        .occurrences
        .iter()
        .filter_map(|spell| {
            let CreatureCapability::Spell(capability) = &spell.capability else {
                return None;
            };
            matches!(spell.parent, CreatureOccurrenceParent::Creature).then_some(())?;
            surface_spell_view(
                spell,
                capability,
                creature,
                embedded,
                content_placement,
                unavailable,
            )
        })
        .collect::<Vec<_>>();
    standalone_spells.sort_by_key(|value| value.authored_order);
    (non_empty(projected), non_empty(standalone_spells))
}

fn surface_spell_view(
    spell: &CreatureEntityOccurrence,
    capability: &atlas_record::CreatureSpellCapability,
    creature: &CreatureRecord,
    embedded: &CreatureEmbeddedEntities,
    content_placement: &CreatureContentPlacement,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceSpellView> {
    let component_id = spell.id.as_str().to_string();
    if content_placement.failure(&spell.id).is_some() {
        unsupported(
            unavailable,
            SurfaceDomain::Spellcasting,
            CreatureSurfaceUnavailableFieldView::SpellContent,
            CreatureSurfaceSourceFieldView::EmbeddedEntities,
            Some(component_id),
        );
        return None;
    }
    let rank = required_fact(
        &spell.context.rank,
        unavailable,
        SurfaceDomain::Spellcasting,
        CreatureSurfaceUnavailableFieldView::SpellRank,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(component_id.clone()),
    )
    .copied();
    let traits = required_fact(
        &capability.traits,
        unavailable,
        SurfaceDomain::Spellcasting,
        CreatureSurfaceUnavailableFieldView::SpellTraits,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(component_id.clone()),
    )
    .cloned()
    .unwrap_or_default();
    Some(CreatureSurfaceSpellView {
        occurrence_id: component_id.clone(),
        authored_order: spell.authored_order,
        provenance: occurrence_provenance(spell),
        label: occurrence_label(spell, embedded),
        target_record_key: match &spell.target {
            CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
            CreatureEntityTarget::ActorOwned(_) => None,
        },
        rank,
        context: spell_context(spell, unavailable),
        traits,
        content: content_for_occurrence(creature, content_placement, &spell.id),
    })
}

fn spell_slots(
    values: &FactValue<Vec<atlas_record::CreatureSpellSlot>>,
    unavailable: &mut SurfaceUnavailableDomains,
    entry_id: &str,
) -> Option<Vec<CreatureSurfaceSpellSlotView>> {
    let slots = optional_fact(values)?;
    non_empty(
        slots
            .iter()
            .map(|slot| {
                let component_id = format!("{entry_id}/rank-{}", slot.rank);
                let maximum = match required_fact(
                    &slot.maximum,
                    unavailable,
                    SurfaceDomain::Spellcasting,
                    CreatureSurfaceUnavailableFieldView::SpellSlotMaximum,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id.clone()),
                ) {
                    Some(CreatureSourceScalar::Value(value)) => Some(*value),
                    Some(CreatureSourceScalar::Unsupported(_)) => {
                        unsupported(
                            unavailable,
                            SurfaceDomain::Spellcasting,
                            CreatureSurfaceUnavailableFieldView::SpellSlotMaximum,
                            CreatureSurfaceSourceFieldView::EmbeddedEntities,
                            Some(component_id),
                        );
                        None
                    }
                    None => None,
                };
                CreatureSurfaceSpellSlotView {
                    rank: slot.rank,
                    maximum,
                }
            })
            .collect(),
    )
}

fn spell_context(
    spell: &CreatureEntityOccurrence,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceSpellOccurrenceContextView> {
    let component_id = spell.id.as_str();
    let context = CreatureSurfaceSpellOccurrenceContextView {
        group: text(&spell.context.group),
        location: text(&spell.context.location),
        slot: text(&spell.context.slot),
        uses: optional_fact(&spell.context.uses).and_then(|uses| {
            uses_view(
                uses,
                unavailable,
                SurfaceDomain::Spellcasting,
                CreatureSurfaceUnavailableFieldView::SpellUsesMaximum,
                component_id,
            )
        }),
        contextual_label: text(&spell.context.contextual_label),
    };
    (context.group.is_some()
        || context.location.is_some()
        || context.slot.is_some()
        || context.uses.is_some()
        || context.contextual_label.is_some())
    .then_some(context)
}

fn activity_content_placement(creature: &CreatureRecord) -> CreatureContentPlacement {
    place_creature_content_for_families(
        creature,
        &[
            atlas_record::CreatureEntityFamily::Action,
            atlas_record::CreatureEntityFamily::Strike,
            atlas_record::CreatureEntityFamily::Spell,
        ],
    )
}

fn compose_encounter_payload(
    creature: &CreatureRecord,
    placement: &CreatureContentPlacement,
    runtime: &mut EncounterRuntimeView,
) {
    let Some(embedded) = creature.embedded_entities.value.as_value() else {
        for activity in std::mem::take(&mut runtime.activities) {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterPayloadContentUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Activity {
                    activity_id: activity.activity_id,
                },
                "Canonical occurrence ownership is unavailable for this activity.",
            );
        }
        runtime.spellcasting.clear();
        runtime.standalone_spells.clear();
        return;
    };

    let occurrence_counts = embedded.occurrences.iter().fold(
        BTreeMap::<&str, usize>::new(),
        |mut counts, occurrence| {
            *counts.entry(occurrence.id.as_str()).or_default() += 1;
            counts
        },
    );
    let entity_counts =
        embedded
            .entities
            .iter()
            .fold(BTreeMap::<&str, usize>::new(), |mut counts, entity| {
                *counts.entry(entity.id.as_str()).or_default() += 1;
                counts
            });

    let mut runtime_activities = BTreeMap::<String, Vec<EncounterRuntimeActivityView>>::new();
    for activity in std::mem::take(&mut runtime.activities) {
        runtime_activities
            .entry(activity.activity_id.clone())
            .or_default()
            .push(activity);
    }

    let mut retained_activities = Vec::new();
    let mut activity_occurrences = embedded
        .occurrences
        .iter()
        .filter(|occurrence| {
            matches!(
                occurrence.capability,
                CreatureCapability::Action(_) | CreatureCapability::Strike(_)
            )
        })
        .collect::<Vec<_>>();
    activity_occurrences.sort_by_key(|occurrence| occurrence.authored_order);
    for occurrence in activity_occurrences {
        let occurrence_id = occurrence.id.as_str();
        let mut matches = runtime_activities.remove(occurrence_id).unwrap_or_default();
        let safe = occurrence_counts.get(occurrence_id) == Some(&1)
            && actor_owned_target_is_unique(occurrence, &entity_counts)
            && placement.failure(&occurrence.id).is_none()
            && matches.len() == 1;
        if !safe {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterPayloadContentUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Activity {
                    activity_id: occurrence_id.to_string(),
                },
                "This activity was omitted because its canonical occurrence/content association was not unique.",
            );
            continue;
        }
        let mut activity = matches.remove(0);
        activity.traits = activity_traits(occurrence)
            .and_then(FactValue::as_value)
            .cloned()
            .unwrap_or_default();
        activity.content = content_for_occurrence(creature, placement, &occurrence.id);
        retained_activities.push(activity);
    }
    runtime.activities = retained_activities;

    let mut adjusted_entries = BTreeMap::<String, Vec<EncounterRuntimeSpellcastingView>>::new();
    for entry in std::mem::take(&mut runtime.spellcasting) {
        adjusted_entries
            .entry(entry.entry_id.clone())
            .or_default()
            .push(entry);
    }

    let mut entries = embedded
        .occurrences
        .iter()
        .filter(|occurrence| {
            matches!(
                occurrence.capability,
                CreatureCapability::SpellcastingEntry(_)
            )
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.authored_order);
    let entry_counts = entries
        .iter()
        .fold(BTreeMap::<&str, usize>::new(), |mut counts, entry| {
            *counts.entry(entry.id.as_str()).or_default() += 1;
            counts
        });

    let mut grouped_spells = BTreeMap::<String, Vec<&CreatureEntityOccurrence>>::new();
    let mut standalone_spells = Vec::new();
    for occurrence in embedded
        .occurrences
        .iter()
        .filter(|occurrence| matches!(occurrence.capability, CreatureCapability::Spell(_)))
    {
        match &occurrence.parent {
            CreatureOccurrenceParent::SpellcastingEntry(parent) => grouped_spells
                .entry(parent.as_str().to_string())
                .or_default()
                .push(occurrence),
            CreatureOccurrenceParent::Creature => standalone_spells.push(occurrence),
        }
    }

    let mut projected_entries = Vec::new();
    for entry in entries {
        let entry_id = entry.id.as_str();
        let mut adjusted = adjusted_entries.remove(entry_id).unwrap_or_default();
        let safe = entry_counts.get(entry_id) == Some(&1)
            && occurrence_counts.get(entry_id) == Some(&1)
            && matches!(entry.parent, CreatureOccurrenceParent::Creature)
            && actor_owned_target_is_unique(entry, &entity_counts)
            && adjusted.len() <= 1;
        if !safe {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Spellcasting {
                    entry_id: entry_id.to_string(),
                },
                "This spellcasting entry was omitted because its canonical identity or grouping was not unique.",
            );
            grouped_spells.remove(entry_id);
            continue;
        }
        let CreatureCapability::SpellcastingEntry(capability) = &entry.capability else {
            continue;
        };
        let mut view = adjusted
            .pop()
            .unwrap_or_else(|| EncounterRuntimeSpellcastingView {
                entry_id: entry_id.to_string(),
                authored_order: entry.authored_order,
                label: occurrence_label(entry, embedded),
                preparation: None,
                tradition: None,
                attack: None,
                dc: None,
                slots: Vec::new(),
                spells: Vec::new(),
            });
        view.authored_order = entry.authored_order;
        view.label = occurrence_label(entry, embedded);
        view.preparation = capability.preparation.as_value().and_then(preparation_name);
        view.tradition = capability.tradition.as_value().cloned();
        view.slots.sort_by_key(|slot| slot.rank);
        view.spells = grouped_spells
            .remove(entry_id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|spell| {
                encounter_spell_view(
                    spell,
                    creature,
                    embedded,
                    placement,
                    &occurrence_counts,
                    &entity_counts,
                    &mut runtime_activities,
                    runtime,
                )
            })
            .collect();
        view.spells.sort_by_key(|spell| spell.authored_order);
        projected_entries.push(view);
    }
    projected_entries.sort_by_key(|entry| entry.authored_order);
    runtime.spellcasting = projected_entries;

    standalone_spells.sort_by_key(|spell| spell.authored_order);
    runtime.standalone_spells = standalone_spells
        .into_iter()
        .filter_map(|spell| {
            encounter_spell_view(
                spell,
                creature,
                embedded,
                placement,
                &occurrence_counts,
                &entity_counts,
                &mut runtime_activities,
                runtime,
            )
        })
        .collect();

    for spells in grouped_spells.into_values() {
        for spell in spells {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Spell {
                    occurrence_id: spell.id.as_str().to_string(),
                },
                "This spell was omitted because its spellcasting parent is dangling.",
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn encounter_spell_view(
    spell: &CreatureEntityOccurrence,
    creature: &CreatureRecord,
    embedded: &CreatureEmbeddedEntities,
    placement: &CreatureContentPlacement,
    occurrence_counts: &BTreeMap<&str, usize>,
    entity_counts: &BTreeMap<&str, usize>,
    runtime_activities: &mut BTreeMap<String, Vec<EncounterRuntimeActivityView>>,
    runtime: &mut EncounterRuntimeView,
) -> Option<EncounterRuntimeSpellView> {
    let occurrence_id = spell.id.as_str();
    let mut activities = runtime_activities.remove(occurrence_id).unwrap_or_default();
    let safe = occurrence_counts.get(occurrence_id) == Some(&1)
        && actor_owned_target_is_unique(spell, entity_counts)
        && placement.failure(&spell.id).is_none()
        && activities.len() <= 1;
    if !safe {
        push_payload_limitation(
            runtime,
            EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable,
            EncounterRuntimeAutomationLimitationTargetView::Spell {
                occurrence_id: occurrence_id.to_string(),
            },
            "This spell was omitted because its canonical occurrence, content, or runtime activity association was not unique.",
        );
        return None;
    }
    let CreatureCapability::Spell(capability) = &spell.capability else {
        return None;
    };
    let activity = activities
        .pop()
        .filter(|activity| activity.kind == EncounterRuntimeActivityKindView::Spell);
    let provenance = activity
        .as_ref()
        .map_or_else(canonical_runtime_provenance, |activity| {
            activity.provenance.clone()
        });
    Some(EncounterRuntimeSpellView {
        occurrence_id: occurrence_id.to_string(),
        authored_order: spell.authored_order,
        label: occurrence_label(spell, embedded),
        target_record_key: match &spell.target {
            CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
            CreatureEntityTarget::ActorOwned(_) => None,
        },
        rank: spell.context.rank.as_value().copied(),
        traits: capability.traits.as_value().cloned().unwrap_or_default(),
        content: content_for_occurrence(creature, placement, &spell.id),
        activity,
        cast: unavailable_spell_cast(),
        provenance,
    })
}

fn actor_owned_target_is_unique(
    occurrence: &CreatureEntityOccurrence,
    entity_counts: &BTreeMap<&str, usize>,
) -> bool {
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(_) => true,
        CreatureEntityTarget::ActorOwned(entity_id) => {
            entity_counts.get(entity_id.as_str()) == Some(&1)
        }
    }
}

fn preparation_name(value: &CreatureSpellPreparation) -> Option<String> {
    match value {
        CreatureSpellPreparation::Prepared => Some("prepared".to_string()),
        CreatureSpellPreparation::Spontaneous => Some("spontaneous".to_string()),
        CreatureSpellPreparation::Focus => Some("focus".to_string()),
        CreatureSpellPreparation::Innate => Some("innate".to_string()),
        CreatureSpellPreparation::Ritual => Some("ritual".to_string()),
        CreatureSpellPreparation::Unsupported(_) => None,
    }
}

fn canonical_runtime_provenance() -> RuntimeFactProvenanceView {
    RuntimeFactProvenanceView {
        source: RuntimeFactSourceView::CanonicalRecord,
        canonical_target: None,
    }
}

fn push_payload_limitation(
    runtime: &mut EncounterRuntimeView,
    code: EncounterRuntimeAutomationLimitationCodeView,
    target: EncounterRuntimeAutomationLimitationTargetView,
    message: &str,
) {
    runtime
        .automation_limitations
        .push(EncounterRuntimeAutomationLimitationView {
            code,
            target,
            message: message.to_string(),
        });
}

fn content_for_occurrence(
    creature: &CreatureRecord,
    placement: &CreatureContentPlacement,
    occurrence_id: &atlas_record::CreatureOccurrenceId,
) -> Option<Vec<CreatureSurfaceContentView>> {
    non_empty(
        placement
            .documents_for_occurrence(creature, occurrence_id)
            .filter_map(content_view)
            .collect(),
    )
}

fn content(
    creature: &CreatureRecord,
    activity_content: &CreatureContentPlacement,
) -> Option<Vec<CreatureSurfaceContentView>> {
    non_empty(
        activity_content
            .association_safe_general_documents(creature)
            .filter_map(content_view)
            .collect(),
    )
}

pub(crate) fn content_view(
    document: &atlas_record::OwnedRichContentDocument,
) -> Option<CreatureSurfaceContentView> {
    content_view_from_projection(document, project_record_surface_content(&document.document))
}

fn content_view_from_projection(
    document: &atlas_record::OwnedRichContentDocument,
    content: PresentationContent,
) -> Option<CreatureSurfaceContentView> {
    let blocks = project_content(content);
    (!blocks.is_empty()).then(|| CreatureSurfaceContentView {
        content_key: document.id.content_key.as_str().to_string(),
        role: content_role(document.role),
        authored_order: document.authored_order,
        label: document.label.clone(),
        blocks,
        content_hash: document.content_hash.as_str().to_string(),
        visibility: document.visibility.as_str().to_string(),
        provenance: CreatureSurfaceContentProvenanceView {
            source_record_key: document.provenance.source_record_key.to_string(),
            relative_source_path: document.provenance.relative_source_path.clone(),
            field_family: document.provenance.field_or_pointer_family.clone(),
            nested_source_id: document.provenance.nested_source_id.clone(),
        },
    })
}

pub(crate) fn project_content(
    content: PresentationContent,
) -> Vec<CreatureSurfaceContentBlockView> {
    content
        .blocks
        .into_iter()
        .map(project_content_block)
        .collect()
}

fn project_content_block(block: PresentationContentBlock) -> CreatureSurfaceContentBlockView {
    match block {
        PresentationContentBlock::Heading { level, text } => {
            CreatureSurfaceContentBlockView::Heading { level, text }
        }
        PresentationContentBlock::Paragraph { spans } => {
            CreatureSurfaceContentBlockView::Paragraph {
                spans: spans.into_iter().map(project_content_inline).collect(),
            }
        }
        PresentationContentBlock::List { ordered, items } => {
            CreatureSurfaceContentBlockView::List {
                ordered,
                items: items
                    .into_iter()
                    .map(|item| CreatureSurfaceContentListItemView {
                        blocks: item.blocks.into_iter().map(project_content_block).collect(),
                    })
                    .collect(),
            }
        }
        PresentationContentBlock::Table { caption, rows } => {
            CreatureSurfaceContentBlockView::Table {
                caption,
                rows: rows
                    .into_iter()
                    .map(|row| CreatureSurfaceContentTableRowView {
                        cells: row.cells.into_iter().map(project_content).collect(),
                    })
                    .collect(),
            }
        }
        PresentationContentBlock::Rule => CreatureSurfaceContentBlockView::Divider,
    }
}

fn project_content_inline(span: PresentationInline) -> CreatureSurfaceContentInlineView {
    match span {
        PresentationInline::Text { text } => CreatureSurfaceContentInlineView::Text { text },
        PresentationInline::Strong { spans } => CreatureSurfaceContentInlineView::Strong {
            spans: spans.into_iter().map(project_content_inline).collect(),
        },
        PresentationInline::Emphasis { spans } => CreatureSurfaceContentInlineView::Emphasis {
            spans: spans.into_iter().map(project_content_inline).collect(),
        },
        PresentationInline::Code { text } => CreatureSurfaceContentInlineView::Code { text },
        PresentationInline::Reference {
            label,
            record_key,
            embedded,
        } => CreatureSurfaceContentInlineView::Reference {
            label,
            record_key: record_key.map(|key| key.to_string()),
            embedded,
        },
        PresentationInline::Check {
            display,
            statistic,
            difficulty_class,
        } => CreatureSurfaceContentInlineView::Check {
            display,
            statistic,
            difficulty_class,
        },
        PresentationInline::LineBreak => CreatureSurfaceContentInlineView::LineBreak,
    }
}

fn content_role(role: ContentRole) -> CreatureSurfaceContentRoleView {
    match role {
        ContentRole::PrimaryDescription => CreatureSurfaceContentRoleView::PrimaryDescription,
        ContentRole::Summary => CreatureSurfaceContentRoleView::Summary,
        ContentRole::SupplementalRules => CreatureSurfaceContentRoleView::SupplementalRules,
        ContentRole::EmbeddedCapability => CreatureSurfaceContentRoleView::EmbeddedCapability,
        ContentRole::JournalPage => CreatureSurfaceContentRoleView::JournalPage,
        ContentRole::TableResult => CreatureSurfaceContentRoleView::TableResult,
        ContentRole::GeneratedNarrative => CreatureSurfaceContentRoleView::GeneratedNarrative,
        ContentRole::Provenance => CreatureSurfaceContentRoleView::Provenance,
    }
}

fn occurrence_label(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
) -> String {
    if let Some(label) = occurrence.context.contextual_label.as_value() {
        return label.clone();
    }
    if let CreatureEntityTarget::ActorOwned(id) = &occurrence.target
        && let Some(entity) = embedded.entities.iter().find(|entity| &entity.id == id)
    {
        return entity.label.clone();
    }
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(key) => key.to_string(),
        CreatureEntityTarget::ActorOwned(id) => id.as_str().to_string(),
    }
}

fn action_cost(
    value: &CreatureActionCost,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<CreatureSurfaceActionCostView> {
    match value {
        CreatureActionCost::Passive => Some(CreatureSurfaceActionCostView::Passive),
        CreatureActionCost::Reaction => Some(CreatureSurfaceActionCostView::Reaction),
        CreatureActionCost::FreeAction => Some(CreatureSurfaceActionCostView::FreeAction),
        CreatureActionCost::Actions(count) => {
            Some(CreatureSurfaceActionCostView::Actions { count: *count })
        }
        CreatureActionCost::Time(value) => Some(CreatureSurfaceActionCostView::Time {
            value: value.clone(),
        }),
        CreatureActionCost::Unsupported(_) => {
            unsupported(
                unavailable,
                SurfaceDomain::Activities,
                CreatureSurfaceUnavailableFieldView::ActivityActionCost,
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
                Some(component_id.to_string()),
            );
            None
        }
    }
}

fn rolls(
    values: &[CreatureRoll],
    unavailable: &mut SurfaceUnavailableDomains,
    activity_id: &str,
) -> Vec<CreatureSurfaceRollView> {
    values
        .iter()
        .map(|value| {
            let component_id = format!("{activity_id}/{}", value.id);
            CreatureSurfaceRollView {
                roll_id: value.id.clone(),
                label: value.label.clone(),
                kind: match value.kind {
                    CreatureRollKind::Attack => "attack",
                    CreatureRollKind::DifficultyClass => "difficulty_class",
                    CreatureRollKind::Check => "check",
                }
                .to_string(),
                modifier: required_fact(
                    &value.value,
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::ActivityRoll,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id),
                )
                .copied(),
            }
        })
        .collect()
}

fn damage(
    projection: ActivityDamageProjection<'_>,
    unavailable: &mut SurfaceUnavailableDomains,
    activity_id: &str,
) -> Vec<CreatureSurfaceDamageView> {
    let values = match projection {
        ActivityDamageProjection::RequiredStrike(values) => required_fact(
            values,
            unavailable,
            SurfaceDomain::Activities,
            CreatureSurfaceUnavailableFieldView::ActivityDamage,
            CreatureSurfaceSourceFieldView::EmbeddedEntities,
            Some(activity_id.to_string()),
        ),
        ActivityDamageProjection::OptionalAction(values) => optional_fact(values),
    };
    let Some(values) = values else {
        return Vec::new();
    };
    values
        .iter()
        .map(|value| {
            let component_id = format!("{activity_id}/{}", value.id);
            CreatureSurfaceDamageView {
                damage_id: value.id.clone(),
                formula: required_fact(
                    &value.formula,
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::DamageFormula,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id.clone()),
                )
                .cloned(),
                damage_type: required_fact(
                    &value.damage_type,
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::DamageType,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id),
                )
                .cloned(),
                category: text(&value.category),
            }
        })
        .collect()
}

fn preparation(
    value: &CreatureSpellPreparation,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<String> {
    match value {
        CreatureSpellPreparation::Prepared => Some("prepared".to_string()),
        CreatureSpellPreparation::Spontaneous => Some("spontaneous".to_string()),
        CreatureSpellPreparation::Focus => Some("focus".to_string()),
        CreatureSpellPreparation::Innate => Some("innate".to_string()),
        CreatureSpellPreparation::Ritual => Some("ritual".to_string()),
        CreatureSpellPreparation::Unsupported(_) => {
            unsupported(
                unavailable,
                SurfaceDomain::Spellcasting,
                CreatureSurfaceUnavailableFieldView::SpellPreparation,
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
                Some(component_id.to_string()),
            );
            None
        }
    }
}

fn integer(value: &FactValue<i64>) -> Option<i64> {
    value.as_value().copied()
}

fn boolean(value: &FactValue<bool>) -> Option<bool> {
    value.as_value().copied()
}

fn text(value: &FactValue<String>) -> Option<String> {
    value.as_value().cloned()
}

fn note(value: &FactValue<atlas_record::CreatureNote>) -> Option<String> {
    value.as_value().map(|value| value.as_str().to_string())
}

fn non_empty<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use atlas_domain::{PackName, RecordId, RecordKey, RecordKind, RemasterLinkSource};
    use atlas_index::SqliteIndexReader;
    use atlas_ingest::{BuildArtifactOptions, build_artifact};
    use atlas_record::{
        ContentId, ContentIdentityStability, ContentKey, ContentOrigin, ContentOwner,
        ContentProvenance, ContentRole, ContentSourceKind, ContentVisibility,
        CreatureActionCapability, CreatureArmorClass, CreatureDefenses, CreatureEmbeddedEntities,
        CreatureEntity, CreatureEntityFamily, CreatureEntityTarget, CreatureFact, CreatureFamily,
        CreatureHitPoints, CreatureIdentity, CreatureIwr, CreatureIwrKind, CreatureLanguages,
        CreatureLegacyAbilities, CreatureNote, CreatureNumber, CreatureOccurrenceParent,
        CreaturePerception, CreatureProvenance, CreatureSave, CreatureSaveKind, CreatureSaves,
        CreatureSense, CreatureSourceField, DuplicateContentStatus, FactValue, FoundryNode,
        OwnedRichContent, OwnedRichContentDocument, RecordBody, RichDocument, RichNode,
        UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
    };

    use super::{
        activity_content_placement, compose_encounter_payload, creature_surface, non_empty,
        teaser_from_text,
    };
    use atlas_app_model::{
        CreatureSurfaceContentBlockView, CreatureSurfaceContentInlineView,
        CreatureSurfaceDomainUnavailableView, CreatureSurfaceFactOwnerView,
        CreatureSurfaceSourceFieldView, CreatureSurfaceUnavailableFieldView,
        CreatureSurfaceUnavailableStateView, EncounterRuntimeActivityKindView,
        EncounterRuntimeActivityUsageView, EncounterRuntimeActivityView,
        EncounterRuntimeAutomationLimitationCodeView, EncounterRuntimeSpellcastingView,
        EncounterRuntimeView, EncounterRuntimeVitalsView, RecordDetailRequest,
        RecordSurfaceEditionCounterpartRoleView, RecordSurfaceEditionStatusView,
        RecordSurfaceProfileView, RuntimeFactProvenanceView, RuntimeFactSourceView,
        RuntimeNumberView,
    };
    use atlas_search::{
        AtlasRetrievalService, GetRecordRequest, RecordRetrieval, RemasterLinkResult,
        RemasterLinksResult,
    };

    type CauseTuple = (
        CreatureSurfaceUnavailableStateView,
        CreatureSurfaceUnavailableFieldView,
        Option<String>,
        CreatureSurfaceFactOwnerView,
        CreatureSurfaceSourceFieldView,
    );

    const NIGHT_HAG_KEY: &str = "pathfinder-bestiary:WQy7HBUcgDLsfVJd";
    const GIANT_RAT_KEY: &str = "pathfinder-monster-core:iIJPJcDT8wlJ8z5M";
    const AIR_MEPHIT_KEY: &str = "pathfinder-bestiary:KDRlxdIUADWHI6Vr";
    const AIR_SCAMP_KEY: &str = "pathfinder-monster-core:MSm1im7lZA5i82rz";

    struct TemporarySpellSurfaceArtifact {
        root: PathBuf,
        artifact: PathBuf,
    }

    impl TemporarySpellSurfaceArtifact {
        fn new() -> Result<Self, std::io::Error> {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "pf2e-atlas-h2-e-spell-surface-{}-{nonce}",
                std::process::id()
            ));
            std::fs::create_dir_all(&root)?;
            Ok(Self {
                artifact: root.join("index.sqlite"),
                root,
            })
        }
    }

    impl Drop for TemporarySpellSurfaceArtifact {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn known_spell<T>(value: T) -> atlas_record::SpellFact<T> {
        FactValue::Value(atlas_record::SpellSourceValue::Known(value))
    }

    fn spell_rule_element(
        authored_order: u32,
        rule: atlas_record::SpellRule,
    ) -> atlas_record::SpellRuleElement {
        atlas_record::SpellRuleElement {
            authored_order,
            source_path: format!("/system/rules/{authored_order}"),
            authored_key: "fixture-rule".to_string(),
            authored_object_json: "{}".to_string(),
            rule,
        }
    }

    #[test]
    fn spell_surface_heightening_uses_semantic_unique_member_associations() {
        let damage = atlas_record::SpellOrderedMember {
            key: "damage-member".to_string(),
            authored_order: 0,
            value: atlas_record::SpellDamage {
                formula: known_spell("2d4".to_string()),
                damage_type: known_spell("cold".to_string()),
                kinds: known_spell(vec!["damage".to_string()]),
                ..atlas_record::SpellDamage::default()
            },
        };
        let healing = atlas_record::SpellOrderedMember {
            key: "healing-member".to_string(),
            authored_order: 1,
            value: atlas_record::SpellDamage {
                formula: known_spell("2d8".to_string()),
                kinds: known_spell(vec!["healing".to_string()]),
                ..atlas_record::SpellDamage::default()
            },
        };
        let duplicate = atlas_record::SpellOrderedMember {
            key: "duplicate".to_string(),
            authored_order: 2,
            value: damage.value.clone(),
        };
        let base = vec![damage, healing, duplicate.clone(), duplicate];

        let interval =
            atlas_record::SpellHeightening::Interval(atlas_record::SpellIntervalHeightening {
                interval: known_spell(1),
                area: FactValue::Missing,
                damage: known_spell(vec![atlas_record::SpellOrderedMember {
                    key: "damage-member".to_string(),
                    authored_order: 0,
                    value: "2d4".to_string(),
                }]),
            });
        let interval = serde_json::to_value(super::spell_heightening(&interval, Some(&base)))
            .expect("interval heightening should serialize");
        assert_eq!(interval["damage"]["value"][0]["label"], "Damage");
        assert_eq!(interval["damage"]["value"][0]["value"], "2d4");

        let fixed =
            atlas_record::SpellHeightening::Fixed(vec![atlas_record::SpellFixedHeighteningLayer {
                key: "5".to_string(),
                authored_order: 0,
                rank: atlas_record::SpellSourceValue::Known(5),
                patch: atlas_record::SpellPatch {
                    damage: known_spell(atlas_record::SpellKeyedPatch {
                        members: vec![
                            atlas_record::SpellKeyedPatchMember {
                                key: "damage-member".to_string(),
                                authored_order: 0,
                                operation: atlas_record::SpellKeyedPatchOperation::Merge(
                                    atlas_record::SpellDamagePatch {
                                        formula: known_spell("8d4".to_string()),
                                        ..atlas_record::SpellDamagePatch::default()
                                    },
                                ),
                            },
                            atlas_record::SpellKeyedPatchMember {
                                key: "healing-member".to_string(),
                                authored_order: 1,
                                operation: atlas_record::SpellKeyedPatchOperation::Delete,
                            },
                            atlas_record::SpellKeyedPatchMember {
                                key: "duplicate".to_string(),
                                authored_order: 2,
                                operation: atlas_record::SpellKeyedPatchOperation::Delete,
                            },
                        ],
                    }),
                    ..atlas_record::SpellPatch::default()
                },
            }]);
        let fixed = serde_json::to_value(super::spell_heightening(&fixed, Some(&base)))
            .expect("fixed heightening should serialize");
        let changes = fixed["layers"][0]["changes"]
            .as_array()
            .expect("fixed changes");
        assert_eq!(changes[0]["label"], "Damage");
        assert_eq!(changes[0]["value"]["formula"]["value"], "8d4");
        assert_eq!(changes[1]["label"], "Healing");
        assert_eq!(changes[2]["label"], "Effect");
        assert!(!fixed.to_string().contains("damage-member"));
        assert!(!fixed.to_string().contains("healing-member"));
        assert!(!fixed.to_string().contains("duplicate"));
    }

    #[test]
    fn spell_surface_maps_all_supported_rule_variants_without_raw_owner_fields() {
        use atlas_record::SpellRule;

        let rules = vec![
            spell_rule_element(
                0,
                SpellRule::DamageDice(atlas_record::SpellDamageDiceRule {
                    selector: known_spell("damage".to_string()),
                    hide_if_disabled: known_spell(false),
                    ..atlas_record::SpellDamageDiceRule::default()
                }),
            ),
            spell_rule_element(
                1,
                SpellRule::EphemeralEffect(atlas_record::SpellEphemeralEffectRule {
                    selectors: known_spell(vec!["target".to_string()]),
                    ..atlas_record::SpellEphemeralEffectRule::default()
                }),
            ),
            spell_rule_element(
                2,
                SpellRule::DamageAlteration(atlas_record::SpellDamageAlterationRule {
                    mode: known_spell("override".to_string()),
                    ..atlas_record::SpellDamageAlterationRule::default()
                }),
            ),
            spell_rule_element(
                3,
                SpellRule::RollOption(atlas_record::SpellRollOptionRule {
                    toggleable: known_spell(false),
                    ..atlas_record::SpellRollOptionRule::default()
                }),
            ),
            spell_rule_element(
                4,
                SpellRule::ItemAlteration(atlas_record::SpellItemAlterationRule {
                    property: known_spell("badge-value".to_string()),
                    ..atlas_record::SpellItemAlterationRule::default()
                }),
            ),
        ];
        let projected = serde_json::to_value(super::spell_rules(&rules))
            .expect("supported rules should serialize");
        assert_eq!(
            projected
                .as_array()
                .expect("rules")
                .iter()
                .map(|rule| rule["rule"]["kind"].as_str().expect("rule kind"))
                .collect::<Vec<_>>(),
            vec![
                "damage_dice",
                "ephemeral_effect",
                "damage_alteration",
                "roll_option",
                "item_alteration",
            ]
        );
        assert_eq!(
            projected[0]["rule"]["value"]["hide_if_disabled"]["value"],
            false
        );
        assert_eq!(projected[3]["rule"]["value"]["toggleable"]["value"], false);
        for forbidden in ["source_path", "authored_key", "authored_object_json"] {
            assert!(!projected.to_string().contains(forbidden));
        }
    }

    #[test]
    fn source_backed_fireball_key_zero_is_internal_to_semantic_app_association()
    -> Result<(), Box<dyn std::error::Error>> {
        // This one-record fixture is copied from the exact pinned source path so the authored
        // member key and its semantic app projection are exercised without a corpus build.
        let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../atlas-ingest/tests/fixtures/foundry-source/fireball-source-contract");
        let artifact = TemporarySpellSurfaceArtifact::new()?;
        build_artifact(BuildArtifactOptions {
            source_root,
            output_path: artifact.artifact.clone(),
            manifest_path: None,
            embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
            embedding_cache_root: None,
            reuse_embeddings: true,
            embedding_batch_size: 8,
        })?;
        let retrieval = AtlasRetrievalService::from_prepared_index_without_embeddings(
            SqliteIndexReader::open_read_only(&artifact.artifact)?,
        );
        let fireball = retrieve_spell(&retrieval, "spells-srd:sxQZ6yqTn0czJxVd")?;
        let Some(RecordBody::Spell(spell)) = fireball.body.as_ref() else {
            panic!("Fireball should hydrate as a spell");
        };
        assert_eq!(
            spell.definition.provenance.source_path,
            "packs/spells/3rd-rank/fireball.json"
        );
        let FactValue::Value(atlas_record::SpellSourceValue::Known(damage)) =
            &spell.definition.damage
        else {
            panic!("Fireball damage should be known");
        };
        assert_eq!(damage.len(), 1);
        assert_eq!(damage[0].key, "0");
        assert_eq!(damage[0].value.formula, known_spell("6d6".to_string()));
        let FactValue::Value(atlas_record::SpellSourceValue::Known(
            atlas_record::SpellHeightening::Interval(heightening),
        )) = &spell.definition.heightening
        else {
            panic!("Fireball interval heightening should be known");
        };
        let FactValue::Value(atlas_record::SpellSourceValue::Known(heightening_damage)) =
            &heightening.damage
        else {
            panic!("Fireball heightening damage should be known");
        };
        assert_eq!(heightening_damage.len(), 1);
        assert_eq!(heightening_damage[0].key, "0");
        assert_eq!(heightening_damage[0].value, "2d6");

        let surface = spell_surface_json(&fireball);
        assert!(surface["metadata"].get("level").is_none());
        assert!(surface["metadata"].get("traits").is_none());
        assert_eq!(surface["presentation"]["body"]["family"], "spell");
        assert_eq!(
            surface.pointer(
                "/presentation/body/effective_form/result/definition/classification/value/value/rank/value"
            ),
            Some(&serde_json::json!(3))
        );
        assert_eq!(
            surface.pointer(
                "/presentation/body/effective_form/result/definition/classification/value/value/traits/value"
            ),
            Some(&serde_json::json!(["concentrate", "fire", "manipulate"]))
        );
        let damage = surface
            .pointer("/presentation/body/effective_form/result/definition/damage/value/value/0")
            .expect("Fireball app damage");
        assert_eq!(damage["label"], "Damage");
        assert_eq!(damage["formula"]["value"], "6d6");
        assert_eq!(damage["damage_type"]["value"], "fire");
        assert!(damage.get("key").is_none());
        let increment = surface
            .pointer("/presentation/body/effective_form/result/definition/heightening/value/value/damage/value/0")
            .expect("Fireball app heightening damage");
        assert_eq!(increment["label"], "Damage");
        assert_eq!(increment["value"], "2d6");
        assert!(increment.get("key").is_none());

        Ok(())
    }

    #[test]
    fn blazing_content_uses_only_resolved_selection_and_preserves_source()
    -> Result<(), Box<dyn std::error::Error>> {
        // Exact 4cbdaa37 packs/spells/2nd-rank/blazing-blade.json, unchanged.
        let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../atlas-ingest/tests/fixtures/foundry-source/spell-source-contract");
        let artifact = TemporarySpellSurfaceArtifact::new()?;
        build_artifact(BuildArtifactOptions {
            source_root,
            output_path: artifact.artifact.clone(),
            manifest_path: None,
            embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
            embedding_cache_root: None,
            reuse_embeddings: true,
            embedding_batch_size: 8,
        })?;
        let retrieval = AtlasRetrievalService::from_prepared_index_without_embeddings(
            SqliteIndexReader::open_read_only(&artifact.artifact)?,
        );
        let record = retrieve_spell(&retrieval, "spells-srd:NacrNSvfODxpZena")?;
        let Some(RecordBody::Spell(spell)) = record.body.as_ref() else {
            panic!("spell");
        };
        let before = spell.clone();
        let defaults = || {
            spell
                .definition
                .content
                .documents
                .iter()
                .map(|document| {
                    (
                        atlas_record::ContentHash::for_document(&document.document),
                        atlas_record::render_plain_text(&document.document),
                        atlas_record::render_markdown_like(&document.document),
                        atlas_record::project_presentation_content(&document.document),
                    )
                })
                .collect::<Vec<_>>()
        };
        let default_before = defaults();
        let semantic_inputs = || {
            let fts = atlas_record::build_search_fts_projection(
                &record.record,
                &[],
                record.body.as_ref(),
            );
            let search = atlas_record::build_search_presentation_document_with_content_filter(
                &record.record,
                record.body.as_ref(),
                |_| false,
            );
            let embedding = atlas_embedding::render_presentation_document_for_embedding(&search);
            let hash = atlas_embedding::hash_document_embedding_input(&embedding);
            (fts, search, embedding, hash)
        };
        let semantic_before = semantic_inputs();
        fn find_damage(nodes: &[atlas_record::RichNode]) -> Option<&atlas_record::FoundryNode> {
            nodes.iter().find_map(|node| match node {
                atlas_record::RichNode::Foundry {
                    node: node @ atlas_record::FoundryNode::Damage { .. },
                } => Some(node),
                atlas_record::RichNode::HtmlElement { children, .. } => find_damage(children),
                _ => None,
            })
        }
        let damage = spell
            .definition
            .content
            .documents
            .iter()
            .find_map(|doc| find_damage(&doc.document.nodes))
            .expect("authored damage");
        let atlas_record::FoundryNode::Damage {
            formula,
            damage_parts,
            ..
        } = damage
        else {
            panic!("damage");
        };
        assert_eq!(
            formula,
            "(ternary(gte(@item.level,8),3,ternary(gte(@item.level,6),2,1)))d6[persistent,spirit]"
        );
        assert_eq!(
            damage_parts
                .iter()
                .map(|part| part.formula.as_str())
                .collect::<Vec<_>>(),
            [
                "(ternary(gte(@item.level",
                "8)",
                "3",
                "ternary(gte(@item.level",
                "6)",
                "2",
                "1)))d6"
            ]
        );

        let id = atlas_record::SpellFormId::base(&spell.identity.record_key);
        for (rank, expected) in [
            (2, "1d6 persistent spirit"),
            (4, "1d6 persistent spirit"),
            (6, "2d6 persistent spirit"),
            (8, "3d6 persistent spirit"),
        ] {
            let surface = spell_surface_json_with_selection(&record, Some((id.clone(), rank)));
            assert_eq!(
                surface["presentation"]["body"]["effective_form"]["id"],
                id.as_str()
            );
            assert_eq!(
                surface["presentation"]["body"]["effective_form"]["cast_rank"],
                rank
            );
            let content = surface["presentation"]["body"]["content"].to_string();
            assert!(content.contains(expected), "{rank}: {content}");
            assert!(!content.contains("ternary") && !content.contains("@item.level"));
        }
        for (form, rank) in [
            (id.clone(), 1),
            (
                atlas_record::SpellFormId::base(&atlas_domain::RecordKey::parse(
                    "spells-srd:wrong",
                )?),
                8,
            ),
        ] {
            let surface = spell_surface_json_with_selection(&record, Some((form, rank)));
            assert_eq!(
                surface["presentation"]["body"]["effective_form"]["result"]["state"],
                "unavailable"
            );
            let content = surface["presentation"]["body"]["content"].to_string();
            assert!(content.contains("Damage unavailable"));
            assert!(!content.contains("ternary") && !content.contains("@item.level"));
            assert!(
                !content.contains("\"text\":\"3d6 persistent spirit\"")
                    && !content.contains("\"text\":\"1d6 persistent spirit\"")
            );
            let issues = surface["issues"].as_array().expect("issues");
            assert_eq!(
                issues
                    .iter()
                    .filter(|issue| issue["fact_label"] == "Description damage")
                    .count(),
                1
            );
        }
        // Ordinary Damage remains ordinary in selected spell, creature and hazard
        // surface owners, including an unavailable spell selection.
        let mut ordinary_record = record.clone();
        let Some(RecordBody::Spell(ordinary_spell)) = ordinary_record.body.as_mut() else {
            panic!("spell")
        };
        let document = ordinary_spell
            .definition
            .content
            .documents
            .first_mut()
            .expect("document");
        document.document.nodes = vec![atlas_record::RichNode::Foundry {
            node: atlas_record::FoundryNode::Damage {
                formula: "(1)d6[fire]".into(),
                options: Default::default(),
                label: None,
                damage_parts: vec![atlas_record::DamagePart {
                    formula: "(1)d6".into(),
                    damage_type: Some("fire".into()),
                }],
            },
        }];
        document.content_hash = atlas_record::ContentHash::for_document(&document.document);
        let ordinary_document = document.clone();
        for rank in [1, 2, 8] {
            let surface =
                spell_surface_json_with_selection(&ordinary_record, Some((id.clone(), rank)));
            let content = surface["presentation"]["body"]["content"].to_string();
            assert!(content.contains("(1)d6 fire"));
            assert!(!content.contains("Damage unavailable"));
            assert!(!surface["issues"].as_array().is_some_and(|issues| {
                issues
                    .iter()
                    .any(|issue| issue["fact_label"] == "Description damage")
            }));
        }
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        creature.content.documents.push(content_document(
            &owner,
            "public-notes",
            ContentOwner::Record(owner.clone()),
            0,
            "Public Notes",
            ordinary_document.document.nodes,
        ));
        let surface = record_surface_for_creature(creature);
        let content = serde_json::to_string(&surface)?;
        assert!(content.contains("(1)d6 fire"));

        assert_eq!(&before, spell);
        assert_eq!(default_before, defaults());
        assert_eq!(semantic_before, semantic_inputs());
        let hydrated = retrieve_spell(&retrieval, "spells-srd:NacrNSvfODxpZena")?;
        assert_eq!(record.body, hydrated.body);
        Ok(())
    }

    #[test]
    fn real_heal_rime_deity_and_planar_records_expose_typed_spell_api_surfaces_and_unsafe_form_state()
    -> Result<(), Box<dyn std::error::Error>> {
        let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../atlas-ingest/tests/fixtures/foundry-source/spell-source-contract");
        let artifact = TemporarySpellSurfaceArtifact::new()?;
        build_artifact(BuildArtifactOptions {
            source_root,
            output_path: artifact.artifact.clone(),
            manifest_path: None,
            embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
            embedding_cache_root: None,
            reuse_embeddings: true,
            embedding_batch_size: 8,
        })?;
        let retrieval = AtlasRetrievalService::from_prepared_index_without_embeddings(
            SqliteIndexReader::open_read_only(&artifact.artifact)?,
        );
        let service_artifact: &'static Path =
            Box::leak(artifact.artifact.clone().into_boxed_path());
        let service = crate::test_support::fixture_worker_with_executor(
            crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
                let reader = SqliteIndexReader::open_read_only(service_artifact)?;
                Ok((
                    AtlasRetrievalService::from_prepared_index_without_embeddings(reader),
                    (),
                ))
            }),
        );

        let mut heal = retrieve_spell(&retrieval, "spells-srd:rfZpqmj0AIIdkVIs")?;
        let heal_surface = spell_surface_json(&heal);
        assert_eq!(
            heal_surface
                .pointer("/presentation/body/effective_form/result/definition/targeting/value/value/range/value/authored_text"),
            Some(&serde_json::json!("varies"))
        );
        assert_eq!(
            heal_surface.pointer(
                "/presentation/body/effective_form/result/definition/heightening/value/value/kind"
            ),
            Some(&serde_json::json!("interval"))
        );
        let heal_forms = heal_surface["presentation"]["body"]["forms"]
            .as_array()
            .expect("Heal forms");
        assert_eq!(
            heal_forms
                .iter()
                .map(|form| form["label"].as_str().expect("form label"))
                .collect::<Vec<_>>(),
            vec![
                "Base",
                "1 action — touch",
                "Heal (vs. Living)",
                "Heal (vs. Undead)",
                "3 actions — 30-foot emanation",
            ]
        );
        for (order, form) in heal_forms.iter().enumerate() {
            assert_eq!(form["order"], serde_json::json!(order));
            assert!(
                form["id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("spell-form:"))
            );
            assert_eq!(form["minimum_cast_rank"], 1);
            assert!(form.get("authored_patch").is_none());
            assert!(form.get("result").is_none());
        }
        assert!(
            heal_surface["presentation"]["body"]
                .get("definition")
                .is_none()
        );
        assert!(
            heal_surface["presentation"]["body"]
                .get("selected_form")
                .is_none()
        );

        for (label, time, range, target, area) in [
            (
                "1 action — touch",
                "1",
                Some("touch"),
                Some("1 willing living creature or 1 undead"),
                None,
            ),
            (
                "Heal (vs. Living)",
                "2",
                Some("30 feet"),
                Some("1 willing living creature or 1 undead"),
                None,
            ),
            (
                "Heal (vs. Undead)",
                "2",
                Some("30 feet"),
                Some("1 undead"),
                None,
            ),
            (
                "3 actions — 30-foot emanation",
                "3",
                Some(""),
                Some("all living and undead creatures"),
                Some((30, "emanation")),
            ),
        ] {
            let form = heal_forms
                .iter()
                .find(|form| form["label"] == label)
                .expect("semantic Heal form");
            let selected = service.worker.record_detail(
                "spells-srd:rfZpqmj0AIIdkVIs",
                RecordDetailRequest {
                    spell_form_id: Some(form["id"].as_str().expect("form id").to_string()),
                    spell_cast_rank: Some(1),
                    ..RecordDetailRequest::default()
                },
            )?;
            let selected = serde_json::to_value(selected.surface)?;
            let definition = selected
                .pointer("/presentation/body/effective_form/result/definition")
                .expect("selected Heal definition");
            assert_eq!(
                definition.pointer("/casting/value/value/time/value"),
                Some(&serde_json::json!(time)),
                "{label} casting time"
            );
            assert_eq!(
                definition
                    .pointer("/targeting/value/value/range/value/authored_text")
                    .and_then(serde_json::Value::as_str),
                range,
                "{label} range"
            );
            assert_eq!(
                definition
                    .pointer("/targeting/value/value/target/value")
                    .and_then(serde_json::Value::as_str),
                target,
                "{label} target"
            );
            assert_eq!(
                definition
                    .pointer("/targeting/value/value/area/value")
                    .map(|area| {
                        (
                            area["value"]["value"].as_u64().expect("area value") as u32,
                            area["area_type"]["value"].as_str().expect("area type"),
                        )
                    }),
                area,
                "{label} area"
            );
        }

        let mut rime_record = retrieve_spell(&retrieval, "spells-srd:Popa5umI3H33levx")?;
        let rime = spell_surface_json(&rime_record);
        let rime_forms = rime["presentation"]["body"]["forms"]
            .as_array()
            .expect("Rime form catalog");
        assert_eq!(rime_forms.len(), 1);
        assert_eq!(rime_forms[0]["label"], "Base");
        assert_eq!(rime_forms[0]["kind"], "base");
        assert_eq!(
            rime.pointer(
                "/presentation/body/effective_form/result/definition/classification/value/value/rank/value"
            ),
            Some(&serde_json::json!(2))
        );
        assert_eq!(
            rime.pointer(
                "/presentation/body/effective_form/result/definition/targeting/value/value/area/value/value/value"
            ),
            Some(&serde_json::json!(15))
        );
        assert_eq!(
            rime.pointer(
                "/presentation/body/effective_form/result/definition/damage/value/value/0/formula/value"
            ),
            Some(&serde_json::json!("2d4"))
        );
        assert_eq!(
            rime.pointer("/presentation/body/effective_form/result/definition/applied_fixed_ranks"),
            Some(&serde_json::json!([]))
        );
        let rime_layers = rime
            .pointer("/presentation/body/effective_form/result/definition/heightening/value/value/layers")
            .and_then(serde_json::Value::as_array)
            .expect("Rime fixed heightening layers");
        assert_eq!(rime_layers[0]["rank"]["value"], 5);
        assert_eq!(
            rime_layers[0]["changes"][1]["value"]["formula"]["value"],
            "8d4"
        );
        assert_eq!(rime_layers[0]["changes"][1]["label"], "Damage");
        assert!(
            serde_json::to_string(&rime_layers)?
                .find("\"key\"")
                .is_none()
        );
        assert_eq!(rime_layers[1]["rank"]["value"], 8);
        assert_eq!(
            rime_layers[1]["changes"][1]["value"]["formula"]["value"],
            "14d4"
        );
        let base_form_id = rime
            .pointer("/presentation/body/forms/0/id")
            .and_then(serde_json::Value::as_str)
            .expect("Rime base form id")
            .to_string();
        assert_eq!(
            rime.pointer("/presentation/body/effective_form/cast_rank"),
            Some(&serde_json::json!(2))
        );
        for (cast_rank, applied_ranks, area, formula) in [
            (5, serde_json::json!([5]), 30, "8d4"),
            (8, serde_json::json!([5, 8]), 60, "14d4"),
        ] {
            let selected = service.worker.record_detail(
                "spells-srd:Popa5umI3H33levx",
                RecordDetailRequest {
                    spell_form_id: Some(base_form_id.clone()),
                    spell_cast_rank: Some(cast_rank),
                    ..RecordDetailRequest::default()
                },
            )?;
            let selected = serde_json::to_value(selected.surface)?;
            assert_eq!(
                selected.pointer("/presentation/body/effective_form/id"),
                Some(&serde_json::json!(base_form_id))
            );
            assert_eq!(
                selected.pointer("/presentation/body/effective_form/cast_rank"),
                Some(&serde_json::json!(cast_rank))
            );
            assert_eq!(
                selected.pointer("/presentation/body/forms/0/minimum_cast_rank"),
                Some(&serde_json::json!(2))
            );
            assert_eq!(
                selected.pointer(
                    "/presentation/body/effective_form/result/definition/applied_fixed_ranks"
                ),
                Some(&applied_ranks)
            );
            assert_eq!(
                selected.pointer(
                    "/presentation/body/effective_form/result/definition/damage/value/value/0/formula/value"
                ),
                Some(&serde_json::json!(formula))
            );
            assert_eq!(
                selected.pointer(
                    "/presentation/body/effective_form/result/definition/targeting/value/value/area/value/value/value"
                ),
                Some(&serde_json::json!(area))
            );
            assert_eq!(
                selected.pointer(
                    "/presentation/body/effective_form/result/definition/damage/value/value/0/label"
                ),
                Some(&serde_json::json!("Damage"))
            );
        }
        let below_base = service.worker.record_detail(
            "spells-srd:Popa5umI3H33levx",
            RecordDetailRequest {
                spell_form_id: Some(base_form_id.clone()),
                spell_cast_rank: Some(1),
                ..RecordDetailRequest::default()
            },
        )?;
        let below_base = serde_json::to_value(below_base.surface)?;
        assert_eq!(
            below_base.pointer("/presentation/body/effective_form/result/reason"),
            Some(&serde_json::json!({
                "reason": "cast_rank_below_base",
                "base_rank": 2,
                "cast_rank": 1
            }))
        );

        let RecordBody::Spell(rime_spell) = rime_record.body.as_mut().expect("Rime spell body")
        else {
            panic!("Rime should hydrate as a spell");
        };
        let FactValue::Value(atlas_record::SpellSourceValue::Known(
            atlas_record::SpellHeightening::Fixed(layers),
        )) = &mut rime_spell.definition.heightening
        else {
            panic!("Rime should retain fixed heightening");
        };
        layers[0].key = "wrong-rank-key".to_string();
        let localized = spell_surface_json_with_selection(
            &rime_record,
            Some((
                atlas_record::SpellFormId::new(base_form_id)
                    .expect("public base form id should be valid"),
                5,
            )),
        );
        assert_eq!(
            localized.pointer("/presentation/body/effective_form/result/state"),
            Some(&serde_json::json!("available"))
        );
        assert_eq!(
            localized.pointer("/presentation/body/effective_form/result/definition/damage"),
            Some(&serde_json::json!({
                "state": "unavailable",
                "unavailable": {
                    "field": "damage",
                    "source": "fixed_heightening",
                    "reason": "fixed_rank_key_mismatch"
                }
            }))
        );

        let deity = spell_surface_json(&retrieve_spell(&retrieval, "spells-srd:x9RIFhquazom4p02")?);
        assert_eq!(
            deity.pointer("/presentation/body/effective_form/result/definition/defense/value/value/passive/value"),
            Some(&serde_json::json!("ac"))
        );
        assert_eq!(
            deity.pointer("/presentation/body/effective_form/result/definition/defense/value/value/save/value/statistic/value"),
            Some(&serde_json::json!("reflex"))
        );
        assert_eq!(
            deity.pointer("/presentation/body/effective_form/result/definition/defense/value/value/save/value/basic/value"),
            Some(&serde_json::json!(true))
        );

        let planar =
            spell_surface_json(&retrieve_spell(&retrieval, "spells-srd:HmKajQS0DP23bipp")?);
        assert_eq!(planar["metadata"]["kind_label"], "Spell");
        assert_eq!(planar["presentation"]["body"]["family"], "ritual");
        assert_eq!(
            planar.pointer("/presentation/body/effective_form/result/definition/ritual/value/primary_check/value"),
            Some(&serde_json::json!(
                "Arcana (master), Nature (master), Occultism (master), or Religion (master)"
            ))
        );
        assert_eq!(
            planar.pointer("/presentation/body/effective_form/result/definition/ritual/value/secondary_casters/value"),
            Some(&serde_json::json!(2))
        );
        assert_eq!(
            planar
                .pointer("/presentation/body/effective_form/result/definition/targeting/value/value/range/value/authored_text"),
            Some(&serde_json::json!("20 feet"))
        );

        let spell_body = &heal_surface["presentation"]["body"];
        let serialized_body = serde_json::to_string(spell_body)?;
        for forbidden in [
            "authored_object_json",
            "overlay_id",
            "publication_license",
            "raw_json",
            "source_id",
            "\"numeric\"",
            "\"image\"",
        ] {
            assert!(
                !serialized_body.contains(forbidden),
                "spell app body must not expose `{forbidden}`"
            );
        }

        let RecordBody::Spell(spell) = heal.body.as_mut().expect("Heal spell body") else {
            panic!("Heal should hydrate as a spell");
        };
        let FactValue::Value(atlas_record::SpellSourceValue::Known(overlays)) =
            &mut spell.definition.overlays
        else {
            panic!("Heal should retain known overlays");
        };
        overlays[0].source_id = FactValue::Value(atlas_record::SpellSourceValue::Known(
            atlas_record::SpellOverlayId::new("different-overlay")
                .expect("valid mismatched overlay id"),
        ));
        let unsafe_surface = spell_surface_json(&heal);
        let unsafe_forms = unsafe_surface["presentation"]["body"]["forms"]
            .as_array()
            .expect("unsafe Heal forms");
        assert_eq!(unsafe_forms.len(), 4);
        assert!(unsafe_forms.iter().all(|form| form.get("result").is_none()));
        assert_eq!(
            unsafe_surface["issues"]
                .as_array()
                .expect("catalog issue")
                .iter()
                .filter(|issue| issue["placement"] == "forms")
                .count(),
            1
        );

        Ok(())
    }

    #[test]
    fn source_backed_hazard_applicability_preserves_authored_zero_and_missing_flags()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact = TemporarySpellSurfaceArtifact::new()?;
        build_artifact(BuildArtifactOptions {
            source_root: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/hazard-applicability"),
            output_path: artifact.artifact.clone(),
            manifest_path: None,
            embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
            embedding_cache_root: None,
            reuse_embeddings: false,
            embedding_batch_size: 8,
        })?;
        let retrieval = AtlasRetrievalService::from_prepared_index_without_embeddings(
            SqliteIndexReader::open_read_only(&artifact.artifact)?,
        );
        for (key, expected) in [
            (
                "pfs-season-4-bestiary:Rq4b1pFU36QDkW7c",
                atlas_app_model::HazardSurfaceApplicabilityStateView::Inapplicable,
            ),
            (
                "hazards:BHq5wpQU8hQEke8D",
                atlas_app_model::HazardSurfaceApplicabilityStateView::Applicable,
            ),
        ] {
            let key = RecordKey::parse(key)?;
            let record = retrieval
                .get_record(GetRecordRequest { record_key: &key })?
                .expect("hazard fixture");
            let Some(RecordBody::Hazard(hazard)) = record.body.as_ref() else {
                panic!("hazard body")
            };
            let view = crate::hazard_surface::hazard_surface(
                hazard,
                RecordSurfaceProfileView::RecordDetail,
                None,
            );
            if key.to_string() == "pfs-season-4-bestiary:Rq4b1pFU36QDkW7c" {
                let disable =
                    serde_json::to_value(&view.lifecycle.as_ref().expect("lifecycle").disable)?;
                let text = disable.to_string();
                assert!(
                    text.contains("spells-srd:9HpwDN4MYQJnW0LG"),
                    "resolved Disable link: {text}"
                );
                assert!(text.contains("Dispel Magic"));
                assert!(text.contains("counteract DC 20"));
            }
            let defenses = view.defenses.expect("defenses");
            assert_eq!(defenses.applicability.structure, expected);
            if expected == atlas_app_model::HazardSurfaceApplicabilityStateView::Inapplicable {
                assert_eq!(defenses.armor_class, Some(0));
                assert_eq!(defenses.hardness, Some(0));
                let hp = defenses.hit_points.expect("HP");
                assert_eq!(
                    (hp.current, hp.maximum, hp.temporary, hp.broken_threshold),
                    (Some(0), Some(0), Some(0), Some(0))
                );
                let saves = defenses.saves.expect("saves");
                assert_eq!(
                    (saves.fortitude, saves.reflex, saves.will),
                    (Some(0), Some(0), Some(0))
                );
                assert!(matches!(
                    hazard
                        .defenses
                        .typed()
                        .expect("defenses")
                        .source_metadata
                        .has_health
                        .value,
                    FactValue::Missing
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn source_backed_hazard_damage_surface_keeps_canonical_and_search_inputs_unchanged()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact = TemporarySpellSurfaceArtifact::new()?;
        build_artifact(BuildArtifactOptions {
            source_root: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/hazard-applicability"),
            output_path: artifact.artifact.clone(),
            manifest_path: None,
            embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
            embedding_cache_root: None,
            reuse_embeddings: false,
            embedding_batch_size: 8,
        })?;
        let retrieval = AtlasRetrievalService::from_prepared_index_without_embeddings(
            SqliteIndexReader::open_read_only(&artifact.artifact)?,
        );
        for (key, natural) in [
            ("hazards:BHq5wpQU8hQEke8D", "10 bludgeoning"),
            (
                "agents-of-edgewatch-bestiary:u1cuwAE3xzhYW4Mi",
                "5d6 poison",
            ),
            (
                "age-of-ashes-bestiary:HbgvZUYFJd2VfwmQ",
                "(1d10+12) bludgeoning",
            ),
        ] {
            let key = RecordKey::parse(key)?;
            let record = retrieval
                .get_record(GetRecordRequest { record_key: &key })?
                .expect("hazard");
            let Some(RecordBody::Hazard(hazard)) = record.body.as_ref() else {
                panic!("hazard")
            };
            let before = hazard.clone();
            let semantic_inputs = || {
                let fts = atlas_record::build_search_fts_projection(
                    &record.record,
                    &[],
                    record.body.as_ref(),
                );
                let search = atlas_record::build_search_presentation_document_with_content_filter(
                    &record.record,
                    record.body.as_ref(),
                    |_| false,
                );
                let embedding =
                    atlas_embedding::render_presentation_document_for_embedding(&search);
                let hash = atlas_embedding::hash_document_embedding_input(&embedding);
                let units = atlas_embedding::build_document_embedding_units(&[
                    atlas_embedding::DocumentEmbeddingSource {
                        record_key: key.to_string(),
                        record_name: record.record.identity.name.clone(),
                        document: search.clone(),
                        aliases: Vec::new(),
                        content_documents: hazard
                            .content
                            .documents
                            .iter()
                            .filter(|doc| {
                                matches!(
                                    doc.duplicate_status,
                                    atlas_record::DuplicateContentStatus::Unique
                                )
                            })
                            .map(|doc| atlas_embedding::DocumentEmbeddingContentSource {
                                source_kind: doc.source_kind,
                                label: doc.label.clone(),
                                document: doc.document.clone(),
                            })
                            .collect(),
                    },
                ]);
                (fts, search, embedding, hash, units)
            };
            let semantic_before = semantic_inputs();
            let inputs = hazard
                .content
                .documents
                .iter()
                .map(|doc| {
                    (
                        atlas_record::ContentHash::for_document(&doc.document),
                        atlas_record::render_plain_text(&doc.document),
                        atlas_record::project_presentation_content(&doc.document),
                    )
                })
                .collect::<Vec<_>>();
            let view = crate::hazard_surface::hazard_surface(
                hazard,
                RecordSurfaceProfileView::RecordDetail,
                None,
            );
            let serialized = serde_json::to_string(&view)?;
            assert!(serialized.contains(natural), "{key}: missing {natural}");
            assert!(!serialized.contains("damage damage"));
            let mut ordinary_hazard = hazard.clone();
            let document = ordinary_hazard
                .content
                .documents
                .first_mut()
                .expect("hazard content");
            document.document.nodes = vec![atlas_record::RichNode::Foundry {
                node: atlas_record::FoundryNode::Damage {
                    formula: "(1)d6[fire]".into(),
                    options: Default::default(),
                    label: None,
                    damage_parts: vec![atlas_record::DamagePart {
                        formula: "(1)d6".into(),
                        damage_type: Some("fire".into()),
                    }],
                },
            }];
            document.content_hash = atlas_record::ContentHash::for_document(&document.document);
            let ordinary = crate::hazard_surface::hazard_surface(
                &ordinary_hazard,
                RecordSurfaceProfileView::RecordDetail,
                None,
            );
            let ordinary = serde_json::to_string(&ordinary)?;
            assert!(ordinary.contains("(1)d6 fire"), "{key}: {ordinary}");
            assert_eq!(*hazard, before);
            let after = hazard
                .content
                .documents
                .iter()
                .map(|doc| {
                    (
                        atlas_record::ContentHash::for_document(&doc.document),
                        atlas_record::render_plain_text(&doc.document),
                        atlas_record::project_presentation_content(&doc.document),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(inputs, after);
            assert_eq!(semantic_before, semantic_inputs());
            assert!(
                semantic_before.0.body.contains("[bludgeoning]")
                    || semantic_before.0.embedded_content.contains("[bludgeoning]")
                    || semantic_before.0.embedded_content.contains("[poison]")
            );
            assert!(
                semantic_before
                    .4
                    .iter()
                    .any(|unit| unit.input_text.contains("[bludgeoning]")
                        || unit.input_text.contains("[poison]"))
            );
            assert!(
                inputs
                    .iter()
                    .any(|(_, plain, _)| plain.contains("[bludgeoning]")
                        || plain.contains("[poison]")),
                "default text still has authored formula"
            );
        }
        Ok(())
    }

    fn retrieve_spell(
        retrieval: &AtlasRetrievalService,
        record_key: &str,
    ) -> Result<atlas_record::RetrievedRecord, Box<dyn std::error::Error>> {
        let record_key = RecordKey::parse(record_key)?;
        retrieval
            .get_record(GetRecordRequest {
                record_key: &record_key,
            })?
            .ok_or_else(|| format!("missing spell fixture {record_key}").into())
    }

    fn spell_surface_json(retrieved: &atlas_record::RetrievedRecord) -> serde_json::Value {
        spell_surface_json_with_selection(retrieved, None)
    }

    fn spell_surface_json_with_selection(
        retrieved: &atlas_record::RetrievedRecord,
        selection: Option<(atlas_record::SpellFormId, u8)>,
    ) -> serde_json::Value {
        let remaster_lookup =
            crate::retrieval::VerifiedRemasterLookup::from_test_result(RemasterLinksResult {
                seed: retrieved.clone(),
                links: Vec::new(),
            });
        serde_json::to_value(super::record_surface(
            retrieved,
            RecordSurfaceProfileView::RecordDetail,
            None,
            selection,
            &remaster_lookup,
        ))
        .expect("spell surface should serialize")
    }

    #[test]
    fn real_creature_editions_expose_unlinked_controls_and_both_air_pair_directions() {
        let night_hag = edition_record(NIGHT_HAG_KEY, "Night Hag", false);
        let giant_rat = edition_record(GIANT_RAT_KEY, "Giant Rat", true);
        let air_mephit = edition_record(AIR_MEPHIT_KEY, "Air Mephit", false);
        let air_scamp = edition_record(AIR_SCAMP_KEY, "Air Scamp", true);
        let air_link = remaster_link(&air_scamp, &air_mephit);

        let night_hag_edition =
            edition_for(&night_hag, None, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            night_hag_edition.status,
            RecordSurfaceEditionStatusView::Legacy
        );
        assert!(night_hag_edition.counterparts.is_empty());

        let giant_rat_edition =
            edition_for(&giant_rat, None, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            giant_rat_edition.status,
            RecordSurfaceEditionStatusView::Remaster
        );
        assert!(giant_rat_edition.counterparts.is_empty());

        let legacy_links = remaster_result(&air_mephit, vec![air_link.clone()]);
        let legacy_edition = edition_for(
            &air_mephit,
            Some(&legacy_links),
            RecordSurfaceProfileView::RecordDetail,
        );
        assert_eq!(
            legacy_edition.status,
            RecordSurfaceEditionStatusView::Legacy
        );
        assert_eq!(legacy_edition.counterparts.len(), 1);
        assert_eq!(
            legacy_edition.counterparts[0].role,
            RecordSurfaceEditionCounterpartRoleView::RemasteredCounterpart
        );
        assert_eq!(legacy_edition.counterparts[0].record_key, AIR_SCAMP_KEY);
        assert_eq!(legacy_edition.counterparts[0].title, "Air Scamp");

        let remaster_links = remaster_result(&air_scamp, vec![air_link]);
        let remaster_edition = edition_for(
            &air_scamp,
            Some(&remaster_links),
            RecordSurfaceProfileView::RecordDetail,
        );
        assert_eq!(
            remaster_edition.status,
            RecordSurfaceEditionStatusView::Remaster
        );
        assert_eq!(remaster_edition.counterparts.len(), 1);
        assert_eq!(
            remaster_edition.counterparts[0].role,
            RecordSurfaceEditionCounterpartRoleView::LegacyCounterpart
        );
        assert_eq!(remaster_edition.counterparts[0].record_key, AIR_MEPHIT_KEY);
        assert_eq!(remaster_edition.counterparts[0].title, "Air Mephit");
    }

    #[test]
    fn edition_metadata_is_identical_across_all_record_surface_profiles() {
        let air_mephit = edition_record(AIR_MEPHIT_KEY, "Air Mephit", false);
        let air_scamp = edition_record(AIR_SCAMP_KEY, "Air Scamp", true);
        let links = remaster_result(&air_mephit, vec![remaster_link(&air_scamp, &air_mephit)]);
        let expected = edition_for(
            &air_mephit,
            Some(&links),
            RecordSurfaceProfileView::RecordDetail,
        );

        for profile in [
            RecordSurfaceProfileView::SearchCompact,
            RecordSurfaceProfileView::RecordDetail,
            RecordSurfaceProfileView::EncounterParticipant,
        ] {
            assert_eq!(edition_for(&air_mephit, Some(&links), profile), expected);
        }
    }

    #[test]
    fn contradictory_edition_direction_drops_only_the_counterpart() {
        let air_mephit = edition_record(AIR_MEPHIT_KEY, "Air Mephit", false);
        let contradictory_target = edition_record(AIR_SCAMP_KEY, "Air Scamp", false);
        let links = remaster_result(
            &air_mephit,
            vec![remaster_link(&contradictory_target, &air_mephit)],
        );

        let edition = edition_for(
            &air_mephit,
            Some(&links),
            RecordSurfaceProfileView::RecordDetail,
        );
        assert_eq!(edition.status, RecordSurfaceEditionStatusView::Legacy);
        assert!(edition.counterparts.is_empty());
    }

    #[test]
    fn multiple_counterparts_are_deduplicated_and_sorted_by_exact_record_identity() {
        let legacy = edition_record("legacy-pack:seed", "Legacy Seed", false);
        let remaster_a = edition_record("remaster-pack:a", "First Remaster", true);
        let remaster_z = edition_record("remaster-pack:z", "Last Remaster", true);
        let link_a = remaster_link(&remaster_a, &legacy);
        let link_z = remaster_link(&remaster_z, &legacy);
        let links = remaster_result(&legacy, vec![link_z, link_a.clone(), link_a]);

        let edition = edition_for(
            &legacy,
            Some(&links),
            RecordSurfaceProfileView::RecordDetail,
        );
        assert_eq!(
            edition
                .counterparts
                .iter()
                .map(|counterpart| counterpart.record_key.as_str())
                .collect::<Vec<_>>(),
            vec!["remaster-pack:a", "remaster-pack:z"]
        );
    }

    fn edition_record(
        record_key: &str,
        title: &str,
        remaster: bool,
    ) -> atlas_record::RetrievedRecord {
        let mut record = atlas_record::AtlasRecord::new(
            atlas_record::RecordIdentity::new(
                RecordKey::parse(record_key).expect("edition fixture key"),
                title,
            ),
            atlas_record::RecordClassification::new(RecordKind::Creature),
            atlas_record::FoundryRecordInfo::new(
                "Edition Fixture",
                atlas_record::FoundryDocumentType::Actor,
                atlas_record::FoundryRecordType::Npc,
            ),
            atlas_record::RecordProvenance::new(format!("fixtures/{record_key}.json")),
        );
        record.publication.remaster = remaster;
        atlas_record::RetrievedRecord {
            record,
            body: None,
            spell_children: Vec::new(),
        }
    }

    fn remaster_link(
        remaster_record: &atlas_record::RetrievedRecord,
        legacy_record: &atlas_record::RetrievedRecord,
    ) -> RemasterLinkResult {
        RemasterLinkResult {
            remaster_record: remaster_record.clone(),
            legacy_record: legacy_record.clone(),
            source: RemasterLinkSource::RemasterJournal,
            source_ref: "journal:Bestiaries".to_string(),
        }
    }

    fn remaster_result(
        seed: &atlas_record::RetrievedRecord,
        links: Vec<RemasterLinkResult>,
    ) -> RemasterLinksResult {
        RemasterLinksResult {
            seed: seed.clone(),
            links,
        }
    }

    fn edition_for(
        record: &atlas_record::RetrievedRecord,
        remaster_links: Option<&RemasterLinksResult>,
        profile: RecordSurfaceProfileView,
    ) -> atlas_app_model::RecordSurfaceEditionView {
        let remaster_lookup = crate::retrieval::VerifiedRemasterLookup::from_test_result(
            remaster_links
                .cloned()
                .unwrap_or_else(|| remaster_result(record, Vec::new())),
        );
        super::record_surface(record, profile, None, None, &remaster_lookup)
            .metadata
            .edition
            .expect("canonical fixture record should expose edition metadata")
    }

    #[test]
    fn collection_projection_distinguishes_populated_from_known_empty() {
        assert_eq!(non_empty::<u8>(Vec::new()), None);
        assert_eq!(non_empty(vec![7_u8]), Some(vec![7_u8]));
    }

    #[test]
    fn known_empty_domains_omit_collections_without_reporting_failure() {
        let surface = creature_surface(
            &known_empty_creature(),
            RecordSurfaceProfileView::RecordDetail,
        );

        assert!(surface.skills.is_none());
        assert!(surface.movement.is_none());
        assert!(surface.resources.is_none());
        assert!(surface.spellcasting.is_none());
        assert!(surface.standalone_spells.is_none());
        assert!(surface.activities.is_none());
        assert!(surface.relationships.is_none());
        assert!(surface.unavailable_domains.is_none());
    }

    #[test]
    fn static_record_detail_retains_canonical_focus_provenance() {
        let mut night_hag = known_empty_creature();
        night_hag.identity.name = "Night Hag".to_string();
        night_hag.resources.value = FactValue::Value(vec![atlas_record::CreatureResource {
            id: atlas_record::CreatureComponentId::new("resource:focus")
                .expect("focus resource id"),
            authored_order: 0,
            kind: atlas_record::CreatureResourceKind::new("focus").expect("focus resource kind"),
            label: "Focus".to_string(),
            maximum: FactValue::Value(atlas_record::CreatureResourceAmount::Integer(1)),
            serialized_value: FactValue::Value(atlas_record::CreatureResourceAmount::Integer(1)),
            source_drift: FactValue::Missing,
            current_policy: atlas_record::ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
        }]);

        let surface = creature_surface(&night_hag, RecordSurfaceProfileView::RecordDetail);

        let resources = surface.resources.expect("static Focus resource");
        assert_eq!(resources.len(), 1);
        let focus = &resources[0];
        assert_eq!(focus.component_id, "resource:focus");
        assert_eq!(focus.authored_order, 0);
        assert_eq!(focus.kind, "focus");
        assert_eq!(focus.label, "Focus");
        assert_eq!(focus.maximum, Some(1));
    }

    #[test]
    fn giant_rat_search_compact_projects_only_the_approved_static_summary_and_teaser_limit() {
        let mut giant_rat = known_empty_creature();
        giant_rat.identity.name = "Giant Rat".to_string();
        set_hit_points(&mut giant_rat, 3, 8);

        let surface = creature_surface(&giant_rat, RecordSurfaceProfileView::SearchCompact);
        assert_eq!(
            surface.vitals.as_ref().and_then(|value| value.hit_points),
            Some(8)
        );
        assert_eq!(
            surface
                .defenses
                .as_ref()
                .and_then(|value| value.armor_class),
            Some(20)
        );
        let saves = surface.saves.expect("compact saves");
        assert_eq!(saves.fortitude.and_then(|value| value.modifier), Some(10));
        assert_eq!(saves.reflex.and_then(|value| value.modifier), Some(10));
        assert_eq!(saves.will.and_then(|value| value.modifier), Some(10));
        assert_eq!(
            surface
                .awareness
                .as_ref()
                .and_then(|value| value.perception),
            Some(10)
        );
        assert!(surface.size.is_none());
        assert!(surface.skills.is_none());
        assert!(surface.activities.is_none());
        assert!(surface.unavailable_domains.is_none());

        let text = (1..=55)
            .map(|index| format!("word{index}"))
            .collect::<Vec<_>>()
            .join(" \n");
        let teaser = teaser_from_text(&text).expect("non-empty teaser");
        assert_eq!(teaser.split_whitespace().count(), 50);
        assert!(teaser.ends_with("word50"));
        assert_eq!(teaser_from_text(" \n\t "), None);
    }

    #[test]
    fn night_hag_search_compact_wires_teaser_and_dense_scan_facts_through_record_surface() {
        let creature = spell_payload_fixture();
        let record_key = creature.identity.record_key.clone();
        let mut record = atlas_record::AtlasRecord::new(
            atlas_record::RecordIdentity::new(record_key, "Night Hag"),
            atlas_record::RecordClassification::new(RecordKind::Creature),
            atlas_record::FoundryRecordInfo::new(
                "Bestiary",
                atlas_record::FoundryDocumentType::Actor,
                atlas_record::FoundryRecordType::Npc,
            ),
            atlas_record::RecordProvenance::new("packs/creatures/night-hag.json"),
        );
        record
            .content
            .documents
            .push(atlas_record::RecordContentDocument {
                source_kind: ContentSourceKind::Description,
                label: Some("Description".to_string()),
                document: RichDocument::new(vec![RichNode::Text {
                    text: (1..=55)
                        .map(|index| format!("hag{index}"))
                        .collect::<Vec<_>>()
                        .join(" "),
                }]),
            });
        let retrieved = atlas_record::RetrievedRecord {
            record,
            body: Some(atlas_record::RecordBody::Creature(creature)),
            spell_children: Vec::new(),
        };

        let remaster_lookup =
            crate::retrieval::VerifiedRemasterLookup::from_test_result(RemasterLinksResult {
                seed: retrieved.clone(),
                links: Vec::new(),
            });
        let surface = super::record_surface(
            &retrieved,
            RecordSurfaceProfileView::SearchCompact,
            None,
            None,
            &remaster_lookup,
        );
        assert!(surface.references.is_none());
        let atlas_app_model::RecordSurfacePresentationView::Creature { body } =
            surface.presentation
        else {
            panic!("creature surface");
        };
        assert_eq!(
            body.teaser.as_deref(),
            Some(
                (1..=50)
                    .map(|index| format!("hag{index}"))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .as_str()
            )
        );
        assert_eq!(
            body.awareness.as_ref().and_then(|value| value.perception),
            Some(10)
        );
        assert_eq!(
            body.defenses.as_ref().and_then(|value| value.armor_class),
            Some(20)
        );
        assert_eq!(
            body.vitals.as_ref().and_then(|value| value.hit_points),
            Some(50)
        );
        assert!(body.saves.as_ref().is_some_and(|saves| {
            saves.fortitude.as_ref().and_then(|value| value.modifier) == Some(10)
                && saves.reflex.as_ref().and_then(|value| value.modifier) == Some(10)
                && saves.will.as_ref().and_then(|value| value.modifier) == Some(10)
        }));
        assert!(body.spellcasting.is_none());
    }

    #[test]
    fn calcifda_dense_detail_projects_the_minimum_typed_profile_in_authored_order() {
        let mut calcifda = known_empty_creature();
        calcifda.identity.name = "Calcifda".to_string();
        calcifda.size.value = FactValue::Value(atlas_record::CreatureSize::Large);
        calcifda.adjustment.value = FactValue::Value(atlas_record::CreatureAdjustment::Elite);
        calcifda.initiative.value = FactValue::Value(atlas_record::CreatureInitiative {
            statistic: FactValue::Value(atlas_record::CreatureInitiativeStatistic::Named(
                atlas_record::CreatureStatistic::new("stealth").expect("statistic"),
            )),
        });
        if let FactValue::Value(defenses) = &mut calcifda.defenses.value {
            defenses.shield = FactValue::Value(atlas_record::CreatureShield {
                armor_class_bonus: FactValue::Value(2),
                broken_threshold: FactValue::Value(10),
                hardness: FactValue::Value(5),
                maximum_hit_points: FactValue::Value(20),
                serialized_hit_points: FactValue::Value(17),
                current_policy:
                    atlas_record::ShieldCurrentPolicy::SerializedHitPointsAreProvenanceOnly,
            });
        }
        calcifda.skills.value = FactValue::Value(vec![atlas_record::CreatureSkill {
            id: atlas_record::CreatureComponentId::new("stealth").expect("skill id"),
            authored_order: 0,
            source_entries: vec![atlas_record::CreatureSkillSourceEntry {
                authored_key: "stealth".to_string(),
                modifier: FactValue::Value(21),
            }],
            kind: atlas_record::CreatureSkillKind::Stealth,
            label: "Stealth".to_string(),
            modifier: FactValue::Value(21),
            note: FactValue::Missing,
            variants: FactValue::Value(vec![atlas_record::CreatureSkillVariant {
                id: atlas_record::CreatureComponentId::new("stealth-in-stone").expect("variant id"),
                authored_order: 0,
                modifier: FactValue::Value(23),
                label: FactValue::Value("in stone".to_string()),
                predicate: FactValue::Value(vec![atlas_record::CreaturePredicate::Term(
                    atlas_record::PredicateTerm::new("terrain:stone").expect("predicate"),
                )]),
            }]),
            source_item_id: FactValue::Missing,
            unmodeled: FactValue::Missing,
        }]);

        let owner = calcifda.identity.record_key.clone();
        let mut action = action_occurrence(&owner, "eruption", "eruption", 3);
        action.source_identity.source_locators = vec![
            atlas_record::CreatureSourceLocator {
                source_path: format!("{owner}/items/eruption-secondary"),
                locator: atlas_record::StableSourceLocator::new("items/eruption-secondary")
                    .expect("secondary locator"),
                precedence: 1,
            },
            atlas_record::CreatureSourceLocator {
                source_path: format!("{owner}/items/eruption"),
                locator: atlas_record::StableSourceLocator::new("items/eruption")
                    .expect("primary locator"),
                precedence: 0,
            },
        ];
        action.context.uses = FactValue::Value(atlas_record::CreatureUseLimit {
            maximum: FactValue::Value(2),
            serialized_value: FactValue::Value(1),
        });
        let atlas_record::CreatureCapability::Action(action_capability) = &mut action.capability
        else {
            panic!("action fixture");
        };
        action_capability.category = FactValue::Value("offensive".to_string());
        action_capability.frequency = FactValue::Value(atlas_record::CreatureFrequency {
            maximum: FactValue::Value(1),
            period: FactValue::Value("day".to_string()),
            serialized_value: FactValue::Value(1),
        });
        action_capability.requirements = FactValue::Value("standing on stone".to_string());
        action_capability.cost = FactValue::Value("one ember".to_string());
        action_capability.self_effect = FactValue::Value("heated".to_string());
        action_capability.self_effect_label = FactValue::Value("Heated".to_string());

        let strike = occurrence(
            &owner,
            "claw",
            1,
            CreatureEntityFamily::Strike,
            CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::Strike(atlas_record::CreatureStrikeCapability {
                traits: FactValue::Value(vec!["agile".to_string()]),
                attack_effects: FactValue::Value(vec!["grab".to_string()]),
                rolls: Vec::new(),
                damage: FactValue::Value(Vec::new()),
                action_cost: atlas_record::CreatureActionCost::Actions(1),
                unsupported_notes: Vec::new(),
            }),
        );
        let mut equipment = occurrence(
            &owner,
            "gear-obsidian-key",
            5,
            CreatureEntityFamily::Equipment,
            CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::Equipment(
                atlas_record::CreatureEquipmentCapability {
                    traits: FactValue::Value(vec!["magical".to_string()]),
                    level: FactValue::Value(12),
                    usage: FactValue::Value("held-in-one-hand".to_string()),
                    quantity: FactValue::Value(1),
                    uses: FactValue::Value(atlas_record::CreatureUseLimit {
                        maximum: FactValue::Value(3),
                        serialized_value: FactValue::Value(2),
                    }),
                    unsupported_notes: Vec::new(),
                },
            ),
        );
        equipment.source_identity.nested_source_id = FactValue::Value(
            atlas_record::CreatureSourceId::new("source-gear-key").expect("source id"),
        );
        let mut lore = occurrence(
            &owner,
            "lore-volcanic",
            2,
            CreatureEntityFamily::Lore,
            CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::Lore(atlas_record::CreatureLoreCapability {
                modifier: FactValue::Value(19),
                unsupported_notes: Vec::new(),
            }),
        );
        lore.source_identity.nested_source_id = FactValue::Value(
            atlas_record::CreatureSourceId::new("source-lore-volcanic").expect("source id"),
        );
        calcifda.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![
                entity(&owner, "eruption", CreatureEntityFamily::Action),
                entity(&owner, "claw", CreatureEntityFamily::Strike),
                entity(&owner, "gear-obsidian-key", CreatureEntityFamily::Equipment),
                entity(&owner, "lore-volcanic", CreatureEntityFamily::Lore),
            ],
            occurrences: vec![equipment, action, lore, strike],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });

        let surface = creature_surface(&calcifda, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            surface.size.as_ref().map(|value| value.value),
            Some(atlas_app_model::CreatureSurfaceSizeValueView::Large)
        );
        assert_eq!(
            surface.adjustment.as_ref().map(|value| value.value),
            Some(atlas_app_model::CreatureSurfaceAdjustmentValueView::Elite)
        );
        assert_eq!(
            surface
                .initiative
                .as_ref()
                .map(|value| value.statistic.as_str()),
            Some("stealth")
        );
        let shield = surface
            .defenses
            .as_ref()
            .and_then(|value| value.shield.as_ref())
            .expect("shield");
        assert_eq!(
            (
                shield.armor_class_bonus,
                shield.broken_threshold,
                shield.hardness,
                shield.maximum_hit_points
            ),
            (Some(2), Some(10), Some(5), Some(20))
        );
        let skill = &surface.skills.as_ref().expect("skill")[0];
        assert_eq!(
            skill.source_entries.as_ref().unwrap()[0].authored_key,
            "stealth"
        );
        assert_eq!(
            skill.variants.as_ref().unwrap()[0].component_id,
            "stealth-in-stone"
        );
        assert_eq!(
            surface
                .activities
                .as_ref()
                .unwrap()
                .iter()
                .map(|value| value.occurrence_id.as_str())
                .collect::<Vec<_>>(),
            ["claw", "eruption"]
        );
        let strike = &surface.activities.as_ref().unwrap()[0];
        assert_eq!(
            strike.provenance.nested_source_id.as_deref(),
            Some("source-claw")
        );
        assert_eq!(
            strike.provenance.stable_source_locator.as_deref(),
            Some("items/claw")
        );
        assert_eq!(
            strike
                .provenance
                .source_locators
                .as_ref()
                .expect("source locator")[0]
                .locator,
            "items/claw"
        );
        assert_eq!(
            strike.attack_effects.as_deref(),
            Some(["grab".to_string()].as_slice())
        );
        let action = &surface.activities.as_ref().unwrap()[1];
        assert_eq!(
            action
                .provenance
                .source_locators
                .as_ref()
                .expect("ordered source locators")
                .iter()
                .map(|value| (value.locator.as_str(), value.precedence))
                .collect::<Vec<_>>(),
            [("items/eruption-secondary", 1), ("items/eruption", 0)]
        );
        assert_eq!(action.category.as_deref(), Some("offensive"));
        assert_eq!(
            action.frequency.as_ref().and_then(|value| value.maximum),
            Some(1)
        );
        assert_eq!(action.requirements.as_deref(), Some("standing on stone"));
        assert_eq!(action.cost.as_deref(), Some("one ember"));
        assert_eq!(
            action.uses.as_ref().and_then(|value| value.maximum),
            Some(2)
        );
        assert_eq!(
            action
                .self_effect
                .as_ref()
                .and_then(|value| value.value.as_deref()),
            Some("heated")
        );
        let equipment = &surface.equipment.as_ref().expect("equipment")[0];
        assert_eq!(equipment.occurrence_id, "gear-obsidian-key");
        assert_eq!(
            equipment.provenance.nested_source_id.as_deref(),
            Some("source-gear-key")
        );
        assert_eq!(
            equipment.provenance.stable_source_locator.as_deref(),
            Some("items/gear-obsidian-key")
        );
        assert_eq!(
            equipment.uses.as_ref().and_then(|value| value.maximum),
            Some(3)
        );
        let lore = &surface.lore.as_ref().expect("lore")[0];
        assert_eq!(
            (
                lore.occurrence_id.as_str(),
                lore.authored_order,
                lore.modifier
            ),
            ("lore-volcanic", 2, Some(19))
        );
        assert_eq!(
            lore.provenance.nested_source_id.as_deref(),
            Some("source-lore-volcanic")
        );
        assert_eq!(
            lore.provenance.stable_source_locator.as_deref(),
            Some("items/lore-volcanic")
        );
        assert!(surface.unavailable_domains.is_none());
    }

    #[test]
    fn beluthus_actor_ritual_dc_31_projects_as_a_named_optional_domain() {
        let mut beluthus = known_empty_creature();
        beluthus.identity.name = "Beluthus".to_string();
        let embedded = beluthus
            .embedded_entities
            .value
            .as_value()
            .expect("embedded")
            .clone();
        beluthus.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            actor_spellcasting: FactValue::Value(atlas_record::CreatureActorSpellcastingContext {
                rituals_dc: FactValue::Value(atlas_record::CreatureSourceScalar::Value(31)),
                unsupported_notes: Vec::new(),
            }),
            ..embedded
        });

        let surface = creature_surface(&beluthus, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            surface.rituals.as_ref().map(|value| value.difficulty_class),
            Some(31)
        );
        assert!(surface.unavailable_domains.is_none());
    }

    #[test]
    fn optional_missing_and_null_are_silent_while_populated_required_shield_failures_are_typed() {
        for optional_state in [
            CreatureSurfaceUnavailableStateView::Missing,
            CreatureSurfaceUnavailableStateView::Null,
        ] {
            let mut creature = known_empty_creature();
            creature.adjustment.value = non_value(optional_state);
            creature.initiative.value = FactValue::Value(atlas_record::CreatureInitiative {
                statistic: non_value(optional_state),
            });
            if let FactValue::Value(defenses) = &mut creature.defenses.value {
                defenses.shield = non_value(optional_state);
            }
            if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
                embedded.actor_spellcasting = non_value(optional_state);
            }

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert!(surface.adjustment.is_none());
            assert!(surface.initiative.is_none());
            assert!(
                surface
                    .defenses
                    .as_ref()
                    .is_some_and(|value| value.shield.is_none())
            );
            assert!(surface.rituals.is_none());
            assert!(surface.unavailable_domains.is_none());
        }

        let mut creature = known_empty_creature();
        if let FactValue::Value(defenses) = &mut creature.defenses.value {
            defenses.shield = FactValue::Value(atlas_record::CreatureShield {
                armor_class_bonus: FactValue::Missing,
                broken_threshold: FactValue::Null,
                hardness: FactValue::Missing,
                maximum_hit_points: FactValue::Null,
                serialized_hit_points: FactValue::Value(17),
                current_policy:
                    atlas_record::ShieldCurrentPolicy::SerializedHitPointsAreProvenanceOnly,
            });
        }
        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert!(
            surface
                .defenses
                .as_ref()
                .is_some_and(|value| value.shield.is_some())
        );
        assert_causes(
            &surface
                .unavailable_domains
                .expect("required shield causes")
                .defenses,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::ShieldArmorClassBonus,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::ShieldBrokenThreshold,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::ShieldHardness,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::ShieldMaximumHitPoints,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
            ],
        );
    }

    #[test]
    fn mixed_shield_and_frequency_children_preserve_supported_siblings_and_exact_causes() {
        // These canonical child types are FactValue<i64/String>; unlike source-scalar
        // fields, they do not admit a populated Unsupported variant.
        for (failed, expected_state) in [
            (
                FactValue::Missing,
                CreatureSurfaceUnavailableStateView::Missing,
            ),
            (FactValue::Null, CreatureSurfaceUnavailableStateView::Null),
        ] {
            let mut creature = known_empty_creature();
            if let FactValue::Value(defenses) = &mut creature.defenses.value {
                defenses.shield = FactValue::Value(atlas_record::CreatureShield {
                    armor_class_bonus: FactValue::Value(2),
                    broken_threshold: failed,
                    hardness: FactValue::Value(5),
                    maximum_hit_points: FactValue::Value(20),
                    serialized_hit_points: FactValue::Value(17),
                    current_policy:
                        atlas_record::ShieldCurrentPolicy::SerializedHitPointsAreProvenanceOnly,
                });
            }

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let shield = surface
                .defenses
                .as_ref()
                .and_then(|value| value.shield.as_ref())
                .expect("populated shield parent");
            assert_eq!(shield.armor_class_bonus, Some(2));
            assert_eq!(shield.broken_threshold, None);
            assert_eq!(shield.hardness, Some(5));
            assert_eq!(shield.maximum_hit_points, Some(20));
            assert_causes(
                &surface
                    .unavailable_domains
                    .as_ref()
                    .expect("one failed shield child")
                    .defenses,
                vec![cause(
                    expected_state,
                    CreatureSurfaceUnavailableFieldView::ShieldBrokenThreshold,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                )],
            );
            let payload = serde_json::to_value(&surface).expect("shield API payload");
            let shield = &payload["defenses"]["shield"];
            assert_eq!(shield["armor_class_bonus"], 2);
            assert!(shield.get("broken_threshold").is_none());
            assert_eq!(shield["hardness"], 5);
            assert_eq!(shield["maximum_hit_points"], 20);
            assert_eq!(
                payload["unavailable_domains"]["defenses"]["causes"]
                    .as_array()
                    .map(Vec::len),
                Some(1)
            );
        }

        for (maximum, period, failed_field, expected_state) in [
            (
                FactValue::Value(1),
                FactValue::Missing,
                CreatureSurfaceUnavailableFieldView::ActionFrequencyPeriod,
                CreatureSurfaceUnavailableStateView::Missing,
            ),
            (
                FactValue::Null,
                FactValue::Value("day".to_string()),
                CreatureSurfaceUnavailableFieldView::ActionFrequencyMaximum,
                CreatureSurfaceUnavailableStateView::Null,
            ),
        ] {
            let mut creature = known_empty_creature();
            let owner = creature.identity.record_key.clone();
            let mut action = action_occurrence(&owner, "mixed-frequency", "mixed-frequency", 0);
            let atlas_record::CreatureCapability::Action(capability) = &mut action.capability
            else {
                panic!("action fixture");
            };
            capability.frequency = FactValue::Value(atlas_record::CreatureFrequency {
                maximum,
                period,
                serialized_value: FactValue::Value(1),
            });
            creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
                entities: vec![entity(
                    &owner,
                    "mixed-frequency",
                    CreatureEntityFamily::Action,
                )],
                occurrences: vec![action],
                relationships: Vec::new(),
                actor_spellcasting: FactValue::Missing,
            });

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let frequency = surface.activities.as_ref().expect("action")[0]
                .frequency
                .as_ref()
                .expect("populated frequency parent");
            assert!(frequency.maximum.is_some() || frequency.period.is_some());
            assert_eq!(
                frequency.display.as_deref(),
                if failed_field == CreatureSurfaceUnavailableFieldView::ActionFrequencyPeriod {
                    Some("1")
                } else {
                    Some("day")
                }
            );
            assert_causes(
                &surface
                    .unavailable_domains
                    .as_ref()
                    .expect("one failed frequency child")
                    .activities,
                vec![cause(
                    expected_state,
                    failed_field,
                    Some("mixed-frequency"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            let payload = serde_json::to_value(&surface).expect("frequency API payload");
            let frequency = &payload["activities"][0]["frequency"];
            if failed_field == CreatureSurfaceUnavailableFieldView::ActionFrequencyPeriod {
                assert_eq!(frequency["maximum"], 1);
                assert!(frequency.get("period").is_none());
                assert_eq!(frequency["display"], "1");
            } else {
                assert!(frequency.get("maximum").is_none());
                assert_eq!(frequency["period"], "day");
                assert_eq!(frequency["display"], "day");
            }
            assert_eq!(
                payload["unavailable_domains"]["activities"]["causes"]
                    .as_array()
                    .map(Vec::len),
                Some(1)
            );
        }
    }

    #[test]
    fn frequency_display_retains_raw_period_and_rejects_unknown_display_text() {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let mut action = action_occurrence(&owner, "timed-action", "timed-action", 0);
        let atlas_record::CreatureCapability::Action(capability) = &mut action.capability else {
            panic!("action fixture");
        };
        capability.frequency = FactValue::Value(atlas_record::CreatureFrequency {
            maximum: FactValue::Value(1),
            period: FactValue::Value("PT1M".to_string()),
            serialized_value: FactValue::Value(1),
        });
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(&owner, "timed-action", CreatureEntityFamily::Action)],
            occurrences: vec![action],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });

        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let frequency = surface.activities.as_ref().expect("action")[0]
            .frequency
            .as_ref()
            .expect("frequency");
        assert_eq!(frequency.maximum, Some(1));
        assert_eq!(frequency.period.as_deref(), Some("PT1M"));
        assert_eq!(frequency.display.as_deref(), Some("1 per minute"));
        assert!(surface.unavailable_domains.is_none());
        let payload = serde_json::to_value(&surface).expect("frequency API payload");
        assert_eq!(payload["activities"][0]["frequency"]["maximum"], 1);
        assert_eq!(payload["activities"][0]["frequency"]["period"], "PT1M");
        assert_eq!(
            payload["activities"][0]["frequency"]["display"],
            "1 per minute"
        );

        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let mut action = action_occurrence(&owner, "unknown-period", "unknown-period", 0);
        let atlas_record::CreatureCapability::Action(capability) = &mut action.capability else {
            panic!("action fixture");
        };
        capability.frequency = FactValue::Value(atlas_record::CreatureFrequency {
            maximum: FactValue::Value(1),
            period: FactValue::Value("per-moon".to_string()),
            serialized_value: FactValue::Value(1),
        });
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(
                &owner,
                "unknown-period",
                CreatureEntityFamily::Action,
            )],
            occurrences: vec![action],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });

        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let frequency = surface.activities.as_ref().expect("action")[0]
            .frequency
            .as_ref()
            .expect("frequency");
        assert_eq!(frequency.maximum, Some(1));
        assert_eq!(frequency.period.as_deref(), Some("per-moon"));
        assert_eq!(frequency.display.as_deref(), Some("1"));
        assert_ne!(frequency.display.as_deref(), frequency.period.as_deref());
        assert_causes(
            &surface
                .unavailable_domains
                .as_ref()
                .expect("unsupported period")
                .activities,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Unsupported,
                CreatureSurfaceUnavailableFieldView::ActionFrequencyPeriod,
                Some("unknown-period"),
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
            )],
        );
        let payload = serde_json::to_value(&surface).expect("unknown frequency API payload");
        assert_eq!(payload["activities"][0]["frequency"]["period"], "per-moon");
        assert_eq!(payload["activities"][0]["frequency"]["display"], "1");
        assert_eq!(
            payload["unavailable_domains"]["activities"]["causes"][0]["state"],
            "unsupported"
        );
        assert_eq!(
            payload["unavailable_domains"]["activities"]["causes"][0]["field"],
            "action_frequency_period"
        );
        assert_eq!(
            payload["unavailable_domains"]["activities"]["causes"][0]["component_id"],
            "unknown-period"
        );
    }

    #[test]
    fn optional_iwr_absence_is_silent_while_populated_amounts_remain_required() {
        for values in [
            FactValue::Missing,
            FactValue::Null,
            FactValue::Value(Vec::new()),
        ] {
            let mut creature = known_empty_creature();
            let FactValue::Value(defenses) = &mut creature.defenses.value else {
                panic!("defenses");
            };
            defenses.immunities = values.clone();
            defenses.resistances = values.clone();
            defenses.weaknesses = values;

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert!(surface.unavailable_domains.is_none());
            let payload = serde_json::to_value(&surface).expect("optional IWR payload");
            let defenses = &payload["defenses"];
            assert!(defenses.get("immunities").is_none());
            assert!(defenses.get("resistances").is_none());
            assert!(defenses.get("weaknesses").is_none());
        }

        for qualifiers in [
            FactValue::Missing,
            FactValue::Null,
            FactValue::Value(Vec::new()),
        ] {
            let mut creature = known_empty_creature();
            let FactValue::Value(defenses) = &mut creature.defenses.value else {
                panic!("defenses");
            };
            defenses.resistances = FactValue::Value(vec![CreatureIwr {
                id: atlas_record::CreatureComponentId::new("fire").expect("iwr id"),
                authored_order: 0,
                kind: CreatureIwrKind::Resistance,
                iwr_type: atlas_record::IwrType::new("fire").expect("iwr type"),
                value: FactValue::Value(10),
                exceptions: qualifiers.clone(),
                double_vs: qualifiers,
                apply_once: FactValue::Missing,
            }]);

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let resistance = &surface.defenses.as_ref().expect("defenses").resistances[0];
            assert_eq!(resistance.amount, Some(10));
            assert!(resistance.exceptions.is_empty());
            assert!(resistance.double_vs.is_empty());
            assert!(surface.unavailable_domains.is_none());
            let payload = serde_json::to_value(&surface).expect("IWR qualifier payload");
            let resistance = &payload["defenses"]["resistances"][0];
            assert!(resistance.get("exceptions").is_none());
            assert!(resistance.get("double_vs").is_none());
        }

        let mut creature = known_empty_creature();
        let FactValue::Value(defenses) = &mut creature.defenses.value else {
            panic!("defenses");
        };
        let iwr = |id: &str, kind, value| CreatureIwr {
            id: atlas_record::CreatureComponentId::new(id).expect("iwr id"),
            authored_order: 0,
            kind,
            iwr_type: atlas_record::IwrType::new(id).expect("iwr type"),
            value,
            exceptions: FactValue::Value(Vec::new()),
            double_vs: FactValue::Value(Vec::new()),
            apply_once: FactValue::Missing,
        };
        defenses.resistances = FactValue::Value(vec![iwr(
            "fire",
            CreatureIwrKind::Resistance,
            FactValue::Missing,
        )]);
        defenses.weaknesses = FactValue::Value(vec![iwr(
            "cold",
            CreatureIwrKind::Weakness,
            FactValue::Null,
        )]);

        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let causes = &surface
            .unavailable_domains
            .as_ref()
            .expect("required IWR amounts")
            .defenses;
        assert_causes(
            causes,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::IwrAmount,
                    Some("fire"),
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::IwrAmount,
                    Some("cold"),
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
            ],
        );
        assert!(
            causes
                .as_ref()
                .expect("defense causes")
                .causes
                .iter()
                .all(|cause| cause.message == "A value is required here.")
        );
    }

    #[test]
    fn action_damage_absence_is_silent_while_strike_damage_remains_required() {
        for damage in [
            FactValue::Missing,
            FactValue::Null,
            FactValue::Value(Vec::new()),
        ] {
            let mut creature = known_empty_creature();
            let owner = creature.identity.record_key.clone();
            let mut action = action_occurrence(&owner, "action", "action", 0);
            let atlas_record::CreatureCapability::Action(capability) = &mut action.capability
            else {
                panic!("action fixture");
            };
            capability.damage = damage;
            creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
                entities: vec![entity(&owner, "action", CreatureEntityFamily::Action)],
                occurrences: vec![action],
                relationships: Vec::new(),
                actor_spellcasting: FactValue::Missing,
            });

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert!(
                surface.activities.as_ref().expect("action")[0]
                    .damage
                    .is_empty()
            );
            assert!(surface.unavailable_domains.is_none());
            let payload = serde_json::to_value(&surface).expect("optional action damage payload");
            assert!(payload["activities"][0].get("damage").is_none());
        }

        let supported_damage = atlas_record::CreatureDamage {
            id: "main".to_string(),
            formula: FactValue::Value("2d6".to_string()),
            damage_type: FactValue::Value("fire".to_string()),
            category: FactValue::Missing,
            kinds: FactValue::Value(Vec::new()),
            apply_modifier: FactValue::Missing,
        };
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let mut action = action_occurrence(&owner, "action", "action", 0);
        let atlas_record::CreatureCapability::Action(capability) = &mut action.capability else {
            panic!("action fixture");
        };
        capability.damage = FactValue::Value(vec![supported_damage.clone()]);
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(&owner, "action", CreatureEntityFamily::Action)],
            occurrences: vec![action],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });

        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let damage = &surface.activities.as_ref().expect("action")[0].damage;
        assert_eq!(damage.len(), 1);
        assert_eq!(damage[0].damage_id, "main");
        assert_eq!(damage[0].formula.as_deref(), Some("2d6"));
        assert_eq!(damage[0].damage_type.as_deref(), Some("fire"));
        assert!(surface.unavailable_domains.is_none());
        let payload = serde_json::to_value(&surface).expect("supported action damage payload");
        assert_eq!(
            payload["activities"][0]["damage"][0],
            serde_json::json!({
                "damage_id": "main",
                "formula": "2d6",
                "damage_type": "fire"
            })
        );

        for (formula, damage_type, state, state_name, field, field_name) in [
            (
                FactValue::Missing,
                FactValue::Value("fire".to_string()),
                CreatureSurfaceUnavailableStateView::Missing,
                "missing",
                CreatureSurfaceUnavailableFieldView::DamageFormula,
                "damage_formula",
            ),
            (
                FactValue::Null,
                FactValue::Value("fire".to_string()),
                CreatureSurfaceUnavailableStateView::Null,
                "null",
                CreatureSurfaceUnavailableFieldView::DamageFormula,
                "damage_formula",
            ),
            (
                FactValue::Value("2d6".to_string()),
                FactValue::Missing,
                CreatureSurfaceUnavailableStateView::Missing,
                "missing",
                CreatureSurfaceUnavailableFieldView::DamageType,
                "damage_type",
            ),
            (
                FactValue::Value("2d6".to_string()),
                FactValue::Null,
                CreatureSurfaceUnavailableStateView::Null,
                "null",
                CreatureSurfaceUnavailableFieldView::DamageType,
                "damage_type",
            ),
        ] {
            let mut creature = known_empty_creature();
            let owner = creature.identity.record_key.clone();
            let mut action = action_occurrence(&owner, "action", "action", 0);
            let atlas_record::CreatureCapability::Action(capability) = &mut action.capability
            else {
                panic!("action fixture");
            };
            capability.damage = FactValue::Value(vec![atlas_record::CreatureDamage {
                formula,
                damage_type,
                ..supported_damage.clone()
            }]);
            creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
                entities: vec![entity(&owner, "action", CreatureEntityFamily::Action)],
                occurrences: vec![action],
                relationships: Vec::new(),
                actor_spellcasting: FactValue::Missing,
            });

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let damage = &surface.activities.as_ref().expect("action")[0].damage;
            assert_eq!(damage.len(), 1);
            assert_eq!(damage[0].damage_id, "main");
            assert_eq!(
                damage[0].formula.as_deref(),
                (field != CreatureSurfaceUnavailableFieldView::DamageFormula).then_some("2d6")
            );
            assert_eq!(
                damage[0].damage_type.as_deref(),
                (field != CreatureSurfaceUnavailableFieldView::DamageType).then_some("fire")
            );
            assert_causes(
                &surface
                    .unavailable_domains
                    .as_ref()
                    .expect("required damage child")
                    .activities,
                vec![cause(
                    state,
                    field,
                    Some("action/main"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            let payload = serde_json::to_value(&surface).expect("partial action damage payload");
            assert_eq!(payload["activities"][0]["occurrence_id"], "action");
            assert_eq!(payload["activities"][0]["damage"][0]["damage_id"], "main");
            if field == CreatureSurfaceUnavailableFieldView::DamageFormula {
                assert!(
                    payload["activities"][0]["damage"][0]
                        .get("formula")
                        .is_none()
                );
                assert_eq!(payload["activities"][0]["damage"][0]["damage_type"], "fire");
            } else {
                assert_eq!(payload["activities"][0]["damage"][0]["formula"], "2d6");
                assert!(
                    payload["activities"][0]["damage"][0]
                        .get("damage_type")
                        .is_none()
                );
            }
            assert_eq!(
                payload["unavailable_domains"]["activities"]["causes"]
                    .as_array()
                    .expect("activity causes")
                    .len(),
                1
            );
            assert_eq!(
                payload["unavailable_domains"]["activities"]["causes"][0]["component_id"],
                "action/main"
            );
            assert_eq!(
                payload["unavailable_domains"]["activities"]["causes"][0]["state"],
                state_name
            );
            assert_eq!(
                payload["unavailable_domains"]["activities"]["causes"][0]["field"],
                field_name
            );
        }

        for (damage, state) in [
            (
                FactValue::Missing,
                CreatureSurfaceUnavailableStateView::Missing,
            ),
            (FactValue::Null, CreatureSurfaceUnavailableStateView::Null),
        ] {
            let mut creature = known_empty_creature();
            let owner = creature.identity.record_key.clone();
            let strike = occurrence(
                &owner,
                "strike",
                0,
                CreatureEntityFamily::Strike,
                atlas_record::CreatureOccurrenceParent::Creature,
                atlas_record::CreatureCapability::Strike(atlas_record::CreatureStrikeCapability {
                    traits: FactValue::Value(Vec::new()),
                    attack_effects: FactValue::Value(Vec::new()),
                    rolls: Vec::new(),
                    damage,
                    action_cost: atlas_record::CreatureActionCost::Actions(1),
                    unsupported_notes: Vec::new(),
                }),
            );
            creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
                entities: vec![entity(&owner, "strike", CreatureEntityFamily::Strike)],
                occurrences: vec![strike],
                relationships: Vec::new(),
                actor_spellcasting: FactValue::Missing,
            });

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let causes = &surface
                .unavailable_domains
                .as_ref()
                .expect("required strike damage")
                .activities;
            assert_causes(
                causes,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::ActivityDamage,
                    Some("strike"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            assert_eq!(
                causes.as_ref().expect("activity cause").causes[0].message,
                "A value is required here."
            );
        }
    }

    #[test]
    fn implicit_precise_special_vision_omits_missing_acuity_without_hiding_other_failures() {
        for sense_type in [
            "low-light-vision",
            "darkvision",
            "greater-darkvision",
            "see-invisibility",
        ] {
            let mut creature = known_empty_creature();
            creature.perception.value = FactValue::Value(CreaturePerception {
                modifier: FactValue::Value(10),
                details: FactValue::Missing,
                has_vision: FactValue::Value(true),
                senses: FactValue::Value(vec![CreatureSense {
                    id: atlas_record::CreatureComponentId::new(sense_type).expect("sense id"),
                    authored_order: 0,
                    sense_type: atlas_record::SenseType::new(sense_type).expect("sense type"),
                    acuity: FactValue::Missing,
                    range: FactValue::Missing,
                }]),
            });

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let senses = &surface.awareness.as_ref().expect("awareness").senses;
            assert_eq!(senses.len(), 1);
            assert_eq!(senses[0].component_id, sense_type);
            assert_eq!(senses[0].kind, sense_type);
            assert_eq!(senses[0].acuity, None);
            assert!(surface.unavailable_domains.is_none());
            let payload = serde_json::to_value(&surface).expect("special vision payload");
            assert_eq!(
                payload["awareness"]["senses"][0]["component_id"],
                sense_type
            );
            assert!(payload["awareness"]["senses"][0].get("acuity").is_none());
        }

        let mut creature = known_empty_creature();
        creature.perception.value = FactValue::Value(CreaturePerception {
            modifier: FactValue::Value(10),
            details: FactValue::Missing,
            has_vision: FactValue::Value(true),
            senses: FactValue::Value(vec![CreatureSense {
                id: atlas_record::CreatureComponentId::new("scent").expect("sense id"),
                authored_order: 0,
                sense_type: atlas_record::SenseType::new("scent").expect("sense type"),
                acuity: FactValue::Value(atlas_record::SenseAcuity::Imprecise),
                range: FactValue::Value(30),
            }]),
        });
        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let scent = &surface.awareness.as_ref().expect("awareness").senses[0];
        assert_eq!(scent.component_id, "scent");
        assert_eq!(scent.acuity.as_deref(), Some("imprecise"));
        assert!(surface.unavailable_domains.is_none());

        for (sense_type, acuity, state) in [
            (
                "scent",
                FactValue::Missing,
                CreatureSurfaceUnavailableStateView::Missing,
            ),
            (
                "scent",
                FactValue::Null,
                CreatureSurfaceUnavailableStateView::Null,
            ),
            (
                "darkvision",
                FactValue::Null,
                CreatureSurfaceUnavailableStateView::Null,
            ),
        ] {
            let mut creature = known_empty_creature();
            creature.perception.value = FactValue::Value(CreaturePerception {
                modifier: FactValue::Value(10),
                details: FactValue::Missing,
                has_vision: FactValue::Value(true),
                senses: FactValue::Value(vec![CreatureSense {
                    id: atlas_record::CreatureComponentId::new(sense_type).expect("sense id"),
                    authored_order: 0,
                    sense_type: atlas_record::SenseType::new(sense_type).expect("sense type"),
                    acuity,
                    range: FactValue::Missing,
                }]),
            });
            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert_causes(
                &surface
                    .unavailable_domains
                    .as_ref()
                    .expect("expected acuity failure")
                    .awareness,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::SenseAcuity,
                    Some(sense_type),
                    CreatureSurfaceSourceFieldView::Perception,
                )],
            );
        }
    }

    #[test]
    fn magical_forge_static_profiles_project_maximum_hp_and_equal_hp_guard() {
        let mut magical_forge = known_empty_creature();
        magical_forge.identity.name = "Magical Forge".to_string();
        set_hit_points(&mut magical_forge, 115, 135);

        for profile in [
            RecordSurfaceProfileView::SearchCompact,
            RecordSurfaceProfileView::RecordDetail,
        ] {
            let surface = creature_surface(&magical_forge, profile);
            let vitals = surface.vitals.expect("static vitals");
            assert_eq!(vitals.hit_points, Some(135), "profile {profile:?}");
            let payload = serde_json::to_value(&vitals).expect("static vitals payload");
            assert_eq!(
                payload.get("hit_points").and_then(|value| value.as_i64()),
                Some(135)
            );
            assert!(payload.get("current_hp").is_none());
            assert!(payload.get("maximum").is_none());
            assert!(payload.get("value").is_none());
        }

        set_hit_points(&mut magical_forge, 50, 50);
        assert_eq!(
            creature_surface(&magical_forge, RecordSurfaceProfileView::RecordDetail)
                .vitals
                .expect("equal HP vitals")
                .hit_points,
            Some(50)
        );
    }

    #[test]
    fn encounter_profile_omits_static_hp_and_preserves_runtime_vitals() {
        let mut magical_forge = known_empty_creature();
        magical_forge.identity.name = "Magical Forge".to_string();
        set_hit_points(&mut magical_forge, 115, 135);

        let mut runtime = empty_runtime();
        runtime.vitals = Some(EncounterRuntimeVitalsView {
            maximum_hp: Some(RuntimeNumberView {
                label: "Maximum HP".to_string(),
                base_value: 135,
                adjusted_value: 135,
                modifiers: Vec::new(),
                suppressed_modifiers: Vec::new(),
                provenance: RuntimeFactProvenanceView {
                    source: RuntimeFactSourceView::CanonicalRecord,
                    canonical_target: None,
                },
            }),
            current_hp: Some(115),
            temporary_hp: 7,
        });
        let expected_runtime_vitals = runtime.vitals.clone();

        compose_encounter_payload(
            &magical_forge,
            &activity_content_placement(&magical_forge),
            &mut runtime,
        );

        assert_eq!(runtime.vitals, expected_runtime_vitals);
        let surface = creature_surface(
            &magical_forge,
            RecordSurfaceProfileView::EncounterParticipant,
        );
        assert!(surface.vitals.is_none());
        let payload = serde_json::to_value(&surface).expect("encounter creature payload");
        assert!(payload.get("vitals").is_none());
        let runtime_payload = serde_json::to_value(&runtime).expect("runtime payload");
        assert_eq!(
            runtime_payload
                .pointer("/vitals/maximum_hp/adjusted_value")
                .and_then(|value| value.as_i64()),
            Some(135)
        );
        assert_eq!(
            runtime_payload
                .pointer("/vitals/current_hp")
                .and_then(|value| value.as_i64()),
            Some(115)
        );
    }

    #[test]
    fn gray_master_alias_and_exact_unmodeled_skill_evidence_project_without_inference() {
        let mut creature = known_empty_creature();
        let hostile_key = "<img src=x onerror=alert(1)> ../../etc/passwd\nskill";
        creature.legacy_abilities.value = FactValue::Value(CreatureLegacyAbilities {
            strength: FactValue::Value(5),
            dexterity: FactValue::Value(4),
            constitution: FactValue::Value(6),
            intelligence: FactValue::Value(4),
            wisdom: FactValue::Value(5),
            charisma: FactValue::Value(3),
        });
        creature.skills.value = FactValue::Value(vec![
            atlas_record::CreatureSkill {
                id: atlas_record::CreatureComponentId::new("intimidation").expect("skill id"),
                authored_order: 0,
                source_entries: vec![atlas_record::CreatureSkillSourceEntry {
                    authored_key: "intimidate".to_string(),
                    modifier: FactValue::Value(38),
                }],
                kind: atlas_record::CreatureSkillKind::Intimidation,
                label: "Intimidation".to_string(),
                modifier: FactValue::Value(38),
                note: FactValue::Missing,
                variants: FactValue::Missing,
                source_item_id: FactValue::Missing,
                unmodeled: FactValue::Missing,
            },
            atlas_record::CreatureSkill {
                id: atlas_record::CreatureComponentId::new("alpha-unmodeled").expect("skill id"),
                authored_order: 2,
                source_entries: vec![atlas_record::CreatureSkillSourceEntry {
                    authored_key: "acrobatics+13".to_string(),
                    modifier: FactValue::Null,
                }],
                kind: atlas_record::CreatureSkillKind::Unmodeled,
                label: "acrobatics+13".to_string(),
                modifier: FactValue::Null,
                note: FactValue::Missing,
                variants: FactValue::Missing,
                source_item_id: FactValue::Missing,
                unmodeled: FactValue::Value(atlas_record::CreatureUnmodeledSkill {
                    authored_key: "acrobatics+13".to_string(),
                    base: FactValue::Null,
                    reason: atlas_record::CreatureUnmodeledSkillReason::UnknownAuthoredKey,
                }),
            },
            atlas_record::CreatureSkill {
                id: atlas_record::CreatureComponentId::new("zeta-hostile").expect("skill id"),
                authored_order: 1,
                source_entries: vec![
                    atlas_record::CreatureSkillSourceEntry {
                        authored_key: hostile_key.to_string(),
                        modifier: FactValue::Null,
                    },
                    atlas_record::CreatureSkillSourceEntry {
                        authored_key: "second exact member".to_string(),
                        modifier: FactValue::Value(17),
                    },
                ],
                kind: atlas_record::CreatureSkillKind::Unmodeled,
                label: hostile_key.to_string(),
                modifier: FactValue::Null,
                note: FactValue::Missing,
                variants: FactValue::Missing,
                source_item_id: FactValue::Value(
                    atlas_record::CreatureSourceId::new("source-hostile-skill").expect("source id"),
                ),
                unmodeled: FactValue::Value(atlas_record::CreatureUnmodeledSkill {
                    authored_key: hostile_key.to_string(),
                    base: FactValue::Null,
                    reason: atlas_record::CreatureUnmodeledSkillReason::UnknownAuthoredKey,
                }),
            },
        ]);

        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let abilities = surface.abilities.as_ref().expect("canonical abilities");
        assert_eq!(
            [
                abilities.strength,
                abilities.dexterity,
                abilities.constitution,
                abilities.intelligence,
                abilities.wisdom,
                abilities.charisma,
            ],
            [Some(5), Some(4), Some(6), Some(4), Some(5), Some(3)]
        );
        let skills = surface.skills.as_ref().expect("modeled skill");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].kind, "intimidation");
        assert_eq!(skills[0].modifier, Some(38));
        assert_eq!(skills[0].source_entries.as_ref().map(Vec::len), Some(1));
        assert_eq!(
            skills[0].source_entries.as_ref().unwrap()[0].authored_key,
            "intimidate"
        );
        let unmodeled = surface
            .unmodeled_skills
            .as_ref()
            .expect("ordered unmodeled skill facts");
        assert_eq!(
            unmodeled
                .iter()
                .map(|value| (value.component_id.as_str(), value.authored_order))
                .collect::<Vec<_>>(),
            [("zeta-hostile", 1), ("alpha-unmodeled", 2)]
        );
        assert_eq!(unmodeled[0].authored_key, hostile_key);
        assert_eq!(
            unmodeled[0].source_item_id.as_deref(),
            Some("source-hostile-skill")
        );
        assert_eq!(
            unmodeled[0]
                .source_entries
                .as_ref()
                .expect("source members")
                .iter()
                .map(|entry| (entry.authored_order, entry.authored_key.as_str()))
                .collect::<Vec<_>>(),
            [(0, hostile_key), (1, "second exact member")]
        );
        let payload = serde_json::to_value(&surface).expect("unmodeled API payload");
        assert_eq!(payload["unmodeled_skills"][0]["authored_key"], hostile_key);
        assert_eq!(
            payload["unmodeled_skills"][0]["source_entries"][1]["authored_key"],
            "second exact member"
        );
        assert!(payload["unmodeled_skills"][0].get("source_path").is_none());
        assert!(payload["unmodeled_skills"][0].get("raw_json").is_none());

        let causes = &surface
            .unavailable_domains
            .expect("unmodeled authored key is populated unsupported evidence")
            .skills
            .expect("skills availability domain")
            .causes;
        assert_eq!(causes.len(), 2);
        assert!(causes.iter().all(|cause| {
            cause.state == CreatureSurfaceUnavailableStateView::Unsupported
                && cause.field == CreatureSurfaceUnavailableFieldView::UnmodeledSkill
                && cause.message == "The source supplied an unrecognized skill key."
        }));
        assert_eq!(
            causes
                .iter()
                .map(|cause| cause.component_id.as_deref())
                .collect::<Vec<_>>(),
            [Some("alpha-unmodeled"), Some("zeta-hostile")],
            "cause sort is diagnostic and does not own authored ordering"
        );
        let detail = causes
            .iter()
            .find(|cause| cause.component_id.as_deref() == Some("alpha-unmodeled"))
            .expect("acrobatics cause")
            .unmodeled_skill
            .as_ref()
            .expect("typed unmodeled skill detail");
        assert_eq!(detail.authored_key, "acrobatics+13");
        assert!(matches!(
            detail.base,
            atlas_app_model::CreatureSurfaceIntegerPresenceView::Null
        ));
        assert_eq!(detail.authored_order, 2);
        assert_eq!(detail.source_entries.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn encounter_profile_retains_only_non_runtime_canonical_context() {
        let mut creature = activity_content_fixture();
        let mut defenses = creature
            .defenses
            .value
            .as_value()
            .expect("defenses")
            .clone();
        defenses.armor_class = FactValue::Value(CreatureArmorClass {
            value: FactValue::Value(25),
            details: FactValue::Value(CreatureNote::new("against spells")),
        });
        defenses.hardness = FactValue::Value(5);
        defenses.all_saves_note = FactValue::Value(CreatureNote::new("+1 status vs. magic"));
        let mut saves = defenses.saves.as_value().expect("saves").clone();
        let mut fortitude = saves.fortitude.as_value().expect("fortitude").clone();
        fortitude.details = FactValue::Value(CreatureNote::new("+2 vs. disease"));
        saves.fortitude = FactValue::Value(fortitude);
        defenses.saves = FactValue::Value(saves);
        let iwr = |id: &str, authored_order, kind, iwr_type: &str, value| CreatureIwr {
            id: atlas_record::CreatureComponentId::new(id).expect("iwr id"),
            authored_order,
            kind,
            iwr_type: atlas_record::IwrType::new(iwr_type).expect("iwr type"),
            value,
            exceptions: FactValue::Value(Vec::new()),
            double_vs: FactValue::Value(Vec::new()),
            apply_once: FactValue::Missing,
        };
        defenses.immunities = FactValue::Value(vec![iwr(
            "sleep",
            0,
            CreatureIwrKind::Immunity,
            "sleep",
            FactValue::Missing,
        )]);
        defenses.resistances = FactValue::Value(vec![iwr(
            "mental",
            0,
            CreatureIwrKind::Resistance,
            "mental",
            FactValue::Value(10),
        )]);
        defenses.weaknesses = FactValue::Value(vec![iwr(
            "cold-iron",
            0,
            CreatureIwrKind::Weakness,
            "cold-iron",
            FactValue::Value(10),
        )]);
        creature.defenses.value = FactValue::Value(defenses);
        creature.perception.value = FactValue::Value(CreaturePerception {
            modifier: FactValue::Value(18),
            details: FactValue::Value(CreatureNote::new("keen awareness")),
            has_vision: FactValue::Value(true),
            senses: FactValue::Value(vec![CreatureSense {
                id: atlas_record::CreatureComponentId::new("darkvision").expect("sense id"),
                authored_order: 0,
                sense_type: atlas_record::SenseType::new("darkvision").expect("sense type"),
                acuity: FactValue::Value(atlas_record::SenseAcuity::Precise),
                range: FactValue::Missing,
            }]),
        });
        creature.languages.value = FactValue::Value(CreatureLanguages {
            values: FactValue::Value(
                ["common", "aklo", "infernal", "jotun", "necril"]
                    .into_iter()
                    .map(|value| atlas_record::Language::new(value).expect("language"))
                    .collect(),
            ),
            details: FactValue::Value(CreatureNote::new("telepathy 100 feet")),
        });
        let embedded = creature
            .embedded_entities
            .value
            .as_value()
            .expect("embedded entities");
        let mut embedded = embedded.clone();
        let atlas_record::CreatureCapability::Action(capability) =
            &mut embedded.occurrences[0].capability
        else {
            panic!("fixture activity should be an action");
        };
        capability.traits = FactValue::Value(vec!["disease".to_string(), "divine".to_string()]);
        creature.embedded_entities.value = FactValue::Value(embedded);

        let placement = activity_content_placement(&creature);
        let mut runtime = empty_runtime();
        runtime.activities.push(runtime_activity(
            "activity",
            EncounterRuntimeActivityKindView::Other,
        ));
        compose_encounter_payload(&creature, &placement, &mut runtime);
        let surface = creature_surface(&creature, RecordSurfaceProfileView::EncounterParticipant);

        let defenses = surface.defenses.expect("encounter defense context");
        assert_eq!(defenses.armor_class, None);
        assert_eq!(
            defenses.armor_class_details.as_deref(),
            Some("against spells")
        );
        assert_eq!(defenses.hardness, Some(5));
        assert_eq!(defenses.immunities[0].kind, "sleep");
        assert_eq!(defenses.resistances[0].amount, Some(10));
        assert_eq!(defenses.weaknesses[0].amount, Some(10));
        let saves = surface.saves.expect("encounter save context");
        assert_eq!(saves.all_saves_note.as_deref(), Some("+1 status vs. magic"));
        assert!(saves.fortitude.as_ref().is_some_and(|save| {
            save.modifier.is_none() && save.details.as_deref() == Some("+2 vs. disease")
        }));
        let awareness = surface.awareness.expect("encounter awareness context");
        assert_eq!(awareness.perception, None);
        assert_eq!(awareness.senses[0].kind, "darkvision");
        assert_eq!(awareness.languages.len(), 5);
        assert_eq!(
            awareness.language_details.as_deref(),
            Some("telepathy 100 feet")
        );
        assert!(surface.activities.is_none());
        assert_eq!(runtime.activities[0].traits, ["disease", "divine"]);
        assert!(surface.unavailable_domains.is_none());
    }

    #[test]
    fn encounter_activity_traits_keep_typed_unavailable_state_without_static_rows() {
        let mut creature = activity_content_fixture();
        let embedded = creature
            .embedded_entities
            .value
            .as_value()
            .expect("embedded entities");
        let mut embedded = embedded.clone();
        let atlas_record::CreatureCapability::Action(capability) =
            &mut embedded.occurrences[0].capability
        else {
            panic!("fixture activity should be an action");
        };
        capability.traits = FactValue::Missing;
        creature.embedded_entities.value = FactValue::Value(embedded);

        let surface = creature_surface(&creature, RecordSurfaceProfileView::EncounterParticipant);
        assert!(surface.activities.is_none());
        assert_causes(
            &surface
                .unavailable_domains
                .expect("missing activity traits should stay typed")
                .activities,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Missing,
                CreatureSurfaceUnavailableFieldView::ActivityTraits,
                Some("activity"),
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
            )],
        );
    }

    #[test]
    fn missing_and_null_roots_report_each_affected_named_domain() {
        for state in [
            CreatureSurfaceUnavailableStateView::Missing,
            CreatureSurfaceUnavailableStateView::Null,
        ] {
            let mut creature = known_empty_creature();
            creature.defenses.value = non_value(state);
            creature.perception.value = non_value(state);
            creature.languages.value = non_value(state);
            creature.skills.value = non_value(state);
            creature.legacy_abilities.value = non_value(state);
            creature.movement.value = non_value(state);
            creature.resources.value = non_value(state);
            creature.embedded_entities.value = non_value(state);

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let unavailable = surface
                .unavailable_domains
                .expect("non-value roots must be public typed failures");
            assert_causes(
                &unavailable.vitals,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Defenses,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                )],
            );
            assert_causes(
                &unavailable.defenses,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Defenses,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                )],
            );
            assert_causes(
                &unavailable.saves,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Defenses,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                )],
            );
            assert_causes(
                &unavailable.awareness,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Perception,
                    None,
                    CreatureSurfaceSourceFieldView::Perception,
                )],
            );
            assert_causes(
                &unavailable.abilities,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::LegacyAbilities,
                    None,
                    CreatureSurfaceSourceFieldView::LegacyAbilities,
                )],
            );
            assert_causes(
                &unavailable.skills,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Skills,
                    None,
                    CreatureSurfaceSourceFieldView::Skills,
                )],
            );
            assert_causes(
                &unavailable.movement,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Movement,
                    None,
                    CreatureSurfaceSourceFieldView::Movement,
                )],
            );
            assert_causes(
                &unavailable.resources,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Resources,
                    None,
                    CreatureSurfaceSourceFieldView::Resources,
                )],
            );
            assert_causes(
                &unavailable.equipment,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Equipment,
                    None,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            assert_causes(
                &unavailable.lore,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Lore,
                    None,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            assert_causes(
                &unavailable.spellcasting,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
                    None,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            assert_causes(
                &unavailable.activities,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
                    None,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            assert!(unavailable.relationships.is_none());
            assert!(surface.relationships.is_none());

            let outer = record_surface_for_creature(creature);
            let issues = outer.issues.expect("required creature issues");
            assert_eq!(
                issues
                    .iter()
                    .filter(|issue| {
                        issue.placement
                            == atlas_app_model::RecordSurfaceIssuePlacementView::Defenses
                            && issue.message == "A value is required here."
                    })
                    .count(),
                1,
                "the shared owner must collapse one defenses fact reused by three domains"
            );
            assert!(issues.iter().all(|issue| {
                !issue.message.contains("surface-fixture") && !issue.message.contains('/')
            }));
        }
    }

    fn record_surface_for_creature(
        creature: atlas_record::CreatureRecord,
    ) -> atlas_app_model::RecordSurfaceView {
        let fixture = crate::test_support::encounter_fixture_worker();
        let mut record = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");
        record.record.identity.key = creature.identity.record_key.clone();
        record.record.identity.name = creature.identity.name.clone();
        record.record.classification.kind = RecordKind::Creature;
        record.body = Some(RecordBody::Creature(creature));
        let remaster_lookup =
            crate::retrieval::VerifiedRemasterLookup::from_test_result(RemasterLinksResult {
                seed: record.clone(),
                links: Vec::new(),
            });
        super::record_surface(
            &record,
            RecordSurfaceProfileView::RecordDetail,
            None,
            None,
            &remaster_lookup,
        )
    }

    #[test]
    fn unavailable_perception_still_projects_or_reports_independent_languages() {
        let cases = [
            (
                CreatureSurfaceUnavailableStateView::Missing,
                FactValue::Value(CreatureLanguages {
                    values: FactValue::Value(vec![
                        atlas_record::Language::new("common").expect("language"),
                    ]),
                    details: FactValue::Missing,
                }),
                vec![cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::Perception,
                    None,
                    CreatureSurfaceSourceFieldView::Perception,
                )],
                vec!["common".to_string()],
            ),
            (
                CreatureSurfaceUnavailableStateView::Null,
                FactValue::Missing,
                vec![cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::Perception,
                    None,
                    CreatureSurfaceSourceFieldView::Perception,
                )],
                Vec::new(),
            ),
            (
                CreatureSurfaceUnavailableStateView::Missing,
                FactValue::Null,
                vec![cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::Perception,
                    None,
                    CreatureSurfaceSourceFieldView::Perception,
                )],
                Vec::new(),
            ),
        ];

        for (perception_state, languages, expected_causes, expected_languages) in cases {
            let mut creature = known_empty_creature();
            creature.perception.value = non_value(perception_state);
            creature.languages.value = languages;
            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert_eq!(
                surface
                    .awareness
                    .as_ref()
                    .expect("awareness should preserve independent language payload")
                    .languages,
                expected_languages
            );
            assert_causes(
                &surface
                    .unavailable_domains
                    .expect("perception failure should remain typed")
                    .awareness,
                expected_causes,
            );
        }
    }

    #[test]
    fn populated_and_nested_unsupported_values_are_distinct_and_order_independent() {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let defenses = creature.defenses.value.as_value().expect("defenses");
        let mut defenses = defenses.clone();
        defenses.hit_points.as_value().expect("hp");
        defenses.hit_points = FactValue::Value(CreatureHitPoints {
            value: FactValue::Value(CreatureNumber::Unsupported(unsupported_value())),
            maximum: FactValue::Null,
            temporary: FactValue::Missing,
            temporary_maximum: FactValue::Missing,
            details: FactValue::Missing,
        });
        defenses.saves.as_value().expect("saves");
        let mut saves = defenses.saves.as_value().expect("saves").clone();
        saves.fortitude.as_value().expect("fortitude");
        let mut fortitude = saves.fortitude.as_value().expect("fortitude").clone();
        fortitude.value = FactValue::Null;
        saves.fortitude = FactValue::Value(fortitude);
        defenses.saves = FactValue::Value(saves);
        defenses.resistances = FactValue::Value(vec![atlas_record::CreatureIwr {
            id: atlas_record::CreatureComponentId::new("fire").expect("iwr id"),
            authored_order: 0,
            kind: atlas_record::CreatureIwrKind::Resistance,
            iwr_type: atlas_record::IwrType::new("fire").expect("iwr type"),
            value: FactValue::Missing,
            exceptions: FactValue::Null,
            double_vs: FactValue::Missing,
            apply_once: FactValue::Missing,
        }]);
        creature.defenses.value = FactValue::Value(defenses);

        let perception = creature.perception.value.as_value().expect("perception");
        creature.perception.value = FactValue::Value(CreaturePerception {
            modifier: FactValue::Value(10),
            details: FactValue::Missing,
            has_vision: FactValue::Value(true),
            senses: FactValue::Value(vec![atlas_record::CreatureSense {
                id: atlas_record::CreatureComponentId::new("odd-sense").expect("sense id"),
                authored_order: 0,
                sense_type: atlas_record::SenseType::new("odd-sense").expect("sense type"),
                acuity: FactValue::Value(atlas_record::SenseAcuity::Unsupported(
                    unsupported_value(),
                )),
                range: perception
                    .senses
                    .as_value()
                    .and(Some(30))
                    .map_or(FactValue::Missing, FactValue::Value),
            }]),
        });
        creature.languages.value = FactValue::Value(CreatureLanguages {
            values: FactValue::Value(vec![
                atlas_record::Language::new("common").expect("language"),
            ]),
            details: FactValue::Missing,
        });
        creature.skills.value = FactValue::Value(vec![atlas_record::CreatureSkill {
            id: atlas_record::CreatureComponentId::new("athletics").expect("skill id"),
            authored_order: 0,
            source_entries: vec![atlas_record::CreatureSkillSourceEntry {
                authored_key: "athletics".to_string(),
                modifier: FactValue::Missing,
            }],
            kind: atlas_record::CreatureSkillKind::Athletics,
            label: "Athletics".to_string(),
            modifier: FactValue::Missing,
            note: FactValue::Missing,
            variants: FactValue::Value(Vec::new()),
            source_item_id: FactValue::Missing,
            unmodeled: FactValue::Missing,
        }]);
        let mut abilities = creature
            .legacy_abilities
            .value
            .as_value()
            .expect("abilities")
            .clone();
        abilities.strength = FactValue::Null;
        creature.legacy_abilities.value = FactValue::Value(abilities);

        let movement = |id: &str, order, value| atlas_record::CreatureSpeed {
            id: atlas_record::CreatureComponentId::new(id).expect("speed id"),
            authored_order: order,
            mode: atlas_record::CreatureMovementMode::Unsupported(unsupported_value()),
            value,
            label: FactValue::Value(id.to_string()),
            details: FactValue::Missing,
        };
        creature.movement.value = FactValue::Value(vec![
            movement("zeta", 1, FactValue::Missing),
            movement("alpha", 0, FactValue::Null),
        ]);
        creature.resources.value = FactValue::Value(vec![atlas_record::CreatureResource {
            id: atlas_record::CreatureComponentId::new("focus").expect("resource id"),
            authored_order: 0,
            kind: atlas_record::CreatureResourceKind::new("focus").expect("resource kind"),
            label: "Focus".to_string(),
            maximum: FactValue::Value(atlas_record::CreatureResourceAmount::Unsupported(
                unsupported_value(),
            )),
            serialized_value: FactValue::Missing,
            source_drift: FactValue::Value(Vec::new()),
            current_policy: atlas_record::ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
        }]);

        let entry_id = atlas_record::CreatureOccurrenceId::new("entry").expect("entry id");
        let strike = occurrence(
            &owner,
            "strike",
            0,
            atlas_record::CreatureEntityFamily::Strike,
            atlas_record::CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::Strike(atlas_record::CreatureStrikeCapability {
                traits: FactValue::Missing,
                attack_effects: FactValue::Value(Vec::new()),
                rolls: vec![atlas_record::CreatureRoll {
                    id: "attack".to_string(),
                    label: "Attack".to_string(),
                    kind: atlas_record::CreatureRollKind::Attack,
                    value: FactValue::Null,
                    ability: FactValue::Missing,
                }],
                damage: FactValue::Value(vec![atlas_record::CreatureDamage {
                    id: "main".to_string(),
                    formula: FactValue::Missing,
                    damage_type: FactValue::Null,
                    category: FactValue::Missing,
                    kinds: FactValue::Value(Vec::new()),
                    apply_modifier: FactValue::Missing,
                }]),
                action_cost: atlas_record::CreatureActionCost::Unsupported(unsupported_value()),
                unsupported_notes: Vec::new(),
            }),
        );
        let entry = occurrence(
            &owner,
            "entry",
            1,
            atlas_record::CreatureEntityFamily::SpellcastingEntry,
            atlas_record::CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::SpellcastingEntry(
                atlas_record::CreatureSpellcastingEntryCapability {
                    preparation: FactValue::Value(
                        atlas_record::CreatureSpellPreparation::Unsupported(unsupported_value()),
                    ),
                    tradition: FactValue::Missing,
                    attack: FactValue::Null,
                    dc: FactValue::Missing,
                    slots: FactValue::Value(Vec::new()),
                    unsupported_notes: Vec::new(),
                },
            ),
        );
        let mut spell = occurrence(
            &owner,
            "spell",
            2,
            atlas_record::CreatureEntityFamily::Spell,
            atlas_record::CreatureOccurrenceParent::SpellcastingEntry(entry_id),
            atlas_record::CreatureCapability::Spell(atlas_record::CreatureSpellCapability {
                traits: FactValue::Missing,
                base_rank: FactValue::Value(9),
                signature: FactValue::Missing,
                traditions: FactValue::Missing,
                requirements: FactValue::Missing,
                cost: FactValue::Missing,
                counteraction: FactValue::Missing,
                ritual: FactValue::Missing,
                target: FactValue::Missing,
                area: FactValue::Missing,
                range: FactValue::Missing,
                time: FactValue::Missing,
                duration: FactValue::Missing,
                defense: FactValue::Missing,
                damage: FactValue::Value(Vec::new()),
                action_cost: atlas_record::CreatureActionCost::Actions(1),
                unsupported_notes: Vec::new(),
            }),
        );
        spell.context.rank = FactValue::Null;
        spell.target = CreatureEntityTarget::CanonicalRecord(
            RecordKey::parse("spells:unsupported-fixture").expect("spell key"),
        );
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(&owner, "strike", CreatureEntityFamily::Strike)],
            occurrences: vec![strike, entry, spell],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });

        let forward = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let unavailable = forward
            .unavailable_domains
            .as_ref()
            .expect("nested failures");
        assert!(
            forward
                .defenses
                .as_ref()
                .is_some_and(|value| value.resistances.len() == 1)
        );
        assert!(
            forward
                .awareness
                .as_ref()
                .is_some_and(|value| value.languages == ["common"])
        );
        assert_causes(
            &unavailable.vitals,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Null,
                CreatureSurfaceUnavailableFieldView::HitPoints,
                None,
                CreatureSurfaceSourceFieldView::Defenses,
            )],
        );
        assert_causes(
            &unavailable.defenses,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Missing,
                CreatureSurfaceUnavailableFieldView::IwrAmount,
                Some("fire"),
                CreatureSurfaceSourceFieldView::Defenses,
            )],
        );
        assert_causes(
            &unavailable.saves,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Null,
                CreatureSurfaceUnavailableFieldView::Saves,
                Some("fortitude"),
                CreatureSurfaceSourceFieldView::Defenses,
            )],
        );
        assert_causes(
            &unavailable.awareness,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Unsupported,
                CreatureSurfaceUnavailableFieldView::SenseAcuity,
                Some("odd-sense"),
                CreatureSurfaceSourceFieldView::Perception,
            )],
        );
        assert_causes(
            &unavailable.abilities,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Null,
                CreatureSurfaceUnavailableFieldView::LegacyAbilities,
                Some("strength"),
                CreatureSurfaceSourceFieldView::LegacyAbilities,
            )],
        );
        assert_causes(
            &unavailable.skills,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Missing,
                CreatureSurfaceUnavailableFieldView::SkillModifier,
                Some("athletics"),
                CreatureSurfaceSourceFieldView::Skills,
            )],
        );
        assert_causes(
            &unavailable.movement,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::MovementSpeed,
                    Some("alpha"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::MovementMode,
                    Some("alpha"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::MovementSpeed,
                    Some("zeta"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::MovementMode,
                    Some("zeta"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
            ],
        );
        assert_causes(
            &unavailable.resources,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Unsupported,
                CreatureSurfaceUnavailableFieldView::ResourceMaximum,
                Some("focus"),
                CreatureSurfaceSourceFieldView::Resources,
            )],
        );
        assert_causes(
            &unavailable.spellcasting,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::SpellPreparation,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::SpellTradition,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::SpellAttack,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::SpellDifficultyClass,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::SpellTraits,
                    Some("spell"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::SpellRank,
                    Some("spell"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
            ],
        );
        assert_causes(
            &unavailable.activities,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::ActivityTraits,
                    Some("strike"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityActionCost,
                    Some("strike"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::ActivityRoll,
                    Some("strike/attack"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::DamageFormula,
                    Some("strike/main"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::DamageType,
                    Some("strike/main"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
            ],
        );
        assert_eq!(
            unavailable
                .vitals
                .as_ref()
                .expect("required null cause")
                .causes[0]
                .message,
            "A value is required here."
        );
        assert_eq!(
            unavailable
                .awareness
                .as_ref()
                .expect("unsupported cause")
                .causes[0]
                .message,
            "The authored value is not supported."
        );

        creature.movement.value.as_value().expect("movement");
        if let FactValue::Value(values) = &mut creature.movement.value {
            values.reverse();
        }
        if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
            embedded.occurrences.reverse();
        }
        let reverse = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            forward.unavailable_domains, reverse.unavailable_domains,
            "complete typed cause tuples must be order independent"
        );
    }

    #[test]
    fn activity_content_uses_typed_targets_once_and_preserves_blocks_order_and_provenance() {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let entity_id = "entity-plague";
        let occurrence_id = "occurrence-plague";
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(&owner, entity_id, CreatureEntityFamily::Action)],
            occurrences: vec![action_occurrence(&owner, occurrence_id, entity_id, 3)],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });
        creature.content.documents = vec![
            content_document(
                &owner,
                "public-notes",
                ContentOwner::Record(owner.clone()),
                0,
                "Public Notes",
                vec![
                    paragraph(vec![RichNode::Text {
                        text: "First paragraph.".to_string(),
                    }]),
                    RichNode::HtmlElement {
                        tag: "hr".to_string(),
                        attributes: BTreeMap::new(),
                        children: Vec::new(),
                    },
                    paragraph(vec![RichNode::Text {
                        text: "After divider.".to_string(),
                    }]),
                ],
            ),
            content_document(
                &owner,
                "item:plague:occurrence-note",
                ContentOwner::CreatureOccurrence(
                    atlas_record::CreatureOccurrenceId::new(occurrence_id).expect("occurrence id"),
                ),
                1,
                "Occurrence Note",
                vec![paragraph(vec![RichNode::Text {
                    text: "Occurrence-owned.".to_string(),
                }])],
            ),
            content_document(
                &owner,
                "item:plague:description",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new(entity_id).expect("entity id"),
                ),
                2,
                "Abyssal Plague",
                vec![paragraph(vec![
                    RichNode::Text {
                        text: "Saving Throw ".to_string(),
                    },
                    RichNode::Foundry {
                        node: FoundryNode::Check {
                            statistic: Some("fortitude".to_string()),
                            options: BTreeMap::from([("dc".to_string(), "28".to_string())]),
                            label: None,
                        },
                    },
                ])],
            ),
            content_document(
                &owner,
                "item:equipment:description",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new("unrelated-equipment").expect("entity id"),
                ),
                4,
                "Unrelated Equipment",
                vec![paragraph(vec![RichNode::Text {
                    text: "Stays general.".to_string(),
                }])],
            ),
        ];

        let forward = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let activity = &forward.activities.as_ref().expect("activity")[0];
        let attached = activity.content.as_ref().expect("attached content");
        assert_eq!(
            attached
                .iter()
                .map(|content| content.content_key.as_str())
                .collect::<Vec<_>>(),
            ["item:plague:occurrence-note", "item:plague:description"]
        );
        assert_eq!(attached[1].authored_order, 2);
        assert_eq!(attached[1].provenance.source_record_key, owner.to_string());
        assert!(content_contains_text(
            &attached[1].blocks,
            "Fortitude DC 28"
        ));
        assert!(matches!(
            &attached[1].blocks[0],
            CreatureSurfaceContentBlockView::Paragraph { spans }
                if matches!(
                    spans.as_slice(),
                    [
                        CreatureSurfaceContentInlineView::Text { text },
                        CreatureSurfaceContentInlineView::Check {
                            display,
                            statistic: Some(statistic),
                            difficulty_class: Some(28),
                        },
                    ] if text == "Saving Throw "
                        && display == "Fortitude DC 28"
                        && statistic == "fortitude"
                )
        ));
        let general = forward.content.as_ref().expect("general content");
        assert_eq!(
            general
                .iter()
                .map(|content| content.content_key.as_str())
                .collect::<Vec<_>>(),
            ["public-notes", "item:equipment:description"]
        );
        assert!(matches!(
            general[0].blocks.get(1),
            Some(CreatureSurfaceContentBlockView::Divider)
        ));

        creature.content.documents.reverse();
        let reverse = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            forward, reverse,
            "authored order must not depend on input order"
        );
    }

    #[test]
    fn activity_content_association_failures_omit_activity_and_never_fall_back_to_general() {
        let base = activity_content_fixture();
        let cases = [
            ("missing-target", {
                let mut creature = base.clone();
                creature
                    .embedded_entities
                    .value
                    .as_value()
                    .expect("embedded");
                if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
                    embedded.entities.clear();
                }
                creature
            }),
            ("duplicate-entity", {
                let mut creature = base.clone();
                if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
                    embedded.entities.push(embedded.entities[0].clone());
                }
                creature
            }),
            ("duplicate-occurrence", {
                let mut creature = base.clone();
                if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
                    let other_id =
                        atlas_record::CreatureEntityId::new("other-entity").expect("other entity");
                    let mut other_entity = embedded.entities[0].clone();
                    other_entity.id = other_id.clone();
                    embedded.entities.push(other_entity);
                    let mut duplicate = embedded.occurrences[0].clone();
                    duplicate.target = CreatureEntityTarget::ActorOwned(other_id);
                    duplicate.authored_order += 1;
                    embedded.occurrences.push(duplicate);
                }
                creature
            }),
            ("duplicate-content", {
                let mut creature = base.clone();
                creature
                    .content
                    .documents
                    .push(creature.content.documents[0].clone());
                creature
            }),
        ];

        for (case, creature) in cases {
            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert!(surface.activities.is_none(), "{case} must fail closed");
            assert!(
                surface.content.is_none(),
                "{case} must not fall back to lore"
            );
            assert_causes(
                &surface
                    .unavailable_domains
                    .expect("failure should remain typed")
                    .activities,
                vec![cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    Some("activity"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
        }
    }

    #[test]
    fn one_entity_document_matching_multiple_activities_fails_both_activities_closed() {
        let mut creature = activity_content_fixture();
        let owner = creature.identity.record_key.clone();
        if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
            embedded
                .occurrences
                .push(action_occurrence(&owner, "activity-two", "entity", 1));
        }

        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert!(surface.activities.is_none());
        assert!(surface.content.is_none());
        assert_causes(
            &surface
                .unavailable_domains
                .expect("multi-match should remain typed")
                .activities,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    Some("activity"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    Some("activity-two"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
            ],
        );
    }

    #[test]
    fn encounter_activity_content_is_nested_once_and_removed_from_general_content() {
        let creature = activity_content_fixture();
        let placement = activity_content_placement(&creature);
        let mut runtime = empty_runtime();
        runtime.activities.push(runtime_activity(
            "activity",
            EncounterRuntimeActivityKindView::Other,
        ));

        compose_encounter_payload(&creature, &placement, &mut runtime);

        assert_eq!(runtime.activities.len(), 1);
        assert!(
            runtime.activities[0]
                .content
                .as_ref()
                .is_some_and(|content| {
                    content.len() == 1
                        && content_contains_text(&content[0].blocks, "Activity content.")
                })
        );
        assert!(super::content(&creature, &placement).is_none());
        assert!(runtime.automation_limitations.is_empty());
    }

    #[test]
    fn shared_placement_preserves_pre_extraction_app_content_semantics() {
        let mut creature = activity_content_fixture();
        let owner = creature.identity.record_key.clone();
        creature.content.documents.push(content_document(
            &owner,
            "public-notes",
            ContentOwner::Record(owner.clone()),
            1,
            "Public Notes",
            vec![paragraph(vec![RichNode::Text {
                text: "General content.".to_string(),
            }])],
        ));
        let resolved = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            resolved.activities.as_ref().expect("activity")[0]
                .content
                .as_ref()
                .expect("attached content")[0]
                .content_key,
            "item:activity:description"
        );
        assert_eq!(
            resolved.content.as_ref().expect("general content")[0].content_key,
            "public-notes"
        );

        for state in ["missing", "null"] {
            let mut unavailable = creature.clone();
            unavailable.embedded_entities.value = if state == "missing" {
                FactValue::Missing
            } else {
                FactValue::Null
            };
            let surface = creature_surface(&unavailable, RecordSurfaceProfileView::RecordDetail);
            assert!(surface.activities.is_none());
            assert_eq!(
                surface.content.as_ref().expect("record-owned content")[0].content_key,
                "public-notes"
            );
            assert_eq!(surface.content.as_ref().expect("content").len(), 1);
        }
    }

    #[test]
    fn night_hag_static_spells_attach_typed_content_once_and_preserve_authored_occurrences() {
        let creature = spell_payload_fixture();
        let forward = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let spellcasting = forward.spellcasting.as_ref().expect("spellcasting");
        assert_eq!(spellcasting.len(), 2);
        assert_eq!(spellcasting[0].label, "Occult Innate Spells");
        assert_eq!(spellcasting[1].label, "Coven Spells");
        assert_eq!(
            spellcasting[0].provenance.nested_source_id.as_deref(),
            Some("source-entry-occult")
        );
        assert_eq!(
            spellcasting[0].provenance.stable_source_locator.as_deref(),
            Some("items/entry-occult")
        );
        assert_eq!(
            spellcasting[0]
                .slots
                .as_ref()
                .expect("slot maxima")
                .iter()
                .map(|slot| (slot.rank, slot.maximum))
                .collect::<Vec<_>>(),
            [(3, Some(4))]
        );
        let spells = spellcasting
            .iter()
            .flat_map(|entry| entry.spells.iter())
            .collect::<Vec<_>>();
        assert_eq!(spells.len(), 26);

        let magic_missile = spells
            .iter()
            .find(|spell| spell.occurrence_id == "magic-missile")
            .expect("Magic Missile occurrence");
        assert_eq!(magic_missile.rank, Some(3));
        assert_eq!(
            magic_missile.provenance.nested_source_id.as_deref(),
            Some("source-magic-missile")
        );
        assert_eq!(
            magic_missile.provenance.stable_source_locator.as_deref(),
            Some("items/magic-missile")
        );
        assert_eq!(spell_base_rank(&creature, "magic-missile"), Some(1));
        let magic_missile_payload =
            serde_json::to_value(magic_missile).expect("static Magic Missile payload");
        assert_eq!(
            magic_missile_payload
                .get("rank")
                .and_then(|value| value.as_i64()),
            Some(3)
        );
        assert!(magic_missile_payload.get("base_rank").is_none());
        let context = magic_missile.context.as_ref().expect("occurrence context");
        assert_eq!(context.group.as_deref(), Some("at-will"));
        assert_eq!(context.location.as_deref(), Some("innate"));
        assert_eq!(context.slot.as_deref(), Some("3"));
        assert_eq!(context.uses.as_ref().and_then(|uses| uses.maximum), Some(3));

        let bind_soul = spells
            .iter()
            .find(|spell| spell.occurrence_id == "bind-soul")
            .expect("Bind Soul");
        assert!(bind_soul.target_record_key.is_none());
        assert_eq!(
            bind_soul
                .content
                .as_ref()
                .expect("Bind Soul content")
                .iter()
                .map(|content| content.content_key.as_str())
                .collect::<Vec<_>>(),
            ["bind-soul-first", "bind-soul-second"]
        );
        let dream_council = spells
            .iter()
            .find(|spell| spell.occurrence_id == "dream-council")
            .expect("Dream Council");
        assert!(dream_council.target_record_key.is_none());
        assert!(dream_council.content.is_some());

        for (label, expected_occurrences, expected_rank, expected_base_rank) in [
            ("Nightmare", ["nightmare-1", "nightmare-2"], 5, 4),
            (
                "Dream Message",
                ["dream-message-1", "dream-message-2"],
                3,
                3,
            ),
        ] {
            let repeated = spells
                .iter()
                .filter(|spell| spell.label == label)
                .copied()
                .collect::<Vec<_>>();
            assert_eq!(repeated.len(), 2);
            assert_eq!(
                repeated
                    .iter()
                    .map(|spell| spell.occurrence_id.as_str())
                    .collect::<Vec<_>>(),
                expected_occurrences
            );
            assert!(
                repeated
                    .iter()
                    .all(|spell| spell.rank == Some(expected_rank))
            );
            assert!(expected_occurrences.iter().all(|occurrence_id| {
                spell_base_rank(&creature, occurrence_id) == Some(expected_base_rank)
            }));
            assert_eq!(repeated[0].target_record_key, repeated[1].target_record_key);
            assert_ne!(repeated[0].content, repeated[1].content);
        }

        let standalone = forward
            .standalone_spells
            .as_ref()
            .expect("standalone spells");
        assert_eq!(standalone.len(), 1);
        assert_eq!(standalone[0].label, "Control Weather");
        assert_eq!(standalone[0].rank, Some(6));
        assert!(standalone[0].content.is_some());
        let general = forward
            .content
            .as_ref()
            .expect("general Heartstone content");
        assert_eq!(general.len(), 1);
        assert_eq!(general[0].label.as_deref(), Some("Heartstone"));

        let mut reversed_creature = creature.clone();
        if let FactValue::Value(embedded) = &mut reversed_creature.embedded_entities.value {
            embedded.occurrences.reverse();
        }
        reversed_creature.content.documents.reverse();
        let reversed = creature_surface(&reversed_creature, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(forward, reversed, "typed authored order must be stable");
    }

    #[test]
    fn encounter_spells_group_by_typed_parent_preserve_standalone_and_ignore_input_order() {
        let creature = spell_payload_fixture();
        let placement = activity_content_placement(&creature);
        let runtime = spell_payload_runtime();

        let mut forward = runtime.clone();
        compose_encounter_payload(&creature, &placement, &mut forward);

        let mut reversed_creature = creature.clone();
        if let FactValue::Value(embedded) = &mut reversed_creature.embedded_entities.value {
            embedded.occurrences.reverse();
        }
        let reversed_placement = activity_content_placement(&reversed_creature);
        let mut reversed = runtime;
        reversed.activities.reverse();
        reversed.spellcasting.reverse();
        compose_encounter_payload(&reversed_creature, &reversed_placement, &mut reversed);

        assert_eq!(forward, reversed);
        assert_eq!(forward.spellcasting.len(), 2);
        assert_eq!(forward.spellcasting[0].label, "Occult Innate Spells");
        assert_eq!(
            forward.spellcasting[0].preparation.as_deref(),
            Some("innate")
        );
        assert_eq!(forward.spellcasting[0].tradition.as_deref(), Some("occult"));
        assert_eq!(
            forward
                .spellcasting
                .iter()
                .map(|entry| entry.spells.len())
                .sum::<usize>(),
            26
        );
        let spells = forward
            .spellcasting
            .iter()
            .flat_map(|entry| entry.spells.iter())
            .collect::<Vec<_>>();
        let magic_missile = spells
            .iter()
            .find(|spell| spell.occurrence_id == "magic-missile")
            .expect("runtime Magic Missile occurrence");
        assert_eq!(magic_missile.rank, Some(3));
        let magic_missile_payload =
            serde_json::to_value(magic_missile).expect("runtime Magic Missile payload");
        assert_eq!(
            magic_missile_payload
                .get("rank")
                .and_then(|value| value.as_i64()),
            Some(3)
        );
        assert!(magic_missile_payload.get("base_rank").is_none());
        for (label, expected_occurrences, expected_rank) in [
            ("Nightmare", ["nightmare-1", "nightmare-2"], 5),
            ("Dream Message", ["dream-message-1", "dream-message-2"], 3),
        ] {
            let repeated = spells
                .iter()
                .filter(|spell| spell.label == label)
                .copied()
                .collect::<Vec<_>>();
            assert_eq!(
                repeated
                    .iter()
                    .map(|spell| spell.occurrence_id.as_str())
                    .collect::<Vec<_>>(),
                expected_occurrences
            );
            assert!(
                repeated
                    .iter()
                    .all(|spell| spell.rank == Some(expected_rank))
            );
        }
        for id in [
            "bind-soul",
            "dream-council",
            "nightmare-1",
            "nightmare-2",
            "dream-message-1",
            "dream-message-2",
        ] {
            assert!(
                spells
                    .iter()
                    .any(|spell| spell.occurrence_id == id && spell.content.is_some()),
                "{id} should retain its own typed content"
            );
        }
        assert_eq!(forward.standalone_spells.len(), 1);
        assert_eq!(forward.standalone_spells[0].label, "Control Weather");
        assert_eq!(forward.standalone_spells[0].rank, Some(6));
        assert!(forward.standalone_spells[0].content.is_some());
        assert!(
            forward
                .activities
                .iter()
                .all(|activity| activity.kind != EncounterRuntimeActivityKindView::Spell)
        );
        let general = super::content(&creature, &placement).expect("Heartstone content");
        assert_eq!(general.len(), 1);
        assert_eq!(general[0].label.as_deref(), Some("Heartstone"));
        assert!(forward.automation_limitations.is_empty());
    }

    #[test]
    fn encounter_spell_grouping_fails_only_affected_rows_closed() {
        let mut creature = spell_payload_fixture();
        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("embedded entities should be present");
        };
        let duplicate = embedded
            .occurrences
            .iter()
            .find(|occurrence| occurrence.id.as_str() == "bind-soul")
            .expect("bind-soul")
            .clone();
        embedded.occurrences.push(duplicate);
        let dangling = spell_occurrence(
            &creature.identity.record_key,
            "dangling-spell",
            "Dangling Spell",
            9,
            CreatureOccurrenceParent::SpellcastingEntry(
                atlas_record::CreatureOccurrenceId::new("missing-entry").expect("entry id"),
            ),
            "spells:dangling-spell",
        );
        embedded.occurrences.push(dangling);
        let placement = activity_content_placement(&creature);
        let mut runtime = spell_payload_runtime();
        runtime.activities.push(runtime_activity(
            "bind-soul",
            EncounterRuntimeActivityKindView::Spell,
        ));

        compose_encounter_payload(&creature, &placement, &mut runtime);

        assert_eq!(
            runtime
                .spellcasting
                .iter()
                .map(|entry| entry.spells.len())
                .sum::<usize>(),
            25
        );
        assert!(runtime.spellcasting.iter().all(|entry| {
            entry
                .spells
                .iter()
                .all(|spell| spell.occurrence_id != "bind-soul")
        }));
        assert_eq!(runtime.standalone_spells.len(), 1);
        assert!(runtime.automation_limitations.iter().any(|limitation| {
            limitation.code
                == EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable
        }));
        assert!(
            runtime
                .activities
                .iter()
                .all(|activity| activity.kind != EncounterRuntimeActivityKindView::Spell)
        );
    }

    #[test]
    fn spell_content_association_ambiguity_fails_only_affected_rows_closed() {
        let base = spell_payload_fixture();
        let mut cases = Vec::new();

        let mut missing_target = base.clone();
        if let FactValue::Value(embedded) = &mut missing_target.embedded_entities.value {
            embedded
                .entities
                .retain(|entity| entity.id.as_str() != "bind-soul-entity");
        }
        cases.push(("missing-target", missing_target));

        let mut duplicate_target = base.clone();
        if let FactValue::Value(embedded) = &mut duplicate_target.embedded_entities.value {
            let duplicate = embedded
                .entities
                .iter()
                .find(|entity| entity.id.as_str() == "bind-soul-entity")
                .expect("Bind Soul entity")
                .clone();
            embedded.entities.push(duplicate);
        }
        cases.push(("duplicate-target", duplicate_target));

        let mut duplicate_occurrence = base.clone();
        if let FactValue::Value(embedded) = &mut duplicate_occurrence.embedded_entities.value {
            let duplicate = embedded
                .occurrences
                .iter()
                .find(|occurrence| occurrence.id.as_str() == "bind-soul")
                .expect("Bind Soul occurrence")
                .clone();
            embedded.occurrences.push(duplicate);
        }
        cases.push(("duplicate-occurrence", duplicate_occurrence));

        let mut duplicate_content = base.clone();
        let duplicate = duplicate_content
            .content
            .documents
            .iter()
            .find(|document| document.id.content_key.as_str() == "bind-soul-first")
            .expect("Bind Soul content")
            .clone();
        duplicate_content.content.documents.push(duplicate);
        cases.push(("duplicate-content", duplicate_content));

        let mut multiple_matches = base;
        if let FactValue::Value(embedded) = &mut multiple_matches.embedded_entities.value {
            let mut shadow = spell_occurrence(
                &multiple_matches.identity.record_key,
                "bind-soul-shadow",
                "Bind Soul",
                30,
                CreatureOccurrenceParent::SpellcastingEntry(
                    atlas_record::CreatureOccurrenceId::new("entry-coven").expect("entry id"),
                ),
                "spells:bind-soul",
            );
            shadow.target = CreatureEntityTarget::ActorOwned(
                atlas_record::CreatureEntityId::new("bind-soul-entity").expect("entity id"),
            );
            embedded.occurrences.push(shadow);
        }
        cases.push(("multiple-matches", multiple_matches));

        for (case, creature) in cases {
            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let spells = surface
                .spellcasting
                .as_ref()
                .expect("unaffected spellcasting entries")
                .iter()
                .flat_map(|entry| entry.spells.iter())
                .collect::<Vec<_>>();
            assert!(
                spells
                    .iter()
                    .all(|spell| !spell.occurrence_id.starts_with("bind-soul")),
                "{case} must omit only ambiguously associated Bind Soul rows"
            );
            assert!(
                spells
                    .iter()
                    .any(|spell| spell.occurrence_id == "dream-council"),
                "{case} must retain unambiguous rows"
            );
            assert_eq!(
                surface.standalone_spells.as_ref().expect("Control Weather")[0].label,
                "Control Weather"
            );
            let general = surface.content.as_ref().expect("Heartstone content");
            assert_eq!(general.len(), 1, "{case} must not leak spell lore");
            assert_eq!(general[0].label.as_deref(), Some("Heartstone"));
            let causes = &surface
                .unavailable_domains
                .as_ref()
                .expect("typed unavailability")
                .spellcasting
                .as_ref()
                .expect("spellcasting cause")
                .causes;
            assert!(causes.iter().any(|cause| {
                cause.field == CreatureSurfaceUnavailableFieldView::SpellContent
                    && cause
                        .component_id
                        .as_deref()
                        .is_some_and(|id| id.starts_with("bind-soul"))
            }));

            let placement = activity_content_placement(&creature);
            let mut runtime = spell_payload_runtime();
            compose_encounter_payload(&creature, &placement, &mut runtime);
            assert!(runtime.spellcasting.iter().all(|entry| {
                entry
                    .spells
                    .iter()
                    .all(|spell| !spell.occurrence_id.starts_with("bind-soul"))
            }));
            assert_eq!(runtime.standalone_spells[0].label, "Control Weather");
        }
    }

    fn empty_runtime() -> EncounterRuntimeView {
        EncounterRuntimeView {
            hazard: None,
            level: None,
            vitals: None,
            defenses: None,
            saves: None,
            awareness: None,
            abilities: None,
            skills: Vec::new(),
            movement: None,
            resources: Vec::new(),
            spellcasting: Vec::new(),
            standalone_spells: Vec::new(),
            activities: Vec::new(),
            action_budget: None,
            conditions: Vec::new(),
            automation_limitations: Vec::new(),
        }
    }

    fn runtime_activity(
        activity_id: &str,
        kind: EncounterRuntimeActivityKindView,
    ) -> EncounterRuntimeActivityView {
        EncounterRuntimeActivityView {
            activity_id: activity_id.to_string(),
            label: activity_id.to_string(),
            kind,
            usage: EncounterRuntimeActivityUsageView::Unlimited,
            availability: None,
            traits: Vec::new(),
            action_cost: None,
            frequency: None,
            uses: None,
            rolls: Vec::new(),
            damage: Vec::new(),
            modes: Vec::new(),
            content: None,
            provenance: RuntimeFactProvenanceView {
                source: RuntimeFactSourceView::CanonicalRecord,
                canonical_target: None,
            },
        }
    }

    fn spell_payload_runtime() -> EncounterRuntimeView {
        let mut runtime = empty_runtime();
        for entry_id in ["entry-occult", "entry-coven"] {
            runtime.spellcasting.push(EncounterRuntimeSpellcastingView {
                entry_id: entry_id.to_string(),
                authored_order: 0,
                label: "Spell Attack".to_string(),
                preparation: None,
                tradition: None,
                attack: None,
                dc: None,
                slots: Vec::new(),
                spells: Vec::new(),
            });
            runtime.activities.push(runtime_activity(
                entry_id,
                EncounterRuntimeActivityKindView::Spell,
            ));
        }
        for spell_id in [
            "bind-soul",
            "dream-council",
            "nightmare-1",
            "nightmare-2",
            "dream-message-1",
            "dream-message-2",
            "magic-missile",
            "control-weather",
        ] {
            runtime.activities.push(runtime_activity(
                spell_id,
                EncounterRuntimeActivityKindView::Spell,
            ));
        }
        for index in 0..19 {
            runtime.activities.push(runtime_activity(
                &format!("grouped-spell-{index:02}"),
                EncounterRuntimeActivityKindView::Spell,
            ));
        }
        runtime
    }

    fn spell_payload_fixture() -> atlas_record::CreatureRecord {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        creature.identity.name = "Night Hag".to_string();
        let mut occult = spellcasting_entry_occurrence(
            &owner,
            "entry-occult",
            "Occult Innate Spells",
            0,
            "occult",
        );
        let atlas_record::CreatureCapability::SpellcastingEntry(occult_capability) =
            &mut occult.capability
        else {
            panic!("spellcasting entry fixture");
        };
        occult_capability.slots = FactValue::Value(vec![atlas_record::CreatureSpellSlot {
            rank: 3,
            maximum: FactValue::Value(atlas_record::CreatureSourceScalar::Value(4)),
            serialized_value: FactValue::Value(atlas_record::CreatureSourceScalar::Value(2)),
            prepared: FactValue::Missing,
        }]);
        let coven =
            spellcasting_entry_occurrence(&owner, "entry-coven", "Coven Spells", 1, "occult");
        let occult_parent =
            atlas_record::CreatureOccurrenceId::new("entry-occult").expect("entry id");
        let coven_parent =
            atlas_record::CreatureOccurrenceId::new("entry-coven").expect("entry id");
        let mut bind_soul = spell_occurrence(
            &owner,
            "bind-soul",
            "Bind Soul",
            2,
            CreatureOccurrenceParent::SpellcastingEntry(occult_parent.clone()),
            "spells:bind-soul",
        );
        bind_soul.target = CreatureEntityTarget::ActorOwned(
            atlas_record::CreatureEntityId::new("bind-soul-entity").expect("entity id"),
        );
        let mut dream_council = spell_occurrence(
            &owner,
            "dream-council",
            "Dream Council",
            3,
            CreatureOccurrenceParent::SpellcastingEntry(occult_parent.clone()),
            "spells:dream-council",
        );
        dream_council.target = CreatureEntityTarget::ActorOwned(
            atlas_record::CreatureEntityId::new("dream-council-entity").expect("entity id"),
        );
        let mut magic_missile = with_spell_ranks(
            spell_occurrence(
                &owner,
                "magic-missile",
                "Magic Missile (At Will)",
                8,
                CreatureOccurrenceParent::SpellcastingEntry(occult_parent.clone()),
                "spells:magic-missile",
            ),
            1,
            3,
        );
        magic_missile.context.group = FactValue::Value("at-will".to_string());
        magic_missile.context.location = FactValue::Value("innate".to_string());
        magic_missile.context.slot = FactValue::Value("3".to_string());
        magic_missile.context.uses = FactValue::Value(atlas_record::CreatureUseLimit {
            maximum: FactValue::Value(3),
            serialized_value: FactValue::Value(2),
        });
        let mut grouped = vec![
            bind_soul,
            dream_council,
            with_spell_ranks(
                spell_occurrence(
                    &owner,
                    "nightmare-1",
                    "Nightmare",
                    4,
                    CreatureOccurrenceParent::SpellcastingEntry(occult_parent.clone()),
                    "spells:nightmare",
                ),
                4,
                5,
            ),
            with_spell_ranks(
                spell_occurrence(
                    &owner,
                    "nightmare-2",
                    "Nightmare",
                    5,
                    CreatureOccurrenceParent::SpellcastingEntry(occult_parent.clone()),
                    "spells:nightmare",
                ),
                4,
                5,
            ),
            with_spell_ranks(
                spell_occurrence(
                    &owner,
                    "dream-message-1",
                    "Dream Message",
                    6,
                    CreatureOccurrenceParent::SpellcastingEntry(occult_parent.clone()),
                    "spells:dream-message",
                ),
                3,
                3,
            ),
            with_spell_ranks(
                spell_occurrence(
                    &owner,
                    "dream-message-2",
                    "Dream Message",
                    7,
                    CreatureOccurrenceParent::SpellcastingEntry(occult_parent.clone()),
                    "spells:dream-message",
                ),
                3,
                3,
            ),
            magic_missile,
        ];
        for index in 0..19 {
            let parent = if index < 7 {
                occult_parent.clone()
            } else {
                coven_parent.clone()
            };
            grouped.push(spell_occurrence(
                &owner,
                &format!("grouped-spell-{index:02}"),
                &format!("Grouped Spell {index:02}"),
                9 + index,
                CreatureOccurrenceParent::SpellcastingEntry(parent),
                &format!("spells:grouped-spell-{index:02}"),
            ));
        }
        let standalone = spell_occurrence(
            &owner,
            "control-weather",
            "Control Weather",
            28,
            CreatureOccurrenceParent::Creature,
            "spells:control-weather",
        );
        let mut occurrences = vec![coven, standalone, occult];
        occurrences.extend(grouped.into_iter().rev());
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![
                entity(
                    &owner,
                    "entry-occult",
                    CreatureEntityFamily::SpellcastingEntry,
                ),
                entity(
                    &owner,
                    "entry-coven",
                    CreatureEntityFamily::SpellcastingEntry,
                ),
                entity(&owner, "bind-soul-entity", CreatureEntityFamily::Spell),
                entity(&owner, "dream-council-entity", CreatureEntityFamily::Spell),
                entity(&owner, "heartstone-entity", CreatureEntityFamily::Action),
            ],
            occurrences,
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });
        let occurrence_document = |id: &str, order: u32, text: &str| {
            content_document(
                &owner,
                &format!("{id}-rules"),
                ContentOwner::CreatureOccurrence(
                    atlas_record::CreatureOccurrenceId::new(id).expect("spell id"),
                ),
                order,
                id,
                vec![paragraph(vec![RichNode::Text {
                    text: text.to_string(),
                }])],
            )
        };
        creature.content.documents = vec![
            content_document(
                &owner,
                "bind-soul-second",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new("bind-soul-entity").expect("entity id"),
                ),
                2,
                "Bind Soul",
                vec![paragraph(vec![RichNode::Text {
                    text: "Bind Soul second rules.".to_string(),
                }])],
            ),
            occurrence_document("nightmare-2", 5, "Nightmare second occurrence rules."),
            content_document(
                &owner,
                "heartstone-description",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new("heartstone-entity").expect("entity id"),
                ),
                29,
                "Heartstone",
                vec![paragraph(vec![RichNode::Text {
                    text: "Heartstone general rules.".to_string(),
                }])],
            ),
            occurrence_document("control-weather", 28, "Control Weather rules."),
            content_document(
                &owner,
                "dream-council-rules",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new("dream-council-entity").expect("entity id"),
                ),
                3,
                "Dream Council",
                vec![paragraph(vec![RichNode::Text {
                    text: "Dream Council rules.".to_string(),
                }])],
            ),
            occurrence_document("nightmare-1", 4, "Nightmare first occurrence rules."),
            occurrence_document(
                "dream-message-2",
                7,
                "Dream Message second occurrence rules.",
            ),
            content_document(
                &owner,
                "bind-soul-first",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new("bind-soul-entity").expect("entity id"),
                ),
                1,
                "Bind Soul",
                vec![paragraph(vec![RichNode::Text {
                    text: "Bind Soul first rules.".to_string(),
                }])],
            ),
            occurrence_document(
                "dream-message-1",
                6,
                "Dream Message first occurrence rules.",
            ),
        ];
        creature
    }

    fn spellcasting_entry_occurrence(
        owner: &RecordKey,
        id: &str,
        label: &str,
        authored_order: u32,
        tradition: &str,
    ) -> atlas_record::CreatureEntityOccurrence {
        let mut occurrence = occurrence(
            owner,
            id,
            authored_order,
            CreatureEntityFamily::SpellcastingEntry,
            CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::SpellcastingEntry(
                atlas_record::CreatureSpellcastingEntryCapability {
                    preparation: FactValue::Value(atlas_record::CreatureSpellPreparation::Innate),
                    tradition: FactValue::Value(tradition.to_string()),
                    attack: FactValue::Value(20),
                    dc: FactValue::Value(30),
                    slots: FactValue::Value(Vec::new()),
                    unsupported_notes: Vec::new(),
                },
            ),
        );
        occurrence.context.contextual_label = FactValue::Value(label.to_string());
        occurrence
    }

    fn spell_occurrence(
        owner: &RecordKey,
        id: &str,
        label: &str,
        authored_order: u32,
        parent: CreatureOccurrenceParent,
        target: &str,
    ) -> atlas_record::CreatureEntityOccurrence {
        let mut occurrence = occurrence(
            owner,
            id,
            authored_order,
            CreatureEntityFamily::Spell,
            parent,
            atlas_record::CreatureCapability::Spell(atlas_record::CreatureSpellCapability {
                traits: FactValue::Value(vec!["spell".to_string()]),
                base_rank: FactValue::Value(6),
                signature: FactValue::Missing,
                traditions: FactValue::Missing,
                requirements: FactValue::Missing,
                cost: FactValue::Missing,
                counteraction: FactValue::Missing,
                ritual: FactValue::Missing,
                target: FactValue::Missing,
                area: FactValue::Missing,
                range: FactValue::Missing,
                time: FactValue::Missing,
                duration: FactValue::Missing,
                defense: FactValue::Missing,
                damage: FactValue::Value(Vec::new()),
                action_cost: atlas_record::CreatureActionCost::Actions(2),
                unsupported_notes: Vec::new(),
            }),
        );
        occurrence.context.rank = FactValue::Value(6);
        occurrence.context.contextual_label = FactValue::Value(label.to_string());
        occurrence.target = CreatureEntityTarget::CanonicalRecord(
            RecordKey::parse(target).expect("target key should parse"),
        );
        occurrence
    }

    fn with_spell_ranks(
        mut occurrence: atlas_record::CreatureEntityOccurrence,
        base_rank: i64,
        occurrence_rank: i64,
    ) -> atlas_record::CreatureEntityOccurrence {
        let atlas_record::CreatureCapability::Spell(capability) = &mut occurrence.capability else {
            panic!("spell occurrence should carry a spell capability");
        };
        capability.base_rank = FactValue::Value(base_rank);
        occurrence.context.rank = FactValue::Value(occurrence_rank);
        occurrence
    }

    fn spell_base_rank(
        creature: &atlas_record::CreatureRecord,
        occurrence_id: &str,
    ) -> Option<i64> {
        creature
            .embedded_entities
            .value
            .as_value()
            .expect("embedded entities")
            .occurrences
            .iter()
            .find(|occurrence| occurrence.id.as_str() == occurrence_id)
            .and_then(|occurrence| {
                let atlas_record::CreatureCapability::Spell(capability) = &occurrence.capability
                else {
                    return None;
                };
                capability.base_rank.as_value().copied()
            })
    }

    fn set_hit_points(creature: &mut atlas_record::CreatureRecord, value: i64, maximum: i64) {
        let FactValue::Value(defenses) = &mut creature.defenses.value else {
            panic!("creature defenses should be present");
        };
        let FactValue::Value(hit_points) = &mut defenses.hit_points else {
            panic!("creature hit points should be present");
        };
        hit_points.value = FactValue::Value(CreatureNumber::Integer(value));
        hit_points.maximum = FactValue::Value(maximum);
    }

    fn known_empty_creature() -> atlas_record::CreatureRecord {
        let record_key = RecordKey::new(
            PackName::new("creatures".to_string()).expect("pack"),
            RecordId::new("surface-fixture".to_string()).expect("id"),
        );
        let save = |id: &str, kind| CreatureSave {
            id: atlas_record::CreatureComponentId::new(id).expect("save id"),
            kind,
            value: FactValue::Value(10),
            details: FactValue::Missing,
        };
        atlas_record::CreatureRecord {
            identity: CreatureIdentity {
                record_key,
                source_id: atlas_record::CreatureSourceId::new("surface-fixture")
                    .expect("source id"),
                name: "Surface Fixture".to_string(),
                family: CreatureFamily::Npc,
            },
            level: missing(CreatureSourceField::Level),
            rarity: missing(CreatureSourceField::Rarity),
            traits: missing(CreatureSourceField::Traits),
            size: CreatureFact::source(
                FactValue::Value(atlas_record::CreatureSize::Medium),
                CreatureSourceField::Size,
            ),
            publication: missing(CreatureSourceField::Publication),
            adjustment: missing(CreatureSourceField::Adjustment),
            source_alliance: missing(CreatureSourceField::SourceAlliance),
            perception: CreatureFact::source(
                FactValue::Value(CreaturePerception {
                    modifier: FactValue::Value(10),
                    details: FactValue::Missing,
                    has_vision: FactValue::Value(true),
                    senses: FactValue::Value(Vec::new()),
                }),
                CreatureSourceField::Perception,
            ),
            initiative: missing(CreatureSourceField::Initiative),
            languages: CreatureFact::source(
                FactValue::Value(CreatureLanguages {
                    values: FactValue::Value(Vec::new()),
                    details: FactValue::Missing,
                }),
                CreatureSourceField::Languages,
            ),
            skills: CreatureFact::source(FactValue::Value(Vec::new()), CreatureSourceField::Skills),
            legacy_abilities: CreatureFact::source(
                FactValue::Value(CreatureLegacyAbilities {
                    strength: FactValue::Value(0),
                    dexterity: FactValue::Value(0),
                    constitution: FactValue::Value(0),
                    intelligence: FactValue::Value(0),
                    wisdom: FactValue::Value(0),
                    charisma: FactValue::Value(0),
                }),
                CreatureSourceField::LegacyAbilities,
            ),
            defenses: CreatureFact::source(
                FactValue::Value(CreatureDefenses {
                    armor_class: FactValue::Value(CreatureArmorClass {
                        value: FactValue::Value(20),
                        details: FactValue::Missing,
                    }),
                    hit_points: FactValue::Value(CreatureHitPoints {
                        value: FactValue::Value(CreatureNumber::Integer(50)),
                        maximum: FactValue::Value(50),
                        temporary: FactValue::Missing,
                        temporary_maximum: FactValue::Missing,
                        details: FactValue::Missing,
                    }),
                    hardness: FactValue::Missing,
                    shield: FactValue::Missing,
                    saves: FactValue::Value(CreatureSaves {
                        fortitude: FactValue::Value(save("fortitude", CreatureSaveKind::Fortitude)),
                        reflex: FactValue::Value(save("reflex", CreatureSaveKind::Reflex)),
                        will: FactValue::Value(save("will", CreatureSaveKind::Will)),
                    }),
                    all_saves_note: FactValue::Missing,
                    immunities: FactValue::Value(Vec::new()),
                    resistances: FactValue::Value(Vec::new()),
                    weaknesses: FactValue::Value(Vec::new()),
                }),
                CreatureSourceField::Defenses,
            ),
            movement: CreatureFact::source(
                FactValue::Value(Vec::new()),
                CreatureSourceField::Movement,
            ),
            resources: CreatureFact::source(
                FactValue::Value(Vec::new()),
                CreatureSourceField::Resources,
            ),
            embedded_entities: CreatureFact::source(
                FactValue::Value(CreatureEmbeddedEntities::default()),
                CreatureSourceField::EmbeddedEntities,
            ),
            content: OwnedRichContent::default(),
            provenance: CreatureProvenance {
                source_path: "packs/creatures/surface-fixture.json".to_string(),
                source_contract_version: "test".to_string(),
                source_system_version: "test".to_string(),
                source_upstream_commit: "test".to_string(),
            },
        }
    }

    fn activity_content_fixture() -> atlas_record::CreatureRecord {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(&owner, "entity", CreatureEntityFamily::Action)],
            occurrences: vec![action_occurrence(&owner, "activity", "entity", 0)],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });
        creature.content.documents = vec![content_document(
            &owner,
            "item:activity:description",
            ContentOwner::CreatureEntity(
                atlas_record::CreatureEntityId::new("entity").expect("entity id"),
            ),
            0,
            "Activity",
            vec![paragraph(vec![RichNode::Text {
                text: "Activity content.".to_string(),
            }])],
        )];
        creature
    }

    fn entity(owner: &RecordKey, id: &str, family: CreatureEntityFamily) -> CreatureEntity {
        CreatureEntity {
            id: atlas_record::CreatureEntityId::new(id).expect("entity id"),
            family,
            label: id.to_string(),
            source_identity: atlas_record::CreatureEntitySourceIdentity {
                nested_source_id: FactValue::Missing,
                stable_source_locator: FactValue::Missing,
                source_locators: vec![atlas_record::CreatureSourceLocator {
                    source_path: format!("{owner}/items/{id}"),
                    locator: atlas_record::StableSourceLocator::new(format!("items/{id}"))
                        .expect("source locator"),
                    precedence: 0,
                }],
            },
        }
    }

    fn action_occurrence(
        owner: &RecordKey,
        occurrence_id: &str,
        entity_id: &str,
        authored_order: u32,
    ) -> atlas_record::CreatureEntityOccurrence {
        let mut occurrence = occurrence(
            owner,
            occurrence_id,
            authored_order,
            CreatureEntityFamily::Action,
            atlas_record::CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::Action(CreatureActionCapability {
                category: FactValue::Missing,
                traits: FactValue::Value(Vec::new()),
                action_cost: atlas_record::CreatureActionCost::Actions(1),
                frequency: FactValue::Missing,
                self_effect: FactValue::Missing,
                self_effect_label: FactValue::Missing,
                requirements: FactValue::Missing,
                cost: FactValue::Missing,
                rolls: Vec::new(),
                damage: FactValue::Value(Vec::new()),
                unsupported_notes: Vec::new(),
            }),
        );
        occurrence.target = atlas_record::CreatureEntityTarget::ActorOwned(
            atlas_record::CreatureEntityId::new(entity_id).expect("entity id"),
        );
        occurrence
    }

    fn content_document(
        record_key: &RecordKey,
        content_key: &str,
        owner: ContentOwner,
        authored_order: u32,
        label: &str,
        nodes: Vec<RichNode>,
    ) -> OwnedRichContentDocument {
        let record_owned = matches!(&owner, ContentOwner::Record(_));
        let relative_source_path = if record_owned {
            "system.details.publicNotes".to_string()
        } else {
            format!("items[{label}].system.description.value")
        };
        let source_kind = if record_owned {
            ContentSourceKind::PublicNotes
        } else {
            ContentSourceKind::EmbeddedItemDescription
        };
        let origin = if record_owned {
            ContentOrigin::RecordField {
                source_kind,
                relative_source_path: relative_source_path.clone(),
            }
        } else {
            ContentOrigin::EmbeddedEntityField {
                family: CreatureEntityFamily::Action,
                nested_source_id: Some(label.to_string()),
                relative_source_path: relative_source_path.clone(),
            }
        };
        OwnedRichContentDocument::new(
            ContentId::new(
                record_key.clone(),
                ContentKey::new(content_key).expect("content key"),
            ),
            ContentIdentityStability::StableSourceIdentity,
            owner,
            if record_owned {
                ContentRole::SupplementalRules
            } else {
                ContentRole::EmbeddedCapability
            },
            origin,
            ContentVisibility::Public,
            ContentProvenance {
                source_record_key: record_key.clone(),
                relative_source_path,
                field_or_pointer_family: source_kind.as_str().to_string(),
                nested_source_id: (!record_owned).then(|| label.to_string()),
                authored_ordinal_or_range: Some(authored_order.to_string()),
                authored_label: Some(label.to_string()),
            },
            source_kind,
            authored_order,
            Some(label.to_string()),
            RichDocument::new(nodes),
            DuplicateContentStatus::Unique,
            Vec::new(),
        )
    }

    fn paragraph(children: Vec<RichNode>) -> RichNode {
        RichNode::HtmlElement {
            tag: "p".to_string(),
            attributes: BTreeMap::new(),
            children,
        }
    }

    fn content_contains_text(blocks: &[CreatureSurfaceContentBlockView], needle: &str) -> bool {
        blocks.iter().any(|block| match block {
            CreatureSurfaceContentBlockView::Heading { text, .. } => text.contains(needle),
            CreatureSurfaceContentBlockView::Paragraph { spans } => {
                spans.iter().any(|span| inline_contains_text(span, needle))
            }
            CreatureSurfaceContentBlockView::List { items, .. } => items
                .iter()
                .any(|item| content_contains_text(&item.blocks, needle)),
            CreatureSurfaceContentBlockView::Table { caption, rows } => {
                caption.as_ref().is_some_and(|value| value.contains(needle))
                    || rows.iter().any(|row| {
                        row.cells
                            .iter()
                            .any(|cell| content_contains_text(cell, needle))
                    })
            }
            CreatureSurfaceContentBlockView::Divider => false,
        })
    }

    fn inline_contains_text(span: &CreatureSurfaceContentInlineView, needle: &str) -> bool {
        match span {
            CreatureSurfaceContentInlineView::Text { text }
            | CreatureSurfaceContentInlineView::Code { text } => text.contains(needle),
            CreatureSurfaceContentInlineView::Strong { spans }
            | CreatureSurfaceContentInlineView::Emphasis { spans } => {
                spans.iter().any(|span| inline_contains_text(span, needle))
            }
            CreatureSurfaceContentInlineView::Reference { label, .. } => label.contains(needle),
            CreatureSurfaceContentInlineView::Check { display, .. } => display.contains(needle),
            CreatureSurfaceContentInlineView::LineBreak => false,
        }
    }

    fn missing<T>(field: CreatureSourceField) -> CreatureFact<T> {
        CreatureFact::source(FactValue::Missing, field)
    }

    fn non_value<T>(state: CreatureSurfaceUnavailableStateView) -> FactValue<T> {
        match state {
            CreatureSurfaceUnavailableStateView::Missing => FactValue::Missing,
            CreatureSurfaceUnavailableStateView::Null => FactValue::Null,
            CreatureSurfaceUnavailableStateView::Unsupported => unreachable!(),
        }
    }

    fn unsupported_value() -> UnsupportedSourceValue {
        UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Object,
            value: "{}".to_string(),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        }
    }

    fn occurrence(
        owner: &RecordKey,
        id: &str,
        authored_order: u32,
        family: atlas_record::CreatureEntityFamily,
        parent: atlas_record::CreatureOccurrenceParent,
        capability: atlas_record::CreatureCapability,
    ) -> atlas_record::CreatureEntityOccurrence {
        atlas_record::CreatureEntityOccurrence {
            id: atlas_record::CreatureOccurrenceId::new(id).expect("occurrence id"),
            identity_stability: atlas_record::OccurrenceIdentityStability::StableNestedSourceId,
            owner: owner.clone(),
            target: atlas_record::CreatureEntityTarget::ActorOwned(
                atlas_record::CreatureEntityId::new(id).expect("entity id"),
            ),
            family,
            authored_order,
            source_sort: FactValue::Value(authored_order.into()),
            source_folder: FactValue::Missing,
            source_identity: atlas_record::CreatureEntitySourceIdentity {
                nested_source_id: FactValue::Value(
                    atlas_record::CreatureSourceId::new(format!("source-{id}")).expect("source id"),
                ),
                stable_source_locator: FactValue::Value(
                    atlas_record::StableSourceLocator::new(format!("items/{id}"))
                        .expect("stable locator"),
                ),
                source_locators: vec![atlas_record::CreatureSourceLocator {
                    source_path: format!("{owner}/items/{id}"),
                    locator: atlas_record::StableSourceLocator::new(format!("items/{id}"))
                        .expect("source locator"),
                    precedence: 0,
                }],
            },
            parent,
            context: atlas_record::CreatureOccurrenceContext {
                contextual_label: FactValue::Value(id.to_string()),
                ..atlas_record::CreatureOccurrenceContext::default()
            },
            capability,
            deltas: Vec::new(),
        }
    }

    fn cause(
        state: CreatureSurfaceUnavailableStateView,
        field: CreatureSurfaceUnavailableFieldView,
        component_id: Option<&str>,
        source_field: CreatureSurfaceSourceFieldView,
    ) -> CauseTuple {
        (
            state,
            field,
            component_id.map(str::to_string),
            CreatureSurfaceFactOwnerView::CanonicalCreature,
            source_field,
        )
    }

    fn assert_causes(
        domain: &Option<CreatureSurfaceDomainUnavailableView>,
        mut expected: Vec<CauseTuple>,
    ) {
        let actual = domain
            .as_ref()
            .expect("expected unavailable domain")
            .causes
            .iter()
            .map(|cause| {
                (
                    cause.state,
                    cause.field,
                    cause.component_id.clone(),
                    cause.provenance.owner,
                    cause.provenance.field,
                )
            })
            .collect::<Vec<_>>();
        let mut sorted_actual = actual.clone();
        sorted_actual.sort();
        assert_eq!(
            actual, sorted_actual,
            "typed causes must use canonical order"
        );
        expected.sort();
        assert_eq!(actual, expected);
    }
}
