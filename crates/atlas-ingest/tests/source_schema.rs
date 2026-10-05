use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use atlas_ingest::{SourcePathAuditOptions, SourcePathAuditReport, audit_source_paths};
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
