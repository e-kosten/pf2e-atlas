use super::reference::{decode, encode};
use super::{Character, CharacterParty, CharacterSummary};
use crate::{LocalStateError, LocalStateResult, PcReference};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy)]
pub struct CharactersRead<'a> {
    pub(crate) connection: &'a Connection,
    pub(crate) owner_id: i64,
}
pub struct CharactersWrite<'a> {
    read: CharactersRead<'a>,
}
impl<'a> std::ops::Deref for CharactersWrite<'a> {
    type Target = CharactersRead<'a>;
    fn deref(&self) -> &Self::Target {
        &self.read
    }
}
impl<'a> CharactersRead<'a> {
    pub(crate) fn from_connection(connection: &'a Connection, owner_id: i64) -> Self {
        Self {
            connection,
            owner_id,
        }
    }
    pub fn list(&self) -> LocalStateResult<Vec<CharacterSummary>> {
        let mut statement=self.connection.prepare(
            "SELECT c.character_key,c.name,c.level,c.revision,c.reference_payload IS NOT NULL,p.party_key,p.name
             FROM characters c LEFT JOIN party_character_memberships m USING(character_key)
             LEFT JOIN parties p ON p.party_key=m.party_key AND p.owner_id=c.owner_id
             WHERE c.owner_id=?1 ORDER BY c.name COLLATE NOCASE,c.character_key,p.name COLLATE NOCASE,p.party_key")?;
        let mut rows = statement.query([self.owner_id])?;
        let mut characters: Vec<CharacterSummary> = Vec::new();
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            if characters
                .last()
                .is_none_or(|character| character.character_key != key)
            {
                characters.push(CharacterSummary {
                    character_key: key,
                    name: row.get(1)?,
                    level: row.get(2)?,
                    revision: row.get(3)?,
                    imported: row.get(4)?,
                    parties: Vec::new(),
                });
            }
            if let Some(party_key) = row.get::<_, Option<String>>(5)?
                && let Some(character) = characters.last_mut()
            {
                character.parties.push(CharacterParty {
                    party_key,
                    name: row.get(6)?,
                });
            }
        }
        Ok(characters)
    }
    pub fn get(&self, key: &str) -> LocalStateResult<Character> {
        let row = self.connection.query_row("SELECT name,level,revision,reference_payload FROM characters WHERE character_key=?1 AND owner_id=?2", params![key,self.owner_id], |r| Ok((r.get::<_,String>(0)?,r.get(1)?,r.get(2)?,r.get::<_,Option<String>>(3)?))).optional()?.ok_or_else(|| LocalStateError::CharacterNotFound(key.into()))?;
        let mut statement = self.connection.prepare("SELECT p.party_key,p.name FROM party_character_memberships m JOIN parties p USING(party_key) WHERE m.character_key=?1 AND p.owner_id=?2 ORDER BY p.name COLLATE NOCASE,p.party_key")?;
        let parties = statement
            .query_map(params![key, self.owner_id], |r| {
                Ok(CharacterParty {
                    party_key: r.get(0)?,
                    name: r.get(1)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(Character {
            character_key: key.into(),
            name: row.0,
            level: row.1,
            revision: row.2,
            reference: row.3.as_deref().map(decode).transpose()?,
            parties,
        })
    }
    pub fn get_at_revision(&self, key: &str, revision: i64) -> LocalStateResult<Character> {
        let character = self.get(key)?;
        if character.revision != revision {
            return Err(LocalStateError::CharacterConflict);
        }
        Ok(character)
    }
    pub fn creation_receipt(
        &self,
        request_id: &str,
        digest: &str,
    ) -> LocalStateResult<Option<String>> {
        validate_request(request_id, digest)?;
        let result: Option<(String,String)> = self.connection.query_row("SELECT intent_digest,character_key FROM character_creation_receipts WHERE owner_id=?1 AND request_id=?2",params![self.owner_id,request_id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        result
            .map(|(intent, key)| {
                if intent != digest {
                    return Err(LocalStateError::InvalidParty(
                        "This creation request was already used with different details.",
                    ));
                }
                Ok(key)
            })
            .transpose()
    }
    pub fn encounter_character(
        &self,
        encounter: &str,
        participant: &str,
    ) -> LocalStateResult<Option<String>> {
        let id = encounter_pc(self.connection, self.owner_id, encounter, participant)?.0;
        Ok(self.connection.query_row("SELECT a.character_key FROM encounter_pc_characters a JOIN characters c USING(character_key) WHERE a.participant_id=?1 AND c.owner_id=?2",params![id,self.owner_id],|r|r.get(0)).optional()?)
    }
}
impl<'a> CharactersWrite<'a> {
    pub(crate) fn from_connection(connection: &'a Connection, owner_id: i64) -> Self {
        Self {
            read: CharactersRead::from_connection(connection, owner_id),
        }
    }
    pub fn create(
        &self,
        name: &str,
        level: Option<i64>,
        reference: Option<&PcReference>,
    ) -> LocalStateResult<Character> {
        crate::atomic_action::run(self.connection, || {
            validate_fields(name, level)?;
            if let Some(reference) = reference {
                reference.validate()?;
            }
            let key = format!("character_{:032x}", rand::random::<u128>());
            self.connection.execute("INSERT INTO characters(character_key,owner_id,name,level,reference_payload) VALUES(?1,?2,?3,?4,?5)",params![key,self.owner_id,name.trim(),level,reference.map(encode).transpose()?])?;
            self.get(&key)
        })
    }
    pub fn record_creation(&self, request: &str, digest: &str, key: &str) -> LocalStateResult<()> {
        validate_request(request, digest)?;
        self.get(key)?;
        self.connection.execute("INSERT INTO character_creation_receipts(owner_id,request_id,intent_digest,character_key) VALUES(?1,?2,?3,?4)",params![self.owner_id,request,digest,key])?;
        Ok(())
    }
    pub fn update(
        &self,
        key: &str,
        revision: i64,
        name: &str,
        level: Option<i64>,
        reference: Option<&PcReference>,
    ) -> LocalStateResult<Character> {
        crate::atomic_action::run(self.connection, || {
            self.get_at_revision(key, revision)?;
            validate_fields(name, level)?;
            if let Some(reference) = reference {
                reference.validate()?;
            }
            self.connection.execute("UPDATE characters SET name=?1,level=?2,reference_payload=?3,revision=revision+1 WHERE character_key=?4 AND owner_id=?5",params![name.trim(),level,reference.map(encode).transpose()?,key,self.owner_id])?;
            self.get(key)
        })
    }
    pub fn delete(&self, key: &str, revision: i64) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            self.get_at_revision(key, revision)?;
            self.connection.execute("UPDATE parties SET revision=revision+1 WHERE owner_id=?1 AND party_key IN (SELECT party_key FROM party_character_memberships WHERE character_key=?2)",params![self.owner_id,key])?;
            self.connection.execute("UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE owner_id=?1 AND id IN (SELECT p.encounter_id FROM encounter_pc_characters a JOIN encounter_participants p ON p.id=a.participant_id WHERE a.character_key=?2)",params![self.owner_id,key])?;
            self.connection.execute(
                "DELETE FROM characters WHERE character_key=?1 AND owner_id=?2",
                params![key, self.owner_id],
            )?;
            Ok(())
        })
    }
    pub fn associate_added_pc(
        &self,
        encounter: &str,
        participant: &str,
        character: &str,
    ) -> LocalStateResult<()> {
        let id = encounter_pc(self.connection, self.owner_id, encounter, participant)?.0;
        self.get(character)?;
        self.connection.execute("INSERT INTO encounter_pc_characters(participant_id,character_key) VALUES(?1,?2) ON CONFLICT(participant_id) DO UPDATE SET character_key=excluded.character_key",params![id,character])?;
        Ok(())
    }
    pub fn set_encounter_character(
        &self,
        encounter: &str,
        participant: &str,
        expected_revision: i64,
        character: Option<&str>,
    ) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            let (id, revision) =
                encounter_pc(self.connection, self.owner_id, encounter, participant)?;
            if revision != expected_revision {
                return Err(LocalStateError::StaleEncounterRevision {
                    encounter_key: encounter.into(),
                    expected_revision,
                    actual_revision: revision,
                });
            }
            if let Some(key) = character {
                self.associate_added_pc(encounter, participant, key)?;
            } else {
                self.connection.execute(
                    "DELETE FROM encounter_pc_characters WHERE participant_id=?1",
                    [id],
                )?;
            }
            self.connection.execute("UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE encounter_key=?1 AND owner_id=?2",params![encounter,self.owner_id])?;
            Ok(())
        })
    }
}
fn validate_fields(name: &str, level: Option<i64>) -> LocalStateResult<()> {
    if level.is_some_and(|level| !(1..=20).contains(&level)) {
        return Err(LocalStateError::InvalidParty(
            "Enter a PC level from 1 to 20, or leave it blank.",
        ));
    }
    if !(1..=255).contains(&name.trim().chars().count()) {
        return Err(LocalStateError::InvalidParty(
            "Enter a PC name of 1 to 255 characters.",
        ));
    }
    Ok(())
}

fn validate_request(id: &str, digest: &str) -> LocalStateResult<()> {
    if id.is_empty()
        || id.len() > 128
        || digest.len() != 64
        || !digest.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(LocalStateError::InvalidParty(
            "Invalid character creation request.",
        ));
    }
    Ok(())
}
fn encounter_pc(
    db: &Connection,
    owner: i64,
    encounter: &str,
    participant: &str,
) -> LocalStateResult<(i64, i64)> {
    db.query_row("SELECT p.id,e.revision FROM encounter_participants p JOIN encounters e ON e.id=p.encounter_id WHERE e.owner_id=?1 AND e.encounter_key=?2 AND p.participant_key=?3 AND p.participant_kind='pc'",params![owner,encounter,participant],|r|Ok((r.get(0)?,r.get(1)?))).optional()?.ok_or_else(|| LocalStateError::ParticipantNotFound(participant.into()))
}
