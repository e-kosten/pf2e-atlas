use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AddEncounterPartyRequest, CreatePartyRequest, DeletePartyRequest, EncounterMutationView,
    PartyIndexView, PartyMemberView, PartyMutationView, PartyView, UpdatePartyRequest,
};
use atlas_local_state::{
    AddEncounterParticipant, ParticipantHpOrigin, ParticipantKind, ParticipantSide,
    ParticipantVariant, ParticipantVariantOrigin,
};

impl AtlasAppService {
    pub fn parties(&self) -> AppServiceResult<PartyIndexView> {
        self.with_state(|session| {
            Ok(PartyIndexView {
                parties: session
                    .local_state()
                    .parties()
                    .list()?
                    .into_iter()
                    .map(party_view)
                    .collect(),
            })
        })
    }
    pub fn create_party(&self, request: CreatePartyRequest) -> AppServiceResult<PartyView> {
        self.with_write(move |session| {
            Ok(party_view(
                session
                    .local_state()
                    .parties()
                    .create(&request.name, &member_inputs(request.members))?,
            ))
        })
    }
    pub fn update_party(
        &self,
        key: &str,
        request: UpdatePartyRequest,
    ) -> AppServiceResult<PartyView> {
        self.with_write(move |session| {
            Ok(party_view(session.local_state().parties().update(
                key,
                request.expected_revision,
                &request.name,
                &member_inputs(request.members),
            )?))
        })
    }
    pub fn delete_party(
        &self,
        key: &str,
        request: DeletePartyRequest,
    ) -> AppServiceResult<PartyMutationView> {
        self.with_write(move |session| {
            session
                .local_state()
                .parties()
                .delete(key, request.expected_revision)?;
            Ok(PartyMutationView {
                party_key: key.into(),
            })
        })
    }
    pub fn add_encounter_party(
        &self,
        encounter_ref: &str,
        request: AddEncounterPartyRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
            let state = session.local_state();
            let party = state
                .parties()
                .get_at_revision(&request.party_key, request.expected_revision)?;
            let mut selected = std::collections::HashSet::new();
            if request.member_keys.is_empty()
                || request.member_keys.iter().any(|key| {
                    !selected.insert(key) || !party.members.iter().any(|m| &m.member_key == key)
                })
            {
                return Err(AppServiceError::invalid_request(
                    "Select at least one PC from this party, without duplicate members.",
                ));
            }
            let encounter = state.encounters().get(encounter_ref)?.ok_or_else(|| {
                atlas_local_state::LocalStateError::EncounterNotFound(encounter_ref.into())
            })?;
            let participants = party
                .members
                .into_iter()
                .filter(|member| selected.contains(&member.member_key))
                .map(|member| AddEncounterParticipant {
                    spell_resources: vec![],
                    participant_variant: ParticipantVariant::Normal,
                    hp_origin: ParticipantHpOrigin::Unknown,
                    variant_origin: ParticipantVariantOrigin::Explicit,
                    record_key: None,
                    participant_kind: ParticipantKind::Pc,
                    display_name: member.name,
                    record_title_snapshot: None,
                    record_kind_snapshot: None,
                    side: ParticipantSide::Pc,
                    initiative: None,
                    max_hp: None,
                    current_hp: None,
                    temporary_hp: 0,
                    note: None,
                })
                .collect();
            state
                .encounters()
                .add_participants(&encounter.encounter_key, participants)?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }
}
fn member_inputs(
    members: Vec<atlas_app_model::PartyMemberInput>,
) -> Vec<atlas_local_state::PartyMemberInput> {
    members
        .into_iter()
        .map(|m| atlas_local_state::PartyMemberInput {
            member_key: m.member_key,
            name: m.name,
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
                member_key: m.member_key,
                name: m.name,
            })
            .collect(),
    }
}
