use std::collections::BTreeSet;

use atlas_domain::{RecordKey, RecordKind};

use crate::records::{AtlasRecord, RemasterLink};

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
        record.visibility.visible_by_default()
            && record.classification.kind != RecordKind::Tooling
            && !self.hidden_record_keys.contains(&record.identity.key)
    }
}
