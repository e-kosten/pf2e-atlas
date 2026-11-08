//! Contextual rule help over existing records and exact vocabulary identities.
use crate::RecordNavigationView;
use atlas_domain::QueryPredicate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A source token and its supplied presentation; the label never becomes identity.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceValueView {
    pub value: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub help: Option<ReferenceHelpView>,
}
impl From<String> for ReferenceValueView {
    fn from(value: String) -> Self {
        Self {
            label: value.clone(),
            value,
            help: None,
        }
    }
}
impl From<&str> for ReferenceValueView {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceDestinationView {
    Record {
        navigation: Box<RecordNavigationView>,
    },
    Definition {
        key: String,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceHelpView {
    /// Property identity is independent of a shared real-record destination.
    #[cfg_attr(feature = "openapi", schema(required))]
    pub definition_key: Option<String>,
    pub destination: ReferenceDestinationView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceFilterActionView {
    pub label: String,
    pub predicate: QueryPredicate,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DefinitionDetailRequest {
    pub definition_key: String,
    pub expected_artifact_generation: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionDetailView {
    pub artifact_generation: String,
    pub definition_key: String,
    pub label: String,
    pub help: Option<ReferenceHelpView>,
    /// Record-backed entries retain a destination, without a second body.
    pub content: Option<ReferenceContentView>,
    pub publications: Vec<DefinitionPublicationView>,
    pub notices: Vec<DefinitionNoticeView>,
    pub related_filters: Vec<ReferenceFilterActionView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionPublicationView {
    pub title: Option<String>,
    pub page: Option<u32>,
    pub license: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionNoticeView {
    pub id: String,
    pub text: String,
    pub source_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn raw_token_identity_and_optional_help_are_distinct_from_labels() {
        let value = ReferenceValueView {
            value: "chthonian".into(),
            label: "Chthonian".into(),
            help: None,
        };
        assert_eq!(
            serde_json::to_value(&value).unwrap(),
            json!({"value":"chthonian","label":"Chthonian"})
        );
        assert_eq!(
            serde_json::from_value::<ReferenceValueView>(
                json!({"value":"custom","label":"Custom"})
            )
            .unwrap()
            .help,
            None
        );
    }
    #[test]
    fn reading_help_requires_a_destination() {
        assert!(
            serde_json::from_value::<ReferenceHelpView>(json!({
                "definition_key":"language:common", "destination":null
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<DefinitionDetailRequest>(json!({
                "definition_key":"language:common", "record_key":"fake"
            }))
            .is_err()
        );
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceContentView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub section_rendering: Option<crate::PreparedSectionRenderingView>,
    pub body: crate::PreparedFieldBodyView,
    pub references: Vec<crate::PreparedReferenceView>,
}
