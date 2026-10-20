use atlas_app_model::*;

fn service() -> (tempfile::TempDir, crate::AtlasAppService) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace.sqlite");
    atlas_runtime::initialize_workspace(&path).unwrap();
    let service = crate::AtlasAppService::new(
        crate::service::RetrievalBackend::command(
            crate::service::CommandRetrievalMode::NoEmbeddings,
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
                .map(|name| PartyMemberInput {
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
            member_key: None,
            name: " ".into(),
        },
        PartyMemberInput {
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
