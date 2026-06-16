use std::fs;
use std::path::PathBuf;

use atlas_domain::RecordKey;
use time::OffsetDateTime;

use super::*;
use crate::{LocalStateError, LocalStateStore};

#[test]
fn saved_lists_preserve_order_and_noop_duplicate_adds() -> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("saved-lists-order"))?;
    let lists = store.saved_lists();
    lists.create(NewSavedList {
        slug: "undead-research".to_string(),
        name: "Undead Research".to_string(),
        description: Some("Campaign notes".to_string()),
    })?;

    assert_eq!(
        lists.add_resolved_item(
            "undead-research",
            resolved_item("actions:first", None, "First", Some("rule")),
        )?,
        AddSavedListItemOutcome::Added
    );
    lists.add_resolved_item(
        "undead-research",
        resolved_item("actions:second", Some("important"), "Second", Some("rule")),
    )?;
    assert_eq!(
        lists.add_resolved_item(
            "undead-research",
            resolved_item(
                "actions:first",
                Some("ignored"),
                "First duplicate",
                Some("rule"),
            ),
        )?,
        AddSavedListItemOutcome::AlreadyPresent
    );

    let list = lists
        .get_with_items("undead-research")?
        .expect("list should exist");
    assert_eq!(list.list.name, "Undead Research");
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].record_key, "actions:first");
    assert_eq!(list.items[0].position, 1);
    assert_eq!(list.items[1].record_key, "actions:second");
    assert_eq!(list.items[1].position, 2);
    Ok(())
}

#[test]
fn remove_item_compacts_positions() -> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("saved-lists-remove"))?;
    let lists = store.saved_lists();
    lists.create(NewSavedList {
        slug: "test-list".to_string(),
        name: "Test List".to_string(),
        description: None,
    })?;
    for key in ["actions:first", "actions:second", "actions:third"] {
        lists.add_resolved_item("test-list", resolved_item(key, None, key, None))?;
    }

    assert!(lists.remove_item("test-list", "actions:second")?);
    let list = lists
        .get_with_items("test-list")?
        .expect("list should exist");
    assert_eq!(list.items[0].record_key, "actions:first");
    assert_eq!(list.items[0].position, 1);
    assert_eq!(list.items[1].record_key, "actions:third");
    assert_eq!(list.items[1].position, 2);
    Ok(())
}

#[test]
fn invalid_slugs_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("saved-lists-slug"))?;
    let result = store.saved_lists().create(NewSavedList {
        slug: "Bad Slug".to_string(),
        name: "Bad".to_string(),
        description: None,
    });
    assert!(matches!(result, Err(LocalStateError::InvalidSlug { .. })));
    Ok(())
}

#[test]
fn invalid_record_keys_are_rejected_on_remove() -> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("saved-lists-record-key"))?;
    let lists = store.saved_lists();
    lists.create(NewSavedList {
        slug: "test-list".to_string(),
        name: "Test List".to_string(),
        description: None,
    })?;
    let result = lists.remove_item("test-list", "not a key");
    assert!(matches!(
        result,
        Err(LocalStateError::InvalidRecordKey { .. })
    ));
    Ok(())
}

#[test]
fn hydrated_item_projection_marks_active_and_unresolved_rows() {
    let active = hydrate_saved_list_item(
        SavedListItem {
            record_key: "actions:first".to_string(),
            position: 1,
            note: Some("note".to_string()),
            record_title_snapshot: "First Snapshot".to_string(),
            record_kind_snapshot: Some("rule".to_string()),
            added_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        },
        Some("hydrated record"),
    );
    assert_eq!(active.status, SavedListItemStatus::Active);
    assert_eq!(active.snapshot.title, "First Snapshot");

    let unresolved = hydrate_saved_list_item::<&str>(
        SavedListItem {
            record_key: "actions:missing".to_string(),
            position: 2,
            note: None,
            record_title_snapshot: "Missing Snapshot".to_string(),
            record_kind_snapshot: None,
            added_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        },
        None,
    );
    assert_eq!(unresolved.status, SavedListItemStatus::Unresolved);
    assert_eq!(unresolved.snapshot.title, "Missing Snapshot");
}

fn resolved_item(
    record_key: &str,
    note: Option<&str>,
    title_snapshot: &str,
    kind_snapshot: Option<&str>,
) -> ResolvedSavedListItem {
    ResolvedSavedListItem {
        record_key: RecordKey::parse(record_key).expect("fixture record key should parse"),
        title_snapshot: title_snapshot.to_string(),
        kind_snapshot: kind_snapshot.map(str::to_string),
        note: note.map(str::to_string),
    }
}

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-local-state-{name}-{}-{}.sqlite",
        std::process::id(),
        OffsetDateTime::now_utc().unix_timestamp_nanos()
    ));
    let _ = fs::remove_file(&path);
    path
}
