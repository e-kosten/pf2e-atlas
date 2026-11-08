use atlas_app_model::{
    AddSavedListItemRequest, AppErrorCode, CreateEncounterRequest, CreateSavedListRequest,
    EncounterStatusView, FieldEdit, RemoveSavedListItemRequest, ReorderSavedListItemPlacementView,
    ReorderSavedListItemRequest, UpdateEncounterRequest, UpdateSavedListItemNoteRequest,
    UpdateSavedListRequest,
};

use crate::AtlasAppService;
use crate::test_support::fixture_worker;

fn edit<T>(expected: T, value: T) -> FieldEdit<T> {
    FieldEdit { expected, value }
}

fn list_fixture(worker: &AtlasAppService) -> String {
    worker
        .create_saved_list(CreateSavedListRequest {
            slug: "planning".into(),
            name: "Planning".into(),
            description: Some("Existing description".into()),
            tags: vec!["Next".into(), "Élite".into()],
        })
        .unwrap()
        .list
        .list_key
}

const FIRST: &str = "actions:testAction100000";
const SECOND: &str = "actions:testAction200000";

fn add_note(worker: &AtlasAppService, list_ref: &str, record_ref: &str, note: Option<&str>) {
    worker
        .add_saved_list_item(AddSavedListItemRequest {
            list_ref: list_ref.into(),
            record_ref: record_ref.into(),
            note: note.map(str::to_owned),
        })
        .unwrap();
}

fn note_patch(
    list_ref: &str,
    record_key: &str,
    expected: Option<&str>,
    value: Option<&str>,
) -> UpdateSavedListItemNoteRequest {
    UpdateSavedListItemNoteRequest {
        list_ref: list_ref.into(),
        record_key: record_key.into(),
        note: edit(expected.map(str::to_owned), value.map(str::to_owned)),
    }
}

#[test]
fn list_note_save_merges_with_reorder_and_unrelated_edits() {
    let worker = fixture_worker().worker;
    let key = list_fixture(&worker);
    add_note(&worker, &key, FIRST, Some("Original"));
    add_note(&worker, &key, SECOND, None);
    let draft = note_patch(&key, FIRST, Some("Original"), Some("First draft"));

    worker
        .reorder_saved_list_item(ReorderSavedListItemRequest {
            list_ref: key.clone(),
            record_ref: SECOND.into(),
            target_record_ref: FIRST.into(),
            placement: ReorderSavedListItemPlacementView::Before,
        })
        .unwrap();
    worker
        .update_saved_list_item_note(note_patch(&key, SECOND, None, Some("Second note")))
        .unwrap();
    worker
        .edit_saved_list_metadata(&key, Some("renamed".into()), None, None, None)
        .unwrap();
    let saved = worker.update_saved_list_item_note(draft).unwrap();
    assert_eq!(saved.slug, "renamed");
    let current = worker.saved_list_snapshot(&key).unwrap();
    assert_eq!(
        current
            .items
            .iter()
            .map(|item| (item.record_key.as_str(), item.note.as_deref()))
            .collect::<Vec<_>>(),
        [(SECOND, Some("Second note")), (FIRST, Some("First draft"))]
    );
    worker
        .reorder_saved_list_item(ReorderSavedListItemRequest {
            list_ref: key.clone(),
            record_ref: FIRST.into(),
            target_record_ref: SECOND.into(),
            placement: ReorderSavedListItemPlacementView::Before,
        })
        .unwrap();
    let reordered = worker.saved_list_snapshot(&key).unwrap();
    assert_eq!(reordered.items[0].note.as_deref(), Some("First draft"));
}

#[test]
fn list_note_conflict_and_missing_entry_do_not_mutate_saved_state() {
    let worker = fixture_worker().worker;
    let key = list_fixture(&worker);
    add_note(&worker, &key, FIRST, None);
    worker
        .update_saved_list_item_note(note_patch(&key, FIRST, None, Some("Winning note")))
        .unwrap();
    let stale = note_patch(&key, FIRST, None, Some("Winning note"));
    let error = worker
        .update_saved_list_item_note(stale.clone())
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::SavedEditConflict);
    assert_eq!(
        error.details.unwrap()["fields"],
        serde_json::json!(["note"])
    );
    let remove = || {
        worker
            .remove_saved_list_item(RemoveSavedListItemRequest {
                list_ref: key.clone(),
                record_ref: FIRST.into(),
            })
            .unwrap();
    };
    remove();
    for error in [
        worker.saved_list_item_note(&key, FIRST).unwrap_err(),
        worker
            .update_saved_list_item_note(stale.clone())
            .unwrap_err(),
    ] {
        assert_eq!(
            error.into_app_error().code,
            AppErrorCode::SavedListItemNotFound
        );
    }
    assert!(worker.saved_list_snapshot(&key).unwrap().items.is_empty());
    add_note(&worker, &key, FIRST, Some("Re-added note"));
    assert_eq!(
        worker
            .update_saved_list_item_note(stale.clone())
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::SavedEditConflict,
    );
    assert_eq!(
        worker
            .saved_list_item_note(&key, FIRST)
            .unwrap()
            .note
            .as_deref(),
        Some("Re-added note")
    );
    remove();
    add_note(&worker, &key, FIRST, None);
    assert_eq!(
        worker
            .update_saved_list_item_note(stale)
            .unwrap()
            .note
            .as_deref(),
        Some("Winning note"),
    );
}

#[test]
fn list_metadata_saves_merge_disjoint_fields_and_reject_partial_conflicts() {
    let worker = fixture_worker().worker;
    let key = list_fixture(&worker);
    let request = || UpdateSavedListRequest {
        list_key: key.clone(),
        slug: None,
        name: None,
        description: None,
        tags: None,
    };
    worker
        .update_saved_list(UpdateSavedListRequest {
            description: Some(edit(Some("Existing description".into()), None)),
            ..request()
        })
        .unwrap();
    let current = worker
        .update_saved_list(UpdateSavedListRequest {
            name: Some(edit("Planning".into(), "Planning session".into())),
            ..request()
        })
        .unwrap()
        .list;
    assert_eq!(current.description, None);

    let saved = || {
        worker
            .with_state(|session| Ok(session.local_state()?.saved_lists().get(&key)?.unwrap()))
            .unwrap()
    };
    let before_conflict = saved();
    let error = worker
        .update_saved_list(UpdateSavedListRequest {
            name: Some(edit("Planning".into(), "Other name".into())),
            tags: Some(edit(current.tags.clone(), vec![])),
            ..request()
        })
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::SavedEditConflict);
    assert_eq!(
        error.details.unwrap()["fields"],
        serde_json::json!(["name"])
    );
    assert_eq!(saved(), before_conflict);

    worker
        .update_saved_list(UpdateSavedListRequest {
            tags: Some(edit(
                current.tags,
                vec![" Élite ".into(), "Next".into(), "next".into()],
            )),
            ..request()
        })
        .unwrap();
    worker.update_saved_list(request()).unwrap();
    assert_eq!(saved(), before_conflict);
}

#[test]
fn encounter_metadata_patches_preserve_other_fields_and_validate_atomically() {
    let worker = fixture_worker().worker;
    let original = worker
        .create_encounter(CreateEncounterRequest {
            name: "Bridge patrol".into(),
            description: Some("Bridge".into()),
            note: Some("Keep this note".into()),
        })
        .unwrap()
        .encounter;
    let request = || UpdateEncounterRequest {
        encounter_key: original.encounter_key.clone(),
        round_duration_seconds: None,
        slug: None,
        name: None,
        description: None,
        note: None,
        status: None,
    };
    worker
        .update_encounter(UpdateEncounterRequest {
            status: Some(edit(original.status, EncounterStatusView::Running)),
            ..request()
        })
        .unwrap();
    worker
        .update_encounter(UpdateEncounterRequest {
            name: Some(edit(original.name.clone(), "Forest patrol".into())),
            note: Some(edit(Some("Keep this note".into()), None)),
            ..request()
        })
        .unwrap();

    let saved = || {
        worker
            .with_state(|session| {
                Ok(session
                    .local_state()?
                    .encounters()
                    .get(&original.encounter_key)?
                    .unwrap())
            })
            .unwrap()
    };
    let before_conflict = saved();
    assert_eq!(
        before_conflict.status,
        atlas_local_state::EncounterStatus::Running
    );
    assert_eq!(before_conflict.note, None);
    let error = worker
        .update_encounter(UpdateEncounterRequest {
            status: Some(edit(original.status, EncounterStatusView::Archived)),
            description: Some(edit(Some("Bridge".into()), None)),
            ..request()
        })
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::SavedEditConflict);
    assert_eq!(
        error.details.unwrap()["fields"],
        serde_json::json!(["status"])
    );
    assert_eq!(saved(), before_conflict);

    let error = worker
        .update_encounter(UpdateEncounterRequest {
            description: Some(edit(Some("Bridge".into()), None)),
            round_duration_seconds: Some(edit(original.round_duration_seconds, 0)),
            ..request()
        })
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::InvalidRequest);
    assert_eq!(saved(), before_conflict);
    worker.update_encounter(request()).unwrap();
    assert_eq!(saved(), before_conflict);
}
