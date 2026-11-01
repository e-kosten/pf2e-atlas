//! Authored ability state persisted independently of derived condition causes.
use super::{EncountersWrite, receipt_codec};
use crate::{LocalStateError, LocalStateResult};
use atlas_domain::OwnedContentLocator;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantAbilityState {
    pub owner: OwnedContentLocator,
    pub rule_ordinal: Option<usize>,
    pub source_fingerprint: String,
    #[serde(default)]
    pub declaration_identity: Option<String>,
    pub value: AbilityStateValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AbilityStateValue {
    Option {
        active: bool,
        selection: Option<String>,
    },
    Uses {
        remaining: i64,
    },
}

#[derive(Debug, Clone)]
pub struct AbilityStateChange {
    pub participant_key: String,
    pub expected_revision: i64,
    pub state: ParticipantAbilityState,
}

pub(super) fn read(
    connection: &Connection,
    key: &str,
) -> LocalStateResult<Vec<ParticipantAbilityState>> {
    let mut statement = connection.prepare("SELECT payload_json FROM encounter_ability_states WHERE participant_key=?1 ORDER BY owner_key,rule_ordinal")?;
    let rows = statement.query_map([key], |row| row.get::<_, String>(0))?;
    rows.map(|row| receipt_codec::decode(&row?)).collect()
}
pub(super) fn read_for_encounter(
    connection: &Connection,
    encounter: &str,
) -> LocalStateResult<std::collections::BTreeMap<String, Vec<ParticipantAbilityState>>> {
    let mut statement = connection.prepare("SELECT p.participant_key,a.payload_json FROM encounter_participants p JOIN encounter_ability_states a USING(participant_key) WHERE p.encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) ORDER BY p.participant_key,a.owner_key,a.rule_ordinal")?;
    let rows = statement.query_map([encounter], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut grouped = std::collections::BTreeMap::<String, Vec<ParticipantAbilityState>>::new();
    for row in rows {
        let (key, payload) = row?;
        grouped
            .entry(key)
            .or_default()
            .push(receipt_codec::decode(&payload)?);
    }
    Ok(grouped)
}

impl EncountersWrite<'_> {
    pub(super) fn save_ability_state(
        &self,
        encounter_key: &str,
        change: &AbilityStateChange,
    ) -> LocalStateResult<()> {
        let changed = self.connection.execute(
            "UPDATE encounter_participants SET revision=revision+1 WHERE participant_key=?1 AND encounter_id=(SELECT id FROM encounters WHERE encounter_key=?2)",
            params![change.participant_key,encounter_key],
        )?;
        if changed != 1 {
            return Err(LocalStateError::InvalidRollRequest(
                "ability state participant changed",
            ));
        }
        let ordinal = change
            .state
            .rule_ordinal
            .map(i64::try_from)
            .transpose()
            .map_err(|_| LocalStateError::InvalidRollRequest("invalid ability rule ordinal"))?
            .unwrap_or(-1);
        self.connection.execute(
            "INSERT INTO encounter_ability_states(participant_key,owner_key,rule_ordinal,payload_json) VALUES (?1,?2,?3,?4) ON CONFLICT(participant_key,owner_key,rule_ordinal) DO UPDATE SET payload_json=excluded.payload_json",
            params![change.participant_key,serde_json::to_string(&change.state.owner)?,ordinal,receipt_codec::encode(&change.state)?],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_payloads_decode_without_a_declaration_binding() {
        let state: ParticipantAbilityState = receipt_codec::decode(r#"{"version":1,"payload":{"owner":{"collection":"/items","identity":{"Stable":"ability000000000"}},"rule_ordinal":null,"source_fingerprint":"old","value":{"kind":"uses","remaining":0}}}"#).unwrap();
        assert_eq!(state.declaration_identity, None);
        assert_eq!(state.value, AbilityStateValue::Uses { remaining: 0 });
    }
}
