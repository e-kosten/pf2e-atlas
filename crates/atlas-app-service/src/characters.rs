use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    CharacterIndexView, CharacterMutationView, CharacterPartyTarget, CharacterPartyView,
    CharacterSummaryView, CreateCharacterRequest, DeleteCharacterRequest, UpdateCharacterRequest,
};
use atlas_local_state::{
    Character, CharacterCreationIntent, CharactersRead, LocalStateError, LocalStateWrite,
    PartyMemberInput,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

impl AtlasAppService {
    pub fn characters(&self) -> AppServiceResult<CharacterIndexView> {
        self.with_state(|session| {
            Ok(CharacterIndexView {
                characters: session
                    .local_state()?
                    .characters()
                    .list()?
                    .into_iter()
                    .map(character_summary)
                    .collect(),
            })
        })
    }
    pub fn create_character(
        &self,
        request: CreateCharacterRequest,
    ) -> AppServiceResult<CharacterMutationView> {
        let digest = intent_digest(&(request.name.trim(), request.level, &request.party))?;
        let intent = CharacterCreationIntent::Legacy(&digest);
        self.with_write(|session| {
            let state = session.local_state()?;
            let characters = state.characters();
            if let Some(key) = characters.creation_receipt(&request.request_id, intent)? {
                return creation_view(&characters, &key);
            }
            if let Some(party) = &request.party {
                state
                    .parties()
                    .get_at_revision(&party.party_key, party.expected_revision)?;
            }
            let character = characters.create(&request.name, request.level, None)?;
            join_party(&state, request.party.as_ref(), &character)?;
            characters.record_creation(&request.request_id, intent, &character.character_key)?;
            creation_view(&characters, &character.character_key)
        })
    }
    pub fn update_character(
        &self,
        key: &str,
        request: UpdateCharacterRequest,
    ) -> AppServiceResult<CharacterMutationView> {
        self.with_write(|session| {
            let characters = session.local_state()?.characters();
            let existing = characters.get_at_revision(key, request.expected_revision)?;
            characters.update(
                key,
                request.expected_revision,
                &request.name,
                request.level,
                existing.reference.as_ref(),
            )?;
            creation_view(&characters, key)
        })
    }
    pub fn delete_character(
        &self,
        key: &str,
        request: DeleteCharacterRequest,
    ) -> AppServiceResult<CharacterMutationView> {
        self.with_write(|session| {
            session
                .local_state()?
                .characters()
                .delete(key, request.expected_revision)?;
            Ok(CharacterMutationView {
                character_key: key.into(),
                available: false,
            })
        })
    }
}
pub(super) fn character_summary(
    character: atlas_local_state::CharacterSummary,
) -> CharacterSummaryView {
    CharacterSummaryView {
        character_key: character.character_key,
        name: character.name,
        level: character.level,
        revision: character.revision,
        imported: character.imported,
        parties: character
            .parties
            .into_iter()
            .map(|party| CharacterPartyView {
                party_key: party.party_key,
                name: party.name,
            })
            .collect(),
    }
}
pub(super) fn creation_view(
    characters: &CharactersRead<'_>,
    key: &str,
) -> AppServiceResult<CharacterMutationView> {
    let available = match characters.get(key) {
        Ok(_) => true,
        Err(LocalStateError::CharacterNotFound(_)) => false,
        Err(e) => return Err(e.into()),
    };
    Ok(CharacterMutationView {
        character_key: key.into(),
        available,
    })
}
pub(super) fn join_party(
    state: &LocalStateWrite<'_>,
    target: Option<&CharacterPartyTarget>,
    character: &Character,
) -> AppServiceResult<()> {
    if let Some(target) = target {
        let parties = state.parties();
        let party = parties.get_at_revision(&target.party_key, target.expected_revision)?;
        if party
            .members
            .iter()
            .any(|m| m.character_key == character.character_key)
        {
            return Err(AppServiceError::invalid_request(
                "This character is already in the party.",
            ));
        }
        let mut members: Vec<_> = party
            .members
            .into_iter()
            .map(|m| PartyMemberInput {
                character_key: m.character_key,
            })
            .collect();
        members.push(PartyMemberInput {
            character_key: character.character_key.clone(),
        });
        parties.update(
            &target.party_key,
            target.expected_revision,
            &party.name,
            &members,
        )?;
    }
    Ok(())
}
pub(super) fn intent_digest(value: &impl Serialize) -> AppServiceResult<String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| AppServiceError::invalid_request("Invalid character creation details."))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
