use super::*;
use atlas_app_model::{
    CharacterAbilityFilter, CharacterContextRequest, CharacterSection as S, CharacterSourceState,
};
use serde_json::json;

fn import_source(service: &crate::AtlasAppService, json: String) -> String {
    let preview = service.preview_pathbuilder_pc(&json, None).unwrap();
    service
        .save_reviewed_character_import(
            SavePathbuilderPcRequest {
                request_id: format!("context-{}", next_id()),
                json: json.clone(),
                rule_links: Default::default(),
                party: None,
                expected_character_revision: None,
                expected_artifact_generation: preview.artifact_generation,
                character_key: None,
            },
            &crate::character_context::character_source_sha256(&json),
        )
        .unwrap()
        .character_key
}

#[test]
fn context_preserves_typed_availability_occurrences_and_exact_source() {
    let (_directory, service) = pathbuilder_service();
    let mut source: serde_json::Value = serde_json::from_str(&pathbuilder_export(7)).unwrap();
    let build = &mut source["build"];
    build["xp"] = json!(0);
    build["alignment"] = serde_json::Value::Null;
    build["equipment"] =
        json!([["Rope", 0, "Not Invested"], ["Rope", 2, "bag", "Invested"], {"future": true}]);
    build["armor"] = json!([{"name":"Clothing", "worn":false, "qty":"many"}]);
    build["money"] = json!({"gp":0, "sp":null, "cp":"lots"});
    build["spellCasters"] = json!([{"spells":[], "prepared":[{"spellLevel":1,"list":["Test Focus Spell","Test Focus Spell"]}]}]);
    let original = format!(" {}\n", source);
    let key = import_source(&service, original.clone());
    let view = service
        .character_context(&key, CharacterContextRequest::default())
        .unwrap();
    assert_eq!(view.source_state, CharacterSourceState::Available);
    assert!(!view.sections.contains_key(&S::Source));
    let json = serde_json::to_value(&view).unwrap();
    assert_eq!(
        json["sections"]["identity"]["fields"]["xp"],
        json!({"state":"value","value":0})
    );
    assert_eq!(
        json["sections"]["identity"]["fields"]["alignment"]["state"],
        "null"
    );
    let entries = &json["sections"]["equipment"]["fields"]["equipment"]["value"];
    assert_eq!(entries.as_array().unwrap().len(), 3);
    assert_eq!(entries[0]["value"]["quantity"]["value"], 0);
    assert_eq!(entries[1]["value"]["container_id"]["value"], "bag");
    assert_eq!(entries[2]["state"], "unrecognized");
    let armor = &json["sections"]["equipment"]["fields"]["armor"]["value"][0]["value"];
    assert_eq!(armor["worn"]["value"], false);
    assert_eq!(armor["quantity"]["state"], "unrecognized");
    let money = &json["sections"]["money"]["fields"]["money"]["value"];
    assert_eq!(money["pp"]["state"], "missing");
    assert_eq!(money["gp"]["value"], 0);
    assert_eq!(money["sp"]["state"], "null");
    assert_eq!(money["cp"]["state"], "unrecognized");
    let prepared = &json["sections"]["spellcasting"]["fields"]["spell_casters"]["value"][0]["value"]
        ["prepared"]["value"][0]["value"]["list"]["value"];
    assert_eq!(prepared.as_array().unwrap().len(), 2);
    assert_eq!(prepared[0], prepared[1]);
    let raw = service
        .character_context(
            &key,
            CharacterContextRequest {
                sections: vec![S::Source],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        raw.sections[&S::Source].source_json.as_deref(),
        Some(original.as_str())
    );
}

#[test]
fn current_identity_and_party_order_are_independent_of_imported_sheet() {
    let (_directory, service) = pathbuilder_service();
    let key = import_source(&service, pathbuilder_export(7));
    let first = service.character(&key).unwrap();
    service
        .edit_character_metadata(
            &key,
            first.character_revision,
            Some("Current wizard".into()),
            Some(Some(8)),
        )
        .unwrap();
    let second = character(&service, "Manual PC", None);
    let current = service.character(&key).unwrap();
    let party = party(&service, "Context party", &[&second, &current]);
    let view = service
        .party_context(
            &party.party_key,
            CharacterContextRequest {
                sections: vec![S::Equipment],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        view.members[0].character.character_key,
        second.character_key
    );
    assert_eq!(view.members[0].source_state, CharacterSourceState::NoImport);
    assert_eq!(view.members[1].character.name, "Current wizard");
    assert_eq!(view.members[1].character.level, Some(8));
    assert_eq!(view.members[1].imported_identity.as_ref().unwrap().level, 7);
    assert_eq!(view.members[1].selected_sections, [S::Equipment]);
}

#[test]
fn reviewed_import_checks_exact_source_before_writing_and_preview_is_read_only() {
    let (_directory, service) = pathbuilder_service();
    let original = pathbuilder_export(7);
    let preview = service
        .preview_character_import(&original, None, CharacterContextRequest::default())
        .unwrap();
    assert!(service.characters().unwrap().characters.is_empty());
    let changed = format!("{} ", original);
    let request = SavePathbuilderPcRequest {
        request_id: "changed-export".into(),
        json: changed,
        rule_links: Default::default(),
        party: None,
        expected_character_revision: None,
        expected_artifact_generation: preview.artifact_generation,
        character_key: None,
    };
    assert!(
        service
            .save_reviewed_character_import(request, &preview.source_sha256)
            .is_err()
    );
    assert!(service.characters().unwrap().characters.is_empty());
}

#[test]
fn valid_manual_link_is_not_misclassified_by_automatic_ambiguity() {
    let (_directory, service) = pathbuilder_service();
    let key = import_source(&service, pathbuilder_export(7));
    let request = CharacterContextRequest {
        sections: vec![S::Abilities],
        abilities: CharacterAbilityFilter {
            needs_review: true,
            ..Default::default()
        },
    };
    let before = service.character_context(&key, request.clone()).unwrap();
    let unresolved = before.sections[&S::Abilities]
        .abilities
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .find(|a| a.ability.label == "Counterspell")
        .unwrap();
    let generation = service
        .preview_pathbuilder_pc(&pathbuilder_export(7), Some(&key))
        .unwrap()
        .artifact_generation;
    service
        .save_pc_ability_link(
            &key,
            &unresolved.ability.ability_key,
            SavePcAbilityLinkRequest {
                expected_character_revision: before.character.revision,
                selection: PcAbilitySelection::Manual {
                    record_key: "feats:testfeat00000001".parse().unwrap(),
                    expected_artifact_generation: generation,
                },
            },
        )
        .unwrap();
    let after = service.character_context(&key, request).unwrap();
    assert!(
        !after.sections[&S::Abilities]
            .abilities
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .any(|a| a.ability.label == "Counterspell")
    );
}

#[test]
fn discovery_rejects_duplicate_names_and_stale_roster_edits_preserve_state() {
    let (_directory, service) = service();
    let first = character(&service, "Same name", Some(7));
    let second = character(&service, "Same name", Some(8));
    let error = service
        .character_context("Same name", CharacterContextRequest::default())
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, atlas_app_model::AppErrorCode::InvalidRequest);
    assert_eq!(
        error.details.unwrap()["candidates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let party = party(&service, "Party", &[&first, &second]);
    let changed = service
        .edit_party_name(&party.party_key, party.revision, "Renamed".into())
        .unwrap();
    assert!(
        service
            .remove_party_members(
                &party.party_key,
                party.revision,
                vec![first.character_key.clone()]
            )
            .is_err()
    );
    let mut stale = member(&second);
    stale.expected_character_revision += 1;
    assert!(
        service
            .set_party_character_levels(
                &party.party_key,
                changed.revision,
                vec![member(&first), stale],
                Some(9)
            )
            .is_err()
    );
    assert_eq!(
        service.character(&first.character_key).unwrap().level,
        Some(7)
    );
    assert_eq!(
        service
            .party_roster(&party.party_key)
            .unwrap()
            .members
            .len(),
        2
    );
}

#[test]
fn saved_sheet_states_and_owner_isolation_work_without_reference_content() {
    let (_directory, service) = service();
    let manual = character(&service, "Owned PC", Some(7));
    let request = CharacterContextRequest {
        sections: vec![S::Equipment],
        ..Default::default()
    };
    assert_eq!(
        service
            .character_context(&manual.character_key, request.clone())
            .unwrap()
            .source_state,
        CharacterSourceState::NoImport
    );
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let source = r#"{"success":true,"build":{"name":"Imported PC","level":7,"feats":[],"specials":[],"spellCasters":[],"equipment":[["Rope",1,"Not Invested"]]}}"#;
    write
        .connection()
        .execute(
            "UPDATE characters SET reference_payload='{\"abilities\":[]}' WHERE character_key=?1",
            [&manual.character_key],
        )
        .unwrap();
    write.commit().unwrap();
    assert_eq!(
        service
            .character_context(&manual.character_key, request.clone())
            .unwrap()
            .source_state,
        CharacterSourceState::ReferenceOnly
    );
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute(
            "UPDATE characters SET pathbuilder_source=?1 WHERE character_key=?2",
            [source, manual.character_key.as_str()],
        )
        .unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO principals(principal_key,name) VALUES('foreign-context-owner','Foreign')",
            [],
        )
        .unwrap();
    write.connection().execute("INSERT INTO characters(character_key,owner_id,name,level,reference_payload,pathbuilder_source) SELECT 'foreign-context-pc',p.id,c.name,c.level,c.reference_payload,c.pathbuilder_source FROM characters c,principals p WHERE c.character_key=?1 AND p.principal_key='foreign-context-owner'", [&manual.character_key]).unwrap();
    write.commit().unwrap();
    let view = service
        .character_context(&manual.character_key, request.clone())
        .unwrap();
    assert_eq!(view.source_state, CharacterSourceState::Available);
    let json = serde_json::to_value(view).unwrap();
    assert_eq!(
        json["sections"]["equipment"]["fields"]["equipment"]["value"][0]["value"]["name"]["value"],
        "Rope"
    );
    assert_eq!(
        service
            .character_context("foreign-context-pc", request)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::CharacterNotFound
    );
}

#[test]
fn every_typed_source_field_has_a_selected_section_and_round_trips() {
    use crate::character_context::facts::FactSource;
    let (_directory, service) = pathbuilder_service();
    let source = pathbuilder_export(7);
    let decoded = crate::pathbuilder_sheet::read_sheet(&source).unwrap();
    let ImportedFactValue::Object(all) = decoded.fact_value() else {
        panic!("expected source fields")
    };
    let key = import_source(&service, source);
    let view = service
        .character_context(&key, CharacterContextRequest::default())
        .unwrap();
    let keys: std::collections::BTreeSet<_> = view
        .sections
        .values()
        .flat_map(|s| s.fields.keys())
        .collect();
    assert_eq!(keys, all.keys().collect());
    let json = serde_json::to_value(&view).unwrap();
    let decoded: CharacterContextView = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), json);
}
