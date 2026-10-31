use std::fs;
use std::path::PathBuf;

use atlas_domain::RecordKey;
use time::OffsetDateTime;

use super::*;
use crate::{LocalStateError, TestStore};

#[test]
fn saved_lists_order_by_recent_changes_with_slug_ties() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-recency"))?;
    let mut connection = store.connection()?;
    let transaction = connection.transaction()?;
    let lists = SavedListsWrite::from_connection(&transaction);
    for slug in ["zulu", "middle", "alpha"] {
        lists.create(NewSavedList {
            slug: slug.into(),
            name: slug.into(),
            description: None,
            tags: vec![],
        })?;
        lists.add_resolved_item(slug, resolved_item("actions:first", None, "First", None))?;
    }
    transaction.execute_batch(
        "UPDATE saved_lists SET updated_at='2026-01-01T00:00:00Z';
         UPDATE saved_lists SET updated_at='2026-01-02T00:00:00Z' WHERE slug='middle';",
    )?;
    assert_eq!(
        lists
            .list()?
            .iter()
            .map(|list| list.slug.as_str())
            .collect::<Vec<_>>(),
        ["middle", "alpha", "zulu"]
    );
    lists.update_item_note("zulu", "actions:first", Some("Prepare this next"))?;
    assert_eq!(
        lists
            .list()?
            .iter()
            .map(|list| list.slug.as_str())
            .collect::<Vec<_>>(),
        ["zulu", "middle", "alpha"]
    );
    let before_duplicate = lists.list()?;
    assert_eq!(
        lists.add_resolved_item(
            "alpha",
            resolved_item("actions:first", Some("Ignored"), "First", None)
        )?,
        AddSavedListItemOutcome::AlreadyPresent
    );
    assert_eq!(lists.list()?, before_duplicate);
    transaction.commit()?;
    Ok(())
}

#[test]
fn saved_lists_preserve_order_and_noop_duplicate_adds() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-order"))?;
    let lists = store.saved_lists();
    lists.create(NewSavedList {
        slug: "undead-research".to_string(),
        name: "Undead Research".to_string(),
        description: Some("Campaign notes".to_string()),
        tags: vec![],
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
    let before_duplicate = lists.get_with_items("undead-research")?.unwrap();
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
    assert_eq!(list, before_duplicate);
    assert_eq!(list.items[0].record_title_snapshot, "First");
    assert_eq!(list.items[0].note, None);
    assert_eq!(list.items[1].note.as_deref(), Some("important"));
    assert!(list.list.list_key.starts_with("list_"));
    assert_eq!(list.list.name, "Undead Research");
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].record_key, "actions:first");
    assert_eq!(list.items[0].position, 1);
    assert_eq!(list.items[1].record_key, "actions:second");
    assert_eq!(list.items[1].position, 2);
    Ok(())
}

#[test]
fn failed_tag_insert_rolls_back_creation_and_metadata_replacement()
-> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("tags-rollback"))?;
    let lists = store.saved_lists();
    let original = lists.create(NewSavedList {
        slug: "original".into(),
        name: "Original".into(),
        description: Some("Old".into()),
        tags: vec!["old".into()],
    })?;
    store.connection()?.execute_batch(
        "CREATE TRIGGER reject_tag BEFORE INSERT ON saved_list_tags WHEN NEW.tag='reject'
         BEGIN SELECT RAISE(ABORT,'injected tag failure'); END;",
    )?;
    assert!(
        lists
            .create(NewSavedList {
                slug: "failed".into(),
                name: "Failed".into(),
                description: None,
                tags: vec!["first".into(), "reject".into()],
            })
            .is_err()
    );
    assert!(lists.get("failed")?.is_none());
    assert!(
        lists
            .update(UpdateSavedList {
                list_key: original.list_key.clone(),
                slug: "changed".into(),
                name: "Changed".into(),
                description: None,
                tags: vec!["first".into(), "reject".into()],
            })
            .is_err()
    );
    assert_eq!(lists.get(&original.list_key)?, Some(original));
    assert!(lists.get("changed")?.is_none());
    Ok(())
}

#[test]
fn saved_list_reorders_persist_snapshots_and_contiguous_positions()
-> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("list-reorder"))?;
    let lists = store.saved_lists();
    let list = lists.import(ImportSavedList {
        slug: "ordered".into(),
        name: "Ordered".into(),
        description: None,
        tags: vec![],
        replace: false,
        items: ["first", "second", "missing", "last"]
            .into_iter()
            .map(|key| ImportSavedListItem {
                record_key: format!("actions:{key}"),
                note: Some(format!("Note {key}")),
                record_title_snapshot: format!("Snapshot {key}"),
                record_kind_snapshot: Some("rule".into()),
            })
            .collect(),
    })?;
    assert!(lists.reorder_item(
        "ordered",
        ReorderSavedListItem {
            record_key: "actions:missing".into(),
            target_record_key: "actions:first".into(),
            placement: SavedListItemPlacement::Before,
        }
    )?);
    assert!(lists.reorder_item(
        &list.list.list_key,
        ReorderSavedListItem {
            record_key: "actions:first".into(),
            target_record_key: "actions:last".into(),
            placement: SavedListItemPlacement::After,
        }
    )?);
    let reopened = TestStore::open(store.path())?;
    let moved = reopened
        .saved_lists()
        .get_with_items("ordered")?
        .expect("persisted list");
    assert_eq!(
        moved
            .items
            .iter()
            .map(|item| item.record_key.as_str())
            .collect::<Vec<_>>(),
        [
            "actions:missing",
            "actions:second",
            "actions:last",
            "actions:first"
        ]
    );
    assert_eq!(
        moved
            .items
            .iter()
            .map(|item| item.position)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(moved.list.revision, list.list.revision + 2);
    for item in &moved.items {
        let original = list
            .items
            .iter()
            .find(|before| before.record_key == item.record_key)
            .expect("same members");
        assert_eq!(item.note, original.note);
        assert_eq!(item.record_title_snapshot, original.record_title_snapshot);
        assert_eq!(item.record_kind_snapshot, original.record_kind_snapshot);
        assert_eq!(item.added_at, original.added_at);
    }
    // Invalid anchors and self drops leave the entire persisted snapshot intact.
    for (source, target, expected) in [
        ("actions:first", "actions:absent", false),
        ("actions:absent", "actions:first", false),
        ("actions:first", "actions:first", true),
    ] {
        assert_eq!(
            lists.reorder_item(
                "ordered",
                ReorderSavedListItem {
                    record_key: source.into(),
                    target_record_key: target.into(),
                    placement: SavedListItemPlacement::After,
                }
            )?,
            expected
        );
        assert_eq!(lists.get_with_items("ordered")?.expect("list"), moved);
    }
    assert!(lists.remove_item("ordered", "actions:second")?);
    lists.add_resolved_item(
        "ordered",
        resolved_item("actions:new", None, "New", Some("rule")),
    )?;
    let appended = lists.get_with_items("ordered")?.expect("list");
    assert_eq!(
        appended
            .items
            .iter()
            .map(|item| (item.record_key.as_str(), item.position))
            .collect::<Vec<_>>(),
        [
            ("actions:missing", 1),
            ("actions:last", 2),
            ("actions:first", 3),
            ("actions:new", 4)
        ]
    );
    Ok(())
}

#[test]
fn failed_saved_list_reorder_rolls_back_inside_the_callers_transaction()
-> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("list-reorder-rollback"))?;
    let before = store.saved_lists().import(ImportSavedList {
        slug: "atomic".into(),
        name: "Atomic".into(),
        description: None,
        tags: vec![],
        replace: false,
        items: ["first", "second", "third"]
            .into_iter()
            .map(|key| ImportSavedListItem {
                record_key: format!("actions:{key}"),
                note: Some(format!("Note {key}")),
                record_title_snapshot: key.into(),
                record_kind_snapshot: None,
            })
            .collect(),
    })?;
    let mut connection = store.connection()?;
    connection.execute_batch("CREATE TRIGGER reject_position BEFORE UPDATE OF position ON saved_list_items WHEN NEW.position=2 BEGIN SELECT RAISE(ABORT,'injected reorder failure'); END;")?;
    let transaction = connection.transaction()?;
    let lists = SavedListsWrite::from_connection(&transaction);
    assert!(
        lists
            .reorder_item(
                "atomic",
                ReorderSavedListItem {
                    record_key: "actions:third".into(),
                    target_record_key: "actions:first".into(),
                    placement: SavedListItemPlacement::Before,
                }
            )
            .is_err()
    );
    assert_eq!(lists.get_with_items("atomic")?.expect("list"), before);
    transaction.commit()?;
    assert_eq!(
        store.saved_lists().get_with_items("atomic")?.expect("list"),
        before
    );
    Ok(())
}

#[test]
fn list_key_can_identify_saved_lists() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-key"))?;
    let lists = store.saved_lists();
    let list = lists.create(NewSavedList {
        slug: "keyed-list".to_string(),
        name: "Keyed List".to_string(),
        description: None,
        tags: vec![],
    })?;

    lists.add_resolved_item(
        &list.list_key,
        resolved_item("actions:first", None, "First", Some("rule")),
    )?;
    let keyed = lists
        .get_with_items(&list.list_key)?
        .expect("list should load by key");
    assert_eq!(keyed.list.slug, "keyed-list");
    assert_eq!(keyed.items[0].record_key, "actions:first");

    assert!(lists.delete(&list.list_key)?);
    assert!(lists.get("keyed-list")?.is_none());
    Ok(())
}

#[test]
fn update_list_renames_and_edits_description_by_key() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-update"))?;
    let lists = store.saved_lists();
    let list = lists.create(NewSavedList {
        slug: "old-name".to_string(),
        name: "Old Name".to_string(),
        description: Some("Old description".to_string()),
        tags: vec![
            " story ".to_string(),
            "Story".to_string(),
            "finale".to_string(),
        ],
    })?;

    let updated = lists
        .update(UpdateSavedList {
            list_key: list.list_key.clone(),
            slug: "new-name".to_string(),
            name: "New Name".to_string(),
            description: Some("New description".to_string()),
            tags: vec!["updated".to_string()],
        })?
        .expect("list should update");

    assert_eq!(updated.list_key, list.list_key);
    assert_eq!(updated.slug, "new-name");
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.description.as_deref(), Some("New description"));
    assert_eq!(updated.tags, vec!["updated"]);
    assert!(lists.get("old-name")?.is_none());
    assert_eq!(
        lists
            .get("new-name")?
            .expect("new slug should resolve")
            .list_key,
        list.list_key
    );
    Ok(())
}

#[test]
fn update_list_rejects_duplicate_slug() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-update-conflict"))?;
    let lists = store.saved_lists();
    let first = lists.create(NewSavedList {
        slug: "first".to_string(),
        name: "First".to_string(),
        description: None,
        tags: vec![],
    })?;
    lists.create(NewSavedList {
        slug: "second".to_string(),
        name: "Second".to_string(),
        description: None,
        tags: vec![],
    })?;

    let result = lists.update(UpdateSavedList {
        list_key: first.list_key,
        slug: "second".to_string(),
        name: "Conflict".to_string(),
        description: None,
        tags: vec![],
    });
    assert!(matches!(result, Err(LocalStateError::ListAlreadyExists(_))));
    Ok(())
}

#[test]
fn import_creates_saved_list_with_dense_positions() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-import-create"))?;
    let imported = store.saved_lists().import(ImportSavedList {
        slug: "imported-prep".to_string(),
        name: "Imported Prep".to_string(),
        description: Some("From export".to_string()),
        tags: vec![
            "Beat Two".to_string(),
            "beat two".to_string(),
            " ".to_string(),
        ],
        replace: false,
        items: vec![
            imported_item("actions:second", Some("Review"), "Second", Some("rule")),
            imported_item("actions:first", None, "First", Some("rule")),
        ],
    })?;

    assert_eq!(imported.list.slug, "imported-prep");
    assert_eq!(imported.list.name, "Imported Prep");
    assert_eq!(imported.list.tags, vec!["Beat Two"]);
    assert_eq!(imported.items.len(), 2);
    assert_eq!(imported.items[0].record_key, "actions:second");
    assert_eq!(imported.items[0].position, 1);
    assert_eq!(imported.items[0].note.as_deref(), Some("Review"));
    assert_eq!(imported.items[1].record_key, "actions:first");
    assert_eq!(imported.items[1].position, 2);
    Ok(())
}

#[test]
fn import_requires_replace_for_existing_slug() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-import-conflict"))?;
    let lists = store.saved_lists();
    lists.create(NewSavedList {
        slug: "existing".to_string(),
        name: "Existing".to_string(),
        description: None,
        tags: vec![],
    })?;

    let result = lists.import(ImportSavedList {
        slug: "existing".to_string(),
        name: "Imported".to_string(),
        description: None,
        tags: vec![],
        replace: false,
        items: vec![imported_item("actions:first", None, "First", Some("rule"))],
    });

    assert!(matches!(result, Err(LocalStateError::ListAlreadyExists(_))));
    Ok(())
}

#[test]
fn import_replace_updates_metadata_and_items() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-import-replace"))?;
    let lists = store.saved_lists();
    let existing = lists.create(NewSavedList {
        slug: "existing".to_string(),
        name: "Existing".to_string(),
        description: Some("Old".to_string()),
        tags: vec!["old-tag".to_string()],
    })?;
    lists.add_resolved_item(
        "existing",
        resolved_item("actions:old", None, "Old", Some("rule")),
    )?;

    let imported = lists.import(ImportSavedList {
        slug: "existing".to_string(),
        name: "Imported".to_string(),
        description: Some("New".to_string()),
        tags: vec!["new-tag".to_string()],
        replace: true,
        items: vec![imported_item(
            "actions:new",
            Some("Note"),
            "New",
            Some("rule"),
        )],
    })?;

    assert_eq!(imported.list.list_key, existing.list_key);
    assert_eq!(imported.list.name, "Imported");
    assert_eq!(imported.list.description.as_deref(), Some("New"));
    assert_eq!(imported.list.tags, vec!["new-tag"]);
    assert_eq!(imported.items.len(), 1);
    assert_eq!(imported.items[0].record_key, "actions:new");
    assert_eq!(imported.items[0].position, 1);
    assert_eq!(imported.items[0].note.as_deref(), Some("Note"));
    Ok(())
}

#[test]
fn import_rejects_duplicate_record_keys() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-import-duplicates"))?;
    let result = store.saved_lists().import(ImportSavedList {
        slug: "duplicates".to_string(),
        name: "Duplicates".to_string(),
        description: None,
        tags: vec![],
        replace: false,
        items: vec![
            imported_item("actions:first", None, "First", Some("rule")),
            imported_item("actions:first", Some("Again"), "First Again", Some("rule")),
        ],
    });

    assert!(matches!(result, Err(LocalStateError::InvalidListImport(_))));
    assert!(store.saved_lists().get("duplicates")?.is_none());
    Ok(())
}

#[test]
fn remove_item_compacts_positions() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-remove"))?;
    let lists = store.saved_lists();
    lists.create(NewSavedList {
        slug: "test-list".to_string(),
        name: "Test List".to_string(),
        description: None,
        tags: vec![],
    })?;
    for key in ["actions:first", "actions:second", "actions:third"] {
        lists.add_resolved_item("test-list", resolved_item(key, None, key, None))?;
    }

    assert!(lists.remove_item("test-list", "actions:second")?);
    let list = lists
        .get_with_items("test-list")?
        .expect("list should exist");
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].record_key, "actions:first");
    assert_eq!(list.items[0].position, 1);
    assert_eq!(list.items[1].record_key, "actions:third");
    assert_eq!(list.items[1].position, 2);
    Ok(())
}

#[test]
fn invalid_slugs_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-slug"))?;
    let result = store.saved_lists().create(NewSavedList {
        slug: "Bad Slug".to_string(),
        name: "Bad".to_string(),
        description: None,
        tags: vec![],
    });
    assert!(matches!(result, Err(LocalStateError::InvalidSlug { .. })));
    Ok(())
}

#[test]
fn invalid_record_keys_are_rejected_on_remove() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("saved-lists-record-key"))?;
    let lists = store.saved_lists();
    lists.create(NewSavedList {
        slug: "test-list".to_string(),
        name: "Test List".to_string(),
        description: None,
        tags: vec![],
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
    assert_eq!(active.record, Some("hydrated record"));
    assert_eq!(active.record_key, "actions:first");
    assert_eq!(active.position, 1);
    assert_eq!(active.note.as_deref(), Some("note"));
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
    assert_eq!(unresolved.record, None);
    assert_eq!(unresolved.record_key, "actions:missing");
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

fn imported_item(
    record_key: &str,
    note: Option<&str>,
    title_snapshot: &str,
    kind_snapshot: Option<&str>,
) -> ImportSavedListItem {
    ImportSavedListItem {
        record_key: record_key.to_string(),
        note: note.map(str::to_string),
        record_title_snapshot: title_snapshot.to_string(),
        record_kind_snapshot: kind_snapshot.map(str::to_string),
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
