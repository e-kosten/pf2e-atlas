use super::*;
use crate::{BuildArtifactOptions, ReferenceSupplementSelection};
use atlas_index::{ReferenceCoverage, test_support};
use rusqlite::Connection;
use std::{fs, path::PathBuf};

const DARKVISION: &str = "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6";
const AIR: &str = "PF2E.Item.Deity.Domain.Air.Description";
const AGILE: &str = "PF2E.TraitDescriptionAgile";
const FAMILY_TEMPLATES: &[(&str, &str)] = &[
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
            serde_json::from_slice(include_bytes!("../../reference-data/base-notices.json"))
                .unwrap();
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
    fn family_templates(&self) {
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
        let templates: serde_json::Value = serde_json::from_str(include_str!(
            "../../../atlas-record/src/source_record/fixtures/general-definition-templates.json"
        ))
        .unwrap();
        let catalog_path = self.source.join("static/lang/en.json");
        let mut catalog: serde_json::Value =
            serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
        for (id, name) in FAMILY_TEMPLATES {
            let item = templates["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["_id"] == *id)
                .unwrap();
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
    fn options(&self, selection: ReferenceSupplementSelection) -> BuildArtifactOptions {
        BuildArtifactOptions {
            source_root: self.source.clone(),
            output_path: self.database.clone(),
            manifest_path: None,
            locale: "en".into(),
            reference_supplement: selection,
            search_profiles: vec![],
            reuse_embeddings: true,
            reuse_passage_layouts: true,
            embedding_batch_size: 8,
        }
    }
    fn build(
        &self,
        selection: ReferenceSupplementSelection,
    ) -> Result<crate::BuildArtifactReport, IngestError> {
        let workspace = test_support::initialize_workspace(&self.database).unwrap();
        crate::build_artifact(
            &workspace,
            test_support::TEST_WORKSPACE_SCHEMA_VERSION,
            self.options(selection),
        )
    }
    fn entry(&self, key: &str) -> ReferenceEntry {
        test_support::open_reader(&self.database)
            .unwrap()
            .reference_entry(&key.parse().unwrap())
            .unwrap()
            .unwrap()
    }
    fn edges(&self) -> Vec<DefinitionReferenceInput> {
        Connection::open(&self.database)
            .unwrap()
            .prepare("SELECT evidence FROM definition_reference_occurrences ORDER BY occurrence_id")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|row| serde_json::from_str(&row.unwrap()).unwrap())
            .collect()
    }
}

#[test]
fn family_templates_reuse_the_actual_body_and_keep_checked_reading_beside_it() {
    let fixture = Fixture::new("pf2e");
    fixture.family_templates();
    fixture
        .build(ReferenceSupplementSelection::Bundled)
        .unwrap();
    for (id, name) in FAMILY_TEMPLATES {
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
        let reading = body
            .general_definition_reading
            .as_ref()
            .expect("exact template has a checked general reading");
        assert_eq!(reading.template_key, target);
        assert!(!reading.parameters.is_empty());
        assert!(!reading.excluded_parameter_interaction_ordinals.is_empty());
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

    // Exact source identity still maps after a customization, but template reading
    // must fall back to the ordinary full body when its signature changes.
    let path = fixture.source.join("packs/family/GolemAntimagic.json");
    let mut item: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    item["system"]["description"]["value"] = serde_json::json!(
        "<p>Actual creature parameters.</p><p>@Localize[PF2E.NPC.Abilities.Glossary.GolemAntimagic]</p>"
    );
    fs::write(path, item.to_string()).unwrap();
    fs::remove_file(fixture.source.join("packs/family/WindUp.json")).unwrap();
    fixture
        .build(ReferenceSupplementSelection::Bundled)
        .unwrap();
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
        bundle
            .fields
            .iter()
            .all(|field| field.general_definition_reading.is_none())
    );
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
fn source_free_publication_reuses_actual_record_and_preserves_authored_edge_scope() {
    let fixture = Fixture::new("pf2e");
    let report = fixture
        .build(ReferenceSupplementSelection::Bundled)
        .unwrap();
    assert_eq!(report.inferred_inputs, 0);
    atlas_index::validate_artifact(&fixture.database).unwrap();
    let reader = test_support::open_reader(&fixture.database).unwrap();
    assert_eq!(reader.reference_coverage(), ReferenceCoverage::Complete);
    let (_, languages) = reader
        .reference_entries(DefinitionNamespace::Language, 100, 100)
        .unwrap();
    assert_eq!(
        languages, 119,
        "supported zero-match vocabulary remains discoverable"
    );
    drop(reader);
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
            .any(|id| id == "reference-subset-manifest")
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
    let edges = fixture.edges();
    assert!(edges.iter().any(|edge| matches!(&edge.source, DefinitionReferenceSource::Record { key, .. } if key.to_string()=="items:aaaaaaaaaaaaaaaa") && matches!(&edge.target,DefinitionReferenceTarget::Record { key, owners } if key.to_string()==DARKVISION && owners.is_empty())));
    assert!(edges.iter().any(|edge| matches!(&edge.source, DefinitionReferenceSource::Definition { key } if key.to_string()=="trait:agile") && matches!(&edge.target,DefinitionReferenceTarget::Definition { key } if key.to_string()=="domain:air")));
    assert!(edges.iter().any(|edge| !edge.visible && matches!(&edge.source, DefinitionReferenceSource::Record { key, .. } if key.to_string()=="items:aaaaaaaaaaaaaaaa") && matches!(&edge.target,DefinitionReferenceTarget::Definition { key } if key.to_string()=="domain:air")));
    assert!(!edges.iter().any(|edge| edge.visible && matches!(&edge.source, DefinitionReferenceSource::Record { key, .. } if key.to_string()=="items:aaaaaaaaaaaaaaaa") && matches!(&edge.target,DefinitionReferenceTarget::Definition { key } if key.to_string()=="domain:air")), "expanded localization prose must not become authored record evidence");
    fs::rename(
        &fixture.source,
        fixture.directory.path().join("source-offline"),
    )
    .unwrap();
    assert_eq!(fixture.entry("domain:air"), air);
    assert!(
        test_support::open_reader(&fixture.database)
            .unwrap()
            .reference_notice("foundry-orc")
            .unwrap()
            .is_some()
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
        fixture
            .build(ReferenceSupplementSelection::Bundled)
            .unwrap();
        let rule = fixture.entry("rule:pf2e:PF2E.NPC.Abilities.Glossary.Darkvision");
        assert_eq!(rule.availability, availability, "{case}");
        assert_eq!(rule.backing, ReferenceBacking::None);
        assert!(
            !fixture
                .edges()
                .iter()
                .any(|edge| edge.evidence == "PF2E.NPC.Abilities.Glossary.Darkvision"),
            "{case}: unsupported mapping must not create a synthetic definition target"
        );
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
    let mut options = fixture.options(ReferenceSupplementSelection::Bundled);
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
fn incremental_notice_change_removal_and_failed_asset_keep_committed_state_honest() {
    let fixture = Fixture::new("pf2e");
    fixture
        .build(ReferenceSupplementSelection::Bundled)
        .unwrap();
    let report = fixture
        .build(ReferenceSupplementSelection::Bundled)
        .unwrap();
    assert_eq!(report.inferred_inputs, 0);
    assert_eq!(report.source_report.reused_preparations, 2);
    let before = test_support::open_reader(&fixture.database)
        .unwrap()
        .context()
        .reference_preparation
        .clone()
        .unwrap();
    let source_fingerprint =
        CapturedBuildInputs::capture(fixture.options(ReferenceSupplementSelection::Bundled))
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
    let inputs =
        CapturedBuildInputs::capture(fixture.options(ReferenceSupplementSelection::Bundled))
            .unwrap();
    assert_eq!(
        inputs.fingerprint(),
        &source_fingerprint,
        "notice updates do not falsify Foundry record provenance"
    );
    assert_ne!(inputs.reference_inputs_identity(), before.input_hash);
    fixture
        .build(ReferenceSupplementSelection::Bundled)
        .unwrap();
    assert!(
        test_support::open_reader(&fixture.database)
            .unwrap()
            .reference_notice("foundry-orc")
            .unwrap()
            .unwrap()
            .text
            .ends_with("Updated source attribution.\n")
    );
    let revision = test_support::open_reader(&fixture.database)
        .unwrap()
        .artifact_instance()
        .reference_revision();
    assert!(
        fixture
            .build(ReferenceSupplementSelection::Directory {
                path: fixture.directory.path().join("missing-required-asset")
            })
            .is_err()
    );
    assert_eq!(
        test_support::open_reader(&fixture.database)
            .unwrap()
            .artifact_instance()
            .reference_revision(),
        revision
    );
    assert_eq!(
        fixture.entry("language:chthonian").availability,
        ReferenceAvailability::Available
    );
    fixture
        .build(ReferenceSupplementSelection::Absent {
            revision: "removed/v2".into(),
        })
        .unwrap();
    assert_eq!(
        fixture.entry("language:chthonian").backing,
        ReferenceBacking::None
    );
    assert_eq!(
        fixture.entry("domain:air").availability,
        ReferenceAvailability::Available,
        "removing supplemental context does not remove source-localized content"
    );
    fs::remove_file(&notice_path).unwrap();
    fixture
        .build(ReferenceSupplementSelection::Absent {
            revision: "removed/v2".into(),
        })
        .unwrap();
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
        reference_supplement: ReferenceSupplementSelection::Bundled,
        search_profiles: vec![],
        reuse_embeddings: true,
        reuse_passage_layouts: true,
        embedding_batch_size: 8,
    };
    let inputs = CapturedBuildInputs::capture(options).unwrap();
    let catalog =
        LocalizationCatalog::load_captured(&source, "en", None, Some(inputs.source())).unwrap();
    let mut builder = ReferenceBuild::new(
        &inputs,
        &catalog,
        directory.path(),
        crate::build::default_audience(),
    )
    .unwrap();
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
    let mut notices = inputs.reference_notices().notices.clone();
    for notice in &inputs.reference_supplement().notices {
        notices.insert(notice.id.clone(), notice.clone());
    }
    let resolver = SourceReferenceIndex::default();
    let mut prepared = 0;
    for authored in reference_definitions()
        .iter()
        .filter(|entry| !entry.excluded)
    {
        let entry = builder.entry(authored, &resolver, &notices).unwrap();
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
