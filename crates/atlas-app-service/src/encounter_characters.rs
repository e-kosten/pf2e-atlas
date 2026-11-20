use crate::{
    AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext, pathbuilder,
};
use atlas_app_model::{
    AddEncounterCharacterRequest, AddEncounterPartyRequest, EncounterCharacterMutationView,
    EncounterCharacterSource, EncounterMutationView,
};
use atlas_local_state::{
    AddEncounterParticipant, Character, ParticipantHpOrigin, ParticipantKind, ParticipantSide,
    ParticipantVariant, ParticipantVariantOrigin, PcAdditionReceipt,
};

impl AtlasAppService {
    pub fn add_encounter_character(
        &self,
        encounter: &str,
        request: AddEncounterCharacterRequest,
    ) -> AppServiceResult<EncounterCharacterMutationView> {
        let (source_digest, imported) = match &request.source {
            EncounterCharacterSource::Existing {
                character_key,
                expected_character_revision,
            } => (
                crate::characters::intent_digest(&(
                    "existing",
                    character_key,
                    expected_character_revision,
                ))?,
                None,
            ),
            EncounterCharacterSource::Manual {
                name,
                level,
                max_hp,
                initiative,
            } => (
                crate::characters::intent_digest(&(
                    "manual",
                    name.trim(),
                    level,
                    max_hp,
                    initiative,
                ))?,
                None,
            ),
            EncounterCharacterSource::Pathbuilder {
                json,
                expected_artifact_generation,
                rule_links,
            } => {
                let imported = pathbuilder::parse(json)?;
                let digest = crate::pc_reference::import_intent_digest(
                    &(
                        "pathbuilder",
                        &imported.name,
                        imported.level,
                        &imported.reference,
                        expected_artifact_generation,
                    ),
                    rule_links,
                )?;
                (
                    digest,
                    Some((imported, expected_artifact_generation.clone(), rule_links)),
                )
            }
        };
        let digest = crate::characters::intent_digest(&(
            source_digest,
            &request.party,
            request.expected_encounter_revision,
        ))?;
        let intent = serde_json::json!({"digest":digest});
        let (encounter_key, recovered) = self.with_state(|session| {
            let state = session.local_state()?;
            let encounters = state.encounters();
            let current = encounters.get(encounter)?.ok_or_else(|| {
                atlas_local_state::LocalStateError::EncounterNotFound(encounter.into())
            })?;
            let recovered = encounters
                .pc_addition_receipt(&current.encounter_key, &request.request_id, &intent)?
                .map(|receipt| addition_view(&state.characters(), &current.encounter_key, &receipt))
                .transpose()?;
            Ok((current.encounter_key, recovered))
        })?;
        if let Some(recovered) = recovered {
            return Ok(recovered);
        }
        let prepared = if let Some((mut imported, expected, rule_links)) = imported {
            let rule_links = rule_links.clone();
            let prepared =
                self.with_workspace(&ReadExecutionContext::uncancellable(), move |session| {
                    crate::pc_reference::require_generation(session, &expected)?;
                    crate::pc_reference::resolve_import(
                        session.retrieval(),
                        &mut imported.reference,
                        rule_links,
                    )?;
                    Ok((imported, session.reference_revision()?))
                })?;
            Some(prepared)
        } else {
            None
        };
        self.with_write(|session| {
            let state = session.local_state()?;
            let encounters = state.encounters();
            let encounter = encounters.get(&encounter_key)?.ok_or_else(|| {
                atlas_local_state::LocalStateError::EncounterNotFound(encounter_key.clone())
            })?;
            if let Some(receipt) = encounters.pc_addition_receipt(
                &encounter.encounter_key,
                &request.request_id,
                &intent,
            )? {
                return addition_view(&state.characters(), &encounter.encounter_key, &receipt);
            }
            require_revision(&encounter, request.expected_encounter_revision)?;
            let characters = state.characters();
            let (character, max_hp, initiative) = match &request.source {
                EncounterCharacterSource::Existing {
                    character_key,
                    expected_character_revision,
                } => (
                    characters.get_at_revision(character_key, *expected_character_revision)?,
                    None,
                    None,
                ),
                EncounterCharacterSource::Manual {
                    name,
                    level,
                    max_hp,
                    initiative,
                } => (characters.create(name, *level, None)?, *max_hp, *initiative),
                EncounterCharacterSource::Pathbuilder { .. } => {
                    let (imported, revision) = prepared.as_ref().ok_or_else(|| {
                        AppServiceError::invalid_request("Review the import before adding it.")
                    })?;
                    session.require_reference_revision(*revision)?;
                    (
                        characters.create(
                            &imported.name,
                            Some(imported.level),
                            Some(&imported.reference),
                        )?,
                        None,
                        None,
                    )
                }
            };
            crate::characters::join_party(&state, request.party.as_ref(), &character)?;
            let participant = encounters.add_participant(
                &encounter.encounter_key,
                participant(&character, max_hp, initiative),
            )?;
            characters.associate_added_pc(
                &encounter.encounter_key,
                &participant.participant_key,
                &character.character_key,
            )?;
            let receipt = PcAdditionReceipt {
                characters: vec![character.character_key],
                participants: vec![participant.participant_key],
            };
            encounters.record_pc_addition(
                &encounter.encounter_key,
                &request.request_id,
                &intent,
                &receipt,
            )?;
            addition_view(&characters, &encounter.encounter_key, &receipt)
        })
    }
    pub fn add_encounter_party(
        &self,
        encounter_ref: &str,
        request: AddEncounterPartyRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let digest = crate::characters::intent_digest(&(
            &request.party_key,
            request.expected_revision,
            request.expected_encounter_revision,
            &request.characters,
        ))?;
        let intent = serde_json::json!({"digest":digest});
        self.with_write(|session| {
            let state = session.local_state()?;
            let encounters = state.encounters();
            let encounter = encounters.get(encounter_ref)?.ok_or_else(|| {
                atlas_local_state::LocalStateError::EncounterNotFound(encounter_ref.into())
            })?;
            if encounters
                .pc_addition_receipt(&encounter.encounter_key, &request.request_id, &intent)?
                .is_some()
            {
                return Ok(EncounterMutationView {
                    encounter_key: encounter.encounter_key,
                });
            }
            require_revision(&encounter, request.expected_encounter_revision)?;
            let party = state
                .parties()
                .get_at_revision(&request.party_key, request.expected_revision)?;
            if party.members.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "Add a PC to this party before adding it to an encounter.",
                ));
            }
            if request.characters.len() != party.members.len() {
                return Err(AppServiceError::invalid_request(
                    "Review the current party characters before adding them.",
                ));
            }
            let characters = state.characters();
            let mut added = PcAdditionReceipt {
                characters: Vec::new(),
                participants: Vec::new(),
            };
            for (member, reviewed) in party.members.into_iter().zip(&request.characters) {
                if member.character_key != reviewed.character_key {
                    return Err(AppServiceError::invalid_request(
                        "The party order changed. Review its characters.",
                    ));
                }
                let character = characters
                    .get_at_revision(&reviewed.character_key, reviewed.expected_revision)?;
                let pc = encounters.add_participant(
                    &encounter.encounter_key,
                    participant(&character, None, None),
                )?;
                characters.associate_added_pc(
                    &encounter.encounter_key,
                    &pc.participant_key,
                    &character.character_key,
                )?;
                added.characters.push(character.character_key);
                added.participants.push(pc.participant_key);
            }
            encounters.record_pc_addition(
                &encounter.encounter_key,
                &request.request_id,
                &intent,
                &added,
            )?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }
}
fn require_revision(
    encounter: &atlas_local_state::Encounter,
    expected: i64,
) -> AppServiceResult<()> {
    if encounter.revision != expected {
        return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
            encounter_key: encounter.encounter_key.clone(),
            expected_revision: expected,
            actual_revision: encounter.revision,
        }
        .into());
    }
    Ok(())
}
fn participant(
    character: &Character,
    max_hp: Option<i64>,
    initiative: Option<i64>,
) -> AddEncounterParticipant {
    AddEncounterParticipant {
        level_override: character.level,
        spell_resources: Vec::new(),
        participant_variant: ParticipantVariant::Normal,
        hp_origin: if max_hp.is_some() {
            ParticipantHpOrigin::Explicit
        } else {
            ParticipantHpOrigin::Unknown
        },
        variant_origin: ParticipantVariantOrigin::Explicit,
        record_key: None,
        participant_kind: ParticipantKind::Pc,
        display_name: character.name.clone(),
        record_title_snapshot: None,
        record_kind_snapshot: None,
        side: ParticipantSide::Pc,
        initiative,
        max_hp,
        current_hp: max_hp,
        temporary_hp: 0,
        note: None,
    }
}

fn addition_view(
    characters: &atlas_local_state::CharactersRead<'_>,
    encounter: &str,
    receipt: &PcAdditionReceipt,
) -> AppServiceResult<EncounterCharacterMutationView> {
    if receipt.characters.len() != 1 || receipt.participants.len() != 1 {
        return Err(AppServiceError::invalid_request(
            "Stored PC addition has an invalid outcome.",
        ));
    }
    let character_key = receipt.characters[0].clone();
    let participant_key = receipt.participants[0].clone();
    let available = match characters.encounter_character(encounter, &participant_key) {
        Ok(key) => key.as_deref() == Some(character_key.as_str()),
        Err(atlas_local_state::LocalStateError::ParticipantNotFound(_)) => false,
        Err(error) => return Err(error.into()),
    };
    Ok(EncounterCharacterMutationView {
        encounter_key: encounter.into(),
        character_key,
        participant_key,
        available,
    })
}
