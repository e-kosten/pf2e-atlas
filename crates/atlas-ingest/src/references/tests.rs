use super::*;
use crate::BuildArtifactOptions;
use atlas_index::{ReferenceCoverage, test_support};
use rusqlite::Connection;
use std::{fs, path::PathBuf};

const DARKVISION: &str = "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6";
const AIR: &str = "PF2E.Item.Deity.Domain.Air.Description";
const AGILE: &str = "PF2E.TraitDescriptionAgile";

#[test]
fn structured_language_speakers_have_context_and_preserve_source_names() {
    for (speakers, expected) in [
        (vec!["Demons"], "demons"),
        (
            vec!["Dragons", "reptilian humanoids"],
            "dragons and reptilian humanoids",
        ),
        (
            vec!["Fey", "centaurs", "plant", "fungus creatures"],
            "fey, centaurs, plant and fungus creatures",
        ),
        (
            vec!["Nidalese", "Netherworld creatures"],
            "Nidalese and Netherworld creatures",
        ),
        (vec!["Absalom", "Golarion"], "Absalom and Golarion"),
        (
            vec!["<new speakers>", "A&B"],
            "&lt;new speakers&gt; and A&amp;B",
        ),
    ] {
        let speakers = speakers.into_iter().map(str::to_owned).collect::<Vec<_>>();
        assert_eq!(
            language_speaker_markup(&speakers).unwrap(),
            format!("<p><strong>Spoken by</strong> {expected}</p>")
        );
    }
    assert!(language_speaker_markup(&[]).is_none());
}

#[test]
fn prepared_language_speaker_context_is_readable_without_changing_other_prose() {
    let fixture = Fixture::new("pf2e");
    fixture.build().unwrap();
    for (key, expected) in [
        ("language:chthonian", "Spoken by demons"),
        (
            "language:draconic",
            "Spoken by dragons and reptilian humanoids",
        ),
        (
            "language:common",
            "Spoken by humans, dwarves, elves, halflings and other common ancestries",
        ),
        (
            "language:shadowtongue",
            "Spoken by Nidalese and Netherworld creatures",
        ),
        ("domain:air", "You can control winds and the weather."),
    ] {
        let ReferenceBacking::Prepared { content, .. } = fixture.entry(key).backing else {
            panic!("prepared {key}")
        };
        assert_eq!(content.text, expected, "{key}");
        if key.starts_with("language:") {
            assert!(
                content.html.starts_with("<p><strong>Spoken by</strong> "),
                "{key}: {}",
                content.html
            );
            assert!(!content.html.contains("Spoken by:"));
        }
    }
    let ReferenceBacking::Prepared {
        authored_markup,
        content,
    } = fixture.entry("weapon_group:dart").backing
    else {
        panic!("prepared weapon specialization")
    };
    assert!(!content.text.starts_with("Spoken by"));
    assert!(!authored_markup.contains("<strong>Spoken by</strong>"));
}
const FAMILY_ABILITIES: &[(&str, &str)] = &[
    ("r34QDwKiWZoVymJa", "GolemAntimagic"),
    ("xS8ybzuqPSi3Jb8k", "WindUp"),
];

struct Fixture {
    directory: tempfile::TempDir,
    source: PathBuf,
    database: PathBuf,
}
impl Fixture {
    fn new(package: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        let database = directory.path().join("workspace.sqlite");
        for path in [
            "packs/items",
            "packs/general",
            "static/lang",
            "static/licenses",
        ] {
            fs::create_dir_all(source.join(path)).unwrap();
        }
        fs::write(source.join("module.json"), serde_json::json!({"id":package,"version":"8.6.1","packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"},{"name":"bestiary-ability-glossary-srd","label":"Abilities","type":"Item","path":"packs/general"}]}).to_string()).unwrap();
        fs::write(
            source.join("static/lang/en.json"),
            serde_json::json!({
                "PF2E.Actor.Creature.Language.chthonian":"Chthonian",
                "PF2E.Actor.Creature.Sense.Type.Darkvision":"Darkvision",
                "PF2E.Item.Deity.Domain.Air.Label":"Air",
                "PF2E.Item.Deity.Domain.Air.Description":"You can control winds and the weather.",
                "PF2E.TraitAgile":"Agile", "PF2E.TraitDescriptionAgile":format!("@Localize[{AIR}]"),
                "PF2E.NPC.Abilities.Glossary.Darkvision":"A fixture definition."
            })
            .to_string(),
        )
        .unwrap();
        let notices: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../reference-data/notices.json")).unwrap();
        let notice = notices["notices"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == "player-core")
            .unwrap()["text"]
            .as_str()
            .unwrap();
        fs::write(source.join("static/licenses/ORCLicense.md"), notice).unwrap();
        let fixture = Self {
            directory,
            source,
            database,
        };
        fixture.darkvision(
            "action",
            "A customized definition. @Localize[PF2E.NPC.Abilities.Glossary.Darkvision]",
        );
        fixture.observer(&format!("<p>@Localize[{AGILE}] @Localize[PF2E.NPC.Abilities.Glossary.Darkvision]</p><p data-visibility=\"none\">@Localize[{AIR}]</p>"));
        fixture
    }
    fn darkvision(&self, family: &str, body: &str) {
        fs::write(self.source.join("packs/general/darkvision.json"), serde_json::json!({"_id":"qCCLZhnp2HhP3Ex6","name":"Selected Darkvision","type":family,"system":{"description":{"value":body}}}).to_string()).unwrap();
    }
    fn observer(&self, body: &str) {
        fs::write(self.source.join("packs/items/observer.json"), serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Observer","type":"effect","system":{"description":{"value":body},"traits":{"value":["agile"]}}}).to_string()).unwrap();
    }
    fn family_abilities(&self) {
        fs::create_dir_all(self.source.join("packs/family")).unwrap();
        let manifest_path = self.source.join("module.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["packs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name":"bestiary-family-ability-glossary", "label":"Family abilities",
                "type":"Item", "path":"packs/family"
            }));
        fs::write(manifest_path, manifest.to_string()).unwrap();
        let catalog_path = self.source.join("static/lang/en.json");
        let mut catalog: serde_json::Value =
            serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
        for (id, name) in FAMILY_ABILITIES {
            let item = serde_json::json!({"_id":id,"name":name,"type":"action","system":{"description":{"value":format!("<p>@Damage[(0)[bludgeoning]], @Check[fortitude|dc:|basic]</p><p>@Localize[PF2E.NPC.Abilities.Glossary.{name}]</p>")}}});
            fs::write(
                self.source.join(format!("packs/family/{name}.json")),
                item.to_string(),
            )
            .unwrap();
            catalog[format!("PF2E.NPC.Abilities.Glossary.{name}")] = serde_json::json!(format!(
                "Reusable {name} rules, including @Damage[2d6[untyped]]."
            ));
        }
        fs::write(catalog_path, catalog.to_string()).unwrap();
    }
    fn options(&self) -> BuildArtifactOptions {
        BuildArtifactOptions {
            source_root: self.source.clone(),
            output_path: self.database.clone(),
            manifest_path: None,
            locale: "en".into(),
            search_profiles: vec![],
            reuse_embeddings: true,
            reuse_passage_layouts: true,
            embedding_batch_size: 8,
        }
    }
    fn pack_system(&self, pack: &str, system: Option<serde_json::Value>) {
        let path = self.source.join("module.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let declaration = manifest["packs"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|declaration| declaration["name"] == pack)
            .unwrap()
            .as_object_mut()
            .unwrap();
        if let Some(system) = system {
            declaration.insert("system".into(), system);
        } else {
            declaration.remove("system");
        }
        fs::write(path, manifest.to_string()).unwrap();
    }
    fn build(&self) -> Result<crate::BuildArtifactReport, IngestError> {
        let workspace = test_support::initialize_workspace(&self.database).unwrap();
        crate::build_artifact(
            &workspace,
            test_support::TEST_WORKSPACE_SCHEMA_VERSION,
            self.options(),
        )
    }
    fn entry(&self, key: &str) -> ReferenceEntry {
        test_support::open_reader(&self.database)
            .unwrap()
            .reference_entry(&key.parse().unwrap())
            .unwrap()
            .unwrap()
    }
}

#[test]
fn captured_pack_system_controls_only_new_associations_and_restores_on_omission() {
    let fixture = Fixture::new("pf2e");
    fixture.family_abilities();
    fixture.observer(&format!("<p>@Localize[{AGILE}] @Localize[{AIR}] @Localize[PF2E.NPC.Abilities.Glossary.Darkvision]</p>"));
    let declarations = [
        Some(serde_json::json!("sf2e")), // initial full publication
        None,                            // compatible export inherits root
        Some(serde_json::json!("pf2e")), // explicit matching declaration
        Some(serde_json::Value::Null),   // malformed explicit declaration remains admitted
        Some(serde_json::json!(17)),
        None, // incremental restoration
    ];
    for system in declarations {
        let applicable = system.is_none() || system.as_ref().is_some_and(|value| value == "pf2e");
        for pack in [
            "items",
            "bestiary-ability-glossary-srd",
            "bestiary-family-ability-glossary",
        ] {
            fixture.pack_system(pack, system.clone());
        }
        fixture.build().unwrap();
        let reader = test_support::open_reader(&fixture.database).unwrap();
        assert_eq!(
            fixture.entry("sense:darkvision").availability,
            if applicable {
                ReferenceAvailability::Available
            } else {
                ReferenceAvailability::SourceInapplicable
            }
        );
        for (id, name) in FAMILY_ABILITIES {
            let key = format!("bestiary-family-ability-glossary:{id}")
                .parse()
                .unwrap();
            let bundle = reader.read_content_for_record(&key, 16).unwrap();
            assert_eq!(
                bundle.fields.len(),
                1,
                "ordinary record survives every source-system state"
            );
            let entry = fixture.entry(&format!("rule:pf2e:PF2E.NPC.Abilities.Glossary.{name}"));
            assert_eq!(
                matches!(entry.backing, ReferenceBacking::Record { .. }),
                applicable
            );
        }
        let observer: RecordKey = "items:aaaaaaaaaaaaaaaa".parse().unwrap();
        assert_eq!(
            reader
                .read_content_for_record(&observer, 16)
                .unwrap()
                .fields
                .len(),
            1
        );
        assert!(
            reader
                .trait_reference_record_keys(&"trait:agile".parse().unwrap())
                .unwrap()
                .contains(&observer)
        );
        let preparation = reader.context().reference_preparation.as_ref().unwrap();
        assert_eq!(preparation.source_package.as_deref(), Some("pf2e"));
        assert_eq!(
            preparation.pack_systems.get("items"),
            system
                .as_ref()
                .map(|value| value.as_str().map(str::to_owned))
                .as_ref()
        );
        atlas_index::validate_artifact(&fixture.database).unwrap();
    }
}

#[test]
fn pack_system_scope_is_owned_by_capture_without_tightening_manifest_admission() {
    let fixture = Fixture::new("pf2e");
    let captured = CapturedBuildInputs::capture(fixture.options()).unwrap();
    fixture.pack_system("items", Some(serde_json::json!("sf2e")));
    let later = CapturedBuildInputs::capture(fixture.options()).unwrap();
    assert!(!captured.reference_pack_systems().contains_key("items"));
    assert_eq!(
        later.reference_pack_systems().get("items"),
        Some(&Some("sf2e".into()))
    );
    assert_ne!(
        captured.reference_inputs_identity(),
        later.reference_inputs_identity()
    );
    let manifest = br#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items","system":17,"system":"pf2e"}]}"#;
    let parsed = crate::source::discovery::parse_manifest_bytes(manifest).unwrap();
    assert_eq!(
        parsed.manifest.packs[0].metadata.get("system"),
        Some(&serde_json::json!("pf2e")),
        "previously unknown duplicate fields retain their prior last-value semantics"
    );
}

#[test]
fn family_abilities_reuse_the_actual_body_with_authored_placeholders_and_custom_values() {
    let fixture = Fixture::new("pf2e");
    fixture.family_abilities();
    fixture.build().unwrap();
    for (id, name) in FAMILY_ABILITIES {
        let target: RecordKey = format!("bestiary-family-ability-glossary:{id}")
            .parse()
            .unwrap();
        let entry = fixture.entry(&format!("rule:pf2e:PF2E.NPC.Abilities.Glossary.{name}"));
        assert_eq!(
            entry.backing,
            ReferenceBacking::Record {
                key: target.clone()
            }
        );
        let reader = test_support::open_reader(&fixture.database).unwrap();
        let bundle = reader.read_content_for_record(&target, 16).unwrap();
        let body = bundle
            .fields
            .iter()
            .find(|field| field.locator.field == "/system/description/value")
            .unwrap();
        assert!(body.html.as_ref().unwrap().contains("0"));
        assert!(!body.interactions.is_empty());
        assert!(
            body.html
                .as_ref()
                .unwrap()
                .contains(&format!("Reusable {name} rules"))
        );
        assert_eq!(
            bundle.fields.len(),
            1,
            "one canonical field, no reference-body copy"
        );
    }
    atlas_index::validate_artifact(&fixture.database).unwrap();

    let path = fixture.source.join("packs/family/GolemAntimagic.json");
    let mut item: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    item["system"]["description"]["value"] = serde_json::json!(
        "<p>Actual creature parameters.</p><p>@Localize[PF2E.NPC.Abilities.Glossary.GolemAntimagic]</p>"
    );
    fs::write(path, item.to_string()).unwrap();
    fs::remove_file(fixture.source.join("packs/family/WindUp.json")).unwrap();
    fixture.build().unwrap();
    let target: RecordKey = "bestiary-family-ability-glossary:r34QDwKiWZoVymJa"
        .parse()
        .unwrap();
    assert_eq!(
        fixture
            .entry("rule:pf2e:PF2E.NPC.Abilities.Glossary.GolemAntimagic")
            .backing,
        ReferenceBacking::Record {
            key: target.clone()
        }
    );
    let reader = test_support::open_reader(&fixture.database).unwrap();
    let bundle = reader.read_content_for_record(&target, 16).unwrap();
    assert!(
        bundle.fields[0]
            .html
            .as_ref()
            .unwrap()
            .contains("Actual creature parameters.")
    );
    assert_eq!(
        fixture
            .entry("rule:pf2e:PF2E.NPC.Abilities.Glossary.WindUp")
            .availability,
        ReferenceAvailability::MissingTarget
    );
    atlas_index::validate_artifact(&fixture.database).unwrap();
}

#[test]
fn source_free_publication_reuses_actual_record_and_preserves_attribution() {
    let fixture = Fixture::new("pf2e");
    let report = fixture.build().unwrap();
    assert_eq!(report.inferred_inputs, 0);
    atlas_index::validate_artifact(&fixture.database).unwrap();
    let reader = test_support::open_reader(&fixture.database).unwrap();
    assert_eq!(reader.reference_coverage(), ReferenceCoverage::Complete);
    drop(reader);
    let connection = Connection::open(&fixture.database).unwrap();
    let languages: usize = connection
        .query_row(
            "SELECT count(*) FROM reference_entries WHERE key LIKE 'language:%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(languages, 119, "zero-match language entries are retained");
    drop(connection);
    let sense = fixture.entry("sense:darkvision");
    assert_eq!(sense.availability, ReferenceAvailability::Available);
    assert_eq!(
        sense.backing,
        ReferenceBacking::Record {
            key: DARKVISION.parse().unwrap()
        }
    );
    assert!(matches!(
        fixture
            .entry("rule:pf2e:PF2E.NPC.Abilities.Glossary.Darkvision")
            .backing,
        ReferenceBacking::Record { .. }
    ));
    let air = fixture.entry("domain:air");
    let ReferenceBacking::Prepared { content, .. } = &air.backing else {
        panic!("expected prepared source body")
    };
    assert_eq!(content.text, "You can control winds and the weather.");
    assert_eq!(air.provenance.content_locale.as_deref(), Some("en"));
    assert_eq!(
        air.provenance.source_file.as_deref(),
        Some("static/lang/en.json")
    );
    assert_eq!(
        fixture.entry("language:chthonian").availability,
        ReferenceAvailability::Available
    );
    let language = fixture.entry("language:chthonian");
    assert_eq!(
        language.provenance.source_file.as_deref(),
        Some("data/languages.json")
    );
    assert_eq!(
        language.provenance.source_sha256.as_deref(),
        Some("4f5d90cca213300fb84463dfdcc17ab5b8b109030f7689644a68ad97aae24804")
    );
    assert!(
        language
            .permission
            .notices
            .iter()
            .any(|id| id == "pf2etools-formatting")
    );
    assert_eq!(language.permission.body_grants, grant_uses(&["ORC".into()]));
    assert_eq!(
        language.permission.label_grants,
        grant_uses(&["CUP".into()])
    );
    let shadowtongue = fixture.entry("language:shadowtongue");
    assert!(
        shadowtongue
            .permission
            .body_grants
            .contains(&ReferenceGrantUse {
                license: "ORC".into(),
                scope: ReferenceGrantScope::FunctionalContent
            })
    );
    assert!(
        shadowtongue
            .permission
            .body_grants
            .contains(&ReferenceGrantUse {
                license: "CUP".into(),
                scope: ReferenceGrantScope::DescriptiveReferences
            })
    );
    fs::rename(
        &fixture.source,
        fixture.directory.path().join("source-offline"),
    )
    .unwrap();
    assert_eq!(fixture.entry("domain:air"), air);
    assert!(
        !test_support::open_reader(&fixture.database)
            .unwrap()
            .reference_notices(&["foundry-orc".into()])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn unsupported_record_mapping_never_becomes_a_standalone_localization_fallback() {
    for (case, availability) in [
        ("missing", ReferenceAvailability::MissingTarget),
        ("family", ReferenceAvailability::WrongFamily),
        ("wrapper", ReferenceAvailability::ConflictingEvidence),
        ("module", ReferenceAvailability::SourceInapplicable),
    ] {
        let fixture = Fixture::new(if case == "module" {
            "custom-module"
        } else {
            "pf2e"
        });
        match case {
            "missing" => {
                fs::remove_file(fixture.source.join("packs/general/darkvision.json")).unwrap()
            }
            "family" => fixture.darkvision(
                "effect",
                "@Localize[PF2E.NPC.Abilities.Glossary.Darkvision]",
            ),
            "wrapper" => {
                fixture.darkvision("action", "Customized content without the required wrapper.")
            }
            _ => (),
        }
        fixture.build().unwrap();
        let rule = fixture.entry("rule:pf2e:PF2E.NPC.Abilities.Glossary.Darkvision");
        assert_eq!(rule.availability, availability, "{case}");
        assert_eq!(rule.backing, ReferenceBacking::None);
        if case == "wrapper" {
            assert_eq!(
                fixture.entry("sense:darkvision").availability,
                ReferenceAvailability::Available,
                "sense identity requires family, not pinned prose"
            );
        }
        atlas_index::validate_artifact(&fixture.database).unwrap();
    }
}

#[test]
fn known_source_attribution_does_not_withhold_updated_description() {
    let fixture = Fixture::new("pf2e");
    let path = fixture.source.join("static/lang/en.json");
    let mut catalog: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    catalog[AIR] = serde_json::json!("Updated source wording for the Air domain.");
    fs::write(path, catalog.to_string()).unwrap();
    fixture.build().unwrap();
    let entry = fixture.entry("domain:air");
    assert_eq!(entry.availability, ReferenceAvailability::Available);
    let ReferenceBacking::Prepared { content, .. } = entry.backing else {
        panic!("standalone content")
    };
    assert!(
        content
            .text
            .contains("Updated source wording for the Air domain.")
    );
    assert_eq!(
        entry.provenance.publications[0].title.as_deref(),
        Some("Pathfinder Player Core")
    );
    assert!(
        entry
            .permission
            .notices
            .iter()
            .any(|id| id == "player-core")
    );
    atlas_index::validate_artifact(&fixture.database).unwrap();
}

#[test]
fn selected_labels_and_traits_keep_their_locale_while_approved_bodies_use_english() {
    let fixture = Fixture::new("pf2e");
    fs::write(
        fixture.source.join("static/lang/fr.json"),
        serde_json::json!({
            "PF2E.Item.Deity.Domain.Air.Label":"Air traduit",
            "PF2E.Item.Deity.Domain.Air.Description":"Unreviewed translated body.",
            "PF2E.TraitAgile":"Agile traduit",
            "PF2E.TraitDescriptionAgile":"Translated trait prose."
        })
        .to_string(),
    )
    .unwrap();
    let mut options = fixture.options();
    options.locale = "fr".into();
    let workspace = test_support::initialize_workspace(&fixture.database).unwrap();
    crate::build_artifact(
        &workspace,
        test_support::TEST_WORKSPACE_SCHEMA_VERSION,
        options,
    )
    .unwrap();
    let air = fixture.entry("domain:air");
    assert_eq!(air.label, "Air traduit");
    assert_eq!(air.provenance.indexing_locale, "fr");
    assert_eq!(air.provenance.content_locale.as_deref(), Some("en"));
    let ReferenceBacking::Prepared { content, .. } = air.backing else {
        panic!("approved English fallback")
    };
    assert_eq!(content.text, "You can control winds and the weather.");
    let agile = fixture.entry("trait:agile");
    assert_eq!(agile.provenance.content_locale.as_deref(), Some("fr"));
    let ReferenceBacking::Prepared { content, .. } = agile.backing else {
        panic!("selected trait body")
    };
    assert_eq!(content.text, "Translated trait prose.");
    assert_eq!(
        fixture
            .entry("language:chthonian")
            .provenance
            .content_locale
            .as_deref(),
        Some("en")
    );
    atlas_index::validate_artifact(&fixture.database).unwrap();
}

#[test]
fn incremental_notice_change_and_removal_update_attribution() {
    let fixture = Fixture::new("pf2e");
    fixture.build().unwrap();
    let report = fixture.build().unwrap();
    assert_eq!(report.inferred_inputs, 0);
    assert_eq!(report.source_report.reused_preparations, 2);
    let before = test_support::open_reader(&fixture.database)
        .unwrap()
        .context()
        .reference_preparation
        .clone()
        .unwrap();
    let source_fingerprint = CapturedBuildInputs::capture(fixture.options())
        .unwrap()
        .fingerprint()
        .clone();
    let notice_path = fixture.source.join("static/licenses/ORCLicense.md");
    let text = fs::read_to_string(&notice_path).unwrap();
    fs::write(
        &notice_path,
        format!("{text}\nUpdated source attribution.\n"),
    )
    .unwrap();
    let inputs = CapturedBuildInputs::capture(fixture.options()).unwrap();
    assert_eq!(
        inputs.fingerprint(),
        &source_fingerprint,
        "notice updates do not falsify Foundry record provenance"
    );
    assert_ne!(inputs.reference_inputs_identity(), before.input_hash);
    fixture.build().unwrap();
    assert!(
        test_support::open_reader(&fixture.database)
            .unwrap()
            .reference_notices(&["foundry-orc".into()])
            .unwrap()
            .remove(0)
            .text
            .ends_with("Updated source attribution.\n")
    );
    fs::remove_file(&notice_path).unwrap();
    fixture.build().unwrap();
    assert_eq!(
        fixture.entry("domain:air").availability,
        ReferenceAvailability::PermissionUnresolved
    );
    assert_eq!(
        fixture.entry("sense:darkvision").availability,
        ReferenceAvailability::Available,
        "existing record body authority survives missing new standalone notices"
    );
    atlas_index::validate_artifact(&fixture.database).unwrap();
}

#[test]
#[ignore = "requires the authenticated pinned PF2e checkout; no inference or database build"]
fn pinned_source_all_approved_standalone_policies_are_readable() {
    let source = PathBuf::from(std::env::var_os("ATLAS_PF2E_SOURCE").expect("ATLAS_PF2E_SOURCE"));
    let directory = tempfile::tempdir().unwrap();
    let options = BuildArtifactOptions {
        source_root: source.clone(),
        output_path: directory.path().join("unused.sqlite"),
        manifest_path: None,
        locale: "en".into(),
        search_profiles: vec![],
        reuse_embeddings: true,
        reuse_passage_layouts: true,
        embedding_batch_size: 8,
    };
    let inputs = CapturedBuildInputs::capture(options).unwrap();
    let catalog =
        LocalizationCatalog::load_captured(&source, "en", None, Some(inputs.source())).unwrap();
    let context = crate::build::build_context(&inputs, &catalog).unwrap();
    let mut builder = ReferenceBuild::new(&inputs, &catalog, directory.path(), &context).unwrap();
    // Verify the producer against actual admitted documents and the captured
    // manifest too, not just the generator's reviewed expected pointers.
    let target_packs = builder
        .expected_records
        .iter()
        .map(|key| key.pack().as_str().to_owned())
        .collect::<BTreeSet<_>>();
    for pack in &inputs.source().manifest().manifest.packs {
        if !target_packs.contains(&pack.name) {
            continue;
        }
        let (directory, _) = inputs.source().pack(&pack.path);
        for path in inputs.source().paths(directory) {
            if path.extension().is_none_or(|extension| extension != "json") {
                continue;
            }
            let bytes = inputs.source().read(&path).unwrap();
            let context = atlas_foundry_model::SourceContext::new(
                path.display().to_string(),
                path.display().to_string(),
                "$",
            );
            let Ok(admission) =
                atlas_foundry_model::admit_document_source(&pack.document_type, context, &bytes)
            else {
                continue;
            };
            if let Some(model) = admission.model {
                let record = SourceBackedRecord::new(&pack.name, model).unwrap();
                if builder.expected_records.contains(record.key()) {
                    builder.observe_record(&record).unwrap();
                }
            }
        }
    }
    let notices = &inputs.reference_notices().notices;
    let resolver = SourceReferenceIndex::default();
    let mut prepared = 0;
    for authored in reference_definitions()
        .iter()
        .filter(|entry| !entry.excluded)
    {
        let entry = builder.entry(authored, &resolver, notices).unwrap();
        if authored.record_key.is_some() {
            assert_eq!(
                entry.availability,
                ReferenceAvailability::Available,
                "{}",
                entry.key
            );
            assert!(
                matches!(entry.backing, ReferenceBacking::Record { .. }),
                "{}",
                entry.key
            );
        }
        if builder.policies.contains_key(&entry.key)
            || inputs
                .reference_supplement()
                .entries
                .contains_key(&entry.key)
        {
            assert_eq!(
                entry.availability,
                ReferenceAvailability::Available,
                "{}",
                entry.key
            );
            assert!(
                matches!(entry.backing, ReferenceBacking::Prepared { .. }),
                "{}",
                entry.key
            );
            assert!(
                entry
                    .permission
                    .notices
                    .iter()
                    .all(|id| notices.contains_key(id))
            );
            prepared += 1;
        }
    }
    assert_eq!(
        prepared, 116,
        "88 base bodies +24 speaker payloads +4 weapon bodies"
    );
}
