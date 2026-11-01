//! Checked client input from the real router; only volatile strings are canonicalized.
//! The flat-check outcome is a deterministic typed serialization control, separate
//! from live HTTP dice assertions. Numeric response fields are never canonicalized.
use super::{ACTOR, Fixture, Method, StatusCode, TABLE, Value, json};
use atlas_app_model::FlatCheckRollView;
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
                    ("turn_key" | "boundary_key", Value::String(text)) => {
                        *text = "fixture-turn".into();
                    }
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

async fn apply_condition(fixture: &Fixture, participant_key: &str, name: &str) {
    let catalog = response(
        fixture,
        Method::GET,
        &format!("/api/encounters/ambush/participants/{participant_key}/condition-definitions"),
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
    response(
        fixture,
        Method::POST,
        &format!("/api/encounters/ambush/participants/{participant_key}/conditions"),
        Some(json!({
            "request_id":format!("{participant_key}-{name}"),
            "expected_encounter_revision":revision,
            "participant_key":participant_key,
            "condition_ref":condition_ref,
            "event_intent":"apply",
            "decisions":{"keep_upright":false},
            "scope":{"kind":"everyone"},
            "extra_action":{"kind":"unspecified"},
            "immunity_override":false,
            "parameters":{"kind":"none"},
            "value":1,
            "duration":{"amount":1,"unit":"minutes","timing":{"participant_key":participant_key,"event":"end"}}
        })),
    )
    .await;
}

#[tokio::test]
async fn browser_wire_fixture_matches_real_router_and_serialized_controls() {
    let fixture = Fixture::new();
    let generation = fixture.service.readiness().artifact_generation.unwrap();
    let trait_records = response(
        &fixture,
        Method::POST,
        "/api/traits/records",
        Some(json!({
            "trait_key":"trait:fire",
            "kind":"members",
            "page":{"number":1,"size":25},
            "expected_artifact_generation":generation,
            "search_profile":null,
            "ranking":null
        })),
    )
    .await;
    assert_eq!(trait_records["page"]["total"], 1);
    let ordinary = response(
        &fixture,
        Method::GET,
        &format!("/api/records/{ACTOR}"),
        None,
    )
    .await;
    assert_eq!(
        ordinary["presentation"]["body"]["value"]["armor_class"]["value"],
        16
    );
    assert!(ordinary["presentation"]["body"]["value"]["armor_class"]["adjustment"].is_null());
    let detail = fixture.service.encounter("ambush").unwrap();
    response(
        &fixture,
        Method::POST,
        &format!(
            "/api/encounters/ambush/participants/{}/hp",
            fixture.participant_key
        ),
        Some(json!({
            "request_id":"unknown-hp",
            "participant_key":fixture.participant_key,
            "expected_revision":detail.participants[0].revision,
            "action":{"kind":"set_values","current":{"input":null},"maximum":{"input":null}}
        })),
    )
    .await;
    response(
        &fixture,
        Method::POST,
        "/api/encounters/ambush/participants/manual",
        Some(json!({"encounter_ref":"ambush","display_name":"Zero hero","current_hp":0,"max_hp":20,"initiative":-1})),
    ).await;
    let detail = fixture.service.encounter("ambush").unwrap();
    let zero_key = detail
        .participants
        .iter()
        .find(|p| p.display_name == "Zero hero")
        .unwrap()
        .participant_key
        .clone();
    apply_condition(&fixture, &fixture.participant_key, "Slowed").await;
    let current = fixture.service.encounter("ambush").unwrap();
    let participant = current
        .participants
        .iter()
        .find(|p| p.participant_key == fixture.participant_key)
        .unwrap();
    response(&fixture,Method::POST,&format!("/api/encounters/ambush/participants/{}/manual-trackers",fixture.participant_key),Some(json!({
        "request_id":"wire-tracker","expected_encounter_revision":current.revision,
        "expected_participant_revision":participant.revision,"participant_key":"path-is-authoritative",
        "action":{"kind":"add","name":"Breath ready","duration":{"amount":1,"unit":"minutes","remaining_rounds":null,"timing":{"participant_key":fixture.participant_key,"event":"end"}}}
    }))).await;
    let current = fixture.service.encounter("ambush").unwrap();
    let participant = current
        .participants
        .iter()
        .find(|p| p.participant_key == fixture.participant_key)
        .unwrap();
    response(&fixture, Method::POST, &format!("/api/encounters/ambush/participants/{}/manual-trackers", fixture.participant_key), Some(json!({
        "request_id":"wire-completion", "expected_encounter_revision":current.revision,
        "expected_participant_revision":participant.revision, "participant_key":fixture.participant_key,
        "action":{"kind":"add","name":"Countdown","duration":{"amount":1,"unit":"rounds","timing":{"participant_key":fixture.participant_key,"event":"start"}}}
    }))).await;
    let pending = response(&fixture, Method::GET, "/api/encounters/ambush", None).await;
    assert!(pending["participants"][0]["turn_resources"]["regular_total"].is_null());
    assert!(pending["participants"][0]["turn_resources"]["round_number"].is_null());
    let revision = fixture.service.encounter("ambush").unwrap().revision;
    response(
        &fixture,
        Method::POST,
        "/api/encounters/ambush/advance",
        Some(json!({"request_id":"start","expected_revision":revision})),
    )
    .await;
    let encounter = response(&fixture, Method::GET, "/api/encounters/ambush", None).await;
    let manual = encounter["participants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["participant_key"] == fixture.participant_key)
        .unwrap();
    let tracker = manual["manual_trackers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "Breath ready")
        .unwrap();
    assert_eq!(tracker["remaining_rounds"], 10);
    assert!(tracker["duration"].get("remaining_rounds").is_none());
    assert_eq!(manual["conditions"][0]["duration"]["remaining_rounds"], 10);
    assert_eq!(manual["turn_resources"]["regular_total"], 2);
    assert_eq!(manual["turn_resources"]["round_number"], 1);
    assert!(manual.get("current_hp").is_none());
    let zero = encounter["participants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["participant_key"] == zero_key)
        .unwrap();
    assert_eq!(zero["current_hp"], 0);
    assert!(zero["turn_resources"]["regular_total"].is_null());
    assert!(zero["turn_resources"]["round_number"].is_null());
    let turn_review = response(
        &fixture,
        Method::GET,
        "/api/encounters/ambush/turn-review",
        None,
    )
    .await;
    assert!(turn_review["outgoing"].is_null());
    assert_eq!(turn_review["incoming"]["event"], "start");
    assert_eq!(turn_review["incoming"]["source_revision"], revision);
    let completion = &turn_review["incoming"]["reminders"][0];
    assert_eq!(completion["kind"], "tracker_elapsed");
    assert_eq!(completion["tracker_key"], "tracker_wire-completion");
    assert!(completion.get("result").is_none());
    let recovery = response(
        &fixture,
        Method::POST,
        "/api/encounters/ambush/recovery/preview",
        Some(json!({
            "expected_encounter_revision":encounter["revision"],
            "action":{"kind":"rest_overnight","healing_mode":"natural","participants":[
                {"participant_key":fixture.participant_key,"healing":0},
                {"participant_key":zero_key,"healing":0}
            ]}
        })),
    )
    .await;
    let recovery_rows = recovery["participants"].as_array().unwrap();
    assert!(recovery_rows.iter().any(|p| p["current_hp"].is_null()
        && p["resulting_hp"].is_null()
        && p["resulting_max_hp"].is_null()
        && p["healing"] == 0));
    assert!(recovery_rows.iter().any(|p| p["current_hp"] == 0
        && p["resulting_hp"] == 0
        && p["resulting_max_hp"] == 20
        && p["healing"] == 0));
    let current = fixture.service.encounter("ambush").unwrap();
    let participant = current
        .participants
        .iter()
        .find(|p| p.participant_key == fixture.participant_key)
        .unwrap();
    response(&fixture, Method::POST, &format!("/api/encounters/ambush/participants/{}/manual-trackers", fixture.participant_key), Some(json!({
        "request_id":"wire-remove-completion", "expected_encounter_revision":current.revision,
        "expected_participant_revision":participant.revision, "participant_key":fixture.participant_key,
        "action":{"kind":"remove", "tracker_key":"tracker_wire-completion"}
    }))).await;
    let turn_review_removed = response(
        &fixture,
        Method::GET,
        "/api/encounters/ambush/turn-review",
        None,
    )
    .await;
    assert_eq!(
        turn_review_removed["incoming"]["reminders"][0]["result"],
        "Removed"
    );
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
    for name in ["Frightened", "Slowed"] {
        apply_condition(&fixture, &actor_key, name).await;
    }
    let record_detail = response(
        &fixture,
        Method::POST,
        &format!("/api/encounters/ambush/participants/{actor_key}/reference"),
        Some(json!({"artifact_generation":generation,"selection":{"record_key":ACTOR,"embedded_path":[],"fields":[],"passage":null,"source_fingerprint":fixture.source_fingerprint}})),
    ).await;
    let armor = &record_detail["presentation"]["body"]["value"]["armor_class"];
    assert_eq!(armor["value"], 17);
    assert_eq!(armor["adjustment"]["authored"], 16);
    assert_eq!(armor["adjustment"]["applied"][0]["value"], 2);
    assert_eq!(armor["adjustment"]["applied"][1]["value"], -1);
    let table = response(
        &fixture,
        Method::POST,
        "/api/rolls/table",
        Some(json!({"artifact_generation":generation,"selection":{"record_key":TABLE,"embedded_path":[],"fields":[],"passage":null,"source_fingerprint":null}})),
    ).await;
    assert!((1..=2).contains(&table["outcome"]["total"].as_i64().unwrap()));
    assert_eq!(table["matches"].as_array().unwrap().len(), 1);
    assert_eq!(
        table["matches"][0]["presentation"]["body"]["kind"],
        "table_result"
    );
    assert_eq!(
        table["matches"][0]["presentation"]["body"]["value"]["weight"]["value"],
        1
    );
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
    let mut payload = json!({
        "trait_records":trait_records,
        "encounter":encounter,
        "record_detail":record_detail,
        "recovery_preview":recovery,
        "turn_review":turn_review,
        "turn_review_removed":turn_review_removed,
        "flat_check_serialization_control":flat_control
    });
    let identities = BTreeMap::from([
        (fixture.encounter_key.clone(), "fixture-encounter".into()),
        (fixture.participant_key.clone(), "fixture-manual".into()),
        (zero_key, "fixture-zero".into()),
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
