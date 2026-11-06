//! Contextual rule help over existing records and exact vocabulary identities.
use crate::RecordNavigationView;
use atlas_domain::{QueryDefinitionReferenceTarget, QueryFieldDefinition, QueryPredicate};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A source token and its supplied presentation; the label never becomes identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceValueView {
    pub value: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionAvailabilityView {
    Available,
    LabelOnly,
    MissingLocalizedBody,
    MissingTarget,
    WrongFamily,
    ConflictingEvidence,
    AmbiguousMapping,
    SourceInapplicable,
    PermissionUnresolved,
    Withdrawn,
    UnavailableUnknown,
    SetupRequired,
    NotInCatalog,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceHelpView {
    /// Property identity is independent of a shared real-record destination.
    pub definition_key: Option<String>,
    pub availability: DefinitionAvailabilityView,
    pub destination: Option<ReferenceDestinationView>,
    pub related_filters: Vec<ReferenceFilterActionView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceFilterActionView {
    Search {
        label: String,
        predicate: QueryPredicate,
        artifact_generation: String,
    },
    ConfigureField {
        label: String,
        field: QueryFieldDefinition,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DefinitionDetailRequest {
    pub definition_key: String,
    pub expected_artifact_generation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionDetailView {
    pub artifact_generation: String,
    pub definition_key: String,
    pub label: String,
    pub help: ReferenceHelpView,
    /// Record-backed entries retain a destination, without a second body.
    pub content: Option<ReferenceContentView>,
    pub attribution: Option<DefinitionAttributionView>,
    pub notices: Vec<DefinitionNoticeView>,
    pub references: Vec<DefinitionReferenceView>,
    pub references_availability: DefinitionAvailabilityView,
    pub references_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionReferenceView {
    pub target: QueryDefinitionReferenceTarget,
    pub destination: Option<ReferenceDestinationView>,
    pub occurrence_path: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionAttributionView {
    pub source_package: String,
    pub source_repository: Option<String>,
    pub source_revision: Option<String>,
    pub source_file: Option<String>,
    pub source_sha256: Option<String>,
    pub label_key: Option<String>,
    pub description_key: Option<String>,
    pub catalogs: Vec<String>,
    pub source_locations: Vec<DefinitionSourceLocationView>,
    pub intended_record_key: Option<String>,
    pub indexing_locale: String,
    pub content_locale: Option<String>,
    pub publications: Vec<DefinitionPublicationView>,
    pub permission_disposition: DefinitionPermissionDispositionView,
    pub body_grants: Vec<DefinitionGrantView>,
    pub label_grants: Vec<DefinitionGrantView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionSourceLocationView {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionPermissionDispositionView {
    ExistingTraitPolicy,
    ExistingRecordPolicy,
    ApprovedContent,
    MetadataOnlyUnresolved,
    ExcludedOrWithdrawn,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionPublicationView {
    pub title: Option<String>,
    pub page: Option<u32>,
    pub license: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionGrantView {
    pub license: String,
    pub scope: DefinitionGrantScopeView,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionGrantScopeView {
    FunctionalContent,
    DescriptiveReferences,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionNoticeView {
    pub id: String,
    pub sha256: String,
    pub text: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DefinitionFilterOptionsRequest {
    pub text: Option<String>,
    #[serde(default)]
    pub offset: usize,
    pub limit: usize,
    #[serde(default)]
    pub selected: Vec<QueryDefinitionReferenceTarget>,
    pub expected_artifact_generation: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionFilterOptionsView {
    pub artifact_generation: String,
    pub options: Vec<DefinitionFilterOptionView>,
    pub selected: Vec<DefinitionFilterOptionView>,
    pub total: u64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DefinitionFilterOptionView {
    pub target: QueryDefinitionReferenceTarget,
    pub label: String,
    pub help: ReferenceHelpView,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn outgoing_reference_preserves_identity_without_reading_capability() {
        let reference: DefinitionReferenceView = serde_json::from_value(json!({
            "target":{"kind":"definition","key":"rule:pf2e:Fixture.Rule.Hidden"},
            "destination":null, "occurrence_path":"/0", "label":null
        }))
        .unwrap();
        let encoded = serde_json::to_value(reference).unwrap();
        assert_eq!(
            encoded["target"],
            json!({"kind":"definition","key":"rule:pf2e:Fixture.Rule.Hidden"})
        );
        assert_eq!(encoded.get("destination"), Some(&serde_json::Value::Null));
    }

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
    fn detail_unknown_and_setup_states_do_not_invent_destinations() {
        for availability in [
            DefinitionAvailabilityView::UnavailableUnknown,
            DefinitionAvailabilityView::SetupRequired,
        ] {
            let help = ReferenceHelpView {
                definition_key: Some("language:custom".into()),
                availability,
                destination: None,
                related_filters: vec![],
            };
            let encoded = serde_json::to_value(&help).unwrap();
            assert_eq!(encoded["destination"], serde_json::Value::Null);
            assert_eq!(
                serde_json::from_value::<ReferenceHelpView>(encoded).unwrap(),
                help
            );
        }
        assert!(serde_json::from_value::<DefinitionDetailRequest>(json!({"definition_key":"language:common","expected_artifact_generation":null,"record_key":"fake"})).is_err());
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceContentView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub section_rendering: Option<crate::PreparedSectionRenderingView>,
    pub body: crate::PreparedFieldBodyView,
    pub references: Vec<crate::PreparedReferenceView>,
}
