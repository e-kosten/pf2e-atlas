//! Admission checks every selected source before Boolean truth or inference can hide it.
use crate::{AtlasRetrievalService, GetRecordRequest, SearchError};
use atlas_domain::{
    OwnedContentIdentity, QueryExpression, QueryPredicate, QueryRecordSelection, RecordKey,
    SourceRecordSummary,
};
use atlas_record::{
    source_content::SourceContentLocator,
    source_record::{SourceContentFormat, SourceFieldView, recover_source_passage},
};

pub struct CheckedRelationshipSelection {
    pub record: SourceRecordSummary,
    pub selected_name: Option<String>,
}
impl AtlasRetrievalService {
    pub fn checked_relationship_selection(
        &self,
        s: &QueryRecordSelection,
    ) -> Result<CheckedRelationshipSelection, SearchError> {
        let invalid = |message: &str| SearchError::invalid_search_options(message);
        if s.owners
            .iter()
            .any(|o| matches!(o.identity, OwnedContentIdentity::SnapshotLocal { .. }))
            && s.source_fingerprint.as_deref() != Some(self.source_fingerprint())
        {
            return Err(invalid(
                "This selection belongs to an older reference library. Open the record again.",
            ));
        }
        let summary = self.index.read_summary(&s.record_key)?.ok_or_else(|| {
            invalid("The relationship record is unavailable. Open the record again.")
        })?;
        if s.owners.is_empty() && s.field.is_none() {
            if s.passage
                .as_ref()
                .is_some_and(|p| !matches!(p, atlas_domain::SourcePassageAddress::Identity {}))
            {
                return Err(invalid("Passage navigation requires one selected field"));
            }
            let selected_name = summary.name.as_value().cloned();
            return Ok(CheckedRelationshipSelection {
                record: summary,
                selected_name,
            });
        }
        let detail = self
            .get_record(GetRecordRequest {
                record_key: &s.record_key,
                selected_content: &[],
            })?
            .ok_or_else(|| invalid("The relationship record is unavailable"))?;
        let node = detail
            .source
            .node_at(&s.owners)
            .ok_or_else(|| invalid("The selected entry is unavailable. Open the record again."))?;
        let selections = node.content_selections();
        if let Some(field) = &s.field {
            let content = selections
                .iter()
                .find(|c| c.field == *field)
                .ok_or_else(|| {
                    invalid("The selected text is unavailable. Open the record again.")
                })?;
            if let Some(passage) = &s.passage {
                if matches!(passage, atlas_domain::SourcePassageAddress::Identity {}) {
                    return Err(invalid("Identity navigation cannot select a prose field"));
                }
                let plain = if matches!(
                    content.format,
                    SourceFieldView::Value(SourceContentFormat::Plain)
                ) {
                    content.text.value().map(|v| v.as_str())
                } else {
                    None
                };
                let locator = SourceContentLocator {
                    record: s.record_key.clone(),
                    owners: s.owners.clone(),
                    field: field.clone(),
                };
                let cache = self.read_content(&[locator])?;
                let html = cache.fields.first().and_then(|f| f.html.as_deref());
                recover_source_passage(html, plain, passage).map_err(|_| {
                    invalid("The selected passage is unavailable. Open the record again.")
                })?;
            }
        } else if s
            .passage
            .as_ref()
            .is_some_and(|p| !matches!(p, atlas_domain::SourcePassageAddress::Identity {}))
        {
            return Err(invalid("Passage navigation requires one selected field"));
        }
        let selected_name = node.name().value().map(|v| (*v).clone());
        Ok(CheckedRelationshipSelection {
            record: summary,
            selected_name,
        })
    }
    pub(crate) fn check_relationship_leaves(&self, p: &QueryPredicate) -> Result<(), SearchError> {
        match &p.expression {
            QueryExpression::Relationship { relationship } => {
                self.checked_relationship_selection(&relationship.selection)?;
            }
            QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
                for child in children {
                    self.check_relationship_leaves(child)?;
                }
            }
            QueryExpression::Not { predicate } | QueryExpression::Exists { predicate, .. } => {
                self.check_relationship_leaves(predicate)?
            }
            _ => {}
        }
        Ok(())
    }
    pub fn relationship_proofs(
        &self,
        p: &QueryPredicate,
        keys: &[RecordKey],
    ) -> Result<Vec<(RecordKey, String)>, SearchError> {
        let query = self.validate_filter(p)?;
        Ok(self.index.relationship_proofs(&query, keys)?)
    }
}
