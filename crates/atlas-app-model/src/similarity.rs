//! Stored-vector similarity controls and selectable source content.
use atlas_domain::{
    OwnedContentLocator, SearchProfileId, SimilarityOptions, SimilarityPartSelector,
    SimilarityRole, SimilaritySeed,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SimilarityPartsRequest {
    pub seed: SimilaritySeed,
    #[ts(optional)]
    pub search_profile: Option<SearchProfileId>,
    pub expected_artifact_generation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityPartsView {
    pub artifact_generation: String,
    pub search_profile: SearchProfileId,
    pub seed: SimilaritySeed,
    pub parts: Vec<SimilarityPartView>,
    pub role_recipe: String,
    pub supports_corresponding_parts: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityConfigurationView {
    pub options: SimilarityOptions,
    pub recipe: String,
    pub role_recipe: String,
    pub diagnostics: SimilarityDiagnosticsView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityDiagnosticsView {
    pub seed_units: usize,
    pub knn_queries: usize,
    pub candidate_vectors: usize,
    pub pairwise_comparisons: usize,
    pub undefined_candidate_centroids: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityPartView {
    /// Record source navigation; passage selects the full section when metadata
    /// provides its canonical address, otherwise the containing field is selected.
    pub navigation: Option<crate::RecordNavigationView>,
    pub selector: SimilarityPartSelector,
    pub label: String,
    pub owner_label: String,
    pub owners: Vec<OwnedContentLocator>,
    pub field: String,
    pub section_ordinal: usize,
    pub role: SimilarityRole,
    pub role_label: String,
    pub has_vectors: bool,
    pub label_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityEvidenceView {
    pub truncated: bool,
    pub score: f64,
    pub aggregation: String,
    pub selected_parts: usize,
    pub comparable_parts: usize,
    pub parts: Vec<SimilarityPartScoreView>,
    pub roles: Vec<SimilarityRoleScoreView>,
    pub pairs: Vec<SimilarityPairView>,
    pub representative: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityPartScoreView {
    pub part: SimilarityPartView,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityRoleScoreView {
    pub role: SimilarityRole,
    pub role_label: String,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityPairView {
    pub seed_part: SimilarityPartSelector,
    pub seed: Option<crate::SearchWitnessView>,
    pub seed_text: String,
    pub candidate: crate::SearchWitnessView,
    pub cosine: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn method_specific_options_reject_ignored_fields() {
        for value in [
            json!({"method":"baseline","decay":0.5}),
            json!({"method":"content-average","decay":0.5}),
            json!({"method":"baseline","parts":["exact-part"]}),
        ] {
            assert!(
                serde_json::from_value::<SimilarityOptions>(value.clone()).is_err(),
                "accepted inapplicable method fields: {value}"
            );
        }
    }

    #[test]
    fn similarity_options_belong_to_similar_criteria_only() {
        let value = json!({"kind":"text_search","query":"fire","filter":null,"mode":"hybrid","similarity":{"method":"baseline"}});
        assert!(serde_json::from_value::<crate::SearchCriteria>(value).is_err());
        let criteria: crate::SearchCriteria = serde_json::from_value(json!({
            "kind":"similar_records","seed_record_key":"items:0000000000000000",
            "filter":null,"similarity":{"method":"passages","decay":0.125,"parts":["exact-part"],"candidate_parts":"corresponding"}
        })).unwrap();
        assert!(matches!(
            criteria,
            crate::SearchCriteria::SimilarRecords {
                similarity: Some(SimilarityOptions::Passages { decay: 0.125, .. }),
                ..
            }
        ));
    }
}
