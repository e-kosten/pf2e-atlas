use atlas_app_model::*;

fn service() -> (tempfile::TempDir, crate::AtlasAppService) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace.sqlite");
    atlas_runtime::initialize_workspace(&path).unwrap();
    let service = crate::AtlasAppService::new(
        crate::service::RetrievalBackend::command(
            crate::service::CommandRetrievalMode::NoQueryInference,
        ),
        atlas_runtime::AtlasRuntimeOptions {
            path_mode: atlas_runtime::AtlasPathMode::Global,
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(path),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    (directory, service)
}

fn next_id() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
fn character(service: &crate::AtlasAppService, name: &str, level: Option<i64>) -> PcReferenceView {
    let saved = service
        .create_character(CreateCharacterRequest {
            request_id: format!("create-{}", next_id()),
            name: name.into(),
            level,
            party: None,
        })
        .unwrap();
    service.character(&saved.character_key).unwrap()
}
fn member(character: &PcReferenceView) -> PartyMemberInput {
    PartyMemberInput {
        character_key: character.character_key.clone(),
        expected_character_revision: character.character_revision,
    }
}
fn party(
    service: &crate::AtlasAppService,
    name: &str,
    characters: &[&PcReferenceView],
) -> PartyView {
    service
        .create_party(CreatePartyRequest {
            name: name.into(),
            members: characters.iter().map(|c| member(c)).collect(),
        })
        .unwrap()
}
fn encounter(service: &crate::AtlasAppService) -> String {
    service
        .create_encounter(CreateEncounterRequest {
            name: format!("Encounter {}", next_id()),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key
}
fn party_add(
    service: &crate::AtlasAppService,
    encounter: &str,
    party: &PartyView,
) -> AddEncounterPartyRequest {
    AddEncounterPartyRequest {
        request_id: format!("add-party-{}", next_id()),
        expected_encounter_revision: service.encounter(encounter).unwrap().revision,
        party_key: party.party_key.clone(),
        expected_revision: party.revision,
        characters: party
            .members
            .iter()
            .map(|m| CharacterRevision {
                character_key: m.character_key.clone(),
                expected_revision: m.character_revision,
            })
            .collect(),
    }
}
fn single_add(
    service: &crate::AtlasAppService,
    encounter: &str,
    character: &PcReferenceView,
) -> AddEncounterCharacterRequest {
    AddEncounterCharacterRequest {
        request_id: format!("add-single-{}", next_id()),
        expected_encounter_revision: service.encounter(encounter).unwrap().revision,
        party: None,
        source: EncounterCharacterSource::Existing {
            character_key: character.character_key.clone(),
            expected_character_revision: character.character_revision,
        },
    }
}
#[test]
fn shared_characters_keep_identity_when_memberships_and_parties_change() {
    let (_directory, service) = service();
    let first = character(&service, "First", Some(7));
    let second = character(&service, "Second", None);
    let full = party(&service, "Full", &[&first, &second]);
    let active = party(&service, "Active", &[&first]);
    assert_eq!(service.characters().unwrap().characters.len(), 2);
    assert_eq!(
        service
            .character(&first.character_key)
            .unwrap()
            .parties
            .len(),
        2
    );
    let key = encounter(&service);
    let add = party_add(&service, &key, &full);
    service.add_encounter_party(&key, add.clone()).unwrap();
    service.add_encounter_party(&key, add).unwrap();
    assert_eq!(service.encounter(&key).unwrap().participants.len(), 2);
    service
        .update_character(
            &first.character_key,
            UpdateCharacterRequest {
                expected_revision: first.character_revision,
                name: "Renamed".into(),
                level: Some(8),
            },
        )
        .unwrap();
    let parties = service.parties().unwrap().parties;
    assert!(
        parties
            .iter()
            .all(|p| p.members[0].name == "Renamed" && p.members[0].level == Some(8))
    );
    assert_eq!(
        service.encounter(&key).unwrap().participants[0].level_override,
        Some(7)
    );
    service
        .update_party(
            &full.party_key,
            UpdatePartyRequest {
                expected_revision: full.revision,
                name: full.name.clone(),
                members: vec![],
                level_updates: vec![],
            },
        )
        .unwrap();
    service
        .delete_party(
            &active.party_key,
            DeletePartyRequest {
                expected_revision: active.revision,
            },
        )
        .unwrap();
    assert!(
        service
            .character(&first.character_key)
            .unwrap()
            .parties
            .is_empty()
    );
    let pc = service.encounter(&key).unwrap().participants[0]
        .participant_key
        .clone();
    assert_eq!(
        service
            .encounter_pc_reference(&key, &pc)
            .unwrap()
            .reference
            .unwrap()
            .name,
        "Renamed"
    );
    let current = service.character(&first.character_key).unwrap();
    service
        .delete_character(
            &first.character_key,
            DeleteCharacterRequest {
                expected_revision: current.character_revision,
            },
        )
        .unwrap();
    assert!(
        service
            .encounter_pc_reference(&key, &pc)
            .unwrap()
            .reference
            .is_none()
    );
    assert_eq!(service.encounter(&key).unwrap().participants.len(), 2);
}
#[test]
fn stale_character_or_party_inputs_preserve_the_reviewed_roster() {
    let (_directory, service) = service();
    let first = character(&service, "First", Some(7));
    let full = party(&service, "Full", &[&first]);
    let key = encounter(&service);
    let stale = party_add(&service, &key, &full);
    service
        .update_character(
            &first.character_key,
            UpdateCharacterRequest {
                expected_revision: 0,
                name: "First".into(),
                level: Some(8),
            },
        )
        .unwrap();
    assert!(service.add_encounter_party(&key, stale).is_err());
    assert!(service.encounter(&key).unwrap().participants.is_empty());
    let current = service.character(&first.character_key).unwrap();
    let membership = member(&current);
    assert!(
        service
            .update_party(
                &full.party_key,
                UpdatePartyRequest {
                    expected_revision: full.revision,
                    name: "Invalid".into(),
                    members: vec![membership.clone(), membership],
                    level_updates: vec![]
                }
            )
            .is_err()
    );
    assert_eq!(service.parties().unwrap().parties[0].name, "Full");
    let empty = party(&service, "Empty", &[]);
    assert!(
        service
            .add_encounter_party(&key, party_add(&service, &key, &empty))
            .is_err()
    );
}
#[test]
fn manual_creation_retry_and_encounter_addition_are_atomic_and_owner_scoped() {
    let (_directory, service) = service();
    let group = party(&service, "Group", &[]);
    let request = CreateCharacterRequest {
        request_id: "manual".into(),
        name: "Name only".into(),
        level: None,
        party: Some(CharacterPartyTarget {
            party_key: group.party_key.clone(),
            expected_revision: group.revision,
        }),
    };
    let created = service.create_character(request.clone()).unwrap();
    let again = service.create_character(request.clone()).unwrap();
    assert_eq!(created.character_key, again.character_key);
    assert_eq!(service.characters().unwrap().characters.len(), 1);
    assert_eq!(service.parties().unwrap().parties[0].members.len(), 1);
    let mut changed = request.clone();
    changed.name = "Changed".into();
    assert!(service.create_character(changed).is_err());
    service
        .delete_character(
            &created.character_key,
            DeleteCharacterRequest {
                expected_revision: 0,
            },
        )
        .unwrap();
    assert!(!service.create_character(request).unwrap().available);
    assert!(service.characters().unwrap().characters.is_empty());
    let key = encounter(&service);
    let manual = AddEncounterCharacterRequest {
        request_id: "encounter-manual".into(),
        expected_encounter_revision: service.encounter(&key).unwrap().revision,
        party: Some(CharacterPartyTarget {
            party_key: "missing".into(),
            expected_revision: 0,
        }),
        source: EncounterCharacterSource::Manual {
            name: "New PC".into(),
            level: None,
            max_hp: Some(25),
            initiative: Some(12),
        },
    };
    assert!(
        service
            .add_encounter_character(&key, manual.clone())
            .is_err()
    );
    assert!(service.characters().unwrap().characters.is_empty());
    let manual = AddEncounterCharacterRequest {
        party: None,
        ..manual
    };
    service
        .add_encounter_character(&key, manual.clone())
        .unwrap();
    service
        .add_encounter_character(&key, manual.clone())
        .unwrap();
    assert_eq!(service.characters().unwrap().characters.len(), 1);
    let detail = service.encounter(&key).unwrap();
    assert_eq!(detail.participants.len(), 1);
    assert_eq!(detail.participants[0].max_hp, Some(25));
    let mut changed = manual;
    changed.source = EncounterCharacterSource::Manual {
        name: "Different".into(),
        level: None,
        max_hp: None,
        initiative: None,
    };
    assert!(service.add_encounter_character(&key, changed).is_err());
    let foreign = service.characters().unwrap().characters[0]
        .character_key
        .clone();
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let db = write.connection();
    db.execute(
        "INSERT INTO principals(principal_key,name) VALUES('other','Other')",
        [],
    )
    .unwrap();
    db.execute("UPDATE characters SET owner_id=(SELECT id FROM principals WHERE principal_key='other') WHERE character_key=?1",[&foreign]).unwrap();
    write.commit().unwrap();
    assert!(service.character(&foreign).is_err());
    assert!(
        service
            .update_character(
                &foreign,
                UpdateCharacterRequest {
                    expected_revision: 0,
                    name: "Hijack".into(),
                    level: None
                }
            )
            .is_err()
    );
    assert!(
        service
            .create_party(CreatePartyRequest {
                name: "Foreign".into(),
                members: vec![PartyMemberInput {
                    character_key: foreign,
                    expected_character_revision: 0
                }]
            })
            .is_err()
    );
}

fn pathbuilder_service() -> (tempfile::TempDir, crate::AtlasAppService) {
    use atlas_search::test_support::{
        ReferencePreparation, hash, input, record, reference_inventory_identity,
    };
    use serde_json::json;
    let (directory, service) = service();
    let path = directory.path().join("workspace.sqlite");
    let spell = |pack, id| {
        record(
            pack,
            "Item",
            json!({
                "_id": id, "type": "spell", "name": "Test Focus Spell", "system": {}
            }),
        )
    };
    let mut index = input(
        vec![
            spell("spells", "testspell0000001"),
            spell("sf-spells", "testspell0000002"),
            record(
                "feats",
                "Item",
                json!({"_id":"testfeat00000001","type":"feat","name":"Prepared Counterspell","system":{}}),
            ),
            record(
                "feats",
                "Item",
                json!({"_id":"testfeat00000002","type":"feat","name":"Practiced Counterspell","system":{}}),
            ),
        ],
        false,
    );
    let (expected_entry_count, entry_keys_sha256) =
        reference_inventory_identity(std::iter::empty()).unwrap();
    index.context.reference_preparation = Some(ReferencePreparation {
        source_package: Some("pf2e".into()),
        pack_systems: [("sf-spells".into(), Some("sf2e".into()))].into(),
        recipe: "test/v1".into(),
        input_hash: hash(b"pc-test"),
        coverage: atlas_search::ReferenceCoverage::Complete,
        expected_entry_count,
        entry_keys_sha256,
    });
    crate::test_support::publish_input(&path, &index).unwrap();
    (directory, service)
}

fn pathbuilder_export(level: i64) -> String {
    serde_json::json!({
        "success": true,
        "build": {
            "name": "Wizard", "level": level, "feats": [["Counterspell", null, "Class Feat", 2]],
            "specials": [], "spellCasters": [],
            "focus": {"arcane": {"int": {"focusSpells": ["Test Focus Spell"]}}}
        }
    })
    .to_string()
}

#[test]
fn imports_reviewed_rule_links_with_party_membership_and_overrides_them_on_refresh() {
    let (_directory, service) = pathbuilder_service();
    let json = pathbuilder_export(7);
    let draft = service.preview_pathbuilder_pc(&json, None).unwrap();
    let ability_key = draft.abilities[0].ability_key.clone();
    let prepared_rule: atlas_domain::RecordKey = "feats:testfeat00000001".parse().unwrap();
    let practiced_rule = "feats:testfeat00000002".parse().unwrap();
    let roster = party(&service, "Party", &[]);
    let mut request = SavePathbuilderPcRequest {
        request_id: "import-with-links".into(),
        json,
        rule_links: [("unknown-ability".into(), prepared_rule.clone())].into(),
        party: Some(CharacterPartyTarget {
            party_key: roster.party_key.clone(),
            expected_revision: roster.revision,
        }),
        expected_character_revision: None,
        expected_artifact_generation: draft.artifact_generation.clone(),
        character_key: None,
    };
    assert!(service.save_pathbuilder_pc(request.clone()).is_err());
    assert!(service.characters().unwrap().characters.is_empty());
    assert!(service.parties().unwrap().parties[0].members.is_empty());

    request.rule_links = [(ability_key.clone(), prepared_rule)].into();
    let saved = service.save_pathbuilder_pc(request.clone()).unwrap();
    let replay = service.save_pathbuilder_pc(request.clone()).unwrap();
    assert_eq!(replay.character_key, saved.character_key);
    let character = service.character(&saved.character_key).unwrap();
    assert!(character.abilities[0].manual);
    assert_eq!(
        character.abilities[0].record_title.as_deref(),
        Some("Prepared Counterspell")
    );
    let party = &service.parties().unwrap().parties[0];
    assert_eq!(party.members.len(), 1);
    assert_eq!(party.members[0].character_key, saved.character_key);

    request.rule_links.insert(ability_key, practiced_rule);
    assert!(service.save_pathbuilder_pc(request.clone()).is_err());
    let review = service
        .preview_pathbuilder_pc(&pathbuilder_export(8), Some(&saved.character_key))
        .unwrap();
    assert_eq!(review.retained_manual_count, 1);
    let refreshed = service
        .save_pathbuilder_pc(SavePathbuilderPcRequest {
            request_id: "refresh-with-links".into(),
            json: pathbuilder_export(8),
            character_key: Some(saved.character_key.clone()),
            expected_character_revision: review.character_revision,
            expected_artifact_generation: review.artifact_generation,
            party: None,
            rule_links: request.rule_links,
        })
        .unwrap();
    assert_eq!(refreshed.character_key, saved.character_key);
    let refreshed = service.character(&refreshed.character_key).unwrap();
    assert_eq!(refreshed.level, Some(8));
    assert!(refreshed.abilities[0].manual);
    assert_eq!(
        refreshed.abilities[0].record_title.as_deref(),
        Some("Practiced Counterspell")
    );
}

#[test]
fn encounter_import_rejects_unavailable_links_and_commits_valid_choices_once() {
    let (_directory, service) = pathbuilder_service();
    let key = encounter(&service);
    let json = pathbuilder_export(7);
    let draft = service.preview_pathbuilder_pc(&json, None).unwrap();
    let ability_key = draft.abilities[0].ability_key.clone();
    let mut request = AddEncounterCharacterRequest {
        request_id: "encounter-import-with-links".into(),
        expected_encounter_revision: service.encounter(&key).unwrap().revision,
        party: None,
        source: EncounterCharacterSource::Pathbuilder {
            json,
            expected_artifact_generation: draft.artifact_generation,
            rule_links: [(
                ability_key.clone(),
                "feats:missing000000001".parse().unwrap(),
            )]
            .into(),
        },
    };
    assert!(
        service
            .add_encounter_character(&key, request.clone())
            .is_err()
    );
    assert!(service.characters().unwrap().characters.is_empty());
    assert!(service.encounter(&key).unwrap().participants.is_empty());

    if let EncounterCharacterSource::Pathbuilder { rule_links, .. } = &mut request.source {
        rule_links.insert(ability_key, "feats:testfeat00000001".parse().unwrap());
    }
    let saved = service
        .add_encounter_character(&key, request.clone())
        .unwrap();
    let replay = service.add_encounter_character(&key, request).unwrap();
    assert_eq!(replay, saved);
    assert_eq!(service.characters().unwrap().characters.len(), 1);
    assert_eq!(service.encounter(&key).unwrap().participants.len(), 1);
    let reference = service
        .encounter_pc_reference(&key, &saved.participant_key)
        .unwrap()
        .reference
        .unwrap();
    assert!(reference.abilities[0].manual);
    assert_eq!(
        reference.abilities[0].record_title.as_deref(),
        Some("Prepared Counterspell")
    );
}

#[test]
fn imported_references_refresh_once_across_shared_parties_encounters_and_recovery() {
    use serde_json::json;
    let (directory, service) = pathbuilder_service();
    let path = directory.path().join("workspace.sqlite");
    let export = pathbuilder_export;
    let draft = service.preview_pathbuilder_pc(&export(7), None).unwrap();
    assert!(draft.abilities[0].record_key.is_none());
    assert_eq!(
        draft.abilities[1].record_key.as_ref().unwrap().to_string(),
        "spells:testspell0000001"
    );
    assert_eq!(draft.sections.len(), 2);
    assert_eq!((draft.resolved_count, draft.unresolved_count), (1, 1));
    let request = SavePathbuilderPcRequest {
        request_id: "import".into(),
        json: export(7),
        expected_artifact_generation: draft.artifact_generation.clone(),
        character_key: None,
        expected_character_revision: None,
        party: None,
        rule_links: Default::default(),
    };
    let mut stale = request.clone();
    stale.expected_artifact_generation = "stale".into();
    assert!(service.save_pathbuilder_pc(stale).is_err());
    let saved = service.save_pathbuilder_pc(request.clone()).unwrap();
    assert_eq!(
        service.save_pathbuilder_pc(request).unwrap().character_key,
        saved.character_key
    );
    let view = service.character(&saved.character_key).unwrap();
    let direct_encounter = encounter(&service);
    let direct_request = AddEncounterCharacterRequest {
        request_id: "direct-import".into(),
        expected_encounter_revision: service.encounter(&direct_encounter).unwrap().revision,
        source: EncounterCharacterSource::Pathbuilder {
            json: export(7),
            expected_artifact_generation: draft.artifact_generation.clone(),
            rule_links: Default::default(),
        },
        party: None,
    };
    let direct = service
        .add_encounter_character(&direct_encounter, direct_request.clone())
        .unwrap();
    assert!(direct.available);
    assert_ne!(direct.character_key, saved.character_key);
    let mut equivalent_request = direct_request.clone();
    if let EncounterCharacterSource::Pathbuilder { json, .. } = &mut equivalent_request.source {
        *json =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(json).unwrap())
                .unwrap();
    }
    let replay = service
        .add_encounter_character(&direct_encounter, equivalent_request)
        .unwrap();
    assert_eq!(replay.character_key, direct.character_key);
    assert_eq!(replay.participant_key, direct.participant_key);
    assert_eq!(
        service
            .encounter(&direct_encounter)
            .unwrap()
            .participants
            .len(),
        1
    );
    let mut changed_request = direct_request;
    if let EncounterCharacterSource::Pathbuilder { json, .. } = &mut changed_request.source {
        *json = export(8);
    }
    assert!(
        service
            .add_encounter_character(&direct_encounter, changed_request)
            .is_err()
    );
    let full = party(&service, "Full", &[&view]);
    let active = party(&service, "Active", &[&view]);
    let key = encounter(&service);
    service
        .add_encounter_character(&key, single_add(&service, &key, &view))
        .unwrap();
    let pc = service.encounter(&key).unwrap().participants[0]
        .participant_key
        .clone();
    let linked = service
        .save_pc_ability_link(
            &view.character_key,
            &view.abilities[0].ability_key,
            SavePcAbilityLinkRequest {
                expected_character_revision: view.character_revision,
                selection: PcAbilitySelection::Manual {
                    record_key: "feats:testfeat00000001".parse().unwrap(),
                    expected_artifact_generation: draft.artifact_generation.clone(),
                },
            },
        )
        .unwrap();
    let refresh_preview = service
        .preview_pathbuilder_pc(&export(8), Some(&linked.character_key))
        .unwrap();
    assert_eq!(refresh_preview.retained_manual_count, 1);
    assert_eq!(
        refresh_preview.character_revision,
        Some(linked.character_revision)
    );
    assert_eq!(refresh_preview.previous_name.as_deref(), Some("Wizard"));
    assert_eq!(refresh_preview.previous_level, Some(7));
    assert!(refresh_preview.abilities[0].manual);
    assert_eq!(
        refresh_preview.abilities[0].record_title.as_deref(),
        Some("Prepared Counterspell")
    );
    service
        .save_pathbuilder_pc(SavePathbuilderPcRequest {
            request_id: "refresh".into(),
            json: export(8),
            expected_artifact_generation: draft.artifact_generation.clone(),
            character_key: Some(linked.character_key.clone()),
            expected_character_revision: Some(linked.character_revision),
            party: None,
            rule_links: Default::default(),
        })
        .unwrap();
    let table = service
        .encounter_pc_reference(&key, &pc)
        .unwrap()
        .reference
        .unwrap();
    assert_eq!(table.level, Some(8));
    assert!(table.abilities[0].manual);
    assert_eq!(
        table.abilities[0].record_title.as_deref(),
        Some("Prepared Counterspell")
    );
    assert_eq!(
        service.encounter(&key).unwrap().participants[0].level_override,
        Some(7)
    );
    assert!(
        service
            .parties()
            .unwrap()
            .parties
            .iter()
            .all(|p| p.members[0].level == Some(8))
    );
    let portable = service.export_encounter(&key).unwrap();
    assert!(!String::from_utf8_lossy(&portable).contains("pc_addition"));
    let copied = service
        .import_encounter(portable.as_slice(), Some("copy".into()))
        .unwrap();
    let copied_pc = service
        .encounter(&copied.encounter_key)
        .unwrap()
        .participants[0]
        .participant_key
        .clone();
    assert!(
        service
            .encounter_pc_reference(&copied.encounter_key, &copied_pc)
            .unwrap()
            .reference
            .is_none()
    );
    let revision = service.encounter(&copied.encounter_key).unwrap().revision;
    assert!(
        service
            .set_encounter_pc_reference(
                &copied.encounter_key,
                &copied_pc,
                SetEncounterPcReferenceRequest {
                    expected_encounter_revision: revision,
                    association: EncounterPcAssociation::Attach {
                        character_key: "foreign".into()
                    }
                }
            )
            .is_err()
    );
    service
        .set_encounter_pc_reference(
            &copied.encounter_key,
            &copied_pc,
            SetEncounterPcReferenceRequest {
                expected_encounter_revision: revision,
                association: EncounterPcAssociation::Attach {
                    character_key: view.character_key.clone(),
                },
            },
        )
        .unwrap();
    let runtime = |path| {
        atlas_runtime::AtlasRuntime::resolve(atlas_runtime::AtlasRuntimeOptions {
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(path),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap()
    };
    let archive = runtime(path).capture_workspace_archive().unwrap();
    let mut bytes = Vec::new();
    archive.write(&mut bytes).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(raw["version"], "4.0");
    let mut invalid = raw.clone();
    invalid["version"] = json!("3.0");
    assert!(atlas_runtime::WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let mut invalid = raw.clone();
    invalid["saved"]["owners"][0]["pc"]["character_associations"][0]["character_key"] =
        json!("foreign");
    assert!(atlas_runtime::WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let restored = directory.path().join("restored.sqlite");
    atlas_runtime::initialize_workspace(&restored).unwrap();
    let other = runtime(restored);
    let preflight = other.preflight_workspace_restore().unwrap();
    other
        .restore_workspace_archive(
            &atlas_runtime::WorkspaceArchive::read(bytes.as_slice()).unwrap(),
            &preflight,
            false,
        )
        .unwrap();
    let state = other.read_state().unwrap();
    let characters = state.local_state().unwrap().characters();
    assert_eq!(
        characters.encounter_character(&key, &pc).unwrap().unwrap(),
        view.character_key
    );
    assert!(
        characters
            .get(&view.character_key)
            .unwrap()
            .reference
            .unwrap()
            .abilities[0]
            .manual
            .is_some()
    );
    service
        .delete_party(
            &full.party_key,
            DeletePartyRequest {
                expected_revision: full.revision,
            },
        )
        .unwrap();
    service
        .delete_party(
            &active.party_key,
            DeletePartyRequest {
                expected_revision: active.revision,
            },
        )
        .unwrap();
    assert!(
        service
            .character(&view.character_key)
            .unwrap()
            .parties
            .is_empty()
    );
    assert!(
        service
            .encounter_pc_reference(&key, &pc)
            .unwrap()
            .reference
            .is_some()
    );
}

#[test]
fn deleted_character_creation_history_round_trips_without_recreating_it() {
    let (directory, service) = service();
    let request = CreateCharacterRequest {
        request_id: "deleted-create".into(),
        name: "Retired PC".into(),
        level: None,
        party: None,
    };
    let created = service.create_character(request.clone()).unwrap();
    service
        .delete_character(
            &created.character_key,
            DeleteCharacterRequest {
                expected_revision: 0,
            },
        )
        .unwrap();
    let runtime = atlas_runtime::AtlasRuntime::resolve(atlas_runtime::AtlasRuntimeOptions {
        overrides: atlas_runtime::AtlasPathOverrides {
            index_path: Some(service.workspace_path().into()),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
    let archive = runtime.capture_workspace_archive().unwrap();
    let mut bytes = Vec::new();
    archive.write(&mut bytes).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["version"],
        "4.0"
    );
    let target_path = directory.path().join("deleted-restored.sqlite");
    atlas_runtime::initialize_workspace(&target_path).unwrap();
    let target = atlas_runtime::AtlasRuntime::resolve(atlas_runtime::AtlasRuntimeOptions {
        overrides: atlas_runtime::AtlasPathOverrides {
            index_path: Some(target_path.clone()),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
    let preflight = target.preflight_workspace_restore().unwrap();
    target
        .restore_workspace_archive(
            &atlas_runtime::WorkspaceArchive::read(bytes.as_slice()).unwrap(),
            &preflight,
            false,
        )
        .unwrap();
    let restored = crate::AtlasAppService::new(
        crate::service::RetrievalBackend::command(
            crate::service::CommandRetrievalMode::NoQueryInference,
        ),
        atlas_runtime::AtlasRuntimeOptions {
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(target_path),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    let replay = restored.create_character(request).unwrap();
    assert_eq!(replay.character_key, created.character_key);
    assert!(!replay.available);
    assert!(restored.characters().unwrap().characters.is_empty());
}

#[test]
fn historical_encounter_addition_alone_requires_v4_and_replays_without_recreation() {
    let (directory, service) = service();
    let encounter_key = encounter(&service);
    let request = AddEncounterCharacterRequest {
        request_id: "retired-encounter-import".into(),
        expected_encounter_revision: service.encounter(&encounter_key).unwrap().revision,
        source: EncounterCharacterSource::Manual {
            name: "Retired PC".into(),
            level: Some(7),
            max_hp: Some(42),
            initiative: Some(12),
        },
        party: None,
    };
    let created = service
        .add_encounter_character(&encounter_key, request.clone())
        .unwrap();
    service
        .delete_character(
            &created.character_key,
            DeleteCharacterRequest {
                expected_revision: 0,
            },
        )
        .unwrap();
    let source = atlas_runtime::AtlasRuntime::resolve(atlas_runtime::AtlasRuntimeOptions {
        overrides: atlas_runtime::AtlasPathOverrides {
            index_path: Some(service.workspace_path().into()),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
    let mut bytes = Vec::new();
    source
        .capture_workspace_archive()
        .unwrap()
        .write(&mut bytes)
        .unwrap();
    let archive: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(archive["version"], "4.0");
    let pc_history = &archive["saved"]["owners"][0]["pc"];
    assert!(pc_history.get("characters").is_none());
    assert!(pc_history.get("creation_receipts").is_none());
    assert_eq!(pc_history["addition_receipts"].as_array().unwrap().len(), 1);
    let mut invalid = archive.clone();
    let receipt_json =
        &mut invalid["saved"]["owners"][0]["pc"]["addition_receipts"][0]["receipt_json"];
    let mut envelope: serde_json::Value =
        serde_json::from_str(receipt_json.as_str().unwrap()).unwrap();
    envelope["unknown"] = serde_json::json!(true);
    *receipt_json = serde_json::json!(envelope.to_string());
    assert!(atlas_runtime::WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let target_path = directory.path().join("addition-restored.sqlite");
    atlas_runtime::initialize_workspace(&target_path).unwrap();
    let options = atlas_runtime::AtlasRuntimeOptions {
        overrides: atlas_runtime::AtlasPathOverrides {
            index_path: Some(target_path),
            ..Default::default()
        },
        ..Default::default()
    };
    let target = atlas_runtime::AtlasRuntime::resolve(options.clone()).unwrap();
    let preflight = target.preflight_workspace_restore().unwrap();
    target
        .restore_workspace_archive(
            &atlas_runtime::WorkspaceArchive::read(bytes.as_slice()).unwrap(),
            &preflight,
            false,
        )
        .unwrap();
    let restored = crate::AtlasAppService::new(
        crate::service::RetrievalBackend::command(
            crate::service::CommandRetrievalMode::NoQueryInference,
        ),
        options,
    )
    .unwrap();
    let replay = restored
        .add_encounter_character(&encounter_key, request)
        .unwrap();
    assert_eq!(replay.encounter_key, created.encounter_key);
    assert_eq!(replay.character_key, created.character_key);
    assert_eq!(replay.participant_key, created.participant_key);
    assert!(!replay.available);
    assert!(restored.characters().unwrap().characters.is_empty());
    let participants = restored.encounter(&encounter_key).unwrap().participants;
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].participant_key, created.participant_key);
    assert_eq!(participants[0].current_hp, Some(42));
    assert_eq!(participants[0].initiative, Some(12));
}
