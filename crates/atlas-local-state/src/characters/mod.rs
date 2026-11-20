mod service;
pub use service::{CharactersRead, CharactersWrite};
pub(crate) mod reference;
pub use reference::{
    MAX_PC_ABILITIES, MAX_PC_ABILITY_LABEL, PcAbility, PcAbilityKind, PcReference,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterParty {
    pub party_key: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    pub character_key: String,
    pub name: String,
    pub level: Option<i64>,
    pub revision: i64,
    pub reference: Option<crate::PcReference>,
    pub parties: Vec<CharacterParty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterSummary {
    pub character_key: String,
    pub name: String,
    pub level: Option<i64>,
    pub revision: i64,
    pub imported: bool,
    pub parties: Vec<CharacterParty>,
}
