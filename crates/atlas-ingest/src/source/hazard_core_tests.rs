use std::path::PathBuf;

use atlas_domain::{PackName, Rarity};
use atlas_record::{
    FactValue, HazardComplexity, HazardSourceShape, HazardSourceValue, RecordBody,
    render_plain_text,
};
use serde_json::Value;

use super::ManifestPack;
use super::dto::{SourceIdentity, parse_hazard_source, pinned_source_version_metadata};
use super::normalize::{normalize_record, normalize_record_from_source_bytes};

#[test]
fn hazard_core_hidden_pit_is_source_faithful_without_generic_mechanics() {
    let loaded = normalize_fixture("hazards", "packs/hazards/hidden-pit.json");
    assert!(loaded.record.mechanics.metrics.is_empty());
    assert!(loaded.record.mechanics.actor().is_none());
    let RecordBody::Hazard(hazard) = loaded.facts.canonical_body.as_ref().expect("hazard body")
    else {
        panic!("hazard body")
    };
    assert_eq!(hazard.identity.source_id.as_str(), "BHq5wpQU8hQEke8D");
    assert_eq!(hazard.identity.name, "Hidden Pit");
    assert_eq!(hazard.level.typed(), Some(&0));
    assert_eq!(hazard.rarity.typed(), Some(&Rarity::Common));
    assert_eq!(hazard.complexity.typed(), Some(&HazardComplexity::Simple));
    assert_eq!(
        hazard
            .traits
            .typed()
            .expect("traits")
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["mechanical", "trap"]
    );
    let detection = hazard.detection.typed().expect("detection");
    assert_eq!(detection.stealth_modifier.typed(), Some(&8));
    assert_eq!(
        render_plain_text(detection.details.typed().expect("details")),
        "(or 0 if the trapdoor is disabled or broken)"
    );
    let defenses = hazard.defenses.typed().expect("defenses");
    assert_eq!(defenses.armor_class.typed(), Some(&10));
    assert_eq!(defenses.hardness.typed(), Some(&3));
    assert_eq!(
        defenses
            .hit_points
            .typed()
            .expect("hit points")
            .maximum
            .typed(),
        Some(&12)
    );
    assert_eq!(
        defenses.saves.typed().expect("saves").will.typed(),
        Some(&0)
    );
    let lifecycle = hazard.lifecycle.typed().expect("lifecycle");
    assert!(render_plain_text(lifecycle.disable.typed().expect("disable")).contains("Thievery"));
    assert!(render_plain_text(lifecycle.reset.typed().expect("reset")).contains("reset manually"));
    assert_eq!(
        hazard.provenance.source_creature_type.typed(),
        Some(&String::new())
    );
    assert_eq!(
        hazard.provenance.source_status_effects.typed(),
        Some(&Vec::new())
    );
    assert!(matches!(
        hazard.provenance.actor_effects.value,
        FactValue::Missing
    ));
}

#[test]
fn hazard_core_preserves_null_and_zero_independently() {
    let voices = normalize_fixture(
        "book-of-the-dead-bestiary",
        "packs/book-of-the-dead-bestiary/disembodied-voices.json",
    );
    let RecordBody::Hazard(voices) = voices.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard")
    };
    let defenses = voices.defenses.typed().expect("defenses");
    assert!(matches!(defenses.armor_class.value, FactValue::Null));
    assert_eq!(defenses.hardness.typed(), Some(&0));
    let hp = defenses.hit_points.typed().expect("hp");
    assert!(matches!(hp.current.value, FactValue::Null));
    assert!(matches!(hp.maximum.value, FactValue::Null));
    let saves = defenses.saves.typed().expect("saves");
    assert_eq!(saves.fortitude.typed(), Some(&0));
    assert_eq!(saves.reflex.typed(), Some(&0));
    assert_eq!(saves.will.typed(), Some(&0));
    assert!(matches!(defenses.immunities.value, FactValue::Missing));

    let rift = normalize_fixture(
        "agents-of-edgewatch-bestiary",
        "packs/agents-of-edgewatch-bestiary/book-6-ruins-of-the-radiant-siege/greater-planar-rift.json",
    );
    let RecordBody::Hazard(rift) = rift.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard")
    };
    assert!(matches!(
        rift.lifecycle.typed().expect("lifecycle").routine.value,
        FactValue::Null
    ));
    assert_eq!(
        rift.defenses.typed().expect("defenses").armor_class.typed(),
        Some(&0)
    );
}

#[test]
fn hazard_core_retains_malformed_scalar_as_typed_unsupported() {
    let source_root = source_root();
    let relative = "packs/hazards/hidden-pit.json";
    let path = source_root.join(relative);
    let mut raw: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("fixture"))
        .expect("fixture JSON");
    raw.pointer_mut("/system/attributes/hardness")
        .expect("hardness")
        .clone_from(&Value::String("3".to_string()));
    let loaded = normalize_raw("hazards", relative, raw);
    let RecordBody::Hazard(hazard) = loaded.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard")
    };
    let value = &hazard.defenses.typed().expect("defenses").hardness.value;
    let FactValue::Value(HazardSourceValue::Unsupported(value)) = value else {
        panic!("unsupported hardness")
    };
    assert_eq!(value.exact_json, "\"3\"");
    assert_eq!(value.actual_shape, HazardSourceShape::String);
    assert_eq!(value.relative_source_path, "/system/attributes/hardness");
}

#[test]
fn hazard_source_dto_retains_exact_audit_source_without_exposing_it_as_semantics() {
    let source_root = source_root();
    let relative = "packs/hazards/hidden-pit.json";
    let raw: Value = serde_json::from_str(
        &std::fs::read_to_string(source_root.join(relative)).expect("fixture"),
    )
    .expect("fixture JSON");
    let serialized =
        crate::source::dto::SerializedSourceObject::from_json(&raw).expect("hazard source object");
    let parsed = parse_hazard_source(
        pinned_source_version_metadata(),
        SourceIdentity::new("hazards:BHq5wpQU8hQEke8D", relative),
        raw.clone(),
        &serialized,
    )
    .expect("typed hazard source");
    assert_eq!(parsed.raw_json_for_audit(), &raw);
    assert_eq!(parsed.source.id, "BHq5wpQU8hQEke8D");
}

fn normalize_fixture(pack: &str, relative: &str) -> crate::records::LoadedSourceRecord {
    let source_root = source_root();
    let serialized = std::fs::read(source_root.join(relative)).expect("fixture source");
    normalize_record_from_source_bytes(
        &ManifestPack {
            name: pack.to_string(),
            label: pack.to_string(),
            document_type: "Actor".to_string(),
            path: format!("packs/{pack}"),
        },
        &PackName::new(pack.to_string()).expect("pack"),
        &source_root.join(relative),
        &source_root,
        &serialized,
        None,
    )
    .expect("hazard normalization")
}

fn normalize_raw(pack: &str, relative: &str, raw: Value) -> crate::records::LoadedSourceRecord {
    let source_root = source_root();
    let path = source_root.join(relative);
    normalize_record(
        &ManifestPack {
            name: pack.to_string(),
            label: pack.to_string(),
            document_type: "Actor".to_string(),
            path: format!("packs/{pack}"),
        },
        &PackName::new(pack.to_string()).expect("pack"),
        &path,
        &source_root,
        raw,
        None,
    )
    .expect("hazard normalization")
}

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hazards/pinned")
}
