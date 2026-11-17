use super::{require, rows, unique};
use crate::encounters::portability::workspace::ValidatedEncounter;
use crate::parties::pc_reference::{decode, encode};
use crate::{LocalStateResult, ParticipantKind, PcReference};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PcState {
    members: Vec<MemberReference>,
    associations: Vec<Association>,
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

impl PcState {
    pub(super) fn capture(db: &Connection, owner: i64) -> LocalStateResult<Option<Self>> {
        let available = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='party_member_references')",
            [], |row| row.get::<_, bool>(0),
        )?;
        if !available {
            return Ok(None);
        }
        let members = rows(
            db,
            "SELECT m.member_key,r.payload FROM party_member_references r
                 JOIN party_members m USING(member_key) JOIN parties p USING(party_key)
                 WHERE p.owner_id=?1 ORDER BY m.member_key",
            [owner],
            |row| Ok((row.get(0)?, row.get::<_, String>(1)?)),
        )?
        .into_iter()
        .map(|(member_key, payload)| {
            Ok(MemberReference {
                member_key,
                reference: decode(&payload)?,
            })
        })
        .collect::<LocalStateResult<Vec<_>>>()?;
        let associations = rows(
            db, "SELECT p.participant_key,a.member_key FROM encounter_pc_party_members a
                 JOIN encounter_participants p ON p.id=a.participant_id JOIN encounters e ON e.id=p.encounter_id
                 WHERE e.owner_id=?1 ORDER BY p.participant_key", [owner],
            |row| Ok(Association { participant_key: row.get(0)?, member_key: row.get(1)? }),
        )?;
        Ok(
            (!members.is_empty() || !associations.is_empty()).then_some(Self {
                members,
                associations,
            }),
        )
    }

    pub(super) fn record_keys(&self) -> impl Iterator<Item = String> + '_ {
        self.members
            .iter()
            .flat_map(|row| row.reference.record_keys().map(ToString::to_string))
    }

    pub(super) fn validate<'a>(
        &self,
        member_keys: impl Iterator<Item = &'a str>,
        encounters: &[ValidatedEncounter],
        references: &BTreeSet<String>,
    ) -> LocalStateResult<()> {
        let owner_members: BTreeSet<_> = member_keys.collect();
        let pcs: BTreeSet<_> = encounters
            .iter()
            .flat_map(ValidatedEncounter::participants)
            .filter(|participant| participant.participant_kind == ParticipantKind::Pc)
            .map(|participant| participant.participant_key.as_str())
            .collect();
        let mut imported = BTreeSet::new();
        for member in &self.members {
            unique(&mut imported, &member.member_key, "imported PC key")?;
            require(
                owner_members.contains(member.member_key.as_str()),
                "PC reference has no owned party member",
            )?;
            member.reference.validate()?;
        }
        require(
            self.record_keys().all(|key| references.contains(&key)),
            "PC reference is missing a record identity",
        )?;
        let mut linked = BTreeSet::new();
        for association in &self.associations {
            unique(
                &mut linked,
                &association.participant_key,
                "associated encounter PC key",
            )?;
            require(
                owner_members.contains(association.member_key.as_str()),
                "PC association has no owned party member",
            )?;
            require(
                pcs.contains(association.participant_key.as_str()),
                "PC association has no owned encounter PC",
            )?;
        }
        Ok(())
    }

    pub(super) fn restore(&self, db: &Connection, owner: i64) -> LocalStateResult<()> {
        for member in &self.members {
            db.execute(
                "INSERT INTO party_member_references(member_key,payload) VALUES(?1,?2)",
                params![member.member_key, encode(&member.reference)?],
            )?;
        }
        for association in &self.associations {
            let changed = db.execute(
                "INSERT INTO encounter_pc_party_members(participant_id,member_key)
                 SELECT p.id,?1 FROM encounter_participants p JOIN encounters e ON e.id=p.encounter_id
                 WHERE e.owner_id=?2 AND p.participant_key=?3 AND p.participant_kind='pc'",
                params![association.member_key, owner, association.participant_key],
            )?;
            require(
                changed == 1,
                "Could not restore the encounter PC association",
            )?;
        }
        Ok(())
    }
}
