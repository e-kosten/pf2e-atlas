use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use atlas_ingest::{
    SourceCoverageDiagnosticKind, SourcePathAuditOptions, SourcePathCoverageDisposition,
    audit_source_paths,
};
use serde_json::{Value, json};

static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn meaningful_unknowns_warn_relaxed_and_fail_strict_without_empty_spam()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("meaningful");
    write_actor_source(
        &root,
        json!({
            "_id": "npc-coverage-1",
            "name": "Coverage Creature",
            "type": "npc",
            "system": {
                "details": {"level": {"value": 3}},
                "skills": {"acrobatics": {"mod": 9}},
                "futureField": {"value": 0},
                "emptyNull": null,
                "emptyString": "",
                "emptyArray": [],
                "emptyObject": {}
            },
            "items": [],
            "effects": [],
            "ownership": {"default": 0}
        }),
    )?;

    let relaxed = audit(&root, false, None)?;
    assert!(relaxed.enforcement.passed);
    assert_eq!(relaxed.summary.unknown_paths, 1);
    assert_eq!(relaxed.enforcement.aggregate_warning_count, 1);
    assert!(relaxed.paths.iter().any(|path| {
        path.path == "$.system.futureField.value"
            && path.disposition == SourcePathCoverageDisposition::Unknown
    }));
    assert!(!relaxed.paths.iter().any(|path| {
        matches!(
            path.path.as_str(),
            "$.system.emptyNull"
                | "$.system.emptyString"
                | "$.system.emptyArray"
                | "$.system.emptyObject"
        )
    }));
    assert!(relaxed.paths.iter().any(|path| {
        path.path == "$.system.skills.*.mod"
            && path.disposition == SourcePathCoverageDisposition::Consumed
    }));

    let strict = audit(&root, true, None)?;
    assert!(!strict.enforcement.passed);
    assert_eq!(strict.enforcement.violation_count, 1);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn b1_type_drift_is_aggregated_and_strictly_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("type-drift");
    write_actor_source(
        &root,
        json!({
            "_id": "npc-coverage-2",
            "name": "Drift Creature",
            "type": "npc",
            "system": {"details": {"level": {"value": 2}}},
            "items": {"not": "an array"}
        }),
    )?;

    let report = audit(&root, true, None)?;
    assert!(!report.enforcement.passed);
    assert_eq!(report.summary.type_drift_diagnostics, 1);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == SourceCoverageDiagnosticKind::MalformedShape
            && diagnostic.json_path == "$.items"
            && diagnostic.occurrence_count == 1
    }));
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn exact_future_family_owner_preserves_creature_first_policy()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("future-weapon");
    write_item_source(
        &root,
        json!({
            "_id": "weapon-coverage-1",
            "name": "Future Weapon",
            "type": "weapon",
            "system": {"runes": {"potency": 1}}
        }),
    )?;

    let report = audit(&root, true, None)?;
    assert!(report.enforcement.passed);
    let path = report
        .paths
        .iter()
        .find(|path| path.path == "$.system.runes.potency")
        .expect("weapon rune path");
    assert_eq!(path.disposition, SourcePathCoverageDisposition::Deferred);
    assert_eq!(path.future_owner.as_deref(), Some("H3"));
    assert_eq!(
        path.future_plan.as_deref(),
        Some("future/physical-items/weapons-ammunition/plan.md")
    );
    assert_eq!(report.registry_assignment_count, 313);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn vendor_diff_and_json_are_stable_and_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("vendor-diff");
    write_actor_source(
        &root,
        json!({
            "_id": "npc-coverage-3",
            "name": "Stable Creature",
            "type": "npc",
            "system": {"details": {"level": {"value": 1}}},
            "items": []
        }),
    )?;
    let baseline = audit(&root, false, None)?;
    assert_eq!(
        serde_json::to_string(&baseline)?,
        serde_json::to_string(&audit(&root, false, None)?)?
    );
    let baseline_path = root.join("coverage-baseline.json");
    fs::write(&baseline_path, serde_json::to_vec_pretty(&baseline)?)?;

    let mut truncated = serde_json::to_value(&baseline)?;
    truncated["paths"]
        .as_array_mut()
        .expect("baseline paths")
        .pop();
    let truncated_path = root.join("coverage-baseline-truncated.json");
    fs::write(&truncated_path, serde_json::to_vec_pretty(&truncated)?)?;
    let truncated_error = audit(&root, true, Some(truncated_path))
        .expect_err("truncated coverage baseline must fail closed");
    assert!(truncated_error.to_string().contains("is truncated"));

    write_actor_source(
        &root,
        json!({
            "_id": "npc-coverage-3",
            "name": "Stable Creature",
            "type": "npc",
            "system": {
                "details": {"level": {"value": 1}},
                "newVendorField": {"value": true}
            },
            "items": []
        }),
    )?;
    let changed = audit(&root, true, Some(baseline_path))?;
    assert!(!changed.enforcement.passed);
    let diff = changed.source_diff.expect("source diff");
    assert_eq!(diff.added_paths.len(), 1);
    assert_eq!(diff.added_paths[0].path, "$.system.newVendorField.value");
    assert_eq!(diff.removed_paths.len(), 0);
    assert_eq!(diff.changed_dispositions.len(), 0);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn npc_new_and_stale_children_are_unknown_instead_of_hidden_by_parent_rules()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("npc-child-mutations");
    write_actor_source(
        &root,
        json!({
            "_id": "npc-coverage-child-mutations",
            "name": "Mutation Creature",
            "type": "npc",
            "system": {
                "abilities": {"str": {"mod": 4, "futureLeaf": true}},
                "attributes": {
                    "futureLeaf": true,
                    "hp": {"max": 20, "maxxFuture": 21}
                },
                "details": {"level": {"value": 3}, "futureLeaf": true},
                "initiative": {"futureLeaf": true},
                "perception": {"mod": 8, "futureLeaf": true},
                "resources": {"focus": {"futureLeaf": true}},
                "saves": {"fortitude": {"value": 10, "futureLeaf": true}},
                "skills": {"acrobatics": {"mod": 9, "futureLeaf": true}},
                "spellcasting": {"futureLeaf": true}
            },
            "items": [{
                "_id": "embedded-mutation",
                "name": "Mutation Item",
                "type": "action",
                "system": {
                    "description": {"value": "Owned", "staleChild": true},
                    "futureLeaf": true
                },
                "flags": {"pf2e": {"futureLeaf": true}}
            }],
            "prototypeToken": {"name": "Mutation Creature", "futureLeaf": true}
        }),
    )?;

    let report = audit(&root, true, None)?;
    assert!(!report.enforcement.passed);
    let expected_unknowns = [
        "$.items[].flags.pf2e.futureLeaf",
        "$.items[].system.description.staleChild",
        "$.items[].system.futureLeaf",
        "$.prototypeToken.futureLeaf",
        "$.system.abilities.str.futureLeaf",
        "$.system.attributes.futureLeaf",
        "$.system.attributes.hp.maxxFuture",
        "$.system.details.futureLeaf",
        "$.system.initiative.futureLeaf",
        "$.system.perception.futureLeaf",
        "$.system.resources.*.futureLeaf",
        "$.system.saves.*.futureLeaf",
        "$.system.skills.*.futureLeaf",
        "$.system.spellcasting.futureLeaf",
    ];
    for expected in expected_unknowns {
        assert!(
            report.paths.iter().any(|path| {
                path.path == expected && path.disposition == SourcePathCoverageDisposition::Unknown
            }),
            "missing unknown mutation {expected}"
        );
    }
    assert_eq!(report.summary.unknown_paths, expected_unknowns.len());
    assert_eq!(report.summary.generic_deferred_paths, 0);
    assert_eq!(report.summary.unowned_recursive_matches, 0);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn consumed_to_deferred_regression_is_explicit_and_strictly_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("consumed-regression");
    write_actor_source(
        &root,
        json!({
            "_id": "npc-coverage-consumed-regression",
            "name": "Regression Creature",
            "type": "npc",
            "system": {
                "details": {"level": {"value": 3}},
                "spellcasting": {"rituals": {"dc": 20}}
            },
            "items": []
        }),
    )?;
    let baseline = audit(&root, false, None)?;
    let mut baseline_json = serde_json::to_value(&baseline)?;
    let deferred_path = baseline_json["paths"]
        .as_array_mut()
        .expect("baseline paths")
        .iter_mut()
        .find(|path| path["path"] == "$.system.spellcasting.rituals.dc")
        .expect("deferred ritual DC path");
    deferred_path["disposition"] = json!("consumed");
    let baseline_path = root.join("coverage-consumed-baseline.json");
    fs::write(&baseline_path, serde_json::to_vec_pretty(&baseline_json)?)?;

    let report = audit(&root, true, Some(baseline_path))?;
    assert!(!report.enforcement.passed);
    assert_eq!(report.summary.consumed_regressions, 1);
    let diff = report.source_diff.expect("source diff");
    assert_eq!(diff.consumed_regressions.len(), 1);
    assert_eq!(
        diff.consumed_regressions[0].path,
        "$.system.spellcasting.rituals.dc"
    );
    assert_eq!(
        diff.consumed_regressions[0].baseline_disposition,
        "consumed"
    );
    assert_eq!(diff.consumed_regressions[0].current_disposition, "deferred");
    fs::remove_dir_all(root)?;
    Ok(())
}

fn audit(
    root: &Path,
    strict: bool,
    baseline_report: Option<PathBuf>,
) -> Result<atlas_ingest::SourcePathAuditReport, atlas_ingest::IngestError> {
    audit_source_paths(SourcePathAuditOptions {
        source_root: root.to_path_buf(),
        manifest_path: None,
        pack_name: None,
        document_type: None,
        record_type: None,
        min_records: 1,
        limit: Some(1_000_000),
        strict,
        baseline_report,
    })
}

fn fixture_root(label: &str) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "atlas-ingest-source-coverage-{label}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}

fn write_actor_source(root: &Path, record: Value) -> Result<(), Box<dyn std::error::Error>> {
    write_source(root, "Actor", "actors", record)
}

fn write_item_source(root: &Path, record: Value) -> Result<(), Box<dyn std::error::Error>> {
    write_source(root, "Item", "items", record)
}

fn write_source(
    root: &Path,
    document_type: &str,
    pack_name: &str,
    record: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let pack_path = root.join("packs").join(pack_name);
    fs::create_dir_all(&pack_path)?;
    fs::write(
        root.join("module.json"),
        serde_json::to_vec_pretty(&json!({
            "packs": [{
                "name": pack_name,
                "label": "Coverage Fixtures",
                "type": document_type,
                "path": format!("packs/{pack_name}")
            }]
        }))?,
    )?;
    fs::write(
        pack_path.join("fixture.json"),
        serde_json::to_vec_pretty(&record)?,
    )?;
    Ok(())
}
