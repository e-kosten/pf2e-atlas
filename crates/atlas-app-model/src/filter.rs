//! The editor sends the shared typed predicate; CEL is an independent CLI surface.
use crate::RetrievalModeView;
use atlas_domain::{
    QueryError, QueryFacetOutcome, QueryFieldDefinition, QueryLimits, QueryLiteral, QueryPredicate,
    QueryValueOptions, QueryVocabularyBasis,
};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FilterDiscoveryContext {
    Filtered {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        connection_scope: Option<crate::SearchConnectionScope>,
        #[serde(default)]
        search_content: Option<atlas_domain::SearchContentScope>,
        search_profile: Option<atlas_domain::SearchProfileId>,
        expected_artifact_generation: Option<String>,
        filter: Option<QueryPredicate>,
        text: Option<String>,
        mode: RetrievalModeView,
    },
    SavedList {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        connection_scope: Option<crate::SearchConnectionScope>,
        #[serde(default)]
        search_content: Option<atlas_domain::SearchContentScope>,
        search_profile: Option<atlas_domain::SearchProfileId>,
        expected_artifact_generation: Option<String>,
        list_ref: String,
        filter: Option<QueryPredicate>,
        text: Option<String>,
        mode: RetrievalModeView,
    },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoverFilterEditorRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_artifact_generation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<QueryPredicate>,
    #[serde(default)]
    pub selected_field_ids: Vec<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiscoverFilterValuesRequest {
    Contextual {
        #[serde(default)]
        only_matching: bool,
        context: Box<FilterDiscoveryContext>,
        field_id: String,
        clause_id: Option<String>,
        text: Option<String>,
        #[serde(default)]
        offset: usize,
        limit: usize,
    },
    Vocabulary {
        scope: FilterVocabularyScope,
        field_id: String,
        text: Option<String>,
        #[serde(default)]
        offset: usize,
        limit: usize,
        #[serde(default)]
        selected_values: Vec<QueryLiteral>,
    },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FilterVocabularyScope {
    Artifact,
    SavedList { list_ref: String },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverFilterCountsRequest {
    pub context: FilterDiscoveryContext,
    pub field_id: String,
    pub clause_id: Option<String>,
    pub values: Vec<QueryLiteral>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEditorView {
    pub artifact_generation: String,
    pub relationship_clauses: Vec<crate::RelationshipClausePresentationView>,
    pub catalog_version: u32,
    pub limits: QueryLimits,
    pub groups: Vec<FilterEditorGroupView>,
    pub common_field_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub sense_qualifiers: Option<Vec<SenseQualifierView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseQualifierView {
    pub sense: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub acuity: Option<String>,
    pub has_range: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEditorGroupView {
    pub id: String,
    pub label: String,
    pub fields: Vec<FilterEditorFieldView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEditorFieldView {
    pub definition: QueryFieldDefinition,
    pub control: FilterControlView,
    pub placement: FilterFieldPlacement,
    pub label: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub help: Option<String>,
    pub choices: Vec<FilterChoiceView>,
    // Short lists retain zero-match choices; searchable lists narrow before paging.
    pub short_choices: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub allow_custom_values: Option<bool>,
    pub applicable: bool,
    pub unit_fields: Vec<FilterUnitFieldView>,
    pub operators: Vec<FilterOperatorView>,
    // Compact editor for collections whose entries have a category and optional amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub collection_control: Option<FilterCollectionControlView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterUnitFieldView {
    pub field_id: String,
    pub unit: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCollectionControlView {
    pub category_field_id: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub amount_field_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub qualifier_field_id: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterChoiceView {
    pub value: QueryLiteral,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub help: Option<crate::ReferenceHelpView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterOperatorView {
    pub operator: String,
    pub label: String,
    pub operand: FilterOperandView,
}
/// The editor input required by an executable operator.
/// Field definitions retain literal types and allowed choices.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperandView {
    Scalar,
    List,
    Range,
    State,
    Predicate,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterControlView {
    Text,
    Numeric,
    Boolean,
    Set,
    Collection,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterFieldPlacement {
    InitiallyVisible,
    Addable,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterValidationResult {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub predicate: Option<QueryPredicate>,
    pub errors: Vec<QueryError>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FilterValueListView {
    Contextual {
        outcome: QueryFacetOutcome<FilterValueOptionsView>,
        search_profile: atlas_domain::SearchProfileId,
        artifact_generation: String,
    },
    Vocabulary {
        scope: FilterVocabularyScope,
        values: FilterVocabularyView,
    },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCountsView {
    pub search_profile: atlas_domain::SearchProfileId,
    pub artifact_generation: String,
    pub counts: QueryFacetOutcome<QueryValueOptions>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterValueOptionView {
    pub value: QueryLiteral,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub help: Option<crate::ReferenceHelpView>,
    pub distinct_roots: u64,
    pub selected: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterValueOptionsView {
    pub field: String,
    pub options: Vec<FilterValueOptionView>,
    pub total_values: u64,
    pub exhaustive: bool,
    pub count_basis: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterVocabularyView {
    pub field: String,
    pub options: Vec<FilterChoiceView>,
    pub selected_values: Vec<QueryLiteral>,
    pub total_values: u64,
    pub basis: QueryVocabularyBasis,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn editor_catalog_request_has_no_query_context() {
        let request: DiscoverFilterEditorRequest = serde_json::from_value(json!({})).unwrap();
        assert!(request.selected_field_ids.is_empty());
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({"selected_field_ids":[]})
        );
        assert!(
            serde_json::from_value::<DiscoverFilterEditorRequest>(json!({
                "context":{"kind":"filtered","filter":null,"text":null,"mode":"hybrid"}
            }))
            .is_err()
        );
    }
    #[test]
    fn operator_contract_preserves_operand_shapes_in_json() {
        for (operand, name) in [
            (FilterOperandView::Scalar, "scalar"),
            (FilterOperandView::List, "list"),
            (FilterOperandView::Range, "range"),
            (FilterOperandView::State, "state"),
            (FilterOperandView::Predicate, "predicate"),
        ] {
            let view = FilterOperatorView {
                operator: "example".into(),
                label: "Example".into(),
                operand,
            };
            let encoded = serde_json::to_value(&view).unwrap();
            assert_eq!(
                encoded,
                json!({"operator":"example","label":"Example","operand":name})
            );
            assert_eq!(
                serde_json::from_value::<FilterOperatorView>(encoded).unwrap(),
                view
            );
        }
    }
    #[test]
    fn vocabulary_contract_cannot_silently_drop_query_context() {
        let input = json!({"kind":"vocabulary","scope":{"kind":"artifact"},"field_id":"traits","text":"fire","limit":20});
        let request: DiscoverFilterValuesRequest = serde_json::from_value(input.clone()).unwrap();
        assert!(
            matches!(request,DiscoverFilterValuesRequest::Vocabulary {selected_values,offset:0,..} if selected_values.is_empty())
        );
        let mut malformed = input;
        malformed["context"] =
            json!({"kind":"filtered","filter":null,"text":"dragon","mode":"hybrid"});
        assert!(serde_json::from_value::<DiscoverFilterValuesRequest>(malformed).is_err());
        assert!(serde_json::from_value::<DiscoverFilterValuesRequest>(json!({"context":{"kind":"filtered","filter":null,"text":null,"mode":"hybrid"},"field_id":"traits","limit":20})).is_err());
    }
    #[test]
    fn optional_contextual_unavailability_has_no_fabricated_counts() {
        let response = FilterCountsView {
            search_profile: atlas_domain::SearchProfileId::new("bge-base"),
            artifact_generation: "fixture-generation".into(),
            counts: QueryFacetOutcome::Unavailable {
                reason: atlas_domain::FacetUnavailableReason::NonConjunctiveContext,
            },
        };
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({"search_profile":"bge-base","artifact_generation":"fixture-generation","counts":{"status":"unavailable","reason":"non_conjunctive_context"}})
        );
    }
}
