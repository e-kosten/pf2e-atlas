//! Historical phases exist only at the file and pre-upgrade backup boundary.
use super::*;
use serde::{Deserializer, Serializer, de::Error as _, ser::Error as _};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LegacyPhase {
    Draft,
    Running,
    Complete,
    Archived,
}

#[derive(Debug)]
pub(super) struct ArchiveRoot {
    pub encounter: Encounter,
    pub legacy_phase: Option<LegacyPhase>,
}

impl Deref for ArchiveRoot {
    type Target = Encounter;
    fn deref(&self) -> &Encounter {
        &self.encounter
    }
}
impl DerefMut for ArchiveRoot {
    fn deref_mut(&mut self) -> &mut Encounter {
        &mut self.encounter
    }
}

impl Serialize for ArchiveRoot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(phase) = self.legacy_phase else {
            return self.encounter.serialize(serializer);
        };
        // A pre-upgrade backup retains the exact original live or frozen phase.
        let Value::Object(mut fields) =
            serde_json::to_value(&self.encounter).map_err(S::Error::custom)?
        else {
            return Err(S::Error::custom("expected encounter metadata object"));
        };
        fields.remove("archived");
        fields.insert(
            "status".into(),
            serde_json::to_value(phase).map_err(S::Error::custom)?,
        );
        fields.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ArchiveRoot {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct Metadata;
        impl<'de> Visitor<'de> for Metadata {
            type Value = ArchiveRoot;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("encounter metadata")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<ArchiveRoot, A::Error> {
                let mut fields = serde_json::Map::new();
                while let Some((key, value)) = map.next_entry::<String, Value>()? {
                    if fields.insert(key, value).is_some() {
                        return Err(A::Error::custom("duplicate encounter metadata field"));
                    }
                }
                let legacy_phase = fields
                    .remove("status")
                    .map(serde_json::from_value::<LegacyPhase>)
                    .transpose()
                    .map_err(A::Error::custom)?;
                if let Some(phase) = legacy_phase
                    && fields
                        .insert("archived".into(), (phase == LegacyPhase::Archived).into())
                        .is_some()
                {
                    return Err(A::Error::custom("encounter has both status and archived"));
                }
                let encounter =
                    serde_json::from_value(Value::Object(fields)).map_err(A::Error::custom)?;
                Ok(ArchiveRoot {
                    encounter,
                    legacy_phase,
                })
            }
        }
        decoder.deserialize_map(Metadata)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ArchiveSnapshot {
    pub encounter: ArchiveRoot,
    pub participants: Vec<EncounterParticipant>,
}

impl From<EncounterWithParticipants> for ArchiveSnapshot {
    fn from(snapshot: EncounterWithParticipants) -> Self {
        Self {
            encounter: ArchiveRoot {
                encounter: snapshot.encounter,
                legacy_phase: None,
            },
            participants: snapshot.participants,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ArchiveTurn {
    pub boundary_key: String,
    pub snapshot: ArchiveSnapshot,
    pub next_participant_key: Option<String>,
}
