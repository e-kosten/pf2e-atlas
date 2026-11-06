use super::*;
use atlas_domain::{QueryDefinitionReferenceTarget, QueryExpression, QueryPredicate, RecordKey};
use serde_json::json;

fn linked_fixture() -> (IndexBuildInput, Vec<ReferenceEntry>, Vec<ReferenceNotice>) {
    let (mut input, entries, notices) = fixture();
    input.records[0] = crate::foundation_tests::record(
        "actors",
        "Actor",
        json!({
            "_id":"aaaaaaaaaaaaaaaa","name":"Owned contextual source","type":"npc",
            "system":{"details":{"level":{"value":1}}},
            "items":[{"_id":"ffffffffffffffff","name":"Owned sense use","type":"action","system":{"description":{"value":"<p>@Localize[Fixture.Sense]</p>"}}}]
        }),
    );
    input.records[1] = crate::foundation_tests::record(
        "items",
        "Item",
        json!({
            "_id":"bbbbbbbbbbbbbbbb","name":"Two exact reference kinds","type":"weapon","system":{"description":{"value":"<p>@Localize[Fixture.Sense] @UUID[Compendium.pf2e.items.Item.eeeeeeeeeeeeeeee] @Localize[Fixture.First] @UUID[Compendium.pf2e.actors.Actor.aaaaaaaaaaaaaaaa.Item.ffffffffffffffff]</p><span data-visibility='none'>@Localize[Fixture.Language]</span>"}}
        }),
    );
    let mut index = atlas_record::source_record::SourceReferenceIndex::default();
    for source in &input.records {
        index.insert_source(source.record.key(), source.record.source());
    }
    input.lexical_units.clear();
    let preparation_hash = input.context.preparation_sha256().unwrap();
    for source in &mut input.records {
        source.preparation_context_hash = preparation_hash.clone();
        source.content = atlas_record::source_record::prepare_record_content(
            &source.record,
            input.context.audience,
            Some(&Locale),
            Some(&index),
        );
        source.relationships =
            atlas_record::source_record::resolve_source_relationships(&source.record, Some(&index));
        let pack = input
            .packs
            .iter()
            .find(|pack| pack.pack_id == source.record.key().pack().as_str())
            .unwrap();
        let selection = atlas_record::source_record::select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            &pack.label,
            &input.context.used_trait_labels,
        )
        .unwrap();
        input
            .lexical_units
            .extend(crate::project_lexical_units(&selection, &[]));
    }
    (input, entries, notices)
}
fn eligible(reader: &SqliteIndexReader, predicate: &QueryPredicate) -> Vec<RecordKey> {
    reader
        .eligible_keys(
            &crate::validate_query(predicate).unwrap(),
            None,
            false,
            atlas_domain::ResultSort::attribute(
                atlas_domain::SortField::Name,
                atlas_domain::SortDirection::Asc,
            ),
            0,
            100,
        )
        .unwrap()
        .keys
}

#[test]
fn definition_reference_union_keeps_distinct_roots_exact_owners_and_bounded_mixed_evidence() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("references.sqlite");
    let (mut input, entries, notices) = linked_fixture();
    publish(&path, &mut input, &entries, &notices).unwrap();
    let reader = open_reader(&path).unwrap();
    let whole = crate::parse_where("references_definition('sense:darkvision')").unwrap();
    assert_eq!(eligible(&reader, whole.predicate()).len(), 2);
    let main =
        crate::parse_where("references_definition('sense:darkvision','main_entry')").unwrap();
    assert_eq!(
        eligible(&reader, main.predicate()),
        vec!["items:bbbbbbbbbbbbbbbb".parse().unwrap()]
    );
    let direct = crate::parse_where("references_definition('items:eeeeeeeeeeeeeeee')").unwrap();
    assert_eq!(
        eligible(&reader, direct.predicate()),
        eligible(&reader, whole.predicate())
    );
    let air = crate::parse_where("references_definition('domain:air')").unwrap();
    assert_eq!(
        eligible(&reader, air.predicate()),
        vec!["items:bbbbbbbbbbbbbbbb".parse().unwrap()]
    );
    let hidden = crate::parse_where("references_definition('language:unspoken')").unwrap();
    assert!(eligible(&reader, hidden.predicate()).is_empty());
    let owned=crate::parse_where("references_definition({'kind':'record','selection':{'record_key':'actors:aaaaaaaaaaaaaaaa','embedded_path':[{'collection':'/items','identity':{'Stable':'ffffffffffffffff'}}]}})").unwrap();
    assert_eq!(
        eligible(&reader, owned.predicate()),
        vec!["items:bbbbbbbbbbbbbbbb".parse().unwrap()]
    );
    let parent = crate::parse_where("references_definition('actors:aaaaaaaaaaaaaaaa')").unwrap();
    assert!(eligible(&reader, parent.predicate()).is_empty());
    let QueryExpression::DefinitionReference { reference } = &whole.predicate().expression else {
        panic!("definition reference expected")
    };
    let first = reader
        .definition_reference_occurrence_page(crate::DefinitionReferenceOccurrencePageRequest {
            reference,
            result: &"items:bbbbbbbbbbbbbbbb".parse().unwrap(),
            offset: 0,
            limit: 1,
        })
        .unwrap();
    let second = reader
        .definition_reference_occurrence_page(crate::DefinitionReferenceOccurrencePageRequest {
            reference,
            result: &"items:bbbbbbbbbbbbbbbb".parse().unwrap(),
            offset: 1,
            limit: 1,
        })
        .unwrap();
    assert_eq!((first.total, second.total), (2, 2));
    assert!(
        matches!(&first.occurrences[0],crate::DefinitionReferenceOccurrence::Definition{occurrence} if matches!(&occurrence.source,DefinitionReferenceSource::Record{owners,..} if owners.is_empty()))
    );
    assert!(
        matches!(&second.occurrences[0],crate::DefinitionReferenceOccurrence::Record{occurrence} if occurrence.locator.record.to_string()=="items:bbbbbbbbbbbbbbbb")
    );
    let owned_evidence = reader
        .definition_reference_occurrence_page(crate::DefinitionReferenceOccurrencePageRequest {
            reference,
            result: &"actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
            offset: 0,
            limit: 100,
        })
        .unwrap();
    assert_eq!(owned_evidence.total, 1);
    assert!(
        matches!(&owned_evidence.occurrences[0],crate::DefinitionReferenceOccurrence::Definition{occurrence} if matches!(&occurrence.source,DefinitionReferenceSource::Record{owners,field:Some(_),..} if owners.len()==1))
    );
    assert_eq!(reader.read_metrics().source_body_decodes, 0);
}

#[test]
fn definition_reference_boolean_proofs_obey_successful_branches_and_shared_limits() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("references.sqlite");
    let (mut input, entries, notices) = linked_fixture();
    publish(&path, &mut input, &entries, &notices).unwrap();
    let reader = open_reader(&path).unwrap();
    let query=crate::parse_where("(references_definition('sense:darkvision') && kind=='creature') || references_definition('domain:air')").unwrap();
    let actor: RecordKey = "actors:aaaaaaaaaaaaaaaa".parse().unwrap();
    let weapon: RecordKey = "items:bbbbbbbbbbbbbbbb".parse().unwrap();
    let proofs = reader
        .relationship_proofs(&query, &[actor.clone(), weapon.clone()])
        .unwrap();
    let clauses = atlas_domain::definition_reference_clauses(query.predicate()).unwrap();
    assert_eq!(
        proofs,
        vec![(actor, clauses[0].0.into()), (weapon, clauses[1].0.into())]
    );
    assert!(
        crate::parse_where("embedded.abilities.exists(x,references_definition('domain:air'))")
            .is_err()
    );
    let leaf = crate::parse_where("references_definition('domain:air')")
        .unwrap()
        .predicate()
        .clone();
    let mut excessive = QueryPredicate::new(QueryExpression::AllOf {
        children: (0..17)
            .map(|_| QueryPredicate::new(leaf.expression.clone()))
            .collect(),
    });
    atlas_domain::assign_relationship_clause_ids(&mut excessive).unwrap();
    assert!(crate::validate_query(&excessive).is_err());
    let round_trip = serde_json::to_value(&leaf).unwrap();
    let restored: QueryPredicate = serde_json::from_value(round_trip.clone()).unwrap();
    assert_eq!(serde_json::to_value(restored).unwrap(), round_trip);
    assert!(
        crate::parse_where(
            "references_definition({'kind':'definition','key':'domain:air','unknown':true})"
        )
        .is_err()
    );
}

#[test]
fn definition_filter_choices_page_all_zero_values_and_keep_distinct_selected_identities() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("references.sqlite");
    let (mut input, mut entries, notices) = fixture();
    let prototype = entries
        .iter()
        .find(|entry| entry.key.to_string() == "language:unspoken")
        .unwrap()
        .clone();
    for definition in atlas_foundry_model::generated::reference_definitions()
        .iter()
        .filter(|definition| definition.namespace == "language")
    {
        let mut entry = prototype.clone();
        entry.key =
            DefinitionKey::new(DefinitionNamespace::Language, definition.identifier).unwrap();
        entry.label = format!("Zero language {}", definition.identifier);
        entry.filter_bindings = vec![ReferenceFilterBinding {
            id: "speakers".into(),
            relation: "property".into(),
            field: Some("languages".into()),
            value: Some(definition.identifier.into()),
        }];
        entries.push(entry);
    }
    let mut excluded = prototype;
    excluded.key = "sense:vision".parse().unwrap();
    excluded.availability = ReferenceAvailability::SourceInapplicable;
    entries.push(excluded);
    publish(&path, &mut input, &entries, &notices).unwrap();
    let reader = open_reader(&path).unwrap();
    let mut targets = std::collections::BTreeSet::new();
    for offset in [0, 100] {
        let page = reader
            .definition_filter_options(crate::DefinitionFilterOptionsRequest {
                text: None,
                offset,
                limit: 100,
                selected: vec![],
            })
            .unwrap();
        assert_eq!(page.total, 123);
        for option in page.options {
            assert!(targets.insert(serde_json::to_string(&option.target()).unwrap()));
        }
    }
    assert_eq!(targets.len(), 123);
    let missing = QueryDefinitionReferenceTarget::Definition {
        key: "language:removed-value".parse().unwrap(),
    };
    let forbidden = QueryDefinitionReferenceTarget::Definition {
        key: "sense:vision".parse().unwrap(),
    };
    let retained = QueryDefinitionReferenceTarget::Definition {
        key: "language:common".parse().unwrap(),
    };
    let custom = QueryDefinitionReferenceTarget::Record {
        selection: Box::new(atlas_domain::QueryRecordSelection {
            record_key: "items:eeeeeeeeeeeeeeee".parse().unwrap(),
            owners: vec![],
            field: None,
            passage: None,
            source_fingerprint: None,
            spell_rank: None,
        }),
    };
    let page = reader
        .definition_filter_options(crate::DefinitionFilterOptionsRequest {
            text: Some("No matching option".into()),
            offset: 100,
            limit: 1,
            selected: vec![missing.clone(), forbidden, retained, custom, missing],
        })
        .unwrap();
    assert_eq!(page.total, 0);
    assert!(page.options.is_empty());
    assert_eq!(page.selected.len(), 4);
    use crate::DefinitionFilterSelectionAvailability::*;
    assert_eq!(
        page.selected
            .iter()
            .map(|selection| selection.availability)
            .collect::<Vec<_>>(),
        vec![Missing, SourceInapplicable, Available, NotInCatalog]
    );
    assert!(
        matches!(&page.selected[3].option,Some(crate::DefinitionFilterOption::Record{summary}) if summary.name.as_value().map(String::as_str)==Some("Context reference"))
    );
    assert_eq!(reader.read_metrics().source_body_decodes, 0);
}

#[test]
fn record_definition_choices_require_actual_admission_family_and_captured_pack_scope() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("references.sqlite");
    let (mut input, entries, notices) = fixture();
    let known = atlas_foundry_model::generated::foundry_glossary_records();
    let first: RecordKey = known[0].0.parse().unwrap();
    let second: RecordKey = known[1].0.parse().unwrap();
    assert_eq!(first.pack(), second.pack());
    let pack = first.pack().as_str().to_owned();
    input.packs.push(crate::IndexBuildPack {
        pack_id: pack.clone(),
        label: "Actual glossary records".into(),
        document_kind: "Item".into(),
        declared_path: format!("packs/{pack}"),
    });
    input.records.push(crate::foundation_tests::record(&pack,"Item",json!({"_id":first.id().as_str(),"name":"Catalog exact admitted record","type":known[0].1,"system":{"description":{"value":"Actual custom prose stays on this record."}}})));
    input.records.push(crate::foundation_tests::record(&pack,"Item",json!({"_id":second.id().as_str(),"name":"Catalog conflicting family","type":"effect","system":{"description":{"value":"Ordinary record remains readable."}}})));
    input.inventory = crate::CompleteSourceInventory::checked(
        &input.context.source_fingerprint,
        input.packs.iter().map(|pack| pack.pack_id.clone()),
        input.packs.iter().map(|pack| pack.pack_id.clone()),
    )
    .unwrap();
    input.context.source_file_count = input.records.len();
    let preparation_hash = input.context.preparation_sha256().unwrap();
    for source in &mut input.records {
        source.preparation_context_hash = preparation_hash.clone();
    }
    for source in input
        .records
        .iter()
        .filter(|source| source.record.key().pack().as_str() == pack)
    {
        let selection = atlas_record::source_record::select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            "Actual glossary records",
            &input.context.used_trait_labels,
        )
        .unwrap();
        input
            .lexical_units
            .extend(crate::project_lexical_units(&selection, &[]));
    }
    publish(&path, &mut input, &entries, &notices).unwrap();
    let reader = open_reader(&path).unwrap();
    let page = reader
        .definition_filter_options(crate::DefinitionFilterOptionsRequest {
            text: Some("Catalog".into()),
            offset: 0,
            limit: 100,
            selected: vec![],
        })
        .unwrap();
    assert_eq!(page.total, 1);
    assert!(
        matches!(&page.options[0],crate::DefinitionFilterOption::Record{summary} if summary.key==first && summary.name.as_value().map(String::as_str)==Some("Catalog exact admitted record"))
    );
    assert_eq!(reader.read_metrics().source_body_decodes, 0);
    drop(reader);
    input
        .context
        .reference_preparation
        .as_mut()
        .unwrap()
        .pack_systems
        .insert(pack, Some("sf2e".into()));
    publish(&path, &mut input, &entries, &notices).unwrap();
    let reader = open_reader(&path).unwrap();
    let page = reader
        .definition_filter_options(crate::DefinitionFilterOptionsRequest {
            text: Some("Catalog".into()),
            offset: 0,
            limit: 100,
            selected: vec![],
        })
        .unwrap();
    assert_eq!(page.total, 0);
    assert!(reader.read_summary(&first).unwrap().is_some());
    assert!(reader.read_summary(&second).unwrap().is_some());
}
