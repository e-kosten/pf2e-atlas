#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    pub party_key: String,
    pub name: String,
    pub revision: i64,
    pub members: Vec<PartyMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyMember {
    pub character_key: String,
    pub revision: i64,
    pub imported: bool,
    pub name: String,
    pub level: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct PartyMemberInput {
    pub character_key: String,
}
