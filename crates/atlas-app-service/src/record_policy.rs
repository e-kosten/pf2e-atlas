use std::collections::BTreeSet;

use atlas_app_model::{
    CreatureSurfaceDomainUnavailableView, CreatureSurfaceUnavailableDomainsView,
    CreatureSurfaceUnavailableFieldView, CreatureSurfaceUnavailableStateView,
    HazardSurfaceUnavailableStateView, HazardSurfaceView, RecordSurfaceIssueCodeView,
    RecordSurfaceIssuePlacementView, RecordSurfaceIssueView, RecordSurfacePresentationView,
};
use atlas_record::{
    FactIssueKind, FactPresentationDisposition, FactPresentationRole, FactPresentationState,
    FactRequirement, HazardSourceMetadataField, HazardSourceMetadataIssueKind, RecordBody,
    RetrievedRecord, SpellPresentationIssue, SpellPresentationIssuePlacement,
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
        (RecordSurfacePresentationView::Unavailable { .. }, _) => {
            issues.push(RecordSurfaceIssueView {
                code: RecordSurfaceIssueCodeView::Unavailable,
                placement: RecordSurfaceIssuePlacementView::Record,
                message: "This record does not have an available typed presentation.".to_string(),
            })
        }
        _ => {}
    }
    (!issues.is_empty()).then_some(issues)
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
            message: issue.message().to_string(),
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
        push_policy_issue(
            issues,
            FactPresentationRole::Gameplay,
            state,
            FactRequirement::Optional,
            if unavailable.component_id.is_some() {
                RecordSurfaceIssuePlacementView::Activity
            } else {
                RecordSurfaceIssuePlacementView::Record
            },
            &unavailable.message,
        );
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
                code: issue_code(issue.kind),
                placement: spell_issue_placement(issue.placement()),
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
        code: issue_code(kind),
        placement,
        message: message.to_string(),
    });
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
        FactValue, RecordBody, SpellClassification, SpellFixedHeighteningLayer, SpellFormId,
        SpellHeightening, SpellIdentity, SpellKeyedPatch, SpellKeyedPatchMember,
        SpellKeyedPatchOperation, SpellOverlay, SpellOverlayId, SpellOverlayType, SpellPatch,
        SpellProvenance, SpellRecord, SpellRollOptionRule, SpellRule, SpellRuleElement,
        SpellRulePredicate, SpellRuleSuboption, SpellSourceId, SpellSourceValue,
        SpellUnsupportedRule, SpellUnsupportedRulePredicate, SpellUnsupportedSourceFact,
        SpellUnsupportedSourceField, UnsupportedSourceReason, UnsupportedSourceShape,
        UnsupportedSourceValue,
    };
    use atlas_search::RemasterLinksResult;

    use crate::retrieval::VerifiedRemasterLookup;
    use crate::test_support::encounter_fixture_worker;

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
