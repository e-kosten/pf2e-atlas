//! A versioned encounter document. Storage IDs are scoped to the document.
mod identities;
mod restore;
mod snapshot;

use super::*;
use crate::{LocalStateError, LocalStateResult};
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::io::{Read, Write};

const VERSION: u32 = 1;
const BYTE_LIMIT: usize = 64 * 1024 * 1024;
const ENTRY_LIMIT: usize = 100_000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterDocument {
    version: u32,
    snapshot: EncounterWithParticipants,
    baselines: Vec<Baseline>,
    pending: Vec<SavedPendingConsequence>,
    applications: Vec<Application>,
    claims: Vec<Claim>,
    receipts: Vec<Receipt>,
    outgoing: Option<Outgoing>,
    pub source_fingerprint: Option<String>,
    pub record_sources: Vec<EncounterRecordSource>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterRecordSource {
    pub record_key: atlas_domain::RecordKey,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    participant_key: String,
    gameplay: ParticipantGameplay,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Application {
    participant_key: String,
    application_id: String,
    grant_slot: String,
    condition_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    claim: EncounterBoundaryClaim,
    request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    request_id: String,
    intent: Value,
    outcome: ReceiptOutcome,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ReceiptOutcome {
    Turn { boundary_key: String },
    Event { receipt: EncounterEventReceipt },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Outgoing {
    request_id: String,
    requires_review: bool,
    turn: SavedOutgoingTurn,
}

fn invalid(message: impl Into<String>) -> LocalStateError {
    LocalStateError::InvalidEncounterDocument(message.into())
}

impl EncounterDocument {
    pub fn read(input: impl Read) -> LocalStateResult<Self> {
        let mut bytes = Vec::new();
        input
            .take((BYTE_LIMIT + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > BYTE_LIMIT {
            return Err(invalid("file exceeds 64 MiB"));
        }
        let remaining = remaining_entries(&bytes)?;
        let mut document: Self =
            serde_json::from_slice(&bytes).map_err(|err| invalid(err.to_string()))?;
        document.validate()?.check_budget(remaining)?;
        Ok(document)
    }

    pub fn encode(&mut self) -> LocalStateResult<Vec<u8>> {
        let identities = self.validate()?;
        let mut output = BoundedDocument(Vec::new());
        serde_json::to_writer(&mut output, &self).map_err(|err| invalid(err.to_string()))?;
        output.write_all(b"\n")?;
        identities.check_budget(remaining_entries(&output.0)?)?;
        Ok(output.0)
    }

    pub fn record_keys(&self) -> LocalStateResult<Vec<atlas_domain::RecordKey>> {
        let outgoing = self
            .outgoing
            .iter()
            .flat_map(|row| &row.turn.snapshot.participants);
        self.snapshot
            .participants
            .iter()
            .chain(outgoing)
            .filter_map(|p| p.record_key.as_deref())
            .map(|key| {
                key.parse()
                    .map_err(|err| invalid(format!("invalid record key {key}: {err}")))
            })
            .collect::<LocalStateResult<std::collections::BTreeSet<_>>>()
            .map(|keys| keys.into_iter().collect())
    }
}

struct BoundedDocument(Vec<u8>);
impl Write for BoundedDocument {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > BYTE_LIMIT - self.0.len() {
            return Err(std::io::Error::other("encounter file exceeds 64 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn remaining_entries(bytes: &[u8]) -> LocalStateResult<usize> {
    let mut remaining = ENTRY_LIMIT;
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    EntryCounter(&mut remaining)
        .deserialize(&mut decoder)
        .and_then(|()| decoder.end())
        .map_err(|error| invalid(error.to_string()))?;
    Ok(remaining)
}

struct EntryCounter<'a>(&'a mut usize);

impl EntryCounter<'_> {
    fn charge<E: de::Error>(&mut self) -> Result<(), E> {
        *self.0 = self
            .0
            .checked_sub(1)
            .ok_or_else(|| E::custom("encounter exceeds 100,000 entries"))?;
        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for EntryCounter<'_> {
    type Value = ();

    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for EntryCounter<'_> {
    type Value = ();

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bounded encounter JSON")
    }

    fn visit_seq<A: SeqAccess<'de>>(mut self, mut sequence: A) -> Result<(), A::Error> {
        while sequence.next_element_seed(EntryCounter(self.0))?.is_some() {
            self.charge()?;
        }
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(mut self, mut map: A) -> Result<(), A::Error> {
        while map.next_key::<de::IgnoredAny>()?.is_some() {
            self.charge()?;
            map.next_value_seed(EntryCounter(self.0))?;
        }
        Ok(())
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_byte_limit_accepts_exact_fit_and_rejects_an_extra_byte() {
        let mut output = BoundedDocument(vec![b' '; BYTE_LIMIT - 1]);
        output.write_all(b"\n").unwrap();
        assert_eq!(output.0.len(), BYTE_LIMIT);
        assert!(output.write_all(b" ").is_err());
        assert_eq!(output.0.len(), BYTE_LIMIT);
    }

    #[test]
    fn nested_entry_budget_includes_object_fields_and_array_members() {
        let boundary = format!("[{}0]", "0,".repeat(ENTRY_LIMIT - 1));
        assert_eq!(remaining_entries(boundary.as_bytes()).unwrap(), 0);
        let oversized = format!(r#"{{"entries":{boundary}}}"#);
        assert!(
            remaining_entries(oversized.as_bytes())
                .unwrap_err()
                .to_string()
                .contains("100,000")
        );
    }
}
