use super::{require, rows, unique};
use crate::characters::reference::{decode, encode};
use crate::encounters::portability::workspace::ValidatedEncounter;
use crate::{LocalStateResult, ParticipantKind, PcReference};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PcState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    members: Vec<MemberReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    associations: Vec<Association>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    characters: Vec<Character>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    memberships: Vec<Membership>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    character_associations: Vec<CharacterAssociation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    creation_receipts: Vec<CreationReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    addition_receipts: Vec<AdditionReceipt>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberReference {
    member_key: String,
    reference: PcReference,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Association {
    participant_key: String,
    member_key: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Character {
    character_key: String,
    name: String,
    level: Option<i64>,
    revision: i64,
    reference: Option<PcReference>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Membership {
    party_key: String,
    character_key: String,
    position: i64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterAssociation {
    participant_key: String,
    character_key: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreationReceipt {
    request_id: String,
    intent_digest: String,
    character_key: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdditionReceipt {
    encounter_key: String,
    request_id: String,
    intent_json: String,
    receipt_json: String,
}

impl PcState {
    pub(super) fn capture(db: &Connection, owner: i64) -> LocalStateResult<Option<Self>> {
        let current = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='characters')",
            [],
            |r| r.get::<_, bool>(0),
        )?;
        let mut state = Self::default();
        if current {
            state.characters=rows(db,"SELECT character_key,name,level,revision,reference_payload FROM characters WHERE owner_id=?1 ORDER BY character_key",[owner],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get::<_,Option<String>>(4)?)))?.into_iter().map(|(character_key,name,level,revision,payload)|Ok(Character{character_key,name,level,revision,reference:payload.as_deref().map(decode).transpose()?})).collect::<LocalStateResult<_>>()?;
            state.memberships = rows(
                db,
                "SELECT m.party_key,m.character_key,m.position FROM party_character_memberships m JOIN parties p USING(party_key) WHERE p.owner_id=?1 ORDER BY m.party_key,m.position",
                [owner],
                |r| {
                    Ok(Membership {
                        party_key: r.get(0)?,
                        character_key: r.get(1)?,
                        position: r.get(2)?,
                    })
                },
            )?;
            state.character_associations = rows(
                db,
                "SELECT p.participant_key,a.character_key FROM encounter_pc_characters a JOIN encounter_participants p ON p.id=a.participant_id JOIN encounters e ON e.id=p.encounter_id WHERE e.owner_id=?1 ORDER BY p.participant_key",
                [owner],
                |r| {
                    Ok(CharacterAssociation {
                        participant_key: r.get(0)?,
                        character_key: r.get(1)?,
                    })
                },
            )?;
            state.creation_receipts = rows(
                db,
                "SELECT request_id,intent_digest,character_key FROM character_creation_receipts WHERE owner_id=?1 ORDER BY request_id",
                [owner],
                |r| {
                    Ok(CreationReceipt {
                        request_id: r.get(0)?,
                        intent_digest: r.get(1)?,
                        character_key: r.get(2)?,
                    })
                },
            )?;
            state.addition_receipts = rows(
                db,
                "SELECT e.encounter_key,r.request_id,r.intent_json,r.receipt_json FROM encounter_command_receipts r JOIN encounters e ON e.id=r.encounter_id WHERE e.owner_id=?1 AND r.family='pc_addition' ORDER BY e.encounter_key,r.request_id",
                [owner],
                |r| {
                    Ok(AdditionReceipt {
                        encounter_key: r.get(0)?,
                        request_id: r.get(1)?,
                        intent_json: r.get(2)?,
                        receipt_json: r.get(3)?,
                    })
                },
            )?;
        } else {
            let available = db.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='party_member_references')",
                [],
                |r| r.get::<_, bool>(0),
            )?;
            if available {
                state.members=rows(db,"SELECT m.member_key,r.payload FROM party_member_references r JOIN party_members m USING(member_key) JOIN parties p USING(party_key) WHERE p.owner_id=?1 ORDER BY m.member_key",[owner],|r|Ok((r.get(0)?,r.get::<_,String>(1)?)))?.into_iter().map(|(member_key,payload)|Ok(MemberReference{member_key,reference:decode(&payload)?})).collect::<LocalStateResult<_>>()?;
                state.associations = rows(
                    db,
                    "SELECT p.participant_key,a.member_key FROM encounter_pc_party_members a JOIN encounter_participants p ON p.id=a.participant_id JOIN encounters e ON e.id=p.encounter_id WHERE e.owner_id=?1 ORDER BY p.participant_key",
                    [owner],
                    |r| {
                        Ok(Association {
                            participant_key: r.get(0)?,
                            member_key: r.get(1)?,
                        })
                    },
                )?;
            }
        }
        Ok(
            (state.has_characters() || !state.members.is_empty() || !state.associations.is_empty())
                .then_some(state),
        )
    }
    pub(super) fn has_characters(&self) -> bool {
        !self.characters.is_empty()
            || !self.memberships.is_empty()
            || !self.character_associations.is_empty()
            || !self.creation_receipts.is_empty()
            || !self.addition_receipts.is_empty()
    }
    pub(super) fn normalize_legacy(
        &mut self,
        parties: &mut [super::Party],
    ) -> LocalStateResult<()> {
        let mut references = HashMap::new();
        for member in std::mem::take(&mut self.members) {
            require(
                references
                    .insert(member.member_key, member.reference)
                    .is_none(),
                "duplicate legacy PC reference",
            )?;
        }
        for party in parties {
            for member in party.take_members() {
                require(
                    !self
                        .characters
                        .iter()
                        .any(|c| c.character_key == member.key),
                    "duplicate legacy character key",
                )?;
                self.characters.push(Character {
                    character_key: member.key.clone(),
                    name: member.name,
                    level: member.level,
                    revision: 0,
                    reference: references.remove(&member.key),
                });
                self.memberships.push(Membership {
                    party_key: party.key().into(),
                    character_key: member.key,
                    position: member.position,
                });
            }
        }
        require(
            references.is_empty(),
            "PC reference has no owned party member",
        )?;
        self.character_associations
            .extend(std::mem::take(&mut self.associations).into_iter().map(|a| {
                CharacterAssociation {
                    participant_key: a.participant_key,
                    character_key: a.member_key,
                }
            }));
        Ok(())
    }
    pub(super) fn record_keys(&self) -> impl Iterator<Item = String> + '_ {
        self.members
            .iter()
            .map(|m| &m.reference)
            .chain(self.characters.iter().filter_map(|c| c.reference.as_ref()))
            .flat_map(|r| r.record_keys().map(ToString::to_string))
    }
    pub(super) fn validate<'a>(
        &self,
        parties: impl Iterator<Item = &'a str>,
        encounters: &[ValidatedEncounter],
        references: &BTreeSet<String>,
        global_characters: &mut BTreeSet<String>,
    ) -> LocalStateResult<()> {
        require(
            self.members.is_empty() && self.associations.is_empty(),
            "legacy PC state was not normalized",
        )?;
        let parties: BTreeSet<_> = parties.collect();
        let mut characters = BTreeSet::new();
        for character in &self.characters {
            unique(&mut characters, &character.character_key, "character key")?;
            unique(global_characters, &character.character_key, "character key")?;
            require(
                (1..=255).contains(&character.name.trim().chars().count())
                    && character.level.is_none_or(|l| (1..=20).contains(&l))
                    && character.revision >= 0,
                "invalid saved character",
            )?;
            if let Some(reference) = &character.reference {
                reference.validate()?;
            }
        }
        require(
            self.record_keys().all(|key| references.contains(&key)),
            "PC reference is missing a record identity",
        )?;
        let mut memberships = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for membership in &self.memberships {
            require(
                parties.contains(membership.party_key.as_str())
                    && characters.contains(&membership.character_key),
                "membership has no owned character or party",
            )?;
            require(
                membership.position >= 0
                    && memberships.insert((&membership.party_key, &membership.character_key))
                    && positions.insert((&membership.party_key, membership.position)),
                "invalid or duplicate membership",
            )?;
        }
        for party in &parties {
            require(
                self.memberships
                    .iter()
                    .filter(|m| m.party_key == *party)
                    .count()
                    <= 50,
                "party has too many characters",
            )?;
        }
        let pcs: BTreeSet<_> = encounters
            .iter()
            .flat_map(ValidatedEncounter::participants)
            .filter(|p| p.participant_kind == ParticipantKind::Pc)
            .map(|p| p.participant_key.as_str())
            .collect();
        let mut associated = BTreeSet::new();
        for association in &self.character_associations {
            require(
                characters.contains(&association.character_key)
                    && pcs.contains(association.participant_key.as_str())
                    && associated.insert(&association.participant_key),
                "invalid character association or owner",
            )?;
        }
        let mut requests = BTreeSet::new();
        for receipt in &self.creation_receipts {
            unique(
                &mut requests,
                &receipt.request_id,
                "character creation request",
            )?;
            require(
                receipt.request_id.len() <= 128
                    && valid_digest(&receipt.intent_digest)
                    && (1..=255).contains(&receipt.character_key.len()),
                "invalid character creation receipt",
            )?;
        }
        let encounter_keys: BTreeSet<_> = encounters
            .iter()
            .map(|e| e.root().encounter_key.as_str())
            .collect();
        let mut additions = BTreeSet::new();
        for receipt in &self.addition_receipts {
            require(
                encounter_keys.contains(receipt.encounter_key.as_str())
                    && additions.insert((&receipt.encounter_key, &receipt.request_id))
                    && !receipt.request_id.is_empty()
                    && receipt.request_id.len() <= 128
                    && receipt.intent_json.len() <= 64 * 1024
                    && receipt.receipt_json.len() <= 64 * 1024,
                "invalid PC addition receipt",
            )?;
            let intent: serde_json::Value = serde_json::from_str(&receipt.intent_json)?;
            require(
                intent
                    .get("digest")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(valid_digest),
                "invalid PC addition intent",
            )?;
            let value = crate::PcAdditionReceipt::decode(&receipt.receipt_json)?;
            require(
                value.characters.len() == value.participants.len()
                    && !value.characters.is_empty()
                    && value.characters.len() <= 50
                    && value
                        .characters
                        .iter()
                        .chain(&value.participants)
                        .all(|key| !key.is_empty() && key.len() <= 255),
                "invalid PC addition outcome",
            )?;
        }
        Ok(())
    }
    pub(super) fn restore(&self, db: &Connection, owner: i64) -> LocalStateResult<()> {
        for c in &self.characters {
            db.execute("INSERT INTO characters(character_key,owner_id,name,level,revision,reference_payload) VALUES(?1,?2,?3,?4,?5,?6)",params![c.character_key,owner,c.name,c.level,c.revision,c.reference.as_ref().map(encode).transpose()?])?;
        }
        for m in &self.memberships {
            db.execute("INSERT INTO party_character_memberships(party_key,character_key,position) VALUES(?1,?2,?3)",params![m.party_key,m.character_key,m.position])?;
        }
        for a in &self.character_associations {
            db.execute("INSERT INTO encounter_pc_characters(participant_id,character_key) SELECT p.id,?1 FROM encounter_participants p JOIN encounters e ON e.id=p.encounter_id WHERE p.participant_key=?2 AND e.owner_id=?3",params![a.character_key,a.participant_key,owner])?;
        }
        for r in &self.creation_receipts {
            db.execute("INSERT INTO character_creation_receipts(owner_id,request_id,intent_digest,character_key) VALUES(?1,?2,?3,?4)",params![owner,r.request_id,r.intent_digest,r.character_key])?;
        }
        for r in &self.addition_receipts {
            db.execute("INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,receipt_json) SELECT id,'pc_addition',?2,?3,?4 FROM encounters WHERE encounter_key=?1 AND owner_id=?5",params![r.encounter_key,r.request_id,r.intent_json,r.receipt_json,owner])?;
        }
        Ok(())
    }
}
fn valid_digest(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
