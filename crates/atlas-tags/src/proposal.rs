use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

use crate::{TagApplicability, TagId, TagPresentation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologySuggestionFile {
    pub suggestions: Vec<OntologySuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologySuggestion {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_id: Option<TagId>,
    pub label: String,
    pub display: TagPresentation,
    pub applicability: TagApplicability,
    pub rationale: String,
    pub triggering_record_key: RecordKey,
    pub follow_up_research_needed: bool,
}
