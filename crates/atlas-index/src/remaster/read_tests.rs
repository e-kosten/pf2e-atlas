use super::*;
use crate::foundation_tests::{fixture, record};
use atlas_domain::OwnedContentIdentity;
use atlas_record::source_record::{
    SourceReferenceIndex, prepare_record_content, resolve_source_relationships,
    select_record_search,
};
use serde_json::json;

fn add_journal(input: &mut crate::IndexBuildInput, pages: serde_json::Value) {
    let mut journal = record(
        "journals",
        "JournalEntry",
        json!({
            "_id":"eeeeeeeeeeeeeeee","name":"Remaster Changes","pages":pages,
        }),
    );
    let mut references = SourceReferenceIndex::default();
    for source in &input.records {
        references.insert_source(source.record.key(), source.record.source());
    }
    references.insert_source(journal.record.key(), journal.record.source());
    journal.content = prepare_record_content(
        &journal.record,
        input.context.audience,
        None,
        Some(&references),
    );
    journal.relationships = resolve_source_relationships(&journal.record, Some(&references));
    input.packs.push(crate::IndexBuildPack {
        pack_id: "journals".into(),
        label: "Journals".into(),
        document_kind: "JournalEntry".into(),
        declared_path: "packs/journals".into(),
    });
    input.context.source_file_count += 1;
    let context_hash = input.context.preparation_sha256().unwrap();
    for source in &mut input.records {
        source.preparation_context_hash = context_hash.clone();
    }
    journal.preparation_context_hash = context_hash;
    let selection = select_record_search(
        &journal.record,
        &journal.content,
        input.context.audience,
        "Journals",
        &input.context.used_trait_labels,
    )
    .unwrap();
    input
        .lexical_units
        .extend(crate::project_lexical_units(&selection, &[]));
    input.records.push(journal);
    input.inventory = crate::CompleteSourceInventory::checked(
        &input.context.source_fingerprint,
        input.packs.iter().map(|pack| pack.pack_id.clone()),
        input.packs.iter().map(|pack| pack.pack_id.clone()),
    )
    .unwrap();
}

#[test]
fn alias_batch_hydrates_shared_declarations_fields_and_capture_once() {
    let mut input = fixture(false);
    for (ordinal, name) in ["Former Hammer", "Former Staff"].into_iter().enumerate() {
        let source = record(
            "items",
            "Item",
            json!({"_id":format!("{ordinal:016}"),"name":name,"type":"weapon","system":{"description":{"value":""},"traits":{"value":[]}}}),
        );
        let selection = select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            "Items",
            &input.context.used_trait_labels,
        )
        .unwrap();
        input
            .lexical_units
            .extend(crate::project_lexical_units(&selection, &[]));
        input.records.push(source);
        input.context.source_file_count += 1;
    }
    add_journal(
        &mut input,
        json!([{
            "_id":"ffffffffffffffff","name":"Equipment","type":"text","text":{"format":1,
            "content":"<ul><li>Former Hammer and Former Staff are merged into @UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]{Dragon Staff}.</li></ul>"}
        }]),
    );
    // Two captured declarations share one file; the journal declaration above
    // independently supports both aliases, sharing one checked field as well.
    let entries = ["Former Hammer", "Former Staff"]
        .into_iter()
        .enumerate()
        .map(|(ordinal, label)| ReviewedCorrespondence {
            before: format!("items:{ordinal:016}").parse().unwrap(),
            after: "items:bbbbbbbbbbbbbbbb".parse().unwrap(),
            before_label: label.into(),
            after_label: "Dragon Staff".into(),
            namespace: "fixture".into(),
            source_type: "weapon".into(),
            rationale: "Explicit fixture correspondence".into(),
        })
        .collect();
    input.remaster_sources.push(RemasterSourceInput::new(
        RemasterInputKind::AtlasCorrespondence,
        RemasterOrigin::AtlasReviewed,
        "fixture/v1".into(),
        "fixtures/shared-correspondences.json".into(),
        serde_json::to_vec(&ReviewedCorrespondences { entries }).unwrap(),
    ));
    crate::test_support::capture_remaster_declarations(&mut input).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace.sqlite");
    crate::test_support::publish_input(&path, &input).unwrap();
    let reader = crate::test_support::open_reader(&path).unwrap();
    let aliases = ["Former Hammer", "Former Staff"]
        .map(|alias| ("items:bbbbbbbbbbbbbbbb".parse().unwrap(), alias.to_owned()));
    reader.reset_read_metrics();
    let one = reader
        .remaster_alias_information(&aliases[..1], 20)
        .unwrap();
    let single_metrics = reader.read_metrics();
    assert_eq!(one[&aliases[0]].declarations.len(), 2);
    reader.reset_read_metrics();
    let repeated = [aliases[0].clone(), aliases[1].clone(), aliases[0].clone()];
    let all = reader.remaster_alias_information(&repeated, 20).unwrap();
    let all_metrics = reader.read_metrics();
    assert_eq!(all.len(), 2);
    assert!(
        all.values()
            .all(|info| info.declarations.len() == 2 && !info.truncated)
    );
    let first_ids = all[&aliases[0]]
        .declarations
        .iter()
        .map(|value| value.declaration.id.clone())
        .collect::<BTreeSet<_>>();
    let second_ids = all[&aliases[1]]
        .declarations
        .iter()
        .map(|value| value.declaration.id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(first_ids.intersection(&second_ids).count(), 1);
    let origins = all
        .values()
        .flat_map(|value| &value.declarations)
        .flat_map(|value| &value.citations)
        .map(|citation| citation.origin)
        .collect::<Vec<_>>();
    assert!(origins.contains(&RemasterOrigin::Foundry));
    assert!(origins.contains(&RemasterOrigin::AtlasReviewed));
    // Count every executed membership/declaration/binding/citation/selected-field/
    // capture statement. More aliases and repeated supports add no SQL rounds.
    assert_eq!(single_metrics.sql_statements, 6);
    assert_eq!(all_metrics.sql_statements, 6);
    assert_eq!(all_metrics.source_body_decodes, 0);
    assert_eq!(all_metrics.prepared_content_batches, 0);
    let bounded = reader.remaster_alias_information(&aliases, 1).unwrap();
    assert!(
        bounded
            .values()
            .all(|info| info.declarations.len() == 1 && info.truncated)
    );
    assert_eq!(reader.remaster_alias_information(&[], 20).unwrap().len(), 0);
}

#[test]
fn discussion_is_readable_on_exact_source_journal_and_page_without_other_associations() {
    let mut input = fixture(false);
    add_journal(
        &mut input,
        json!([
            {"_id":"1111111111111111","name":"Alignment","type":"text","text":{"format":1,"content":"<p>Alignment is removed. Dragon Staff retains its authored rules.</p><p>Spirit damage replaces the earlier alignment damage categories.</p>"}},
            {"_id":"2222222222222222","name":"Spell Schools","type":"text","text":{"format":1,"content":"<p>Spell schools are removed from the remastered rules.</p>"}},
            {"_id":"3333333333333333","name":"Components","type":"text","text":{"format":1,"content":"<p>Spell components are replaced by the appropriate action traits.</p>"}}
        ]),
    );
    crate::test_support::capture_remaster_declarations(&mut input).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace.sqlite");
    crate::test_support::publish_input(&path, &input).unwrap();
    let reader = crate::test_support::open_reader(&path).unwrap();
    let journal = "journals:eeeeeeeeeeeeeeee".parse().unwrap();
    let root = reader
        .remaster_record_information(&journal, &[], 2)
        .unwrap();
    assert_eq!(root.declarations.len(), 2);
    assert!(root.truncated);
    let all = reader
        .remaster_record_information(&journal, &[], 100)
        .unwrap();
    assert_eq!(all.declarations.len(), 4);
    assert!(!all.truncated);
    for (page, count) in [
        ("1111111111111111", 2),
        ("2222222222222222", 1),
        ("3333333333333333", 1),
    ] {
        let owners = [OwnedContentLocator {
            collection: "/pages".into(),
            identity: OwnedContentIdentity::Stable(page.into()),
        }];
        reader.reset_read_metrics();
        let selected = reader
            .remaster_record_information(&journal, &owners, 20)
            .unwrap();
        assert_eq!(selected.declarations.len(), count);
        assert!(
            selected
                .declarations
                .iter()
                .all(|value| value.declaration.kinds == [RemasterChangeKind::Discussion])
        );
        assert!(
            selected
                .declarations
                .iter()
                .flat_map(|value| &value.citations)
                .all(|citation| citation
                    .locator
                    .as_ref()
                    .is_some_and(|locator| locator.record == journal && locator.owners == owners))
        );
        assert_eq!(reader.read_metrics().source_body_decodes, 0);
    }
    // Mentioning an entry in global rules prose grants no record association,
    // alias, preferred replacement or synthetic identity.
    let staff = "items:bbbbbbbbbbbbbbbb".parse().unwrap();
    assert!(
        reader
            .remaster_record_information(&staff, &[], 20)
            .unwrap()
            .declarations
            .is_empty()
    );
    assert!(
        reader
            .remaster_term_information("Alignment", 20)
            .unwrap()
            .declarations
            .is_empty()
    );
    assert!(
        reader
            .remaster_record_information(&"journals:0000000000000000".parse().unwrap(), &[], 20)
            .unwrap()
            .declarations
            .is_empty()
    );
}
