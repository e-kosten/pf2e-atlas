use super::*;
use atlas_app_model::{CharacterSheetResponse, SavePathbuilderPcRequest};

#[tokio::test]
async fn party_directory_roster_and_list_header_match_their_readers() {
    let fixture = Fixture::new();
    let (status, created) = fixture
        .json(
            Method::POST,
            "/api/characters",
            Some(json!({"request_id":"directory-pc","name":"Sage","level":7})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let pc = &created["character_key"];
    let (status, party) = fixture.json(
        Method::POST, "/api/parties",
        Some(json!({"name":"Team","members":[{"character_key":pc,"expected_character_revision":0}]})),
    ).await;
    assert_eq!(status, StatusCode::OK);
    let key = party["party_key"].as_str().unwrap();
    let (status, directory) = fixture.json(Method::GET, "/api/parties", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        directory["parties"],
        json!([{"party_key":key,"name":"Team","revision":party["revision"],"member_count":1}])
    );
    let path = format!("/api/characters/{}", pc.as_str().unwrap());
    let (status, _) = fixture
        .json(
            Method::PATCH,
            &path,
            Some(json!({"expected_revision":0,"name":"Seer"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, renamed) = fixture
        .json(
            Method::PATCH,
            &format!("/api/parties/{key}/name"),
            Some(json!({"expected_revision":party["revision"],"name":"New team"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["members"][0]["level"], 7);
    assert_eq!(renamed["members"][0]["name"], "Seer");
    let (status, _) = fixture
        .json(
            Method::PATCH,
            &path,
            Some(json!({"expected_revision":1,"name":"Seer","level":null})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, roster) = fixture
        .json(Method::GET, &format!("/api/parties/{key}"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(roster["members"][0].get("level").is_none());
    let (status, bulk) = fixture
        .json(Method::GET, "/api/parties/rosters", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bulk["parties"], json!([roster]));
    let (status, header) = fixture
        .json(Method::GET, "/api/lists/research/header", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(header["list_key"], fixture.list_key);
    assert_eq!(header["item_count"], 1);
    assert!(header.get("items").is_none());
}

#[tokio::test]
async fn bulk_party_levels_require_an_explicit_value_and_reject_partial_writes() {
    let fixture = Fixture::new();
    let mut members = Vec::new();
    for name in ["Sage", "Rook"] {
        let (status, pc) = fixture
            .json(
                Method::POST,
                "/api/characters",
                Some(json!({"request_id": name, "name": name, "level": 5})),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{pc}");
        members.push(json!({"character_key":pc["character_key"], "expected_character_revision":0}));
    }
    let (ok, party) = fixture
        .json(
            Method::POST,
            "/api/parties",
            Some(json!({"name":"Team", "members":members})),
        )
        .await;
    assert_eq!(ok, StatusCode::OK, "{party}");
    let path = format!("/api/parties/{}", party["party_key"].as_str().unwrap());
    let mut request =
        json!({"name":"Team","members":members,"expected_revision":party["revision"]});
    let (ok, unchanged) = fixture
        .json(Method::PATCH, &path, Some(request.clone()))
        .await;
    assert_eq!(ok, StatusCode::OK, "{unchanged}");
    assert_eq!(unchanged["members"], party["members"]);

    request["expected_revision"] = unchanged["revision"].clone();
    let first_key = &members[0]["character_key"];
    let second_key = &members[1]["character_key"];
    let edit = |key: &Value, revision, level: Value| json!({"character_key":key, "expected_character_revision":revision,"level":level});
    for (invalid, expected) in [
        (members[1].clone(), StatusCode::BAD_REQUEST),
        (edit(second_key, 1, json!(null)), StatusCode::CONFLICT),
    ] {
        request["level_updates"] = json!([edit(first_key, 0, json!(9)), invalid]);
        assert_eq!(
            fixture
                .json(Method::PATCH, &path, Some(request.clone()))
                .await
                .0,
            expected
        );
        let (_, roster) = fixture.json(Method::GET, &path, None).await;
        assert_eq!(roster, unchanged);
        for member in &members {
            let pc_path = format!(
                "/api/characters/{}",
                member["character_key"].as_str().unwrap()
            );
            let (ok, pc) = fixture.json(Method::GET, &pc_path, None).await;
            assert_eq!(ok, StatusCode::OK, "{pc}");
            assert_eq!(pc["level"], 5);
            assert_eq!(pc["character_revision"], 0);
        }
    }
    request["level_updates"] = json!([
        edit(first_key, 0, json!(9)),
        edit(second_key, 0, json!(null))
    ]);
    let (ok, saved) = fixture.json(Method::PATCH, &path, Some(request)).await;
    assert_eq!(ok, StatusCode::OK, "{saved}");
    assert_eq!(saved["members"][0]["level"], 9);
    assert!(saved["members"][1].get("level").is_none());
}

#[tokio::test]
async fn sheet_wire_preserves_layout_roles_and_omits_absent_optional_fields() {
    let fixture = Fixture::new();
    let raw = json!({"success":true,"build":{
        "name":"Layout wire example","level":4,"class":"Wizard",
        "acTotal":{"acTotal":19},"proficiencies":{"reflex":2},
        "feats":[["Assurance","Athletics","Skill Feat",2]],
        "specials":[],"spellCasters":[],
        "equipment":[["Container",0,"bag","Custom state"]],
        "equipmentContainers":{"bag":{"containerName":"Backpack","backpack":false}},
        "weapons":[{"name":"Dagger","attack":0}]
    }})
    .to_string();
    let preview = fixture.service.preview_pathbuilder_pc(&raw, None).unwrap();
    let saved = fixture
        .service
        .save_pathbuilder_pc(SavePathbuilderPcRequest {
            request_id: "sheet-wire-layout".into(),
            json: raw,
            expected_artifact_generation: preview.artifact_generation,
            character_key: None,
            expected_character_revision: None,
            party: None,
            rule_links: Default::default(),
        })
        .unwrap();
    let (status, body) = fixture
        .json(
            Method::GET,
            &format!("/api/characters/{}/sheet", saved.character_key),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let decoded: CharacterSheetResponse = serde_json::from_value(body.clone()).unwrap();
    assert_eq!(decoded.character_key, saved.character_key);
    let section = |kind| {
        body["sheet"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .find(|section| section["kind"] == kind)
            .unwrap()
    };
    let stats: Vec<_> = section("stats")["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|group| group["label"].as_str().unwrap())
        .collect();
    assert_eq!(
        stats,
        [
            "Defenses & movement",
            "Perception & saves",
            "Character details"
        ]
    );
    let progression = &section("progression")["groups"][0];
    assert_eq!(progression["kind"], "advancement");
    assert_eq!(progression["label"], "Level 2");
    let feat = &progression["rows"][0];
    assert!(feat.get("kind").is_none());
    assert_eq!(
        feat["facts"][1],
        json!({"label":"Acquired at level","value":"2","role":"acquired_level"})
    );
    let inventory = section("inventory")["groups"].as_array().unwrap();
    let bag = inventory
        .iter()
        .find(|group| group["container_id"] == "bag")
        .unwrap();
    assert_eq!(bag["rows"][0]["kind"], "container_metadata");
    assert_eq!(
        bag["rows"][0]["facts"][0],
        json!({"label":"Backpack","value":"No"})
    );
    assert!(bag["rows"][1].get("kind").is_none());
    assert_eq!(
        bag["rows"][1]["facts"][0],
        json!({"label":"Quantity","value":"0","role":"quantity"})
    );
    assert_eq!(bag["rows"][1]["facts"][1]["role"], "state");
    let weapon = &inventory
        .iter()
        .find(|group| group["kind"] == "weapons")
        .unwrap()["rows"][0];
    assert_eq!(
        weapon["facts"][0],
        json!({"label":"Attack","value":"+0","role":"primary"})
    );
}
