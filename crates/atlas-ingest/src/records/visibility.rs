use std::collections::BTreeSet;

use atlas_domain::{RecordKey, RecordKind};
use atlas_record::RemasterLink;

use crate::generated::afflictions::GeneratedAfflictionRole;

use super::LoadedSourceRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductRetrievalDisposition {
    Ordinary,
    DirectOnly,
    InspectionOnly,
}

pub(crate) struct ProductRetrievalPolicy {
    demoted_legacy_record_keys: BTreeSet<RecordKey>,
}

impl ProductRetrievalPolicy {
    pub(crate) fn from_remaster_links(remaster_links: &[RemasterLink]) -> Self {
        Self {
            demoted_legacy_record_keys: remaster_links
                .iter()
                .map(|link| link.legacy_record_key.clone())
                .collect(),
        }
    }

    pub(crate) fn disposition(&self, loaded: &LoadedSourceRecord) -> ProductRetrievalDisposition {
        if loaded.record.classification.kind == RecordKind::Tooling {
            return ProductRetrievalDisposition::InspectionOnly;
        }
        if self
            .demoted_legacy_record_keys
            .contains(&loaded.record.identity.key)
            || loaded.facts.generated_affliction_role
                == Some(GeneratedAfflictionRole::SourceInstance)
        {
            return ProductRetrievalDisposition::DirectOnly;
        }
        ProductRetrievalDisposition::Ordinary
    }

    pub(crate) fn is_ordinary(&self, loaded: &LoadedSourceRecord) -> bool {
        self.disposition(loaded) == ProductRetrievalDisposition::Ordinary
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, RecordId, RecordKey, RecordKind};
    use atlas_record::{
        AtlasRecord, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordIdentity, RecordProvenance, RecordVisibility,
        RecordVisibilityReason,
    };

    use super::*;
    use crate::records::SourceConstructionFacts;

    #[test]
    fn authored_visibility_classification_does_not_control_product_retrieval() {
        let mut record = record("authored", RecordKind::Rule);
        record.visibility = RecordVisibility::hidden(RecordVisibilityReason::SourceRecord);
        let loaded = LoadedSourceRecord::new(record, SourceConstructionFacts::empty());

        assert_eq!(
            ProductRetrievalPolicy::from_remaster_links(&[]).disposition(&loaded),
            ProductRetrievalDisposition::Ordinary
        );
    }

    #[test]
    fn generated_role_and_tooling_kind_have_explicit_non_ordinary_dispositions() {
        let mut source_instance_facts = SourceConstructionFacts::empty();
        source_instance_facts.generated_affliction_role =
            Some(GeneratedAfflictionRole::SourceInstance);
        let source_instance = LoadedSourceRecord::new(
            record("source-instance", RecordKind::Affliction),
            source_instance_facts,
        );
        let tooling = LoadedSourceRecord::new(
            record("tooling", RecordKind::Tooling),
            SourceConstructionFacts::empty(),
        );
        let policy = ProductRetrievalPolicy::from_remaster_links(&[]);

        assert_eq!(
            policy.disposition(&source_instance),
            ProductRetrievalDisposition::DirectOnly
        );
        assert_eq!(
            policy.disposition(&tooling),
            ProductRetrievalDisposition::InspectionOnly
        );
    }

    fn record(id: &str, kind: RecordKind) -> AtlasRecord {
        let pack_name = PackName::new("test-pack").expect("pack parses");
        let id = RecordId::new(id).expect("id parses");
        AtlasRecord::new(
            RecordIdentity::new(RecordKey::new(pack_name, id), "Test Record"),
            RecordClassification::new(kind),
            FoundryRecordInfo::new(
                "Test Pack",
                FoundryDocumentType::Item,
                FoundryRecordType::Action,
            ),
            RecordProvenance::new("test.json").with_raw_json("{}"),
        )
    }
}
