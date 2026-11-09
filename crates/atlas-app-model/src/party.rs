use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartyMemberView {
    pub member_key: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub level: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartyView {
    pub party_key: String,
    pub name: String,
    pub revision: i64,
    pub members: Vec<PartyMemberView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartyIndexView {
    pub parties: Vec<PartyView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartyMemberInput {
    #[serde(default)]
    pub member_key: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePartyRequest {
    pub name: String,
    pub members: Vec<PartyMemberInput>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePartyRequest {
    pub expected_revision: i64,
    pub name: String,
    pub members: Vec<PartyMemberInput>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePartyRequest {
    pub expected_revision: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartyMutationView {
    pub party_key: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddEncounterPartyRequest {
    pub party_key: String,
    pub expected_revision: i64,
}
