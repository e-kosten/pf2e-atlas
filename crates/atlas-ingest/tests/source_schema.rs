use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use atlas_ingest::{
    SourcePathAuditFilters, SourcePathAuditOptions, SourcePathAuditReport,
    SourceValueDiscoveryOptions, SourceValueDiscoveryReport, audit_source_paths,
    discover_source_values,
};
use serde_json::json;

struct SourceFixture(PathBuf);
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

impl SourceFixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("atlas-schema-{}-{nonce}-{id}", std::process::id()));
        fs::create_dir_all(root.join("packs/items")).expect("create fixture");
        fs::write(
            root.join("module.json"),
            json!({"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"}]})
                .to_string(),
        )
        .expect("manifest");
        Self(root)
    }

    fn record(&self, name: &str, json: &str) {
        fs::write(self.0.join("packs/items").join(name), json).expect("record");
    }
    fn options(&self) -> SourcePathAuditOptions {
        SourcePathAuditOptions {
            source_root: self.0.clone(),
            ..Default::default()
        }
    }
    fn scan(&self) -> SourcePathAuditReport {
        audit_source_paths(self.options()).expect("schema discovery")
    }
    fn baseline(&self, report: &SourcePathAuditReport) -> PathBuf {
        let path = self.0.join("baseline.json");
        fs::write(&path, serde_json::to_vec(report).expect("serialize")).expect("baseline");
        path
    }
}

impl Drop for SourceFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn value_options(source: &SourceFixture, path: &str) -> SourceValueDiscoveryOptions {
    SourceValueDiscoveryOptions {
        source_root: source.0.clone(),
        manifest_path: None,
        filters: SourcePathAuditFilters {
            record_type: Some("spell".into()),
            ..Default::default()
        },
        path: path.into(),
        sample_limit: 2,
        limit: None,
    }
}

fn values(source: &SourceFixture, path: &str) -> SourceValueDiscoveryReport {
    discover_source_values(value_options(source, path)).expect("value discovery")
}

#[test]
fn value_discovery_counts_all_values_and_samples_rare_variants() {
    let source = SourceFixture::new();
    source.record("a.json", r#"{"_id":"same","type":"spell","tags":["common","common",null,false,0,"",{},[]],"duplicate":"first","duplicate":null}"#);
    source.record(
        "b.json",
        r#"{"_id":"same","type":"spell","tags":["common","rare"]}"#,
    );
    source.record("c.json", r#"{"type":"spell"}"#);
    source.record("d.json", r#"{"type":"action","tags":["excluded"]}"#);
    let report = values(&source, "$.tags[]");
    assert_eq!(report.record_count, 3);
    assert_eq!(report.fields.len(), 1);
    let field = &report.fields[0];
    assert_eq!(field.record_count, 2);
    assert_eq!(field.missing_record_count, 1);
    assert_eq!(field.occurrence_count, 10);
    assert_eq!(field.distinct_value_count, 8);
    assert!(report.complete);
    let common = &field.values[0];
    assert_eq!(common.value_json, "\"common\"");
    assert_eq!(common.occurrence_count, 3);
    assert_eq!(
        common.record_count, 2,
        "count documents, not IDs or occurrences"
    );
    assert_eq!(common.examples.len(), 2);
    assert_eq!(common.examples[1].source_pointer, "/tags/0");
    assert_eq!(common.examples[1].source_path, "packs/items/b.json");
    for expected in ["null", "false", "0", "\"\"", "{}", "[]", "\"rare\""] {
        let value = field
            .values
            .iter()
            .find(|value| value.value_json == expected)
            .expect("distinct value");
        assert_eq!(value.occurrence_count, 1);
        assert!(!value.examples.is_empty(), "rare values retain references");
    }
    let rare = field
        .values
        .iter()
        .find(|value| value.value_json == "\"rare\"")
        .unwrap();
    assert_eq!(rare.examples[0].source_path, "packs/items/b.json");
    assert_eq!(rare.examples[0].source_pointer, "/tags/1");
    assert_eq!(
        values(&source, "$.tags[]"),
        report,
        "deterministic ordering"
    );

    let duplicate = values(&source, "$.duplicate");
    let field = &duplicate.fields[0];
    assert_eq!((field.record_count, field.missing_record_count), (1, 2));
    assert_eq!(
        (field.occurrence_count, field.duplicate_member_count),
        (2, 2)
    );
    assert_eq!(
        field.distinct_value_count, 2,
        "duplicate members cannot be collapsed by JSON parsing"
    );

    let absent = values(&source, "$.absent");
    assert_eq!(absent.fields[0].missing_record_count, 3);
    assert!(absent.fields[0].values.is_empty());
    let mut options = value_options(&source, "$.tags[]");
    options.limit = Some(1);
    options.sample_limit = 0;
    let limited = discover_source_values(options).unwrap();
    assert!(!limited.complete);
    assert_eq!(limited.fields[0].distinct_value_count, 8);
    assert_eq!(limited.fields[0].occurrence_count, 10);
    assert_eq!(limited.fields[0].values.len(), 1);
    assert!(limited.fields[0].values[0].examples.is_empty());
}

#[test]
fn value_discovery_preserves_long_values_structures_and_concrete_keyed_references() {
    let source = SourceFixture::new();
    let prefix = "a".repeat(180);
    let record = json!({"type":"spell", "system":{"damage": {
        "id/~": {"formula": format!("{prefix}x")},
        "another": {"formula": format!("{prefix}y")}
    }}, "odd.key": [{"x": 1}] })
    .to_string();
    source.record("a.json", &record);
    let path = "$.system.damage.*.formula";
    let report = values(&source, path);
    assert_eq!(
        report.fields[0].distinct_value_count, 2,
        "untruncated values cannot collide"
    );
    let first = &report.fields[0].values[0];
    assert_eq!(first.value_json, format!("\"{prefix}x\""));
    assert_eq!(
        first.examples[0].source_pointer,
        "/system/damage/id~1~0/formula"
    );
    assert!(
        source
            .scan()
            .paths
            .iter()
            .any(|field| field.key.path == path)
    );
    let punctuated = values(&source, "$[\"odd.key\"][]");
    assert_eq!(punctuated.fields[0].values[0].value_json, "{\"x\":1}");
    assert_eq!(
        punctuated.fields[0].values[0].examples[0].source_pointer,
        "/odd.key/0"
    );

    let elsewhere = SourceFixture::new();
    elsewhere.record("a.json", &record);
    assert_eq!(
        values(&elsewhere, path),
        report,
        "checkout-independent signature and references"
    );
    source.record(
        "a.json",
        r#"{"type":"spell","x":{"a":1,"a":2},"y":{"a":2,"a":1}}"#,
    );
    assert_eq!(
        values(&source, "$.x").fields[0].values[0].value_json,
        "{\"a\":1,\"a\":2}"
    );
}

#[test]
fn value_discovery_keeps_families_and_source_filters_separate_and_propagates_errors() {
    let source = SourceFixture::new();
    source.record("a.json", r#"{"type":"spell","x":null}"#);
    source.record("b.json", r#"{"type":"action","x":1}"#);
    let mut options = value_options(&source, "$.x");
    options.filters.record_type = None;
    let report = discover_source_values(options.clone()).unwrap();
    assert_eq!(report.fields.len(), 2);
    assert_eq!(report.fields[0].key.record_type, "action");
    assert_eq!(report.fields[1].values[0].value_json, "null");
    options.filters.document_type = Some("Actor".into());
    let empty = discover_source_values(options.clone()).unwrap();
    assert_eq!(empty.pack_count, 0);
    assert!(empty.fields.is_empty());
    options.filters.document_type = None;
    options.filters.pack_name = Some("unknown".into());
    assert_eq!(discover_source_values(options).unwrap().record_count, 0);
    let mut invalid = value_options(&source, "x");
    assert!(discover_source_values(invalid.clone()).is_err());
    invalid.path = "$.x".into();
    source.record("a.json", "{");
    assert!(discover_source_values(invalid).is_err());
}

#[test]
fn discovers_all_shapes_without_losing_duplicate_values_or_document_counts() {
    let source = SourceFixture::new();
    source.record("a.json", r#"{"_id":"same","type":"new-family","system":{"flag":false,"zero":0,"blank":"","null":null,"empty":{},"array":[],"mixed":[0,null,{}],"duplicate":0,"duplicate":null,"skills":{"intimidate":{"base":3}},"damageRolls":{"arbitrary":{"damage":"1d6"}},"odd.key":{"x":1},"odd":{"key":2}}}"#);
    source.record(
        "b.json",
        r#"{"_id":"same","type":"new-family","system":{"zero":1}}"#,
    );
    let report = source.scan();
    assert_eq!(report.record_count, 2);
    let get = |path: &str| {
        report
            .paths
            .iter()
            .find(|entry| entry.key.path == path)
            .expect("path discovered")
    };
    assert_eq!(
        get("$.system.zero").record_count,
        2,
        "IDs are not record counters"
    );
    assert_eq!(get("$.system.duplicate").record_count, 1);
    assert_eq!(get("$.system.duplicate").occurrence_count, 2);
    assert_eq!(get("$.system.duplicate").duplicate_member_count, 2);
    assert_eq!(
        get("$.system.duplicate")
            .value_types
            .iter()
            .map(|kind| kind.kind.as_str())
            .collect::<Vec<_>>(),
        ["null", "number"]
    );
    assert_eq!(get("$.system.mixed[]").occurrence_count, 3);
    assert_eq!(get("$.system.empty").value_types[0].kind, "object");
    assert_eq!(get("$.system.array").value_types[0].kind, "array");
    assert_eq!(get("$.system.flag").examples[0].value, "false");
    assert_eq!(get("$.system.blank").examples[0].value, "\"\"");
    get("$.system.skills.intimidate.base");
    get("$.system.damageRolls.*.damage");
    get("$.system[\"odd.key\"].x");
    get("$.system.odd.key");
    assert_eq!(source.scan(), report, "stable ordering and samples");
    assert!(report.complete);
}

#[test]
fn diffs_shapes_but_ignores_value_frequency_and_checkout_churn() {
    let source = SourceFixture::new();
    source.record(
        "a.json",
        r#"{"type":"spell","old":1,"same":0,"changed":null,"duplicate":0,"duplicate":1}"#,
    );
    let before = source.scan();
    let baseline = source.baseline(&before);
    source.record(
        "a.json",
        r#"{"type":"spell","new":false,"same":5,"changed":[],"duplicate":0}"#,
    );
    let mut options = source.options();
    options.baseline_report = Some(baseline.clone());
    options.limit = Some(1);
    let after = audit_source_paths(options).expect("diff");
    let diff = after.source_diff.expect("diff");
    assert_eq!(diff.added_paths[0].path, "$.new");
    assert_eq!(diff.removed_paths[0].path, "$.old");
    assert_eq!(diff.changed_types[0].key.path, "$.changed");
    assert_eq!(diff.changed_types[0].before, ["null"]);
    assert_eq!(diff.changed_types[0].after, ["array"]);
    assert_eq!(diff.changed_duplicate_members[0].path, "$.duplicate");
    assert_eq!(diff.change_count(), 4);
    assert!(!after.complete, "limited output isn't a snapshot");

    source.record(
        "a.json",
        r#"{"type":"spell","old":9,"same":10,"changed":null,"duplicate":7,"duplicate":8}"#,
    );
    source.record(
        "b.json",
        r#"{"type":"spell","old":4,"same":4,"changed":null,"duplicate":3}"#,
    );
    let mut options = source.options();
    options.baseline_report = Some(baseline);
    let current = audit_source_paths(options).expect("diff");
    assert_eq!(
        current.source_diff.as_ref().expect("diff").change_count(),
        0
    );
    assert_ne!(current.source_signature, before.source_signature);
    let elsewhere = SourceFixture::new();
    elsewhere.record(
        "a.json",
        r#"{"type":"spell","old":9,"same":10,"changed":null,"duplicate":7,"duplicate":8}"#,
    );
    elsewhere.record(
        "b.json",
        r#"{"type":"spell","old":4,"same":4,"changed":null,"duplicate":3}"#,
    );
    let mut without_diff = current;
    without_diff.source_diff = None;
    assert_eq!(elsewhere.scan(), without_diff, "portable snapshot");
}

#[test]
fn rejects_partial_incompatible_and_duplicate_baselines() {
    let source = SourceFixture::new();
    source.record("a.json", r#"{"type":"spell","x":1}"#);
    let full = source.scan();
    for change in 0..5 {
        let mut bad = full.clone();
        match change {
            0 => bad.complete = false,
            1 => bad.schema_version = "legacy".into(),
            2 => bad.path_count += 1,
            3 => bad.filters.record_type = Some("npc".into()),
            _ => {
                bad.paths.push(bad.paths[0].clone());
                bad.path_count += 1;
            }
        }
        let mut options = source.options();
        options.baseline_report = Some(source.baseline(&bad));
        assert!(
            audit_source_paths(options).is_err(),
            "invalid baseline {change}"
        );
    }
    let mut limited = source.options();
    limited.min_records = 2;
    assert!(!audit_source_paths(limited).expect("limited").complete);
    let mut strict = source.options();
    strict.strict = true;
    assert!(audit_source_paths(strict).is_err());
}

#[test]
fn keyed_member_identity_churn_does_not_hide_new_member_fields() {
    let source = SourceFixture::new();
    source.record("a.json", r#"{"type":"spell","system":{"damage":{"old-id":{"formula":"1d6"}},"items":{"old-grant":{"level":1}},"rules":[{"damage":{"named-field":1}}]}}"#);
    let baseline = source.baseline(&source.scan());
    source.record("a.json", r#"{"type":"spell","system":{"damage":{"new-id":{"formula":"2d6","new-field":false}},"items":{"new-grant":{"level":2}},"rules":[{"damage":{"named-field":1,"added-field":false}}]}}"#);
    let mut options = source.options();
    options.baseline_report = Some(baseline);
    let diff = audit_source_paths(options)
        .expect("schema diff")
        .source_diff
        .expect("diff");
    assert_eq!(diff.change_count(), 2);
    assert_eq!(diff.added_paths[0].path, "$.system.damage.*.new-field");
    assert_eq!(
        diff.added_paths[1].path,
        "$.system.rules[].damage.added-field"
    );
}

#[test]
fn missing_packs_invalid_json_and_ambiguous_identity_are_errors() {
    let source = SourceFixture::new();
    for json in [
        "{",
        "[]",
        r#"{"type":"spell","type":"npc"}"#,
        r#"{"type":null}"#,
        r#"{"_id":0}"#,
    ] {
        source.record("a.json", json);
        assert!(audit_source_paths(source.options()).is_err(), "{json}");
    }
    fs::remove_dir_all(source.0.join("packs/items")).expect("remove pack");
    assert!(audit_source_paths(source.options()).is_err());
}
