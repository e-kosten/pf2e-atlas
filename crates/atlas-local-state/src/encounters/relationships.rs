//! Durable directed facts. Defaults and competing-source resolution belong to app-service.
use super::{EncounterWithParticipants, EncountersRead};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipProperty {
    Detection,
    Awareness,
    Attitude,
    Control,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionState {
    Observed,
    Hidden,
    Undetected,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttitudeState {
    Helpful,
    Friendly,
    Indifferent,
    Unfriendly,
    Hostile,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RelationshipValue {
    Detection(DetectionState),
    Awareness(bool),
    Attitude(AttitudeState),
    NoControl,
    Control {
        controller_key: String,
        instructions: Option<String>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelationshipProvenance {
    Adjudication {
        sources: Vec<RelationshipSourceSnapshot>,
    },
    Table,
    Source {
        condition_id: Option<i64>,
        record_key: Option<String>,
        application_id: Option<String>,
    },
    Sources {
        condition_ids: Vec<i64>,
    },
    SourceDefault,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipCandidate {
    pub value: RelationshipValue,
    pub provenance: RelationshipProvenance,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelationshipKnowledge {
    Known {
        value: RelationshipValue,
        provenance: RelationshipProvenance,
    },
    Unknown {
        required_facts: Vec<String>,
    },
    Conflicting {
        candidates: Vec<RelationshipCandidate>,
    },
    Invalidated {
        prior_source: RelationshipProvenance,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipLayer {
    Baseline,
    Adjudication,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipSourceSnapshot {
    pub condition_id: i64,
    pub condition_key: Option<String>,
    pub value: Option<i64>,
    pub source_participant_key: Option<String>,
    pub scope: super::ConditionScope,
    pub parameters: super::ConditionParameters,
    pub note: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectedRelationship {
    pub layer: RelationshipLayer,
    pub subject_key: String,
    pub other_key: String,
    pub property: RelationshipProperty,
    pub knowledge: RelationshipKnowledge,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipChange {
    Set(DirectedRelationship),
    Remove {
        layer: RelationshipLayer,
        subject_key: String,
        other_key: String,
        property: RelationshipProperty,
    },
}

impl EncountersRead<'_> {
    pub fn relationships(
        &self,
        encounter_key: &str,
    ) -> LocalStateResult<Vec<DirectedRelationship>> {
        let mut statement=self.connection.prepare("SELECT subject.participant_key,other.participant_key,r.property,r.codec_version,r.knowledge_json,r.layer FROM encounter_relationships r JOIN encounters e ON e.id=r.encounter_id JOIN encounter_participants subject ON subject.id=r.subject_id JOIN encounter_participants other ON other.id=r.other_id WHERE e.encounter_key=?1 ORDER BY subject.participant_key,other.participant_key,r.property,r.layer")?;
        let rows = statement.query_map([encounter_key], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        rows.map(|row| {
            let (subject_key, other_key, property, version, json, layer) = row?;
            if version != 1 {
                return Err(LocalStateError::InvalidRollRequest(
                    "relationship codec version is unsupported",
                ));
            }
            Ok(DirectedRelationship {
                layer: serde_json::from_str(&layer)?,
                subject_key,
                other_key,
                property: serde_json::from_str(&property)?,
                knowledge: serde_json::from_str(&json)?,
            })
        })
        .collect()
    }
}
pub(super) fn write(
    connection: &Connection,
    snapshot: &EncounterWithParticipants,
    change: RelationshipChange,
) -> LocalStateResult<()> {
    let (subject, other, property) = match &change {
        RelationshipChange::Set(row) => (&row.subject_key, &row.other_key, row.property),
        RelationshipChange::Remove {
            subject_key,
            other_key,
            property,
            ..
        } => (subject_key, other_key, *property),
    };
    if !snapshot
        .participants
        .iter()
        .any(|p| &p.participant_key == subject)
        || !snapshot
            .participants
            .iter()
            .any(|p| &p.participant_key == other)
    {
        return Err(LocalStateError::InvalidRollRequest(
            "relationship endpoints must belong to the event encounter",
        ));
    }
    let property_json = serde_json::to_string(&property)?;
    match change {
        RelationshipChange::Set(row) => {
            let candidates: Vec<&RelationshipValue> = match &row.knowledge {
                RelationshipKnowledge::Known { value, .. } => vec![value],
                RelationshipKnowledge::Conflicting { candidates } => candidates
                    .iter()
                    .map(|candidate| &candidate.value)
                    .collect(),
                _ => vec![],
            };
            for value in candidates {
                let valid = matches!(
                    (property, value),
                    (
                        RelationshipProperty::Detection,
                        RelationshipValue::Detection(..)
                    ) | (
                        RelationshipProperty::Awareness,
                        RelationshipValue::Awareness(..)
                    ) | (
                        RelationshipProperty::Attitude,
                        RelationshipValue::Attitude(..)
                    )
                ) || matches!(
                    (property, value),
                    (RelationshipProperty::Control, RelationshipValue::NoControl)
                ) || matches!((property,value),(RelationshipProperty::Control,RelationshipValue::Control{controller_key,..}) if controller_key==&row.other_key);
                if !valid {
                    return Err(LocalStateError::InvalidRollRequest(
                        "relationship value must match its property and directed controller",
                    ));
                }
            }
            let provenances: Vec<&RelationshipProvenance> = match &row.knowledge {
                RelationshipKnowledge::Known { provenance, .. } => vec![provenance],
                RelationshipKnowledge::Conflicting { candidates } => candidates
                    .iter()
                    .map(|candidate| &candidate.provenance)
                    .collect(),
                RelationshipKnowledge::Invalidated { prior_source } => vec![prior_source],
                RelationshipKnowledge::Unknown { .. } => vec![],
            };
            for provenance in provenances {
                if let RelationshipProvenance::Sources { condition_ids } = provenance {
                    let distinct = condition_ids
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>();
                    if condition_ids.is_empty()
                        || condition_ids.len() > 200
                        || condition_ids.iter().any(|id| *id <= 0)
                        || distinct.len() != condition_ids.len()
                    {
                        return Err(LocalStateError::InvalidRollRequest(
                            "relationship source IDs must be nonempty, positive, unique and bounded",
                        ));
                    }
                }
            }
            let layer = serde_json::to_string(&row.layer)?;
            if row.layer == RelationshipLayer::Adjudication
                && !matches!(&row.knowledge,RelationshipKnowledge::Known{provenance:RelationshipProvenance::Adjudication{sources},..} if !sources.is_empty() && sources.len()<=200)
            {
                return Err(LocalStateError::InvalidRollRequest(
                    "adjudication requires its current source snapshot",
                ));
            }
            let json = serde_json::to_string(&row.knowledge)?;
            if json.len() > 32_768 {
                return Err(LocalStateError::InvalidRollRequest(
                    "relationship knowledge is too large",
                ));
            }
            connection.execute("INSERT INTO encounter_relationships(encounter_id,subject_id,other_id,property,codec_version,knowledge_json,layer) SELECT e.id,s.id,o.id,?4,1,?5,?6 FROM encounters e JOIN encounter_participants s ON s.encounter_id=e.id JOIN encounter_participants o ON o.encounter_id=e.id WHERE e.encounter_key=?1 AND s.participant_key=?2 AND o.participant_key=?3 ON CONFLICT(subject_id,other_id,property,layer) DO UPDATE SET codec_version=excluded.codec_version,knowledge_json=excluded.knowledge_json",params![snapshot.encounter.encounter_key,row.subject_key,row.other_key,property_json,json,layer])?;
        }
        RelationshipChange::Remove {
            subject_key,
            other_key,
            layer,
            ..
        } => {
            let layer = serde_json::to_string(&layer)?;
            connection.execute("DELETE FROM encounter_relationships WHERE subject_id=(SELECT id FROM encounter_participants WHERE participant_key=?1) AND other_id=(SELECT id FROM encounter_participants WHERE participant_key=?2) AND property=?3 AND layer=?4",params![subject_key,other_key,property_json,layer])?;
        }
    }
    Ok(())
}
