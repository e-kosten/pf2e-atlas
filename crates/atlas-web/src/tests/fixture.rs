//! Small checked-source fixture; no query model assets or native inference.
use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddSavedListItemRequest, CreateEncounterRequest,
    CreateSavedListRequest,
};
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_domain::{SearchProfileId, TraitKey};
use atlas_index::TraitDefinition;
use atlas_search::test_support::{input, publish_input, record};
use serde_json::json;

use super::AtlasWebState;
#[path = "../../examples/browser_fixture/reference_data.rs"]
mod reference_data;

pub(super) const ACTION: &str = "actions:aaaaaaaaaaaaaaaa";
pub(super) const SECOND_ACTION: &str = "actions:bbbbbbbbbbbbbbbb";
pub(super) const ACTOR: &str = "actors:cccccccccccccccc";
pub(super) const TABLE: &str = "tables:dddddddddddddddd";

pub(super) struct Fixture {
    pub service: AtlasAppService,
    pub encounter_key: String,
    pub participant_key: String,
    pub list_key: String,
    pub source_fingerprint: String,
    // Keep the database and isolated source/model paths alive after every handle.
    _directory: tempfile::TempDir,
}

impl Fixture {
    pub fn corrupt_remaster_evidence(&self) {
        let workspace =
            atlas_runtime::initialize_workspace(self._directory.path().join("workspace.sqlite"))
                .unwrap();
        let write = workspace
            .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        assert_eq!(
            write
                .connection()
                .execute("UPDATE remaster_pairs SET evidence_json='{}'", [])
                .unwrap(),
            1
        );
        write.commit().unwrap();
    }

    pub fn without_reference() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let service = AtlasAppService::start(AtlasAppServiceOptions {
            default_search_profile: None,
            path_mode: atlas_runtime::AtlasPathMode::Global,
            source_root: Some(directory.path().join("unused-source")),
            embedding_cache_root: Some(directory.path().join("unused-models")),
            index_path: Some(directory.path().join("absent.sqlite")),
            retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
        })
        .unwrap();
        Self {
            service,
            encounter_key: String::new(),
            participant_key: String::new(),
            list_key: String::new(),
            source_fingerprint: String::new(),
            _directory: directory,
        }
    }
    pub fn new() -> Self {
        Self::with_remaster(false)
    }
    pub fn with_remaster(remaster: bool) -> Self {
        Self::with_actor_adjustment(remaster, None)
    }
    pub fn with_actor_adjustment(remaster: bool, adjustment: Option<&str>) -> Self {
        Self::with_reference_data(remaster, adjustment, None, 16)
    }
    pub fn with_glossary() -> Self {
        Self::with_reference_data(false, None, Some(true), 16)
    }
    pub fn with_legacy_glossary() -> Self {
        Self::with_reference_data(false, None, Some(false), 16)
    }
    pub fn with_actor_armor_class(armor_class: i64) -> Self {
        Self::with_reference_data(false, None, None, armor_class)
    }
    fn with_reference_data(
        remaster: bool,
        adjustment: Option<&str>,
        coverage: Option<bool>,
        armor_class: i64,
    ) -> Self {
        let glossary = coverage == Some(true);
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace.sqlite");
        let mut actor = json!({"_id":"cccccccccccccccc","type":"npc","name":"Test Creature","system":{"details":{"level":{"value":1}},"traits":{"value":[]},"attributes":{"ac":{"value":armor_class},"hp":{"max":20}},"saves":{"fortitude":{"value":5},"reflex":{"value":6},"will":{"value":4}}},"items":[{"_id":"eeeeeeeeeeeeeeee","type":"melee","name":"Claw","system":{"bonus":{"value":7},"traits":{"value":["agile"]},"damageRolls":{"main":{"damage":"1d6+2","damageType":"slashing"}},"description":{"value":"<p>A claw.</p>"}}},{"_id":"eeeeeeeeeeeeeeee","type":"action","name":"Duplicate identity","system":{"description":{"value":"<p>Snapshot-local identity.</p>"}}}]});
        if let Some(adjustment) = adjustment {
            actor["system"]["attributes"]["adjustment"] = json!(adjustment);
        }
        if coverage.is_some() {
            actor["system"]["details"]["languages"] =
                json!({"value":["common","chthonian"],"details":"Communicates by signing."});
            actor["system"]["perception"] =
                json!({"mod":7,"senses":[{"type":"darkvision","acuity":"precise","range":60}]});
        }
        if glossary {
            actor["items"][0]["system"]["description"]["value"] =
                json!("<p>@Localize[Fixture.Language.Chthonian]</p>");
        }
        let mut records = vec![
            record(
                "actions",
                "Item",
                json!({"_id":"aaaaaaaaaaaaaaaa","type":"action","name":"Test Action 1","system":{"publication":{"remaster":false},"actionType":{"value":"action"},"actions":{"value":1},"traits":{"value":["fire"]},"description":{"value":"<p>Flame @Damage[1d6[fire]] burns.</p>"}}}),
            ),
            record(
                "actions",
                "Item",
                json!({"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Test Action 2","system":{"publication":{"remaster":true},"description":{"value":"<p>A second flame.</p>"}}}),
            ),
            record("actors", "Actor", actor),
            record(
                "tables",
                "RollTable",
                json!({"_id":"dddddddddddddddd","name":"Signal","formula":"1d2","results":[{"_id":"ffffffffffffffff","type":"text","range":[1,2],"weight":1,"text":"A lantern flashes."}]}),
            ),
            record(
                "conditionitems",
                "Item",
                json!({"_id":"TBSHQspnbcqxsmjL","type":"condition","name":"Frightened","system":{"slug":"frightened","value":{"isValued":true,"value":1}}}),
            ),
        ];
        if glossary {
            records[0] = record(
                "actions",
                "Item",
                json!({"_id":"aaaaaaaaaaaaaaaa","type":"action","name":"Test Action 1","system":{"actionType":{"value":"action"},"actions":{"value":1},"traits":{"value":["fire"]},"description":{"value":"<p>@Localize[Fixture.Language.Chthonian] @Localize[Fixture.Sense.Darkvision] @UUID[Compendium.pf2e.bestiary-ability-glossary-srd.Item.qCCLZhnp2HhP3Ex6]</p>"}}}),
            );
        }
        if remaster {
            records.push(record("journals", "JournalEntry", json!({"_id":"editionjournal01","name":"Remaster Changes","pages":[{"_id":"editionpage00001","name":"Class Features","type":"text","text":{"content":"<p>Explicit old and new references.</p>","format":1}}]})));
        }
        if glossary {
            records.push(reference_data::darkvision_record());
            records.push(record("equipment", "Item", json!({"_id":"armor00000000000","type":"armor","name":"Fixture Chain Mail","system":{"baseItem":"chain-mail"}})));
        }
        let mut data = input(records, true);
        if remaster {
            data.remaster_pairs
                .push(atlas_index::SourceRemasterPairInput {
                    legacy: ACTION.parse().unwrap(),
                    remaster: SECOND_ACTION.parse().unwrap(),
                    provenance: Some(serde_json::from_value(json!({"record":"journals:editionjournal01","owners":[{"collection":"/pages","identity":{"Stable":"editionpage00001"}}],"field":"/text/content"})).unwrap()),
                });
        }
        let key = TraitKey::new("fire").unwrap();
        data.traits.push(TraitDefinition {
            key: key.clone(),
            label: "Fire".into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec![],
            source_locations: vec![],
            source_revision: None,
            indexing_locale: "en".into(),
            content_locale: None,
        });
        let source_fingerprint = data.context.source_fingerprint.clone();
        if glossary {
            let glossary = reference_data::attach(&mut data).unwrap();
            atlas_index::test_support::publish_input_with_references(
                &path,
                &data,
                &glossary.entries,
                &glossary.notices,
            )
            .unwrap();
        } else {
            publish_input(&path, &data).unwrap();
        }
        let service = AtlasAppService::start(AtlasAppServiceOptions {
            default_search_profile: Some(SearchProfileId::new("bge-base")),
            path_mode: atlas_runtime::AtlasPathMode::Global,
            source_root: Some(directory.path().join("unused-source")),
            embedding_cache_root: Some(directory.path().join("unused-models")),
            index_path: Some(path),
            retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
        })
        .unwrap();
        let encounter_key = service
            .create_encounter(CreateEncounterRequest {
                name: "Ambush".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter
            .encounter_key;
        service
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: encounter_key.clone(),
                display_name: "Kyra".into(),
                max_hp: Some(24),
                current_hp: Some(20),
                initiative: Some(15),
            })
            .unwrap();
        let participant_key = service.encounter(&encounter_key).unwrap().participants[0]
            .participant_key
            .clone();
        let list_key = service
            .create_saved_list(CreateSavedListRequest {
                slug: "research".into(),
                name: "Research".into(),
                description: None,
                tags: vec![],
            })
            .unwrap()
            .list
            .list_key;
        service
            .add_saved_list_item(AddSavedListItemRequest {
                list_ref: list_key.clone(),
                record_ref: ACTION.into(),
                note: Some("fixture note".into()),
            })
            .unwrap();
        Self {
            service,
            encounter_key,
            participant_key,
            list_key,
            source_fingerprint,
            _directory: directory,
        }
    }
    pub fn state(&self, capacity: usize) -> AtlasWebState {
        AtlasWebState::with_blocking_capacity(self.service.clone(), capacity)
    }
    pub fn router(&self) -> axum::Router {
        crate::router(self.service.clone())
    }
    pub async fn json(
        &self,
        method: axum::http::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> (axum::http::StatusCode, serde_json::Value) {
        use tower::ServiceExt;
        let mut request = axum::http::Request::builder().method(method).uri(path);
        let body = if let Some(body) = body {
            request = request.header("content-type", "application/json");
            axum::body::Body::from(body.to_string())
        } else {
            axum::body::Body::empty()
        };
        super::response_json(
            self.router()
                .oneshot(request.body(body).unwrap())
                .await
                .unwrap(),
        )
        .await
    }
}
