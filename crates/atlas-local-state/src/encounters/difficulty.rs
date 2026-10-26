use super::{Encounter, EncounterDifficultyReference, EncountersWrite, storage};
use crate::{LocalStateError, LocalStateResult};

impl EncounterDifficultyReference {
    pub fn validate(self) -> LocalStateResult<()> {
        if self
            .party_level
            .is_some_and(|level| !(1..=20).contains(&level))
        {
            return Err(LocalStateError::InvalidDifficultyReference(
                "Choose a party level from 1 to 20.",
            ));
        }
        if self
            .party_size
            .is_some_and(|size| !(1..=100).contains(&size))
        {
            return Err(LocalStateError::InvalidDifficultyReference(
                "Choose a PC count from 1 to 100.",
            ));
        }
        Ok(())
    }
}
impl EncountersWrite<'_> {
    pub fn set_difficulty_reference(
        &self,
        encounter_ref: &str,
        expected_revision: i64,
        reference: EncounterDifficultyReference,
    ) -> LocalStateResult<Encounter> {
        crate::atomic_action::run(self.connection, || {
            reference.validate()?;
            let encounter = storage::get(self.connection, encounter_ref)?
                .ok_or_else(|| LocalStateError::EncounterNotFound(encounter_ref.into()))?;
            if encounter.revision != expected_revision {
                return Err(LocalStateError::StaleEncounterRevision {
                    encounter_key: encounter.encounter_key,
                    expected_revision,
                    actual_revision: encounter.revision,
                });
            }
            self.connection.execute("UPDATE encounters SET difficulty_party_level=?1, difficulty_party_size=?2, revision=revision+1, updated_at=?3 WHERE owner_id=1 AND encounter_key=?4", rusqlite::params![reference.party_level, reference.party_size, storage::now_rfc3339()?, encounter.encounter_key])?;
            storage::get(self.connection, &encounter.encounter_key)?
                .ok_or(LocalStateError::EncounterNotFound(encounter.encounter_key))
        })
    }
}
