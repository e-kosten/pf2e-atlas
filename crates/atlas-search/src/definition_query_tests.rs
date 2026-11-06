use crate::{
    test_support::{input, record},
    *,
};
use atlas_domain::{
    DefinitionKey, QueryDefinitionReferenceTarget, QueryExpression, QueryPredicate,
};
use atlas_index::{
    ReferenceAvailability, ReferenceBacking, ReferenceCoverage, ReferenceEntry,
    ReferenceFilterBinding, ReferencePermission, ReferencePermissionDisposition,
    ReferencePreparation, ReferenceProvenance, ReferenceRecordMapping,
};
use serde_json::json;

fn fixture() -> (AtlasRetrievalService, tempfile::TempDir) {
    let target = "aaaaaaaaaaaaaaaa";
    let mut records = vec![record(
        "items",
        "Item",
        json!({"_id":target,"name":"Reference Target","type":"action","system":{"description":{"value":"<p>Reference definition.</p>"}}}),
    )];
    for index in 0..125 {
        records.push(record("items","Item",json!({"_id":format!("{index:016}"),"name":format!("Reference source {index:03}"),"type":if index==124 {"weapon"} else {"effect"},"system":{"description":{"value":format!("<p>@UUID[Compendium.pf2e.items.Item.{target}] @UUID[Compendium.pf2e.items.Item.{target}]</p>")}}})));
    }
    records.push(record("actors","Actor",json!({"_id":"cccccccccccccccc","name":"Owned reference source","type":"npc","system":{"details":{"level":{"value":1}}},"items":[{"_id":"dddddddddddddddd","name":"Owned reference","type":"action","system":{"description":{"value":format!("<p>@UUID[Compendium.pf2e.items.Item.{target}]</p>")}}}]})));
    let mut input = input(records, true);
    let provenance = ReferenceProvenance {
        source_package: "pf2e".into(),
        source_repository: None,
        source_revision: None,
        source_file: None,
        source_sha256: None,
        label_key: None,
        description_key: None,
        catalogs: vec![],
        source_locations: vec![],
        indexing_locale: "en".into(),
        content_locale: None,
        publications: vec![],
        record_mapping: None,
        recipe: "search-reference-fixture/v1".into(),
    };
    let label_only = ReferenceEntry {
        key: "language:common".parse().unwrap(),
        label: "Common".into(),
        availability: ReferenceAvailability::LabelOnly,
        backing: ReferenceBacking::None,
        provenance: provenance.clone(),
        permission: ReferencePermission {
            disposition: ReferencePermissionDisposition::MetadataOnlyUnresolved,
            body_grants: vec![],
            label_grants: vec![],
            notices: vec![],
        },
        filter_bindings: vec![ReferenceFilterBinding {
            id: "speakers".into(),
            relation: "property".into(),
            field: Some("languages".into()),
            value: Some("common".into()),
        }],
    };
    let mut mapped = label_only.clone();
    mapped.key = "rule:pf2e:Fixture.Reference".parse().unwrap();
    mapped.label = "Mapped reference".into();
    mapped.availability = ReferenceAvailability::Available;
    mapped.backing = ReferenceBacking::Record {
        key: format!("items:{target}").parse().unwrap(),
    };
    mapped.permission.disposition = ReferencePermissionDisposition::ExistingRecordPolicy;
    mapped.provenance.record_mapping = Some(ReferenceRecordMapping {
        target: format!("items:{target}").parse().unwrap(),
        package: "pf2e".into(),
        expected_document_kind: "Item".into(),
        expected_item_type: Some("action".into()),
        required_localization_key: None,
    });
    let mut missing = mapped.clone();
    missing.key = "rule:pf2e:Fixture.Removed".parse().unwrap();
    missing.availability = ReferenceAvailability::MissingTarget;
    missing.backing = ReferenceBacking::None;
    missing.provenance.record_mapping.as_mut().unwrap().target =
        "items:xxxxxxxxxxxxxxxx".parse().unwrap();
    let entries = vec![label_only, mapped, missing];
    let (expected_entry_count, entry_keys_sha256) =
        atlas_index::reference_inventory_identity(entries.iter().map(|entry| entry.key.clone()))
            .unwrap();
    input.context.reference_preparation = Some(ReferencePreparation {
        source_package: Some("pf2e".into()),
        pack_systems: Default::default(),
        recipe: "search-reference-fixture/v1".into(),
        input_hash: crate::test_support::hash(b"complete reference fixture"),
        coverage: ReferenceCoverage::Complete,
        expected_entry_count,
        entry_keys_sha256,
    });
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace.sqlite");
    atlas_index::test_support::publish_input_with_references(&path, &input, &entries, &[], &[])
        .unwrap();
    let reader = atlas_index::test_support::open_reader(&path).unwrap();
    (
        AtlasRetrievalService::from_index(
            reader,
            atlas_domain::SearchProfileId::new("bge-base"),
            None,
        )
        .unwrap(),
        directory,
    )
}

#[test]
fn exact_definition_eligibility_precedes_pagination_text_similarity_and_saved_scope() {
    let (mut service, _directory) = fixture();
    let predicate = service
        .parse_where("references_definition('rule:pf2e:Fixture.Reference')")
        .unwrap()
        .predicate()
        .clone();
    let first = service
        .list_records(ListRecordsRequest::new(
            Some(&predicate),
            SearchPage::new(1, 100).unwrap(),
        ))
        .unwrap();
    assert_eq!((first.total, first.records.len()), (126, 100));
    let second = service
        .list_records(ListRecordsRequest::new(
            Some(&predicate),
            SearchPage::new(2, 100).unwrap(),
        ))
        .unwrap();
    assert_eq!((second.total, second.records.len()), (126, 26));
    let narrowed = service
        .parse_where("references_definition('rule:pf2e:Fixture.Reference') && kind=='equipment'")
        .unwrap()
        .predicate()
        .clone();
    let page = service
        .list_records(ListRecordsRequest::new(
            Some(&narrowed),
            SearchPage::default(),
        ))
        .unwrap();
    assert_eq!(
        page.record_keys,
        vec!["items:0000000000000124".parse().unwrap()]
    );
    let mut text = TextSearchRequest::new("Reference", SearchPage::default());
    text.mode = RetrievalMode::Fts;
    text.filter = Some(&narrowed);
    assert_eq!(service.search_text(text).unwrap().page.total, 1);
    let keys = vec!["items:0000000000000124".parse().unwrap()];
    let scoped = service
        .list_records(
            ListRecordsRequest::new(Some(&predicate), SearchPage::default())
                .with_scope(RecordScope::Keys(&keys)),
        )
        .unwrap();
    assert_eq!(scoped.record_keys, keys);
    let similar = service
        .similar_records(SimilarRecordRequest {
            content_scope: atlas_domain::SearchContentScope::All,
            sort: atlas_domain::ResultSort::Relevance {},
            search_profile: None,
            record_key: &"items:aaaaaaaaaaaaaaaa".parse().unwrap(),
            filter: Some(&narrowed),
            scope: RecordScope::All,
            page: SearchPage::default(),
            prefer_remaster: true,
        })
        .unwrap()
        .unwrap();
    assert_eq!(similar.results.page.total, 1);
    let zero = service
        .parse_where("references_definition('language:common')")
        .unwrap();
    assert_eq!(
        service
            .list_records(ListRecordsRequest::new(
                Some(zero.predicate()),
                SearchPage::default()
            ))
            .unwrap()
            .total,
        0
    );
}

#[test]
fn missing_definition_targets_are_rejected_even_under_unused_or_and_negation_before_inference() {
    let (mut service, _directory) = fixture();
    for expression in [
        "!references_definition('language:removed')",
        "true || references_definition('language:removed')",
        "false && references_definition('items:zzzzzzzzzzzzzzzz')",
        "!references_definition('rule:pf2e:Fixture.Removed')",
    ] {
        let parsed = atlas_index::parse_where(expression).unwrap();
        assert_eq!(
            service
                .validate_filter(parsed.predicate())
                .unwrap_err()
                .kind(),
            SearchErrorKind::InvalidOptions,
            "{expression}"
        );
        let mut request = TextSearchRequest::new("Reference", SearchPage::default());
        request.mode = RetrievalMode::Vector;
        request.filter = Some(parsed.predicate());
        assert_eq!(
            service.search_text(request).unwrap_err().kind(),
            SearchErrorKind::InvalidOptions,
            "invalid target must precede unavailable query inference: {expression}"
        );
    }
}

#[test]
fn definition_evidence_requires_a_successful_positive_proof_and_keeps_owned_source() {
    let (service, _directory) = fixture();
    let query=service.parse_where("(references_definition('rule:pf2e:Fixture.Reference') && kind=='creature') || source.type=='effect'").unwrap();
    let clauses = atlas_domain::definition_reference_clauses(query.predicate()).unwrap();
    let clause = clauses[0].0;
    let result = "actors:cccccccccccccccc".parse().unwrap();
    let (evidence, page) = service
        .definition_reference_evidence(query.predicate(), clause, &result, SearchPage::default())
        .unwrap();
    assert_eq!(page.total, 1);
    assert!(
        matches!(&evidence[0],DefinitionReferenceOccurrence::Record{occurrence} if occurrence.locator.owners.len()==1 && occurrence.locator.field=="/system/description/value")
    );
    assert_eq!(
        service
            .definition_reference_evidence(
                query.predicate(),
                clause,
                &"items:0000000000000000".parse().unwrap(),
                SearchPage::default()
            )
            .unwrap_err()
            .kind(),
        SearchErrorKind::InvalidOptions
    );
    let negative = service
        .parse_where("!references_definition('language:common')")
        .unwrap();
    let id = atlas_domain::definition_reference_clauses(negative.predicate()).unwrap()[0].0;
    assert!(
        service
            .definition_reference_evidence(negative.predicate(), id, &result, SearchPage::default())
            .is_err()
    );
    let parsed:QueryPredicate=serde_json::from_value(json!({"clause_id":"caller-owned","kind":"definition_reference","reference":{"target":{"kind":"definition","key":"language:common"},"include_contents":true}})).unwrap();
    let QueryExpression::DefinitionReference { reference } = parsed.expression else {
        panic!("expected reference")
    };
    assert_eq!(
        reference.target,
        QueryDefinitionReferenceTarget::Definition {
            key: "language:common".parse::<DefinitionKey>().unwrap()
        }
    );
}
