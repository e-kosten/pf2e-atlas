use crate::{RecordNavigationView, RecordSummaryView};
use atlas_domain::QueryPredicate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalModeView {
    Lexical,
    Semantic,
    Hybrid,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SearchPageRequest {
    pub number: u32,
    pub size: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SearchPageView {
    pub number: u32,
    pub size: u32,
    pub count: usize,
    pub total: u64,
    pub has_more: bool,
    pub next_page: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SearchResultsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sort: Option<atlas_domain::ResultSort>,
    #[serde(default)]
    #[ts(optional)]
    pub ranking: Option<atlas_domain::SearchRanking>,
    #[ts(optional)]
    pub search_profile: Option<atlas_domain::SearchProfileId>,
    pub criteria: SearchCriteria,
    pub page: SearchPageRequest,
    pub expected_artifact_generation: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SearchCriteria {
    SimilarRecords {
        seed_record_key: String,
        filter: Option<QueryPredicate>,
        #[serde(default)]
        #[ts(optional)]
        similarity: Option<atlas_domain::SimilarityOptions>,
        #[serde(default)]
        #[ts(optional)]
        source_sections: Option<atlas_domain::SimilaritySourceSections>,
        #[serde(default)]
        #[ts(optional)]
        search_content: Option<atlas_domain::SearchContentScope>,
    },
    SimilarTraits {
        seed_trait_key: String,
        filter: Option<QueryPredicate>,
        #[serde(default)]
        #[ts(optional)]
        similarity: Option<atlas_domain::SimilarityOptions>,
        #[serde(default)]
        #[ts(optional)]
        source_sections: Option<atlas_domain::SimilaritySourceSections>,
        #[serde(default)]
        #[ts(optional)]
        search_content: Option<atlas_domain::SearchContentScope>,
    },
    TraitReferences {
        trait_key: String,
        query: Option<String>,
        filter: Option<QueryPredicate>,
    },
    ListRecords {
        filter: Option<QueryPredicate>,
    },
    TextSearch {
        query: String,
        filter: Option<QueryPredicate>,
        mode: RetrievalModeView,
        #[serde(default)]
        #[ts(optional)]
        search_content: Option<atlas_domain::SearchContentScope>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchModeSummary {
    SimilarRecords {
        seed: Box<RecordSummaryView>,
    },
    SimilarTraits {
        trait_key: String,
        title: String,
    },
    TraitReferences {
        trait_key: String,
        title: String,
    },
    ListRecords,
    TextSearch {
        query: String,
        mode: RetrievalModeView,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchResultsPage {
    pub search_content: atlas_domain::SearchContentScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub core_recipe: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub similarity: Option<crate::SimilarityConfigurationView>,
    pub sort: atlas_domain::ResultSort,
    pub relationship_clauses: Vec<RelationshipClauseSummaryView>,
    pub ranking: atlas_domain::SearchRanking,
    pub search_profile: atlas_domain::SearchProfileId,
    pub artifact_generation: String,
    pub mode: SearchModeSummary,
    pub page: SearchPageView,
    pub rows: Vec<SearchResultRow>,
    pub coverage: Option<SearchCoverageView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchResultRow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub similarity: Option<crate::SimilarityEvidenceView>,
    pub relationship_evidence: Vec<RelationshipEvidenceView>,
    pub record: RecordSummaryView,
    pub matches: Vec<SearchWitnessView>,
    /// Display destinations coalesced across retrieval lanes; raw witnesses remain above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub matching_sections: Option<Vec<SearchMatchSectionView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchMatchSectionView {
    pub navigation: RecordNavigationView,
    pub label: String,
    pub snippet: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchWitnessView {
    pub navigation: RecordNavigationView,
    pub lane: SearchLaneView,
    pub label: Option<String>,
    pub snippet: Option<String>,
    pub lexical_rank: Option<f64>,
    pub semantic_similarity: Option<f64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SearchLaneView {
    Lexical,
    Semantic,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchCoverageView {
    pub exhaustive: bool,
    pub semantic_unit_window: Option<usize>,
    pub candidate_roots: usize,
    pub count_basis: String,
}

/// Executable reference condition, independent of its human label and reading destination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceClauseConditionView {
    Record {
        relationship: Box<atlas_domain::QueryRelationship>,
    },
    Definition {
        reference: atlas_domain::QueryDefinitionReference,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelationshipClauseSummaryView {
    pub clause_id: String,
    pub condition: ReferenceClauseConditionView,
    pub destination: Option<crate::ReferenceDestinationView>,
    /// Supplemental actual-record metadata; never an identity or navigation fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record: Option<Box<RecordSummaryView>>,
    pub title: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RelationshipClausePresentationView {
    Available {
        clause_id: String,
        condition: ReferenceClauseConditionView,
        destination: Option<crate::ReferenceDestinationView>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        record: Option<Box<RecordSummaryView>>,
        title: String,
    },
    Unavailable {
        clause_id: String,
        condition: ReferenceClauseConditionView,
        message: String,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReferenceOccurrenceView {
    Record {
        occurrence: Box<crate::RecordRelationshipView>,
        destination: Box<crate::ReferenceDestinationView>,
    },
    Definition {
        occurrence: Box<DefinitionReferenceEvidenceView>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionReferenceEvidenceView {
    pub source: atlas_domain::QueryDefinitionReferenceTarget,
    pub source_destination: Option<crate::ReferenceDestinationView>,
    pub source_title: String,
    pub source_field: Option<String>,
    pub target: atlas_domain::QueryDefinitionReferenceTarget,
    pub target_destination: Option<crate::ReferenceDestinationView>,
    pub target_title: String,
    pub occurrence_path: String,
    pub evidence: String,
    pub label: Option<String>,
    pub visible: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelationshipEvidenceView {
    pub clause_id: String,
    pub occurrences: Vec<ReferenceOccurrenceView>,
    pub page: SearchPageView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelationshipOccurrencesRequest {
    pub filter: QueryPredicate,
    pub clause_id: String,
    pub result_record_key: String,
    pub page: SearchPageRequest,
    pub expected_artifact_generation: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelationshipOccurrencesPage {
    pub artifact_generation: String,
    pub clause: RelationshipClauseSummaryView,
    pub result_record: RecordSummaryView,
    pub occurrences: Vec<ReferenceOccurrenceView>,
    pub page: SearchPageView,
}

#[cfg(test)]
mod reference_reading_contract_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn queryable_clause_and_evidence_keep_identity_with_explicit_null_reading() {
        let identity = json!({"kind":"definition","key":"rule:pf2e:Fixture.Rule.Hidden"});
        let condition =
            json!({"kind":"definition","reference":{"target":identity,"include_contents":true}});
        let clause: RelationshipClauseSummaryView = serde_json::from_value(json!({
            "clause_id":"hidden", "condition":condition, "destination":null, "title":"Hidden rule"
        }))
        .unwrap();
        let encoded = serde_json::to_value(clause).unwrap();
        assert_eq!(encoded["condition"], condition);
        assert_eq!(encoded.get("destination"), Some(&serde_json::Value::Null));
        let presentation: RelationshipClausePresentationView = serde_json::from_value(json!({
            "status":"available", "clause_id":"hidden", "condition":condition, "destination":null, "title":"Hidden rule"
        })).unwrap();
        assert_eq!(
            serde_json::to_value(presentation)
                .unwrap()
                .get("destination"),
            Some(&serde_json::Value::Null)
        );
        let evidence: DefinitionReferenceEvidenceView = serde_json::from_value(json!({
            "source":identity, "source_destination":null, "source_title":"Hidden rule", "source_field":null,
            "target":identity, "target_destination":null, "target_title":"Hidden rule", "occurrence_path":"/0", "evidence":"Fixture.Rule.Hidden", "label":null, "visible":true
        })).unwrap();
        let encoded = serde_json::to_value(evidence).unwrap();
        assert_eq!(encoded["source"], identity);
        assert_eq!(encoded["target"], identity);
        assert_eq!(
            encoded.get("source_destination"),
            Some(&serde_json::Value::Null)
        );
        assert_eq!(
            encoded.get("target_destination"),
            Some(&serde_json::Value::Null)
        );
    }
}
