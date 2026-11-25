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

#[derive(Clone)]
pub struct CheckedRelationshipSelection {
    pub record: SourceRecordSummary,
    pub selected_name: Option<String>,
}
/// Reused only inside this admitted index reader. Each filter has at most 16 link conditions.
pub(crate) struct PreparedFilter {
    pub(crate) query: atlas_index::ValidatedQuery,
    selections: Vec<(QueryRecordSelection, CheckedRelationshipSelection)>,
}

impl AtlasRetrievalService {
    pub fn connected_record_keys(
        &self,
        filter: Option<&QueryPredicate>,
        scope: crate::RecordScope<'_>,
        direction: atlas_domain::QueryRelationshipDirection,
        include_contents: bool,
    ) -> Result<Vec<atlas_domain::RecordKey>, crate::SearchError> {
        Ok(self.index.connected_record_keys(
            &self.predicate(filter)?,
            scope.keys(),
            direction,
            include_contents,
        )?)
    }
    /// Admit the page predicate before connection-scope resolution and retain its selections
    /// for clause titles, retrieval, and evidence in the same snapshot.
    pub fn prepare_page_filter(&mut self, predicate: &QueryPredicate) -> Result<(), SearchError> {
        let query = atlas_index::validate_query(predicate)?;
        let mut selections = Vec::new();
        visit_selections(predicate, &mut |selection| {
            if !selections.iter().any(|(key, _)| key == selection) {
                selections.push((
                    selection.clone(),
                    self.checked_relationship_selection(selection)?,
                ));
            }
            Ok(())
        })?;
        self.prepared_filter = Some(PreparedFilter { query, selections });
        Ok(())
    }
    pub fn checked_relationship_selection(
        &self,
        s: &QueryRecordSelection,
    ) -> Result<CheckedRelationshipSelection, SearchError> {
        if let Some((_, checked)) = self
            .prepared_filter
            .as_ref()
            .and_then(|filter| filter.selections.iter().find(|(key, _)| key == s))
        {
            return Ok(checked.clone());
        }
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
    pub fn relationship_proofs(
        &self,
        p: &QueryPredicate,
        keys: &[RecordKey],
    ) -> Result<Vec<(RecordKey, String)>, SearchError> {
        let query = self.validate_filter(p)?;
        Ok(self.index.relationship_proofs(&query, keys)?)
    }
}

pub(crate) fn visit_selections(
    p: &QueryPredicate,
    visit: &mut impl FnMut(&QueryRecordSelection) -> Result<(), SearchError>,
) -> Result<(), SearchError> {
    match &p.expression {
        QueryExpression::Relationship { relationship } => {
            visit(&relationship.selection)?;
        }
        QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
            for child in children {
                visit_selections(child, visit)?;
            }
        }
        QueryExpression::Not { predicate } | QueryExpression::Exists { predicate, .. } => {
            visit_selections(predicate, visit)?
        }
        _ => {}
    }
    Ok(())
}
