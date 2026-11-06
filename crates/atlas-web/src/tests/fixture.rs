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
#[path = "../../examples/browser_fixture/reference_data.rs"]
mod reference_data;

fn add_bodyless_reference(
    data: &mut atlas_index::IndexBuildInput,
    glossary: &mut reference_data::ReferenceFixture,
) {
    let key: atlas_domain::DefinitionKey = "rule:pf2e:Fixture.Rule.Hidden".parse().unwrap();
    let mut entry = glossary
        .entries
        .iter()
        .find(|entry| entry.key.to_string() == "language:draconic")
        .unwrap()
        .clone();
    entry.key = key.clone();
    entry.label = "Hidden rule".into();
    entry.availability = atlas_index::ReferenceAvailability::PermissionUnresolved;
    entry.permission.disposition =
        atlas_index::ReferencePermissionDisposition::MetadataOnlyUnresolved;
    entry.provenance.label_key = Some("Fixture.Rule.Hidden".into());
    entry.filter_bindings = vec![atlas_index::ReferenceFilterBinding {
        id: "references".into(),
        relation: "references_definition".into(),
        field: None,
        value: None,
    }];
    glossary.entries.push(entry);
    let keys = [(
        "Fixture.Rule.Hidden".into(),
        [key.clone()].into_iter().collect(),
    )]
    .into_iter()
    .collect();
    glossary.occurrences.extend(
        data.records
            .iter()
            .flat_map(|source| {
                atlas_record::source_record::definition_record_localization_references(
                    &source.record,
                    data.context.audience,
                    &keys,
                )
            })
            .map(|reference| atlas_index::DefinitionReferenceInput {
                source: atlas_index::DefinitionReferenceSource::Record {
                    key: reference.locator.record,
                    owners: reference.locator.owners,
                    field: Some(reference.locator.field),
                },
                target: atlas_index::DefinitionReferenceTarget::Definition { key: key.clone() },
                occurrence_path: reference.path,
                evidence: reference.localization_key,
                label: None,
                visible: reference.visible,
            }),
    );
    let source = glossary
        .entries
        .iter_mut()
        .find(|entry| entry.key.to_string() == "language:chthonian")
        .unwrap();
    let body = "<p>Typically spoken by demons. @Localize[Fixture.Rule.Hidden]</p>";
    source.backing = atlas_index::ReferenceBacking::Prepared {
        authored_markup: body.into(),
        content: atlas_record::source_content::prepare_reference_definition(
            source.key.clone(),
            body,
            data.context.audience,
            None,
            None,
        )
        .unwrap(),
    };
    source.provenance.source_sha256 = Some(hash(body.as_bytes()));
    glossary.occurrences.extend(
        atlas_record::source_content::definition_body_localization_references(
            body,
            &keys,
            data.context.audience,
            atlas_record::source_content::ContentVisibilityRule::All,
        )
        .into_iter()
        .map(|reference| atlas_index::DefinitionReferenceInput {
            source: atlas_index::DefinitionReferenceSource::Definition {
                key: source.key.clone(),
            },
            target: atlas_index::DefinitionReferenceTarget::Definition { key: reference.key },
            occurrence_path: reference.path,
            evidence: reference.localization_key,
            label: None,
            visible: reference.visible,
        }),
    );
    let (count, digest) = atlas_index::reference_inventory_identity(
        data.traits
            .iter()
            .map(|entry| atlas_domain::DefinitionKey::from(entry.key.clone()))
            .chain(glossary.entries.iter().map(|entry| entry.key.clone())),
    )
    .unwrap();
    let preparation = data.context.reference_preparation.as_mut().unwrap();
    preparation.expected_entry_count = count;
    preparation.entry_keys_sha256 = digest;
    preparation.input_hash =
        hash(&serde_json::to_vec(&(&glossary.entries, &glossary.notices)).unwrap());
}

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
        Self::with_reference_data(remaster, adjustment, None, false)
    }
    pub fn with_glossary() -> Self {
        Self::with_reference_data(false, None, Some(true), false)
    }
    pub fn with_bodyless_glossary_reference() -> Self {
        Self::with_reference_data(false, None, Some(true), true)
    }
    pub fn with_legacy_glossary() -> Self {
        Self::with_reference_data(false, None, Some(false), false)
    }
    fn with_reference_data(
        remaster: bool,
        adjustment: Option<&str>,
        coverage: Option<bool>,
        bodyless_reference: bool,
    ) -> Self {
        let glossary = coverage == Some(true);
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace.sqlite");
        let mut actor = json!({"_id":"cccccccccccccccc","type":"npc","name":"Test Creature","system":{"details":{"level":{"value":1}},"traits":{"value":[]},"attributes":{"ac":{"value":16},"hp":{"max":20}},"saves":{"fortitude":{"value":5},"reflex":{"value":6},"will":{"value":4}}},"items":[{"_id":"eeeeeeeeeeeeeeee","type":"melee","name":"Claw","system":{"bonus":{"value":7},"traits":{"value":["agile"]},"damageRolls":{"main":{"damage":"1d6+2","damageType":"slashing"}},"description":{"value":"<p>A claw.</p>"}}},{"_id":"eeeeeeeeeeeeeeee","type":"action","name":"Duplicate identity","system":{"description":{"value":"<p>Snapshot-local identity.</p>"}}}]});
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
            actor["items"][0]["system"]["description"]["value"] = json!(if bodyless_reference {
                "<p>@Localize[Fixture.Language.Chthonian] @Localize[Fixture.Rule.Hidden]</p>"
            } else {
                "<p>@Localize[Fixture.Language.Chthonian]</p>"
            });
        }
        let mut records = vec![
            record(
                "actions",
                "Item",
                json!({"_id":"aaaaaaaaaaaaaaaa","type":"action","name":"Test Action 1","system":{"actionType":{"value":"action"},"actions":{"value":1},"traits":{"value":["fire"]},"description":{"value":"<p>Flame @Damage[1d6[fire]] burns.</p>"}}}),
            ),
            record(
                "actions",
                "Item",
                json!({"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Test Action 2","system":{"description":{"value":"<p>A second flame.</p>"}}}),
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
                    provenance: serde_json::from_value(json!({"record":"journals:editionjournal01","owners":[{"collection":"/pages","identity":{"Stable":"editionpage00001"}}],"field":"/text/content"})).unwrap(),
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
        if glossary {
            let mut glossary = reference_data::attach(&mut data).unwrap();
            if bodyless_reference {
                add_bodyless_reference(&mut data, &mut glossary);
            }
            atlas_index::test_support::publish_input_with_references(
                &path,
                &data,
                &glossary.entries,
                &glossary.notices,
                &glossary.occurrences,
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
