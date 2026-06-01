use std::collections::BTreeSet;

use atlas_domain::{RecordFamily, RecordKey};
use atlas_record::{NormalizedRecord, RemasterLink};

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

    pub(crate) fn is_default_visible(&self, record: &NormalizedRecord) -> bool {
        record.is_default_visible
            && record.record_family != RecordFamily::Tooling
            && !self.hidden_record_keys.contains(&record.key)
    }
}
