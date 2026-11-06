//! Exact incoming definition references use the ordinary search predicate.
use crate::{AtlasRetrievalService, SearchError, SearchPage, SearchPageInfo};
use atlas_domain::{
    QueryDefinitionReferenceTarget, QueryPredicate, RecordKey, SourceRecordSummary,
};
pub use atlas_index::{
    DefinitionReferenceOccurrence, DefinitionReferenceOccurrencePage,
    DefinitionReferenceOccurrencePageRequest,
};
use atlas_index::{
    ReferenceAvailability, ReferenceCoverage, ReferenceSummary, ReferenceSummaryBacking,
};

#[derive(Debug, Clone)]
pub enum CheckedDefinitionReferenceTarget {
    Record {
        record: Box<SourceRecordSummary>,
        selected_name: Option<String>,
    },
    Definition {
        definition: Box<ReferenceSummary>,
    },
}
impl AtlasRetrievalService {
    pub fn checked_definition_reference_target(
        &self,
        target: &QueryDefinitionReferenceTarget,
    ) -> Result<CheckedDefinitionReferenceTarget, SearchError> {
        let invalid = SearchError::invalid_search_options;
        if self.index.reference_coverage() != ReferenceCoverage::Complete {
            return Err(SearchError::index_unavailable(
                "Definition reference coverage requires explicit atlas setup",
            ));
        }
        match target {
            QueryDefinitionReferenceTarget::Record { selection } => {
                let selected = self.checked_relationship_selection(selection)?;
                Ok(CheckedDefinitionReferenceTarget::Record {
                    record: Box::new(selected.record),
                    selected_name: selected.selected_name,
                })
            }
            QueryDefinitionReferenceTarget::Definition { key } => {
                let definition = self.index.reference_summary(key)?.ok_or_else(|| {
                    invalid("The selected definition is unavailable. Open its source again.")
                })?;
                if matches!(
                    definition.availability,
                    ReferenceAvailability::SourceInapplicable | ReferenceAvailability::Withdrawn
                ) {
                    return Err(invalid(
                        "The selected definition is not applicable to this reference collection",
                    ));
                }
                match &definition.backing {
                    ReferenceSummaryBacking::Record { key } => {
                        self.checked_relationship_selection(&atlas_domain::QueryRecordSelection {
                            record_key: key.clone(),
                            owners: vec![],
                            field: None,
                            passage: None,
                            source_fingerprint: None,
                            spell_rank: None,
                        })?;
                    }
                    _ if definition.provenance.record_mapping.is_some() => {
                        return Err(invalid(
                            "The selected definition's record mapping is unavailable. Open its source again.",
                        ));
                    }
                    _ => {}
                }
                Ok(CheckedDefinitionReferenceTarget::Definition {
                    definition: Box::new(definition),
                })
            }
        }
    }

    /// A complete predicate and positive Boolean proof authorize exact evidence.
    pub fn definition_reference_evidence(
        &self,
        predicate: &QueryPredicate,
        clause_id: &str,
        result: &RecordKey,
        page: SearchPage,
    ) -> Result<(Vec<DefinitionReferenceOccurrence>, SearchPageInfo), SearchError> {
        let query = self.validate_filter(predicate)?;
        atlas_domain::select_relationship_clause_id(predicate, Some(clause_id))?;
        let reference = atlas_domain::definition_reference_clauses(predicate)?
            .into_iter()
            .find(|(id, _)| *id == clause_id)
            .map(|(_, reference)| reference)
            .ok_or_else(|| {
                SearchError::invalid_search_options("Choose a definition reference condition")
            })?;
        if self.index.read_summary(result)?.is_none()
            || !self
                .index
                .relationship_proofs(&query, std::slice::from_ref(result))?
                .iter()
                .any(|(key, id)| key == result && id == clause_id)
        {
            return Err(SearchError::invalid_search_options(
                "The selected result has no positive proof for this definition reference",
            ));
        }
        let result = self.index.definition_reference_occurrence_page(
            DefinitionReferenceOccurrencePageRequest {
                reference,
                result,
                offset: page.offset()? as usize,
                limit: page.size() as usize,
            },
        )?;
        let info = SearchPageInfo::from_page(page, result.occurrences.len(), result.total)?;
        Ok((result.occurrences, info))
    }
}
