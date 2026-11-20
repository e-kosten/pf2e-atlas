use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    CreatePartyRequest, DeletePartyRequest, PartyIndexView, PartyMemberView, PartyMutationView,
    PartyView, UpdatePartyRequest,
};

impl AtlasAppService {
    pub fn parties(&self) -> AppServiceResult<PartyIndexView> {
        self.with_state(|session| {
            Ok(PartyIndexView {
                parties: session
                    .local_state()?
                    .parties()
                    .list()?
                    .into_iter()
                    .map(party_view)
                    .collect(),
            })
        })
    }
    pub fn create_party(&self, request: CreatePartyRequest) -> AppServiceResult<PartyView> {
        self.with_write(|session| {
            let state = session.local_state()?;
            let members = member_inputs(&state.characters(), request.members)?;
            Ok(party_view(state.parties().create(&request.name, &members)?))
        })
    }
    pub fn update_party(
        &self,
        key: &str,
        request: UpdatePartyRequest,
    ) -> AppServiceResult<PartyView> {
        self.with_write(|session| {
            let state = session.local_state()?;
            state
                .parties()
                .get_at_revision(key, request.expected_revision)?;
            let members = member_inputs(&state.characters(), request.members)?;
            let mut changed = std::collections::HashSet::new();
            for update in request.level_updates {
                if !changed.insert(update.character_key.clone()) {
                    return Err(AppServiceError::invalid_request(
                        "Character level was supplied twice.",
                    ));
                }
                if !members
                    .iter()
                    .any(|m| m.character_key == update.character_key)
                {
                    return Err(AppServiceError::invalid_request(
                        "The level update must belong to this party.",
                    ));
                }
                let current = state
                    .characters()
                    .get_at_revision(&update.character_key, update.expected_character_revision)?;
                state.characters().update(
                    &update.character_key,
                    update.expected_character_revision,
                    &current.name,
                    update.level,
                    current.reference.as_ref(),
                )?;
            }
            Ok(party_view(state.parties().update(
                key,
                request.expected_revision,
                &request.name,
                &members,
            )?))
        })
    }
    pub fn delete_party(
        &self,
        key: &str,
        request: DeletePartyRequest,
    ) -> AppServiceResult<PartyMutationView> {
        self.with_write(|session| {
            session
                .local_state()?
                .parties()
                .delete(key, request.expected_revision)?;
            Ok(PartyMutationView {
                party_key: key.into(),
            })
        })
    }
}
fn member_inputs(
    characters: &atlas_local_state::CharactersRead<'_>,
    members: Vec<atlas_app_model::PartyMemberInput>,
) -> AppServiceResult<Vec<atlas_local_state::PartyMemberInput>> {
    members
        .into_iter()
        .map(|m| {
            let character =
                characters.get_at_revision(&m.character_key, m.expected_character_revision)?;
            Ok(atlas_local_state::PartyMemberInput {
                character_key: character.character_key,
            })
        })
        .collect()
}
fn party_view(party: atlas_local_state::Party) -> PartyView {
    PartyView {
        party_key: party.party_key,
        name: party.name,
        revision: party.revision,
        members: party
            .members
            .into_iter()
            .map(|m| PartyMemberView {
                character_key: m.character_key,
                name: m.name,
                level: m.level,
                character_revision: m.revision,
                imported: m.imported,
            })
            .collect(),
    }
}
