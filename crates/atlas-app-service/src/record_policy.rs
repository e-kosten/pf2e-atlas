use std::collections::BTreeSet;

use atlas_app_model::{
    CreatureSurfaceDomainUnavailableView, CreatureSurfaceUnavailableDomainsView,
    CreatureSurfaceUnavailableFieldView, CreatureSurfaceUnavailableStateView,
    HazardSurfaceUnavailableStateView, HazardSurfaceView, RecordSurfaceIssueCodeView,
    RecordSurfaceIssuePlacementView, RecordSurfaceIssueSubjectView, RecordSurfaceIssueTargetView,
    RecordSurfaceIssueView, RecordSurfacePresentationView,
};
use atlas_record::{
    FactIssueKind, FactPresentationDisposition, FactPresentationRole, FactPresentationState,
    FactRequirement, FactValue, HazardSourceMetadataField, HazardSourceMetadataIssueKind,
    RecordBody, RetrievedRecord, SpellPresentationIssue, SpellPresentationIssuePlacement,
    classify_fact_presentation, merge_spell_presentation_issues, project_hazard_source_metadata,
    project_spell_presentation_issues,
};

pub(crate) fn record_surface_issues(
    retrieved: &RetrievedRecord,
    presentation: &RecordSurfacePresentationView,
    selected_spell_issues: &[SpellPresentationIssue],
) -> Option<Vec<RecordSurfaceIssueView>> {
    let mut issues = Vec::new();
    match (presentation, &retrieved.body) {
        (RecordSurfacePresentationView::Creature { body }, Some(RecordBody::Creature(_))) => {
            creature_issues(body.unavailable_domains.as_ref(), &mut issues)
        }
        (RecordSurfacePresentationView::Hazard { body }, Some(RecordBody::Hazard(hazard))) => {
            hazard_issues(hazard, body, &mut issues);
        }
        (RecordSurfacePresentationView::Spell { .. }, Some(RecordBody::Spell(spell))) => {
            spell_issues(spell, selected_spell_issues, &mut issues);
        }
        (RecordSurfacePresentationView::Journal { .. }, Some(RecordBody::Journal(journal))) => {
            journal_issues(journal, &mut issues);
        }
        (RecordSurfacePresentationView::RollTable { .. }, Some(RecordBody::RollTable(table))) => {
            roll_table_issues(table, &mut issues);
        }
        (RecordSurfacePresentationView::Unavailable { .. }, _) => {
            issues.push(RecordSurfaceIssueView {
                consequence: None,
                fact_id: None,
                code: RecordSurfaceIssueCodeView::Unavailable,
                placement: RecordSurfaceIssuePlacementView::Record,
                subject: None,
                fact_label: None,
                message: "This record does not have an available typed presentation.".to_string(),
            })
        }
        _ => {}
    }
    (!issues.is_empty()).then_some(issues)
}

fn h8_unknown_issues<'a>(
    fields: &[atlas_record::H8UnsupportedField],
    children: impl Iterator<Item = &'a atlas_record::H8UnsupportedChild>,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    issues.extend(fields.iter().map(|field| RecordSurfaceIssueView {
        consequence: Some("The exact authored value is retained in machine provenance but is not presented as a modeled field.".to_string()),
        fact_id: Some(format!("h8:{}:{}", field.relative_path, field.authored_order)),
        code: RecordSurfaceIssueCodeView::Unsupported,
        placement: RecordSurfaceIssuePlacementView::Record,
        subject: None,
        fact_label: Some(field.relative_path.clone()),
        message: "This source field does not yet have an H8 product meaning.".to_string(),
    }));
    issues.extend(children.map(|child| RecordSurfaceIssueView {
        consequence: Some("Valid sibling content remains available.".to_string()),
        fact_id: Some(format!(
            "h8-child:{}",
            atlas_record::encode_content_child_locator(&child.locator)
        )),
        code: RecordSurfaceIssueCodeView::Unsupported,
        placement: RecordSurfaceIssuePlacementView::Record,
        subject: None,
        fact_label: Some(format!("Child {}", child.source_ordinal.saturating_add(1))),
        message: child.reason.clone(),
    }));
}

fn journal_issues(journal: &atlas_record::JournalRecord, issues: &mut Vec<RecordSurfaceIssueView>) {
    required_h8_fact_issue(&journal.pages, "pages", "Journal pages", issues);
    h8_source_metadata_issues(&journal.source_metadata, "journal", issues);
    h8_unknown_issues(
        &journal.unsupported_fields,
        journal
            .pages
            .as_value()
            .and_then(atlas_record::H8FieldValue::known)
            .into_iter()
            .flatten()
            .filter_map(|entry| match entry {
                atlas_record::JournalPageEntry::Unsupported(value) => Some(value),
                atlas_record::JournalPageEntry::Page(_) => None,
            }),
        issues,
    );
    let Some(pages) = journal
        .pages
        .as_value()
        .and_then(atlas_record::H8FieldValue::known)
    else {
        return;
    };
    for entry in pages {
        let atlas_record::JournalPageEntry::Page(page) = entry else {
            continue;
        };
        let prefix = format!("page.{}", page.source_ordinal);
        required_h8_fact_issue(
            &page.source_id,
            &format!("{prefix}.source_id"),
            "Page source ID",
            issues,
        );
        h8_unstable_identity_issue(
            &page.locator,
            &page.source_id,
            &format!("{prefix}.source_id"),
            "Page source ID",
            issues,
        );
        h8_fact_issue(&page.name, &format!("{prefix}.name"), "Page title", issues);
        h8_fact_issue(&page.sort, &format!("{prefix}.sort"), "Page sort", issues);
        h8_fact_issue(
            &page.source,
            &format!("{prefix}.src"),
            "Page media source",
            issues,
        );
        h8_fact_issue(
            &page.image_caption,
            &format!("{prefix}.image.caption"),
            "Image caption",
            issues,
        );
        h8_fact_issue(
            &page.source_system,
            &format!("{prefix}.system"),
            "Page type data",
            issues,
        );
        if let FactValue::Value(atlas_record::H8FieldValue::Known(title)) = &page.title {
            required_h8_fact_issue(
                &title.show,
                &format!("{prefix}.title.show"),
                "Title visibility",
                issues,
            );
            required_h8_fact_issue(
                &title.level,
                &format!("{prefix}.title.level"),
                "Title level",
                issues,
            );
        } else {
            h8_fact_issue(
                &page.title,
                &format!("{prefix}.title"),
                "Page title metadata",
                issues,
            );
        }
        if let FactValue::Value(atlas_record::H8FieldValue::Known(text)) = &page.text {
            h8_fact_issue(
                &text.content,
                &format!("{prefix}.text.content"),
                "Page content",
                issues,
            );
            h8_fact_issue(
                &text.format,
                &format!("{prefix}.text.format"),
                "Page text format",
                issues,
            );
            h8_fact_issue(
                &text.markdown,
                &format!("{prefix}.text.markdown"),
                "Page Markdown",
                issues,
            );
        } else {
            h8_fact_issue(&page.text, &format!("{prefix}.text"), "Page text", issues);
        }
        if let FactValue::Value(atlas_record::H8FieldValue::Known(video)) = &page.video {
            h8_fact_issue(
                &video.controls,
                &format!("{prefix}.video.controls"),
                "Video controls",
                issues,
            );
            h8_fact_issue(
                &video.loop_playback,
                &format!("{prefix}.video.loop"),
                "Video loop",
                issues,
            );
            h8_fact_issue(
                &video.autoplay,
                &format!("{prefix}.video.autoplay"),
                "Video autoplay",
                issues,
            );
            h8_fact_issue(
                &video.volume,
                &format!("{prefix}.video.volume"),
                "Video volume",
                issues,
            );
            h8_fact_issue(
                &video.timestamp,
                &format!("{prefix}.video.timestamp"),
                "Video timestamp",
                issues,
            );
            h8_fact_issue(
                &video.width,
                &format!("{prefix}.video.width"),
                "Video width",
                issues,
            );
            h8_fact_issue(
                &video.height,
                &format!("{prefix}.video.height"),
                "Video height",
                issues,
            );
        } else {
            h8_fact_issue(
                &page.video,
                &format!("{prefix}.video"),
                "Page video",
                issues,
            );
        }
        h8_fact_issue(
            &page.source_metadata.ownership,
            &format!("{prefix}.ownership"),
            "Page ownership metadata",
            issues,
        );
        h8_fact_issue(
            &page.source_metadata.flags,
            &format!("{prefix}.flags"),
            "Page flags metadata",
            issues,
        );
        h8_fact_issue(
            &page.source_metadata.stats,
            &format!("{prefix}._stats"),
            "Page source statistics",
            issues,
        );
        h8_unknown_issues(&page.unsupported_fields, std::iter::empty(), issues);
    }
}

fn roll_table_issues(
    table: &atlas_record::RollTableRecord,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    required_h8_fact_issue(&table.results, "results", "Table results", issues);
    h8_fact_issue(
        &table.description,
        "description",
        "Table description",
        issues,
    );
    h8_fact_issue(&table.formula, "formula", "Table formula", issues);
    h8_fact_issue(
        &table.replacement,
        "replacement",
        "Replacement policy",
        issues,
    );
    h8_fact_issue(
        &table.display_roll,
        "display_roll",
        "Display-roll policy",
        issues,
    );
    h8_fact_issue(&table.image, "img", "Table image", issues);
    h8_source_metadata_issues(&table.source_metadata, "roll_table", issues);
    h8_unknown_issues(
        &table.unsupported_fields,
        table
            .results
            .as_value()
            .and_then(atlas_record::H8FieldValue::known)
            .into_iter()
            .flatten()
            .filter_map(|entry| match entry {
                atlas_record::TableResultEntry::Unsupported(value) => Some(value),
                atlas_record::TableResultEntry::Result(_) => None,
            }),
        issues,
    );
    let Some(results) = table
        .results
        .as_value()
        .and_then(atlas_record::H8FieldValue::known)
    else {
        return;
    };
    for entry in results {
        let atlas_record::TableResultEntry::Result(result) = entry else {
            continue;
        };
        let prefix = format!("result.{}", result.source_ordinal);
        required_h8_fact_issue(
            &result.source_id,
            &format!("{prefix}.source_id"),
            "Result source ID",
            issues,
        );
        h8_unstable_identity_issue(
            &result.locator,
            &result.source_id,
            &format!("{prefix}.source_id"),
            "Result source ID",
            issues,
        );
        h8_fact_issue(
            &result.text,
            &format!("{prefix}.text"),
            "Result text",
            issues,
        );
        h8_fact_issue(
            &result.target.collection,
            &format!("{prefix}.collection"),
            "Result collection",
            issues,
        );
        h8_fact_issue(
            &result.target.document_id,
            &format!("{prefix}.document_id"),
            "Result document ID",
            issues,
        );
        h8_fact_issue(
            &result.weight,
            &format!("{prefix}.weight"),
            "Result weight",
            issues,
        );
        h8_fact_issue(
            &result.range,
            &format!("{prefix}.range"),
            "Result range",
            issues,
        );
        h8_fact_issue(
            &result.drawn,
            &format!("{prefix}.drawn"),
            "Result drawn state",
            issues,
        );
        h8_fact_issue(
            &result.image,
            &format!("{prefix}.img"),
            "Result image",
            issues,
        );
        h8_fact_issue(
            &result.source_metadata.flags,
            &format!("{prefix}.flags"),
            "Result flags metadata",
            issues,
        );
        h8_unknown_issues(&result.unsupported_fields, std::iter::empty(), issues);
    }
}

fn h8_source_metadata_issues(
    metadata: &atlas_record::H8RecordSourceMetadata,
    prefix: &str,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    h8_fact_issue(
        &metadata.folder,
        &format!("{prefix}.folder"),
        "Source folder",
        issues,
    );
    h8_fact_issue(
        &metadata.sort,
        &format!("{prefix}.sort"),
        "Source sort",
        issues,
    );
    h8_fact_issue(
        &metadata.ownership,
        &format!("{prefix}.ownership"),
        "Source ownership",
        issues,
    );
    h8_fact_issue(
        &metadata.flags,
        &format!("{prefix}.flags"),
        "Source flags",
        issues,
    );
    h8_fact_issue(
        &metadata.stats,
        &format!("{prefix}._stats"),
        "Source statistics",
        issues,
    );
}

fn required_h8_fact_issue<T>(
    fact: &atlas_record::H8Fact<T>,
    fact_id: &str,
    label: &str,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    match fact {
        FactValue::Missing => push_h8_issue(
            issues,
            RecordSurfaceIssueCodeView::Unavailable,
            fact_id,
            label,
            format!("{label} are missing from the authored record."),
        ),
        FactValue::Null => push_h8_issue(
            issues,
            RecordSurfaceIssueCodeView::Unavailable,
            fact_id,
            label,
            format!("{label} are null in the authored record."),
        ),
        FactValue::Value(atlas_record::H8FieldValue::Unsupported(_)) => push_h8_issue(
            issues,
            RecordSurfaceIssueCodeView::Unsupported,
            fact_id,
            label,
            format!("{label} have an unsupported authored value."),
        ),
        FactValue::Value(atlas_record::H8FieldValue::Known(_)) => {}
    }
}

fn h8_unstable_identity_issue(
    locator: &atlas_record::ContentChildLocator,
    source_id: &atlas_record::H8Fact<atlas_record::SourceDocumentId>,
    fact_id: &str,
    label: &str,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    if matches!(
        locator.identity,
        atlas_record::ContentChildIdentity::Unstable { .. }
    ) && matches!(
        source_id,
        FactValue::Value(atlas_record::H8FieldValue::Known(_))
    ) {
        push_h8_issue(
            issues,
            RecordSurfaceIssueCodeView::Ambiguous,
            fact_id,
            label,
            format!(
                "{label} is duplicated within its parent; this child uses an authored-order locator."
            ),
        );
    }
}

fn h8_fact_issue<T>(
    fact: &atlas_record::H8Fact<T>,
    fact_id: &str,
    label: &str,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    if matches!(
        fact,
        FactValue::Value(atlas_record::H8FieldValue::Unsupported(_))
    ) {
        push_h8_issue(
            issues,
            RecordSurfaceIssueCodeView::Unsupported,
            fact_id,
            label,
            format!("{label} has an unsupported authored value."),
        );
    }
}

fn push_h8_issue(
    issues: &mut Vec<RecordSurfaceIssueView>,
    code: RecordSurfaceIssueCodeView,
    fact_id: &str,
    label: &str,
    message: String,
) {
    issues.push(RecordSurfaceIssueView {
        consequence: Some(
            "The exact authored state remains available in the typed machine response.".to_string(),
        ),
        fact_id: Some(format!("h8:{fact_id}")),
        code,
        placement: RecordSurfaceIssuePlacementView::Record,
        subject: None,
        fact_label: Some(label.to_string()),
        message,
    });
}

fn creature_issues(
    unavailable: Option<&CreatureSurfaceUnavailableDomainsView>,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    let Some(unavailable) = unavailable else {
        return;
    };
    let mut seen = BTreeSet::new();
    for (placement, domain) in [
        (
            RecordSurfaceIssuePlacementView::Classification,
            unavailable.classification.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Classification,
            unavailable.initiative.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Defenses,
            unavailable.vitals.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Defenses,
            unavailable.defenses.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Defenses,
            unavailable.saves.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.awareness.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.abilities.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.skills.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.movement.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.resources.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.equipment.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.lore.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Casting,
            unavailable.spellcasting.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Activity,
            unavailable.activities.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.relationships.as_ref(),
        ),
    ] {
        extend_creature_domain(issues, &mut seen, placement, domain);
    }
}

fn extend_creature_domain(
    issues: &mut Vec<RecordSurfaceIssueView>,
    seen: &mut BTreeSet<(
        CreatureSurfaceUnavailableStateView,
        CreatureSurfaceUnavailableFieldView,
        Option<String>,
    )>,
    placement: RecordSurfaceIssuePlacementView,
    domain: Option<&CreatureSurfaceDomainUnavailableView>,
) {
    let Some(domain) = domain else {
        return;
    };
    for cause in &domain.causes {
        if !seen.insert((cause.state, cause.field, cause.component_id.clone())) {
            continue;
        }
        let (role, state) = if cause.field == CreatureSurfaceUnavailableFieldView::UnmodeledSkill {
            (
                FactPresentationRole::Unmodeled,
                FactPresentationState::Known,
            )
        } else {
            (
                FactPresentationRole::Gameplay,
                match cause.state {
                    CreatureSurfaceUnavailableStateView::Missing => FactPresentationState::Missing,
                    CreatureSurfaceUnavailableStateView::Null => FactPresentationState::Null,
                    CreatureSurfaceUnavailableStateView::Unsupported => {
                        FactPresentationState::Unsupported
                    }
                },
            )
        };
        push_policy_issue(
            issues,
            role,
            state,
            FactRequirement::Required,
            placement,
            &cause.message,
        );
    }
}

fn hazard_issues(
    hazard: &atlas_record::HazardRecord,
    body: &HazardSurfaceView,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    let metadata = project_hazard_source_metadata(hazard);
    let mut metadata_issue_keys = BTreeSet::new();
    for issue in metadata.issues {
        metadata_issue_keys.insert((issue.field_key(), issue.component_id().map(str::to_string)));
        issues.push(RecordSurfaceIssueView {
            consequence: None,
            fact_id: None,
            code: match issue.kind {
                HazardSourceMetadataIssueKind::Malformed => RecordSurfaceIssueCodeView::Malformed,
                HazardSourceMetadataIssueKind::HasHealthConflict
                | HazardSourceMetadataIssueKind::WeaponTypeConflict => {
                    RecordSurfaceIssueCodeView::SourceConsistencyConflict
                }
                HazardSourceMetadataIssueKind::NonZeroTemporaryMaximum
                | HazardSourceMetadataIssueKind::NonEmptySaveDetail
                | HazardSourceMetadataIssueKind::NonEmptyAttackEffectsCustom => {
                    RecordSurfaceIssueCodeView::Unmodeled
                }
            },
            placement: match issue.field {
                HazardSourceMetadataField::HasHealth
                | HazardSourceMetadataField::TemporaryMaximum
                | HazardSourceMetadataField::SaveDetail(_) => {
                    RecordSurfaceIssuePlacementView::Defenses
                }
                HazardSourceMetadataField::ItemRarity
                | HazardSourceMetadataField::ItemLineage
                | HazardSourceMetadataField::StrikeAttack
                | HazardSourceMetadataField::StrikeWeaponType
                | HazardSourceMetadataField::StrikeAttackEffectsCustom => {
                    RecordSurfaceIssuePlacementView::Activity
                }
                HazardSourceMetadataField::TokenName => RecordSurfaceIssuePlacementView::Record,
            },
            subject: issue
                .entity_id
                .as_ref()
                .and_then(|entity_id| hazard_issue_subject(hazard, entity_id.as_str())),
            fact_label: Some(hazard_metadata_fact_label(issue.field).to_string()),
            message: hazard_metadata_issue_message(hazard, &issue),
        });
    }

    for unavailable in body.unavailable_fields.iter().flatten() {
        if metadata_issue_keys
            .contains(&(unavailable.field.as_str(), unavailable.component_id.clone()))
        {
            continue;
        }
        let state = match unavailable.state {
            HazardSurfaceUnavailableStateView::Missing => FactPresentationState::Missing,
            HazardSurfaceUnavailableStateView::Null => FactPresentationState::Null,
            HazardSurfaceUnavailableStateView::Unsupported => FactPresentationState::Unsupported,
        };
        let FactPresentationDisposition::Issue(kind) = classify_fact_presentation(
            FactPresentationRole::Gameplay,
            state,
            FactRequirement::Optional,
        ) else {
            continue;
        };
        issues.push(RecordSurfaceIssueView {
            fact_id: unavailable.fact_id.clone(),
            code: issue_code(kind),
            placement: if unavailable.component_id.is_some() {
                RecordSurfaceIssuePlacementView::Activity
            } else {
                RecordSurfaceIssuePlacementView::Record
            },
            subject: unavailable
                .component_id
                .as_deref()
                .and_then(|component_id| hazard_issue_subject(hazard, component_id)),
            fact_label: hazard_unavailable_fact_label(&unavailable.field).map(str::to_string),
            message: unavailable.message.clone(),
            consequence: unavailable.consequence.clone(),
        });
    }
}

fn spell_issues(
    spell: &atlas_record::SpellRecord,
    selected: &[SpellPresentationIssue],
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    let projected = project_spell_presentation_issues(spell);
    issues.extend(
        merge_spell_presentation_issues(projected, selected.iter().copied())
            .into_iter()
            .map(|issue| RecordSurfaceIssueView {
                consequence: None,
                fact_id: None,
                code: issue_code(issue.kind),
                placement: spell_issue_placement(issue.placement()),
                subject: None,
                fact_label: Some(issue.field.label().to_string()),
                message: issue.message(),
            }),
    );
}

fn spell_issue_placement(
    placement: SpellPresentationIssuePlacement,
) -> RecordSurfaceIssuePlacementView {
    match placement {
        SpellPresentationIssuePlacement::Record => RecordSurfaceIssuePlacementView::Record,
        SpellPresentationIssuePlacement::Classification => {
            RecordSurfaceIssuePlacementView::Classification
        }
        SpellPresentationIssuePlacement::Casting => RecordSurfaceIssuePlacementView::Casting,
        SpellPresentationIssuePlacement::Targeting => RecordSurfaceIssuePlacementView::Targeting,
        SpellPresentationIssuePlacement::Defenses => RecordSurfaceIssuePlacementView::Defenses,
        SpellPresentationIssuePlacement::Damage => RecordSurfaceIssuePlacementView::Damage,
        SpellPresentationIssuePlacement::Duration => RecordSurfaceIssuePlacementView::Duration,
        SpellPresentationIssuePlacement::Heightening => {
            RecordSurfaceIssuePlacementView::Heightening
        }
        SpellPresentationIssuePlacement::Ritual => RecordSurfaceIssuePlacementView::Ritual,
        SpellPresentationIssuePlacement::Rules => RecordSurfaceIssuePlacementView::Rules,
        SpellPresentationIssuePlacement::Forms => RecordSurfaceIssuePlacementView::Forms,
    }
}

fn push_policy_issue(
    issues: &mut Vec<RecordSurfaceIssueView>,
    role: FactPresentationRole,
    state: FactPresentationState,
    requirement: FactRequirement,
    placement: RecordSurfaceIssuePlacementView,
    message: &str,
) {
    let FactPresentationDisposition::Issue(kind) =
        classify_fact_presentation(role, state, requirement)
    else {
        return;
    };
    issues.push(RecordSurfaceIssueView {
        consequence: None,
        fact_id: None,
        code: issue_code(kind),
        placement,
        subject: None,
        fact_label: None,
        message: message.to_string(),
    });
}

fn hazard_issue_subject(
    hazard: &atlas_record::HazardRecord,
    component_id: &str,
) -> Option<RecordSurfaceIssueSubjectView> {
    let embedded = hazard.embedded_entities.typed()?;
    let occurrence = embedded.occurrences.iter().find(|occurrence| {
        occurrence.id.as_str() == component_id || occurrence.entity_id.as_str() == component_id
    })?;
    let entity = embedded
        .entities
        .iter()
        .find(|entity| entity.id == occurrence.entity_id)?;
    let label = occurrence
        .contextual_label
        .typed()
        .cloned()
        .unwrap_or_else(|| entity.label.clone());
    Some(RecordSurfaceIssueSubjectView {
        label,
        target: Some(RecordSurfaceIssueTargetView::Activity {
            occurrence_id: occurrence.id.as_str().to_string(),
        }),
    })
}

fn hazard_metadata_issue_message(
    hazard: &atlas_record::HazardRecord,
    issue: &atlas_record::HazardSourceMetadataIssue,
) -> String {
    if issue.kind != HazardSourceMetadataIssueKind::Malformed {
        return issue.message().to_string();
    }
    if issue.field == HazardSourceMetadataField::StrikeAttack {
        let value = hazard
            .embedded_entities
            .typed()
            .and_then(|embedded| {
                embedded
                    .entities
                    .iter()
                    .find(|entity| Some(&entity.id) == issue.entity_id.as_ref())
            })
            .and_then(|entity| match &entity.capability {
                atlas_record::HazardCapability::Strike(strike) => {
                    match &strike.source_metadata.attack.value {
                        atlas_record::FactValue::Value(
                            atlas_record::HazardSourceValue::Unsupported(value),
                        ) => Some(value),
                        _ => None,
                    }
                }
                _ => None,
            });
        let shape = value
            .map(|value| hazard_source_shape_label(value.actual_shape))
            .unwrap_or("an unsupported value");
        return format!(
            "The extra source attack field contains {shape} where an integer is expected. It does not supply the displayed attack bonus; that bonus comes from the separate gameplay attack field. The original value is retained in Source & provenance."
        );
    }
    format!(
        "{} has an unsupported source value. It is retained in Source & provenance and is not used as a gameplay fallback.",
        hazard_metadata_fact_label(issue.field)
    )
}

pub(crate) fn hazard_source_shape_label(shape: atlas_record::HazardSourceShape) -> &'static str {
    use atlas_record::HazardSourceShape;
    match shape {
        HazardSourceShape::Missing => "a missing value",
        HazardSourceShape::Null => "an explicit null",
        HazardSourceShape::Boolean => "a yes/no value",
        HazardSourceShape::Number => "a number",
        HazardSourceShape::String => "text",
        HazardSourceShape::Array => "a list",
        HazardSourceShape::Object => "a structured value",
    }
}

fn hazard_metadata_fact_label(field: HazardSourceMetadataField) -> &'static str {
    match field {
        HazardSourceMetadataField::TokenName => "Token name",
        HazardSourceMetadataField::HasHealth => "Health compatibility",
        HazardSourceMetadataField::TemporaryMaximum => "Temporary maximum HP",
        HazardSourceMetadataField::SaveDetail(atlas_record::HazardSaveKind::Fortitude) => {
            "Fortitude source note"
        }
        HazardSourceMetadataField::SaveDetail(atlas_record::HazardSaveKind::Reflex) => {
            "Reflex source note"
        }
        HazardSourceMetadataField::SaveDetail(atlas_record::HazardSaveKind::Will) => {
            "Will source note"
        }
        HazardSourceMetadataField::ItemRarity => "Component rarity",
        HazardSourceMetadataField::ItemLineage => "Component lineage",
        HazardSourceMetadataField::StrikeAttack => "Strike source attack",
        HazardSourceMetadataField::StrikeWeaponType => "Strike source mode",
        HazardSourceMetadataField::StrikeAttackEffectsCustom => "Custom attack effect",
    }
}

fn hazard_unavailable_fact_label(field: &str) -> Option<&'static str> {
    match field {
        "activity.attack_effects" => Some("Attack effect"),
        "activity.action.unexpected" => Some("Authored action field"),
        "activity.strike.unexpected" => Some("Authored strike field"),
        "activity.condition.unexpected" => Some("Authored condition field"),
        "activity.effect.unexpected" => Some("Authored effect field"),
        "activity.unsupported_child.field" => Some("Content-only component field"),
        "activity.child_type" => Some("Content-only component type"),
        "activity.rules" => Some("Activity rule"),
        _ => None,
    }
}

fn issue_code(kind: FactIssueKind) -> RecordSurfaceIssueCodeView {
    match kind {
        FactIssueKind::RequiredMissing => RecordSurfaceIssueCodeView::RequiredMissing,
        FactIssueKind::RequiredNull => RecordSurfaceIssueCodeView::RequiredNull,
        FactIssueKind::RequiredEmpty => RecordSurfaceIssueCodeView::RequiredEmpty,
        FactIssueKind::Unsupported => RecordSurfaceIssueCodeView::Unsupported,
        FactIssueKind::Malformed => RecordSurfaceIssueCodeView::Malformed,
        FactIssueKind::Ambiguous => RecordSurfaceIssueCodeView::Ambiguous,
        FactIssueKind::Unavailable => RecordSurfaceIssueCodeView::Unavailable,
        FactIssueKind::Unmodeled => RecordSurfaceIssueCodeView::Unmodeled,
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        RecordSurfaceIssueCodeView, RecordSurfaceIssuePlacementView, RecordSurfacePresentationView,
        RecordSurfaceProfileView, SpellFormResultView, SpellResolvedFieldView,
    };
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        ContentChildIdentity, ContentChildKind, ContentChildLocator, FactValue, H8FieldValue,
        H8Identity, H8PageSourceMetadata, H8Provenance, H8RecordSourceMetadata, H8UnsupportedChild,
        JournalPage, JournalPageEntry, JournalRecord, OwnedRichContent, RecordBody,
        RollTableRecord, SourceDocumentId, SpellClassification, SpellFixedHeighteningLayer,
        SpellFormId, SpellHeightening, SpellIdentity, SpellKeyedPatch, SpellKeyedPatchMember,
        SpellKeyedPatchOperation, SpellOverlay, SpellOverlayId, SpellOverlayType, SpellPatch,
        SpellProvenance, SpellRecord, SpellRollOptionRule, SpellRule, SpellRuleElement,
        SpellRulePredicate, SpellRuleSuboption, SpellSourceId, SpellSourceValue,
        SpellUnsupportedRule, SpellUnsupportedRulePredicate, SpellUnsupportedSourceFact,
        SpellUnsupportedSourceField, TableResultEntry, UnsupportedSourceReason,
        UnsupportedSourceShape, UnsupportedSourceValue,
    };
    use atlas_search::RemasterLinksResult;

    use crate::retrieval::VerifiedRemasterLookup;
    use crate::test_support::encounter_fixture_worker;

    #[test]
    fn h8_journal_surface_preserves_four_state_metadata_without_fabricated_media() {
        let key = RecordKey::parse("journals:h8Surface").expect("journal key");
        let page_id = SourceDocumentId::new("page-system").expect("page source id");
        let record = h8_fixture(
            key.clone(),
            RecordKind::Journal,
            RecordBody::Journal(JournalRecord {
                identity: H8Identity {
                    record_key: key.clone(),
                    source_id: SourceDocumentId::new("h8-journal").expect("source id"),
                    name: "H8 Journal".to_string(),
                },
                pages: FactValue::Value(H8FieldValue::Known(vec![JournalPageEntry::Page(
                    Box::new(JournalPage {
                        locator: atlas_record::ContentChildLocator {
                            parent: key.clone(),
                            kind: ContentChildKind::JournalPage,
                            identity: ContentChildIdentity::Stable(page_id.clone()),
                        },
                        source_id: FactValue::Value(H8FieldValue::Known(page_id)),
                        source_ordinal: 0,
                        name: FactValue::Value(H8FieldValue::Known("Page".to_string())),
                        page_kind: FactValue::Missing,
                        sort: FactValue::Missing,
                        title: FactValue::Missing,
                        text: FactValue::Missing,
                        source: FactValue::Missing,
                        image_source: FactValue::Missing,
                        image_caption: FactValue::Missing,
                        video: FactValue::Missing,
                        source_system: FactValue::Value(H8FieldValue::Unsupported(
                            unsupported_value(),
                        )),
                        source_metadata: H8PageSourceMetadata {
                            ownership: FactValue::Missing,
                            flags: FactValue::Missing,
                            stats: FactValue::Missing,
                        },
                        unsupported_fields: Vec::new(),
                    }),
                )])),
                source_metadata: H8RecordSourceMetadata {
                    folder: FactValue::Null,
                    sort: FactValue::Value(H8FieldValue::Known(0)),
                    ownership: FactValue::Missing,
                    flags: FactValue::Value(H8FieldValue::Unsupported(unsupported_value())),
                    stats: FactValue::Missing,
                },
                content: OwnedRichContent::default(),
                unsupported_fields: Vec::new(),
                provenance: h8_provenance(),
            }),
        );

        let surface = surface(&record, None);
        let RecordSurfacePresentationView::Journal { body } = &surface.presentation else {
            panic!("journal presentation");
        };
        assert!(matches!(
            body.source_metadata.folder,
            atlas_app_model::H8FactView::Null
        ));
        assert!(matches!(
            body.source_metadata.sort,
            atlas_app_model::H8FactView::Known(0)
        ));
        assert!(matches!(
            body.source_metadata.ownership,
            atlas_app_model::H8FactView::Missing
        ));
        assert!(matches!(
            body.source_metadata.flags,
            atlas_app_model::H8FactView::Unsupported(_)
        ));
        assert!(
            surface
                .issues
                .as_ref()
                .is_some_and(|issues| issues.iter().any(|issue| {
                    issue.code == RecordSurfaceIssueCodeView::Unsupported
                        && issue.fact_id.as_deref() == Some("h8:journal.flags")
                }))
        );
        assert!(surface.issues.as_ref().is_some_and(|issues| {
            issues.iter().any(|issue| {
                issue.code == RecordSurfaceIssueCodeView::Unsupported
                    && issue.fact_id.as_deref() == Some("h8:page.0.system")
            })
        }));
    }

    #[test]
    fn h8_roll_table_surface_localizes_unsupported_child_and_keeps_siblings_available() {
        let key = RecordKey::parse("roll-tables:h8Surface").expect("table key");
        let unsupported_child = |source_ordinal, source_id| {
            TableResultEntry::Unsupported(H8UnsupportedChild {
                locator: ContentChildLocator {
                    parent: key.clone(),
                    kind: ContentChildKind::TableResult,
                    identity: ContentChildIdentity::Unstable { source_ordinal },
                },
                source_id,
                source_ordinal,
                exact_source: atlas_record::H8ExactSourceObject {
                    compact_json: format!(r#"{{"ordinal":{source_ordinal}}}"#),
                },
                reason: "table result has an unsupported identity".to_string(),
            })
        };
        let record = h8_fixture(
            key.clone(),
            RecordKind::RollTable,
            RecordBody::RollTable(RollTableRecord {
                identity: H8Identity {
                    record_key: key.clone(),
                    source_id: SourceDocumentId::new("h8-table").expect("source id"),
                    name: "H8 Table".to_string(),
                },
                description: FactValue::Missing,
                results: FactValue::Value(H8FieldValue::Known(vec![
                    unsupported_child(
                        0,
                        FactValue::Value(H8FieldValue::Known(
                            SourceDocumentId::new("broken-result").expect("result id"),
                        )),
                    ),
                    unsupported_child(
                        1,
                        FactValue::Value(H8FieldValue::Known(
                            SourceDocumentId::new("broken-result").expect("duplicate result id"),
                        )),
                    ),
                    unsupported_child(2, FactValue::Missing),
                    unsupported_child(3, FactValue::Null),
                    unsupported_child(
                        4,
                        FactValue::Value(H8FieldValue::Unsupported(UnsupportedSourceValue {
                            shape: UnsupportedSourceShape::String,
                            value: r#"""#.to_string(),
                            reason: UnsupportedSourceReason::SourceFieldDrift,
                        })),
                    ),
                ])),
                formula: FactValue::Missing,
                replacement: FactValue::Missing,
                display_roll: FactValue::Missing,
                image: FactValue::Missing,
                source_metadata: empty_h8_metadata(),
                content: OwnedRichContent::default(),
                unsupported_fields: Vec::new(),
                provenance: h8_provenance(),
            }),
        );

        let surface = surface(&record, None);
        let RecordSurfacePresentationView::RollTable { body } = &surface.presentation else {
            panic!("roll-table presentation");
        };
        let atlas_app_model::H8FactView::Known(results) = &body.results else {
            panic!("table results");
        };
        let unsupported = results
            .iter()
            .map(|entry| match entry {
                atlas_app_model::TableResultEntryView::Unsupported { unsupported } => unsupported,
                atlas_app_model::TableResultEntryView::Result { .. } => {
                    panic!("unsupported result")
                }
            })
            .collect::<Vec<_>>();
        assert!(unsupported.iter().all(|child| {
            child.identity_stability
                == atlas_app_model::H8IdentityStabilityView::UnstableAuthoredOrdinal
        }));
        assert!(matches!(
            &unsupported[0].source_id,
            atlas_app_model::H8FactView::Known(value) if value == "broken-result"
        ));
        assert!(matches!(
            &unsupported[1].source_id,
            atlas_app_model::H8FactView::Known(value) if value == "broken-result"
        ));
        assert!(matches!(
            &unsupported[2].source_id,
            atlas_app_model::H8FactView::Missing
        ));
        assert!(matches!(
            &unsupported[3].source_id,
            atlas_app_model::H8FactView::Null
        ));
        assert!(matches!(
            &unsupported[4].source_id,
            atlas_app_model::H8FactView::Unsupported(value)
                if value.shape == "string"
                    && value.exact_value == r#"""#
                    && value.reason == "source_field_drift"
        ));
        assert!(
            surface
                .issues
                .as_ref()
                .is_some_and(|issues| issues.iter().any(|issue| {
                    issue.code == RecordSurfaceIssueCodeView::Unsupported
                        && issue.message == "table result has an unsupported identity"
                }))
        );
    }

    #[test]
    fn spell_expectedness_keeps_optional_absence_quiet_and_unsupported_actionable() {
        for classification in [FactValue::Missing, FactValue::Null] {
            let record = spell_fixture(classification);
            let surface = surface(&record, None);
            assert_spell_metadata_has_no_classification(&surface);
            assert!(surface.issues.iter().flatten().all(|issue| {
                issue.placement != RecordSurfaceIssuePlacementView::Classification
            }));
        }

        let known_empty = SpellClassification {
            rank: known(2),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let record = spell_fixture(FactValue::Value(SpellSourceValue::Known(known_empty)));
        let known_empty_surface = surface(&record, None);
        assert_spell_metadata_has_no_classification(&known_empty_surface);
        assert!(known_empty_surface.issues.is_none());

        let record = spell_fixture(FactValue::Value(SpellSourceValue::Unsupported(
            unsupported_value(),
        )));
        let surface = surface(&record, None);
        assert_spell_metadata_has_no_classification(&surface);
        let issues = surface
            .issues
            .as_ref()
            .expect("unsupported base fact issue");
        assert_eq!(
            issues
                .iter()
                .filter(|issue| {
                    issue.code == RecordSurfaceIssueCodeView::Unsupported
                        && issue.placement == RecordSurfaceIssuePlacementView::Classification
                })
                .count(),
            1
        );
        assert!(issues.iter().all(|issue| {
            !issue.message.contains("/system/")
                && !issue.message.contains("spell-form:")
                && !issue.message.contains("unsupported-fixture")
        }));
        let RecordSurfacePresentationView::Spell { body } = &surface.presentation else {
            panic!("spell surface");
        };
        let SpellFormResultView::Available { definition } = &body.effective_form.result else {
            panic!("base Unsupported remains a localized field failure");
        };
        assert!(matches!(
            definition.classification,
            SpellResolvedFieldView::Unavailable { .. }
        ));
    }

    #[test]
    fn populated_spell_rule_and_resolved_unavailability_each_project_once() {
        let classification = SpellClassification {
            rank: known(2),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let mut record = spell_fixture(FactValue::Value(SpellSourceValue::Known(classification)));
        let Some(RecordBody::Spell(spell)) = record.body.as_mut() else {
            panic!("spell fixture body");
        };
        spell.definition.rules = known(vec![SpellRuleElement {
            authored_order: 0,
            source_path: "/system/rules/0".to_string(),
            authored_key: "FutureRule".to_string(),
            authored_object_json: "{}".to_string(),
            rule: SpellRule::Unsupported(SpellUnsupportedRule {
                authored_key: "FutureRule".to_string(),
                source_path: "/system/rules/0".to_string(),
                value: unsupported_value(),
            }),
        }]);
        let base_form = SpellFormId::base(&record.record.identity.key);
        let issues = surface(&record, Some((base_form, 1)))
            .issues
            .expect("rule and selection issues");
        assert_eq!(
            issues
                .iter()
                .filter(|issue| issue.placement == RecordSurfaceIssuePlacementView::Rules)
                .count(),
            1
        );
        assert_eq!(
            issues
                .iter()
                .filter(|issue| {
                    issue.code == RecordSurfaceIssueCodeView::Unavailable
                        && issue.placement == RecordSurfaceIssuePlacementView::Forms
                })
                .count(),
            1
        );
        assert!(issues.iter().all(|issue| !issue.message.contains('/')));
    }

    #[test]
    fn nested_spell_rank_predicate_and_suboption_issues_are_localized_once() {
        let classification = SpellClassification {
            rank: unsupported_fact(),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let mut record = spell_fixture(FactValue::Value(SpellSourceValue::Known(classification)));
        let Some(RecordBody::Spell(spell)) = record.body.as_mut() else {
            panic!("spell fixture body");
        };
        spell.definition.rules = known(vec![SpellRuleElement {
            authored_order: 0,
            source_path: "/system/rules/0".to_string(),
            authored_key: "RollOption".to_string(),
            authored_object_json: "{}".to_string(),
            rule: SpellRule::RollOption(SpellRollOptionRule {
                domain: known("all".to_string()),
                label: FactValue::Missing,
                option: known("fixture-option".to_string()),
                placement: known("spellcasting".to_string()),
                predicate: known(vec![SpellRulePredicate::Unsupported(
                    SpellUnsupportedRulePredicate {
                        source_path: "/system/rules/0/predicate/0".to_string(),
                        authored_key: None,
                        authored_order: 0,
                        value: unsupported_value(),
                    },
                )]),
                suboptions: known(vec![SpellRuleSuboption {
                    label: unsupported_fact(),
                    value: known("fire".to_string()),
                }]),
                toggleable: known(false),
            }),
        }]);

        let base_form = SpellFormId::base(&record.record.identity.key);
        let issues = surface(&record, Some((base_form, 1)))
            .issues
            .expect("nested unsupported spell facts should remain actionable");
        for (placement, message) in [
            (RecordSurfaceIssuePlacementView::Classification, "Rank"),
            (
                RecordSurfaceIssuePlacementView::Rules,
                "Roll option predicate",
            ),
            (
                RecordSurfaceIssuePlacementView::Rules,
                "Roll option suboption label",
            ),
        ] {
            assert_eq!(
                issues
                    .iter()
                    .filter(|issue| {
                        issue.code == RecordSurfaceIssueCodeView::Unsupported
                            && issue.placement == placement
                            && issue.message.contains(message)
                    })
                    .count(),
                1,
                "{message} should have one naturally placed issue"
            );
        }
        assert!(issues.iter().all(|issue| {
            !issue.message.contains("/system/")
                && !issue.message.contains("fixture-option")
                && !issue.message.contains("issue-source")
                && issue.message != "A resolved spell field is unavailable."
        }));
    }

    #[test]
    fn selected_overlay_uses_actual_resolved_field_issues_and_shared_source_notes() {
        let classification = SpellClassification {
            rank: known(2),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let mut record = spell_fixture(known(classification));
        let Some(RecordBody::Spell(spell)) = record.body.as_mut() else {
            panic!("spell fixture body");
        };
        spell
            .definition
            .unsupported_notes
            .push(SpellUnsupportedSourceFact {
                field: SpellUnsupportedSourceField::RitualMember,
                source_path: "/system/ritual/private".to_string(),
                authored_key: "private".to_string(),
                authored_order: Some(0),
                value: unsupported_value(),
            });
        spell.definition.heightening =
            known(SpellHeightening::Fixed(vec![SpellFixedHeighteningLayer {
                key: "5".to_string(),
                authored_order: 0,
                rank: SpellSourceValue::Known(5),
                patch: SpellPatch {
                    damage: known(SpellKeyedPatch {
                        members: vec![
                            SpellKeyedPatchMember {
                                key: "duplicate".to_string(),
                                authored_order: 0,
                                operation: SpellKeyedPatchOperation::Delete,
                            },
                            SpellKeyedPatchMember {
                                key: "duplicate".to_string(),
                                authored_order: 1,
                                operation: SpellKeyedPatchOperation::Delete,
                            },
                        ],
                    }),
                    ..SpellPatch::default()
                },
            }]));
        let overlay_id = SpellOverlayId::new("alternate").expect("overlay id");
        let overlay = SpellOverlay {
            key: overlay_id.as_str().to_string(),
            authored_order: 0,
            overlay_id: overlay_id.clone(),
            source_id: known(overlay_id.clone()),
            sort: known(0),
            name: known("Alternate".to_string()),
            overlay_type: known(SpellOverlayType::Override),
            patch: SpellPatch {
                classification: unsupported_fact(),
                ..SpellPatch::default()
            },
        };
        let form_id = overlay.form_id(&record.record.identity.key);
        let selected_form_id = form_id.as_str().to_string();
        spell.definition.overlays = known(vec![overlay]);

        let surface = surface(&record, Some((form_id, 5)));
        assert_spell_metadata_has_no_classification(&surface);
        let issues = surface
            .issues
            .as_ref()
            .expect("selected field and source note issues");
        for (code, placement, label) in [
            (
                RecordSurfaceIssueCodeView::Unavailable,
                RecordSurfaceIssuePlacementView::Damage,
                "Damage",
            ),
            (
                RecordSurfaceIssueCodeView::Unmodeled,
                RecordSurfaceIssuePlacementView::Ritual,
                "Ritual source fact",
            ),
        ] {
            assert_eq!(
                issues
                    .iter()
                    .filter(|issue| {
                        issue.code == code
                            && issue.placement == placement
                            && issue.message.contains(label)
                    })
                    .count(),
                1,
                "{label} should be reported exactly once"
            );
        }
        assert_eq!(
            issues
                .iter()
                .filter(|issue| {
                    issue.placement == RecordSurfaceIssuePlacementView::Classification
                })
                .count(),
            1,
            "selected classification failure should have one localized issue"
        );
        assert!(issues.iter().all(|issue| {
            !issue.message.contains("/system/")
                && !issue.message.contains("private")
                && !issue.message.contains("alternate")
        }));

        let RecordSurfacePresentationView::Spell { body } = &surface.presentation else {
            panic!("spell surface");
        };
        let SpellFormResultView::Available { definition } = &body.effective_form.result else {
            panic!("overall selected form remains available");
        };
        assert_eq!(body.effective_form.id, selected_form_id);
        assert!(matches!(
            definition.classification,
            SpellResolvedFieldView::Unavailable { .. }
        ));
        assert!(matches!(
            definition.casting,
            SpellResolvedFieldView::Available { .. }
        ));
        assert!(matches!(
            definition.damage,
            SpellResolvedFieldView::Unavailable { .. }
        ));
    }

    fn surface(
        record: &atlas_record::RetrievedRecord,
        selection: Option<(SpellFormId, u8)>,
    ) -> atlas_app_model::RecordSurfaceView {
        let remaster_lookup = VerifiedRemasterLookup::from_test_result(RemasterLinksResult {
            seed: record.clone(),
            links: Vec::new(),
        });
        crate::surface::record_surface(
            record,
            RecordSurfaceProfileView::RecordDetail,
            None,
            selection,
            &remaster_lookup,
        )
    }

    fn h8_fixture(
        key: RecordKey,
        kind: RecordKind,
        body: RecordBody,
    ) -> atlas_record::RetrievedRecord {
        let fixture = encounter_fixture_worker();
        let mut record = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");
        record.record.identity.key = key;
        record.record.identity.name = "H8 Fixture".to_string();
        record.record.classification.kind = kind;
        record.record.classification.level = None;
        record.record.classification.traits.clear();
        record.body = Some(body);
        record
    }

    fn h8_provenance() -> H8Provenance {
        H8Provenance {
            source_path: "packs/h8/fixture.json".to_string(),
            source_contract_version: "pf2e-serialized-source/v1".to_string(),
            source_system_version: "7.7.0".to_string(),
            source_upstream_commit: "fixture".to_string(),
        }
    }

    fn empty_h8_metadata() -> H8RecordSourceMetadata {
        H8RecordSourceMetadata {
            folder: FactValue::Missing,
            sort: FactValue::Missing,
            ownership: FactValue::Missing,
            flags: FactValue::Missing,
            stats: FactValue::Missing,
        }
    }

    fn spell_fixture(
        classification: atlas_record::SpellFact<SpellClassification>,
    ) -> atlas_record::RetrievedRecord {
        let fixture = encounter_fixture_worker();
        let key = RecordKey::parse("spells:expectednessFixture").expect("spell key");
        let mut record = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");
        record.record.identity.key = key.clone();
        record.record.identity.name = "Expectedness Fixture".to_string();
        record.record.classification.kind = RecordKind::Spell;
        record.record.classification.level = Some(99);
        record.record.classification.traits = vec!["stale-query-trait".to_string()];
        let mut spell = SpellRecord::new(
            SpellIdentity {
                record_key: key,
                source_id: SpellSourceId::new("expectedness-source").expect("source id"),
                name: "Expectedness Fixture".to_string(),
            },
            SpellProvenance {
                source_path: "fixture.json".to_string(),
                source_contract_version: "v1".to_string(),
                source_system_version: "7".to_string(),
                source_upstream_commit: "fixture".to_string(),
                standalone_location: FactValue::Missing,
            },
        );
        spell.definition.classification = classification;
        record.body = Some(RecordBody::Spell(spell));
        record
    }

    fn assert_spell_metadata_has_no_classification(surface: &atlas_app_model::RecordSurfaceView) {
        assert_eq!(surface.metadata.level, None);
        assert!(surface.metadata.traits.is_empty());
    }

    fn known<T>(value: T) -> atlas_record::SpellFact<T> {
        FactValue::Value(SpellSourceValue::Known(value))
    }

    fn unsupported_fact<T>() -> atlas_record::SpellFact<T> {
        FactValue::Value(SpellSourceValue::Unsupported(unsupported_value()))
    }

    fn unsupported_value() -> UnsupportedSourceValue {
        UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Object,
            value: "{}".to_string(),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        }
    }
}
