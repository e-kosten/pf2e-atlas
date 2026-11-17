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
fn create(service: &crate::AtlasAppService, name: &str) -> PartyView {
    service
        .create_party(CreatePartyRequest {
            name: name.into(),
            members: ["First PC", "Second PC", "Second PC"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| PartyMemberInput {
                    level: [None, Some(4), Some(5)][index],
                    member_key: None,
                    name: name.into(),
                })
                .collect(),
        })
        .unwrap()
}
fn encounter(service: &crate::AtlasAppService) -> String {
    service
        .create_encounter(CreateEncounterRequest {
            name: "Encounter".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key
}
#[test]
fn party_addition_is_source_free_ordered_and_independent_of_later_edits_and_deletion() {
    let (_directory, service) = service();
    let party = create(&service, "  Campaign  ");
    assert_eq!(party.name, "Campaign");
    assert_ne!(party.members[1].member_key, party.members[2].member_key);
    create(&service, "Other campaign");
    let key = encounter(&service);
    service
        .add_encounter_party(
            &key,
            AddEncounterPartyRequest {
                party_key: party.party_key.clone(),
                expected_revision: party.revision,
            },
        )
        .unwrap();
    let updated = service
        .update_party(
            &party.party_key,
            UpdatePartyRequest {
                expected_revision: party.revision,
                name: "Renamed".into(),
                members: vec![PartyMemberInput {
                    level: None,
                    member_key: Some(party.members[0].member_key.clone()),
                    name: "Changed PC".into(),
                }],
            },
        )
        .unwrap();
    service
        .delete_party(
            &party.party_key,
            DeletePartyRequest {
                expected_revision: updated.revision,
            },
        )
        .unwrap();
    assert_eq!(service.parties().unwrap().parties.len(), 1);
    let detail = service.encounter(&key).unwrap();
    assert_eq!(detail.artifact_generation, None);
    assert_eq!(
        detail
            .participants
            .iter()
            .map(|p| p.display_name.as_str())
            .collect::<Vec<_>>(),
        ["First PC", "Second PC", "Second PC"]
    );
    assert_eq!(
        detail
            .participants
            .iter()
            .map(|pc| pc.level_override)
            .collect::<Vec<_>>(),
        vec![None, Some(4), Some(5)]
    );
    for pc in detail.participants {
        assert_eq!(pc.participant_kind, EncounterParticipantKindView::Pc);
        assert_eq!(pc.side, EncounterParticipantSideView::Pc);
        assert_eq!(pc.current_hp, None);
        assert_eq!(pc.max_hp, None);
        assert_eq!(pc.initiative, None);
        assert!(pc.conditions.is_empty());
    }
}
#[test]
fn stale_empty_and_deleted_parties_add_nothing() {
    let (_directory, service) = service();
    let party = create(&service, "Campaign");
    let key = encounter(&service);
    let updated = service
        .update_party(
            &party.party_key,
            UpdatePartyRequest {
                expected_revision: party.revision,
                name: "Changed".into(),
                members: vec![],
            },
        )
        .unwrap();
    assert_eq!(
        service
            .add_encounter_party(
                &key,
                AddEncounterPartyRequest {
                    party_key: updated.party_key.clone(),
                    expected_revision: updated.revision
                }
            )
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    let request = AddEncounterPartyRequest {
        party_key: party.party_key.clone(),
        expected_revision: party.revision,
    };
    assert_eq!(
        service
            .add_encounter_party(&key, request.clone())
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::PartyConflict
    );
    service
        .delete_party(
            &party.party_key,
            DeletePartyRequest {
                expected_revision: updated.revision,
            },
        )
        .unwrap();
    assert_eq!(
        service
            .add_encounter_party(&key, request)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::PartyNotFound
    );
    assert!(service.encounter(&key).unwrap().participants.is_empty());
}
#[test]
fn invalid_party_edit_preserves_the_complete_roster_and_rejects_foreign_member_keys() {
    let (_directory, service) = service();
    let party = create(&service, "Campaign");
    let other = create(&service, "Other");
    for member in [
        PartyMemberInput {
            level: None,
            member_key: None,
            name: " ".into(),
        },
        PartyMemberInput {
            level: None,
            member_key: Some(other.members[0].member_key.clone()),
            name: "Foreign PC".into(),
        },
    ] {
        assert!(
            service
                .update_party(
                    &party.party_key,
                    UpdatePartyRequest {
                        expected_revision: party.revision,
                        name: "Changed".into(),
                        members: vec![member]
                    }
                )
                .is_err()
        );
        assert_eq!(
            service
                .parties()
                .unwrap()
                .parties
                .into_iter()
                .find(|p| p.party_key == party.party_key)
                .unwrap(),
            party
        );
    }
    assert!(
        service
            .create_party(CreatePartyRequest {
                name: " ".into(),
                members: vec![]
            })
            .is_err()
    );
}

#[test]
fn adding_a_party_requires_both_documents_to_belong_to_the_local_owner() {
    let (_directory, service) = service();
    let party = create(&service, "Local party");
    let encounter_key = encounter(&service);
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute_batch(
            "INSERT INTO principals VALUES (7, 'other', 'Other');
         INSERT INTO parties(owner_id,party_key,name) VALUES (7,'foreign-party','Other party');
         INSERT INTO encounters(owner_id,encounter_key,slug,name,status,created_at,updated_at)
         VALUES (7,'foreign-encounter','encounter','Other encounter','draft','now','now');",
        )
        .unwrap();
    write.commit().unwrap();

    let from_other_party = service.add_encounter_party(
        &encounter_key,
        AddEncounterPartyRequest {
            party_key: "foreign-party".into(),
            expected_revision: 0,
        },
    );
    assert_eq!(
        from_other_party.unwrap_err().into_app_error().code,
        AppErrorCode::PartyNotFound
    );

    let into_other_encounter = service.add_encounter_party(
        "foreign-encounter",
        AddEncounterPartyRequest {
            party_key: party.party_key,
            expected_revision: party.revision,
        },
    );
    assert_eq!(
        into_other_encounter.unwrap_err().into_app_error().code,
        AppErrorCode::EncounterNotFound
    );
    let read = workspace
        .read(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let participant_count: i64 = read
        .connection()
        .query_row("SELECT COUNT(*) FROM encounter_participants", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(participant_count, 0);
}

#[test]
fn pathbuilder_links_recover_and_refresh_in_linked_encounters() {
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

    let party = create(&service, "Campaign");
    let export = |level| {
        json!({
            "success": true, "build": { "name": "Wizard", "level": level,
                "feats": [["Counterspell",null,"Class Feat",2]], "specials": [],
                "spellCasters": [], "focus": {"arcane":{"int":{"focusSpells":["Test Focus Spell"]}}}
            }
        })
        .to_string()
    };
    let draft = service.preview_pathbuilder_pc(&export(7)).unwrap();
    assert!(draft.abilities[0].record_key.is_none());
    assert_eq!(
        draft.abilities[1].record_key.as_ref().unwrap().to_string(),
        "spells:testspell0000001"
    );
    assert!(
        service
            .save_pathbuilder_pc(
                &party.party_key,
                SavePathbuilderPcRequest {
                    json: export(7),
                    expected_party_revision: party.revision,
                    expected_artifact_generation: "stale-exhibit".into(),
                    member_key: None,
                }
            )
            .is_err()
    );
    assert!(
        service
            .save_pathbuilder_pc(
                &party.party_key,
                SavePathbuilderPcRequest {
                    json: export(7),
                    expected_party_revision: party.revision + 1,
                    expected_artifact_generation: draft.artifact_generation.clone(),
                    member_key: None,
                }
            )
            .is_err()
    );
    assert_eq!(service.parties().unwrap().parties[0].members, party.members);
    let saved = service
        .save_pathbuilder_pc(
            &party.party_key,
            SavePathbuilderPcRequest {
                json: export(7),
                expected_party_revision: party.revision,
                expected_artifact_generation: draft.artifact_generation.clone(),
                member_key: None,
            },
        )
        .unwrap();
    let encounter_key = encounter(&service);
    service
        .add_encounter_party(
            &encounter_key,
            AddEncounterPartyRequest {
                party_key: saved.party_key.clone(),
                expected_revision: saved.party_revision,
            },
        )
        .unwrap();
    let detail = service.encounter(&encounter_key).unwrap();
    let pc = detail.participants.last().unwrap();
    assert_eq!(pc.level_override, Some(7));

    let linked = service
        .save_pc_ability_link(
            &saved.party_key,
            &saved.member_key,
            &saved.abilities[0].ability_key,
            SavePcAbilityLinkRequest {
                expected_party_revision: saved.party_revision,
                selection: PcAbilitySelection::Manual {
                    record_key: "feats:testfeat00000001".parse().unwrap(),
                    expected_artifact_generation: draft.artifact_generation.clone(),
                },
            },
        )
        .unwrap();
    let refreshed = service
        .save_pathbuilder_pc(
            &party.party_key,
            SavePathbuilderPcRequest {
                json: export(8),
                expected_party_revision: linked.party_revision,
                expected_artifact_generation: draft.artifact_generation,
                member_key: Some(linked.member_key.clone()),
            },
        )
        .unwrap();
    let table = service
        .encounter_pc_reference(&encounter_key, &pc.participant_key)
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
        service
            .encounter(&encounter_key)
            .unwrap()
            .participants
            .last()
            .unwrap()
            .level_override,
        Some(7)
    );

    let revised = service
        .update_party(
            &party.party_key,
            UpdatePartyRequest {
                name: "New campaign".into(),
                expected_revision: refreshed.party_revision,
                members: vec![PartyMemberInput {
                    member_key: Some(saved.member_key),
                    name: "Changed Wizard".into(),
                    level: Some(8),
                }],
            },
        )
        .unwrap();
    let table = service
        .encounter_pc_reference(&encounter_key, &pc.participant_key)
        .unwrap()
        .reference
        .unwrap();
    assert_eq!(table.name, "Changed Wizard");
    assert!(table.abilities[0].manual);

    let portable = service.export_encounter(&encounter_key).unwrap();
    let copied = service
        .import_encounter(portable.as_slice(), Some("portable-pc-test".into()))
        .unwrap();
    let imported_detail = service.encounter(&copied.encounter_key).unwrap();
    let imported_pc = imported_detail.participants.last().unwrap();
    assert!(
        service
            .encounter_pc_reference(&copied.encounter_key, &imported_pc.participant_key)
            .unwrap()
            .reference
            .is_none()
    );
    let attach = |revision, member: &str| SetEncounterPcReferenceRequest {
        expected_encounter_revision: revision,
        association: EncounterPcAssociation::Attach {
            member_key: member.into(),
        },
    };
    let revision = imported_detail.revision;
    let member = &revised.members[0].member_key;
    assert!(
        service
            .set_encounter_pc_reference(
                &copied.encounter_key,
                &imported_pc.participant_key,
                attach(revision, "unknown")
            )
            .is_err()
    );
    assert!(
        service
            .set_encounter_pc_reference(
                &copied.encounter_key,
                &imported_pc.participant_key,
                attach(revision + 1, member)
            )
            .is_err()
    );
    service
        .set_encounter_pc_reference(
            &copied.encounter_key,
            &imported_pc.participant_key,
            attach(revision, member),
        )
        .unwrap();
    assert_eq!(
        service
            .encounter_pc_reference(&copied.encounter_key, &imported_pc.participant_key)
            .unwrap()
            .reference
            .unwrap()
            .member_key,
        *member
    );
    let detail = service.encounter(&copied.encounter_key).unwrap();
    service
        .set_encounter_pc_reference(
            &copied.encounter_key,
            &imported_pc.participant_key,
            SetEncounterPcReferenceRequest {
                expected_encounter_revision: detail.revision,
                association: EncounterPcAssociation::Detach,
            },
        )
        .unwrap();
    assert!(
        service
            .encounter_pc_reference(&copied.encounter_key, &imported_pc.participant_key)
            .unwrap()
            .reference
            .is_none()
    );

    let archive_runtime = |path: std::path::PathBuf| {
        atlas_runtime::AtlasRuntime::resolve(atlas_runtime::AtlasRuntimeOptions {
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(path),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap()
    };
    let runtime = archive_runtime(path);
    let archive = runtime.capture_workspace_archive().unwrap();
    let mut bytes = Vec::new();
    archive.write(&mut bytes).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["version"], "3.0");
    let mut cross_owner = json.clone();
    cross_owner["saved"]["owners"][0]["pc"]["associations"][0]["member_key"] =
        json!("foreign-member");
    cross_owner["saved"]["owners"].as_array_mut().unwrap().push(json!({
        "key": "another-owner", "name": "Another GM", "lists": [], "encounters": [],
        "parties": [{
            "key": "foreign-party", "name": "Foreign party", "revision": 0,
            "members": [{"key": "foreign-member", "name": "Foreign PC", "level": 7, "position": 0}]
        }],
    }));
    assert!(
        atlas_runtime::WorkspaceArchive::read(cross_owner.to_string().as_bytes())
            .err()
            .is_some_and(|error| error.contains("PC association has no owned party member"))
    );
    let restored_path = directory.path().join("restored.sqlite");
    atlas_runtime::initialize_workspace(&restored_path).unwrap();
    json["version"] = json!("2.0");
    assert!(atlas_runtime::WorkspaceArchive::read(json.to_string().as_bytes()).is_err());
    let archive = atlas_runtime::WorkspaceArchive::read(bytes.as_slice()).unwrap();
    let other = archive_runtime(restored_path);
    let preflight = other.preflight_workspace_restore().unwrap();
    other
        .restore_workspace_archive(&archive, &preflight, false)
        .unwrap();
    let state = other.read_state().unwrap();
    let store = state.local_state().unwrap();
    let association = store
        .parties()
        .encounter_pc_member(&encounter_key, &pc.participant_key)
        .unwrap()
        .unwrap();
    assert_eq!(association.member_key, revised.members[0].member_key);
    assert!(
        store
            .parties()
            .pc_reference(&association.party_key, &association.member_key)
            .unwrap()
            .unwrap()
            .abilities[0]
            .manual
            .is_some()
    );

    service
        .delete_party(
            &revised.party_key,
            DeletePartyRequest {
                expected_revision: revised.revision,
            },
        )
        .unwrap();
    assert!(
        service
            .encounter_pc_reference(&encounter_key, &pc.participant_key)
            .unwrap()
            .reference
            .is_none()
    );
    assert_eq!(
        service
            .encounter(&encounter_key)
            .unwrap()
            .participants
            .last()
            .unwrap()
            .level_override,
        Some(7)
    );
}
