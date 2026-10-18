use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    ApplyEncounterSpellActionRequest, EncounterMutationView, EncounterSpellAction,
};
use atlas_local_state::SpellResource;

impl AtlasAppService {
    pub fn apply_encounter_spell_action(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterSpellActionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let store = self.local_state_store()?;
        let encounter = store
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        let participant = encounter
            .participants
            .iter()
            .find(|p| p.participant_key == request.participant_key)
            .ok_or_else(|| {
                AppServiceError::invalid_request("Participant is no longer in this encounter")
            })?;
        if participant.revision != request.expected_revision {
            return Err(
                atlas_local_state::LocalStateError::StaleParticipantRevision {
                    participant_key: request.participant_key,
                    expected_revision: request.expected_revision,
                    actual_revision: participant.revision,
                }
                .into(),
            );
        }
        let record_key = participant.record_key.as_deref().ok_or_else(|| {
            AppServiceError::invalid_request("This participant has no spell source")
        })?;
        let record = super::hydration::resolve_record_ref(self, record_key)?;
        let catalog = super::spell_catalog::catalog(
            &record.source,
            &participant.spell_resources,
            &participant.expended_spells,
        );
        // Retain saved pools even when the current source no longer exposes them.
        let mut resources = participant.spell_resources.clone();
        for resource in &catalog.resources {
            if !resources.iter().any(|s| s.key == resource.key) {
                resources.push(SpellResource {
                    key: resource.key.clone(),
                    capacity: resource.capacity,
                    remaining: resource.remaining,
                    undo_remaining: None,
                });
            }
        }
        match request.action {
            EncounterSpellAction::SetExpended {
                selection,
                expended,
            } => {
                if selection.record_key != record_key
                    || selection.field.is_some()
                    || selection.passage.is_some()
                {
                    return Err(AppServiceError::invalid_request(
                        "Select this participant's spell",
                    ));
                }
                let spell = catalog
                    .spells
                    .iter()
                    .find(|s| s.navigation.owners == selection.owners && s.expended.is_some())
                    .ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "This spell does not use an expenditure marker",
                        )
                    })?;
                let atlas_record::source_content::OwnedContentIdentity::Stable(id) =
                    &spell.navigation.owners[0].identity
                else {
                    return Err(AppServiceError::invalid_request(
                        "Spell identity unavailable",
                    ));
                };
                store.encounters().set_spell_expended(
                    &participant.participant_key,
                    request.expected_revision,
                    id,
                    expended,
                )?;
                return Ok(EncounterMutationView {
                    encounter_key: encounter.encounter.encounter_key,
                });
            }
            EncounterSpellAction::Cast { selection } => {
                if selection.record_key != record_key
                    || selection.field.is_some()
                    || selection.passage.is_some()
                {
                    return Err(AppServiceError::invalid_request(
                        "Select this participant's spell to cast",
                    ));
                }
                let spell = catalog
                    .spells
                    .iter()
                    .find(|s| s.navigation.owners == selection.owners)
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("Spell is no longer available")
                    })?;
                let cast = spell
                    .ranks
                    .iter()
                    .find(|r| Some(r.rank) == selection.spell_rank)
                    .ok_or_else(|| AppServiceError::invalid_request("Choose a casting rank"))?;
                if let Some(reason) = &cast.unavailable_reason {
                    return Err(AppServiceError::invalid_request(reason));
                }
                if let Some(key) = &cast.resource_key {
                    let resource =
                        resources
                            .iter_mut()
                            .find(|s| s.key == *key)
                            .ok_or_else(|| {
                                AppServiceError::invalid_request("Spell resource is unavailable")
                            })?;
                    if resource.remaining == 0 {
                        return Err(AppServiceError::invalid_request("No uses remaining"));
                    }
                    let before = resource.remaining;
                    resource.remaining -= 1;
                    for resource in &mut resources {
                        resource.undo_remaining = if resource.key == *key {
                            Some(before)
                        } else {
                            None
                        };
                    }
                } else {
                    // Unlimited casting has no durable resource effect.
                    return Ok(EncounterMutationView {
                        encounter_key: encounter.encounter.encounter_key,
                    });
                }
            }
            EncounterSpellAction::Undo { resource_key } => {
                let resource = resources
                    .iter_mut()
                    .find(|s| s.key == resource_key)
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("Spell resource is unavailable")
                    })?;
                resource.remaining = resource.undo_remaining.take().ok_or_else(|| {
                    AppServiceError::invalid_request("That cast can no longer be undone")
                })?;
            }
            EncounterSpellAction::SetEntryResources {
                entry_id,
                remaining,
            } => {
                let entry = catalog
                    .resources
                    .iter()
                    .filter(|r| r.entry_id == entry_id)
                    .collect::<Vec<_>>();
                if entry.is_empty()
                    || entry.len() != remaining.len()
                    || entry.iter().any(|r| !remaining.contains_key(&r.key))
                {
                    return Err(AppServiceError::invalid_request(
                        "Resource entries changed; reopen the editor",
                    ));
                }
                for resource in entry {
                    let value = remaining[&resource.key];
                    if !(0..=resource.capacity).contains(&value) {
                        return Err(AppServiceError::invalid_request(
                            "Remaining uses must be between zero and capacity",
                        ));
                    }
                }
                for resource in &mut resources {
                    if let Some(value) = remaining.get(&resource.key) {
                        resource.remaining = *value;
                    }
                    resource.undo_remaining = None;
                }
            }
        }
        store.encounters().replace_spell_resources(
            &participant.participant_key,
            request.expected_revision,
            &resources,
        )?;
        Ok(EncounterMutationView {
            encounter_key: encounter.encounter.encounter_key,
        })
    }
}
