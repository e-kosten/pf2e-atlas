use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::RecordKey;
use atlas_record::remaster::{RecordRemasterContext, RemasterRelation, RemasterTarget};
impl AtlasRetrievalService {
    pub fn record_remaster_context(
        &self,
        key: &RecordKey,
        owners: &[atlas_domain::OwnedContentLocator],
    ) -> Result<Vec<RecordRemasterContext>, SearchError> {
        let mut contexts = self.index.read_remaster_context(key, owners)?;
        if !owners.is_empty() {
            return Ok(contexts);
        }
        for pair in self.index.read_remaster_links(key)? {
            let (target, relation) = if &pair.legacy == key {
                (pair.remaster, RemasterRelation::ReplacedBy)
            } else {
                (pair.legacy, RemasterRelation::Replaces)
            };
            let Some(summary) = self.index.read_summary(&target)? else {
                continue;
            };
            let scope = contexts
                .iter()
                .find(|c| {
                    c.owners.is_empty()
                        && c.journal == pair.provenance
                        && c.targets
                            .iter()
                            .any(|t| t.record.as_ref() == Some(&target) && t.owners.is_empty())
                })
                .map(|c| c.scope.clone())
                .unwrap_or_else(|| "counterpart".into());
            contexts.push(RecordRemasterContext {
                record: key.clone(),
                owners: vec![],
                relation,
                targets: vec![RemasterTarget {
                    name: summary
                        .name
                        .as_value()
                        .cloned()
                        .unwrap_or_else(|| target.to_string()),
                    record: Some(target),
                    owners: vec![],
                }],
                journal: pair.provenance,
                scope,
            });
        }
        Ok(contexts)
    }
}
