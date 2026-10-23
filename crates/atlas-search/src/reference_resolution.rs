//! Bounded, body-free resolution of supplied root references.
use crate::{AtlasRetrievalService, GetRecordsRequest, ResolveRecordRequest, SearchError};
use atlas_record::{source_content::ResolvedContentReference, source_record::SourceReferenceIndex};

impl AtlasRetrievalService {
    pub fn resolve_source_root_reference(
        &self,
        target: &str,
    ) -> Result<Option<ResolvedContentReference>, SearchError> {
        let Some((pack, identifier)) = SourceReferenceIndex::root_lookup_hint(target) else {
            return Ok(None);
        };
        let mut candidates = self
            .resolve_record(ResolveRecordRequest {
                query: identifier,
                filter: None,
            })?
            .into_iter()
            .map(|candidate| candidate.record)
            .filter(|record| record.key.pack().as_str() == pack)
            .collect::<Vec<_>>();
        if let Ok(key) = atlas_domain::RecordKey::parse(&format!("{pack}:{identifier}")) {
            for record in self
                .get_records(GetRecordsRequest {
                    record_keys: &[key],
                })?
                .records
            {
                if !candidates
                    .iter()
                    .any(|candidate| candidate.key == record.key)
                {
                    candidates.push(record);
                }
            }
        }
        let mut index = SourceReferenceIndex::default();
        for candidate in candidates {
            index.insert_root_summary(&candidate);
        }
        Ok(index.resolve_global_reference(target))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::{open_source_fixture, record};
    use serde_json::json;
    #[test]
    fn root_reference_resolution_preserves_collisions_kind_and_pack() {
        let (retrieval, _artifact) = open_source_fixture(
            vec![
                record(
                    "items",
                    "Item",
                    json!({"_id":"aaaaaaaaaaaaaaaa","type":"feat","name":"Same Name"}),
                ),
                record(
                    "items",
                    "Item",
                    json!({"_id":"bbbbbbbbbbbbbbbb","type":"feat","name":"Same Name"}),
                ),
                record(
                    "other",
                    "Item",
                    json!({"_id":"cccccccccccccccc","type":"feat","name":"Unique"}),
                ),
                record(
                    "items",
                    "Item",
                    json!({"_id":"dddddddddddddddd","type":"feat","name":"parent.Item.child"}),
                ),
            ],
            false,
        )
        .unwrap();
        for target in [
            "Compendium.pf2e.items.Item.Same Name",
            "Compendium.pf2e.items.Item.Unique",
            "Compendium.pf2e.items.Actor.aaaaaaaaaaaaaaaa",
            "Same Name",
            "Compendium.pf2e.items.Item.parent.Item.child",
        ] {
            assert!(
                retrieval
                    .resolve_source_root_reference(target)
                    .unwrap()
                    .is_none(),
                "{target}"
            );
        }
        let resolved = retrieval
            .resolve_source_root_reference("Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa")
            .unwrap()
            .unwrap();
        assert_eq!(resolved.display_name.as_deref(), Some("Same Name"));
        assert_eq!(resolved.item_family.as_deref(), Some("feat"));
    }
}
