use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PartyMemberView {
    pub member_key: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PartyView {
    pub party_key: String,
    pub name: String,
    #[ts(type = "number")]
    pub revision: i64,
    pub members: Vec<PartyMemberView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PartyIndexView {
    pub parties: Vec<PartyView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PartyMemberInput {
    #[serde(default)]
    #[ts(optional)]
    pub member_key: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct CreatePartyRequest {
    pub name: String,
    pub members: Vec<PartyMemberInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct UpdatePartyRequest {
    #[ts(type = "number")]
    pub expected_revision: i64,
    pub name: String,
    pub members: Vec<PartyMemberInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DeletePartyRequest {
    #[ts(type = "number")]
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PartyMutationView {
    pub party_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AddEncounterPartyRequest {
    pub party_key: String,
    #[ts(type = "number")]
    pub expected_revision: i64,
    pub member_keys: Vec<String>,
}
