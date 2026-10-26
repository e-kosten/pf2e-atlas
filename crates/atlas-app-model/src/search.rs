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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchCriteria {
    SimilarRecords {
        seed_record_key: String,
        filter: Option<QueryPredicate>,
    },
    SimilarTraits {
        seed_trait_key: String,
        filter: Option<QueryPredicate>,
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
    pub relationship_evidence: Vec<RelationshipEvidenceView>,
    pub record: RecordSummaryView,
    pub matches: Vec<SearchWitnessView>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelationshipClauseSummaryView {
    pub clause_id: String,
    pub relationship: atlas_domain::QueryRelationship,
    pub record: RecordSummaryView,
    pub title: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RelationshipClausePresentationView {
    Available {
        clause_id: String,
        relationship: atlas_domain::QueryRelationship,
        record: Box<RecordSummaryView>,
        title: String,
    },
    Unavailable {
        clause_id: String,
        relationship: atlas_domain::QueryRelationship,
        message: String,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelationshipEvidenceView {
    pub clause_id: String,
    pub occurrences: Vec<crate::RecordRelationshipView>,
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
    pub occurrences: Vec<crate::RecordRelationshipView>,
    pub page: SearchPageView,
}
