mod get;
pub(crate) mod resolution;
mod types;

use atlas_record::AtlasRecord;

use crate::SearchError;

pub use types::{
    GetRecordRequest, GetRecordsRequest, ListRecordsRequest, ListRecordsResult, RecordListSort,
    RecordRefResolutionResult, RecordResolutionMatchKind, RecordResolutionResult,
    ResolveRecordRefRequest, ResolveRecordRequest,
};

pub trait RecordRetrieval {
    fn get_records(&self, request: GetRecordsRequest<'_>) -> Result<Vec<AtlasRecord>, SearchError>;

    fn get_record(&self, request: GetRecordRequest<'_>)
    -> Result<Option<AtlasRecord>, SearchError>;

    fn list_records(
        &self,
        request: ListRecordsRequest<'_>,
    ) -> Result<ListRecordsResult, SearchError>;

    fn resolve_record(
        &self,
        request: ResolveRecordRequest<'_>,
    ) -> Result<Vec<RecordResolutionResult>, SearchError>;

    fn resolve_record_ref(
        &self,
        request: ResolveRecordRefRequest<'_>,
    ) -> Result<RecordRefResolutionResult, SearchError> {
        if let Ok(key) = atlas_domain::RecordKey::parse(request.record_ref) {
            return Ok(RecordRefResolutionResult::Key(key));
        }
        let matches = self.resolve_record(ResolveRecordRequest {
            query: request.record_ref,
            filter: request.filter,
        })?;
        Ok(match matches.len() {
            0 => RecordRefResolutionResult::Miss,
            1 => RecordRefResolutionResult::Key(matches[0].record.identity.key.clone()),
            _ => RecordRefResolutionResult::Ambiguous(matches),
        })
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        AtlasRecord, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordIdentity, RecordProvenance,
    };

    use super::*;

    struct FakeRecordRetrieval {
        matches: Vec<RecordResolutionResult>,
    }

    impl RecordRetrieval for FakeRecordRetrieval {
        fn get_records(
            &self,
            _request: GetRecordsRequest<'_>,
        ) -> Result<Vec<AtlasRecord>, SearchError> {
            unimplemented!("record-ref resolution does not load records by key")
        }

        fn get_record(
            &self,
            _request: GetRecordRequest<'_>,
        ) -> Result<Option<AtlasRecord>, SearchError> {
            unimplemented!("record-ref resolution does not load records by key")
        }

        fn list_records(
            &self,
            _request: ListRecordsRequest<'_>,
        ) -> Result<ListRecordsResult, SearchError> {
            unimplemented!("record-ref resolution does not list records")
        }

        fn resolve_record(
            &self,
            _request: ResolveRecordRequest<'_>,
        ) -> Result<Vec<RecordResolutionResult>, SearchError> {
            Ok(self.matches.clone())
        }
    }

    #[test]
    fn record_ref_resolution_accepts_canonical_key_without_lookup() {
        let service = FakeRecordRetrieval {
            matches: Vec::new(),
        };

        let result = service
            .resolve_record_ref(ResolveRecordRefRequest {
                record_ref: "actions:testAction",
                filter: None,
            })
            .expect("record ref should resolve");

        assert_eq!(
            result,
            RecordRefResolutionResult::Key(record_key("actions:testAction"))
        );
    }

    #[test]
    fn record_ref_resolution_reports_miss_and_ambiguity_for_names() {
        let miss = FakeRecordRetrieval {
            matches: Vec::new(),
        }
        .resolve_record_ref(ResolveRecordRefRequest {
            record_ref: "Missing",
            filter: None,
        })
        .expect("record ref should resolve");
        assert_eq!(miss, RecordRefResolutionResult::Miss);

        let ambiguous = FakeRecordRetrieval {
            matches: vec![
                resolution("actions:first", "Duplicate"),
                resolution("actions:second", "Duplicate"),
            ],
        }
        .resolve_record_ref(ResolveRecordRefRequest {
            record_ref: "Duplicate",
            filter: None,
        })
        .expect("record ref should resolve");

        match ambiguous {
            RecordRefResolutionResult::Ambiguous(matches) => assert_eq!(matches.len(), 2),
            other => panic!("expected ambiguity, got {other:?}"),
        }
    }

    fn resolution(key: &str, name: &str) -> RecordResolutionResult {
        RecordResolutionResult {
            query: name.to_string(),
            normalized_query: name.to_lowercase(),
            match_kind: RecordResolutionMatchKind::Name,
            matched_text: name.to_string(),
            alias_source: None,
            alias_source_ref: None,
            record: fake_record(key, name),
        }
    }

    fn fake_record(key: &str, name: &str) -> AtlasRecord {
        AtlasRecord::new(
            RecordIdentity::new(record_key(key), name),
            RecordClassification::new(RecordKind::Rule),
            FoundryRecordInfo::new(
                "Actions",
                FoundryDocumentType::Item,
                FoundryRecordType::Action,
            ),
            RecordProvenance {
                source_path: format!("packs/actions/{name}.json"),
                raw_json: Some("{}".to_string()),
            },
        )
    }

    fn record_key(value: &str) -> RecordKey {
        RecordKey::parse(value).expect("fixture key should parse")
    }
}
