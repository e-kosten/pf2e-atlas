#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    pub party_key: String,
    pub name: String,
    pub revision: i64,
    pub members: Vec<PartyMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyMember {
    pub member_key: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PartyMemberInput {
    pub member_key: Option<String>,
    pub name: String,
}
