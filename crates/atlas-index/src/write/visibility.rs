use std::collections::BTreeSet;

use atlas_domain::{RecordKey, RecordKind};
use atlas_record::{AtlasRecord, RecordVisibilityReason, RemasterLink};

pub(crate) struct RetrievalVisibility {
    hidden_record_keys: BTreeSet<RecordKey>,
}

impl RetrievalVisibility {
    pub(crate) fn from_remaster_links(remaster_links: &[RemasterLink]) -> Self {
        Self {
            hidden_record_keys: remaster_links
                .iter()
                .map(|link| link.legacy_record_key.clone())
                .collect(),
        }
    }

    pub(crate) fn is_default_visible(&self, record: &AtlasRecord) -> bool {
        self.policy(record).1 == "ordinary"
    }

    pub(crate) fn policy(
        &self,
        record: &AtlasRecord,
    ) -> (&'static str, &'static str, &'static str) {
        if record.classification.kind == RecordKind::Tooling {
            return (
                "source",
                "inspection_only",
                "tooling_no_addressable_product_meaning",
            );
        }
        if self.hidden_record_keys.contains(&record.identity.key) {
            return ("source", "direct_only", "canonical_edition_duplicate");
        }
        if record.visibility.reason() == RecordVisibilityReason::GeneratedInstance {
            return (
                "source_instance",
                "direct_only",
                "duplicate_source_instance",
            );
        }
        if record.visibility.reason() == RecordVisibilityReason::GeneratedCanonical {
            return ("canonical", "ordinary", "generated_canonical");
        }
        ("source", "ordinary", "source_record")
    }
}
