//! Application reads over canonical, checked-source glossary fixtures.
use super::*;
use atlas_domain::{DefinitionKey, DefinitionNamespace as N};
use atlas_search::test_support::{hash, input, record};
use atlas_search::{
    DefinitionReferenceInput, DefinitionReferenceSource, DefinitionReferenceTarget,
    ReferenceAvailability, ReferenceBacking, ReferenceCoverage, ReferenceEntry,
    ReferenceFilterBinding, ReferencePermission, ReferencePermissionDisposition,
    ReferenceProvenance, ReferenceRecordMapping,
};
use serde_json::json;

const HOLDER: &str = "actors:holder0000000001";
const GENERAL: &str = "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6";

fn metadata(
    namespace: N,
    token: &str,
    label: &str,
    localization: Option<&str>,
    field: &str,
) -> ReferenceEntry {
    ReferenceEntry {
        key: DefinitionKey::new(namespace, token).unwrap(),
        label: label.into(),
        availability: ReferenceAvailability::LabelOnly,
        backing: ReferenceBacking::None,
        provenance: ReferenceProvenance {
            source_package: "pf2e".into(),
            source_repository: None,
            source_revision: None,
            source_file: Some("fixtures/contextual-help.json".into()),
            source_sha256: Some(hash(b"contextual help fixture")),
            label_key: localization.map(str::to_owned),
            description_key: None,
            catalogs: vec!["fixture".into()],
            source_locations: vec![],
            indexing_locale: "en".into(),
            content_locale: Some("en".into()),
            publications: vec![],
            record_mapping: None,
            recipe: "contextual-help-fixture/v1".into(),
        },
        permission: ReferencePermission {
            disposition: ReferencePermissionDisposition::ApprovedContent,
            body_grants: vec![],
            label_grants: vec![],
            notices: vec![],
        },
        filter_bindings: vec![ReferenceFilterBinding {
            id: "property".into(),
            relation: "property".into(),
            field: Some(field.into()),
            value: Some(token.into()),
        }],
    }
}

fn fixture(complete: bool) -> crate::test_support::FixtureWorker {
    crate::test_support::fixture_worker_with_executor(
        crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
            let records = vec![
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"holder0000000001","type":"npc","name":"Holder","system":{"description":{"value":"<p>@UUID[Compendium.pf2e.bestiary-ability-glossary-srd.Item.qCCLZhnp2HhP3Ex6] @Localize[Fixture.Sense.Darkvision] @Localize[Fixture.Rule.Hidden]</p>"},"traits":{"value":["fire"],"rarity":"uncommon"}},"items":[
                        {"_id":"trait00000000001","type":"action","name":"Trait mention","system":{"description":{"value":"<p>@Localize[Fixture.Trait.Fire]</p>"}}},
                        {"_id":"domain0000000001","type":"action","name":"Domain mention","system":{"description":{"value":"<p>@Localize[Fixture.Domain.Air]</p>"}}},
                        {"_id":"ambig00000000001","type":"action","name":"Ambiguous mention","system":{"description":{"value":"<p>@Localize[Fixture.Trait.Fire] @Localize[Fixture.Domain.Air]</p>"}}},
                        {"_id":"general000000001","type":"action","name":"General vision 23","system":{"description":{"value":"<p>@Localize[Fixture.Sense.Darkvision] Actual DC 23. @Damage[0[fire]] @UUID[Compendium.pf2e.bestiary-ability-glossary-srd.Item.qCCLZhnp2HhP3Ex6]</p>"}}},
                        {"_id":"missing000000001","type":"action","name":"No general reference","system":{"description":{"value":"<p>Own actual values remain. @Localize[Fixture.Rule.Hidden]</p>"}}}
                    ]}),
                ),
                record(
                    "bestiary-ability-glossary-srd",
                    "Item",
                    json!({"_id":"qCCLZhnp2HhP3Ex6","type":"action","name":"General darkvision","system":{"description":{"value":"<p>Actual custom prose @Check[type:reflex|dc:0]</p>"}}}),
                ),
                record(
                    "armies",
                    "Actor",
                    json!({"_id":"army000000000001","type":"army","name":"Fire army","system":{"attributes":{"immunities":[{"type":"fire"}],"weaknesses":[{"type":"fire","value":3}],"resistances":[{"type":"fire","value":7}]}}}),
                ),
                record(
                    "ancestries",
                    "Item",
                    json!({"_id":"ancestry00000001","type":"ancestry","name":"Ancestry","system":{"size":"med","vision":"normal","languages":{"value":["common"]},"additionalLanguages":{"value":["draconic"]},"boosts":{"0":{"value":["str"]}},"flaws":{}}}),
                ),
                record(
                    "classes",
                    "Item",
                    json!({"_id":"class00000000001","type":"class","name":"Class","system":{"keyAbility":{"value":["str"]},"trainedSkills":{"value":["arcana"]}}}),
                ),
                record(
                    "deities",
                    "Item",
                    json!({"_id":"deity00000000001","type":"deity","name":"Deity","system":{"attribute":["wis"],"skill":["arcana"]}}),
                ),
                record(
                    "equipment",
                    "Item",
                    json!({"_id":"weapon0000000001","type":"weapon","name":"Alternate weapon","system":{"meleeUsage":{"traits":["fire"],"damage":{"die":"d6","type":"fire"},"group":"sword"}}}),
                ),
                record(
                    "equipment",
                    "Item",
                    json!({"_id":"armor00000000001","type":"armor","name":"Chain Mail","system":{"baseItem":"chain-mail"}}),
                ),
                record(
                    "spells",
                    "Item",
                    json!({"_id":"spell00000000001","type":"spell","name":"Changing spell","system":{"level":{"value":1},"traits":{"value":[],"traditions":[]},"overlays":{"fire":{"overlayType":"override","name":"Fire form","system":{"traits":{"value":["fire"]}}}},"heightening":{"type":"fixed","levels":{"3":{"traits":{"value":["fire"]}}}}}}),
                ),
            ];
            if !complete {
                return Ok(atlas_search::test_support::open_source_fixture(
                    records, false,
                )?);
            }
            let mut data = input(records, false);
            let mut entries = vec![
                metadata(
                    N::Trait,
                    "fire",
                    "Fire",
                    Some("Fixture.Trait.Fire"),
                    "traits",
                ),
                metadata(
                    N::Domain,
                    "air",
                    "Air",
                    Some("Fixture.Domain.Air"),
                    "deity.domains.primary",
                ),
                metadata(
                    N::Sense,
                    "darkvision",
                    "Darkvision",
                    Some("Fixture.Sense.Darkvision"),
                    "senses",
                ),
                metadata(
                    N::ImmunityType,
                    "fire",
                    "Fire immunity",
                    None,
                    "defenses.immunities",
                ),
                metadata(
                    N::WeaknessType,
                    "fire",
                    "Fire weakness",
                    None,
                    "defenses.weaknesses",
                ),
                metadata(
                    N::ResistanceType,
                    "fire",
                    "Fire resistance",
                    None,
                    "defenses.resistances",
                ),
            ];
            entries[2].backing = ReferenceBacking::Record {
                key: GENERAL.parse().unwrap(),
            };
            entries[2].availability = ReferenceAvailability::Available;
            entries[2].provenance.record_mapping = Some(ReferenceRecordMapping {
                target: GENERAL.parse().unwrap(),
                package: "pf2e".into(),
                expected_document_kind: "Item".into(),
                expected_item_type: Some("action".into()),
                required_localization_key: None,
            });
            entries[2].permission.disposition =
                ReferencePermissionDisposition::ExistingRecordPolicy;
            entries.push(metadata(
                N::Rarity,
                "uncommon",
                "Uncommon supplied",
                None,
                "rarity",
            ));
            let mut hidden = metadata(
                N::Rule,
                "pf2e:Fixture.Rule.Hidden",
                "Hidden rule",
                Some("Fixture.Rule.Hidden"),
                "unused",
            );
            hidden.availability = ReferenceAvailability::PermissionUnresolved;
            hidden.permission.disposition = ReferencePermissionDisposition::MetadataOnlyUnresolved;
            hidden.filter_bindings = vec![ReferenceFilterBinding {
                id: "references".into(),
                relation: "references_definition".into(),
                field: None,
                value: None,
            }];
            entries.push(hidden);
            let trait_entry = entries.remove(0);
            data.traits
                .push(atlas_search::test_support::TraitDefinition {
                    key: atlas_domain::TraitKey::new("fire").unwrap(),
                    label: trait_entry.label,
                    label_key: trait_entry.provenance.label_key,
                    description_key: None,
                    description_html: None,
                    authored_markup: None,
                    prepared: None,
                    catalogs: vec!["fixture".into()],
                    source_locations: vec![],
                    source_revision: None,
                    indexing_locale: "en".into(),
                    content_locale: Some("en".into()),
                });
            let (expected_entry_count, entry_keys_sha256) =
                atlas_search::test_support::reference_inventory_identity(
                    entries.iter().map(|e| e.key.clone()).chain(
                        data.traits
                            .iter()
                            .map(|t| DefinitionKey::from(t.key.clone())),
                    ),
                )?;
            data.context.reference_preparation =
                Some(atlas_search::test_support::ReferencePreparation {
                    source_package: Some("pf2e".into()),
                    pack_systems: data
                        .packs
                        .iter()
                        .map(|p| (p.pack_id.clone(), Some("pf2e".into())))
                        .collect(),
                    recipe: "contextual-help-fixture/v1".into(),
                    input_hash: hash(b"contextual help fixture"),
                    coverage: ReferenceCoverage::Complete,
                    expected_entry_count,
                    entry_keys_sha256,
                });
            let mut localizations = atlas_record::source_content::DefinitionLocalizationMap::new();
            for entry in &entries {
                if let Some(key) = &entry.provenance.label_key {
                    localizations
                        .entry(key.clone())
                        .or_default()
                        .insert(entry.key.clone());
                }
            }
            for entry in &data.traits {
                if let Some(key) = &entry.label_key {
                    localizations
                        .entry(key.clone())
                        .or_default()
                        .insert(DefinitionKey::from(entry.key.clone()));
                }
            }
            let mut targets = entries
                .iter()
                .map(|entry| {
                    (
                        entry.key.clone(),
                        match &entry.backing {
                            ReferenceBacking::Record { key } => DefinitionReferenceTarget::Record {
                                key: key.clone(),
                                owners: vec![],
                            },
                            _ => DefinitionReferenceTarget::Definition {
                                key: entry.key.clone(),
                            },
                        },
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            for entry in &data.traits {
                let key = DefinitionKey::from(entry.key.clone());
                targets.insert(key.clone(), DefinitionReferenceTarget::Definition { key });
            }
            let occurrences = data
                .records
                .iter()
                .flat_map(|source| {
                    atlas_record::source_record::definition_record_localization_references(
                        &source.record,
                        data.context.audience,
                        &localizations,
                    )
                })
                .map(|reference| DefinitionReferenceInput {
                    source: DefinitionReferenceSource::Record {
                        key: reference.locator.record,
                        owners: reference.locator.owners,
                        field: Some(reference.locator.field),
                    },
                    target: targets[&reference.key].clone(),
                    occurrence_path: reference.path,
                    evidence: reference.localization_key,
                    label: None,
                    visible: reference.visible,
                })
                .collect::<Vec<_>>();
            Ok(
                atlas_search::test_support::open_input_fixture_with_references(
                    data,
                    &entries,
                    &[],
                    &occurrences,
                )?,
            )
        }),
    )
}

#[test]
fn exact_owned_general_help_rejects_unrelated_traits_domains_and_ambiguity() {
    let fixture = fixture(true);
    let detail = fixture.worker.record_detail(HOLDER).unwrap();
    let RecordBodyView::Creature(actor) = detail.presentation.body else {
        panic!("actor");
    };
    let activities = actor.activities.value.unwrap();
    for title in [
        "Trait mention",
        "Domain mention",
        "Ambiguous mention",
        "No general reference",
    ] {
        assert!(
            activities
                .iter()
                .find(|a| a.title == title)
                .unwrap()
                .help
                .is_none(),
            "{title}"
        );
    }
    let general = activities
        .iter()
        .find(|a| a.title == "General vision 23")
        .unwrap();
    let Some(ReferenceHelpView {
        destination: Some(ReferenceDestinationView::Record { navigation, .. }),
        ..
    }) = &general.help
    else {
        panic!("verified general record");
    };
    assert_eq!(navigation.record_key, GENERAL);
    assert!(
        !general.navigation.owners.is_empty(),
        "the actual owned entry stays primary"
    );
    let badge = &detail.record.traits[0];
    assert_eq!(badge.value, "fire");
}

#[test]
fn legacy_trait_help_retains_membership_without_narrowing_symmetric_references() {
    let fixture = fixture(true);
    let detail = fixture.worker.record_detail(HOLDER).unwrap();
    let help = detail.presentation.identity.traits[0]
        .help
        .as_ref()
        .unwrap();
    assert_eq!(help.related_filters.len(), 1);
    let ReferenceFilterActionView::Search { predicate, .. } = &help.related_filters[0] else {
        panic!("membership filter");
    };
    assert_eq!(
        serde_json::to_value(predicate).unwrap(),
        json!({"kind":"set_match","field":"traits","op":"includes","values":["fire"]})
    );
    assert_eq!(help.availability, DefinitionAvailabilityView::LabelOnly);
}

#[test]
fn alternate_melee_and_spell_patch_traits_keep_contextual_help_and_raw_identity() {
    let fixture = fixture(true);
    let assert_fire = |value: &ReferenceValueView| {
        assert_eq!(value.value, "fire");
        assert_eq!(value.label, "Fire");
        assert_eq!(
            value.help.as_ref().unwrap().definition_key.as_deref(),
            Some("trait:fire")
        );
    };
    let detail = fixture
        .worker
        .record_detail("equipment:weapon0000000001")
        .unwrap();
    let RecordBodyView::PhysicalReference(physical) = detail.presentation.body else {
        panic!("weapon")
    };
    let Some(PhysicalDetailsView::Weapon(weapon)) = physical.details else {
        panic!("weapon details")
    };
    assert_fire(&weapon.melee_usage.value.unwrap().traits.value.unwrap()[0]);
    let detail = fixture
        .worker
        .record_detail("spells:spell00000000001")
        .unwrap();
    let RecordBodyView::Spell(spell) = detail.presentation.body else {
        panic!("spell")
    };
    assert_fire(
        &spell.forms.value.unwrap()[0]
            .changes
            .value
            .as_ref()
            .unwrap()
            .traits
            .value
            .as_ref()
            .unwrap()[0],
    );
    let Some(SpellHeighteningView::Fixed { levels }) = spell.heightening.value else {
        panic!("fixed levels")
    };
    assert_fire(
        &levels.value.unwrap()[0]
            .changes
            .value
            .as_ref()
            .unwrap()
            .traits
            .value
            .as_ref()
            .unwrap()[0],
    );
}

fn list_request(filter: Option<atlas_domain::QueryPredicate>) -> SearchResultsRequest {
    SearchResultsRequest {
        criteria: SearchCriteria::ListRecords { filter },
        page: SearchPageRequest {
            number: 1,
            size: 100,
        },
        sort: None,
        ranking: None,
        search_profile: None,
        expected_artifact_generation: None,
    }
}

#[test]
fn permission_blocked_body_retains_definition_identity_and_executable_reference_action() {
    let fixture = fixture(true);
    let execution = crate::ReadExecutionContext::uncancellable();
    let detail = fixture
        .worker
        .definition_detail(
            DefinitionDetailRequest {
                definition_key: "rule:pf2e:Fixture.Rule.Hidden".into(),
                expected_artifact_generation: None,
            },
            &execution,
        )
        .unwrap();
    assert!(detail.content.is_none());
    assert_eq!(
        detail.help.availability,
        DefinitionAvailabilityView::PermissionUnresolved
    );
    let ReferenceFilterActionView::Search {
        predicate,
        artifact_generation,
        ..
    } = &detail.help.related_filters[0]
    else {
        panic!("reference action")
    };
    let mut request = list_request(Some(predicate.clone()));
    request.expected_artifact_generation = Some(artifact_generation.clone());
    let result = fixture.worker.search_page(request, &execution).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].record.record_key, HOLDER);
    assert!(result.rows[0].relationship_evidence[0].occurrences.iter().any(|occurrence| matches!(occurrence, ReferenceOccurrenceView::Definition { occurrence } if matches!(&occurrence.target, ReferenceDestinationView::Definition { key } if key == "rule:pf2e:Fixture.Rule.Hidden"))));
}

#[test]
fn browse_summary_uses_rust_fallback_then_canonical_rarity_help() {
    let execution = crate::ReadExecutionContext::uncancellable();
    for (complete, label) in [(false, "Uncommon"), (true, "Uncommon supplied")] {
        let fixture = fixture(complete);
        let page = fixture
            .worker
            .search_page(list_request(None), &execution)
            .unwrap();
        let rarity = page
            .rows
            .iter()
            .find(|row| row.record.record_key == HOLDER)
            .unwrap()
            .record
            .rarity
            .as_ref()
            .unwrap();
        assert_eq!(rarity.value, "uncommon");
        assert_eq!(rarity.label, label);
        assert_eq!(
            rarity.help.as_ref().unwrap().availability,
            if complete {
                DefinitionAvailabilityView::LabelOnly
            } else {
                DefinitionAvailabilityView::SetupRequired
            }
        );
    }
}

#[test]
fn dynamic_filter_values_and_vocabulary_supply_existing_base_type_labels() {
    let fixture = fixture(true);
    let execution = crate::ReadExecutionContext::uncancellable();
    let response = fixture
        .worker
        .discover_filter_values(
            DiscoverFilterValuesRequest::Vocabulary {
                scope: FilterVocabularyScope::Artifact,
                field_id: "physical.base_type".into(),
                text: None,
                offset: 0,
                limit: 100,
                selected_values: vec![],
            },
            &execution,
        )
        .unwrap();
    let FilterValueListView::Vocabulary { values, .. } = response else {
        panic!("vocabulary")
    };
    let option = values
        .options
        .iter()
        .find(|option| option.value == atlas_domain::QueryLiteral::String("chain-mail".into()))
        .unwrap();
    assert_eq!(option.label, "Chain Mail");
    let response = fixture
        .worker
        .discover_filter_values(
            DiscoverFilterValuesRequest::Contextual {
                only_matching: false,
                context: Box::new(FilterDiscoveryContext::Filtered {
                    search_content: None,
                    search_profile: None,
                    expected_artifact_generation: None,
                    filter: None,
                    text: None,
                    mode: RetrievalModeView::Lexical,
                }),
                field_id: "physical.base_type".into(),
                clause_id: None,
                text: None,
                offset: 0,
                limit: 100,
            },
            &execution,
        )
        .unwrap();
    let FilterValueListView::Contextual {
        outcome: atlas_domain::QueryFacetOutcome::Available { value: values },
        ..
    } = response
    else {
        panic!("contextual values")
    };
    let option = values
        .options
        .iter()
        .find(|option| option.value == atlas_domain::QueryLiteral::String("chain-mail".into()))
        .unwrap();
    assert_eq!(option.label, "Chain Mail");
}

#[test]
fn definition_reference_evidence_keeps_general_purpose_for_both_reference_origins() {
    let fixture = fixture(true);
    let execution = crate::ReadExecutionContext::uncancellable();
    let predicate = atlas_domain::QueryPredicate {
        clause_id: Some("definition-purpose".into()),
        expression: atlas_domain::QueryExpression::DefinitionReference {
            reference: atlas_domain::QueryDefinitionReference {
                target: atlas_domain::QueryDefinitionReferenceTarget::Record {
                    selection: Box::new(atlas_domain::QueryRecordSelection {
                        record_key: GENERAL.parse().unwrap(),
                        owners: vec![],
                        field: None,
                        passage: None,
                        source_fingerprint: None,
                        spell_rank: None,
                    }),
                },
                include_contents: true,
            },
        },
    };
    let result = fixture
        .worker
        .search_page(list_request(Some(predicate)), &execution)
        .unwrap();
    let row = result
        .rows
        .iter()
        .find(|row| row.record.record_key == HOLDER)
        .unwrap();
    let occurrences = &row.relationship_evidence[0].occurrences;
    assert!(occurrences.iter().any(|occurrence| matches!(occurrence, ReferenceOccurrenceView::Record { destination, .. } if matches!(destination.as_ref(), ReferenceDestinationView::Record { reading_context: Some(RecordReadingContextView::GeneralDefinition), navigation } if navigation.record_key == GENERAL))));
    assert!(occurrences.iter().any(|occurrence| matches!(occurrence, ReferenceOccurrenceView::Definition { occurrence } if matches!(&occurrence.target, ReferenceDestinationView::Record { reading_context: Some(RecordReadingContextView::GeneralDefinition), navigation } if navigation.record_key == GENERAL))));
}

#[test]
fn army_iwr_uses_the_same_definition_labels_and_filters_as_creatures() {
    let fixture = fixture(true);
    let detail = fixture
        .worker
        .record_detail("armies:army000000000001")
        .unwrap();
    let RecordBodyView::Army(army) = detail.presentation.body else {
        panic!("army");
    };
    for (fact, namespace, label) in [
        (&army.immunities, "immunity_type", "Fire immunity"),
        (&army.weaknesses, "weakness_type", "Fire weakness"),
        (&army.resistances, "resistance_type", "Fire resistance"),
    ] {
        let value = fact.value.as_ref().unwrap()[0]
            .damage_type
            .value
            .as_ref()
            .unwrap();
        assert_eq!(value.value, "fire");
        assert_eq!(value.label, label);
        assert_eq!(
            value.help.as_ref().unwrap().definition_key.as_deref(),
            Some(format!("{namespace}:fire").as_str())
        );
        assert_eq!(value.help.as_ref().unwrap().related_filters.len(), 1);
    }
    assert_eq!(
        army.resistances.value.as_ref().unwrap()[0].magnitude.value,
        Some(7.into())
    );
}

#[test]
fn retained_profile_labels_and_normal_vision_preserve_readability_and_raw_identity() {
    for complete in [false, true] {
        let fixture = fixture(complete);
        let detail = fixture
            .worker
            .record_detail("ancestries:ancestry00000001")
            .unwrap();
        let RecordBodyView::Ancestry(ancestry) = detail.presentation.body else {
            panic!("ancestry");
        };
        let vision = ancestry.vision.value.unwrap();
        assert_eq!(
            (vision.value.as_str(), vision.label.as_str()),
            ("normal", "Normal")
        );
        assert!(vision.help.unwrap().destination.is_none());
        let language = &ancestry.languages.value.as_ref().unwrap()[0];
        assert_eq!(
            (language.value.as_str(), language.label.as_str()),
            ("common", "Common")
        );
        let boost = ancestry.boosts.value.as_ref().unwrap()[0]
            .choices
            .value
            .as_ref()
            .unwrap()[0]
            .clone();
        assert_eq!(
            (boost.value.as_str(), boost.label.as_str()),
            ("str", "Strength")
        );
        let detail = fixture
            .worker
            .record_detail("classes:class00000000001")
            .unwrap();
        let RecordBodyView::Class(class) = detail.presentation.body else {
            panic!("class");
        };
        assert_eq!(class.key_attributes.value.unwrap()[0].label, "Strength");
        assert_eq!(class.trained_skills.value.unwrap()[0].label, "Arcana");
        let detail = fixture
            .worker
            .record_detail("deities:deity00000000001")
            .unwrap();
        let RecordBodyView::Deity(deity) = detail.presentation.body else {
            panic!("deity");
        };
        assert_eq!(deity.attributes.value.unwrap()[0].label, "Wisdom");
        assert_eq!(deity.skills.value.unwrap()[0].label, "Arcana");
    }
}

#[test]
fn noncatalog_selected_owners_keep_identity_and_labels_without_glossary_promotion() {
    let fixture = fixture(true);
    let detail = fixture.worker.record_detail(HOLDER).unwrap();
    let RecordBodyView::Creature(actor) = &detail.presentation.body else {
        panic!("actor");
    };
    let selected = crate::relationship_search::query_selection(
        &actor
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .find(|a| a.title == "General vision 23")
            .unwrap()
            .navigation,
    )
    .unwrap();
    let target = atlas_domain::QueryDefinitionReferenceTarget::Record {
        selection: Box::new(selected),
    };
    let options = fixture
        .worker
        .definition_filter_options(
            DefinitionFilterOptionsRequest {
                text: None,
                offset: 0,
                limit: 100,
                selected: vec![target.clone()],
                expected_artifact_generation: Some(detail.artifact_generation),
            },
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    let selected = &options.selected[0];
    assert_eq!(selected.target, target);
    assert_eq!(selected.label, "General vision 23");
    assert_eq!(
        selected.help.availability,
        DefinitionAvailabilityView::NotInCatalog
    );
    assert!(selected.help.destination.is_none());
    assert!(selected.help.related_filters.is_empty());
}

#[test]
fn general_context_preserves_custom_and_owned_values_when_no_template_is_verified() {
    let fixture = fixture(true);
    let request =
        |key: &str, owners: Vec<atlas_domain::OwnedContentLocator>, context| RecordDetailRequest {
            record_key: key.into(),
            owners,
            reading_context: context,
            expected_artifact_generation: None,
            spell_rank: None,
            fields: vec![],
            passage: None,
            source_fingerprint: None,
        };
    let ordinary = fixture
        .worker
        .record_detail_at(request(GENERAL, vec![], None))
        .unwrap();
    let contextual = fixture
        .worker
        .record_detail_at(request(
            GENERAL,
            vec![],
            Some(RecordReadingContextView::GeneralDefinition),
        ))
        .unwrap();
    assert_eq!(
        ordinary.presentation.content,
        contextual.presentation.content
    );
    assert!(contextual.presentation.general_definition.is_none());
    let detail = fixture.worker.record_detail(HOLDER).unwrap();
    let RecordBodyView::Creature(actor) = detail.presentation.body else {
        panic!("actor");
    };
    let navigation = actor
        .activities
        .value
        .unwrap()
        .into_iter()
        .find(|a| a.title == "General vision 23")
        .unwrap()
        .navigation;
    let ordinary = fixture
        .worker
        .record_detail_at(request(HOLDER, navigation.owners.clone(), None))
        .unwrap();
    let contextual = fixture
        .worker
        .record_detail_at(request(
            HOLDER,
            navigation.owners,
            Some(RecordReadingContextView::GeneralDefinition),
        ))
        .unwrap();
    assert_eq!(
        ordinary.presentation.content,
        contextual.presentation.content
    );
    assert!(contextual.presentation.general_definition.is_none());
}
