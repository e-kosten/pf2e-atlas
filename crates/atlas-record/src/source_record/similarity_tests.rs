//! Policy checks start with admitted source, not stored role labels.
use super::*;
use crate::source_content::{ContentAudience, ContentVisibilityRule};
use atlas_domain::SimilarityRole;
use atlas_foundry_model::{SourceContext, admit_document_source};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn source(kind: &str, value: Value) -> SourceBackedRecord {
    let bytes = serde_json::to_vec(&value).unwrap();
    let admitted = admit_document_source(
        kind,
        SourceContext::new("fixture", "similarity", "$"),
        &bytes,
    )
    .unwrap()
    .model
    .unwrap();
    SourceBackedRecord::new("test", admitted).unwrap()
}

fn audience(private: bool) -> ContentAudience {
    ContentAudience {
        include_gm: private,
        include_owner: private,
        implicit_check_dc: ContentVisibilityRule::Gm,
    }
}

fn roles(record: &SourceBackedRecord, private: bool) -> Vec<(usize, String, SimilarityRole)> {
    let content = prepare_record_content(record, audience(private), None, None);
    let selected = select_record_search(
        record,
        &content,
        audience(private),
        "Test",
        &BTreeMap::new(),
    )
    .unwrap();
    selected
        .fields
        .iter()
        .map(|field| {
            (
                field.locator.owners.len(),
                field.locator.field.clone(),
                similarity_content_role(record, &field.locator, field.role)
                    .expect("selected owner must resolve"),
            )
        })
        .collect()
}

#[test]
fn root_and_embedded_item_descriptions_use_typed_family_precedence() {
    for (family, expected) in [
        ("action", SimilarityRole::Ability),
        ("feat", SimilarityRole::Ability),
        ("effect", SimilarityRole::Ability),
        ("spell", SimilarityRole::Description),
        ("equipment", SimilarityRole::Description),
        ("weapon", SimilarityRole::Description),
        ("armor", SimilarityRole::Description),
        ("consumable", SimilarityRole::Description),
        ("ancestry", SimilarityRole::Description),
        ("class", SimilarityRole::Description),
        ("deity", SimilarityRole::Description),
        ("kit", SimilarityRole::Description),
    ] {
        let item = json!({"_id":"bbbbbbbbbbbbbbbb", "name":"Shared body", "type":family,
            "system":{"description":{"value":"<p>Identical text across families.</p>"}}});
        let root = source("Item", item.clone());
        assert_eq!(
            roles(&root, false),
            [(0, "/system/description/value".into(), expected)],
            "root {family}"
        );
        let embedded = source(
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa", "type":"npc", "items":[item]}),
        );
        assert_eq!(
            roles(&embedded, false),
            [(1, "/system/description/value".into(), expected)],
            "embedded {family}"
        );
    }
}

#[test]
fn public_private_and_empty_source_fields_are_selected_before_role_mapping() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa", "type":"npc", "system":{"details":{
            "publicNotes":"<p>Public description</p>", "privateNotes":"<p>GM notes</p>", "blurb":" "
        }},"items":[{"_id":"bbbbbbbbbbbbbbbb", "type":"action", "system":{"description":{
            "value":"<p></p>", "gm":"<p>Private ability</p>"
        }}}]}),
    );
    assert_eq!(
        roles(&record, false),
        [(
            0,
            "/system/details/publicNotes".into(),
            SimilarityRole::Description
        )]
    );
    let all = roles(&record, true);
    assert_eq!(all.len(), 3);
    assert!(all.contains(&(
        0,
        "/system/details/privateNotes".into(),
        SimilarityRole::Notes
    )));
    assert!(all.contains(&(1, "/system/description/gm".into(), SimilarityRole::Ability)));
    assert!(
        !all.iter()
            .any(|(_, field, _)| field.ends_with("/blurb") || field.ends_with("/value"))
    );
}

#[test]
fn biography_roles_follow_typed_purpose_and_independent_visibility_flags() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa", "type":"character", "system":{"details":{"biography":{
            "appearance":"<p>Appearance</p>", "backstory":"<p>Backstory</p>",
            "allies":"<p>Allies</p>", "campaignNotes":"<p>Campaign notes</p>", "attitude":"Friendly",
            "visibility":{"appearance":true, "backstory":false, "campaign":false, "personality":true}
        }}}}),
    );
    let visible = roles(&record, false);
    assert!(visible.contains(&(
        0,
        "/system/details/biography/appearance".into(),
        SimilarityRole::Description
    )));
    assert!(visible.contains(&(
        0,
        "/system/details/biography/attitude".into(),
        SimilarityRole::Notes
    )));
    assert!(
        !visible
            .iter()
            .any(|(_, field, _)| field.ends_with("/backstory")
                || field.ends_with("/allies")
                || field.ends_with("/campaignNotes"))
    );
    let all = roles(&record, true);
    for (field, expected) in [
        ("backstory", SimilarityRole::Description),
        ("allies", SimilarityRole::Notes),
        ("campaignNotes", SimilarityRole::Notes),
    ] {
        assert!(
            all.contains(&(0, format!("/system/details/biography/{field}"), expected)),
            "{field}"
        );
    }
    let unknown_visibility = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa", "type":"character", "system":{"details":{"biography":{
            "appearance":"<p>Unclassified visibility</p>"
        }}}}),
    );
    assert!(roles(&unknown_visibility, true).is_empty());
}

#[test]
fn journal_pages_and_table_results_keep_distinct_non_description_roles() {
    let journal = source(
        "JournalEntry",
        json!({"_id":"aaaaaaaaaaaaaaaa", "name":"Journal", "pages":[
            {"_id":"bbbbbbbbbbbbbbbb", "name":"Entry", "type":"text", "text":{"format":1,"content":"<p>Body</p>"}, "image":{"caption":"Diagram caption"}},
            {"_id":"cccccccccccccccc", "name":"Empty", "type":"text", "text":{"format":1,"content":"<p></p>"}}
        ]}),
    );
    let journal_roles = roles(&journal, false);
    assert_eq!(journal_roles.len(), 2);
    assert!(journal_roles.contains(&(1, "/text/content".into(), SimilarityRole::JournalText)));
    assert!(journal_roles.contains(&(1, "/image/caption".into(), SimilarityRole::Other)));
    let table = source(
        "RollTable",
        json!({"_id":"aaaaaaaaaaaaaaaa", "results":[
            {"_id":"bbbbbbbbbbbbbbbb", "type":"text", "description":"<p>Result prose</p>"}
        ]}),
    );
    assert_eq!(
        roles(&table, false),
        [(1, "/description".into(), SimilarityRole::TableResult)]
    );
}

#[test]
fn hazard_mechanics_and_feat_prerequisites_keep_their_specific_roles() {
    let hazard = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa", "type":"hazard", "system":{
            "details":{"description":"<p>Body</p>", "routine":"<p>Routine</p>", "disable":"<p>Disable</p>", "reset":"<p>Reset</p>"},
            "attributes":{"stealth":{"details":"(trained)"}}
        }}),
    );
    let mapped = roles(&hazard, false);
    assert_eq!(mapped.len(), 5);
    for (field, expected) in [
        ("description", SimilarityRole::Description),
        ("routine", SimilarityRole::Routine),
        ("disable", SimilarityRole::Disable),
        ("reset", SimilarityRole::Reset),
    ] {
        assert!(mapped.contains(&(0, format!("/system/details/{field}"), expected)));
    }
    assert!(mapped.contains(&(
        0,
        "/system/attributes/stealth/details".into(),
        SimilarityRole::Stealth
    )));
    let feat = source(
        "Item",
        json!({"_id":"aaaaaaaaaaaaaaaa", "type":"feat", "system":{
            "description":{"value":"<p>Feat body</p>"}, "prerequisites":{"value":[{"value":"a feat prerequisite"}]}
        }}),
    );
    assert_eq!(
        roles(&feat, false)
            .iter()
            .map(|(_, _, role)| *role)
            .collect::<Vec<_>>(),
        [SimilarityRole::Ability, SimilarityRole::Prerequisites]
    );
}
