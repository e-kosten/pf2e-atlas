use super::*;
use atlas_app_model::{CharacterSheetResponse, SavePathbuilderPcRequest};

#[tokio::test]
async fn sheet_wire_preserves_layout_roles_and_omits_absent_optional_fields() {
    let fixture = Fixture::new();
    let raw = json!({"success":true,"build":{
        "name":"Layout wire example","level":4,
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
