//! Typed policy fixtures; these prove membership, not semantic relevance.
use super::*;
use crate::source_content::{ContentAudience, ContentVisibilityRule};
use atlas_domain::{CoreFallbackReason, CoreSectionFamily, OwnedContentIdentity};
use atlas_foundry_model::{SourceContext, admit_document_source};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn source(kind: &str, value: Value) -> SourceBackedRecord {
    let admitted = admit_document_source(
        kind,
        SourceContext::new("fixture", "core-sections", "$"),
        &serde_json::to_vec(&value).unwrap(),
    )
    .unwrap()
    .model
    .unwrap();
    SourceBackedRecord::new("test", admitted).unwrap()
}

fn selection(record: &SourceBackedRecord, private: bool) -> SourceSearchSelection {
    let audience = ContentAudience {
        include_gm: private,
        include_owner: private,
        implicit_check_dc: ContentVisibilityRule::Gm,
    };
    let prepared = prepare_record_content(record, audience, None, None);
    select_record_search(record, &prepared, audience, "Test", &BTreeMap::new()).unwrap()
}

fn core(record: &SourceBackedRecord, private: bool) -> CoreSearchSelection {
    core_search_selection(record, &selection(record, private)).unwrap()
}

fn owned_id(owners: &[atlas_domain::OwnedContentLocator]) -> &str {
    match owners {
        [] => "root",
        [owner] => match &owner.identity {
            OwnedContentIdentity::Stable(id) => id,
            OwnedContentIdentity::SnapshotLocal { .. } => "snapshot-local",
        },
        _ => "nested",
    }
}

fn field_addresses(selected: &CoreSearchSelection) -> Vec<(&str, &str)> {
    selected
        .fields
        .iter()
        .map(|field| (owned_id(&field.owners), field.field.as_str()))
        .collect()
}

#[test]
fn reviewed_item_families_use_all_main_description_sections_and_exclude_gm_notes() {
    use CoreSectionFamily as F;
    for (family, expected) in [
        ("spell", F::Spell),
        ("action", F::Rule),
        ("feat", F::Rule),
        ("effect", F::Rule),
        ("condition", F::Rule),
        ("ammo", F::Equipment),
        ("armor", F::Equipment),
        ("book", F::Equipment),
        ("consumable", F::Equipment),
        ("backpack", F::Equipment),
        ("equipment", F::Equipment),
        ("kit", F::Equipment),
        ("shield", F::Equipment),
        ("treasure", F::Equipment),
        ("weapon", F::Equipment),
        ("ancestry", F::CharacterOption),
        ("heritage", F::CharacterOption),
        ("background", F::CharacterOption),
        ("class", F::CharacterOption),
        ("deity", F::Deity),
        ("campaignFeature", F::CampaignFeature),
    ] {
        let record = source(
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Same name","type":family,
                "system":{"description":{
                    "value":"<h2>First</h2><p>First body.</p><h2>Second</h2><p>Second body.</p>",
                    "gm":"<p>Private body.</p>"
                },"prerequisites":{"value":[{"value":"Excluded requirement"}]}}}),
        );
        let all = selection(&record, true);
        let result = core_search_selection(&record, &all).unwrap();
        assert_eq!(result.family, expected, "{family}");
        assert_eq!(
            field_addresses(&result),
            [("root", "/system/description/value")],
            "{family}"
        );
        let field = all
            .fields
            .iter()
            .find(|field| field.locator == result.fields[0])
            .unwrap();
        assert_eq!(field.sections.len(), 2, "{family}: no first-section cutoff");
        assert_eq!(result.fallback_reason(), None, "{family}");
    }
}

#[test]
fn unreviewed_families_do_not_guess_descriptions_pages_or_table_results() {
    let mut records = vec![
        source(
            "JournalEntry",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Journal","pages":[
                {"_id":"bbbbbbbbbbbbbbbb","name":"First page","type":"text", "text":{"format":1,"content":"<p>Body</p>"}}
            ]}),
        ),
        source(
            "RollTable",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Table","description":"<p>Body</p>", "results":[
                {"_id":"bbbbbbbbbbbbbbbb","text":"<p>Result</p>","type":0}
            ]}),
        ),
    ];
    for family in ["affliction", "lore", "melee", "spellcastingEntry"] {
        records.push(source(
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Item","type":family,
                "system":{"description":{"value":"<p>Body</p>"}}}),
        ));
    }
    for family in ["loot", "party"] {
        records.push(source(
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Actor","type":family,
                "system":{"details":{"description":"<p>Body</p>"}}}),
        ));
    }
    for record in records {
        let result = core(&record, true);
        assert_eq!(result.family, CoreSectionFamily::Unmodelled);
        assert!(result.fields.is_empty());
        assert_eq!(result.owners, [vec![]]);
        assert_eq!(
            result.fallback_reason(),
            Some(CoreFallbackReason::UnmodelledKind)
        );
    }
}

#[test]
fn creature_core_uses_public_notes_and_blurb_without_abilities_or_private_notes() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Creature","type":"npc", "system":{"details":{
            "publicNotes":"<p>Description</p>", "privateNotes":"<p>Private description</p>", "blurb":"Short description"
        }},"items":[{"_id":"bbbbbbbbbbbbbbbb","name":"Ability","type":"action",
            "system":{"description":{"value":"<p>Ability description</p>"}}}]}),
    );
    let result = core(&record, true);
    assert_eq!(result.family, CoreSectionFamily::Npc);
    assert_eq!(
        field_addresses(&result),
        [
            ("root", "/system/details/blurb"),
            ("root", "/system/details/publicNotes"),
        ]
    );
    assert_eq!(result.owners, [vec![]]);
}

#[test]
fn hazard_routine_and_direct_actions_qualify_without_complexity_or_content_heuristics() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Hazard","type":"hazard",
        "system":{"details":{
            "description":"<p>Description</p>", "routine":"<p>Routine</p>",
            "disable":"<p>Disable</p>","reset":"<p>Reset</p>","isComplex":false
        },"attributes":{"stealth":{"details":"<p>Stealth</p>"}}},
        "items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Reaction","type":"action","system":{"description":{"value":"<p>Same body</p>"}}},
            {"_id":"cccccccccccccccc","name":"Strike","type":"melee","system":{"description":{"value":"<p>Same body</p>"}}},
            {"_id":"dddddddddddddddd","name":"Carried item","type":"equipment","system":{"description":{"value":"<p>Same body</p>"}}}
        ]}),
    );
    let result = core(&record, true);
    assert_eq!(
        field_addresses(&result),
        [
            ("root", "/system/details/description"),
            ("root", "/system/details/routine"),
            ("bbbbbbbbbbbbbbbb", "/system/description/value"),
            ("cccccccccccccccc", "/system/description/value"),
        ]
    );
    assert_eq!(
        result
            .owners
            .iter()
            .map(|owners| owned_id(owners))
            .collect::<Vec<_>>(),
        ["root", "bbbbbbbbbbbbbbbb", "cccccccccccccccc"]
    );
}

#[test]
fn vehicle_operational_actions_qualify_without_root_prose_and_empty_action_names_remain() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Vehicle","type":"vehicle", "items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Operation","type":"action","system":{"description":{"value":"<p>Handling operation</p>"}}},
            {"_id":"cccccccccccccccc","name":"Empty operation","type":"action","system":{"description":{"value":"<p></p>"}}},
            {"_id":"dddddddddddddddd","name":"Temporary effect","type":"effect","system":{"description":{"value":"<p>Effect</p>"}}},
            {"_id":"eeeeeeeeeeeeeeee","name":"Cargo","type":"equipment","system":{"description":{"value":"<p>Cargo</p>"}}}
        ]}),
    );
    let result = core(&record, true);
    assert_eq!(result.family, CoreSectionFamily::Vehicle);
    assert_eq!(
        field_addresses(&result),
        [("bbbbbbbbbbbbbbbb", "/system/description/value")]
    );
    assert_eq!(
        result
            .owners
            .iter()
            .map(|owners| owned_id(owners))
            .collect::<Vec<_>>(),
        ["root", "bbbbbbbbbbbbbbbb", "cccccccccccccccc"]
    );
}

#[test]
fn army_owned_membership_uses_typed_categories_without_name_or_role_leakage() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Army","type":"army", "system":{"details":{"description":"<p>Army</p>"}},"items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Shared name","type":"campaignFeature","system":{"category":"army-tactic","description":{"value":"<p>Same body</p>"}}},
            {"_id":"cccccccccccccccc","name":"Shared name","type":"campaignFeature","system":{"category":"army-war-action","description":{"value":"<p>Same body</p>"}}},
            {"_id":"dddddddddddddddd","name":"Army tactic","type":"campaignFeature","system":{"category":"kingdom-activity","description":{"value":"<p>Same body</p>"}}},
            {"_id":"eeeeeeeeeeeeeeee","name":"Army tactic","type":"campaignFeature","system":{"description":{"value":"<p>Same body</p>"}}},
            {"_id":"ffffffffffffffff","name":"Army tactic","type":"effect","system":{"description":{"value":"<p>Same body</p>"}}}
        ]}),
    );
    let result = core(&record, true);
    assert_eq!(result.family, CoreSectionFamily::Army);
    assert_eq!(
        field_addresses(&result),
        [
            ("root", "/system/details/description"),
            ("bbbbbbbbbbbbbbbb", "/system/description/value"),
            ("cccccccccccccccc", "/system/description/value"),
        ]
    );
    assert_eq!(
        result
            .owners
            .iter()
            .map(|owners| owned_id(owners))
            .collect::<Vec<_>>(),
        ["root", "bbbbbbbbbbbbbbbb", "cccccccccccccccc"]
    );
}

#[test]
fn character_core_respects_biography_visibility_and_only_foundational_owned_documents() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"PC","type":"character", "system":{"details":{"biography":{
            "appearance":"<p>Appearance</p>", "backstory":"<p>Private backstory</p>",
            "campaignNotes":"<p>Campaign diary</p>","allies":"<p>Allies</p>","attitude":"Friendly",
            "visibility":{"appearance":true,"backstory":false,"campaign":false,"personality":true}
        }}},"items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Ancestry","type":"ancestry","system":{"description":{"value":"<p>Main description</p>","gm":"<p>Private additional</p>"}}},
            {"_id":"cccccccccccccccc","name":"Heritage","type":"heritage","system":{"description":{"value":"<p>Main description</p>"}}},
            {"_id":"dddddddddddddddd","name":"Background","type":"background","system":{"description":{"value":"<p>Main description</p>"}}},
            {"_id":"eeeeeeeeeeeeeeee","name":"Class with empty description","type":"class","system":{"description":{"value":"<p></p>"}}},
            {"_id":"ffffffffffffffff","name":"Feat","type":"feat","system":{"description":{"value":"<p>Feat</p>"}}},
            {"_id":"gggggggggggggggg","name":"Spell","type":"spell","system":{"description":{"value":"<p>Spell</p>"}}},
            {"_id":"hhhhhhhhhhhhhhhh","name":"Item","type":"equipment","system":{"description":{"value":"<p>Item</p>"}}}
        ]}),
    );
    let public = core(&record, false);
    assert_eq!(public.family, CoreSectionFamily::Character);
    assert_eq!(
        field_addresses(&public),
        [
            ("root", "/system/details/biography/appearance"),
            ("bbbbbbbbbbbbbbbb", "/system/description/value"),
            ("cccccccccccccccc", "/system/description/value"),
            ("dddddddddddddddd", "/system/description/value"),
        ]
    );
    assert_eq!(
        public
            .owners
            .iter()
            .map(|owners| owned_id(owners))
            .collect::<Vec<_>>(),
        [
            "root",
            "bbbbbbbbbbbbbbbb",
            "cccccccccccccccc",
            "dddddddddddddddd",
            "eeeeeeeeeeeeeeee"
        ]
    );
    let private = core(&record, true);
    let private_root_fields = field_addresses(&private)
        .into_iter()
        .filter(|(owner, _)| *owner == "root")
        .collect::<Vec<_>>();
    assert_eq!(
        private_root_fields,
        [
            ("root", "/system/details/biography/appearance"),
            ("root", "/system/details/biography/backstory"),
        ]
    );
}

#[test]
fn unavailable_character_visibility_never_becomes_permission_for_core_prose() {
    for visibility in [
        json!({}),
        json!({"appearance":null}),
        json!({"appearance":"yes"}),
    ] {
        let record = source(
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"PC","type":"character","system":{"details":{"biography":{
                "appearance":"<p>Unclassified</p>","visibility":visibility
            }}}}),
        );
        for private in [false, true] {
            let result = core(&record, private);
            assert!(result.fields.is_empty());
            assert_eq!(
                result.fallback_reason(),
                Some(CoreFallbackReason::NoCoreProse)
            );
        }
    }
}

#[test]
fn familiar_core_uses_only_familiar_category_abilities_without_master_or_form_expansion() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Familiar","type":"familiar",
            "system":{"details":{"creature":{"value":"Bat"}}},"items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Shared name","type":"action","system":{"category":"familiar","description":{"value":"<p>Selected familiar ability</p>"}}},
            {"_id":"cccccccccccccccc","name":"Familiar ability","type":"action","system":{"category":"offensive","description":{"value":"<p>Same body</p>"}}},
            {"_id":"dddddddddddddddd","name":"Familiar ability","type":"action","system":{"description":{"value":"<p>Same body</p>"}}},
            {"_id":"eeeeeeeeeeeeeeee","name":"Effect","type":"effect","system":{"description":{"value":"<p>Same body</p>"}}}
        ]}),
    );
    let result = core(&record, true);
    assert_eq!(result.family, CoreSectionFamily::Familiar);
    assert_eq!(
        field_addresses(&result),
        [("bbbbbbbbbbbbbbbb", "/system/description/value")]
    );
    assert_eq!(
        result
            .owners
            .iter()
            .map(|owners| owned_id(owners))
            .collect::<Vec<_>>(),
        ["root", "bbbbbbbbbbbbbbbb"]
    );
}

#[test]
fn own_equipment_core_excludes_intrinsic_spells_and_carried_equipment_stays_noncore() {
    let wand = json!({"_id":"bbbbbbbbbbbbbbbb","name":"Wand","type":"consumable", "system":{
        "description":{"value":"<p>Wand description</p>"},
        "spell":{"_id":"cccccccccccccccc","name":"Fireball","type":"spell","system":{"description":{"value":"<p>Spell description</p>"}}}
    }});
    let record = source("Item", wand.clone());
    let all = selection(&record, false);
    assert!(
        all.fields
            .iter()
            .any(|field| !field.locator.owners.is_empty())
    );
    assert_eq!(
        field_addresses(&core_search_selection(&record, &all).unwrap()),
        [("root", "/system/description/value")]
    );
    let owner = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Creature","type":"npc",
            "system":{"details":{"publicNotes":"<p>Creature description</p>"}}, "items":[wand]}),
    );
    let result = core(&owner, false);
    assert_eq!(
        field_addresses(&result),
        [("root", "/system/details/publicNotes")]
    );
    assert_eq!(result.owners, [vec![]]);
}

#[test]
fn empty_modelled_prose_has_a_distinct_reason_and_retains_permitted_empty_owner_names() {
    let record = source(
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Hazard","type":"hazard", "items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Empty reaction","type":"action","system":{"description":{"value":"<p></p>"}}}
        ]}),
    );
    let result = core(&record, false);
    assert!(result.fields.is_empty());
    assert_eq!(
        result.fallback_reason(),
        Some(CoreFallbackReason::NoCoreProse)
    );
    assert_eq!(
        result
            .owners
            .iter()
            .map(|owners| owned_id(owners))
            .collect::<Vec<_>>(),
        ["root", "bbbbbbbbbbbbbbbb"]
    );
}

#[test]
fn foreign_roots_and_foreign_field_addresses_are_rejected() {
    let record = source(
        "Item",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Spell","type":"spell", "system":{"description":{"value":"<p>Body</p>"}}}),
    );
    let mut selected = selection(&record, false);
    selected.root = "other:bbbbbbbbbbbbbbbb".parse().unwrap();
    assert!(core_search_selection(&record, &selected).is_err());
    selected.root = record.key().clone();
    selected.fields[0].locator.record = "other:bbbbbbbbbbbbbbbb".parse().unwrap();
    assert!(core_search_selection(&record, &selected).is_err());
}
