use crate::test_support::{
    add_remaster_alias, add_remaster_relationship, input, open_input_fixture, record,
};
use crate::*;
use atlas_domain::{OwnedContentIdentity, OwnedContentLocator, RecordKey};
use serde_json::json;

fn key(id: &str) -> RecordKey {
    format!("items:{id}").parse().unwrap()
}
fn effect(id: &str, name: &str) -> atlas_index::SourceArtifactRecordInput {
    record(
        "items",
        "Item",
        json!({"_id":id,"name":name,"type":"effect","system":{"description":{"value":""},"traits":{"value":[]}}}),
    )
}
#[test]
fn unavailable_exact_key_information_preserves_identity_and_never_reads_a_body() {
    let old = key("aaaaaaaaaaaaaaaa");
    let current = key("bbbbbbbbbbbbbbbb");
    let mut data = input(vec![effect("bbbbbbbbbbbbbbbb", "Current entry")], false);
    add_remaster_relationship(&mut data, &old, &current).unwrap();
    let (service, _artifact) = open_input_fixture(data).unwrap();
    service.reset_read_metrics();
    let info = service
        .record_remaster_information(RecordRemasterInformationRequest {
            record_key: &old,
            owners: &[],
            limit: 20,
        })
        .unwrap();
    assert_eq!(info.record_key, old);
    assert!(info.seed.is_none());
    assert_eq!(info.availability, RemasterRecordAvailability::Unavailable);
    let change = &info.information.declarations[0];
    assert!(matches!(&change.before[0].endpoints[0].resolution,
        RemasterEndpointResolution::MissingExplicitKey { record_key } if record_key == &old));
    assert!(matches!(&change.after[0].endpoints[0].resolution,
        RemasterEndpointResolution::Current { record, owners } if record.key == current && owners.is_empty()));
    assert_eq!(change.citations[0].origin, RemasterOrigin::AtlasReviewed);
    assert_eq!(service.read_metrics().source_body_decodes, 0);
    assert!(
        service
            .get_record(GetRecordRequest {
                record_key: &old,
                selected_content: &[]
            })
            .unwrap()
            .is_none()
    );
    let unknown = key("cccccccccccccccc");
    assert!(
        service
            .record_remaster_information(RecordRemasterInformationRequest {
                record_key: &unknown,
                owners: &[],
                limit: 20,
            })
            .unwrap()
            .information
            .declarations
            .is_empty()
    );
    let bad_owner = [OwnedContentLocator {
        collection: "items".into(),
        identity: OwnedContentIdentity::Stable("dddddddddddddddd".into()),
    }];
    for root in [&old, &current] {
        assert_eq!(
            service
                .record_remaster_information(RecordRemasterInformationRequest {
                    record_key: root,
                    owners: &bad_owner,
                    limit: 20,
                })
                .unwrap_err()
                .kind(),
            SearchErrorKind::InvalidOptions
        );
    }
}

#[test]
fn alias_context_follows_delivered_alias_witnesses_and_preserves_multiple_sources() {
    let current = key("bbbbbbbbbbbbbbbb");
    let mut data = input(vec![effect("bbbbbbbbbbbbbbbb", "Current dragon")], false);
    add_remaster_alias(&mut data, &current, "Verified dragon").unwrap();
    add_remaster_alias(&mut data, &current, "Verified dragon").unwrap();
    let (mut service, _artifact) = open_input_fixture(data).unwrap();
    let resolved = service
        .resolve_record(ResolveRecordRequest {
            query: "Verified dragon",
            filter: None,
        })
        .unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].support.as_ref().unwrap().declarations.len(), 2);
    let primary = service
        .resolve_record(ResolveRecordRequest {
            query: "Current dragon",
            filter: None,
        })
        .unwrap();
    assert!(primary[0].support.is_none());
    service.reset_read_metrics();
    // The query is only part of the alias. Actual FTS alias attribution still
    // supplies the complete matched source name and its two supports.
    let mut request = TextSearchRequest::new("Verified", SearchPage::default());
    request.mode = RetrievalMode::Fts;
    let result = service.search_text(request).unwrap();
    assert_eq!(result.records.len(), 1);
    assert_eq!(result.records[0].alias_matches[0].alias, "Verified dragon");
    assert_eq!(
        result.records[0].alias_matches[0]
            .support
            .declarations
            .len(),
        2
    );
    assert_eq!(service.read_metrics().source_body_decodes, 0);
    let mut request = TextSearchRequest::new("Current", SearchPage::default());
    request.mode = RetrievalMode::Fts;
    assert!(
        service.search_text(request).unwrap().records[0]
            .alias_matches
            .is_empty()
    );
}

#[test]
fn term_information_retains_removed_context_without_creating_record_identity() {
    let journal = record(
        "journals",
        "JournalEntry",
        json!({
            "_id":"aaaaaaaaaaaaaaaa","name":"Remaster Changes","pages":[{
                "_id":"bbbbbbbbbbbbbbbb","name":"Feats","type":"text","text":{"format":1,
                "content":"<table><tr><th>Old</th><th>Status</th><th>New</th></tr><tr><td>Forgotten option</td><td>Removed</td><td>Folded into the general rules</td></tr></table>"}
            }]
        }),
    );
    let mut data = input(vec![journal], false);
    crate::test_support::capture_remaster_declarations(&mut data).unwrap();
    let (service, _artifact) = open_input_fixture(data).unwrap();
    service.reset_read_metrics();
    let result = service
        .remaster_term_information(RemasterTermInformationRequest {
            term: "Forgotten option",
            limit: 20,
        })
        .unwrap();
    assert_eq!(result.information.declarations.len(), 1);
    let change = &result.information.declarations[0];
    assert_eq!(change.kinds, [RemasterChangeKind::Removed]);
    assert_eq!(change.before[0].text, "Forgotten option");
    assert_eq!(change.after[0].text, "Folded into the general rules");
    assert!(matches!(
        change.before[0].endpoints[0].resolution,
        RemasterEndpointResolution::Unresolved { reference: None }
    ));
    assert!(matches!(
        change.citations[0].source,
        RemasterCitationSource::Journal { .. }
    ));
    assert_eq!(service.read_metrics().source_body_decodes, 0);
    assert!(
        service
            .resolve_record(ResolveRecordRequest {
                query: "Forgotten option",
                filter: None
            })
            .unwrap()
            .is_empty()
    );
    for term in ["", "   ", &"a".repeat(257)] {
        assert_eq!(
            service
                .remaster_term_information(RemasterTermInformationRequest { term, limit: 20 })
                .unwrap_err()
                .kind(),
            SearchErrorKind::InvalidOptions
        );
    }
    for limit in [0, 101] {
        assert!(
            service
                .remaster_term_information(RemasterTermInformationRequest {
                    term: "Forgotten option",
                    limit
                })
                .is_err()
        );
    }
}
