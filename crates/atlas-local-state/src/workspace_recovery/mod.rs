//! All-owner saved state for workspace maintenance.
mod collections;

use crate::encounters::portability::workspace::{RecoveryEncounter, ValidatedEncounter};
use crate::{LocalStateError, LocalStateResult};
use atlas_storage::{ReadSession, WriteSession};
use collections::{List, Party};
use rusqlite::{Connection, OptionalExtension, Params, Row, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct SavedWorkspace {
    owners: Vec<Owner<RecoveryEncounter>>,
    references: Vec<Reference>,
    condition_floor: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Owner<E> {
    key: String,
    name: String,
    lists: Vec<List>,
    parties: Vec<Party>,
    encounters: Vec<E>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Reference {
    key: String,
    last_known_name: Option<String>,
}

pub struct ValidatedWorkspace {
    owners: Vec<Owner<ValidatedEncounter>>,
    references: Vec<Reference>,
    condition_floor: i64,
}

impl SavedWorkspace {
    /// Capture every owner from an already admitted read snapshot.
    pub fn capture(session: &ReadSession) -> LocalStateResult<Self> {
        let db = session.connection();
        let owners = rows(
            db,
            "SELECT id,principal_key,name FROM principals ORDER BY principal_key",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let mut keys = BTreeSet::new();
        let owners = owners
            .into_iter()
            .map(|(id, key, name)| {
                let lists = List::capture(db, id)?;
                let encounters = RecoveryEncounter::capture_all(db, id)?;
                keys.extend(lists.iter().flat_map(List::record_keys).map(str::to_owned));
                for encounter in &encounters {
                    keys.extend(
                        encounter
                            .record_keys()?
                            .into_iter()
                            .map(|key| key.to_string()),
                    );
                }
                Ok(Owner {
                    key,
                    name,
                    lists,
                    parties: Party::capture(db, id)?,
                    encounters,
                })
            })
            .collect::<LocalStateResult<_>>()?;
        let mut names = db.prepare("SELECT last_known_name FROM record_identity WHERE key=?1")?;
        let references = keys
            .into_iter()
            .map(|key| {
                let last_known_name = names
                    .query_row([&key], |row| row.get(0))
                    .optional()?
                    .flatten();
                Ok(Reference {
                    key,
                    last_known_name,
                })
            })
            .collect::<LocalStateResult<_>>()?;
        Ok(Self {
            owners,
            references,
            condition_floor: condition_floor(db)?,
        })
    }

    pub fn validate(self) -> LocalStateResult<ValidatedWorkspace> {
        let mut references = BTreeSet::new();
        for reference in &self.references {
            require(
                reference.key.parse::<atlas_domain::RecordKey>().is_ok(),
                "invalid reference key",
            )?;
            unique(&mut references, &reference.key, "reference key")?;
        }
        let mut owners = BTreeSet::new();
        let mut lists = BTreeSet::new();
        let mut parties = BTreeSet::new();
        let mut members = BTreeSet::new();
        let mut encounters = BTreeSet::new();
        let mut participants = BTreeSet::new();
        let mut conditions = BTreeSet::new();
        let mut boundaries = BTreeSet::new();
        require(
            self.condition_floor >= 0,
            "invalid condition allocation floor",
        )?;
        let validated = self
            .owners
            .into_iter()
            .map(|owner| {
                unique(&mut owners, &owner.key, "principal key")?;
                let mut list_slugs = BTreeSet::new();
                for list in &owner.lists {
                    list.validate(&mut lists, &mut list_slugs)?;
                    for key in list.record_keys() {
                        require(
                            references.contains(key),
                            "list references a missing record identity",
                        )?;
                    }
                }
                for party in &owner.parties {
                    party.validate(&mut parties, &mut members)?;
                }
                let mut encounter_slugs = BTreeSet::new();
                let restored = owner
                    .encounters
                    .into_iter()
                    .map(|encounter| {
                        let encounter = encounter.validate()?;
                        unique(
                            &mut encounters,
                            &encounter.root().encounter_key,
                            "encounter key",
                        )?;
                        unique(
                            &mut encounter_slugs,
                            &encounter.root().slug,
                            "encounter slug",
                        )?;
                        require(
                            encounter.condition_floor <= self.condition_floor,
                            "condition allocation floor is below a saved condition ID",
                        )?;
                        for key in encounter.record_keys()? {
                            require(
                                references.contains(&key.to_string()),
                                "encounter references a missing record identity",
                            )?;
                        }
                        for boundary in encounter.turn_boundaries() {
                            unique(&mut boundaries, boundary, "turn boundary key")?;
                        }
                        for participant in encounter.participants() {
                            unique(
                                &mut participants,
                                &participant.participant_key,
                                "participant key",
                            )?;
                            for condition in &participant.conditions {
                                require(
                                    conditions.insert(condition.condition_id),
                                    "duplicate live condition ID",
                                )?;
                            }
                        }
                        Ok(encounter)
                    })
                    .collect::<LocalStateResult<_>>()?;
                Ok(Owner {
                    key: owner.key,
                    name: owner.name,
                    lists: owner.lists,
                    parties: owner.parties,
                    encounters: restored,
                })
            })
            .collect::<LocalStateResult<_>>()?;
        require(owners.contains("local"), "missing seeded local principal")?;
        Ok(ValidatedWorkspace {
            owners: validated,
            references: self.references,
            condition_floor: self.condition_floor,
        })
    }
}

impl ValidatedWorkspace {
    /// Apply saved state inside the caller's transaction. The caller owns commit.
    pub fn restore(&self, session: &WriteSession) -> LocalStateResult<()> {
        let db = session.connection();
        check_empty(db)?;
        for reference in &self.references {
            db.execute(
                "INSERT INTO record_identity(key,last_known_name,availability)
                 VALUES(?1,?2,'unavailable') ON CONFLICT(key) DO NOTHING",
                params![reference.key, reference.last_known_name],
            )?;
        }
        let floor = self.condition_floor.max(condition_floor(db)?);
        for owner in &self.owners {
            let id = db.query_row(
                "INSERT INTO principals(principal_key,name) VALUES(?1,?2)
                 ON CONFLICT(principal_key) DO UPDATE SET name=excluded.name RETURNING id",
                params![owner.key, owner.name],
                |row| row.get(0),
            )?;
            for list in &owner.lists {
                list.restore(db, id)?;
            }
            for party in &owner.parties {
                party.restore(db, id)?;
            }
            for encounter in &owner.encounters {
                encounter.restore(db, id)?;
            }
        }
        if floor > condition_floor(db)? {
            db.execute(
                "INSERT INTO sqlite_sequence(name,seq) SELECT 'encounter_participant_conditions',0
                 WHERE NOT EXISTS(SELECT 1 FROM sqlite_sequence WHERE name='encounter_participant_conditions')", [],
            )?;
            db.execute(
                "UPDATE sqlite_sequence SET seq=?1 WHERE name='encounter_participant_conditions'",
                [floor],
            )?;
        }
        Ok(())
    }
}

pub fn check_empty(db: &Connection) -> LocalStateResult<()> {
    let empty = db.query_row(
        "SELECT NOT (EXISTS(SELECT 1 FROM saved_lists) OR EXISTS(SELECT 1 FROM parties)
                     OR EXISTS(SELECT 1 FROM encounters)
                     OR EXISTS(SELECT 1 FROM principals WHERE principal_key != 'local'))",
        [],
        |row| row.get(0),
    )?;
    require(empty, "destination workspace already contains saved state")
}

fn condition_floor(db: &Connection) -> LocalStateResult<i64> {
    Ok(db.query_row(
        "SELECT COALESCE(MAX(seq),0) FROM sqlite_sequence WHERE name='encounter_participant_conditions'",
        [], |row| row.get(0),
    )?)
}

fn rows<T>(
    db: &Connection,
    sql: &str,
    params: impl Params,
    map: impl FnMut(&Row<'_>) -> rusqlite::Result<T>,
) -> LocalStateResult<Vec<T>> {
    Ok(db
        .prepare(sql)?
        .query_map(params, map)?
        .collect::<Result<_, _>>()?)
}

fn require(valid: bool, message: &str) -> LocalStateResult<()> {
    if valid {
        Ok(())
    } else {
        Err(LocalStateError::InvalidWorkspaceRecovery(message.into()))
    }
}

fn unique(keys: &mut BTreeSet<String>, key: &str, label: &str) -> LocalStateResult<()> {
    if key.is_empty() || !keys.insert(key.to_owned()) {
        return Err(LocalStateError::InvalidWorkspaceRecovery(format!(
            "empty or duplicate {label}: {key}"
        )));
    }
    Ok(())
}
