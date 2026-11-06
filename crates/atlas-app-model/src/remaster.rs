//! Source-attributed current change information, independent of record identity.
use crate::{RecordNavigationView, RecordSummaryView, RelatedCategoryView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub(crate) const fn default_prefer_remaster() -> bool {
    true
}
fn default_limit() -> usize {
    20
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct RemasterRelationshipsRequest {
    pub subject: RemasterSubjectRequest,
    #[serde(default = "default_limit")]
    #[ts(as = "Option<usize>", optional)]
    pub limit: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub expected_artifact_generation: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RemasterSubjectRequest {
    Record { selection: RecordNavigationView },
    Term { term: String },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterRelationshipsResponse {
    pub artifact_generation: String,
    pub subject: RemasterSubjectView,
    pub relationships: RelatedCategoryView<RemasterRelationshipsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_remaster_context: Option<RelatedCategoryView<RemasterSourceContextView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemasterSubjectView {
    Record {
        selection: RecordNavigationView,
        title: Option<String>,
        available: bool,
    },
    Term {
        query: String,
        normalized_term: String,
        scope_label: String,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterRelationshipsView {
    pub relationships: Vec<RemasterRelationshipView>,
    pub declarations: Vec<RemasterDeclarationView>,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterRelationshipView {
    pub id: String,
    pub label: String,
    pub parts: Vec<RemasterInlinePartView>,
    pub declaration_ids: Vec<String>,
    pub explanation_links: Vec<RemasterExplanationLinkView>,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterExplanationLinkView {
    pub label: String,
    pub navigation: RecordNavigationView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemasterInlinePartView {
    Text {
        text: String,
    },
    Reference {
        label: String,
        navigation: RecordNavigationView,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterDeclarationView {
    pub id: String,
    pub kinds: Vec<RemasterChangeKindView>,
    pub authored_status: Option<String>,
    pub context: Vec<String>,
    pub before: Vec<RemasterSideView>,
    pub after: Vec<RemasterSideView>,
    pub origins: Vec<RemasterCitationOriginView>,
    pub citations: Vec<RemasterCitationView>,
    pub citations_truncated: bool,
    pub truncated: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RemasterChangeKindView {
    Renamed,
    Merged,
    Replaced,
    Removed,
    MechanicsChanged,
    Discussion,
    Unclassified,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterSideView {
    pub text: String,
    pub terms: Vec<String>,
    pub endpoints: Vec<RemasterEndpointView>,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterEndpointView {
    pub label: String,
    pub context: Option<String>,
    pub resolution: RemasterEndpointResolutionView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemasterEndpointResolutionView {
    Current {
        record: RecordSummaryView,
        navigation: RecordNavigationView,
    },
    Absent {
        record_key: String,
        message: String,
    },
    Unresolved {
        reference: Option<String>,
        message: String,
    },
    Ambiguous {
        candidates: Vec<RemasterEndpointCandidateView>,
        truncated: bool,
        message: String,
    },
    Term {
        message: Option<String>,
    },
    SourceText,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterEndpointCandidateView {
    pub record: RecordSummaryView,
    pub navigation: RecordNavigationView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterCitationView {
    pub origin: RemasterCitationOriginView,
    pub attribution: String,
    pub revision: String,
    pub statement: String,
    pub shortened: bool,
    pub destination: RemasterCitationDestinationView,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RemasterCitationOriginView {
    Foundry,
    AtlasReviewed,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemasterCitationDestinationView {
    Internal { navigation: RecordNavigationView },
    External { url: String },
    Attribution { reason: Option<String> },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterSourceContextView {
    pub heading: String,
    pub source: RemasterEndpointView,
    pub relationships: RelatedCategoryView<RemasterRelationshipsView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterLookupContextView {
    pub label: String,
    pub supports: Vec<RemasterLookupSupportView>,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterLookupSupportView {
    pub declaration_id: String,
    pub kind_labels: Vec<String>,
    pub context: Vec<String>,
    pub citations: Vec<RemasterCitationView>,
    pub citations_truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterAliasMatchView {
    pub matched_text: String,
    pub lookup_context: RemasterLookupContextView,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_term_request_defaults_and_bounds_are_not_identity_lookup() {
        let value: RemasterRelationshipsRequest = serde_json::from_value(serde_json::json!({
            "subject":{"kind":"term","term":"old name"}
        }))
        .unwrap();
        assert_eq!(value.limit, 20);
        assert!(matches!(value.subject, RemasterSubjectRequest::Term { .. }));
    }
}

#[cfg(test)]
mod preference_tests {
    use crate::{FilterDiscoveryContext, SearchResultsRequest, TraitRecordsRequest};
    use serde_json::json;
    #[test]
    fn preference_omission_is_true_false_is_preserved_and_null_is_rejected() {
        let search =
            json!({"criteria":{"kind":"list_records","filter":null},"page":{"number":1,"size":20}});
        let filtered = json!({"kind":"filtered","filter":null,"text":null,"mode":"lexical"});
        let similar = json!({"trait_key":"trait:fire","kind":"similar","page":{"number":1,"size":20},"expected_artifact_generation":"generation","search_profile":null,"ranking":null});
        assert!(
            serde_json::from_value::<SearchResultsRequest>(search.clone())
                .unwrap()
                .prefer_remaster
        );
        assert!(
            serde_json::from_value::<TraitRecordsRequest>(similar.clone())
                .unwrap()
                .prefer_remaster
        );
        assert!(matches!(
            serde_json::from_value::<FilterDiscoveryContext>(filtered.clone()).unwrap(),
            FilterDiscoveryContext::Filtered {
                prefer_remaster: true,
                ..
            }
        ));
        for value in [false, true] {
            let mut input = search.clone();
            input["prefer_remaster"] = json!(value);
            assert_eq!(
                serde_json::from_value::<SearchResultsRequest>(input)
                    .unwrap()
                    .prefer_remaster,
                value
            );
        }
        for invalid in [json!(null), json!("false"), json!(0)] {
            let mut input = search.clone();
            input["prefer_remaster"] = invalid.clone();
            assert!(serde_json::from_value::<SearchResultsRequest>(input).is_err());
            let mut input = filtered.clone();
            input["prefer_remaster"] = invalid.clone();
            assert!(serde_json::from_value::<FilterDiscoveryContext>(input).is_err());
            let mut input = similar.clone();
            input["prefer_remaster"] = invalid;
            assert!(serde_json::from_value::<TraitRecordsRequest>(input).is_err());
        }
    }
}
