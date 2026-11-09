//! Application reads over canonical, checked-source glossary fixtures.
use super::*;
use atlas_domain::{DefinitionKey, DefinitionNamespace as N};
use atlas_search::test_support::{
    ReferenceAvailability, ReferenceGrantScope, ReferenceGrantUse, ReferencePermission,
    ReferencePermissionDisposition, ReferenceProvenance, ReferenceRecordMapping, hash, input,
    record,
};
use atlas_search::{ReferenceBacking, ReferenceCoverage, ReferenceEntry};
use serde_json::json;

const HOLDER: &str = "actors:holder0000000001";
const GENERAL: &str = "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6";

fn metadata(namespace: N, token: &str, label: &str, localization: Option<&str>) -> ReferenceEntry {
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
    }
}

fn fixture(complete: bool) -> crate::test_support::FixtureWorker {
    crate::test_support::fixture_worker_with_executor(
        crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
            let records = vec![
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"holder0000000001","type":"npc","name":"Holder","system":{"details":{"publication":{"remaster":true}},"description":{"value":"<p>@UUID[Compendium.pf2e.bestiary-ability-glossary-srd.Item.qCCLZhnp2HhP3Ex6] @Localize[Fixture.Sense.Darkvision] @Localize[PF2E.NPC.Abilities.Glossary.FixtureHidden]</p>"},"traits":{"value":["fire","empty-fixture","illusion"],"rarity":"uncommon","size":{"value":"lg"}}},"items":[
                        {"_id":"trait00000000001","type":"action","name":"Trait mention","system":{"description":{"value":"<p>@Localize[Fixture.Trait.Fire]</p>"}}},
                        {"_id":"domain0000000001","type":"action","name":"Domain mention","system":{"description":{"value":"<p>@Localize[Fixture.Domain.Air]</p>"}}},
                        {"_id":"ambig00000000001","type":"action","name":"Ambiguous mention","system":{"description":{"value":"<p>@Localize[Fixture.Trait.Fire] @Localize[Fixture.Sense.Darkvision]</p>"}}},
                        {"_id":"general000000001","type":"action","name":"General vision 23","system":{"description":{"value":"<p>@Localize[Fixture.Sense.Darkvision] Actual DC 23. @Damage[0[fire]] @UUID[Compendium.pf2e.bestiary-ability-glossary-srd.Item.qCCLZhnp2HhP3Ex6]</p>"}}},
                        {"_id":"missing000000001","type":"action","name":"No general reference","system":{"description":{"value":"<p>Own actual values remain. @Localize[PF2E.NPC.Abilities.Glossary.FixtureHidden]</p>"}}}
                    ]}),
                ),
                record(
                    "bestiary-ability-glossary-srd",
                    "Item",
                    json!({"_id":"qCCLZhnp2HhP3Ex6","type":"action","name":"General darkvision","system":{"publication":{"remaster":false},"traits":{"value":["fire"]},"description":{"value":"<p>Actual custom prose @Check[type:reflex|dc:0]</p>"}}}),
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
                    json!({"_id":"weapon0000000001","type":"weapon","name":"Alternate weapon","system":{"size":"lg","meleeUsage":{"traits":["fire"],"damage":{"die":"d6","type":"fire"},"group":"sword"}}}),
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
                record(
                    "actions",
                    "Item",
                    json!({"_id":"check00000000001","type":"action","name":"Trait check","system":{"description":{"value":"<p>@Check[will|dc:20|traits:illusion,fire,homebrew-trait] save.</p>"}}}),
                ),
                record(
                    "hazards",
                    "Actor",
                    json!({"_id":"hazard0000000001","type":"hazard","name":"Trait routine","system":{"details":{"isComplex":true,"routine":"<p>(7 actions; illusion, fire, homebrew-trait) @Check[will|dc:21|traits:illusion,fire,homebrew-trait] save. Keep the authored routine.</p>"}},"items":[{"_id":"hazardact0000001","type":"action","name":"Hazard vision","system":{"description":{"value":"<p>@Localize[PF2E.NPC.Abilities.Glossary.FixtureVision]</p>"}}}]}),
                ),
            ];
            let mut data = input(records, false);
            let empty_markup = "<p></p>";
            let empty_prepared = atlas_record::source_content::prepare_reference_definition(
                DefinitionKey::new(N::Trait, "empty-fixture").unwrap(),
                empty_markup,
                data.context.audience,
                None,
                None,
            )
            .unwrap();
            data.traits
                .push(atlas_search::test_support::TraitDefinition {
                    key: atlas_domain::TraitKey::new("empty-fixture").unwrap(),
                    label: "Empty fixture".into(),
                    label_key: None,
                    description_key: None,
                    description_html: Some(empty_prepared.html.clone()),
                    authored_markup: Some(empty_markup.into()),
                    prepared: Some(empty_prepared),
                    catalogs: vec!["fixture".into()],
                    source_locations: vec![],
                    source_revision: None,
                    indexing_locale: "en".into(),
                    content_locale: Some("en".into()),
                });
            let prose = "<p>Fixture illusion definition.</p>";
            let prepared = atlas_record::source_content::prepare_reference_definition(
                DefinitionKey::new(N::Trait, "illusion").unwrap(),
                prose,
                data.context.audience,
                None,
                None,
            )
            .unwrap();
            data.traits
                .push(atlas_search::test_support::TraitDefinition {
                    key: atlas_domain::TraitKey::new("illusion").unwrap(),
                    label: "Illusion supplied".into(),
                    label_key: None,
                    description_key: None,
                    description_html: Some(prepared.html.clone()),
                    authored_markup: Some(prose.into()),
                    prepared: Some(prepared),
                    catalogs: vec!["fixture".into()],
                    source_locations: vec![],
                    source_revision: None,
                    indexing_locale: "en".into(),
                    content_locale: Some("en".into()),
                });
            if !complete {
                return Ok(atlas_search::test_support::open_input_fixture(data)?);
            }
            let mut entries = vec![
                metadata(N::Trait, "fire", "Fire", Some("Fixture.Trait.Fire")),
                metadata(N::Domain, "air", "Air", Some("Fixture.Domain.Air")),
                metadata(
                    N::Sense,
                    "darkvision",
                    "Darkvision",
                    Some("Fixture.Sense.Darkvision"),
                ),
                metadata(N::ImmunityType, "fire", "Fire immunity", None),
                metadata(N::WeaknessType, "fire", "Fire weakness", None),
                metadata(N::ResistanceType, "fire", "Fire resistance", None),
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
            let air = &mut entries[1];
            air.backing = ReferenceBacking::Prepared {
                authored_markup: empty_markup.into(),
                content: atlas_record::source_content::prepare_reference_definition(
                    air.key.clone(),
                    empty_markup,
                    data.context.audience,
                    None,
                    None,
                )
                .unwrap(),
            };
            air.availability = ReferenceAvailability::Available;
            air.provenance.source_sha256 = Some(hash(empty_markup.as_bytes()));
            air.permission.body_grants = vec![ReferenceGrantUse {
                license: "CC0-1.0".into(),
                scope: ReferenceGrantScope::FunctionalContent,
            }];
            air.permission.notices = vec!["fixture-empty-content".into()];
            let localization = "PF2E.NPC.Abilities.Glossary.FixtureVision";
            let markup = "<p>Fixture hazard vision rule.</p>";
            let mut rule = air.clone();
            rule.key = DefinitionKey::new(N::Rule, format!("pf2e:{localization}")).unwrap();
            rule.label = "Fixture vision".into();
            rule.provenance.label_key = Some(localization.into());
            rule.provenance.source_sha256 = Some(hash(markup.as_bytes()));
            rule.backing = ReferenceBacking::Prepared {
                authored_markup: markup.into(),
                content: atlas_record::source_content::prepare_reference_definition(
                    rule.key.clone(),
                    markup,
                    data.context.audience,
                    None,
                    None,
                )
                .unwrap(),
            };
            entries.push(rule);
            entries.push(metadata(N::Rarity, "uncommon", "Uncommon supplied", None));
            let size = metadata(N::Size, "med", "Medium", None);
            let mut large = size.clone();
            large.key = DefinitionKey::new(N::Size, "lg").unwrap();
            large.label = "Large supplied".into();
            entries.push(size);
            entries.push(large);
            let mut missing_language = metadata(N::Language, "draconic", "Draconic", None);
            missing_language.availability = ReferenceAvailability::MissingLocalizedBody;
            entries.push(missing_language);
            let mut hidden = metadata(
                N::Rule,
                "pf2e:PF2E.NPC.Abilities.Glossary.FixtureHidden",
                "Hidden rule",
                Some("PF2E.NPC.Abilities.Glossary.FixtureHidden"),
            );
            hidden.availability = ReferenceAvailability::PermissionUnresolved;
            hidden.permission.disposition = ReferencePermissionDisposition::MetadataOnlyUnresolved;
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
            Ok(
                atlas_search::test_support::open_input_fixture_with_references(
                    data,
                    &entries,
                    &[atlas_search::ReferenceNotice {
                        id: "fixture-empty-content".into(),
                        sha256: hash(b"Synthetic empty glossary fixture."),
                        text: "Synthetic empty glossary fixture.".into(),
                        source_url: None,
                    }],
                )?,
            )
        }),
    )
}

fn selected_activity_detail(
    fixture: &crate::test_support::FixtureWorker,
    navigation: RecordNavigationView,
) -> RecordDetailView {
    fixture
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: navigation.record_key,
            owners: navigation.owners,
            expected_artifact_generation: None,
            spell_rank: navigation.spell_rank,
            fields: vec![],
            passage: navigation.passage,
            source_fingerprint: navigation.source_fingerprint,
        })
        .unwrap()
}

#[test]
fn general_help_is_only_projected_for_the_exact_selected_activity() {
    let fixture = fixture(true);
    let actor = fixture.worker.record_detail(HOLDER).unwrap();
    let hazard = fixture
        .worker
        .record_detail("hazards:hazard0000000001")
        .unwrap();
    let RecordBodyView::Creature(actor) = actor.presentation.body else {
        panic!("actor");
    };
    let RecordBodyView::Hazard(hazard) = hazard.presentation.body else {
        panic!("hazard");
    };
    let actor_activities = actor.activities.value.unwrap();
    let hazard_activities = hazard.actor.activities.value.unwrap();
    assert_eq!(actor_activities.len(), 5);
    assert_eq!(hazard_activities.len(), 1);
    for activity in actor_activities.into_iter().chain(hazard_activities) {
        let wire = serde_json::to_value(&activity).unwrap();
        assert!(wire.get("help").is_none(), "{}", activity.title);
        let navigation = activity.navigation;
        assert!(!navigation.owners.is_empty());
        let selected = selected_activity_detail(&fixture, navigation.clone());
        assert_eq!(selected.selected.owners, navigation.owners);
        let RecordBodyView::Activity(selected) = selected.presentation.body else {
            panic!("selected activity");
        };
        match activity.title.as_str() {
            "General vision 23" => assert!(matches!(
                &selected.help,
                Some(ReferenceHelpView {
                    destination: ReferenceDestinationView::Record { navigation },
                    ..
                }) if navigation.record_key == GENERAL && navigation.owners.is_empty()
            )),
            "Hazard vision" => assert!(matches!(
                &selected.help,
                Some(ReferenceHelpView {
                    definition_key: Some(key),
                    destination: ReferenceDestinationView::Definition { key: destination },
                }) if key == "rule:pf2e:PF2E.NPC.Abilities.Glossary.FixtureVision" && key == destination
            )),
            "Trait mention" | "Domain mention" | "Ambiguous mention" | "No general reference" => {
                assert!(selected.help.is_none(), "{}", activity.title);
            }
            title => panic!("unexpected fixture activity {title}"),
        }
    }
}

#[test]
fn alternate_melee_and_spell_patch_traits_keep_labels_and_raw_identity() {
    let fixture = fixture(true);
    let assert_fire = |value: &ReferenceValueView| {
        assert_eq!(value.value, "fire");
        assert_eq!(value.label, "Fire");
        assert!(value.help.is_none());
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
        connection_scope: None,
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
fn bodyless_entries_keep_typed_filters_and_record_backed_help_retains_associations() {
    let fixture = fixture(true);
    let execution = crate::ReadExecutionContext::uncancellable();
    let read = |key: &str| {
        fixture
            .worker
            .definition_detail(
                DefinitionDetailRequest {
                    definition_key: key.into(),
                    expected_artifact_generation: None,
                },
                &execution,
            )
            .unwrap()
    };
    for key in ["size:med", "weakness_type:fire", "language:draconic"] {
        let detail = read(key);
        assert!(detail.help.is_none(), "{key}");
        assert!(detail.content.is_none(), "{key}");
        assert!(detail.related_filters.is_empty(), "{key}");
    }
    let page = fixture
        .worker
        .search_page(
            list_request(Some(atlas_domain::QueryPredicate::new(
                atlas_domain::QueryExpression::Exists {
                    collection: "defenses.weaknesses".into(),
                    scope_id: "weakness".into(),
                    predicate: Box::new(atlas_domain::QueryPredicate::new(
                        atlas_domain::QueryExpression::Compare {
                            field: "type".into(),
                            op: atlas_domain::QueryCompare::Eq,
                            value: atlas_domain::QueryLiteral::String("fire".into()),
                        },
                    )),
                },
            ))),
            &execution,
        )
        .unwrap();
    assert_eq!(page.rows[0].record.record_key, "armies:army000000000001");
    let darkvision = read("sense:darkvision");
    assert!(matches!(
        darkvision.help.map(|help| help.destination),
        Some(ReferenceDestinationView::Record { navigation }) if navigation.record_key == GENERAL
    ));
    assert_eq!(darkvision.related_filters[0].label, "With this sense");
}

#[test]
fn retained_empty_traits_have_membership_without_reading_pages() {
    let execution = crate::ReadExecutionContext::uncancellable();
    for complete in [false, true] {
        let fixture = fixture(complete);
        let specialized = fixture
            .worker
            .trait_definition(
                TraitDetailRequest {
                    trait_key: "trait:empty-fixture".into(),
                    expected_artifact_generation: None,
                },
                &execution,
            )
            .unwrap();
        assert_eq!(specialized.label, "Empty fixture");
        assert!(specialized.content.is_none());
        let record = fixture.worker.record_detail(HOLDER).unwrap();
        let badge = record
            .presentation
            .identity
            .traits
            .iter()
            .find(|badge| badge.value == "empty-fixture")
            .unwrap();
        assert_eq!(badge.label, "Empty fixture");
        assert!(badge.help.is_none());
        let predicate =
            atlas_domain::QueryPredicate::new(atlas_domain::QueryExpression::SetMatch {
                field: "traits".into(),
                op: atlas_domain::QuerySetMatch::Includes,
                values: vec!["empty-fixture".into()],
            });
        let page = fixture
            .worker
            .search_page(list_request(Some(predicate)), &execution)
            .unwrap();
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].record.record_key, HOLDER);
    }
}

#[test]
fn empty_approved_definition_has_no_reading_body() {
    let detail = fixture(true)
        .worker
        .definition_detail(
            DefinitionDetailRequest {
                definition_key: "domain:air".into(),
                expected_artifact_generation: None,
            },
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_eq!(detail.definition_key, "domain:air");
    assert!(detail.help.is_none());
    assert!(detail.content.is_none());
}

#[test]
fn prepared_check_and_routine_badges_use_canonical_trait_reading_capabilities() {
    let fixture = fixture(true);
    let check_traits = |badges: &[RecordBadgeView]| {
        for badge in badges {
            match badge.value.as_str() {
                "illusion" => {
                    assert_eq!(badge.label, "Illusion supplied");
                    assert!(matches!(
                        badge.help.as_ref().map(|help| &help.destination),
                        Some(ReferenceDestinationView::Definition { key }) if key == "trait:illusion"
                    ));
                }
                "fire" => {
                    assert_eq!(badge.label, "Fire");
                    assert!(badge.help.is_none());
                }
                "homebrew-trait" => {
                    assert_eq!(badge.label, "homebrew-trait");
                    assert!(badge.trait_key.is_none());
                    assert!(badge.help.is_none());
                }
                other => panic!("unexpected trait {other}"),
            }
        }
    };
    for key in ["actions:check00000000001", "hazards:hazard0000000001"] {
        let detail = fixture.worker.record_detail(key).unwrap();
        let field = detail
            .presentation
            .content
            .iter()
            .find(|field| {
                if key.starts_with("hazards:") {
                    field.role == "routine"
                } else {
                    field.role == "description"
                }
            })
            .unwrap();
        let PreparedFieldBodyView::Html { html, controls, .. } = &field.body else {
            panic!("prepared HTML")
        };
        assert_eq!(controls[0].ordinal, 0);
        let ContentControlKindView::Check {
            details: Some(details),
            ..
        } = &controls[0].control
        else {
            panic!("check details")
        };
        assert_eq!(
            details
                .traits
                .iter()
                .map(|badge| badge.value.as_str())
                .collect::<Vec<_>>(),
            ["illusion", "fire", "homebrew-trait"]
        );
        check_traits(&details.traits);
        if key.starts_with("hazards:") {
            let routine = field.routine_header.as_ref().unwrap();
            assert_eq!(routine.actions, "7 actions");
            assert_eq!(
                routine
                    .traits
                    .iter()
                    .map(|badge| badge.value.as_str())
                    .collect::<Vec<_>>(),
                ["illusion", "fire", "homebrew-trait"]
            );
            check_traits(&routine.traits);
            assert!(html.contains("Keep the authored routine."));
            assert!(!html.contains("(7 actions"));
        }
    }
}

#[test]
fn browse_summary_keeps_canonical_trait_labels_and_rarity_fallback() {
    let execution = crate::ReadExecutionContext::uncancellable();
    for (complete, label) in [(false, "Uncommon"), (true, "Uncommon supplied")] {
        let fixture = fixture(complete);
        let page = fixture
            .worker
            .search_page(list_request(None), &execution)
            .unwrap();
        let record = &page
            .rows
            .iter()
            .find(|row| row.record.record_key == HOLDER)
            .unwrap()
            .record;
        assert_eq!(record.edition, Some(RecordEditionView::Remaster));
        let badge = record
            .traits
            .iter()
            .find(|badge| badge.value == "illusion")
            .unwrap();
        assert_eq!(badge.label, "Illusion supplied");
        assert!(badge.help.is_none());
        let detail = fixture.worker.record_detail(HOLDER).unwrap();
        assert!(
            detail
                .presentation
                .identity
                .traits
                .iter()
                .any(|badge| { badge.value == "illusion" && badge.help.is_some() })
        );
        let rarity = record.rarity.as_ref().unwrap();
        assert_eq!(rarity.value, "uncommon");
        assert_eq!(rarity.label, label);
        assert!(rarity.help.is_none());
    }
}

#[test]
fn size_aliases_share_label_only_editor_metadata_and_value_transport() {
    use atlas_domain::{QueryFacetOutcome, QueryLiteral};

    let retained = fixture(false);
    let complete = fixture(true);
    for (fixture, expected) in [(&retained, "Lg"), (&complete, "Large supplied")] {
        let editor = fixture
            .worker
            .discover_filter_editor(DiscoverFilterEditorRequest {
                expected_artifact_generation: None,
                filter: None,
                selected_field_ids: vec![],
            })
            .unwrap();
        for field in ["body.size", "physical.size", "ancestry.default_size"] {
            let choice = editor
                .groups
                .iter()
                .flat_map(|group| &group.fields)
                .find(|option| option.definition.id == field)
                .unwrap()
                .choices
                .iter()
                .find(|choice| choice.value == QueryLiteral::String("lg".into()))
                .unwrap();
            assert_eq!(choice.label, expected, "{field}");
            assert!(choice.help.is_none(), "{field}");
        }
    }

    let execution = crate::ReadExecutionContext::uncancellable();
    let vocabulary = complete
        .worker
        .discover_filter_values(
            DiscoverFilterValuesRequest::Vocabulary {
                scope: FilterVocabularyScope::Artifact,
                field_id: "body.size".into(),
                text: None,
                offset: 0,
                limit: 100,
                selected_values: vec![],
            },
            &execution,
        )
        .unwrap();
    let FilterValueListView::Vocabulary { values, .. } = vocabulary else {
        panic!("vocabulary");
    };
    let large = QueryLiteral::String("lg".into());
    let choice = values
        .options
        .iter()
        .find(|choice| choice.value == large)
        .unwrap();
    assert_eq!(choice.label, "Large supplied");
    assert!(choice.help.is_none());

    let contextual = complete
        .worker
        .discover_filter_values(
            DiscoverFilterValuesRequest::Contextual {
                only_matching: false,
                context: Box::new(FilterDiscoveryContext::Filtered {
                    connection_scope: None,
                    search_content: None,
                    search_profile: None,
                    expected_artifact_generation: None,
                    filter: None,
                    text: None,
                }),
                field_id: "body.size".into(),
                clause_id: None,
                text: None,
                offset: 0,
                limit: 100,
            },
            &execution,
        )
        .unwrap();
    let FilterValueListView::Contextual {
        outcome: QueryFacetOutcome::Available { value: values },
        ..
    } = contextual
    else {
        panic!("contextual values");
    };
    let choice = values
        .options
        .iter()
        .find(|choice| choice.value == large)
        .unwrap();
    assert_eq!(choice.label, "Large supplied");
    assert!(choice.help.is_none());
    assert_eq!(choice.distinct_roots, 1);
}

#[test]
fn army_iwr_uses_the_same_definition_labels_as_creatures() {
    let fixture = fixture(true);
    let detail = fixture
        .worker
        .record_detail("armies:army000000000001")
        .unwrap();
    let RecordBodyView::Army(army) = detail.presentation.body else {
        panic!("army");
    };
    for (fact, label) in [
        (&army.immunities, "Fire immunity"),
        (&army.weaknesses, "Fire weakness"),
        (&army.resistances, "Fire resistance"),
    ] {
        let value = fact.value.as_ref().unwrap()[0]
            .damage_type
            .value
            .as_ref()
            .unwrap();
        assert_eq!(value.value, "fire");
        assert_eq!(value.label, label);
        assert!(value.help.is_none());
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
        assert!(vision.help.is_none());
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
        let skills = class.trained_skills.value.unwrap();
        assert_eq!(skills[0].label, "Arcana");
        assert!(skills[0].help.is_none());
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
fn record_backed_help_keeps_ordinary_custom_and_owned_content() {
    let fixture = fixture(true);
    let ordinary = fixture.worker.record_detail(GENERAL).unwrap();
    let serialized = serde_json::to_value(&ordinary).unwrap();
    assert!(
        serialized["presentation"]
            .get("general_definition")
            .is_none()
    );
    assert!(ordinary.presentation.content.iter().any(|field| matches!(&field.body,
        PreparedFieldBodyView::Html { html, controls, .. } if html.contains("Actual custom prose") && !controls.is_empty())));
    let detail = fixture.worker.record_detail(HOLDER).unwrap();
    let RecordBodyView::Creature(actor) = detail.presentation.body else {
        panic!("actor")
    };
    let navigation = actor
        .activities
        .value
        .unwrap()
        .into_iter()
        .find(|a| a.title == "General vision 23")
        .unwrap()
        .navigation;
    let owned = selected_activity_detail(&fixture, navigation);
    assert!(owned.presentation.content.iter().any(|field| matches!(&field.body,
        PreparedFieldBodyView::Html { html, controls, .. } if html.contains("Actual DC 23") && !controls.is_empty())));
}
