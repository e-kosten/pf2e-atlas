use atlas_app_model::*;
use atlas_runtime::{
    AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, WorkspaceArchive,
};
use std::path::Path;

#[path = "character_context_tests.rs"]
mod context_tests;

fn service() -> (tempfile::TempDir, crate::AtlasAppService) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace.sqlite");
    atlas_runtime::initialize_workspace(&path).unwrap();
    let service = service_at(&path);
    (directory, service)
}

fn runtime_options(path: &Path) -> AtlasRuntimeOptions {
    AtlasRuntimeOptions {
        path_mode: AtlasPathMode::Global,
        overrides: AtlasPathOverrides {
            index_path: Some(path.into()),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn service_at(path: &Path) -> crate::AtlasAppService {
    crate::AtlasAppService::new(
        crate::service::RetrievalBackend::command(
            crate::service::CommandRetrievalMode::NoQueryInference,
        ),
        runtime_options(path),
    )
    .unwrap()
}

fn capture_archive(path: &Path) -> Vec<u8> {
    let mut bytes = Vec::new();
    AtlasRuntime::resolve(runtime_options(path))
        .unwrap()
        .capture_workspace_archive()
        .unwrap()
        .write(&mut bytes)
        .unwrap();
    bytes
}

fn restore_archive(path: &Path, bytes: &[u8]) -> AtlasRuntime {
    atlas_runtime::initialize_workspace(path).unwrap();
    let runtime = AtlasRuntime::resolve(runtime_options(path)).unwrap();
    let preflight = runtime.preflight_workspace_restore().unwrap();
    runtime
        .restore_workspace_archive(&WorkspaceArchive::read(bytes).unwrap(), &preflight, false)
        .unwrap();
    runtime
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
                "classes",
                "Item",
                json!({"_id":"testclass0000001","type":"class","name":"Wizard","system":{}}),
            ),
            record(
                "feats",
                "Item",
                json!({"_id":"testfeat00000003","type":"feat","name":"Wizard","system":{}}),
            ),
            record(
                "backgrounds",
                "Item",
                json!({"_id":"testbackgr000001","type":"background","name":"Scholar","system":{}}),
            ),
            record(
                "backgrounds",
                "Item",
                json!({"_id":"testbackgr000002","type":"background","name":"Scholar","system":{}}),
            ),
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
            "name": "Wizard", "level": level,
            "feats": [
                ["Counterspell", null, "Class Feat", 2],
                ["Assurance", "Stealth", "Skill Feat", 2],
                ["Assurance", "Athletics", "Skill Feat", 4]
            ],
            "specials": [], "spellCasters": [],
            "focus": {"arcane": {"int": {"focusSpells": ["Test Focus Spell"]}}}
        }
    })
    .to_string()
}

fn ability_by_label<'a>(sections: &'a [PcAbilitySectionView], label: &str) -> &'a PcAbilityView {
    sections
        .iter()
        .flat_map(|section| &section.abilities)
        .find(|ability| ability.label == label)
        .unwrap()
}

#[test]
fn imports_reviewed_rule_links_with_party_membership_and_overrides_them_on_refresh() {
    let (_directory, service) = pathbuilder_service();
    let json = pathbuilder_export(7);
    let draft = service.preview_pathbuilder_pc(&json, None).unwrap();
    let ability_key = ability_by_label(&draft.sections, "Counterspell")
        .ability_key
        .clone();
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
    let counterspell = ability_by_label(&character.sections, "Counterspell");
    assert!(counterspell.manual);
    assert_eq!(
        counterspell.record_title.as_deref(),
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
    assert!(ability_by_label(&review.sections, "Counterspell").manual);
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
    let counterspell = ability_by_label(&refreshed.sections, "Counterspell");
    assert!(counterspell.manual);
    assert_eq!(
        counterspell.record_title.as_deref(),
        Some("Practiced Counterspell")
    );
    assert!(!counterspell.automatic_match_available);
    let restore = |ability: &PcAbilityView, character_revision| {
        service.save_pc_ability_link(
            &saved.character_key,
            &ability.ability_key,
            SavePcAbilityLinkRequest {
                expected_character_revision: character_revision,
                selection: PcAbilitySelection::Automatic,
            },
        )
    };
    assert!(restore(counterspell, refreshed.character_revision).is_err());
    let retained = service.character(&saved.character_key).unwrap();
    assert_eq!(retained.character_revision, refreshed.character_revision);
    let retained_rule = &ability_by_label(&retained.sections, "Counterspell").record_key;
    assert_eq!(retained_rule, &counterspell.record_key);

    let automatic_spell = ability_by_label(&retained.sections, "Test Focus Spell");
    let linked_spell = service
        .save_pc_ability_link(
            &saved.character_key,
            &automatic_spell.ability_key,
            SavePcAbilityLinkRequest {
                expected_character_revision: retained.character_revision,
                selection: PcAbilitySelection::Manual {
                    record_key: retained_rule.clone().unwrap(),
                    expected_artifact_generation: draft.artifact_generation,
                },
            },
        )
        .unwrap();
    let spell = ability_by_label(&linked_spell.sections, "Test Focus Spell");
    assert!(spell.manual);
    assert!(spell.automatic_match_available);
    let restored = restore(spell, linked_spell.character_revision).unwrap();
    let spell = ability_by_label(&restored.sections, "Test Focus Spell");
    assert!(!spell.manual);
    assert_eq!(spell.record_key, automatic_spell.record_key);
}

#[test]
fn encounter_import_rejects_unavailable_links_and_commits_valid_choices_once() {
    let (_directory, service) = pathbuilder_service();
    let key = encounter(&service);
    let json = pathbuilder_export(7);
    let draft = service.preview_pathbuilder_pc(&json, None).unwrap();
    let ability_key = ability_by_label(&draft.sections, "Counterspell")
        .ability_key
        .clone();
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
    assert!(ability_by_label(&reference.sections, "Counterspell").manual);
    assert_eq!(
        ability_by_label(&reference.sections, "Counterspell")
            .record_title
            .as_deref(),
        Some("Prepared Counterspell")
    );
}

#[test]
fn imported_references_refresh_once_across_shared_parties_encounters_and_recovery() {
    use serde_json::json;
    let (directory, service) = pathbuilder_service();
    let export = pathbuilder_export;
    let draft = service.preview_pathbuilder_pc(&export(7), None).unwrap();
    assert!(
        ability_by_label(&draft.sections, "Counterspell")
            .record_key
            .is_none()
    );
    assert_eq!(
        ability_by_label(&draft.sections, "Test Focus Spell")
            .record_key
            .as_ref()
            .unwrap()
            .to_string(),
        "spells:testspell0000001"
    );
    assert_eq!(
        draft
            .sections
            .iter()
            .map(|section| (section.kind.as_str(), section.abilities.len()))
            .collect::<Vec<_>>(),
        [("feat", 3), ("focus_spell", 1)]
    );
    let feats = &draft.sections[0].abilities;
    assert_eq!(feats[1].choice.as_deref(), Some("Stealth"));
    assert_eq!(feats[2].choice.as_deref(), Some("Athletics"));
    assert_ne!(feats[1].ability_key, feats[2].ability_key);
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
    assert!(
        service
            .add_encounter_character(&direct_encounter, equivalent_request)
            .is_err()
    );
    let replay = service
        .add_encounter_character(&direct_encounter, direct_request.clone())
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
        let mut changed: serde_json::Value = serde_json::from_str(json).unwrap();
        changed["build"]["equipment"] = json!([["Newly purchased rope", 1, "Not Invested"]]);
        *json = changed.to_string();
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
            &ability_by_label(&view.sections, "Counterspell").ability_key,
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
    assert_eq!(
        refresh_preview.character_revision,
        Some(linked.character_revision)
    );
    assert_eq!(refresh_preview.previous_name.as_deref(), Some("Wizard"));
    assert_eq!(refresh_preview.previous_level, Some(7));
    assert!(ability_by_label(&refresh_preview.sections, "Counterspell").manual);
    assert_eq!(
        ability_by_label(&refresh_preview.sections, "Counterspell")
            .record_title
            .as_deref(),
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
    assert!(ability_by_label(&table.sections, "Counterspell").manual);
    assert_eq!(
        ability_by_label(&table.sections, "Counterspell")
            .record_title
            .as_deref(),
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
    let bytes = capture_archive(service.workspace_path());
    let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(raw["version"], "5.0");
    let mut invalid = raw.clone();
    invalid["version"] = json!("4.0");
    assert!(WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let mut invalid = raw.clone();
    invalid["saved"]["owners"][0]["pc"]["character_associations"][0]["character_key"] =
        json!("foreign");
    assert!(WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let mut invalid = raw.clone();
    let character = &mut invalid["saved"]["owners"][0]["pc"]["characters"][0];
    character["pathbuilder_source"] =
        json!(r#"{"success":true,"build":{"name":"Incomplete","level":7}}"#);
    assert!(WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let mut invalid = raw.clone();
    let character = &mut invalid["saved"]["owners"][0]["pc"]["characters"][0];
    character["pathbuilder_source"] = json!(
        r#"{"success":true,"build":{"name":"Different","level":7,"feats":[],"specials":[],"spellCasters":[]}}"#
    );
    assert!(WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let other = restore_archive(&directory.path().join("restored.sqlite"), &bytes);
    let state = other.read_state().unwrap();
    let characters = state.local_state().unwrap().characters();
    assert_eq!(
        characters
            .pathbuilder_source(&view.character_key)
            .unwrap()
            .unwrap()
            .json(),
        export(8)
    );
    assert_eq!(
        characters
            .pathbuilder_source(&direct.character_key)
            .unwrap()
            .unwrap()
            .json(),
        export(7)
    );
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
    let bytes = capture_archive(service.workspace_path());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["version"],
        "4.0"
    );
    let target_path = directory.path().join("deleted-restored.sqlite");
    restore_archive(&target_path, &bytes);
    let restored = service_at(&target_path);
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
    let bytes = capture_archive(service.workspace_path());
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
    assert!(WorkspaceArchive::read(invalid.to_string().as_bytes()).is_err());
    let target_path = directory.path().join("addition-restored.sqlite");
    restore_archive(&target_path, &bytes);
    let restored = service_at(&target_path);
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

#[test]
fn saved_pathbuilder_sheet_preserves_full_source_and_reads_known_fields_independently() {
    use crate::pathbuilder_sheet::{ImportedEquipment, ImportedField::*};
    let (_directory, service) = pathbuilder_service();
    let json = r#" {
      "success": true,
      "build": {
        "name": "Example wizard",
        "level": 7,
        "class": "Wizard",
        "xp": 0,
        "ancestry": "Human",
        "abilities": { "int": 19 },
        "attributes": { "ancestryhp": 8, "classhp": 6, "speed": 25 },
        "proficiencies": { "athletics": 0, "arcana": 6, "newSkill": [] },
        "lores": [["Library", 2]],
        "mods": { "Arcana": { "Item Bonus": 1 } },
        "acTotal": { "acTotal": 22, "shieldBonus": null },
        "feats": [["Counterspell", null, "Class Feat", 2, "Wizard Feat 2", "standardChoice", null, "future"]],
        "specials": [],
        "spellCasters": [{ "name": "Wizard", "perDay": [5, 3], "spells": [
          { "spellLevel": 1, "list": ["Spell A", "Spell B"] }
        ], "prepared": [{ "spellLevel": 1, "list": ["Spell B", "Spell B"] }] }],
        "focusPoints": 0,
        "equipment": [["Rope", 2, "Not Invested"], ["Rope", 1, "bag", "Not Invested"], {"future": 1}],
        "equipmentContainers": {"bag":{"containerName":"Backpack"}},
        "weapons": [{"name":"Staff","attack":9,"die":"d4"}, ["Future", "Future", 1, "simple", 9]],
        "armor": [{"name":"Explorer's Clothing","worn":true}],
        "money": [1, 2, 3, 4],
        "formula": [{"type":"other","known":["Example Formula"]}],
        "pets": [{"type":"Animal Companion","name":"Example Bear","mature":true}],
        "familiars": null,
        "custom": {"number": 123456789012345678901234567890.012345678901}
      }
    } "#;
    let baseline = character(&service, "Manual example", Some(1));
    assert!(
        service
            .character_pathbuilder_sheet(&baseline.character_key)
            .unwrap()
            .is_none()
    );
    let preview = service.preview_pathbuilder_pc(json, None).unwrap();
    let request = SavePathbuilderPcRequest {
        request_id: "full-export".into(),
        json: json.into(),
        expected_artifact_generation: preview.artifact_generation,
        character_key: None,
        expected_character_revision: None,
        party: None,
        rule_links: Default::default(),
    };
    let saved = service.save_pathbuilder_pc(request.clone()).unwrap();
    let mut different = request.clone();
    different.json = json.replace("012345678901}", "012345678902}");
    assert_ne!(different.json, request.json);
    assert!(service.save_pathbuilder_pc(different).is_err());
    assert_eq!(
        service.save_pathbuilder_pc(request).unwrap().character_key,
        saved.character_key
    );
    let view = service.character(&saved.character_key).unwrap();
    service
        .update_character(
            &saved.character_key,
            UpdateCharacterRequest {
                expected_revision: view.character_revision,
                name: "Atlas name".into(),
                level: Some(8),
            },
        )
        .unwrap();
    let sheet = service
        .character_pathbuilder_sheet(&saved.character_key)
        .unwrap()
        .unwrap();
    assert_eq!((sheet.name.as_str(), sheet.level), ("Example wizard", 7));
    let Value(proficiencies) = sheet.proficiencies else {
        panic!("missing proficiencies")
    };
    assert_eq!(proficiencies["athletics"], Value(0));
    assert_eq!(proficiencies["newSkill"], Unrecognized);
    let Value(ac) = sheet.ac_total else {
        panic!("missing armor class")
    };
    assert_eq!(ac.ac_total, Value(22));
    assert_eq!(ac.shield_bonus, Null);
    let Value(equipment) = sheet.equipment else {
        panic!("missing equipment")
    };
    assert!(matches!(equipment.as_slice(), [
        Value(ImportedEquipment::Uncontained(loose, 2, _)),
        Value(ImportedEquipment::InContainer(packed, 1, bag, _)),
        Unrecognized
    ] if loose == "Rope" && packed == "Rope" && bag == "bag"));
    assert!(matches!(sheet.money, Unrecognized));
    let Value(weapons) = sheet.weapons else {
        panic!("missing weapons")
    };
    assert!(matches!(weapons.as_slice(), [Value(staff), Unrecognized] if staff.attack == Value(9)));
    let Value(casters) = sheet.spell_casters else {
        panic!("missing casters")
    };
    let [Value(wizard)] = casters.as_slice() else {
        panic!("missing wizard")
    };
    let Value(prepared) = &wizard.prepared else {
        panic!("missing prepared spells")
    };
    let [Value(rank)] = prepared.as_slice() else {
        panic!("missing rank")
    };
    assert_eq!(rank.spell_level, Value(1));
    assert_eq!(
        rank.list,
        Value(vec![Value("Spell B".into()), Value("Spell B".into())])
    );
    let Value(feats) = sheet.feats else {
        panic!("missing feats")
    };
    let [Value(feat)] = feats.as_slice() else {
        panic!("missing feat")
    };
    assert_eq!((feat.name.as_str(), feat.level), ("Counterspell", 2));

    let original = AtlasRuntime::resolve(runtime_options(service.workspace_path())).unwrap();
    let read = original.read_state().unwrap();
    let characters = read.local_state().unwrap().characters();
    assert_eq!(
        characters
            .pathbuilder_source(&saved.character_key)
            .unwrap()
            .unwrap()
            .json(),
        json
    );
    drop(read);

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
    db.execute(
        "INSERT INTO characters(character_key,owner_id,name,level,reference_payload,pathbuilder_source)
         SELECT 'foreign-import',p.id,c.name,c.level,c.reference_payload,c.pathbuilder_source
         FROM characters c, principals p WHERE c.character_key=?1 AND p.principal_key='other'",
        [&saved.character_key],
    ).unwrap();
    write.commit().unwrap();
    assert!(
        service
            .character_pathbuilder_sheet("foreign-import")
            .is_err()
    );
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute(
            "DELETE FROM characters WHERE character_key='foreign-import'",
            [],
        )
        .unwrap();
    write.commit().unwrap();

    let revision = service
        .character(&saved.character_key)
        .unwrap()
        .character_revision;
    service
        .delete_character(
            &saved.character_key,
            DeleteCharacterRequest {
                expected_revision: revision,
            },
        )
        .unwrap();
    let archive: serde_json::Value =
        serde_json::from_slice(&capture_archive(service.workspace_path())).unwrap();
    assert_eq!(archive["version"], "5.0");
    let pc = &archive["saved"]["owners"][0]["pc"];
    assert!(
        pc["characters"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c.get("pathbuilder_source").is_none())
    );
    assert!(
        pc["creation_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["intent_version"] == 2)
    );
}

#[test]
fn legacy_pathbuilder_receipts_replay_without_replacing_the_saved_character() {
    let (directory, service) = pathbuilder_service();
    let json = pathbuilder_export(7);
    let preview = service.preview_pathbuilder_pc(&json, None).unwrap();
    let generation = preview.artifact_generation;
    let mut create = SavePathbuilderPcRequest {
        request_id: "old-create".into(),
        json: json.clone(),
        expected_artifact_generation: generation.clone(),
        character_key: None,
        expected_character_revision: None,
        party: None,
        rule_links: Default::default(),
    };
    let encounter = encounter(&service);
    let mut add = AddEncounterCharacterRequest {
        request_id: "old-add".into(),
        expected_encounter_revision: service.encounter(&encounter).unwrap().revision,
        source: EncounterCharacterSource::Pathbuilder {
            json: json.clone(),
            expected_artifact_generation: generation.clone(),
            rule_links: Default::default(),
        },
        party: None,
    };
    let original_create = service.save_pathbuilder_pc(create.clone()).unwrap();
    let original_add = service
        .add_encounter_character(&encounter, add.clone())
        .unwrap();

    // Frozen schema-47 digests for this fixture and request context.
    let create_intent = "4da10141c162169d6edbf3576277ddc14bb7f92b69f2250c289f91ecfeb2463a";
    let add_intent =
        r#"{"digest":"12a62b797b6ee0eeddb96a75fef3a963120e5d43eb75e0ef0216b60cda3e7475"}"#;
    let legacy_generation = "recorded-schema-47-generation";
    create.expected_artifact_generation = legacy_generation.into();
    add.expected_encounter_revision = 6;
    let EncounterCharacterSource::Pathbuilder {
        expected_artifact_generation,
        ..
    } = &mut add.source
    else {
        panic!("Pathbuilder source missing");
    };
    *expected_artifact_generation = legacy_generation.into();
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let db = write.connection();
    db.execute("UPDATE characters SET pathbuilder_source=NULL", [])
        .unwrap();
    db.execute(
        "UPDATE character_creation_receipts SET intent_version=1,intent_digest=?1 WHERE request_id='old-create'",
        [&create_intent],
    ).unwrap();
    db.execute(
        "UPDATE encounter_command_receipts SET intent_json=?1 WHERE family='pc_addition' AND request_id='old-add'",
        [&add_intent],
    ).unwrap();
    write.commit().unwrap();

    let bytes = capture_archive(service.workspace_path());
    let archive: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(archive["version"], "4.0");
    let restored_path = directory.path().join("restored-legacy.sqlite");
    restore_archive(&restored_path, &bytes);
    let restored = service_at(&restored_path);
    assert_eq!(
        restored.save_pathbuilder_pc(create).unwrap().character_key,
        original_create.character_key
    );
    let replay = restored.add_encounter_character(&encounter, add).unwrap();
    assert_eq!(replay.participant_key, original_add.participant_key);
    assert_eq!(replay.character_key, original_add.character_key);
    for key in [original_create.character_key, original_add.character_key] {
        assert!(
            restored
                .character_pathbuilder_sheet(&key)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn character_sheet_review_preserves_occurrences_and_detects_same_count_changes() {
    use atlas_app_model::{ImportFieldView, SheetSectionKind};
    use serde_json::json;

    let (_directory, service) = pathbuilder_service();
    let source = json!({"success": true, "build": {
        "name": "Example wizard", "level": 7, "keyability": "int",
        "xp": 300, "alignment": "CG",
        "class": "Wizard", "background": "Scholar", "ancestry": "Custom ancestry",
        "languages": ["None selected", "Common"], "age": "Not set",
        "abilities": {"str": 9, "dex": 14, "con": 16, "int": 19},
        "attributes": {"ancestryhp": 8, "classhp": 6, "bonushp": 0, "bonushpPerLevel": 1},
        "acTotal": {"acTotal": 22, "shieldBonus": null},
        "proficiencies": {"arcana": 4, "athletics": 0, "classDC": 2, "computers": 0, "piloting": 0},
        "lores": [["Academia", 2]],
        "mods": {"Arcana": {"Item Bonus": 1}},
        "equipment": [["Rope", 0, "a", "Not Invested"]],
        "weapons": [{
            "name":"Dagger","attack":12,"die":"d4","damageType":"P",
            "qty":"two","extraDamage":[{"formula":"1d4","type":"fire","enabled":false}]
        }],
        "equipmentContainers": {
            "a": {"containerName": "Backpack", "bagOfHolding": false},
            "b": {"containerName": "Backpack"}
        },
        "feats": [["Test Heritage", null, "Heritage", 1], ["Untrained Improvisation", null, "General Feat", 3], ["Counterspell", null, "Class Feat", 2]],
        "specials": [],
        "spellCasters": [{
            "name": "Wizard", "perDay": [5, 3], "spells": [],
            "prepared": [{"spellLevel": 1, "list": ["Test Focus Spell", "Test Focus Spell"]}]
        }],
        "focusPoints": 0
    }});
    let full_source = |source: &serde_json::Value, precise: &str| {
        format!(
            r#"{{"future":{{"number":{precise},"duplicate":null,"duplicate":false}},"success":true,"build":{}}}"#,
            source["build"]
        )
    };
    let precise = "123456789012345678901234567890.012345678901";
    let original = full_source(&source, precise);
    let preview = service.preview_pathbuilder_pc(&original, None).unwrap();
    let heritage = ability_by_label(&preview.sections, "Test Heritage")
        .ability_key
        .clone();
    let counterspell = ability_by_label(&preview.sections, "Counterspell")
        .ability_key
        .clone();
    let saved = service
        .save_pathbuilder_pc(SavePathbuilderPcRequest {
            request_id: "sheet-review-example".into(),
            json: original,
            expected_artifact_generation: preview.artifact_generation,
            character_key: None,
            expected_character_revision: None,
            party: None,
            rule_links: [
                (heritage.clone(), "feats:testfeat00000001".parse().unwrap()),
                (counterspell, "feats:testfeat00000002".parse().unwrap()),
            ]
            .into(),
        })
        .unwrap();
    let current = service.character_sheet(&saved.character_key).unwrap();
    let sheet = current.sheet.unwrap();
    let rows = |kind| {
        sheet
            .sections
            .iter()
            .find(|section| section.kind == kind)
            .unwrap()
            .groups
            .iter()
            .flat_map(|group| &group.rows)
            .collect::<Vec<_>>()
    };
    let stats = rows(SheetSectionKind::Stats);
    let identity_row = |label| stats.iter().find(|row| row.label == label).unwrap();
    let class_help = identity_row("Class").summary_help.as_ref().unwrap();
    let atlas_app_model::ReferenceDestinationView::Record { navigation } = &class_help.destination
    else {
        panic!("expected the exact class record");
    };
    assert_eq!(navigation.record_key, "classes:testclass0000001");
    assert!(identity_row("Background").summary_help.is_none());
    assert!(identity_row("Ancestry").summary_help.is_none());

    let fact = |label, fact| {
        stats
            .iter()
            .find(|row| row.label == label)
            .unwrap()
            .facts
            .iter()
            .find(|value| value.label == fact)
            .map(|value| value.value.as_str())
    };
    assert_eq!(fact("Max HP", "Max HP"), Some("78"));
    assert_eq!(
        stats
            .iter()
            .find(|row| row.label == "Key ability")
            .unwrap()
            .summary_value
            .as_deref(),
        Some("Intelligence")
    );
    assert_eq!(
        rows(SheetSectionKind::Inventory)
            .iter()
            .find(|row| row.label == "Dagger")
            .unwrap()
            .facts
            .iter()
            .find(|fact| fact.label == "Damage type")
            .unwrap()
            .value,
        "Piercing"
    );
    assert_eq!(fact("Strength", "Modifier"), Some("-1"));
    let skills = sheet
        .sections
        .iter()
        .find(|section| section.kind == SheetSectionKind::Stats)
        .unwrap()
        .groups
        .iter()
        .find(|group| group.kind == Some(atlas_app_model::SheetGroupKind::Skills))
        .unwrap();
    assert_eq!(skills.rows[0].label, "Academia Lore");
    assert_eq!(skills.rows[1].label, "Acrobatics");
    assert_eq!(
        identity_row("Acrobatics").summary_value.as_deref(),
        Some("+2")
    );
    assert_eq!(fact("Acrobatics", "Rank"), Some("Unavailable"));
    assert_eq!(
        identity_row("Athletics").summary_value.as_deref(),
        Some("-1")
    );
    assert_eq!(fact("Athletics", "Rank"), Some("Untrained"));
    assert_eq!(fact("Athletics", "Ability modifier"), Some("-1"));
    assert_eq!(fact("Athletics", "Proficiency"), None);
    assert_eq!(fact("Athletics", "Base modifier"), None);
    assert!(
        !stats
            .iter()
            .any(|row| matches!(row.label.as_str(), "Age" | "XP" | "Alignment"))
    );
    assert_eq!(
        stats
            .iter()
            .find(|row| row.label == "Languages")
            .unwrap()
            .summary_value
            .as_deref(),
        Some("Common")
    );
    assert_eq!(identity_row("Arcana").summary_value.as_deref(), Some("+15"));
    assert_eq!(fact("Arcana", "Base modifier"), Some("+15"));
    assert_eq!(fact("Arcana", "Ability modifier"), Some("+4"));
    assert_eq!(fact("Arcana", "Item Bonus"), Some("+1"));
    assert_eq!(fact("AC", "Shield bonus"), None);
    let proficiencies = sheet
        .sections
        .iter()
        .find(|section| section.kind == SheetSectionKind::Proficiencies)
        .unwrap()
        .groups
        .iter()
        .find(|group| group.kind == Some(atlas_app_model::SheetGroupKind::Proficiencies))
        .unwrap();
    assert!(stats.iter().all(|row| row.label != "Class DC"));
    assert_eq!(
        proficiencies
            .rows
            .iter()
            .map(|row| (row.label.as_str(), row.summary_value.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("Class DC", Some("Trained")),
            ("Computers", Some("Untrained")),
            ("Piloting", Some("Untrained")),
        ],
    );
    assert!(rows(SheetSectionKind::Progression).iter().any(|row| {
        row.label == "Test Heritage" && row.ability_key.as_deref() == Some(heritage.as_str())
    }));
    let magic = sheet
        .sections
        .iter()
        .find(|section| section.kind == SheetSectionKind::Magic)
        .unwrap();
    let prepared = magic
        .groups
        .iter()
        .flat_map(|group| &group.spell_ranks)
        .flat_map(|rank| &rank.prepared)
        .find(|spell| spell.label == "Test Focus Spell")
        .unwrap();
    assert_eq!(prepared.count, 2);

    assert!(rows(SheetSectionKind::Inventory).iter().any(|row| {
        row.label == "Rope"
            && row
                .facts
                .iter()
                .any(|fact| fact.label == "Quantity" && fact.value == "0")
    }));
    let details = service
        .character_import_details(&saved.character_key, current.character_revision)
        .unwrap();
    let field = |fields: Vec<ImportFieldView>, label: &str| {
        fields
            .into_iter()
            .find(|field| field.label == label)
            .unwrap()
    };
    assert!(sheet.has_additional_details);
    assert_eq!(
        details
            .fields
            .iter()
            .map(|field| field.label.as_str())
            .collect::<Vec<_>>(),
        ["weapons", "Export metadata"]
    );
    let weapons = field(details.fields.clone(), "weapons");
    assert_eq!(weapons.children[0].label, "Dagger");
    assert_eq!(
        weapons.children[0]
            .children
            .iter()
            .map(|field| field.label.as_str())
            .collect::<Vec<_>>(),
        ["extraDamage", "qty"],
    );
    let metadata = field(details.fields, "Export metadata");
    let future = field(metadata.children, "future");
    assert_eq!(future.children.len(), 2);
    assert_eq!(future.children[0].value.as_deref(), Some(precise));
    assert_eq!(future.children[1].value.as_deref(), Some("false"));
    assert!(
        service
            .character_import_details(&saved.character_key, current.character_revision - 1)
            .is_err()
    );

    let mut next = source;
    next["build"]["equipment"][0][2] = json!("b");
    next["build"]["spellCasters"][0]["prepared"][0]["list"][1] = json!("Replacement");
    next["build"]["feats"].as_array_mut().unwrap().pop();
    let weapon = next["build"]["weapons"][0].as_object_mut().unwrap();
    weapon.remove("extraDamage");
    weapon.remove("qty");
    let incoming = next.to_string();
    let review = service
        .preview_pathbuilder_pc(&incoming, Some(&saved.character_key))
        .unwrap();
    let changes = review.changes.unwrap();
    assert_eq!(changes.retained_corrections, 1);
    assert_eq!(changes.removed_corrections, ["Counterspell"]);
    assert!(ability_by_label(&review.sections, "Test Heritage").manual);
    let inventory = changes
        .sections
        .iter()
        .find(|section| section.label == "Inventory")
        .unwrap();
    assert_eq!(inventory.added, ["Backpack (2) · Rope"]);
    assert_eq!(inventory.removed, ["Backpack (1) · Rope"]);
    let magic = changes
        .sections
        .iter()
        .find(|section| section.label == "Magic")
        .unwrap();
    assert_eq!(magic.added, ["Wizard · Replacement"]);
    assert_eq!(magic.removed, ["Wizard · Test Focus Spell"]);
    assert!(
        changes
            .additional_fields_changed
            .contains(&"Export metadata · future".to_owned())
    );
    assert!(
        changes
            .additional_fields_changed
            .contains(&"weapons".to_owned())
    );
    assert!(!review.sheet.has_additional_details);
    let comparison = service
        .preview_import_details(
            &incoming,
            Some(&saved.character_key),
            review.character_revision,
        )
        .unwrap();
    assert!(comparison.fields.is_empty());
    let metadata = field(comparison.previous_fields.unwrap(), "Export metadata");
    assert_eq!(
        field(metadata.children, "future").children[0]
            .value
            .as_deref(),
        Some(precise),
    );
}

#[test]
fn character_progression_orders_levels_and_compares_repeated_feats_across_levels() {
    use serde_json::json;
    let (_directory, service) = pathbuilder_service();
    let mut source = json!({"success":true,"build":{
        "name":"Progression example","level":7,"specials":[],"spellCasters":[],
        "feats":[
            ["Repeated feat","First choice","Class Feat",10],
            ["Repeated feat","Second choice","Skill Feat",2],
            ["Another feat",null,"General Feat",2,"Class FeatHuman Feat 1","childChoice","Human Feat 1"],
            ["Starting feat",null,"Ancestry Feat",0]
        ]
    }});
    let preview = service
        .preview_pathbuilder_pc(&source.to_string(), None)
        .unwrap();
    let levels = &preview
        .sheet
        .sections
        .iter()
        .find(|section| section.kind == SheetSectionKind::Progression)
        .unwrap()
        .groups;
    assert_eq!(
        levels
            .iter()
            .map(|group| group.label.as_str())
            .collect::<Vec<_>>(),
        ["Level 0", "Level 2", "Level 10"]
    );
    assert_eq!(
        levels[1]
            .rows
            .iter()
            .map(|row| row.label.as_str())
            .collect::<Vec<_>>(),
        ["Repeated feat", "Another feat"]
    );
    assert!(
        levels
            .iter()
            .all(|group| group.kind == Some(SheetGroupKind::Advancement))
    );
    assert!(
        levels[1].rows[0]
            .facts
            .iter()
            .any(|fact| fact.role == Some(SheetFactRole::Choice) && fact.value == "Second choice")
    );
    let slot = levels[1].rows[1]
        .facts
        .iter()
        .find(|fact| fact.label == "Slot")
        .unwrap();
    assert_eq!(slot.value, "Class FeatHuman Feat 1");
    assert_eq!(
        slot.display_value.as_deref(),
        Some("Class Feat · Human Feat 1")
    );
    let saved = service
        .save_pathbuilder_pc(SavePathbuilderPcRequest {
            request_id: "progression-layout".into(),
            json: source.to_string(),
            expected_artifact_generation: preview.artifact_generation,
            character_key: None,
            expected_character_revision: None,
            party: None,
            rule_links: Default::default(),
        })
        .unwrap();
    source["build"]["feats"].as_array_mut().unwrap().reverse();
    let reordered = service
        .preview_pathbuilder_pc(&source.to_string(), Some(&saved.character_key))
        .unwrap();
    assert!(
        !reordered
            .changes
            .unwrap()
            .sections
            .iter()
            .any(|section| section.label == "Progression")
    );
    // A source correction can look identical after typesetting and must still compare as changed.
    source["build"]["feats"][1][4] = json!("Class Feat · Human Feat 1");
    source["build"]["feats"][3][3] = json!(4);
    let changed = service
        .preview_pathbuilder_pc(&source.to_string(), Some(&saved.character_key))
        .unwrap();
    let changes = changed.changes.unwrap();
    let progression = changes
        .sections
        .iter()
        .find(|section| section.label == "Progression")
        .unwrap();
    assert_eq!(
        progression.changed,
        ["Feats · Another feat", "Feats · Repeated feat"]
    );
    assert!(progression.added.is_empty() && progression.removed.is_empty());
}

#[test]
fn character_inventory_distinguishes_container_metadata_and_retains_zero_and_unknown_state() {
    use serde_json::json;
    let (_directory, service) = pathbuilder_service();
    let source = json!({"success":true,"build":{
        "name":"Inventory example","level":7,"feats":[],"specials":[],"spellCasters":[],
        "equipment":[["Container",0,"a","Custom investment state"],["Rope",2,"unknown","Not Invested"]],
        "equipmentContainers":{
            "a":{"containerName":"Backpack","backpack":false},
            "b":{"containerName":"Backpack","bagOfHolding":false}
        },
        "weapons":[{"name":"Dagger","qty":0,"attack":0}],
        "money":{"gp":0,"sp":0}
    }});
    let preview = service
        .preview_pathbuilder_pc(&source.to_string(), None)
        .unwrap();
    let groups = &preview
        .sheet
        .sections
        .iter()
        .find(|section| section.kind == SheetSectionKind::Inventory)
        .unwrap()
        .groups;
    let container = groups
        .iter()
        .find(|group| group.container_id.as_deref() == Some("a"))
        .unwrap();
    assert_eq!(container.kind, Some(SheetGroupKind::Container));
    assert_eq!(
        container.rows[0].kind,
        Some(SheetRowKind::ContainerMetadata)
    );
    assert_eq!(container.rows[0].facts[0].value, "No");
    assert_eq!(container.rows[1].label, "Container");
    assert!(container.rows[1].kind.is_none());
    assert!(
        container.rows[1]
            .facts
            .iter()
            .any(|fact| fact.role == Some(SheetFactRole::Quantity) && fact.value == "0")
    );
    assert!(
        container.rows[1]
            .facts
            .iter()
            .any(|fact| fact.role == Some(SheetFactRole::State)
                && fact.value == "Custom investment state")
    );
    let empty = groups
        .iter()
        .find(|group| group.container_id.as_deref() == Some("b"))
        .unwrap();
    assert_eq!(empty.rows.len(), 1);
    assert_eq!(empty.label, "Backpack (2)");
    assert!(
        groups
            .iter()
            .any(|group| group.container_id.as_deref() == Some("unknown")
                && group.rows[0].label == "Rope")
    );
    assert!(groups.iter().any(|group| {
        group.kind == Some(SheetGroupKind::Coins)
            && group
                .rows
                .iter()
                .all(|row| row.summary_value.as_deref() == Some("0"))
    }));
}
