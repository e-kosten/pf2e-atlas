//! Small saved edits over the existing guarded character and party workflows.
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    CharacterLevelUpdate, CharacterMutationView, PartyMemberInput, PartyView, UpdatePartyRequest,
};
use std::collections::HashSet;

impl AtlasAppService {
    pub fn edit_character_metadata(
        &self,
        key: &str,
        revision: i64,
        name: Option<String>,
        level: Option<Option<i64>>,
    ) -> AppServiceResult<CharacterMutationView> {
        self.with_write(|session| {
            session.local_state()?.characters().update_metadata(
                key,
                revision,
                name.as_deref(),
                level,
            )?;
            Ok(CharacterMutationView {
                character_key: key.into(),
                available: true,
            })
        })
    }

    pub fn edit_party_name(
        &self,
        key: &str,
        revision: i64,
        name: String,
    ) -> AppServiceResult<PartyView> {
        self.with_write(|state| {
            Ok(crate::parties::party_view(
                state
                    .local_state()?
                    .parties()
                    .rename(key, revision, &name)?,
            ))
        })
    }

    pub fn add_party_members(
        &self,
        key: &str,
        revision: i64,
        members: Vec<PartyMemberInput>,
    ) -> AppServiceResult<PartyView> {
        if members.is_empty() {
            return Err(AppServiceError::invalid_request(
                "Choose at least one character to add.",
            ));
        }
        let mut request = self.party_edit_snapshot(key, revision)?;
        request.members.extend(members);
        self.update_party(key, request)
    }

    pub fn remove_party_members(
        &self,
        key: &str,
        revision: i64,
        keys: Vec<String>,
    ) -> AppServiceResult<PartyView> {
        let mut request = self.party_edit_snapshot(key, revision)?;
        let distinct: HashSet<_> = keys.iter().collect();
        if keys.is_empty()
            || distinct.len() != keys.len()
            || keys
                .iter()
                .any(|key| !request.members.iter().any(|m| &m.character_key == key))
        {
            return Err(AppServiceError::invalid_request(
                "Choose distinct characters from this party to remove.",
            ));
        }
        request.members.retain(|m| !keys.contains(&m.character_key));
        self.update_party(key, request)
    }

    pub fn reorder_party_members(
        &self,
        key: &str,
        revision: i64,
        members: Vec<PartyMemberInput>,
    ) -> AppServiceResult<PartyView> {
        let mut request = self.party_edit_snapshot(key, revision)?;
        let keys: HashSet<_> = members.iter().map(|m| &m.character_key).collect();
        if members.len() != request.members.len()
            || keys.len() != members.len()
            || request
                .members
                .iter()
                .any(|m| !keys.contains(&m.character_key))
        {
            return Err(AppServiceError::invalid_request(
                "Supply every party character once, in the desired order.",
            ));
        }
        request.members = members;
        self.update_party(key, request)
    }

    pub fn set_party_character_levels(
        &self,
        key: &str,
        revision: i64,
        members: Vec<PartyMemberInput>,
        level: Option<i64>,
    ) -> AppServiceResult<PartyView> {
        if members.is_empty() {
            return Err(AppServiceError::invalid_request(
                "Choose at least one character whose level should change.",
            ));
        }
        let mut request = self.party_edit_snapshot(key, revision)?;
        request.level_updates = members
            .into_iter()
            .map(|m| CharacterLevelUpdate {
                character_key: m.character_key,
                expected_character_revision: m.expected_character_revision,
                level,
            })
            .collect();
        self.update_party(key, request)
    }

    fn party_edit_snapshot(
        &self,
        key: &str,
        revision: i64,
    ) -> AppServiceResult<UpdatePartyRequest> {
        self.with_state(|state| {
            let party = state
                .local_state()?
                .parties()
                .get_at_revision(key, revision)?;
            Ok(UpdatePartyRequest {
                expected_revision: revision,
                name: party.name,
                level_updates: Vec::new(),
                members: party
                    .members
                    .into_iter()
                    .map(|m| PartyMemberInput {
                        character_key: m.character_key,
                        expected_character_revision: m.revision,
                    })
                    .collect(),
            })
        })
    }
}
