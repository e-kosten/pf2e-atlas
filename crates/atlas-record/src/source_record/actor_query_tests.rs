use super::*;
use crate::source_record::{FieldAvailability, SourceQueryView};
use atlas_foundry_model::{
    FoundryDocumentSource, SourceContext, SourcePresence, admit_document_source,
};
use serde_json::{Value, json};

/// Local developer evidence; ordinary tests use portable fixtures below. The
/// independent JSON reader here is an oracle, never a production extraction path.
#[test]
#[ignore = "requires pinned PF2e export or vendor/pf2e"]
fn pinned_actor_corpus_matches_authored_values() {
    use std::{fs, path::Path};
    fn files(path: &Path, output: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                files(&path, output);
            } else if path.extension().is_some_and(|e| e == "json") {
                output.push(path);
            }
        }
    }
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|p| {
            [
                p.join("scratch/source-contracts-full/pf2e/packs"),
                p.join("vendor/pf2e/packs"),
            ]
            .into_iter()
            .find(|p| p.is_dir())
        })
        .expect("pinned source export or vendor/pf2e/packs");
    let mut paths = Vec::new();
    files(&source, &mut paths);
    let mut count = 0;
    for path in paths {
        let bytes = fs::read(&path).unwrap();
        let raw: Value = serde_json::from_slice(&bytes).unwrap();
        if !matches!(
            raw["type"].as_str(),
            Some(
                "npc" | "hazard" | "character" | "army" | "vehicle" | "loot" | "familiar" | "party"
            )
        ) {
            continue;
        }
        let admitted = admit_document_source(
            "Actor",
            SourceContext::new(path.display().to_string(), "corpus", "$"),
            &bytes,
        )
        .unwrap();
        let model = admitted.model.unwrap();
        let q = query(&model);
        count += 1;
        let scalar = |field: SourceFieldView<'_, &Number>, pointer: &str| {
            if let Some(value) = field.value() {
                assert_eq!(
                    raw.pointer(pointer),
                    Some(&Value::Number(value.clone())),
                    "{} {pointer}",
                    path.display()
                );
            }
        };
        scalar(q.level(), "/system/details/level/value");
        scalar(q.armor_class(), "/system/attributes/ac/value");
        scalar(q.hp_maximum(), "/system/attributes/hp/max");
        scalar(
            q.save(ActorSave::Fortitude),
            "/system/saves/fortitude/value",
        );
        scalar(q.save(ActorSave::Reflex), "/system/saves/reflex/value");
        scalar(q.save(ActorSave::Will), "/system/saves/will/value");
        scalar(q.perception(), "/system/perception/mod");
        scalar(q.land_speed(), "/system/attributes/speed/value");
        scalar(q.hazard_hardness(), "/system/attributes/hardness");
        if let Some(value) = q.hazard_complexity().value() {
            assert_eq!(
                raw.pointer("/system/details/isComplex"),
                Some(&Value::Bool(value))
            );
        }
        if let Some(value) = q.size().value() {
            assert_eq!(
                raw.pointer("/system/traits/size/value"),
                Some(&serde_json::to_value(value).unwrap())
            );
        }
        if let Some(value) = q.languages().value() {
            assert_eq!(
                raw.pointer("/system/details/languages/value"),
                Some(&serde_json::to_value(value).unwrap())
            );
        }
        if let Some(value) = q.speeds().value() {
            let entries = raw
                .pointer("/system/attributes/speed/otherSpeeds")
                .unwrap()
                .as_array()
                .unwrap();
            assert_eq!(value.len(), entries.len());
            for (entry, raw) in value.iter().zip(entries) {
                if let Some(value) = entry.speed_type().value() {
                    assert_eq!(raw.get("type"), Some(&serde_json::to_value(value).unwrap()));
                }
                if let Some(value) = entry.value().value() {
                    assert_eq!(raw.get("value"), Some(&Value::Number(value.clone())));
                }
            }
        }
        for (kind, field) in [
            (ActorIwrKind::Immunity, "immunities"),
            (ActorIwrKind::Weakness, "weaknesses"),
            (ActorIwrKind::Resistance, "resistances"),
        ] {
            if let Some(value) = q.iwr(kind).value() {
                let raw = raw.pointer(&format!("/system/attributes/{field}"));
                let entries = raw
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                assert_eq!(value.len(), entries.len(), "{} {field}", path.display());
                for (entry, raw) in value.iter().zip(entries) {
                    if let Some(value) = entry.iwr_type().value() {
                        let value = match value {
                            ActorIwrType::Immunity(v) => serde_json::to_value(v),
                            ActorIwrType::Weakness(v) => serde_json::to_value(v),
                            ActorIwrType::Resistance(v) => serde_json::to_value(v),
                        }
                        .unwrap();
                        assert_eq!(raw.get("type"), Some(&value));
                    }
                    if let Some(value) = entry.value().value() {
                        assert_eq!(raw.get("value"), Some(&Value::Number(value.clone())));
                    }
                }
            }
        }
    }
    assert_eq!(count, 6744, "pinned PF2e actor corpus changed");
}

fn actor(family: &str, system: Value) -> FoundryDocumentSource {
    admit_document_source("Actor", SourceContext::new("fixture", "test", "$"),
        &serde_json::to_vec(&json!({"_id":"aaaaaaaaaaaaaaaa","name":family,"type":family,"system":system,"items":[]})).unwrap())
        .unwrap().model.unwrap()
}
fn query(source: &FoundryDocumentSource) -> ActorQueryView<'_> {
    SourceQueryView::new(source, "test", "Test").actor()
}

#[test]
fn family_applicability_is_authored_not_runtime() {
    for family in [
        "npc",
        "hazard",
        "character",
        "army",
        "vehicle",
        "loot",
        "familiar",
        "party",
    ] {
        let source = actor(
            family,
            json!({"details":{"level":{"value":-1},"languages":{"value":[]}},
            "traits":{"size":{"value":"med"},"value":["new-trait"],"rarity":"rare"},
            "attributes":{"ac":{"value":0},"hp":{"max":0},"speed":{"value":0,"otherSpeeds":[]}},
            "saves":{"fortitude":{"value":0},"reflex":{"value":0},"will":{"value":0}},
            "perception":{"mod":0,"senses":[]}}),
        );
        let q = query(&source);
        let applicable = |state: FieldAvailability, yes: bool| {
            assert_eq!(state == FieldAvailability::NotApplicable, !yes, "{family}")
        };
        applicable(
            q.level().availability(),
            !matches!(family, "familiar" | "party"),
        );
        applicable(
            q.size().availability(),
            matches!(family, "npc" | "hazard" | "vehicle"),
        );
        applicable(
            q.armor_class().availability(),
            matches!(family, "npc" | "hazard"),
        );
        applicable(
            q.hp_maximum().availability(),
            matches!(family, "npc" | "hazard"),
        );
        for save in [ActorSave::Fortitude, ActorSave::Reflex, ActorSave::Will] {
            applicable(
                q.save(save).availability(),
                matches!(family, "npc" | "hazard"),
            );
        }
        applicable(q.perception().availability(), family == "npc");
        applicable(q.senses().availability(), family == "npc");
        applicable(
            q.languages().availability(),
            matches!(family, "npc" | "character"),
        );
        applicable(
            q.land_speed().availability(),
            matches!(family, "npc" | "character"),
        );
        applicable(
            q.speeds().availability(),
            matches!(family, "npc" | "character"),
        );
        for kind in [
            ActorIwrKind::Immunity,
            ActorIwrKind::Weakness,
            ActorIwrKind::Resistance,
        ] {
            applicable(
                q.iwr(kind).availability(),
                matches!(family, "npc" | "hazard" | "character" | "army" | "vehicle"),
            );
        }
    }
}

#[test]
fn iwr_defaults_only_missing_leaf_under_known_ancestors() {
    for family in ["npc", "hazard", "character", "army", "vehicle"] {
        let source = actor(family, json!({"attributes":{}}));
        for kind in [
            ActorIwrKind::Immunity,
            ActorIwrKind::Weakness,
            ActorIwrKind::Resistance,
        ] {
            assert!(
                query(&source).iwr(kind).value().unwrap().is_empty(),
                "{family}"
            );
        }
        // Query default does not change the authored source.
        match &source {
            FoundryDocumentSource::Actor(a) => {
                if let ActorSourcePF2e::NPCSource(s) = a.as_ref() {
                    assert!(matches!(
                        s.system
                            .as_value()
                            .unwrap()
                            .attributes
                            .as_value()
                            .unwrap()
                            .immunities,
                        SourcePresence::Missing
                    ));
                }
            }
            _ => unreachable!(),
        }
        for (system, expected) in [
            (json!({}), FieldAvailability::Missing),
            (json!({"attributes":null}), FieldAvailability::Null),
            (
                json!({"attributes":{"resistances":null}}),
                FieldAvailability::Null,
            ),
        ] {
            let source = actor(family, system);
            assert_eq!(
                query(&source).iwr(ActorIwrKind::Resistance).availability(),
                expected,
                "{family}"
            );
        }
        for system in [
            json!({"attributes":false}),
            json!({"attributes":{"resistances":false}}),
            json!({"attributes":{"resistances":[false]}}),
        ] {
            let source = actor(family, system);
            assert!(
                matches!(
                    query(&source).iwr(ActorIwrKind::Resistance).availability(),
                    FieldAvailability::Invalid { .. }
                ),
                "{family}"
            );
        }
    }
    let source = actor(
        "npc",
        json!({"attributes":{},"details":{"languages":{}},"perception":{}}),
    );
    assert_eq!(
        query(&source).languages().availability(),
        FieldAvailability::Missing
    );
    assert_eq!(
        query(&source).speeds().availability(),
        FieldAvailability::Missing
    );
    assert_eq!(
        query(&source).senses().availability(),
        FieldAvailability::Missing
    );
}

#[test]
fn scalar_states_and_baselines_remain_distinct() {
    for (system, expected) in [
        (json!({}), FieldAvailability::Missing),
        (Value::Null, FieldAvailability::Null),
        (json!({"attributes":null}), FieldAvailability::Null),
        (json!({"attributes":{"hp":null}}), FieldAvailability::Null),
        (json!({"attributes":{"hp":{}}}), FieldAvailability::Missing),
        (
            json!({"attributes":{"hp":{"max":null}}}),
            FieldAvailability::Null,
        ),
    ] {
        let source = actor("npc", system);
        assert_eq!(query(&source).hp_maximum().availability(), expected);
    }
    let source = actor(
        "npc",
        json!({"attributes":{"adjustment":"elite","hp":{"max":0,"value":20},"ac":{"value":-1}},"details":{"level":{"value":5}},"saves":{"fortitude":{"value":0},"reflex":{"value":-1},"will":{"value":"2"}}}),
    );
    assert_eq!(query(&source).level().value().unwrap().as_i64(), Some(5));
    assert_eq!(
        query(&source).hp_maximum().value().unwrap().as_i64(),
        Some(0)
    );
    assert_eq!(
        query(&source).armor_class().value().unwrap().as_i64(),
        Some(-1)
    );
    assert_eq!(
        query(&source)
            .save(ActorSave::Fortitude)
            .value()
            .unwrap()
            .as_i64(),
        Some(0)
    );
    assert!(matches!(
        query(&source).save(ActorSave::Will).availability(),
        FieldAvailability::Invalid { .. }
    ));
    assert_eq!(
        query(&source).npc_adjustment().value(),
        Some(&NPCAttributesSourceAdjustment::Elite)
    );
    let source = actor(
        "hazard",
        json!({"details":{"isComplex":false},"attributes":{"hardness":0}}),
    );
    assert_eq!(query(&source).hazard_complexity().value(), Some(false));
    assert_eq!(
        query(&source).hazard_hardness().value().unwrap().as_i64(),
        Some(0)
    );
}

#[test]
fn collection_members_keep_same_entry_availability_and_authored_order() {
    for family in ["npc", "character"] {
        let source = actor(
            family,
            json!({"attributes":{"speed":{"value":null,"otherSpeeds":[{"type":"fly","value":10},{"type":"swim","value":40},{"type":"fly","value":null},{"type":"fly","value":0}]}}}),
        );
        assert_eq!(
            query(&source).land_speed().availability(),
            FieldAvailability::Null
        );
        let entries = query(&source).speeds().value().unwrap();
        assert_eq!(entries.len(), 4);
        assert!(!entries.iter().any(|s| {
            matches!(
                s.speed_type().value(),
                Some(CharacterAttributesSourceSpeedOtherSpeedsEntryType::Fly)
            ) && s
                .value()
                .value()
                .and_then(Number::as_i64)
                .is_some_and(|v| v >= 40)
        }));
        assert_eq!(
            entries.get(2).unwrap().value().availability(),
            FieldAvailability::Null
        );
        assert_eq!(
            entries.get(3).unwrap().value().value().unwrap().as_i64(),
            Some(0)
        );
    }
    for family in ["npc", "hazard"] {
        let source = actor(
            family,
            json!({"attributes":{"resistances":[{"type":"fire","value":2},{"type":"cold","value":10},{"type":"fire","value":"5"}],"immunities":[{"type":"poison"}]}}),
        );
        let entries = query(&source)
            .iwr(ActorIwrKind::Resistance)
            .value()
            .unwrap();
        assert_eq!(entries.len(), 3);
        assert!(!entries.iter().any(|s| {
            matches!(
                s.iwr_type().value(),
                Some(ActorIwrType::Resistance(ResistanceSourceType::Fire))
            ) && s
                .value()
                .value()
                .and_then(Number::as_i64)
                .is_some_and(|v| v >= 5)
        }));
        assert!(matches!(
            entries.get(2).unwrap().value().availability(),
            FieldAvailability::Invalid { .. }
        ));
        assert_eq!(
            query(&source)
                .iwr(ActorIwrKind::Immunity)
                .value()
                .unwrap()
                .get(0)
                .unwrap()
                .value()
                .availability(),
            FieldAvailability::NotApplicable
        );
    }
    let source = actor(
        "npc",
        json!({"perception":{"senses":[{"type":"darkvision"}],"mod":0},"details":{"languages":{"value":[],"details":"granted language"}}}),
    );
    assert_eq!(
        query(&source).senses().value().unwrap()[0].range,
        SourcePresence::Missing
    );
    assert!(query(&source).languages().value().unwrap().is_empty());
}
