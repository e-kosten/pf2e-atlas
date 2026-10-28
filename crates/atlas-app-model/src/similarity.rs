//! Stored-vector similarity controls and selectable source content.
use atlas_domain::{
    CoreSectionSelection, OwnedContentLocator, SearchContentScope, SearchProfileId,
    SimilarityComparisonKey, SimilarityOptions, SimilarityPartSelector, SimilarityRole,
    SimilaritySeed, SimilaritySourceSections,
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
    pub parts: Vec<SimilarityComponentView>,
    pub core: CoreSectionSelection,
    pub role_recipe: String,
    pub supports_corresponding_parts: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityConfigurationView {
    pub source_sections: Option<SimilaritySourceSections>,
    pub search_content: SearchContentScope,
    pub candidate_identity: bool,
    pub core_recipe: Option<String>,
    pub resolved_selectors: Vec<SimilarityPartSelector>,
    pub core: Option<CoreSectionSelection>,
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
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SimilarityComponentView {
    Identity {
        navigation: Option<crate::RecordNavigationView>,
        selector: SimilarityPartSelector,
        label: String,
        owner_label: String,
        has_vectors: bool,
    },
    Section {
        #[serde(flatten)]
        part: SimilarityPartView,
    },
}

impl SimilarityComponentView {
    pub fn part(&self) -> Option<&SimilarityPartView> {
        match self {
            Self::Identity { .. } => None,
            Self::Section { part } => Some(part),
        }
    }
    pub fn navigation(&self) -> Option<&crate::RecordNavigationView> {
        match self {
            Self::Identity { navigation, .. } => navigation.as_ref(),
            Self::Section { part } => part.navigation.as_ref(),
        }
    }
    pub fn selector(&self) -> &SimilarityPartSelector {
        match self {
            Self::Identity { selector, .. } => selector,
            Self::Section { part } => &part.selector,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            Self::Identity { label, .. } => label,
            Self::Section { part } => &part.label,
        }
    }
    pub fn owner_label(&self) -> &str {
        match self {
            Self::Identity { owner_label, .. } => owner_label,
            Self::Section { part } => &part.owner_label,
        }
    }
    pub fn has_vectors(&self) -> bool {
        match self {
            Self::Identity { has_vectors, .. } => *has_vectors,
            Self::Section { part } => part.has_vectors,
        }
    }
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
    pub part: SimilarityComponentView,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarityRoleScoreView {
    pub key: SimilarityComparisonKey,
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

    #[test]
    fn content_scope_is_search_only_and_legacy_links_keep_omitted_scope() {
        for value in [
            json!({"kind":"list_records","filter":null,"search_content":"core"}),
            json!({"kind":"trait_references","trait_key":"trait:fire","query":null,"filter":null,"search_content":"core"}),
            json!({"kind":"text_search","query":"fire","filter":null,"mode":"hybrid","source_sections":"core"}),
        ] {
            assert!(serde_json::from_value::<crate::SearchCriteria>(value).is_err());
        }
        let legacy = serde_json::from_value::<crate::SearchCriteria>(json!({
            "kind":"text_search","query":"fire","filter":null,"mode":"hybrid"
        }))
        .unwrap();
        assert!(matches!(
            legacy,
            crate::SearchCriteria::TextSearch {
                search_content: None,
                ..
            }
        ));
        let core = serde_json::from_value::<crate::SearchCriteria>(json!({
            "kind":"similar_records","seed_record_key":"items:0000000000000000",
            "filter":null,"source_sections":"core","search_content":"all"
        }))
        .unwrap();
        assert!(matches!(
            core,
            crate::SearchCriteria::SimilarRecords {
                source_sections: Some(SimilaritySourceSections::Core),
                search_content: Some(SearchContentScope::All),
                ..
            }
        ));
    }

    #[test]
    fn component_view_has_flat_fields_and_identity_has_no_fake_prose_metadata() {
        let section = SimilarityComponentView::Section {
            part: SimilarityPartView {
                navigation: None,
                selector: SimilarityPartSelector("section".into()),
                label: "Description".into(),
                owner_label: "Creature".into(),
                owners: vec![],
                field: "/system/details/publicNotes".into(),
                section_ordinal: 0,
                role: SimilarityRole::Description,
                role_label: "Description and background".into(),
                has_vectors: true,
                label_only: false,
            },
        };
        let wire = serde_json::to_value(&section).unwrap();
        assert_eq!(wire["kind"], "section");
        assert_eq!(wire["selector"], "section");
        assert!(wire.get("part").is_none());
        assert_eq!(
            serde_json::from_value::<SimilarityComponentView>(wire).unwrap(),
            section
        );
        let identity = SimilarityComponentView::Identity {
            navigation: None,
            selector: SimilarityPartSelector("identity".into()),
            label: "Name and identity".into(),
            owner_label: "Creature".into(),
            has_vectors: true,
        };
        let wire = serde_json::to_value(identity).unwrap();
        for field in ["field", "owners", "section_ordinal", "role", "label_only"] {
            assert!(wire.get(field).is_none(), "identity contains {field}");
        }
    }
}
