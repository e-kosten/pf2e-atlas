use serde_json::Value;
use serde_json::json;
use std::path::PathBuf;

use super::{
    ItemType, PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT,
    PF2E_SOURCE_PINNED_SYSTEM_ID, PF2E_SOURCE_PINNED_SYSTEM_VERSION, SourceDiagnosticKind,
    SourceIdentity, SourceParentContext, SourcePresence, parse_item_source, parse_npc_source,
    pinned_source_version_metadata, validate_pinned_source_version,
};

fn fixture_identity() -> SourceIdentity {
    SourceIdentity::new(
        "fixture-pack:npc-fixture",
        "packs/fixture-pack/versioned-source-fixture.json",
    )
}

#[test]
fn pinned_source_version_is_explicit_build_diagnostic_metadata() {
    let metadata = pinned_source_version_metadata();

    assert_eq!(metadata.contract_version(), PF2E_SOURCE_CONTRACT_VERSION);
    assert_eq!(metadata.system_id(), PF2E_SOURCE_PINNED_SYSTEM_ID);
    assert_eq!(metadata.system_version(), PF2E_SOURCE_PINNED_SYSTEM_VERSION);
    assert_eq!(metadata.upstream_commit(), PF2E_SOURCE_PINNED_COMMIT);
}

#[test]
fn source_version_drift_is_diagnosed_before_dispatch() {
    let error = validate_pinned_source_version(
        &SourceIdentity::new("<source-manifest>", "static/system.json"),
        "pf2e",
        "6.13.0",
        PF2E_SOURCE_PINNED_COMMIT,
    )
    .expect_err("an unreviewed source version must fail");

    assert_eq!(error.kind, SourceDiagnosticKind::UnsupportedSourceVersion);
    assert_eq!(error.record_key(), "<source-manifest>");
    assert_eq!(error.source_path(), "static/system.json");
    assert_eq!(error.json_path(), "$source_version");
    assert_eq!(error.source_contract_version, PF2E_SOURCE_CONTRACT_VERSION);
    assert_eq!(error.source_system_version, "6.12.4");
    assert_eq!(error.source_upstream_commit, PF2E_SOURCE_PINNED_COMMIT);
    assert!(error.expected_shape().contains("version=6.12.4"));
    assert!(error.actual_shape().contains("version=6.13.0"));
    assert_eq!(
        serde_json::to_value(&error)
            .expect("diagnostic serializes")
            .get("record_key"),
        Some(&json!("<source-manifest>"))
    );
}

#[test]
fn representative_npc_dispatches_full_embedded_item_sources() {
    let raw: Value = serde_json::from_str(include_str!("fixtures/representative-npc.json"))
        .expect("fixture is valid JSON");
    let parsed = parse_npc_source(
        pinned_source_version_metadata(),
        fixture_identity(),
        raw.clone(),
    )
    .expect("representative NPC parses");

    assert_eq!(
        parsed.version.contract_version(),
        PF2E_SOURCE_CONTRACT_VERSION
    );
    assert_eq!(parsed.source.id, "npc-fixture");
    assert_eq!(parsed.source.sort, SourcePresence::Value(0));
    assert_eq!(parsed.source.folder, SourcePresence::Missing);
    assert_eq!(parsed.source.ownership, SourcePresence::Null);
    assert!(parsed.source.serialized().contains_field("prototypeToken"));
    assert!(parsed.source.system().contains_field("preparedOnly"));
    assert_eq!(parsed.raw_json_for_audit(), &raw);

    let items = parsed.source.items.as_value().expect("items are authored");
    assert_eq!(
        items
            .iter()
            .map(|item| item.item_type())
            .collect::<Vec<_>>(),
        vec![
            ItemType::Action,
            ItemType::Melee,
            ItemType::SpellcastingEntry,
            ItemType::Spell,
            ItemType::Equipment,
            ItemType::Lore,
        ]
    );
    assert_eq!(
        items[0].source().parent_context,
        Some(SourceParentContext::npc_items())
    );
    assert_eq!(items[0].source().effects, SourcePresence::Missing);
    assert_eq!(items[1].source().effects, SourcePresence::Null);
    assert_eq!(items[2].source().effects.as_value().map(Vec::len), Some(0));
    assert!(items[3].source().system().contains_field("overlays"));
    assert!(items[4].source().serialized().contains_field("system"));
}

#[test]
fn every_closed_item_discriminator_dispatches_a_full_item_source() {
    for (index, item_type) in ItemType::ALL.into_iter().enumerate() {
        let raw = json!({
            "_id": format!("item-{index}"),
            "name": format!("{} fixture", item_type.as_str()),
            "type": item_type.as_str(),
            "system": {
                "dynamicMap": {
                    "key": { "zero": 0, "false": false, "nullable": null }
                }
            },
            "customSourceField": [0, false, null]
        });
        let parsed = parse_item_source(
            pinned_source_version_metadata(),
            SourceIdentity::new(
                format!("item-fixtures:item-{index}"),
                format!("packs/item-fixtures/item-{index}.json"),
            ),
            None,
            raw.clone(),
        )
        .expect("registered Item discriminator parses");

        assert_eq!(parsed.source.item_type(), item_type);
        assert!(parsed.source.source().system().contains_field("dynamicMap"));
        assert!(
            parsed
                .source
                .source()
                .serialized()
                .contains_field("customSourceField")
        );
        assert_eq!(parsed.raw_json_for_audit(), &raw);
    }
}

#[test]
fn malformed_npc_reports_record_source_path_json_path_and_shapes() {
    let raw = serde_json::from_str(include_str!("fixtures/malformed-npc.json"))
        .expect("fixture is valid JSON");
    let error = parse_npc_source(pinned_source_version_metadata(), fixture_identity(), raw)
        .expect_err("malformed system must fail");

    assert_eq!(error.kind, SourceDiagnosticKind::MalformedShape);
    assert_eq!(error.record_key(), "fixture-pack:npc-fixture");
    assert_eq!(
        error.source_path(),
        "packs/fixture-pack/versioned-source-fixture.json"
    );
    assert_eq!(error.json_path(), "$.system");
    assert_eq!(error.expected_shape(), "object");
    assert_eq!(error.actual_shape(), "array");
    assert!(error.to_string().contains("serialized-Source"));
}

#[test]
fn embedded_item_type_drift_reports_the_nested_json_path() {
    let raw = serde_json::from_str(include_str!("fixtures/type-drift-npc.json"))
        .expect("fixture is valid JSON");
    let error = parse_npc_source(pinned_source_version_metadata(), fixture_identity(), raw)
        .expect_err("changed embedded system shape must fail");

    assert_eq!(error.kind, SourceDiagnosticKind::MalformedShape);
    assert_eq!(error.json_path(), "$.items[0].system");
    assert_eq!(error.expected_shape(), "object");
    assert_eq!(error.actual_shape(), "string");
}

#[test]
fn unknown_closed_discriminator_is_diagnosed_instead_of_falling_back() {
    let raw = json!({
        "_id": "unknown-item-npc",
        "name": "Unknown Item NPC",
        "type": "npc",
        "system": {},
        "items": [{
            "_id": "unknown-item",
            "name": "Unknown Item",
            "type": "futureItemType",
            "system": {}
        }]
    });
    let error = parse_npc_source(pinned_source_version_metadata(), fixture_identity(), raw)
        .expect_err("unknown Item discriminator must fail");

    assert_eq!(error.kind, SourceDiagnosticKind::UnknownDiscriminator);
    assert_eq!(error.json_path(), "$.items[0].type");
    assert!(error.expected_shape().contains("spellcastingEntry"));
    assert!(error.actual_shape().contains("futureItemType"));
}

#[test]
fn unknown_actor_discriminator_is_diagnosed_instead_of_falling_back() {
    let error = parse_npc_source(
        pinned_source_version_metadata(),
        fixture_identity(),
        json!({
            "_id": "unknown-actor",
            "name": "Unknown Actor",
            "type": "futureActorType",
            "system": {}
        }),
    )
    .expect_err("unknown Actor discriminator must fail");

    assert_eq!(error.kind, SourceDiagnosticKind::UnknownDiscriminator);
    assert_eq!(error.json_path(), "$.type");
    assert!(error.expected_shape().contains("npc"));
    assert!(error.actual_shape().contains("futureActorType"));
}

#[test]
fn known_item_in_an_invalid_npc_parent_context_is_diagnosed() {
    let raw = json!({
        "_id": "invalid-parent-npc",
        "name": "Invalid Parent NPC",
        "type": "npc",
        "system": {},
        "items": [{
            "_id": "ancestry-item",
            "name": "Ancestry Item",
            "type": "ancestry",
            "system": {}
        }]
    });
    let error = parse_npc_source(pinned_source_version_metadata(), fixture_identity(), raw)
        .expect_err("known but invalid parent pair must fail");

    assert_eq!(error.kind, SourceDiagnosticKind::InvalidParentContext);
    assert_eq!(error.json_path(), "$.items[0].type");
    assert!(error.expected_shape().contains("Actor[npc].items"));
    assert!(error.actual_shape().contains("Ancestry"));
}

#[test]
#[ignore = "requires the explicitly pinned PF2E_SOURCE_ROOT fixture checkout"]
fn pinned_night_hag_and_source_manifest_cross_the_versioned_boundary() {
    let source_root = PathBuf::from(
        std::env::var_os("PF2E_SOURCE_ROOT")
            .expect("PF2E_SOURCE_ROOT must name the pinned checkout"),
    );
    let manifest_path = source_root.join("static/system.json");
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path).expect("pinned source manifest is readable"),
    )
    .expect("pinned source manifest is valid JSON");
    let manifest = manifest.as_object().expect("source manifest is an object");
    let system_id = manifest
        .get("id")
        .and_then(Value::as_str)
        .expect("source manifest has string id");
    let system_version = manifest
        .get("version")
        .and_then(Value::as_str)
        .expect("source manifest has string version");
    let upstream_commit = std::env::var("PF2E_COMMIT").expect("PF2E_COMMIT is exported");
    let version = validate_pinned_source_version(
        &SourceIdentity::new("<source-manifest>", "static/system.json"),
        system_id,
        system_version,
        &upstream_commit,
    )
    .expect("pinned source version is supported");

    let relative_path = "packs/pathfinder-bestiary/night-hag.json";
    let raw: Value = serde_json::from_str(
        &std::fs::read_to_string(source_root.join(relative_path))
            .expect("Night Hag fixture is readable"),
    )
    .expect("Night Hag fixture is valid JSON");
    let parsed = parse_npc_source(
        version,
        SourceIdentity::new("pathfinder-bestiary:WQy7HBUcgDLsfVJd", relative_path),
        raw,
    )
    .expect("Night Hag crosses the serialized-Source boundary");
    let item_types = parsed
        .source
        .items
        .as_value()
        .expect("Night Hag authors embedded items")
        .iter()
        .map(|item| item.item_type())
        .collect::<Vec<_>>();

    for expected in [
        ItemType::Action,
        ItemType::Equipment,
        ItemType::Melee,
        ItemType::Spell,
        ItemType::SpellcastingEntry,
    ] {
        assert!(
            item_types.contains(&expected),
            "Night Hag fixture is missing representative {expected} dispatch"
        );
    }
}
