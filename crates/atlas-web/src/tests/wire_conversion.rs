//! Checked client input from the real router; only volatile strings are canonicalized.
//! Flat/table outcomes below are deterministic typed serialization controls, separate
//! from the live HTTP dice assertions. Numeric response fields are never canonicalized.
use super::{ACTOR, Fixture, Method, StatusCode, TABLE, Value, json};
use atlas_app_model::{FlatCheckRollView, TableRollView};
use std::{collections::BTreeMap, path::Path};

async fn response(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Value {
    let (status, body) = fixture.json(method, path, body).await;
    assert_eq!(status, StatusCode::OK, "{path}: {body}");
    body
}

fn canonicalize(value: &mut Value, identities: &BTreeMap<String, String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                canonicalize(value, identities);
            }
        }
        Value::Object(fields) => {
            for (name, value) in fields {
                match (name.as_str(), &mut *value) {
                    ("created_at" | "updated_at", Value::String(text)) => {
                        *text = "2026-01-01T00:00:00Z".into();
                    }
                    ("artifact_generation", Value::String(text)) => {
                        *text = "fixture-generation".into();
                    }
                    ("turn_key", Value::String(text)) => *text = "fixture-turn".into(),
                    _ => canonicalize(value, identities),
                }
            }
        }
        Value::String(text) => {
            for (identity, replacement) in identities {
                *text = text.replace(identity, replacement);
            }
        }
        _ => {}
    }
}

#[tokio::test]
async fn browser_wire_fixture_matches_real_router_and_serialized_controls() {
    let fixture = Fixture::new();
    let generation = fixture.service.readiness().artifact_generation.unwrap();
    let trait_records = response(
        &fixture,
        Method::POST,
        "/api/traits/records",
        Some(json!({"trait_key":"trait:fire","kind":"members","page":{"number":1,"size":25},"expected_artifact_generation":generation,"search_profile":null,"ranking":null})),
    ).await;
    assert_eq!(trait_records["page"]["total"], 1);
    let record_detail = response(
        &fixture,
        Method::GET,
        &format!("/api/records/{ACTOR}"),
        None,
    )
    .await;
    response(
        &fixture,
        Method::POST,
        "/api/encounters/ambush/participants/record",
        Some(json!({"encounter_ref":"ambush","record_ref":ACTOR,"quantity":1,"participant_variant":"elite","initiative":20})),
    ).await;
    let detail = fixture.service.encounter("ambush").unwrap();
    let actor_key = detail
        .participants
        .iter()
        .find(|p| p.record_key.as_deref() == Some(ACTOR))
        .unwrap()
        .participant_key
        .clone();
    for (index, name) in ["Frightened", "Slowed"].into_iter().enumerate() {
        let catalog = response(
            &fixture,
            Method::GET,
            &format!("/api/encounters/ambush/participants/{actor_key}/condition-definitions"),
            None,
        )
        .await;
        let condition_ref = catalog["conditions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap()["condition_ref"]
            .clone();
        let revision = fixture.service.encounter("ambush").unwrap().revision;
        response(&fixture,Method::POST,&format!("/api/encounters/ambush/participants/{actor_key}/conditions"),Some(json!({"request_id":format!("condition-{index}"),"expected_encounter_revision":revision,"participant_key":actor_key,"condition_ref":condition_ref,"event_intent":"apply","decisions":{"keep_upright":false},"scope":{"kind":"everyone"},"extra_action":{"kind":"unspecified"},"immunity_override":false,"parameters":{"kind":"none"},"value":1}))).await;
    }
    let detail = fixture.service.encounter("ambush").unwrap();
    let manual = detail
        .participants
        .iter()
        .find(|p| p.participant_key == fixture.participant_key)
        .unwrap();
    response(&fixture,Method::POST,&format!("/api/encounters/ambush/participants/{}/hp",manual.participant_key),Some(json!({"request_id":"unknown-hp","participant_key":manual.participant_key,"expected_revision":manual.revision,"action":{"kind":"set_values","current":{"input":null},"maximum":{"input":null}}}))).await;
    let revision = fixture.service.encounter("ambush").unwrap().revision;
    response(
        &fixture,
        Method::POST,
        "/api/encounters/ambush/advance",
        Some(json!({"request_id":"start","expected_revision":revision})),
    )
    .await;
    let encounter = response(&fixture, Method::GET, "/api/encounters/ambush", None).await;
    let actor = encounter["participants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["participant_key"] == actor_key)
        .unwrap();
    assert_eq!(actor["turn_resources"]["regular_total"], 2);
    assert_eq!(actor["turn_resources"]["round_number"], 1);
    let reference = response(&fixture,Method::POST,&format!("/api/encounters/ambush/participants/{actor_key}/reference"),Some(json!({"artifact_generation":generation,"selection":{"record_key":ACTOR,"embedded_path":[],"fields":[],"passage":null,"source_fingerprint":fixture.source_fingerprint}}))).await;
    let recovery = response(&fixture,Method::POST,"/api/encounters/ambush/recovery/preview",Some(json!({"expected_encounter_revision":encounter["revision"],"action":{"kind":"rest_overnight","healing_mode":"natural","participants":[{"participant_key":fixture.participant_key,"healing":0},{"participant_key":actor_key,"healing":0}]}}))).await;
    assert!(
        recovery["participants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["current_hp"].is_null())
    );
    assert!(
        recovery["participants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["healing"] == 0)
    );
    let table = response(&fixture,Method::POST,"/api/rolls/table",Some(json!({"artifact_generation":generation,"selection":{"record_key":TABLE,"embedded_path":[],"fields":[],"passage":null,"source_fingerprint":null}}))).await;
    assert!((1..=2).contains(&table["outcome"]["total"].as_i64().unwrap()));
    let table: TableRollView = serde_json::from_value(table).unwrap();
    let table_control = TableRollView {
        outcome: serde_json::from_value(
            json!({"formula":"1","total":1,"expression":{"kind":"number","value":1}}),
        )
        .unwrap(),
        matches: table.matches,
        invalid_ranges: table.invalid_ranges,
    };
    let flat = response(
        &fixture,
        Method::POST,
        "/api/rolls/flat-check",
        Some(json!({"dc":11})),
    )
    .await;
    assert_eq!(flat["dc"], 11);
    assert!((1..=20).contains(&flat["outcome"]["total"].as_i64().unwrap()));
    let mut flat_control: FlatCheckRollView = serde_json::from_value(flat).unwrap();
    flat_control.outcome = serde_json::from_value(
        json!({"formula":"11","total":11,"expression":{"kind":"number","value":11}}),
    )
    .unwrap();
    flat_control.success = true;
    let mut payload = json!({"trait_records":trait_records,"record_detail":record_detail,"encounter":encounter,"encounter_reference":reference,"recovery_preview":recovery,"table_roll_serialization_control":table_control,"flat_check_serialization_control":flat_control});
    let identities = BTreeMap::from([
        (fixture.encounter_key.clone(), "fixture-encounter".into()),
        (fixture.participant_key.clone(), "fixture-manual".into()),
        (actor_key, "fixture-actor".into()),
    ]);
    canonicalize(&mut payload, &identities);
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../web/atlas-ui/src/api/__fixtures__/wire-conversion.json");
    if std::env::var_os("ATLAS_UPDATE_WIRE_FIXTURE").is_some() {
        std::fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
        )
        .unwrap();
    }
    let checked: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        payload, checked,
        "wire fixture is stale; rerun this test with ATLAS_UPDATE_WIRE_FIXTURE=1, inspect the change, then rerun without it"
    );
}
