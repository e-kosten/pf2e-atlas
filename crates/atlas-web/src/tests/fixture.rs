//! Small checked-source fixture; no query model assets or native inference.
use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddSavedListItemRequest, CreateEncounterRequest,
    CreateSavedListRequest,
};
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_domain::{SearchProfileId, SourceByteRange, SourcePassageAddress, TraitKey};
use atlas_index::{TraitDefinition, TraitSeedChunk, TraitVectorInput};
use atlas_search::test_support::{hash, input, publish_input, record};
use serde_json::json;

use super::AtlasWebState;

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
        atlas_index::test_support::corrupt_remaster_declaration(
            self._directory.path().join("workspace.sqlite"),
            atlas_runtime::WORKSPACE_SCHEMA_VERSION,
        )
        .unwrap();
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
        Self::build(remaster, adjustment, false)
    }
    pub fn with_former_name_only() -> Self {
        Self::build(true, None, true)
    }
    fn build(remaster: bool, adjustment: Option<&str>, former_name_only: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace.sqlite");
        let mut actor = json!({"_id":"cccccccccccccccc","type":"npc","name":"Test Creature","system":{"details":{"level":{"value":1}},"traits":{"value":[]},"attributes":{"ac":{"value":16},"hp":{"max":20}},"saves":{"fortitude":{"value":5},"reflex":{"value":6},"will":{"value":4}}},"items":[{"_id":"eeeeeeeeeeeeeeee","type":"melee","name":"Claw","system":{"bonus":{"value":7},"traits":{"value":["agile"]},"damageRolls":{"main":{"damage":"1d6+2","damageType":"slashing"}},"description":{"value":"<p>A claw.</p>"}}},{"_id":"eeeeeeeeeeeeeeee","type":"action","name":"Duplicate identity","system":{"description":{"value":"<p>Snapshot-local identity.</p>"}}}]});
        if remaster {
            actor["items"].as_array_mut().unwrap().push(json!({"_id":"owned00000000006","name":"Customized owned action","type":"action","_stats":{"compendiumSource":"Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb"},"system":{"publication":{"remaster":false},"description":{"value":"<p>Keep customized rules.</p>"}}}));
        }
        if let Some(adjustment) = adjustment {
            actor["system"]["attributes"]["adjustment"] = json!(adjustment);
        }
        let second_name = if former_name_only {
            "Force Barrage"
        } else {
            "Test Action 2"
        };
        let second_type = if former_name_only { "spell" } else { "action" };
        let mut second_system = json!({"publication":{"remaster":true},"description":{"value":"<p>A second flame.</p>"}});
        if former_name_only {
            second_system["level"] = json!({"value":1});
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
                json!({"_id":"bbbbbbbbbbbbbbbb","type":second_type,"name":second_name,"system":second_system}),
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
        if remaster {
            let (section, explanation) = if former_name_only {
                (
                    "Spells",
                    "<table><tr><th>Old</th><th>Status</th><th>New</th></tr><tr><td>Magic Missile</td><td>Renamed</td><td>@UUID[Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb]{Force Barrage}</td></tr></table>",
                )
            } else {
                (
                    "Class Features",
                    "<table><tr><th>Old</th><th>Status</th><th>New</th></tr><tr><td>@UUID[Compendium.pf2e.actions.Item.aaaaaaaaaaaaaaaa]{Test Action 1}</td><td>Renamed</td><td>@UUID[Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb]{Test Action 2}</td></tr><tr><td>@UUID[Compendium.pf2e.actions.Item.missing000000001]{Missing old action}</td><td>Replaced</td><td>@UUID[Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb]{Test Action 2}</td></tr></table>",
                )
            };
            records.push(record("journals", "JournalEntry", json!({"_id":"editionjournal01","name":"Remaster Changes","pages":[{"_id":"editionpage00001","name":section,"type":"text","text":{"content":explanation,"format":1}}]})));
        }
        let mut data = input(records, true);
        if remaster {
            atlas_index::test_support::capture_remaster_declarations(&mut data).unwrap();
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
        });
        let mut vector = vec![0.; 768];
        vector[0] = 1.;
        let chunk_hash = hash(b"Fire\n");
        data.trait_vectors.push(TraitVectorInput {
            key,
            profile_id: SearchProfileId::new("bge-base"),
            input_hash: hash(format!("trait-definition-centroid/v1:{chunk_hash}").as_bytes()),
            vector: vector.clone(),
            chunks: vec![TraitSeedChunk {
                input: "Fire\n".into(),
                input_hash: chunk_hash.clone(),
                token_count: 2,
                vector,
                address: SourcePassageAddress::PlainSection {
                    source_text_sha256: chunk_hash,
                    selection_version: "trait-definition-centroid/v1".into(),
                    section_ordinal: 0,
                    label: Some("Fire".into()),
                    chunk_bytes: SourceByteRange { start: 0, end: 5 },
                },
            }],
        });
        let source_fingerprint = data.context.source_fingerprint.clone();
        publish_input(&path, &data).unwrap();
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
