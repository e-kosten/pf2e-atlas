use crate::{test_support::*, *};
use atlas_domain::DefinitionKey;
use atlas_record::source_content::LocalizationResolver;
use rusqlite::Connection;

struct Locale;
impl LocalizationResolver for Locale {
    fn localized_value(&self, key: &str) -> Option<&str> {
        match key {
            "Fixture.Other" => Some("Existing contextual prose."),
            "Fixture.EmptyContent" => Some("<p></p>"),
            _ => None,
        }
    }
}

fn provenance(description_key: &str) -> ReferenceProvenance {
    ReferenceProvenance {
        source_package: "pf2e".into(),
        source_repository: None,
        source_revision: None,
        source_file: Some("fixture/catalog.json".into()),
        source_sha256: Some("a".repeat(64)),
        label_key: None,
        description_key: Some(description_key.into()),
        catalogs: vec!["fixture".into()],
        source_locations: vec![],
        indexing_locale: "en".into(),
        content_locale: Some("en".into()),
        publications: vec![],
        record_mapping: None,
        recipe: "fixture-reference/v1".into(),
    }
}

fn prepared(
    key: &str,
    description_key: &str,
    markup: &str,
    audience: atlas_record::source_content::ContentAudience,
) -> ReferenceEntry {
    let key: DefinitionKey = key.parse().unwrap();
    let content = atlas_record::source_content::prepare_reference_definition(
        key.clone(),
        markup,
        audience,
        Some(&Locale),
        None,
    )
    .unwrap();
    ReferenceEntry {
        key,
        label: "Shared readable title".into(),
        availability: ReferenceAvailability::Available,
        backing: ReferenceBacking::Prepared {
            authored_markup: markup.into(),
            content,
        },
        provenance: provenance(description_key),
        permission: ReferencePermission {
            disposition: ReferencePermissionDisposition::ApprovedContent,
            body_grants: vec![ReferenceGrantUse {
                license: "fixture".into(),
                scope: ReferenceGrantScope::FunctionalContent,
            }],
            label_grants: vec![],
            notices: vec!["fixture-notice".into()],
        },
    }
}

fn fixture() -> (IndexBuildInput, Vec<ReferenceEntry>, Vec<ReferenceNotice>) {
    let mut input =
        crate::foundation_tests::fixture_with_description(false, "<p>@Localize[Fixture.Sense]</p>");
    let target = crate::foundation_tests::record(
        "items",
        "Item",
        serde_json::json!({"_id":"eeeeeeeeeeeeeeee","name":"Context reference","type":"action","system":{"description":{"value":"<p>General supplied definition.</p>"},"actionType":{"value":"passive"}}}),
    );
    input.records.push(target);
    input.context.source_file_count = input.records.len();
    let hash = input.context.preparation_sha256().unwrap();
    for source in &mut input.records {
        source.preparation_context_hash = hash.clone();
    }
    input.lexical_units.clear();
    for source in &input.records {
        let label = &input
            .packs
            .iter()
            .find(|p| p.pack_id == source.record.key().pack().as_str())
            .unwrap()
            .label;
        let selection = atlas_record::source_record::select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            label,
            &input.context.used_trait_labels,
        )
        .unwrap();
        input
            .lexical_units
            .extend(crate::project_lexical_units(&selection, &[]));
    }
    let mut mapped = prepared(
        "sense:darkvision",
        "Fixture.Sense",
        "<p>Unused fixture body.</p>",
        input.context.audience,
    );
    let target = input.records.last().unwrap().record.key().clone();
    mapped.backing = ReferenceBacking::Record {
        key: target.clone(),
    };
    mapped.permission = ReferencePermission {
        disposition: ReferencePermissionDisposition::ExistingRecordPolicy,
        body_grants: vec![],
        label_grants: vec![],
        notices: vec![],
    };
    mapped.provenance.record_mapping = Some(ReferenceRecordMapping {
        target,
        package: "pf2e".into(),
        expected_document_kind: "Item".into(),
        expected_item_type: Some("action".into()),
        required_localization_key: None,
    });
    let mut label_only = prepared(
        "language:unspoken",
        "Fixture.Language",
        "<p>Unused fixture body.</p>",
        input.context.audience,
    );
    label_only.backing = ReferenceBacking::None;
    label_only.availability = ReferenceAvailability::LabelOnly;
    label_only.provenance.description_key = None;
    label_only.permission = ReferencePermission {
        disposition: ReferencePermissionDisposition::MetadataOnlyUnresolved,
        body_grants: vec![],
        label_grants: vec![],
        notices: vec![],
    };
    let entries = vec![
        prepared(
            "domain:air",
            "Fixture.First",
            "<p>@Localize[Fixture.Other]</p>",
            input.context.audience,
        ),
        prepared(
            "domain:water",
            "Fixture.Other",
            "<p>Existing contextual prose.</p>",
            input.context.audience,
        ),
        mapped,
        label_only,
    ];
    let text = "Fixture reference notice.".to_owned();
    let notices = vec![ReferenceNotice {
        id: "fixture-notice".into(),
        sha256: crate::codec::sha256(text.as_bytes()),
        text,
        source_url: Some("https://example.invalid/notice".into()),
    }];
    (input, entries, notices)
}

fn publish(
    path: &std::path::Path,
    input: &mut IndexBuildInput,
    entries: &[ReferenceEntry],
    notices: &[ReferenceNotice],
) -> Result<(), IndexError> {
    let keys = input
        .traits
        .iter()
        .map(|d| d.key.clone().into())
        .chain(entries.iter().map(|e| e.key.clone()));
    let (expected_entry_count, entry_keys_sha256) = reference_inventory_identity(keys)?;
    let (source_package, pack_systems) = input
        .context
        .reference_preparation
        .as_ref()
        .map(|p| (p.source_package.clone(), p.pack_systems.clone()))
        .unwrap_or((Some("pf2e".into()), Default::default()));
    input.context.reference_preparation = Some(ReferencePreparation {
        source_package,
        pack_systems,
        recipe: "fixture-reference/v1".into(),
        input_hash: crate::codec::sha256(&serde_json::to_vec(&(entries, notices))?),
        coverage: ReferenceCoverage::Complete,
        expected_entry_count,
        entry_keys_sha256,
    });
    publish_input_with_references(path, input, entries, notices)
}

#[test]
fn standalone_and_record_backed_definitions_read_without_source_or_copied_records() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("references.sqlite");
    let (mut input, mut entries, notices) = fixture();
    // Approved descriptive language/reference context can have a narrower body basis.
    entries[1].permission.body_grants[0].scope = ReferenceGrantScope::DescriptiveReferences;
    entries[1].permission.label_grants = vec![ReferenceGrantUse {
        license: "fixture-label".into(),
        scope: ReferenceGrantScope::DescriptiveReferences,
    }];
    publish(&path, &mut input, &entries, &notices).unwrap();
    let reader = open_reader(&path).unwrap();
    assert_eq!(reader.reference_coverage(), ReferenceCoverage::Complete);
    assert!(
        reader
            .reference_entry(&"language:homebrew".parse().unwrap())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        reader
            .reference_entry(&"language:unspoken".parse().unwrap())
            .unwrap()
            .unwrap()
            .availability,
        ReferenceAvailability::LabelOnly
    );
    assert_eq!(
        reader
            .reference_notices(&["fixture-notice".into()])
            .unwrap()
            .remove(0),
        notices[0]
    );
    let mapped = reader
        .reference_entry(&"sense:darkvision".parse().unwrap())
        .unwrap()
        .unwrap();
    assert!(matches!(mapped.backing, ReferenceBacking::Record { .. }));
    let json = serde_json::to_value(mapped).unwrap();
    assert!(json["backing"].get("authored_markup").is_none());
    assert!(json["backing"].get("content").is_none());
    drop(reader);
    let c = Connection::open(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM records", [], |r| r.get::<_, usize>(0))
            .unwrap(),
        input.records.len()
    );
    validate_artifact(&path).unwrap();
    let retained: Vec<u8> = c.query_row("SELECT snapshot FROM record_bodies WHERE record_id=(SELECT record_id FROM records WHERE key='items:eeeeeeeeeeeeeeee')", [], |r| r.get(0)).unwrap();
    c.execute("UPDATE record_bodies SET snapshot=x'00' WHERE record_id=(SELECT record_id FROM records WHERE key='items:eeeeeeeeeeeeeeee')", []).unwrap();
    let reader = open_reader(&path).unwrap();
    assert!(matches!(
        reader
            .reference_entry(&"sense:darkvision".parse().unwrap())
            .unwrap()
            .unwrap()
            .backing,
        ReferenceBacking::Record { .. }
    ));
    drop(reader);
    assert!(validate_artifact(&path).is_err());
    c.execute("UPDATE record_bodies SET snapshot=? WHERE record_id=(SELECT record_id FROM records WHERE key='items:eeeeeeeeeeeeeeee')", [retained]).unwrap();
    c.execute(
        "DELETE FROM reference_entries WHERE key='language:unspoken'",
        [],
    )
    .unwrap();
    assert!(
        validate_reference(&c)
            .unwrap_err()
            .to_string()
            .contains("reference inventory")
    );
}

#[test]
fn unchanged_reference_publication_is_quiet_and_failed_notice_update_preserves_committed_generation()
 {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atomic.sqlite");
    let (mut input, entries, notices) = fixture();
    publish(&path, &mut input, &entries, &notices).unwrap();
    let c = Connection::open(&path).unwrap();
    c.execute_batch("CREATE TABLE reference_write_audit(component TEXT NOT NULL)")
        .unwrap();
    for table in [
        "reference_entries",
        "reference_notices",
        "owned_ability_evidence",
    ] {
        for event in ["INSERT", "UPDATE", "DELETE"] {
            c.execute_batch(&format!("CREATE TRIGGER audit_{table}_{event} AFTER {event} ON {table} BEGIN INSERT INTO reference_write_audit VALUES('{table}'); END;")).unwrap();
        }
    }
    publish(&path, &mut input, &entries, &notices).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM reference_write_audit", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    let revision = c
        .query_row("SELECT reference_revision FROM workspace", [], |r| {
            r.get::<_, u64>(0)
        })
        .unwrap();
    let error = publish(&path, &mut input, &entries, &[]).unwrap_err();
    assert!(error.to_string().contains("notice"), "{error}");
    assert_eq!(
        c.query_row("SELECT reference_revision FROM workspace", [], |r| r
            .get::<_, u64>(0))
            .unwrap(),
        revision
    );
    assert!(
        open_reader(&path)
            .unwrap()
            .read_summary(input.records[0].record.key())
            .unwrap()
            .is_some()
    );
    assert_eq!(
        open_reader(&path)
            .unwrap()
            .reference_notices(&["fixture-notice".into()])
            .unwrap()
            .remove(0),
        notices[0]
    );
    validate_artifact(&path).unwrap();
    publish(&path, &mut input, &[], &[]).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM reference_entries", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM owned_ability_evidence", [], |r| r
            .get::<_, usize>(
            0
        ))
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM reference_notices", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    validate_artifact(&path).unwrap();
}

#[test]
fn wrong_family_mapping_is_rejected_and_missing_target_reason_retains_no_readable_pointer() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mapping.sqlite");
    let (mut input, mut entries, notices) = fixture();
    publish(&path, &mut input, &entries, &notices).unwrap();
    let old_generation = open_reader(&path).unwrap().artifact_instance().clone();
    let absent: atlas_domain::RecordKey = "items:ffffffffffffffff".parse().unwrap();
    for (key, expected) in [
        (input.records[1].record.key().clone(), "family"),
        (absent, "no retained snapshot"),
    ] {
        entries[2].backing = ReferenceBacking::Record { key: key.clone() };
        entries[2]
            .provenance
            .record_mapping
            .as_mut()
            .unwrap()
            .target = key;
        let error = publish(&path, &mut input, &entries, &notices).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(
            open_reader(&path).unwrap().artifact_instance(),
            &old_generation
        );
    }
    entries[2].backing = ReferenceBacking::None;
    entries[2].availability = ReferenceAvailability::MissingTarget;
    publish(&path, &mut input, &entries, &notices).unwrap();
    let entry = open_reader(&path)
        .unwrap()
        .reference_entry(&"sense:darkvision".parse().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(entry.availability, ReferenceAvailability::MissingTarget);
    assert!(matches!(entry.backing, ReferenceBacking::None));
    validate_artifact(&path).unwrap();
}

#[test]
fn offline_projection_upgrade_recovers_authored_sense_qualifiers_and_armor_group() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("facts.sqlite");
    let mut input = crate::foundation_tests::fixture(false);
    input.records[0] = crate::foundation_tests::record(
        "actors",
        "Actor",
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Authored senses","type":"npc","system":{"perception":{"senses":[{"type":"darkvision","acuity":"precise","range":60,"source":"authored source"},{"type":"scent","range":null},{"type":"tremorsense","acuity":"unexpected","range":"unknown","source":null}]}}}),
    );
    input.records[1] = crate::foundation_tests::record(
        "items",
        "Item",
        serde_json::json!({"_id":"bbbbbbbbbbbbbbbb","name":"Authored armor","type":"armor","system":{"group":"chain","description":{"value":"<p>Authored armor.</p>"}}}),
    );
    let hash = input.context.preparation_sha256().unwrap();
    input.lexical_units.clear();
    for source in &mut input.records {
        source.preparation_context_hash = hash.clone();
        let label = &input
            .packs
            .iter()
            .find(|p| p.pack_id == source.record.key().pack().as_str())
            .unwrap()
            .label;
        let selection = atlas_record::source_record::select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            label,
            &input.context.used_trait_labels,
        )
        .unwrap();
        input
            .lexical_units
            .extend(crate::project_lexical_units(&selection, &[]));
    }
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    workspace
        .migrate(&TEST_MIGRATIONS[..24], |_| Ok(()))
        .unwrap();
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner.maintenance(24, "historical sense facts").unwrap();
        publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    workspace
        .migrate(&TEST_MIGRATIONS[..32], |_| Ok(()))
        .unwrap();
    crate::test_support::stage_pre_glossary_catalog(
        &workspace,
        &input.context.remaster_inputs_identity,
    )
    .unwrap();
    let read = workspace.read(32).unwrap();
    let original_bodies = read
        .connection()
        .prepare("SELECT snapshot FROM record_bodies ORDER BY record_id")
        .unwrap()
        .query_map([], |r| r.get::<_, Vec<u8>>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let revision = read.reference_revision().unwrap();
    drop(read);
    workspace.migrate(TEST_MIGRATIONS, |_| Ok(())).unwrap();
    assert!(ensure_reference_projections(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), revision + 2);
    assert_eq!(
        read.connection()
            .prepare("SELECT snapshot FROM record_bodies ORDER BY record_id")
            .unwrap()
            .query_map([], |r| r.get::<_, Vec<u8>>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        original_bodies
    );
    let senses = read
        .connection()
        .prepare("SELECT acuity_state,acuity,range_state,range FROM actor_senses ORDER BY original_index")
        .unwrap()
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        senses,
        vec![
            (
                "value".into(),
                Some("precise".into()),
                "value".into(),
                Some(60)
            ),
            ("missing".into(), None, "null".into(), None),
            ("invalid".into(), None, "invalid".into(), None),
        ]
    );
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT armor_group_state,armor_group FROM armor_projection",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            )
            .unwrap(),
        ("value".into(), Some("chain".into()))
    );
    drop(read);
    assert_eq!(
        open_reader(&path).unwrap().reference_coverage(),
        ReferenceCoverage::TraitsOnly
    );
    validate_artifact(&path).unwrap();
}

#[test]
fn captured_pack_scope_distinguishes_inheritance_explicit_match_conflict_and_unknown() {
    let mut preparation = ReferencePreparation {
        source_package: Some("pf2e".into()),
        pack_systems: Default::default(),
        recipe: "fixture/v1".into(),
        input_hash: "a".repeat(64),
        coverage: ReferenceCoverage::Complete,
        expected_entry_count: 0,
        entry_keys_sha256: "b".repeat(64),
    };
    assert!(reference_pack_applies(
        Some(&preparation),
        "pf2e",
        "actions"
    ));
    preparation
        .pack_systems
        .insert("actions".into(), Some("pf2e".into()));
    assert!(reference_pack_applies(
        Some(&preparation),
        "pf2e",
        "actions"
    ));
    for scope in [None, Some("sf2e".into()), Some(String::new())] {
        preparation.pack_systems.insert("actions".into(), scope);
        assert!(!reference_pack_applies(
            Some(&preparation),
            "pf2e",
            "actions"
        ));
        assert!(reference_pack_applies(
            Some(&preparation),
            "pf2e",
            "omitted"
        ));
    }
    preparation.source_package = None;
    assert!(!reference_pack_applies(
        Some(&preparation),
        "pf2e",
        "omitted"
    ));
    assert!(!reference_pack_applies(None, "pf2e", "omitted"));
    preparation.pack_systems.clear();
    let retained = serde_json::to_string(&preparation).unwrap();
    assert!(!retained.contains("source_package"));
    assert!(!retained.contains("pack_systems"));
    let decoded: ReferencePreparation = serde_json::from_str(&retained).unwrap();
    assert_eq!(serde_json::to_string(&decoded).unwrap(), retained);
}

#[test]
fn conflicting_pack_scope_rejects_record_backing_and_excludes_new_source_associations() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("scope.sqlite");
    let (mut input, mut entries, notices) = fixture();
    publish(&path, &mut input, &entries, &notices).unwrap();
    for declared in [None, Some("sf2e".into())] {
        input
            .context
            .reference_preparation
            .as_mut()
            .unwrap()
            .pack_systems
            .insert("items".into(), declared);
        assert!(publish(&path, &mut input, &entries, &notices).is_err());
        validate_artifact(&path).unwrap();
    }
    // Retain honest mapping disposition, while a standalone key remains known.
    entries[2].backing = ReferenceBacking::None;
    entries[2].availability = ReferenceAvailability::SourceInapplicable;
    entries[1].provenance.description_key = Some("Fixture.Sense".into());
    publish(&path, &mut input, &entries, &notices).unwrap();
    validate_artifact(&path).unwrap();
    let c = Connection::open(&path).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM trait_memberships WHERE trait_key='trait:dragon'",
            [],
            |r| r.get::<_, usize>(0)
        )
        .unwrap(),
        1
    );
    c.execute(
        "INSERT INTO owned_ability_evidence VALUES('items:bbbbbbbbbbbbbbbb','[]',NULL,1,'[]')",
        [],
    )
    .unwrap();
    assert!(validate_reference(&c).is_err());
}

#[test]
fn metadata_batches_skip_prepared_body_hydration_and_keep_integrity_checks() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("references.sqlite");
    let (mut input, entries, notices) = fixture();
    publish(&path, &mut input, &entries, &notices).unwrap();
    let key: DefinitionKey = "domain:air".parse().unwrap();
    let c = Connection::open(&path).unwrap();
    let json: String = c
        .query_row(
            "SELECT definition_json FROM reference_entries WHERE key=?",
            [key.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let mut envelope: serde_json::Value = serde_json::from_str(&json).unwrap();
    envelope["backing"]["content"] = serde_json::json!({"intentionally":"not a prepared body"});
    let unreadable_body = serde_json::to_string(&envelope).unwrap();
    c.execute(
        "UPDATE reference_entries SET definition_json=?,definition_hash=? WHERE key=?",
        rusqlite::params![
            unreadable_body,
            crate::codec::sha256(unreadable_body.as_bytes()),
            key.to_string()
        ],
    )
    .unwrap();
    let reader = open_reader(&path).unwrap();
    let missing: DefinitionKey = "language:custom-unpublished".parse().unwrap();
    let batch = reader
        .reference_summaries(&[key.clone(), key.clone(), missing])
        .unwrap();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].key, key);
    assert_eq!(
        batch[0].backing,
        ReferenceSummaryBacking::Prepared { readable: false }
    );
    assert!(reader.reference_entry(&key).is_err());
    assert!(reader.reference_summaries(&vec![key.clone(); 101]).is_err());
    drop(reader);
    let mut envelope: serde_json::Value = serde_json::from_str(&json).unwrap();
    for (field, value) in [("label", " "), ("key", "domain:water")] {
        let original = envelope[field].clone();
        envelope[field] = value.into();
        let json = serde_json::to_string(&envelope).unwrap();
        c.execute(
            "UPDATE reference_entries SET definition_json=?,definition_hash=? WHERE key=?",
            rusqlite::params![json, crate::codec::sha256(json.as_bytes()), key.to_string()],
        )
        .unwrap();
        let reader = open_reader(&path).unwrap();
        assert!(reader.reference_summary(&key).is_err(), "invalid {field}");
        assert!(reader.reference_entry(&key).is_err(), "invalid {field}");
        envelope[field] = original;
    }
    c.execute(
        "UPDATE reference_entries SET definition_json=? WHERE key=?",
        rusqlite::params![json, key.to_string()],
    )
    .unwrap();
    assert!(open_reader(&path).unwrap().reference_summary(&key).is_err());
}

#[test]
fn prepared_reading_metadata_matches_canonical_presence_without_rewriting_bodies() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("references.sqlite");
    let (mut input, mut entries, notices) = fixture();
    let cases = [
        ("<p></p>", false),
        ("<p>&nbsp;</p>", false),
        ("<br>", false),
        ("<hr>", false),
        ("<p title=\"<img \"></p>", false),
        ("@Localize[Fixture.EmptyContent]", false),
        ("<img src=\"https://example.com/image.png\">", true),
        ("<img src=\"https://example.com/image.png\" alt=\"\">", true),
        ("@Damage[1d6[fire]]", true),
        ("<p>Actual explanatory prose.</p>", true),
    ];
    let mut selected = Vec::new();
    for (index, (markup, readable)) in cases.iter().enumerate() {
        let entry = prepared(
            &format!("rule:pf2e:Fixture.Presence{index}"),
            &format!("Fixture.Presence{index}"),
            markup,
            input.context.audience,
        );
        selected.push((entry.key.clone(), *readable));
        entries.push(entry);
    }
    publish(&path, &mut input, &entries, &notices).unwrap();
    let reader = open_reader(&path).unwrap();
    let revision = reader.artifact_instance().reference_revision();
    for (key, readable) in selected {
        let summary = reader.reference_summary(&key).unwrap().unwrap();
        assert_eq!(
            summary.backing,
            ReferenceSummaryBacking::Prepared { readable },
            "{key}"
        );
        let entry = reader.reference_entry(&key).unwrap().unwrap();
        assert_eq!(summary, ReferenceSummary::from(&entry), "{key}");
    }
    assert_eq!(reader.artifact_instance().reference_revision(), revision);
}
