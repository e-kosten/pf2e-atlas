//! Identity-preserving encounter recovery for a whole workspace.
use super::*;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RecoveryEncounter {
    snapshot: EncounterWithParticipants,
    baselines: Vec<RecoveryBaseline>,
    pending: Vec<SavedPendingConsequence>,
    applications: Vec<RecoveryApplication>,
    claims: Vec<RecoveryClaim>,
    receipts: Vec<RecoveryReceipt>,
    outgoing: Option<RecoveryOutgoing>,
    tracker_positions: BTreeMap<String, Vec<i64>>,
}

impl RecoveryEncounter {
    pub(crate) fn capture_all(db: &Connection, owner: i64) -> LocalStateResult<Vec<Self>> {
        storage::list(db, owner)?
            .into_iter()
            .map(|encounter| Self::capture(db, owner, encounter))
            .collect()
    }

    fn capture(db: &Connection, owner: i64, encounter: Encounter) -> LocalStateResult<Self> {
        let key = &encounter.encounter_key;
        let id = db
            .query_row(
                "SELECT id FROM encounters WHERE owner_id=?1 AND encounter_key=?2",
                params![owner, key],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| LocalStateError::EncounterNotFound(key.clone()))?;
        let participants = storage::participants(db, owner, key)?;
        let mut statement = db.prepare(
            "SELECT t.participant_key,t.position FROM encounter_manual_trackers t
             JOIN encounter_participants p USING(participant_key)
             WHERE p.encounter_id=?1 ORDER BY t.participant_key,t.position",
        )?;
        let mut tracker_positions = BTreeMap::<String, Vec<i64>>::new();
        for row in statement.query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))? {
            let (key, position) = row?;
            tracker_positions.entry(key).or_default().push(position);
        }
        let document = snapshot::capture_document(
            db,
            id,
            EncounterWithParticipants {
                encounter,
                participants,
            },
            None,
        )?;
        Ok(Self {
            snapshot: document.snapshot,
            baselines: document.baselines.into_iter().map(Into::into).collect(),
            pending: document.pending,
            applications: document.applications.into_iter().map(Into::into).collect(),
            claims: document.claims.into_iter().map(Into::into).collect(),
            receipts: document.receipts.into_iter().map(Into::into).collect(),
            outgoing: document.outgoing.map(Into::into),
            tracker_positions,
        })
    }

    pub(crate) fn record_keys(&self) -> LocalStateResult<Vec<atlas_domain::RecordKey>> {
        recovery_record_keys(
            &self.snapshot,
            self.outgoing.as_ref().map(|row| &row.turn),
            self.baselines.iter().map(|row| &row.gameplay),
        )
    }

    pub(crate) fn validate(self) -> LocalStateResult<ValidatedEncounter> {
        for participant in &self.snapshot.participants {
            let positions = self.tracker_positions.get(&participant.participant_key);
            let positions = positions.map_or(&[][..], Vec::as_slice);
            if positions.len() != participant.manual_trackers.len()
                || positions.iter().any(|&position| position < 1)
                || positions.windows(2).any(|pair| pair[0] >= pair[1])
            {
                return Err(invalid("invalid recovery tracker positions"));
            }
        }
        let participant_keys: BTreeSet<_> = self
            .snapshot
            .participants
            .iter()
            .map(|participant| &participant.participant_key)
            .collect();
        if self
            .tracker_positions
            .keys()
            .any(|key| !participant_keys.contains(key))
        {
            return Err(invalid(
                "tracker positions reference an unknown participant",
            ));
        }
        let mut document = EncounterDocument {
            version: VERSION,
            snapshot: self.snapshot,
            baselines: self.baselines.into_iter().map(Into::into).collect(),
            pending: self.pending,
            applications: self.applications.into_iter().map(Into::into).collect(),
            claims: self.claims.into_iter().map(Into::into).collect(),
            receipts: self.receipts.into_iter().map(Into::into).collect(),
            outgoing: self.outgoing.map(Into::into),
            source_fingerprint: None,
            record_sources: Vec::new(),
        };
        let condition_floor = document.validate()?.condition_floor();
        Ok(ValidatedEncounter {
            document,
            condition_floor,
            tracker_positions: self.tracker_positions,
        })
    }
}

pub(crate) struct ValidatedEncounter {
    document: EncounterDocument,
    pub(crate) condition_floor: i64,
    tracker_positions: BTreeMap<String, Vec<i64>>,
}

impl ValidatedEncounter {
    pub(crate) fn root(&self) -> &Encounter {
        &self.document.snapshot.encounter
    }

    pub(crate) fn turn_boundaries(&self) -> impl Iterator<Item = &str> {
        self.document
            .receipts
            .iter()
            .filter_map(|row| match &row.outcome {
                ReceiptOutcome::Turn { boundary_key } => Some(boundary_key.as_str()),
                ReceiptOutcome::Event { .. } => None,
            })
    }

    pub(crate) fn participants(&self) -> &[EncounterParticipant] {
        &self.document.snapshot.participants
    }

    pub(crate) fn record_keys(&self) -> LocalStateResult<Vec<atlas_domain::RecordKey>> {
        recovery_record_keys(
            &self.document.snapshot,
            self.document.outgoing.as_ref().map(|row| &row.turn),
            self.document.baselines.iter().map(|row| &row.gameplay),
        )
    }

    pub(crate) fn restore(&self, db: &Connection, owner: i64) -> LocalStateResult<()> {
        let root = self.root();
        db.execute(
            "INSERT INTO encounters(owner_id,encounter_key,slug,name,description,note,status,
                                    round_number,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                owner,
                root.encounter_key,
                root.slug,
                root.name,
                root.description,
                root.note,
                root.status.as_str(),
                root.round_number,
                root.created_at,
                root.updated_at
            ],
        )?;
        restore::write_document(
            db,
            db.last_insert_rowid(),
            &self.document,
            Some(&self.tracker_positions),
        )
    }
}

fn recovery_record_keys<'a>(
    snapshot: &'a EncounterWithParticipants,
    outgoing: Option<&'a SavedOutgoingTurn>,
    baselines: impl Iterator<Item = &'a ParticipantGameplay>,
) -> LocalStateResult<Vec<atlas_domain::RecordKey>> {
    let participants = snapshot.participants.iter().chain(
        outgoing
            .into_iter()
            .flat_map(|row| &row.snapshot.participants),
    );
    let mut keys = BTreeSet::new();
    for participant in participants {
        if let Some(key) = &participant.record_key {
            keys.insert(
                key.parse()
                    .map_err(|error| invalid(format!("invalid record key {key}: {error}")))?,
            );
        }
        keys.extend(origin_record_keys(&participant.conditions));
    }
    for baseline in baselines {
        keys.extend(origin_record_keys(&baseline.conditions));
    }
    Ok(keys.into_iter().collect())
}

fn origin_record_keys(
    conditions: &[EncounterParticipantCondition],
) -> impl Iterator<Item = atlas_domain::RecordKey> {
    conditions.iter().filter_map(|condition| {
        condition
            .origin
            .as_ref()?
            .record_key
            .as_deref()?
            .parse()
            .ok()
    })
}

// Recovery readers accept optional additions. Keep the exchange DTOs strict.
macro_rules! recovery_row {
    ($wire:ident, $snapshot:ident { $($field:ident: $ty:ty),+ $(,)? }) => {
        #[derive(Debug, Serialize, Deserialize)]
        struct $wire { $($field: $ty),+ }
        impl From<$snapshot> for $wire {
            fn from(row: $snapshot) -> Self {
                Self { $($field: row.$field.into()),+ }
            }
        }
        impl From<$wire> for $snapshot {
            fn from(row: $wire) -> Self {
                Self { $($field: row.$field.into()),+ }
            }
        }
    };
}

recovery_row!(
    RecoveryBaseline,
    Baseline {
        participant_key: String,
        gameplay: ParticipantGameplay,
    }
);
recovery_row!(RecoveryApplication, Application {
    participant_key: String,
    application_id: String,
    grant_slot: String,
    condition_id: Option<i64>,
});
recovery_row!(
    RecoveryClaim,
    Claim {
        claim: EncounterBoundaryClaim,
        request_id: String,
    }
);
recovery_row!(
    RecoveryOutgoing,
    Outgoing {
        request_id: String,
        requires_review: bool,
        turn: SavedOutgoingTurn,
    }
);
recovery_row!(
    RecoveryReceipt,
    Receipt {
        request_id: String,
        intent: Value,
        outcome: RecoveryReceiptOutcome,
    }
);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum RecoveryReceiptOutcome {
    #[serde(rename = "turn")]
    Turn { boundary_key: String },
    #[serde(rename = "event")]
    Event { receipt: EncounterEventReceipt },
}

impl From<ReceiptOutcome> for RecoveryReceiptOutcome {
    fn from(outcome: ReceiptOutcome) -> Self {
        match outcome {
            ReceiptOutcome::Turn { boundary_key } => Self::Turn { boundary_key },
            ReceiptOutcome::Event { receipt } => Self::Event { receipt },
        }
    }
}

impl From<RecoveryReceiptOutcome> for ReceiptOutcome {
    fn from(outcome: RecoveryReceiptOutcome) -> Self {
        match outcome {
            RecoveryReceiptOutcome::Turn { boundary_key } => Self::Turn { boundary_key },
            RecoveryReceiptOutcome::Event { receipt } => Self::Event { receipt },
        }
    }
}
