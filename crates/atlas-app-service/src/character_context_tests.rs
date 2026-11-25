use super::*;
use atlas_app_model::{
    CharacterAbilityFilter, CharacterContextRequest, CharacterSection as S, CharacterSourceState,
    SheetFactRole, SheetGroupKind,
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
    build["formula"] = json!([{"type":"Alchemical", "known":["Rope"]}]);
    build["spellCasters"] = json!([{"spells":[], "prepared":[{"spellLevel":1,"list":["Test Focus Spell","Test Focus Spell"]}]}]);
    let original = format!(" {}\n", source);
    let key = import_source(&service, original.clone());
    let view = service
        .character_context(&key, CharacterContextRequest::default())
        .unwrap();
    assert_eq!(view.source_state, CharacterSourceState::Available);
    assert!(!view.sections.contains_key(&S::Source));
    let gear = view.sections[&S::Equipment]
        .presentation
        .iter()
        .find(|group| group.kind == Some(SheetGroupKind::Gear))
        .unwrap();
    assert_eq!(gear.rows[0].label, "Rope");
    assert!(
        gear.rows[0]
            .facts
            .iter()
            .any(|fact| { fact.role == Some(SheetFactRole::Quantity) && fact.value == "0" })
    );
    assert_eq!(
        view.sections[&S::Money].presentation[0].kind,
        Some(SheetGroupKind::Coins)
    );
    assert_eq!(
        view.sections[&S::Crafting].presentation[0].kind,
        Some(SheetGroupKind::Formulas)
    );
    let advancement = view.sections[&S::CharacterOptions]
        .presentation
        .iter()
        .filter(|group| group.kind == Some(SheetGroupKind::Advancement))
        .map(|group| (group.label.as_str(), group.rows.len()))
        .collect::<Vec<_>>();
    assert_eq!(advancement, [("Level 2", 2), ("Level 4", 1)]);
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
fn money_and_crafting_read_only_their_selected_facts_and_links() {
    use atlas_search::test_support::{open_source_fixture, record};
    let (retrieval, _artifact) = open_source_fixture(
        vec![record(
            "items",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Rope","type":"equipment"}),
        )],
        false,
    )
    .unwrap();
    let mut source: serde_json::Value = serde_json::from_str(&pathbuilder_export(7)).unwrap();
    source["build"]["formula"] = json!([{"type":"Alchemical","known":["Rope"]}]);
    source["build"]["money"] = json!({"gp":5});
    let sections = |source: &serde_json::Value, section| {
        let sheet = crate::pathbuilder_sheet::read_sheet(&source.to_string()).unwrap();
        retrieval.reset_read_metrics();
        let view = crate::character_context::source_sections(
            Some(&sheet),
            None,
            None,
            &[section],
            Some(&retrieval),
        )
        .unwrap();
        (
            serde_json::to_value(view).unwrap(),
            retrieval.read_metrics().sql_statements,
        )
    };
    let pristine = sections(&source, S::Crafting);
    source["build"]["weapons"] = json!([{"name":"Longsword"}]);
    source["build"]["armor"] = json!([{"name":"Plate"}]);
    source["build"]["equipment"] = json!([["Bedroll", 1, "Not Invested"]]);
    assert_eq!(sections(&source, S::Crafting), pristine);
    let (money, statements) = sections(&source, S::Money);
    assert_eq!(statements, 0);
    assert_eq!(money["money"]["fields"]["money"]["value"]["gp"]["value"], 5);
}

#[test]
fn sheet_places_imported_stats_by_use_and_keeps_cli_section_membership() {
    use atlas_app_model::{SheetGroupView, SheetSectionKind};

    fn rows<'a>(groups: &'a [SheetGroupView], label: &str) -> Vec<(&'a str, &'a str)> {
        groups
            .iter()
            .find(|group| group.label == label)
            .unwrap()
            .rows
            .iter()
            .map(|row| (row.label.as_str(), row.summary_value.as_deref().unwrap()))
            .collect()
    }

    let (_directory, service) = pathbuilder_service();
    let source = json!({"success": true, "build": {
        "name": "Scholar", "level": 1, "feats": [], "specials": [], "spellCasters": [],
        "keyability": "int", "abilities": {"dex": 10}, "background": "Cook",
        "languages": ["None selected", "Common"], "size": 0, "age": "53",
        "gender": "Woman", "deity": "Brigh", "resistances": ["Fire 5"],
        "acTotal": {"acTotal": 14}
    }});
    let key = import_source(&service, source.to_string());
    let sheet = service.character_sheet(&key).unwrap().sheet.unwrap();
    let groups = &sheet
        .sections
        .iter()
        .find(|s| s.kind == SheetSectionKind::Stats)
        .unwrap()
        .groups;
    assert_eq!(
        rows(groups, "Defenses & movement"),
        [("AC", "14"), ("Resistances", "Fire 5")]
    );
    assert_eq!(
        rows(groups, "Character details"),
        [
            ("Background", "Cook"),
            ("Languages", "Common"),
            ("Size", "0"),
            ("Age", "53"),
            ("Gender", "Woman"),
            ("Deity", "Brigh"),
        ]
    );
    assert_eq!(
        rows(groups, "Ability scores"),
        [("Dexterity", "10 (+0)"), ("Key ability", "Intelligence")]
    );
    assert!(
        !groups
            .iter()
            .any(|group| group.label == "Background & details")
    );

    let context = service
        .character_context(
            &key,
            CharacterContextRequest {
                sections: vec![S::Identity, S::Statistics],
                ..Default::default()
            },
        )
        .unwrap();
    let identity = &context.sections[&S::Identity].presentation;
    assert_eq!(
        rows(identity, "Character details"),
        [("Background", "Cook"), ("Key ability", "Intelligence")]
    );
    assert_eq!(
        rows(identity, "Background & details"),
        [
            ("Languages", "Common"),
            ("Resistances", "Fire 5"),
            ("Size", "0"),
            ("Age", "53"),
            ("Gender", "Woman"),
            ("Deity", "Brigh"),
        ]
    );
    let statistics = &context.sections[&S::Statistics].presentation;
    assert_eq!(
        rows(statistics, "Ability scores"),
        [("Dexterity", "10 (+0)")]
    );
    assert_eq!(rows(statistics, "Defenses & movement"), [("AC", "14")]);

    let sparse = json!({"success": true, "build": {
        "name": "Sparse", "level": 1, "feats": [], "specials": [], "spellCasters": [],
        "keyability": "int"
    }});
    let preview = service
        .preview_pathbuilder_pc(&sparse.to_string(), None)
        .unwrap();
    let groups = &preview
        .sheet
        .sections
        .iter()
        .find(|s| s.kind == SheetSectionKind::Stats)
        .unwrap()
        .groups;
    assert_eq!(groups.len(), 1);
    assert_eq!(
        rows(groups, "Ability scores"),
        [("Key ability", "Intelligence")]
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
fn metadata_edits_preserve_import_bytes_and_distinguish_omitted_and_cleared_levels() {
    let (_directory, service) = pathbuilder_service();
    let key = import_source(&service, pathbuilder_export(7));
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let schema = atlas_runtime::WORKSPACE_SCHEMA_VERSION;
    let write = workspace.write(schema).unwrap();
    // Valid JSON retained from an older writer need not use today's encoding.
    write.connection().execute(
        "UPDATE characters SET reference_payload=' ' || reference_payload WHERE character_key=?1",
        [&key],
    ).unwrap();
    write.commit().unwrap();
    let saved_bytes = || -> (String, String) {
        workspace.read(schema).unwrap().connection().query_row(
            "SELECT reference_payload,pathbuilder_source FROM characters WHERE character_key=?1",
            [&key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap()
    };
    let original = saved_bytes();

    let receipt = service
        .edit_character_metadata(&key, 0, Some(" Current name ".into()), None)
        .unwrap();
    assert!(receipt.available);
    assert_eq!(receipt.character_key, key);
    let renamed = service.character(&key).unwrap();
    assert_eq!(renamed.name, "Current name");
    assert_eq!(renamed.level, Some(7));

    service
        .edit_character_metadata(&key, renamed.character_revision, None, Some(None))
        .unwrap();
    let cleared = service.character(&key).unwrap();
    assert_eq!(cleared.name, "Current name");
    assert_eq!(cleared.level, None);

    service
        .update_character(
            &key,
            UpdateCharacterRequest {
                expected_revision: cleared.character_revision,
                name: cleared.name.clone(),
                level: Some(cleared.level),
            },
        )
        .unwrap();
    assert_eq!(
        service.character(&key).unwrap().character_revision,
        cleared.character_revision + 1
    );
    assert_eq!(saved_bytes(), original);
}

#[test]
fn damaged_reference_rejects_metadata_edits_before_stale_revision_or_invalid_fields() {
    let (_directory, service) = service();
    let pc = character(&service, "Original", None);
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let schema = atlas_runtime::WORKSPACE_SCHEMA_VERSION;
    let write = workspace.write(schema).unwrap();
    write
        .connection()
        .execute(
            "UPDATE characters SET reference_payload='{}' WHERE character_key=?1",
            [&pc.character_key],
        )
        .unwrap();
    write.commit().unwrap();

    let error = service
        .edit_character_metadata(
            &pc.character_key,
            pc.character_revision + 1,
            Some("".into()),
            Some(Some(21)),
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::InvalidRequest);
    assert!(
        error.message.contains("missing field `abilities`"),
        "{}",
        error.message
    );
    let stored = service.characters().unwrap().characters.pop().unwrap();
    assert_eq!(stored.name, pc.name);
    assert_eq!(stored.level, pc.level);
    assert_eq!(stored.revision, pc.character_revision);
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
    let mut keys = [&first.character_key, &second.character_key];
    keys.sort();
    assert_eq!(
        error.details.unwrap()["candidates"],
        json!([{"key":keys[0],"name":"Same name"},{"key":keys[1],"name":"Same name"}])
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
    let (_directory, service) = pathbuilder_service();
    let source = pathbuilder_export(7);
    let key = import_source(&service, source);
    let view = service
        .character_context(&key, CharacterContextRequest::default())
        .unwrap();
    let keys: std::collections::BTreeSet<_> = view
        .sections
        .values()
        .flat_map(|s| s.fields.keys().map(String::as_str))
        .collect();
    assert_eq!(
        keys,
        crate::pathbuilder_sheet::PathbuilderSheet::FACT_NAMES
            .iter()
            .copied()
            .collect()
    );
    let json = serde_json::to_value(&view).unwrap();
    let decoded: CharacterContextView = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), json);
}

#[test]
fn character_lookup_honors_legacy_keys_exact_names_and_owner_scope() {
    let (_directory, service) = service();
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write.connection().execute_batch(
        "INSERT INTO characters(character_key,owner_id,name) VALUES('member_legacy',1,'Legacy');
         INSERT INTO principals(principal_key,name) VALUES('other-legacy-owner','Another owner');
         INSERT INTO characters(character_key,owner_id,name)
         SELECT 'other-member',id,'member_legacy' FROM principals WHERE principal_key='other-legacy-owner';",
    ).unwrap();
    write.commit().unwrap();
    let namesake = character(&service, "member_legacy", None);
    let lookup = |query| service.character_context(query, CharacterContextRequest::default());
    assert_eq!(
        lookup("member_legacy").unwrap().character.character_key,
        "member_legacy"
    );
    assert_eq!(
        lookup(" member_legacy ").unwrap().character.character_key,
        namesake.character_key
    );
    assert_eq!(
        lookup("MEMBER_LEGACY").unwrap_err().into_app_error().code,
        AppErrorCode::CharacterNotFound
    );
}

#[test]
fn party_lookup_prefers_exact_keys_then_owner_scoped_case_sensitive_names() {
    let (_directory, service) = service();
    let hero = party(&service, "Heroes", &[]);
    let namesake = party(&service, &hero.party_key, &[]);
    let other = party(&service, "Heroes", &[]);
    assert_eq!(
        service.party_roster(&hero.party_key).unwrap().party_key,
        hero.party_key
    );
    assert_eq!(
        service
            .party_roster(&format!(" {} ", hero.party_key))
            .unwrap()
            .party_key,
        namesake.party_key
    );
    let miss = service.party_roster("heroes").unwrap_err().into_app_error();
    assert_eq!(miss.code, atlas_app_model::AppErrorCode::PartyNotFound);
    let error = service
        .party_roster(" Heroes ")
        .unwrap_err()
        .into_app_error();
    let mut keys = [hero.party_key, other.party_key];
    keys.sort();
    assert_eq!(error.code, atlas_app_model::AppErrorCode::InvalidRequest);
    assert_eq!(
        error.details.unwrap()["candidates"],
        json!([
            {"key":keys[0],"name":"Heroes"},{"key":keys[1],"name":"Heroes"}
        ])
    );
}

#[test]
fn import_comparison_tracks_container_names_by_id_and_item_edits_separately() {
    let (_directory, service) = pathbuilder_service();
    let mut previous: serde_json::Value = serde_json::from_str(&pathbuilder_export(7)).unwrap();
    previous["build"]["equipmentContainers"] = json!({
        "full":{"containerName":"Backpack"}, "empty":{"containerName":"Spare bag"}
    });
    previous["build"]["equipment"] = json!([["Rope", 1, "full", "Invested"]]);
    let key = import_source(&service, previous.to_string());
    let changes = |source: &serde_json::Value| {
        service
            .preview_pathbuilder_pc(&source.to_string(), Some(&key))
            .unwrap()
            .changes
            .unwrap()
            .sections
            .into_iter()
            .find(|s| s.label == "Inventory")
            .unwrap_or_default()
    };
    assert!(changes(&previous).changed.is_empty());
    let mut next = previous.clone();
    next["build"]["equipmentContainers"]["full"]["containerName"] = json!("Field pack");
    next["build"]["equipmentContainers"]["empty"]["containerName"] = json!("Pouch");
    next["build"]["equipment"][0][1] = json!(2);
    let inventory = changes(&next);
    assert_eq!(
        inventory.changed,
        [
            "Field pack · Rope",
            "Container · Spare bag → Pouch",
            "Container · Backpack → Field pack"
        ]
    );
    assert!(inventory.added.is_empty() && inventory.removed.is_empty());
    next = previous.clone();
    next["build"]["equipmentContainers"]["duplicate"] = json!({"containerName":"Backpack"});
    let inventory = changes(&next);
    assert_eq!(inventory.added.len(), 1);
    assert!(inventory.changed.is_empty() && inventory.removed.is_empty());
    next["build"]["equipmentContainers"]
        .as_object_mut()
        .unwrap()
        .remove("empty");
    next["build"]["equipmentContainers"]["another-id"] = json!({"containerName":"Satchel"});
    let inventory = changes(&next);
    assert!(inventory.changed.is_empty());
    assert!(inventory.added.contains(&"Satchel · Container".into()));
    assert_eq!(inventory.removed, ["Spare bag · Container"]);
}
