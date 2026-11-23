//! All-owner saved state for workspace maintenance.
mod collections;
mod pc;

use pc::PcState;

use crate::encounters::portability::workspace::{RecoveryEncounter, ValidatedEncounter};
use crate::{LocalStateError, LocalStateResult};
use atlas_storage::WriteSession;
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    encounter_creation_receipts: Vec<EncounterCreationReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pc: Option<PcState>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EncounterCreationReceipt {
    request_id: String,
    intent_digest: String,
    encounter_key: String,
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
    pub fn capture(db: &Connection) -> LocalStateResult<Self> {
        // The upgrade backup reads the authenticated schema before the migration runs.
        let has_archive_column = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('saved_lists') WHERE name = 'archived')",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        let has_encounter_archive_column = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('encounters') WHERE name='archived')",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        let has_adjustment_table = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='encounter_adjustments')",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        let has_creation_receipts = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='encounter_creation_receipts')",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        let owners = rows(
            db,
            "SELECT id,principal_key,name FROM principals ORDER BY principal_key",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let mut keys = BTreeSet::new();
        let mut markdown_keys = BTreeSet::new();
        let owners = owners
            .into_iter()
            .map(|(id, key, name)| {
                let lists = List::capture(db, id, has_archive_column)?;
                let encounters = RecoveryEncounter::capture_all(db, id, has_adjustment_table, has_encounter_archive_column)?;
                keys.extend(lists.iter().flat_map(List::record_keys).map(str::to_owned));
                markdown_keys.extend(
                    lists
                        .iter()
                        .flat_map(List::markdown_record_keys)
                        .map(|key| key.to_string()),
                );
                for encounter in &encounters {
                    markdown_keys
                        .extend(encounter.markdown_record_keys().map(|key| key.to_string()));
                    keys.extend(
                        encounter
                            .record_keys()?
                            .into_iter()
                            .map(|key| key.to_string()),
                    );
                }
                let encounter_creation_receipts = if has_creation_receipts {
                    rows(db,
                        "SELECT request_id,intent_digest,encounter_key FROM encounter_creation_receipts WHERE owner_id=?1 ORDER BY request_id",
                        [id], |row| Ok(EncounterCreationReceipt {
                            request_id: row.get(0)?, intent_digest: row.get(1)?, encounter_key: row.get(2)?,
                        }),
                    )?
                } else {
                    Vec::new()
                };
                let pc = PcState::capture(db, id)?;
                if let Some(pc) = &pc {
                    keys.extend(pc.record_keys());
                }
                Ok(Owner {
                    key,
                    name,
                    lists,
                    parties: Party::capture(db, id)?,
                    encounters,
                    encounter_creation_receipts,
                    pc,
                })
            })
            .collect::<LocalStateResult<_>>()?;
        let mut names = db.prepare("SELECT last_known_name FROM record_identity WHERE key=?1")?;
        let mut references = Vec::new();
        for key in keys.union(&markdown_keys) {
            let name: Option<Option<String>> =
                names.query_row([key], |row| row.get(0)).optional()?;
            if name.is_some() || keys.contains(key) {
                references.push(Reference {
                    key: key.clone(),
                    last_known_name: name.flatten(),
                });
            }
        }
        Ok(Self {
            owners,
            references,
            condition_floor: condition_floor(db)?,
        })
    }

    pub fn has_current_encounter_archive_flags(&self) -> bool {
        self.owners.iter().any(|owner| {
            owner
                .encounters
                .iter()
                .any(RecoveryEncounter::has_current_archive_flag)
        })
    }

    pub fn validate_encounter_archive_format(&self, current: bool) -> LocalStateResult<()> {
        require(
            self.owners.iter().all(|owner| {
                owner
                    .encounters
                    .iter()
                    .all(|e| e.matches_archive_format(current))
            }),
            "encounter archive membership disagrees with workspace archive version",
        )
    }

    pub fn has_encounter_creation_receipts(&self) -> bool {
        self.owners
            .iter()
            .any(|owner| !owner.encounter_creation_receipts.is_empty())
    }

    pub fn has_characters(&self) -> bool {
        self.owners
            .iter()
            .any(|owner| owner.pc.as_ref().is_some_and(PcState::has_characters))
    }

    pub fn has_full_pathbuilder_imports(&self) -> bool {
        self.owners.iter().any(|owner| {
            owner
                .pc
                .as_ref()
                .is_some_and(PcState::has_full_pathbuilder_imports)
        })
    }

    pub fn has_pc_references(&self) -> bool {
        self.owners.iter().any(|owner| owner.pc.is_some())
    }

    pub fn has_archived_lists(&self) -> bool {
        self.owners
            .iter()
            .any(|owner| owner.lists.iter().any(List::is_archived))
    }

    pub fn has_encounter_adjustments(&self) -> bool {
        self.owners.iter().any(|owner| {
            owner
                .encounters
                .iter()
                .any(RecoveryEncounter::requires_adjustment_support)
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
        let mut legacy_members = BTreeSet::new();
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
            .map(|mut owner| {
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
                    party.validate(&mut parties, &mut legacy_members)?;
                }
                let mut encounter_slugs = BTreeSet::new();
                let restored: Vec<_> = owner
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
                let mut pc = owner.pc.take().unwrap_or_default();
                pc.normalize_legacy(&mut owner.parties)?;
                pc.validate(
                    owner.parties.iter().map(Party::key),
                    &restored,
                    &references,
                    &mut members,
                )?;
                owner.pc = pc.has_characters().then_some(pc);
                Ok(Owner {
                    key: owner.key,
                    name: owner.name,
                    lists: owner.lists,
                    parties: owner.parties,
                    encounters: restored,
                    encounter_creation_receipts: owner.encounter_creation_receipts,
                    pc: owner.pc,
                })
            })
            .collect::<LocalStateResult<Vec<_>>>()?;
        let mut receipt_targets = BTreeSet::new();
        for owner in &validated {
            let owned: BTreeSet<_> = owner
                .encounters
                .iter()
                .map(|e| e.root().encounter_key.as_str())
                .collect();
            let mut request_ids = BTreeSet::new();
            for receipt in &owner.encounter_creation_receipts {
                unique(
                    &mut request_ids,
                    &receipt.request_id,
                    "encounter creation request",
                )?;
                unique(
                    &mut receipt_targets,
                    &receipt.encounter_key,
                    "encounter creation outcome",
                )?;
                require(
                    crate::encounters::creation_receipts::valid_creation_identity(
                        &receipt.request_id,
                        &receipt.intent_digest,
                    ) && (1..=255).contains(&receipt.encounter_key.len()),
                    "invalid encounter creation receipt",
                )?;
                require(
                    !encounters.contains(&receipt.encounter_key)
                        || owned.contains(receipt.encounter_key.as_str()),
                    "encounter creation receipt belongs to a different owner",
                )?;
            }
        }
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
            for receipt in &owner.encounter_creation_receipts {
                db.execute(
                    "INSERT INTO encounter_creation_receipts(owner_id,request_id,intent_digest,encounter_key) VALUES(?1,?2,?3,?4)",
                    params![id, receipt.request_id, receipt.intent_digest, receipt.encounter_key],
                )?;
            }
            if let Some(pc) = &owner.pc {
                pc.restore(db, id)?;
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
                     OR EXISTS(SELECT 1 FROM encounters) OR EXISTS(SELECT 1 FROM characters)
                     OR EXISTS(SELECT 1 FROM character_creation_receipts)
                     OR EXISTS(SELECT 1 FROM encounter_creation_receipts)
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
