//! Real HTTP contracts over isolated source-free glossary fixtures.
use super::*;

#[tokio::test]
async fn glossary_detail_discovery_and_existing_record_reads_share_generation_and_identity() {
    let fixture = Fixture::with_glossary();
    let (status, detail) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:chthonian"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["definition_key"], "language:chthonian");
    assert_eq!(detail["label"], "Chthonian");
    assert!(
        detail["content"]["body"]["html"]
            .as_str()
            .unwrap()
            .contains("Typically spoken by demons")
    );
    assert!(
        detail["notices"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Synthetic test content")
    );
    assert!(
        detail["attribution"]["body_grants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|grant| grant["license"] == "CC0-1.0")
    );
    let generation = detail["artifact_generation"].clone();
    let (status, sense) = fixture.json(Method::POST, "/api/definitions/detail", Some(json!({"definition_key":"sense:darkvision","expected_artifact_generation":generation}))).await;
    assert_eq!(status, StatusCode::OK, "{sense}");
    assert!(
        sense["content"].is_null(),
        "record content must retain its existing owner"
    );
    assert_eq!(sense["help"]["destination"]["kind"], "record");
    assert_eq!(
        sense["help"]["destination"]["navigation"]["record_key"],
        "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6"
    );
    assert!(
        sense["help"]["destination"]
            .get("reading_context")
            .is_none()
    );
    let mut pointer = sense["help"]["destination"]["navigation"].clone();
    let object = pointer.as_object_mut().unwrap();
    object.remove("field");
    object.insert("expected_artifact_generation".into(), generation.clone());
    let (status, pointed_record) = fixture
        .json(Method::POST, "/api/records/detail", Some(pointer))
        .await;
    assert_eq!(status, StatusCode::OK, "{pointed_record}");
    assert_eq!(
        pointed_record["selected"]["record_key"],
        sense["help"]["destination"]["navigation"]["record_key"]
    );
    assert_eq!(pointed_record["artifact_generation"], generation);
    assert!(
        pointed_record["presentation"]["content"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["body"]["html"]
                .as_str()
                .is_some_and(|html| html.contains("Fixture darkvision reference content")))
    );
    let (status, record) = fixture
        .json(
            Method::POST,
            "/api/records/detail",
            Some(json!({"record_key":ACTOR,"expected_artifact_generation":generation})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{record}");
    let actor = &record["presentation"]["body"]["value"];
    assert_eq!(actor["languages"]["value"][0]["value"], "common");
    assert_eq!(actor["languages"]["value"][0]["label"], "Common");
    assert_eq!(actor["languages"]["value"][1]["label"], "Chthonian");
    assert_eq!(
        actor["language_details"]["value"],
        "Communicates by signing."
    );
    let (status, options) = fixture.json(Method::POST, "/api/filters/definitions", Some(json!({"text":"Chthonian","offset":0,"limit":1,"expected_artifact_generation":generation}))).await;
    assert_eq!(status, StatusCode::OK, "{options}");
    assert_eq!(options["total"], 1);
    assert_eq!(
        options["options"][0]["target"],
        json!({"kind":"definition","key":"language:chthonian"})
    );
    assert_eq!(options["options"][0]["label"], "Chthonian");
    for path in ["/api/definitions/detail", "/api/filters/definitions"] {
        let request = if path.ends_with("detail") {
            json!({"definition_key":"language:chthonian","expected_artifact_generation":"old"})
        } else {
            json!({"offset":0,"limit":10,"expected_artifact_generation":"old"})
        };
        let (status, error) = fixture.json(Method::POST, path, Some(request)).await;
        assert_eq!(status, StatusCode::CONFLICT, "{error}");
        assert_eq!(error["code"], "artifact_generation_changed");
    }
}

#[tokio::test]
async fn glossary_unknown_and_legacy_coverage_remain_distinct_without_breaking_records() {
    let fixture = Fixture::with_glossary();
    let (status, unknown) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:custom-unpublished"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{unknown}");
    assert_eq!(unknown["help"]["availability"], "unavailable_unknown");
    assert!(unknown["help"]["destination"].is_null());
    let legacy = Fixture::with_legacy_glossary();
    let (status, record) = legacy
        .json(
            Method::POST,
            "/api/records/detail",
            Some(json!({"record_key":ACTOR})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{record}");
    let language = &record["presentation"]["body"]["value"]["languages"]["value"][0];
    assert_eq!(language["value"], "common");
    assert_eq!(language["help"]["availability"], "setup_required");
    assert!(language["help"]["destination"].is_null());
}

#[tokio::test]
async fn glossary_http_requests_distinguish_omitted_null_defaults_and_unknown_fields() {
    let fixture = Fixture::with_glossary();
    let (status, omitted) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:chthonian"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, explicit_null) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(
                json!({"definition_key":"language:chthonian","expected_artifact_generation":null}),
            ),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(omitted, explicit_null);
    for (path, request) in [
        ("/api/definitions/detail", json!({})),
        ("/api/definitions/detail", json!({"definition_key":null})),
        (
            "/api/definitions/detail",
            json!({"definition_key":"language:chthonian","unknown":true}),
        ),
        (
            "/api/filters/definitions",
            json!({"offset":0,"limit":10,"unknown":true}),
        ),
        (
            "/api/records/detail",
            json!({"record_key":ACTOR,"reading_context":"invented"}),
        ),
    ] {
        let (status, _) = fixture.json(Method::POST, path, Some(request)).await;
        assert!(status.is_client_error(), "{path}: {status}");
    }
}

#[tokio::test]
async fn mixed_glossary_boolean_clauses_keep_typed_destinations_and_exact_proof_pages() {
    let fixture = Fixture::with_glossary();
    let definition = |id: &str, key: &str| json!({"kind":"definition_reference","clause_id":id,"reference":{"target":{"kind":"definition","key":key},"include_contents":true}});
    let filter = json!({"kind":"all_of","clause_id":"root","children":[
        {"kind":"any_of","clause_id":"choice","children":[
            {"kind":"relationship","clause_id":"native","relationship":{"direction":"incoming","selection":{"record_key":"bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6","embedded_path":[]},"include_contents":true}},
            definition("mapped", "sense:darkvision"),
            definition("standalone", "language:chthonian"),
            definition("zero", "language:draconic")
        ]},
        {"kind":"not","clause_id":"negative","predicate":definition("excluded", "language:common")}
    ]});
    let (status, page) = fixture.json(Method::POST, "/api/search/page", Some(json!({"criteria":{"kind":"list_records","filter":filter},"page":{"number":1,"size":20}}))).await;
    assert_eq!(status, StatusCode::OK, "{page}");
    let clauses = page["relationship_clauses"].as_array().unwrap();
    assert_eq!(
        clauses.len(),
        5,
        "all clauses retain presentation, including unmatched and negative leaves"
    );
    let clause = |id: &str| {
        clauses
            .iter()
            .find(|value| value["clause_id"] == id)
            .unwrap()
    };
    assert_eq!(clause("standalone")["condition"]["kind"], "definition");
    assert_eq!(
        clause("standalone")["destination"],
        json!({"kind":"definition","key":"language:chthonian"})
    );
    assert!(clause("standalone").get("record").is_none());
    assert_eq!(clause("mapped")["destination"]["kind"], "record");
    assert!(
        clause("mapped")["destination"]
            .get("reading_context")
            .is_none()
    );
    assert_eq!(
        clause("mapped")["record"]["record_key"],
        "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6"
    );
    let rows = page["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{page}");
    let action = rows
        .iter()
        .find(|row| row["record"]["record_key"] == ACTION)
        .unwrap();
    let evidence = action["relationship_evidence"].as_array().unwrap();
    assert!(
        evidence
            .iter()
            .any(|entry| entry["clause_id"] == "native"
                && entry["occurrences"][0]["kind"] == "record")
    );
    assert!(
        evidence.iter().any(|entry| entry["clause_id"] == "mapped"
            && entry["occurrences"][0]["kind"] == "definition")
    );
    assert!(
        evidence
            .iter()
            .all(|entry| entry["clause_id"] != "zero" && entry["clause_id"] != "excluded")
    );
    let native = evidence
        .iter()
        .find(|entry| entry["clause_id"] == "native")
        .unwrap();
    assert_eq!(native["occurrences"][0]["destination"]["kind"], "record");
    assert!(
        native["occurrences"][0]["destination"]
            .get("reading_context")
            .is_none(),
        "ordinary relationship navigation retains its ordinary purpose"
    );
    let (status, mapped_proof) = fixture.json(Method::POST, "/api/search/relationship-occurrences", Some(json!({"filter":filter,"clause_id":"mapped","result_record_key":ACTION,"page":{"number":1,"size":100},"expected_artifact_generation":page["artifact_generation"]}))).await;
    assert_eq!(status, StatusCode::OK, "{mapped_proof}");
    let graph = mapped_proof["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["kind"] == "record")
        .expect("definition reference proofs retain the existing authored UUID graph evidence");
    assert!(graph["destination"].get("reading_context").is_none());
    assert_eq!(
        graph["destination"]["navigation"]["record_key"],
        "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6"
    );
    let generation = &page["artifact_generation"];
    let request = |clause: &str| json!({"filter":filter,"clause_id":clause,"result_record_key":ACTOR,"page":{"number":1,"size":1},"expected_artifact_generation":generation});
    let (status, proof) = fixture
        .json(
            Method::POST,
            "/api/search/relationship-occurrences",
            Some(request("standalone")),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{proof}");
    assert_eq!(proof["clause"]["destination"]["kind"], "definition");
    assert_eq!(proof["occurrences"][0]["kind"], "definition");
    let occurrence = &proof["occurrences"][0]["occurrence"];
    assert_eq!(occurrence["source"]["selection"]["record_key"], ACTOR);
    assert_eq!(
        occurrence["source"]["selection"]["embedded_path"][0]["collection"],
        "/items"
    );
    assert_eq!(occurrence["source_field"], "/system/description/value");
    assert_eq!(
        occurrence["source"]["selection"]["field"],
        occurrence["source_field"]
    );
    assert_eq!(
        occurrence["source_destination"]["navigation"]["record_key"],
        ACTOR
    );
    assert!(
        occurrence["source_destination"]
            .get("reading_context")
            .is_none()
    );
    assert!(occurrence["occurrence_path"].is_string());
    assert_eq!(
        occurrence["target"],
        json!({"kind":"definition","key":"language:chthonian"})
    );
    assert_eq!(
        occurrence["target_destination"],
        json!({"kind":"definition","key":"language:chthonian"})
    );
    assert!(clause("zero")["destination"].is_null());
    for id in ["zero", "excluded"] {
        let (status, _) = fixture
            .json(
                Method::POST,
                "/api/search/relationship-occurrences",
                Some(request(id)),
            )
            .await;
        assert!(status.is_client_error(), "{id}: {status}");
    }
}

#[tokio::test]
async fn bodyless_reference_identity_survives_http_without_reading_destinations() {
    let fixture = Fixture::with_bodyless_glossary_reference();
    let key = "rule:pf2e:Fixture.Rule.Hidden";
    let (status, hidden) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":key})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{hidden}");
    assert_eq!(hidden["help"]["availability"], "permission_unresolved");
    assert_eq!(
        hidden["help"].get("destination"),
        Some(&serde_json::Value::Null)
    );
    let predicate = &hidden["help"]["related_filters"][0]["predicate"];
    let (status, source) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:chthonian"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{source}");
    assert_eq!(
        source["help"]["destination"],
        json!({"kind":"definition","key":"language:chthonian"})
    );
    assert!(
        source["content"]["body"]["html"]
            .as_str()
            .unwrap()
            .contains("Typically spoken")
    );
    assert_eq!(
        source["references"][0]["target"],
        json!({"kind":"definition","key":key})
    );
    assert_eq!(
        source["references"][0].get("destination"),
        Some(&serde_json::Value::Null)
    );
    let (status, page) = fixture.json(Method::POST, "/api/search/page", Some(json!({"criteria":{"kind":"list_records","filter":predicate},"page":{"number":1,"size":20},"expected_artifact_generation":hidden["artifact_generation"]}))).await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(page["rows"].as_array().unwrap().len(), 1);
    assert_eq!(page["rows"][0]["record"]["record_key"], ACTOR);
    assert_eq!(
        page["relationship_clauses"][0].get("destination"),
        Some(&serde_json::Value::Null)
    );
    assert_eq!(
        page["relationship_clauses"][0]["condition"]["reference"]["target"],
        json!({"kind":"definition","key":key})
    );
    let (status, proof) = fixture.json(Method::POST, "/api/search/relationship-occurrences", Some(json!({"filter":predicate,"clause_id":predicate["clause_id"],"result_record_key":ACTOR,"page":{"number":1,"size":1},"expected_artifact_generation":page["artifact_generation"]}))).await;
    assert_eq!(status, StatusCode::OK, "{proof}");
    let occurrence = &proof["occurrences"][0]["occurrence"];
    assert_eq!(occurrence["target"], json!({"kind":"definition","key":key}));
    assert_eq!(
        occurrence.get("target_destination"),
        Some(&serde_json::Value::Null)
    );
    assert_eq!(occurrence["target_title"], "Hidden rule");
    assert_eq!(occurrence["source"]["kind"], "record");
    assert_eq!(occurrence["source"]["selection"]["record_key"], ACTOR);
    assert_eq!(
        occurrence["source"]["selection"]["field"],
        "/system/description/value"
    );
    assert_eq!(
        occurrence["source"]["selection"]["embedded_path"],
        occurrence["source_destination"]["navigation"]["embedded_path"]
    );
    assert!(occurrence["source"]["selection"]["source_fingerprint"].is_string());
    assert_eq!(
        occurrence["source"]["selection"]["source_fingerprint"],
        occurrence["source_destination"]["navigation"]["source_fingerprint"]
    );
    assert_eq!(
        occurrence["source_destination"]["navigation"]["field"],
        occurrence["source_field"]
    );
    assert!(
        occurrence["source_destination"]
            .get("reading_context")
            .is_none()
    );
}

#[tokio::test]
async fn selected_non_catalog_records_keep_exact_identity_and_label_without_glossary_purpose() {
    let fixture = Fixture::with_glossary();
    let targets = [
        (
            json!({"kind":"record","selection":{"record_key":ACTOR,"embedded_path":[],"field":null,"passage":null,"source_fingerprint":null}}),
            "Test Creature",
        ),
        (
            json!({"kind":"record","selection":{"record_key":ACTOR,"embedded_path":[{"collection":"/items","identity":{"SnapshotLocal":{"index":0}}}],"field":"/system/description/value","passage":null,"source_fingerprint":fixture.source_fingerprint}}),
            "Claw",
        ),
    ];
    let (status, response) = fixture.json(Method::POST, "/api/filters/definitions", Some(json!({"text":"Chthonian","limit":1,"selected":targets.iter().map(|(target, _)| target).collect::<Vec<_>>()}))).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    let selected = response["selected"].as_array().unwrap();
    assert_eq!(selected.len(), targets.len());
    for ((target, label), selection) in targets.iter().zip(selected) {
        assert_eq!(
            &selection["target"], target,
            "selected identity must retain owners, field and fingerprint"
        );
        assert_eq!(selection["label"], *label);
        assert_ne!(selection["label"], ACTOR);
        assert_eq!(selection["help"]["availability"], "not_in_catalog");
        assert!(selection["help"]["destination"].is_null());
        assert_eq!(selection["help"]["related_filters"], json!([]));
        assert!(
            !response["options"]
                .as_array()
                .unwrap()
                .iter()
                .any(|option| option["target"] == *target),
            "retained selections are not promoted into the catalog"
        );
        let (status, page) = fixture.json(Method::POST, "/api/search/page", Some(json!({"criteria":{"kind":"list_records","filter":{"kind":"definition_reference","clause_id":"selected","reference":{"target":target,"include_contents":true}}},"page":{"number":1,"size":10},"expected_artifact_generation":response["artifact_generation"]}))).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "valid exact-record query admission is independent of glossary catalog membership: {page}"
        );
    }
}

#[tokio::test]
async fn non_glossary_filter_values_and_vocabulary_keep_readable_backend_labels() {
    let fixture = Fixture::with_glossary();
    for request in [
        json!({"kind":"vocabulary","scope":{"kind":"artifact"},"field_id":"physical.base_type","text":"chain-mail","limit":10,"selected_values":["chain-mail"]}),
        json!({"kind":"contextual","context":{"kind":"filtered","filter":null,"text":null,"mode":"hybrid","search_profile":"bge-base"},"field_id":"physical.base_type","text":"chain-mail","limit":10}),
    ] {
        let (status, values) = fixture
            .json(Method::POST, "/api/filters/values", Some(request))
            .await;
        assert_eq!(status, StatusCode::OK, "{values}");
        let data = if values["kind"] == "vocabulary" {
            &values["values"]
        } else {
            assert_eq!(values["outcome"]["status"], "available", "{values}");
            &values["outcome"]["value"]
        };
        let option = data["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| option["value"] == "chain-mail")
            .unwrap();
        assert_eq!(option["label"], "Chain Mail", "{values}");
        assert!(
            option.get("help").is_none(),
            "non-glossary values do not invent glossary help"
        );
    }
}
